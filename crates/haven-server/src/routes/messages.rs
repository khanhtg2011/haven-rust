use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use haven_core::{
    db::DbRepo,
    models::Message,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
}

pub async fn list_channel_messages(
    State(app_state): State<Arc<crate::app::AppState>>,
    Path(code): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<Vec<Message>>, (StatusCode, Json<serde_json::Value>)> {
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

    let limit = query.limit.unwrap_or(50).min(100);
    let messages = DbRepo::list_messages(&conn, channel.id, limit).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(messages))
}
