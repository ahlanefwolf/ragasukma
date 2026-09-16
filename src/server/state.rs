use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::native::pose_system::{AssistantVisualState, PosesConfig};
use crate::engine::ytdlp::YtTrackInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppStatePayload {
    pub state: String,
    pub is_running: bool,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub active_session_token: Option<String>,
}

pub struct ActiveWebSession {
    pub token: String,
    pub last_seen: Instant,
}

pub static ACTIVE_WEB_SESSION: Mutex<Option<ActiveWebSession>> = Mutex::new(None);

pub struct ServerSharedState {
    pub current_state: AssistantVisualState,
    pub current_emotion: String,
    pub is_running: bool,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub poses_updated: bool,
    pub config_updated: bool,
    pub vrm_updated: bool,
    pub tuning_mode: bool,
    pub live_tuning_bones: HashMap<String, [f32; 3]>,
    /// Live preview pose in-memory tanpa overwrite poses.json di disk
    pub preview_poses: Option<PosesConfig>,
    pub state_version: u64,
}

pub static GLOBAL_SHARED_STATE: Mutex<Option<Arc<Mutex<ServerSharedState>>>> = Mutex::new(None);

pub fn register_shared_state(shared: Arc<Mutex<ServerSharedState>>) {
    if let Ok(mut lock) = GLOBAL_SHARED_STATE.lock() {
        *lock = Some(shared);
    }
}

pub fn update_session_running(running: bool) {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                if s.is_running != running {
                    s.is_running = running;
                    if !running {
                        s.current_state = AssistantVisualState::Idle;
                    }
                    s.state_version = s.state_version.wrapping_add(1);
                }
            }
        }
    }
}

pub fn update_visual_state(state: AssistantVisualState) {
    update_visual_state_with_emotion(state, "neutral");
}

pub fn update_visual_state_with_emotion(state: AssistantVisualState, emotion: &str) {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                let changed = s.current_state != state || s.current_emotion != emotion;
                if changed {
                    s.current_state = state;
                    s.current_emotion = emotion.to_string();
                    s.state_version = s.state_version.wrapping_add(1);
                }
            }
        }
    }
}

pub fn is_session_running() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.is_running;
            }
        }
    }
    false
}

pub fn is_tuning_mode() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.tuning_mode;
            }
        }
    }
    false
}

pub fn is_voice_muted() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.voice_muted;
            }
        }
    }
    false
}

pub fn is_mic_muted() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.mic_muted;
            }
        }
    }
    false
}

pub fn is_paused() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.is_paused;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Music Player State
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MusicPlayerState {
    pub current_track: Option<YtTrackInfo>,
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: f32,
}

impl Default for MusicPlayerState {
    fn default() -> Self {
        Self {
            current_track: None,
            is_playing: false,
            is_paused: false,
            volume: 1.0,
        }
    }
}

pub static MUSIC_STATE: Mutex<Option<MusicPlayerState>> = Mutex::new(None);

pub fn get_music_state() -> MusicPlayerState {
    if let Ok(lock) = MUSIC_STATE.lock() {
        lock.clone().unwrap_or_default()
    } else {
        MusicPlayerState::default()
    }
}

/// Play file — simpan state pemutaran untuk web streaming player
pub fn music_play_file(_path: &std::path::Path, track: YtTrackInfo) -> anyhow::Result<()> {
    if let Ok(mut lock) = MUSIC_STATE.lock() {
        *lock = Some(MusicPlayerState {
            current_track: Some(track),
            is_playing: true,
            is_paused: false,
            volume: 1.0,
        });
    }
    Ok(())
}

pub fn music_pause() {
    if let Ok(mut lock) = MUSIC_STATE.lock() {
        if let Some(s) = lock.as_mut() {
            s.is_playing = false;
            s.is_paused = true;
        }
    }
}

pub fn music_resume() {
    if let Ok(mut lock) = MUSIC_STATE.lock() {
        if let Some(s) = lock.as_mut() {
            s.is_playing = true;
            s.is_paused = false;
        }
    }
}

pub fn music_stop() {
    if let Ok(mut lock) = MUSIC_STATE.lock() {
        *lock = Some(MusicPlayerState::default());
    }
}
