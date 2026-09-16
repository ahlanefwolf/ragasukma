use serde::Deserialize;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::app_state::ServerSharedState;
use crate::assets::{load_poses, save_poses};
use crate::server::send_json_response;
use crate::ui::pose_system::AssistantVisualState;

pub fn handle_get_poses(stream: &mut TcpStream) {
    let poses = load_poses();
    let json = serde_json::to_string(&poses).unwrap_or_default();
    send_json_response(stream, 200, &json);
}

pub fn handle_post_poses(
    body_str: &str,
    stream: &mut TcpStream,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    if let Ok(poses) = serde_json::from_str::<crate::ui::pose_system::PosesConfig>(body_str) {
        if let Err(e) = save_poses(&poses) {
            send_json_response(stream, 500, &format!("{{\"error\":\"{}\"}}", e));
        } else {
            if let Ok(mut lock) = shared.lock() {
                lock.poses_updated = true;
                lock.preview_poses = None;
            }
            send_json_response(stream, 200, "{\"success\":true}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid poses format\"}");
    }
}

pub fn handle_tuner_preview(
    body_str: &str,
    stream: &mut TcpStream,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    #[derive(Deserialize)]
    struct TunerPreviewPayload {
        poses: crate::ui::pose_system::PosesConfig,
        emotion: Option<String>,
        state: Option<String>,
    }

    if let Ok(payload) = serde_json::from_str::<TunerPreviewPayload>(body_str) {
        if let Ok(mut lock) = shared.lock() {
            lock.tuning_mode = false;
            lock.live_tuning_bones.clear();
            lock.preview_poses = Some(payload.poses);
            if let Some(st_str) = payload.state {
                lock.current_state = match st_str.to_lowercase().as_str() {
                    "listening" => AssistantVisualState::Listening,
                    "speaking" => AssistantVisualState::Speaking,
                    "thinking" => AssistantVisualState::Thinking,
                    _ => AssistantVisualState::Idle,
                };
            }
            if let Some(emo) = payload.emotion {
                lock.current_emotion = emo;
            }
        }
        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid preview payload\"}");
    }
}

pub fn handle_tuner_live(
    body_str: &str,
    stream: &mut TcpStream,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    #[derive(Deserialize)]
    struct TunerLivePayload {
        active: bool,
        #[serde(default)]
        bones: std::collections::HashMap<String, [f32; 3]>,
        emotion: Option<String>,
        state: Option<String>,
    }

    if let Ok(payload) = serde_json::from_str::<TunerLivePayload>(body_str) {
        if let Ok(mut lock) = shared.lock() {
            if payload.active {
                lock.tuning_mode = true;
                if let Some(st_str) = payload.state {
                    lock.current_state = match st_str.to_lowercase().as_str() {
                        "listening" => AssistantVisualState::Listening,
                        "speaking" => AssistantVisualState::Speaking,
                        "thinking" => AssistantVisualState::Thinking,
                        _ => AssistantVisualState::Idle,
                    };
                }
                lock.live_tuning_bones = payload.bones;
                if let Some(emo) = payload.emotion {
                    lock.current_emotion = emo;
                }
            } else {
                lock.tuning_mode = false;
                lock.live_tuning_bones.clear();
                lock.preview_poses = None;
                lock.poses_updated = true;

                let (real_state, real_emotion) =
                    crate::tray::TrayState::get_current_runtime_state();
                lock.current_state = real_state;
                lock.current_emotion = real_emotion;
            }
        }
        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid live tuner payload\"}");
    }
}
