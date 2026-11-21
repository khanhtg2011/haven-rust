use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use haven_core::db::DbRepo;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::framing::{encode_connect_ack, encode_event, encode_open, parse_packet, SocketIoPacket};
use crate::app::AppState;

#[derive(Debug, Deserialize, Clone)]
pub struct WsQuery {
    pub token: Option<String>,
    #[serde(rename = "EIO")]
    pub eio: Option<String>,
    pub transport: Option<String>,
    pub sid: Option<String>,
}

pub async fn socket_io_handler(
    ws_res: Result<WebSocketUpgrade, axum::extract::ws::rejection::WebSocketUpgradeRejection>,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
    method: Method,
) -> Response {
    if let Ok(ws) = ws_res {
        return ws
            .on_upgrade(move |socket| handle_socket(socket, query, state))
            .into_response();
    }

    // Engine.IO v4 Polling handshake & fallback
    let sid = query
        .sid
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    if method == Method::GET {
        if query.sid.is_none() {
            // Initial handshake
            let body = format!(
                "0{{\"sid\":\"{}\",\"upgrades\":[\"websocket\"],\"pingInterval\":25000,\"pingTimeout\":20000,\"maxPayload\":1000000}}",
                sid
            );
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
                body,
            )
                .into_response();
        } else {
            // Long-poll response: connect packet
            let body = format!("40{{\"sid\":\"{}\"}}", sid);
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
                body,
            )
                .into_response();
        }
    } else if method == Method::POST {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=UTF-8")],
            "ok",
        )
            .into_response();
    }

    StatusCode::BAD_REQUEST.into_response()
}

pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, query, state))
}

