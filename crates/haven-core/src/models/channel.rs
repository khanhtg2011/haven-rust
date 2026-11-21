use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub topic: Option<String>,
    pub position: i64,
    pub is_dm: bool,
    pub is_temp: bool,
    pub voice_enabled: bool,
    pub voice_user_limit: i64,
    pub voice_bitrate: i64,
    pub role_gate: Option<String>,
    pub channel_type: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewChannel {
    pub code: String,
    pub name: String,
    pub topic: Option<String>,
    pub position: i64,
    pub is_dm: bool,
    pub voice_enabled: bool,
    pub voice_user_limit: i64,
    pub voice_bitrate: i64,
    pub role_gate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceChannelAccessible {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub voice_enabled: bool,
    pub voice_user_limit: i64,
    pub voice_bitrate: i64,
}
