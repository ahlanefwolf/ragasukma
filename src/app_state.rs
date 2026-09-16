use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::ui::pose_system::{AssistantVisualState, PosesConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppStatePayload {
    pub state: String,
    pub is_running: bool,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub mic_level: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_session_token: Option<String>,
}

use std::sync::atomic::{AtomicBool, Ordering};

pub struct ActiveWebSession {
    pub token: String,
    pub active_tab: String,
    pub last_seen: Instant,
}

pub static ACTIVE_WEB_SESSION: Mutex<Option<ActiveWebSession>> = Mutex::new(None);

pub struct ServerSharedState {
    pub current_state: AssistantVisualState,
    pub current_emotion: String,
    pub is_running: bool,
    pub is_paused: bool,
    pub is_shutdown: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub mic_level: u8,
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
static AI_SPEAKING_FLAG: AtomicBool = AtomicBool::new(false);
static INTERRUPT_FLAG: AtomicBool = AtomicBool::new(false);

pub fn set_ai_speaking(speaking: bool) {
    AI_SPEAKING_FLAG.store(speaking, Ordering::SeqCst);
}

pub fn is_ai_speaking() -> bool {
    AI_SPEAKING_FLAG.load(Ordering::SeqCst)
}

pub fn trigger_interruption() {
    INTERRUPT_FLAG.store(true, Ordering::SeqCst);
}

pub fn is_interruption_requested() -> bool {
    INTERRUPT_FLAG.load(Ordering::SeqCst)
}

pub fn clear_interruption() {
    INTERRUPT_FLAG.store(false, Ordering::SeqCst);
}

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

pub fn update_mic_level(level: u8) {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                s.mic_level = level;
            }
        }
    }
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

pub fn trigger_app_shutdown() {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                s.is_shutdown = true;
                s.is_running = false;
                s.state_version = s.state_version.wrapping_add(1);
            }
        }
    }
}

pub fn is_shutdown() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.is_shutdown;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Client Action Trigger Queue (Music actions, UI commands, dll)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientAction {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

pub static PENDING_CLIENT_ACTIONS: Mutex<std::collections::VecDeque<ClientAction>> =
    Mutex::new(std::collections::VecDeque::new());

pub fn notify_state_changed() {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                s.state_version = s.state_version.wrapping_add(1);
            }
        }
    }
}

pub fn push_client_action(action: &str, payload: Option<&str>) {
    if let Ok(mut q) = PENDING_CLIENT_ACTIONS.lock() {
        q.push_back(ClientAction {
            action: action.to_string(),
            payload: payload.map(|s| s.to_string()),
        });
    }
    notify_state_changed();
}

pub fn pop_client_actions() -> Vec<ClientAction> {
    if let Ok(mut q) = PENDING_CLIENT_ACTIONS.lock() {
        q.drain(..).collect()
    } else {
        Vec::new()
    }
}
