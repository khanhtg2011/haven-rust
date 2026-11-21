use axum::{
    extract::State,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn get_version() -> Json<serde_json::Value> {
    Json(json!({
        "version": "4.29.0"
    }))
}

pub async fn get_public_config(
    State(app_state): State<Arc<crate::app::AppState>>,
) -> Json<haven_core::models::PublicConfig> {
    let cfg = app_state.public_config.read().await;
    Json((*cfg).clone())
}

pub async fn get_ice_servers() -> Json<serde_json::Value> {
    // Returns default STUN/TURN server list matching Haven
    Json(json!({
        "iceServers": [
            { "urls": ["stun:stun.l.google.com:19302", "stun:stun1.l.google.com:19302"] }
        ]
    }))
}

pub async fn get_plugins() -> Json<serde_json::Value> {
    // Returns installed plugins list matching Haven format
    Json(json!([
        {
            "file": "AutoModQuarantine.plugin.js",
            "name": "AutoMod Quarantine (Experimental)",
            "description": "Quarantines links, scans them via AI, releases safe links, blocks threats.",
            "author": "Amnibro",
            "version": "1.4.0"
        },
        {
            "file": "MessageTimestamps.plugin.js",
            "name": "Message Timestamps",
            "description": "Adds relative timestamps next to every message",
            "author": "Haven",
            "version": "1.0.0"
        }
    ]))
}

pub async fn get_themes() -> Json<serde_json::Value> {
    Json(json!([
        {
            "file": "amni-scient.theme.css",
            "name": "Amni-Scient",
            "description": "Gold on ink, Archivo, same palette as amni-scient.com.",
            "author": "Amnibro",
            "version": "1.0",
            "compatibility": "compatible",
            "compatible": true,
            "palette": true,
            "swatch": { "bg": "#08090B", "panel": "#111418", "accent": "#C89B4E", "text": "#EDEFF2" }
        },
        {
            "file": "braid.theme.css",
            "name": "Braid",
            "description": "Mint-on-slate with soft rounded corners and a calm, decluttered look.",
            "author": "Amnibro",
            "version": "1.0",
            "compatibility": "legacy",
            "compatible": true,
            "palette": true,
            "swatch": { "bg": "#0b0d12", "panel": "#161a22", "accent": "#00ff9d", "text": "#eef1f6" }
        }
    ]))
}
