use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use haven_core::{
    auth::{create_token, hash_password, verify_password},
    db::DbRepo,
    models::{NewUser, UserPublicProfile},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserPublicProfile,
}

pub async fn register(
    State(app_state): State<Arc<crate::app::AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let username = payload.username.trim();
    if username.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Username and password are required" })),
        ));
    }

    let conn = app_state.pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    // Check if any users exist; first user becomes admin
    let user_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .unwrap_or(0);
    let is_admin = user_count == 0;

    let hash = hash_password(&payload.password).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let new_user = NewUser {
        username: username.to_string(),
        password_hash: hash,
        display_name: payload.display_name,
        is_admin,
        is_guest: false,
    };

    let user = DbRepo::create_user(&conn, &new_user).map_err(|_e| {
        (
            StatusCode::CONFLICT,
            Json(json!({ "error": "Username already taken or invalid" })),
        )
    })?;

    let token = create_token(
        user.id,
        &user.username,
        user.is_admin,
        user.is_guest,
        &app_state.jwt_secret,
        7 * 24 * 3600,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn login(
    State(app_state): State<Arc<crate::app::AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let conn = app_state.pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    let user = DbRepo::get_user_by_username(&conn, &payload.username)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid username or password" })),
            )
        })?;

    if user.is_banned {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "Account is banned" })),
        ));
    }

    if !verify_password(&payload.password, &user.password_hash) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid username or password" })),
        ));
    }

    let token = create_token(
        user.id,
        &user.username,
        user.is_admin,
        user.is_guest,
        &app_state.jwt_secret,
        7 * 24 * 3600,
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}
