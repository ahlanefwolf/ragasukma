use serde::Deserialize;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::app_state::{ACTIVE_WEB_SESSION, AppStatePayload, ServerSharedState};
use crate::server::send_json_response;
use crate::ui::pose_system::AssistantVisualState;

pub fn handle_menu_cmd(
    body_str: &str,
    stream: &mut dyn Write,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    #[derive(Deserialize)]
    struct CmdPayload {
        cmd: String,
    }

    if let Ok(payload) = serde_json::from_str::<CmdPayload>(body_str) {
        if payload.cmd == "start_chat" {
            if let Ok(mut lock) = shared.lock() {
                lock.tuning_mode = false;
                lock.live_tuning_bones.clear();
                lock.preview_poses = None;
                lock.poses_updated = true;
                lock.is_running = true;
                lock.current_state = AssistantVisualState::Idle;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if payload.cmd == "stop_chat" {
            if let Ok(mut lock) = shared.lock() {
                lock.is_running = false;
                lock.current_state = AssistantVisualState::Idle;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if payload.cmd == "toggle_pause" {
            if let Ok(mut lock) = shared.lock() {
                lock.is_paused = !lock.is_paused;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if payload.cmd == "mute_voice" {
            if let Ok(mut lock) = shared.lock() {
                lock.voice_muted = true;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if payload.cmd == "unmute_voice" {
            if let Ok(mut lock) = shared.lock() {
                lock.voice_muted = false;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if payload.cmd == "mute_mic" {
            if let Ok(mut lock) = shared.lock() {
                lock.mic_muted = true;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if payload.cmd == "unmute_mic" {
            if let Ok(mut lock) = shared.lock() {
                lock.mic_muted = false;
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        } else if payload.cmd == "mode_remote_desktop" {
            if let Ok(mut lock) = shared.lock() {
                // Mode Remote PC: TTS in PC (voice_muted = false), Mic in HP (mic PC dimute: mic_muted = true), Music in PC
                let changed = lock.voice_muted || !lock.mic_muted;
                lock.voice_muted = false;
                lock.mic_muted = true;
                if changed {
                    lock.state_version = lock.state_version.wrapping_add(1);
                }
            }
        } else if payload.cmd == "mode_full_mobile" {
            if let Ok(mut lock) = shared.lock() {
                // Mode Full HP: TTS in HP, Mic in HP, Music in HP (all audio PC dimute)
                let changed = !lock.voice_muted || !lock.mic_muted;
                lock.voice_muted = true;
                lock.mic_muted = true;
                if changed {
                    lock.state_version = lock.state_version.wrapping_add(1);
                }
            }
        } else if payload.cmd == "mode_reset_desktop" || payload.cmd == "reset_desktop_mode" || payload.cmd == "mode_desktop" {
            if let Ok(mut lock) = shared.lock() {
                // Mode Full PC: TTS in PC, Mic in PC, Music in PC (all active normal in PC)
                let changed = lock.voice_muted || lock.mic_muted;
                lock.voice_muted = false;
                lock.mic_muted = false;
                if changed {
                    lock.state_version = lock.state_version.wrapping_add(1);
                }
            }
        }

        crate::tray::TrayState::push_command(payload.cmd);
        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid cmd payload\"}");
    }
}

pub fn handle_get_state(stream: &mut dyn Write, shared: &Arc<Mutex<ServerSharedState>>) {
    if let Ok(lock) = shared.lock() {
        let st_str = match lock.current_state {
            AssistantVisualState::Idle => "idle",
            AssistantVisualState::Listening => "listening",
            AssistantVisualState::Speaking => "speaking",
            AssistantVisualState::Thinking => "thinking",
        };
        let active_token = ACTIVE_WEB_SESSION
            .lock()
            .ok()
            .and_then(|s| s.as_ref().map(|x| x.token.clone()));
        let payload = AppStatePayload {
            state: st_str.to_string(),
            emotion: Some(lock.current_emotion.clone()),
            is_running: lock.is_running,
            is_paused: lock.is_paused,
            voice_muted: lock.voice_muted,
            mic_muted: lock.mic_muted,
            active_session_token: active_token,
        };
        let json = serde_json::to_string(&payload).unwrap_or_default();
        send_json_response(stream, 200, &json);
    }
}

pub fn handle_post_state(
    body_str: &str,
    stream: &mut dyn Write,
    shared: &Arc<Mutex<ServerSharedState>>,
) {
    if let Ok(payload) = serde_json::from_str::<AppStatePayload>(body_str) {
        let mut cmd_to_send = None;
        if let Ok(mut lock) = shared.lock() {
            let prev_running = lock.is_running;
            let prev_paused = lock.is_paused;

            lock.is_running = payload.is_running;
            lock.is_paused = payload.is_paused;
            lock.voice_muted = payload.voice_muted;
            lock.mic_muted = payload.mic_muted;
            lock.state_version = lock.state_version.wrapping_add(1);

            if payload.is_running && !prev_running {
                lock.tuning_mode = false;
                lock.live_tuning_bones.clear();
                lock.preview_poses = None;
                lock.poses_updated = true;
                lock.current_state = AssistantVisualState::Idle;
                cmd_to_send = Some("start_chat".to_string());
            } else if !payload.is_running && prev_running {
                lock.current_state = AssistantVisualState::Idle;
                cmd_to_send = Some("stop_chat".to_string());
            } else if payload.is_paused != prev_paused {
                cmd_to_send = Some("toggle_pause".to_string());
            }
        }
        if let Some(cmd) = cmd_to_send {
            crate::tray::TrayState::push_command(cmd);
        }
        send_json_response(stream, 200, "{\"success\":true}");
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid state payload\"}");
    }
}
