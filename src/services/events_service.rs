use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};

use crate::app_state::ServerSharedState;
use crate::ui::pose_system::AssistantVisualState;
use serde::Serialize;



// ---------------------------------------------------------------------------
// 1. GLOBAL APP STATE PAYLOAD
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Clone)]
pub struct CoreStatePayload {
    pub state: String,
    pub emotion: String,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub is_running: bool,
}



// ---------------------------------------------------------------------------
// 2. HTTP ACTIONS POLLING (/api/actions/since?last_id=...)
// ---------------------------------------------------------------------------
pub fn handle_get_actions_since(full_url: &str, stream: &mut dyn Write) {
    let last_id = full_url
        .split('?')
        .nth(1)
        .and_then(|q| {
            q.split('&').find_map(|pair| {
                let mut kv = pair.split('=');
                if kv.next()? == "last_id" {
                    kv.next()?.parse::<u64>().ok()
                } else {
                    None
                }
            })
        })
        .unwrap_or(0);

    let (actions, new_last_id) = if last_id == 0 {
        // First connection / refresh: only take ID terbaru without sending repeat aksi history old!
        (Vec::new(), crate::app_state::get_latest_action_id())
    } else {
        crate::app_state::get_client_actions_since(last_id)
    };
    let resp = serde_json::json!({
        "actions": actions,
        "last_id": new_last_id
    })
    .to_string();

    crate::server::send_json_response(stream, 200, &resp);
}

// ---------------------------------------------------------------------------
// 3. AXUM WEBSOCKET HANDLER (/api/ws)
// Realtime 60 FPS sub-millisecond Pose Tuner & Interactive bi-directional link
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum WsIncomingMessage {
    #[serde(rename = "tuner_live")]
    TunerLive {
        active: bool,
        #[serde(default)]
        bones: std::collections::HashMap<String, [f32; 3]>,
        emotion: Option<String>,
        state: Option<String>,
    },
    #[serde(rename = "tuner_preview")]
    TunerPreview {
        poses: crate::ui::pose_system::PosesConfig,
        emotion: Option<String>,
        state: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping,
}

pub async fn axum_ws_handler(
    ws: WebSocketUpgrade,
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, shared))
}

