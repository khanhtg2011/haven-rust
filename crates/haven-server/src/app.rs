use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use haven_core::{db::DbPool, models::PublicConfig};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

pub struct AppState {
    pub pool: DbPool,
    pub jwt_secret: Vec<u8>,
    pub public_config: Arc<RwLock<PublicConfig>>,
    pub voice_rooms: DashMap<String, HashSet<String>>,
    pub tx_broadcast: broadcast::Sender<String>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // === System & Info Routes ===
        .route("/api/version", get(crate::routes::info::get_version))
        .route(
            "/api/public-config",
            get(crate::routes::info::get_public_config),
        )
        .route(
            "/api/ice-servers",
            get(crate::routes::info::get_ice_servers),
        )
        .route("/api/plugins", get(crate::routes::info::get_plugins))
        .route("/api/themes", get(crate::routes::info::get_themes))
        // === Auth Routes ===
        .route("/api/auth/register", post(crate::routes::auth::register))
        .route("/api/auth/login", post(crate::routes::auth::login))
        // === Channels & Messages ===
        .route(
            "/api/channels",
            get(crate::routes::channels::list_channels).post(crate::routes::channels::create_channel),
        )
        .route(
            "/api/channels/:code",
            get(crate::routes::channels::get_channel),
        )
        .route(
            "/api/channels/:code/messages",
            get(crate::routes::messages::list_channel_messages),
        )
        // === Bot API ===
        .route(
            "/api/bot/voice-channels",
            get(crate::routes::bot::get_bot_voice_channels),
        )
        .route(
            "/api/bot/voice-play",
            post(crate::routes::bot::post_bot_voice_play),
        )
        // === WebSocket / Socket.IO ===
        .route("/socket.io/", get(crate::ws::handler::ws_upgrade_handler))
        .route("/ws", get(crate::ws::handler::ws_upgrade_handler))
        .layer(cors)
        .with_state(state)
}