async fn handle_socket(socket: WebSocket, query: WsQuery, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let sid = query
        .sid
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // 1. Send Engine.IO open packet
    let open_pkt = encode_open(&sid, 25000, 20000);
    if sender.send(Message::Text(open_pkt.into())).await.is_err() {
        return;
    }

    // 2. Send Socket.IO connect ack
    let conn_ack = encode_connect_ack(&sid);
    if sender.send(Message::Text(conn_ack.into())).await.is_err() {
        return;
    }

    // Direct channel for messages to this specific client
    let (tx_direct, mut rx_direct) = mpsc::channel::<String>(128);

    // Global broadcast subscriber
    let mut rx_broadcast = state.tx_broadcast.subscribe();

    // Writer task: multiplex direct messages and global broadcast messages
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = rx_direct.recv() => {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                Ok(msg) = rx_broadcast.recv() => {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
                else => break,
            }
        }
    });

    // Check auth from query parameter
    let mut authed_user_id: Option<i64> = None;
    let mut authed_username = "Anonymous".to_string();
    let mut authed_is_admin = false;

    if let Some(ref t) = query.token {
        if let Ok(claims) = haven_core::auth::verify_token(t, &state.jwt_secret) {
            authed_user_id = Some(claims.sub);
            authed_username = claims.username.clone();
            authed_is_admin = claims.is_admin;
        }
    }

    // If authenticated, send session-info
    if let Some(uid) = authed_user_id {
        let session_info = json!({
            "id": uid,
            "username": authed_username,
            "displayName": authed_username,
            "isAdmin": authed_is_admin,
            "roles": [],
            "effectiveLevel": if authed_is_admin { 100 } else { 10 },
            "permissions": ["*"],
            "globalPermissions": ["*"],
            "status": "online"
        });
        let _ = tx_direct.send(encode_event("session-info", &session_info)).await;
    }

    let state_clone = state.clone();
    let sid_clone = sid.clone();
    let tx_direct_clone = tx_direct.clone();

    // Reader task
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Some(packet) = parse_packet(&text) {
                    match packet {
                        SocketIoPacket::Ping => {
                            let _ = tx_direct_clone.send("3".to_string()).await;
                        }
                        SocketIoPacket::PingProbe => {
                            let _ = tx_direct_clone.send("3probe".to_string()).await;
                        }
                        SocketIoPacket::Upgrade => {
                            tracing::debug!(sid = %sid_clone, "Engine.IO upgrade completed");
                        }
                        SocketIoPacket::ConnectAck { .. } => {
                            // Client sent auth packet
                            let _ = tx_direct_clone.send(encode_connect_ack(&sid_clone)).await;
                        }
                        SocketIoPacket::Event { name, payload, .. } => {
                            handle_event(
                                &name,
                                &payload,
                                &state_clone,
                                &sid_clone,
                                authed_user_id,
                                &authed_username,
                                &tx_direct_clone,
                            )
                            .await;
                        }
                        _ => {}
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
    cleanup_session(&sid, &state).await;
}

async fn handle_event(
    event_name: &str,
    payload: &serde_json::Value,
    state: &Arc<AppState>,
    sid: &str,
    user_id: Option<i64>,
    username: &str,
    tx_direct: &mpsc::Sender<String>,
) {
    match event_name {
        // === App Lifecycle & Metadata ===
        "get-channels" => {
            if let Ok(conn) = state.pool.get() {
                if let Ok(channels) = DbRepo::list_channels(&conn) {
                    let evt = encode_event("channels-list", &serde_json::to_value(&channels).unwrap_or_default());
                    let _ = tx_direct.send(evt).await;
                }
            }
        }

        "get-server-settings" => {
            let settings = json!({
                "serverName": "Haven",
                "allowRegistration": true
            });
            let _ = tx_direct.send(encode_event("server-settings", &settings)).await;
        }

        "get-preferences" => {
            let _ = tx_direct.send(encode_event("preferences", &json!({}))).await;
        }

        "get-recovery-notice-state" => {
            let _ = tx_direct.send(encode_event("recovery-notice-state", &json!({ "showNotice": false }))).await;
        }

        "visibility-change" | "reply-ping" | "set-status" => {
            // Heartbeats / client status updates
        }

        // === Voice Mesh WebRTC Signaling ===
        "voice-join" | "voice-rejoin" => {
            let channel_code = payload["channel_code"]
                .as_str()
                .or_else(|| payload["code"].as_str())
                .unwrap_or("default");
            tracing::info!(sid = %sid, channel = %channel_code, "User joined voice channel");

            state
                .voice_rooms
                .entry(channel_code.to_string())
                .or_default()
                .insert(sid.to_string());

            let evt = encode_event(
                "voice-user-joined",
                &json!({
                    "sid": sid,
                    "channel_code": channel_code,
                    "userId": user_id,
                    "username": username
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }

        "voice-leave" => {
            let channel_code = payload["channel_code"]
                .as_str()
                .or_else(|| payload["code"].as_str())
                .unwrap_or("default");
            tracing::info!(sid = %sid, channel = %channel_code, "User left voice channel");

            if let Some(mut members) = state.voice_rooms.get_mut(channel_code) {
                members.remove(sid);
            }

            let evt = encode_event(
                "voice-user-left",
                &json!({
                    "sid": sid,
                    "channel_code": channel_code,
                    "userId": user_id
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }

        "voice-offer" | "voice-answer" | "voice-ice-candidate" | "voice-speaking"
        | "voice-mute-state" | "voice-deafen-state" => {
            let evt = encode_event(
                event_name,
                &json!({
                    "from_sid": sid,
                    "data": payload
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }

        // === Chat Events ===
        "send-message" => {
            let channel_code = payload["channel_code"]
                .as_str()
                .or_else(|| payload["code"].as_str())
                .unwrap_or("general");
            let content = payload["content"].as_str().unwrap_or("");
            tracing::info!(sid = %sid, channel = %channel_code, "New chat message received");

            if let Ok(conn) = state.pool.get() {
                if let Ok(Some(ch)) = DbRepo::get_channel_by_code(&conn, channel_code) {
                    let uid = user_id.unwrap_or(1);
                    if let Ok(msg) = DbRepo::create_message(&conn, ch.id, uid, username, content, None) {
                        let evt = encode_event(
                            "new-message",
                            &serde_json::to_value(&msg).unwrap_or_default(),
                        );
                        let _ = state.tx_broadcast.send(evt);
                    }
                }
            }
        }

        "typing" => {
            let channel_code = payload["channel_code"].as_str().unwrap_or("general");
            let evt = encode_event(
                "user-typing",
                &json!({
                    "channel_code": channel_code,
                    "sid": sid,
                    "username": username
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }

        _ => {
            tracing::debug!(event = %event_name, "Unhandled socket event");
        }
    }
}

async fn cleanup_session(sid: &str, state: &Arc<AppState>) {
    for mut room in state.voice_rooms.iter_mut() {
        if room.value_mut().remove(sid) {
            let evt = encode_event(
                "voice-user-left",
                &json!({
                    "sid": sid,
                    "channel_code": room.key()
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }
    }
}
