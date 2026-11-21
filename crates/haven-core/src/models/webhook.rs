use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub channel_id: i64,
    pub created_by: Option<i64>,
    pub is_bot: bool,
    pub can_use_voice: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotAudioPlayRequest {
    pub channel_code: String,
    pub url: String,
    pub volume: Option<f32>,
}
