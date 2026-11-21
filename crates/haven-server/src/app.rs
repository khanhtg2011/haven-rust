use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use haven_core::{db::DbPool, models::PublicConfig};
use std::{collections::HashSet, path::PathBuf, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

pub struct AppState {
    pub pool: DbPool,
    pub jwt_secret: Vec<u8>,
    pub public_config: Arc<RwLock<PublicConfig>>,
    pub voice_rooms: DashMap<String, HashSet<String>>,
    pub tx_broadcast: broadcast::Sender<String>,
    pub public_dir: String,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_path = PathBuf::from(&state.public_dir);
    let index_file = public_path.join("index.html");
    let app_file = public_path.join("app.html");
    let socket_io_file = public_path.join("socket.io").join("socket.io.js");
    let flappy_file = public_path.join("games").join("flappy.html");
    let theme_preview_file = public_path.join("theme-preview.html");

    let serve_dir = ServeDir::new(&public_path);

    Router::new()
        // === Web UI Routes ===
        .route_service("/", ServeFile::new(&index_file))
        .route_service("/app", ServeFile::new(&app_file))
        .route_service("/socket.io/socket.io.js", ServeFile::new(&socket_io_file))
        .route_service("/games/flappy", ServeFile::new(&flappy_file))
        .route_service("/theme-preview", ServeFile::new(&theme_preview_file))
        .route("/invite/{vanityCode}", get(crate::routes::info::invite_redirect))
        // === System & Info Routes ===
        .route("/api/health", get(crate::routes::info::get_health))
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
        .route("/api/bot-commands", get(crate::routes::info::get_bot_commands))
        .route(
            "/api/auth/registration-info",
            get(crate::routes::info::get_registration_info),
        )
        .route(
            "/api/auth/guest-info",
            get(crate::routes::info::get_guest_info),
        )
        // === Auth Routes ===
        .route("/api/auth/register", post(crate::routes::auth::register))
        .route("/api/auth/login", post(crate::routes::auth::login))
        // === Channels & Messages ===
        .route(
            "/api/channels",
            get(crate::routes::channels::list_channels).post(crate::routes::channels::create_channel),
        )
        .route(
            "/api/channels/{code}",
            get(crate::routes::channels::get_channel),
        )
        .route(
            "/api/channels/{code}/messages",
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
        .route(
            "/socket.io/",
            get(crate::ws::handler::socket_io_handler).post(crate::ws::handler::socket_io_handler),
        )
        .route("/ws", get(crate::ws::handler::ws_upgrade_handler))
        // === Static Assets (CSS, JS, Fonts, Emoji, Uploads...) ===
        .fallback_service(serve_dir)
        .layer(cors)
        .with_state(state)
}