async fn handle_ws_socket(socket: WebSocket, shared: Arc<Mutex<ServerSharedState>>) {
    let (mut sender, mut receiver) = socket.split();

    // Spawn background task to push real-time events (state, action, log, model_status) over WebSocket
    let shared_clone = Arc::clone(&shared);
    let mut ws_last_state = String::new();
    let mut ws_last_emotion = String::new();
    let mut ws_last_voice_muted = false;
    let mut ws_last_mic_muted = false;
    let mut ws_last_is_running = false;
    let mut ws_initial_sent = false;
    let mut ws_last_action_id = crate::app_state::get_latest_action_id();
    let mut ws_last_log_id = crate::config::get_latest_log_id().saturating_sub(40);
    let mut ws_last_model_status_msg = String::new();

    let (push_tx, mut push_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let push_task = tokio::spawn(async move {
        loop {
            crate::app_state::touch_remote_active();

            // 1. AI Visual State
            if let Ok(lock) = shared_clone.lock() {
                let st_str = if lock.is_shutdown {
                    "shutdown"
                } else {
                    match lock.current_state {
                        AssistantVisualState::Idle => "idle",
                        AssistantVisualState::Listening => "listening",
                        AssistantVisualState::Speaking => "speaking",
                        AssistantVisualState::Thinking => "thinking",
                    }
                };

                let v_muted = lock.voice_muted;
                let m_muted = lock.mic_muted;
                let emo = lock.current_emotion.clone();
                let running = lock.is_running;

                let changed = !ws_initial_sent
                    || st_str != ws_last_state
                    || emo != ws_last_emotion
                    || v_muted != ws_last_voice_muted
                    || m_muted != ws_last_mic_muted
                    || running != ws_last_is_running;

                if changed {
                    ws_initial_sent = true;
                    ws_last_state = st_str.to_string();
                    ws_last_emotion = emo.clone();
                    ws_last_voice_muted = v_muted;
                    ws_last_mic_muted = m_muted;
                    ws_last_is_running = running;

                    let msg = serde_json::json!({
                        "event": "state",
                        "data": {
                            "state": st_str,
                            "emotion": emo,
                            "voice_muted": v_muted,
                            "mic_muted": m_muted,
                            "is_running": running
                        }
                    });
                    if push_tx.send(msg.to_string()).is_err() {
                        break;
                    }
                }
            }

            // 2. Client actions (assistant:speak, music:play, etc.)
            let (pending_actions, new_last_id) =
                crate::app_state::get_client_actions_since(ws_last_action_id);
            ws_last_action_id = new_last_id;
            for act in pending_actions {
                let msg = serde_json::json!({
                    "event": "action",
                    "data": act
                });
                if push_tx.send(msg.to_string()).is_err() {
                    break;
                }
            }

            // 3. Realtime Logs
            let (new_logs, new_log_id) = crate::config::get_logs_since(ws_last_log_id);
            ws_last_log_id = new_log_id;
            for entry in new_logs {
                let msg = serde_json::json!({
                    "event": "log",
                    "data": entry
                });
                if push_tx.send(msg.to_string()).is_err() {
                    break;
                }
            }

            // 4. Model Status Broadcast
            let current_model_status = crate::services::model_manager_service::get_model_status();
            let status_sig = format!(
                "{}:{}:{:.1}",
                current_model_status.status,
                current_model_status.downloaded_bytes,
                current_model_status.percent
            );
            if status_sig != ws_last_model_status_msg {
                ws_last_model_status_msg = status_sig;
                let msg = serde_json::json!({
                    "event": "model_status",
                    "data": current_model_status
                });
                if push_tx.send(msg.to_string()).is_err() {
                    break;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    let shared_for_rx = Arc::clone(&shared);
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg_res) = receiver.next().await {
            let msg = match msg_res {
                Ok(m) => m,
                Err(_) => break,
            };

            if let Message::Text(text) = msg {
                if let Ok(incoming) = serde_json::from_str::<WsIncomingMessage>(&text) {
                    match incoming {
                        WsIncomingMessage::TunerLive {
                            active,
                            bones,
                            emotion,
                            state,
                        } => {
                            if let Ok(mut lock) = shared_for_rx.lock() {
                                if active {
                                    lock.tuning_mode = true;
                                    if let Some(st_str) = state {
                                        lock.current_state = match st_str.to_lowercase().as_str() {
                                            "listening" => AssistantVisualState::Listening,
                                            "speaking" => AssistantVisualState::Speaking,
                                            "thinking" => AssistantVisualState::Thinking,
                                            _ => AssistantVisualState::Idle,
                                        };
                                    }
                                    lock.live_tuning_bones = bones;
                                    if let Some(emo) = emotion {
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
                        }
                        WsIncomingMessage::TunerPreview {
                            poses,
                            emotion,
                            state,
                        } => {
                            if let Ok(mut lock) = shared_for_rx.lock() {
                                lock.tuning_mode = false;
                                lock.live_tuning_bones.clear();
                                lock.preview_poses = Some(poses);
                                if let Some(st_str) = state {
                                    lock.current_state = match st_str.to_lowercase().as_str() {
                                        "listening" => AssistantVisualState::Listening,
                                        "speaking" => AssistantVisualState::Speaking,
                                        "thinking" => AssistantVisualState::Thinking,
                                        _ => AssistantVisualState::Idle,
                                    };
                                }
                                if let Some(emo) = emotion {
                                    lock.current_emotion = emo;
                                }
                            }
                        }
                        WsIncomingMessage::Ping => {}
                    }
                }
            }
        }
    });

    loop {
        tokio::select! {
            Some(payload) = push_rx.recv() => {
                if sender.send(Message::Text(payload)).await.is_err() {
                    break;
                }
            }
            _ = &mut recv_task => {
                break;
            }
        }
    }

    push_task.abort();
    recv_task.abort();
}
