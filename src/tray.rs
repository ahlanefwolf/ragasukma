use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistantState {
    Idle,
    Listening,
    #[allow(dead_code)]
    Processing,
    Thinking,
    Speaking,
}

static COMMAND_QUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static CURRENT_STATE: Mutex<Option<serde_json::Value>> = Mutex::new(None);
static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

pub struct TrayState;

impl TrayState {
    pub fn update(state: AssistantState, message: &str) {
        Self::update_with_emotion(state, message, "neutral");
    }

    pub fn update_with_emotion(state: AssistantState, message: &str, emotion: &str) {
        let (_icon, class_name) = match state {
            AssistantState::Idle => (" ", "idle"),
            AssistantState::Listening => (" ", "listening"),
            AssistantState::Processing => (" ", "processing"),
            AssistantState::Thinking => (" ", "thinking"),
            AssistantState::Speaking => (" ", "speaking"),
        };

        let tooltip = format!("{}: {}", class_name, message);

        let json = serde_json::json!({
            "text": " ",
            "alt": class_name,
            "tooltip": tooltip,
            "class": class_name,
            "emotion": emotion,
            "description": message
        });

        if let Ok(mut lock) = CURRENT_STATE.lock() {
            *lock = Some(json.clone());
        }

        Self::emit_event("assistant:state", json);
    }

    pub fn emit_event(event_name: &str, payload: serde_json::Value) {
        if let Ok(guard) = crate::overlay::APP_HANDLE.lock() {
            if let Some(app) = guard.as_ref() {
                use tauri::Emitter;
                let _ = app.emit(event_name, payload);
            }
        }
    }

    pub fn get_state() -> Option<serde_json::Value> {
        if let Ok(lock) = CURRENT_STATE.lock() {
            lock.clone()
        } else {
            None
        }
    }

    pub fn push_command(cmd: String) {
        if cmd == "quit" {
            QUIT_REQUESTED.store(true, Ordering::SeqCst);
        }
        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            // Selalu simpan hanya 1 command terakhir yang relevan agar tidak terjadi stale/lagged state
            queue.clear();
            queue.push_back(cmd);
        }
    }

    pub fn clear_commands() {
        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            queue.clear();
        }
    }

    pub fn pop_command() -> Option<String> {
        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            queue.pop_front()
        } else {
            None
        }
    }

    pub fn is_quit_requested() -> bool {
        QUIT_REQUESTED.load(Ordering::SeqCst)
    }

    pub fn request_quit() {
        QUIT_REQUESTED.store(true, Ordering::SeqCst);
        Self::push_command("quit".to_string());
    }

    pub fn clear() {
        if let Ok(mut lock) = CURRENT_STATE.lock() {
            *lock = None;
        }
    }
}
