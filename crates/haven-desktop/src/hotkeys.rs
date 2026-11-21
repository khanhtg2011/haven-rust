use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShortcutAction {
    PushToTalk,
    PushToMute,
    ToggleDeafen,
    ToggleMute,
}

pub struct HotkeyManager {
    is_ptt_active: Arc<AtomicBool>,
    is_muted: Arc<AtomicBool>,
}

impl HotkeyManager {
    pub fn new() -> Self {
        Self {
            is_ptt_active: Arc::new(AtomicBool::new(false)),
            is_muted: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_ptt_pressed(&self, pressed: bool) {
        self.is_ptt_active.store(pressed, Ordering::SeqCst);
        tracing::debug!(pressed, "Push-to-talk state updated");
    }

    pub fn is_transmitting(&self) -> bool {
        self.is_ptt_active.load(Ordering::SeqCst) && !self.is_muted.load(Ordering::SeqCst)
    }

    pub fn toggle_mute(&self) -> bool {
        let current = self.is_muted.load(Ordering::SeqCst);
        let new_val = !current;
        self.is_muted.store(new_val, Ordering::SeqCst);
        tracing::info!(muted = new_val, "Toggled microphone mute");
        new_val
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_ptt_transmission() {
        let manager = HotkeyManager::new();
        assert!(!manager.is_transmitting());

        manager.set_ptt_pressed(true);
        assert!(manager.is_transmitting());

        manager.toggle_mute(); // now muted
        assert!(!manager.is_transmitting());

        manager.toggle_mute(); // unmuted
        assert!(manager.is_transmitting());
    }
}
