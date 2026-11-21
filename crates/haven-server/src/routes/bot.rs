use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use haven_core::{
    db::DbRepo,
    models::{BotAudioPlayRequest, VoiceChannelAccessible},
};
use serde_json::json;
use std::sync::Arc;

fn extract_token(headers: &HeaderMap) -> Option<&str> {
    let auth = headers.get("Authorization")?.to_str().ok()?;
    if let Some(token) = auth.strip_prefix("Bearer ") {
        Some(token.trim())
    } else {
        None
    }
}

pub async fn get_bot_voice_channels(
    State(app_state): State<Arc<crate::app::AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<VoiceChannelAccessible>>, (StatusCode, Json<serde_json::Value>)> {
    let token = extract_token(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Missing or invalid Authorization header" })),
        )
    })?;

    let conn = app_state.pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let webhook = DbRepo::get_webhook_by_token(&conn, token)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid webhook token" })),
            )
        })?;

    if !webhook.can_use_voice {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Webhook does not have voice permissions" })),
        ));
    }

    let channels = DbRepo::get_accessible_voice_channels(&conn, &webhook).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(channels))
}

pub async fn post_bot_voice_play(
    State(app_state): State<Arc<crate::app::AppState>>,
    headers: HeaderMap,
    Json(payload): Json<BotAudioPlayRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let token = extract_token(&headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Missing or invalid Authorization header" })),
        )
    })?;

    let conn = app_state.pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let webhook = DbRepo::get_webhook_by_token(&conn, token)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid webhook token" })),
            )
        })?;

    if !webhook.can_use_voice {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Webhook does not have voice permissions" })),
        ));
    }

    tracing::info!(
        bot = %webhook.name,
        channel = %payload.channel_code,
        url = %payload.url,
        "Bot voice audio playback queued"
    );

    Ok(Json(json!({
        "status": "ok",
        "message": "Playback started",
        "channel": payload.channel_code
    })))
}
