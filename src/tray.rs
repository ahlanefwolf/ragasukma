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
#[allow(dead_code)]
static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

pub struct TrayState;

impl TrayState {
    pub fn update(state: AssistantState, message: &str) {
        let current_emo = CURRENT_STATE
            .lock()
            .ok()
            .and_then(|l| l.as_ref().and_then(|v| v.get("emotion").and_then(|e| e.as_str().map(|s| s.to_string()))))
            .unwrap_or_else(|| "neutral".to_string());
        Self::update_with_emotion(state, message, &current_emo);
    }

    pub fn update_with_emotion(state: AssistantState, message: &str, emotion: &str) {
        let class_name = match state {
            AssistantState::Idle => "idle",
            AssistantState::Listening => "listening",
            AssistantState::Processing => "processing",
            AssistantState::Thinking => "thinking",
            AssistantState::Speaking => "speaking",
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

        let visual_state = match state {
            AssistantState::Idle => crate::native::pose_system::AssistantVisualState::Idle,
            AssistantState::Listening => crate::native::pose_system::AssistantVisualState::Listening,
            AssistantState::Processing | AssistantState::Thinking => crate::native::pose_system::AssistantVisualState::Thinking,
            AssistantState::Speaking => crate::native::pose_system::AssistantVisualState::Speaking,
        };
        crate::server::update_visual_state_with_emotion(visual_state, emotion);
    }

    pub fn get_current_runtime_state() -> (crate::native::pose_system::AssistantVisualState, String) {
        if let Ok(lock) = CURRENT_STATE.lock() {
            if let Some(val) = lock.as_ref() {
                let class_str = val.get("class").and_then(|v| v.as_str()).unwrap_or("idle");
                let emotion_str = val.get("emotion").and_then(|v| v.as_str()).unwrap_or("neutral").to_string();
                let visual_state = match class_str {
                    "listening" => crate::native::pose_system::AssistantVisualState::Listening,
                    "speaking" => crate::native::pose_system::AssistantVisualState::Speaking,
                    "thinking" | "processing" => crate::native::pose_system::AssistantVisualState::Thinking,
                    _ => crate::native::pose_system::AssistantVisualState::Idle,
                };
                return (visual_state, emotion_str);
            }
        }
        (crate::native::pose_system::AssistantVisualState::Idle, "neutral".to_string())
    }

    pub fn emit_event(_event_name: &str, _payload: serde_json::Value) {}

    pub fn push_command(cmd: String) {
        if cmd == "quit" {
            QUIT_REQUESTED.store(true, Ordering::SeqCst);
        }
        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
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

    pub fn clear() {
        if let Ok(mut lock) = CURRENT_STATE.lock() {
            *lock = None;
        }
    }
}
