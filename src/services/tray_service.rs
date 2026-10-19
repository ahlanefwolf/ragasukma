use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistantState {
    Idle,
    Listening,
    Thinking,
    Speaking,
}

static COMMAND_QUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static CURRENT_STATE: Mutex<Option<serde_json::Value>> = Mutex::new(None);

pub struct TrayState;

impl TrayState {
    pub fn update(state: AssistantState, message: &str) {
        let current_emo = CURRENT_STATE
            .lock()
            .ok()
            .and_then(|l| {
                l.as_ref().and_then(|v| {
                    v.get("emotion")
                        .and_then(|e| e.as_str().map(|s| s.to_string()))
                })
            })
            .unwrap_or_else(|| "neutral".to_string());
        Self::update_with_emotion(state, message, &current_emo);
    }

    pub fn update_with_emotion(state: AssistantState, message: &str, emotion: &str) {
        let class_name = match state {
            AssistantState::Idle => "idle",
            AssistantState::Listening => "listening",
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
            *lock = Some(json);
        }

        let visual_state = match state {
            AssistantState::Idle => crate::ui::pose_system::AssistantVisualState::Idle,
            AssistantState::Listening => crate::ui::pose_system::AssistantVisualState::Listening,
            AssistantState::Thinking => crate::ui::pose_system::AssistantVisualState::Thinking,
            AssistantState::Speaking => crate::ui::pose_system::AssistantVisualState::Speaking,
        };
        crate::app_state::update_visual_state_with_emotion(visual_state, emotion);
    }

    pub fn get_current_runtime_state() -> (crate::ui::pose_system::AssistantVisualState, String) {
        if let Ok(lock) = CURRENT_STATE.lock() {
            if let Some(val) = lock.as_ref() {
                let class_str = val.get("class").and_then(|v| v.as_str()).unwrap_or("idle");
                let emotion_str = val
                    .get("emotion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("neutral")
                    .to_string();
                let visual_state = match class_str {
                    "listening" => crate::ui::pose_system::AssistantVisualState::Listening,
                    "speaking" => crate::ui::pose_system::AssistantVisualState::Speaking,
                    "thinking" => crate::ui::pose_system::AssistantVisualState::Thinking,
                    _ => crate::ui::pose_system::AssistantVisualState::Idle,
                };
                return (visual_state, emotion_str);
            }
        }
        (
            crate::ui::pose_system::AssistantVisualState::Idle,
            "neutral".to_string(),
        )
    }

    pub fn push_command(cmd: String) {
        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            if queue.len() >= 50 {
                queue.pop_front();
            }
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
