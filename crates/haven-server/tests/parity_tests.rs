use dashmap::DashMap;
use haven_core::{
    db::{create_in_memory_pool, DbRepo},
    models::{NewUser, PublicConfig, Webhook},
    sanitizer::{blocked_word_in, parse_blocked_names, parse_slash},
};
use haven_server::app::{create_router, AppState};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[test]
fn test_slash_parse_parity() {
    let fixture_path = "../../tests/fixtures/slash_parse_samples.json";
    if let Ok(content) = std::fs::read_to_string(fixture_path) {
        let samples: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        for s in samples {
            let input = s["input"].as_str().unwrap();
            let parsed = parse_slash(input);
            if s["expected"].is_null() {
                assert!(parsed.is_none(), "Expected null for input: {}", input);
            } else {
                let p = parsed.expect(&format!("Expected command for input: {}", input));
                assert_eq!(p.cmd, s["expected"]["command"].as_str().unwrap());
                assert_eq!(p.arg, s["expected"]["args"].as_str().unwrap());
            }
        }
    }
}

#[test]
fn test_blocked_names_parity() {
    let fixture_path = "../../tests/fixtures/blocked_names_samples.json";
    if let Ok(content) = std::fs::read_to_string(fixture_path) {
        let samples: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        for s in samples {
            let input = s["input"].as_str().unwrap();
            let blocked_list: Vec<String> = s["blocked_list"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            let is_blocked = s["is_blocked"].as_bool().unwrap();

            let matched = blocked_word_in(input, &blocked_list);
            assert_eq!(
                matched.is_some(),
                is_blocked,
                "Mismatch for name: {} (expected blocked: {})",
                input,
                is_blocked
            );
        }
    }
}

#[tokio::test]
async fn test_database_and_bot_voice_parity() {
    let pool = create_in_memory_pool().unwrap();
    let conn = pool.get().unwrap();

    // 1. Create creator user
    let user = DbRepo::create_user(
        &conn,
        &NewUser {
            username: "admin".into(),
            password_hash: "hash".into(),
            display_name: Some("Administrator".into()),
            is_admin: true,
            is_guest: false,
        },
    )
    .unwrap();

    // 2. Create voice channel
    let ch = DbRepo::create_channel(
        &conn,
        &haven_core::models::NewChannel {
            code: "yapping".into(),
            name: "Yapping Voice".into(),
            topic: None,
            position: 1,
            is_dm: false,
            voice_enabled: true,
            voice_user_limit: 10,
            voice_bitrate: 64000,
            role_gate: None,
        },
    )
    .unwrap();

    assert_eq!(ch.code, "yapping");
    assert!(ch.voice_enabled);

    // 3. Create Webhook bot
    conn.execute(
        "INSERT INTO webhooks (name, token, channel_id, created_by, is_bot, can_use_voice)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            "MusicBot",
            "c".repeat(64),
            ch.id,
            user.id,
            1,
            1
        ],
    )
    .unwrap();

    let wh = DbRepo::get_webhook_by_token(&conn, &"c".repeat(64))
        .unwrap()
        .unwrap();
    assert_eq!(wh.name, "MusicBot");
    assert!(wh.can_use_voice);

    // 4. Test accessible voice channels (matching Haven test botVoiceApi.integration.test.js)
    let accessible = DbRepo::get_accessible_voice_channels(&conn, &wh).unwrap();
    assert_eq!(accessible.len(), 1);
    assert_eq!(accessible[0].code, "yapping");
}
