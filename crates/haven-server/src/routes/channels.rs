use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use haven_core::{
    db::DbRepo,
    models::{Channel, NewChannel},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub topic: Option<String>,
    pub voice_enabled: Option<bool>,
    pub voice_user_limit: Option<i64>,
    pub voice_bitrate: Option<i64>,
    pub role_gate: Option<String>,
}

pub async fn list_channels(
    State(app_state): State<Arc<crate::app::AppState>>,
) -> Result<Json<Vec<Channel>>, (StatusCode, Json<serde_json::Value>)> {
    let conn = app_state.pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let channels = DbRepo::list_channels(&conn).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(channels))
}

pub async fn get_channel(
    State(app_state): State<Arc<crate::app::AppState>>,
    Path(code): Path<String>,
) -> Result<Json<Channel>, (StatusCode, Json<serde_json::Value>)> {
    let conn = app_state.pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let channel = DbRepo::get_channel_by_code(&conn, &code)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Channel not found" })),
            )
        })?;

    Ok(Json(channel))
}

pub async fn create_channel(
    State(app_state): State<Arc<crate::app::AppState>>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<Json<Channel>, (StatusCode, Json<serde_json::Value>)> {
    let conn = app_state.pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM channels", [], |row| row.get(0))
        .unwrap_or(0);

    let code = format!(
        "{}-{}",
        payload.name.to_lowercase().replace(' ', "-"),
        uuid::Uuid::new_v4().to_string().chars().take(8).collect::<String>()
    );

    let new_ch = NewChannel {
        code,
        name: payload.name,
        topic: payload.topic,
        position: count,
        is_dm: false,
        voice_enabled: payload.voice_enabled.unwrap_or(false),
        voice_user_limit: payload.voice_user_limit.unwrap_or(0),
        voice_bitrate: payload.voice_bitrate.unwrap_or(64000),
        role_gate: payload.role_gate,
    };

    let channel = DbRepo::create_channel(&conn, &new_ch).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(channel))
}
