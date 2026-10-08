use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::ui::pose_system::{AssistantVisualState, PosesConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppStatePayload {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,
    pub is_running: bool,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
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
    pub poses_updated: bool,
    pub config_updated: bool,
    pub vrm_updated: bool,
    pub tuning_mode: bool,
    pub live_tuning_bones: HashMap<String, [f32; 3]>,
    /// Live in-memory pose preview without overwriting poses.json on disk
    pub preview_poses: Option<PosesConfig>,
    pub state_version: u64,
}

pub static GLOBAL_SHARED_STATE: Mutex<Option<Arc<Mutex<ServerSharedState>>>> = Mutex::new(None);
static AI_SPEAKING_FLAG: AtomicBool = AtomicBool::new(false);
static INTERRUPT_FLAG: AtomicBool = AtomicBool::new(false);
static LAST_REMOTE_ACTIVE: Mutex<Option<Instant>> = Mutex::new(None);

pub fn touch_remote_active() {
    if let Ok(mut lock) = LAST_REMOTE_ACTIVE.lock() {
        *lock = Some(Instant::now());
    }
}

pub fn is_remote_client_active() -> bool {
    if let Ok(lock) = LAST_REMOTE_ACTIVE.lock() {
        if let Some(t) = *lock {
            return t.elapsed().as_secs() < 30;
        }
    }
    false
}

static SPEAKING_EPOCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static LAST_SPEECH_ACTIVITY: Mutex<Option<Instant>> = Mutex::new(None);

pub fn touch_speech_activity() {
    if let Ok(mut lock) = LAST_SPEECH_ACTIVITY.lock() {
        *lock = Some(Instant::now());
    }
}

pub fn last_speech_activity_elapsed() -> std::time::Duration {
    if let Ok(lock) = LAST_SPEECH_ACTIVITY.lock() {
        if let Some(t) = *lock {
            return t.elapsed();
        }
    }
    std::time::Duration::from_secs(999999)
}

pub fn next_speaking_epoch() -> u64 {
    SPEAKING_EPOCH.fetch_add(1, Ordering::SeqCst) + 1
}

pub fn current_speaking_epoch() -> u64 {
    SPEAKING_EPOCH.load(Ordering::SeqCst)
}

pub fn set_ai_speaking(speaking: bool) {
    AI_SPEAKING_FLAG.store(speaking, Ordering::SeqCst);
}

pub fn is_ai_speaking() -> bool {
    AI_SPEAKING_FLAG.load(Ordering::SeqCst)
}

pub fn trigger_interruption() {
    INTERRUPT_FLAG.store(true, Ordering::SeqCst);
    clear_client_actions();
    crate::audio::supertonic::clear_all_audio_cache();
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

// -----------------------------------------------------------------------------
// Anti-Echolalia Speaker Echo Guard (Acoustic Loopback Prevention)
// -----------------------------------------------------------------------------
static LAST_SPOKEN_TEXT: Mutex<Option<(String, Instant)>> = Mutex::new(None);

pub fn record_last_spoken_text(text: &str) {
    if let Ok(mut lock) = LAST_SPOKEN_TEXT.lock() {
        *lock = Some((text.to_lowercase(), Instant::now()));
    }
}

pub fn is_echo_of_last_spoken(transcript: &str) -> bool {
    let Ok(lock) = LAST_SPOKEN_TEXT.lock() else {
        return false;
    };
    let Some((ref text, instant)) = *lock else {
        return false;
    };

    // Speaker voice reflection protection for up to 12 seconds after it is spoken
    if instant.elapsed().as_secs() > 12 {
        return false;
    }

    let trans_clean = transcript.trim().to_lowercase();
    let trans_words: Vec<&str> = trans_clean
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .collect();
    if trans_words.is_empty() {
        return false;
    }

    // 1. Check exact match or substring
    if text.contains(&trans_clean) || trans_clean.contains(text) {
        return true;
    }

    // 2. Check the trigram (3 consecutive words)
    if trans_words.len() >= 3 {
        for window in trans_words.windows(3) {
            let trigram = format!("{} {} {}", window[0], window[1], window[2]);
            if text.contains(&trigram) {
                return true;
            }
        }
    }

    // 3. Check significant word overlap (> 50% words exist in assistant sentence)
    let mut match_count = 0;
    for &word in &trans_words {
        if text.contains(word) {
            match_count += 1;
        }
    }
    let ratio = match_count as f32 / trans_words.len() as f32;
    ratio >= 0.50
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

pub fn set_voice_muted(muted: bool) {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                if s.voice_muted != muted {
                    s.voice_muted = muted;
                    s.state_version = s.state_version.wrapping_add(1);
                }
            }
        }
    }
}

pub fn set_mic_muted(muted: bool) {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                if s.mic_muted != muted {
                    s.mic_muted = muted;
                    s.state_version = s.state_version.wrapping_add(1);
                }
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
    let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
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
// Thread-safe ring buffer with sequence ID per client for multi-client SSE pub-sub
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientAction {
    pub id: u64,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

static NEXT_ACTION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
pub static CLIENT_ACTIONS_BUFFER: Mutex<std::collections::VecDeque<ClientAction>> =
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

pub fn get_latest_action_id() -> u64 {
    NEXT_ACTION_ID
        .load(std::sync::atomic::Ordering::SeqCst)
        .saturating_sub(1)
}

pub fn clear_client_actions() {
    if let Ok(mut q) = CLIENT_ACTIONS_BUFFER.lock() {
        q.clear();
    }
}

pub fn push_client_action(action: &str, payload: Option<&str>) {
    let id = NEXT_ACTION_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let item = ClientAction {
        id,
        action: action.to_string(),
        payload: payload.map(|s| s.to_string()),
    };

    if let Ok(mut q) = CLIENT_ACTIONS_BUFFER.lock() {
        if q.len() >= 50 {
            q.pop_front();
        }
        q.push_back(item);
    }
    notify_state_changed();
}

pub fn get_client_actions_since(last_id: u64) -> (Vec<ClientAction>, u64) {
    if let Ok(q) = CLIENT_ACTIONS_BUFFER.lock() {
        let items: Vec<ClientAction> = q.iter().filter(|a| a.id > last_id).cloned().collect();
        let new_last_id = q.back().map(|a| a.id).unwrap_or(last_id).max(last_id);
        (items, new_last_id)
    } else {
        (Vec::new(), last_id)
    }
}
