use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use super::framing::{encode_connect_ack, encode_event, encode_open, parse_packet, SocketIoPacket};
use crate::app::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
    #[serde(rename = "EIO")]
    pub eio: Option<String>,
    pub transport: Option<String>,
}

pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, query, state))
}

async fn handle_socket(socket: WebSocket, _query: WsQuery, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let sid = uuid::Uuid::new_v4().to_string();

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

    // Subscribe to global broadcast channel
    let mut rx = state.tx_broadcast.subscribe();

    // Spawn a writer task to forward broadcast messages
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let sid_clone = sid.clone();

    // Reader task
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                if let Some(packet) = parse_packet(&text) {
                    match packet {
                        SocketIoPacket::Ping => {
                            let _ = state_clone
                                .tx_broadcast
                                .send("3".to_string());
                        }
                        SocketIoPacket::Event { name, payload, .. } => {
                            handle_event(&name, &payload, &state_clone, &sid_clone).await;
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
) {
    match event_name {
        // === Voice Mesh WebRTC Signaling ===
        "voice-join" => {
            let channel_code = payload["channel_code"].as_str().unwrap_or("default");
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
                    "channel_code": channel_code
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }

        "voice-leave" => {
            let channel_code = payload["channel_code"].as_str().unwrap_or("default");
            tracing::info!(sid = %sid, channel = %channel_code, "User left voice channel");

            if let Some(mut members) = state.voice_rooms.get_mut(channel_code) {
                members.remove(sid);
            }

            let evt = encode_event(
                "voice-user-left",
                &json!({
                    "sid": sid,
                    "channel_code": channel_code
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }

        "voice-offer" | "voice-answer" | "voice-ice-candidate" | "voice-speaking" => {
            // Forward WebRTC signaling to mesh peers
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
            let channel_code = payload["channel_code"].as_str().unwrap_or("general");
            let content = payload["content"].as_str().unwrap_or("");
            tracing::info!(sid = %sid, channel = %channel_code, "New chat message received");

            let evt = encode_event(
                "new-message",
                &json!({
                    "channel_code": channel_code,
                    "content": content,
                    "author": "User",
                    "created_at": chrono::Utc::now().to_rfc3339()
                }),
            );
            let _ = state.tx_broadcast.send(evt);
        }

        "typing" => {
            let channel_code = payload["channel_code"].as_str().unwrap_or("general");
            let evt = encode_event(
                "user-typing",
                &json!({
                    "channel_code": channel_code,
                    "sid": sid
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
