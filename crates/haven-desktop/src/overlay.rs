use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlaySpeaker {
    pub user_id: i64,
    pub username: String,
    pub avatar: Option<String>,
    pub is_speaking: bool,
    pub is_muted: bool,
    pub is_deafened: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub enabled: bool,
    pub position: String, // "top-left", "top-right", "bottom-left", "bottom-right"
    pub opacity: f32,
    pub show_names: bool,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            position: "top-left".into(),
            opacity: 0.85,
            show_names: true,
        }
    }
}

pub struct VoiceOverlayManager {
    pub config: OverlayConfig,
    pub speakers: DashMap<i64, OverlaySpeaker>,
}

impl VoiceOverlayManager {
    pub fn new(config: OverlayConfig) -> Self {
        Self {
            config,
            speakers: DashMap::new(),
        }
    }

    pub fn update_speaking(&self, user_id: i64, is_speaking: bool) {
        if let Some(mut speaker) = self.speakers.get_mut(&user_id) {
            speaker.is_speaking = is_speaking;
        }
    }

    pub fn set_speaker(&self, speaker: OverlaySpeaker) {
        self.speakers.insert(speaker.user_id, speaker);
    }

    pub fn remove_speaker(&self, user_id: i64) {
        self.speakers.remove(&user_id);
    }

    pub fn list_active_speakers(&self) -> Vec<OverlaySpeaker> {
        self.speakers
            .iter()
            .map(|r| r.value().clone())
            .collect()
    }
}

impl Default for VoiceOverlayManager {
    fn default() -> Self {
        Self::new(OverlayConfig::default())
    }
}
