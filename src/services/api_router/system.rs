use std::sync::{Arc, Mutex};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::{ACTIVE_WEB_SESSION, ActiveWebSession, AppStatePayload, ServerSharedState};
use crate::ui::pose_system::AssistantVisualState;

pub fn register_system_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    router
        // System & Model Manager
        .route("/api/system/apps", get(axum_get_apps))
        .route("/api/system/extension-path", get(axum_get_extension_path))
        .route("/api/system/model-status", get(axum_get_model_status))
        .route("/api/system/model-download", post(axum_post_model_download))
        .route("/api/system/python/info", get(axum_python_info))
        .route("/api/system/python/install", post(axum_python_install))
        // Menu & State
        .route("/api/cmd", post(axum_menu_cmd))
        .route("/api/state", get(axum_get_state).post(axum_post_state))
        // Actions Polling
        .route("/api/actions/since", get(axum_get_actions_since))
        // Logs
        .route("/api/logs", get(axum_get_logs))
        .route("/api/logs/clear", post(axum_clear_logs))
        // Sessions
        .route("/api/session/claim", post(axum_session_claim))
        .route("/api/session/tab", post(axum_session_tab))
}

// -----------------------------------------------------------------------------
// 1. System & Model Manager Service
// -----------------------------------------------------------------------------
// -----------------------------------------------------------------------------
pub(crate) async fn axum_get_apps() -> impl IntoResponse {
    Json(crate::engine::get_installed_applications())
}

pub(crate) async fn axum_get_extension_path() -> impl IntoResponse {
    let zip_file = crate::assets::find_file_in_candidates("ragasukma-companion.zip");
    let target_dir = if zip_file.exists() {
        zip_file
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(crate::assets::get_assets_dir)
    } else {
        crate::assets::get_assets_dir()
    };

    let abs_zip = zip_file.canonicalize().unwrap_or(zip_file);
    let abs_folder = target_dir.canonicalize().unwrap_or(target_dir);

    #[cfg(target_os = "windows")]
    let clean_zip_str = abs_zip.to_string_lossy().trim_start_matches(r"\\?\").to_string();
    #[cfg(not(target_os = "windows"))]
    let clean_zip_str = abs_zip.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    let clean_folder_str = abs_folder.to_string_lossy().trim_start_matches(r"\\?\").to_string();
    #[cfg(not(target_os = "windows"))]
    let clean_folder_str = abs_folder.to_string_lossy().to_string();

    Json(json!({
        "folder": clean_folder_str,
        "zip": clean_zip_str
    }))
}

pub(crate) async fn axum_get_model_status() -> impl IntoResponse {
    Json(crate::services::model_manager_service::get_model_status())
}

pub(crate) async fn axum_post_model_download() -> impl IntoResponse {
    crate::services::model_manager_service::ensure_model_ready_background();
    Json(crate::services::model_manager_service::get_model_status())
}

pub(crate) async fn axum_python_info() -> impl IntoResponse {
    Json(crate::engine::python::get_python_info())
}

pub(crate) async fn axum_python_install() -> impl IntoResponse {
    crate::engine::python::ensure_python_installed_background();
    Json(crate::engine::python::get_python_info())
}

// -----------------------------------------------------------------------------
// 3. Menu & State Service
// -----------------------------------------------------------------------------
#[derive(Deserialize)]
pub(crate) struct CmdPayload {
    #[serde(alias = "command")]
    pub(crate) cmd: String,
}

pub(crate) async fn axum_menu_cmd(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(payload): Json<CmdPayload>,
) -> impl IntoResponse {
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
            let changed = lock.voice_muted || !lock.mic_muted;
            lock.voice_muted = false;
            lock.mic_muted = true;
            if changed {
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        }
    } else if payload.cmd == "mode_full_mobile" {
        if let Ok(mut lock) = shared.lock() {
            let changed = !lock.voice_muted || !lock.mic_muted;
            lock.voice_muted = true;
            lock.mic_muted = true;
            if changed {
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        }
    } else if payload.cmd == "mode_reset_desktop" || payload.cmd == "reset_desktop_mode" || payload.cmd == "mode_desktop" {
        if let Ok(mut lock) = shared.lock() {
            let changed = lock.voice_muted || lock.mic_muted;
            lock.voice_muted = false;
            lock.mic_muted = false;
            if changed {
                lock.state_version = lock.state_version.wrapping_add(1);
            }
        }
    }

    crate::tray::TrayState::push_command(payload.cmd);
    Json(json!({ "success": true }))
}

pub(crate) async fn axum_get_state(State(shared): State<Arc<Mutex<ServerSharedState>>>) -> impl IntoResponse {
    let payload = if let Ok(lock) = shared.lock() {
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
        AppStatePayload {
            state: st_str.to_string(),
            emotion: Some(lock.current_emotion.clone()),
            is_running: lock.is_running,
            is_paused: lock.is_paused,
            voice_muted: lock.voice_muted,
            mic_muted: lock.mic_muted,
            active_session_token: active_token,
        }
    } else {
        AppStatePayload {
            state: "idle".to_string(),
            emotion: None,
            is_running: false,
            is_paused: false,
            voice_muted: false,
            mic_muted: false,
            active_session_token: None,
        }
    };
    Json(payload)
}

pub(crate) async fn axum_post_state(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(payload): Json<AppStatePayload>,
) -> impl IntoResponse {
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
    Json(json!({ "success": true }))
}

// -----------------------------------------------------------------------------
// 4. Actions Polling Service
// -----------------------------------------------------------------------------
#[derive(Deserialize)]
pub(crate) struct LastIdQuery {
    #[serde(default)]
    pub(crate) last_id: Option<u64>,
}

pub(crate) async fn axum_get_actions_since(Query(q): Query<LastIdQuery>) -> impl IntoResponse {
    let last_id = q.last_id.unwrap_or(0);
    let (actions, new_last_id) = if last_id == 0 {
        (Vec::new(), crate::app_state::get_latest_action_id())
    } else {
        crate::app_state::get_client_actions_since(last_id)
    };
    Json(json!({
        "actions": actions,
        "last_id": new_last_id
    }))
}

// -----------------------------------------------------------------------------
// Logs Service
// -----------------------------------------------------------------------------
#[derive(Deserialize)]
pub(crate) struct LogsQuery {
    #[serde(default)]
    pub(crate) last_id: Option<usize>,
}

pub(crate) async fn axum_get_logs(Query(q): Query<LogsQuery>) -> impl IntoResponse {
    let last_id = q.last_id.unwrap_or(0);
    let (logs, _) = crate::config::get_logs_since(last_id);
    Json(logs)
}

pub(crate) async fn axum_clear_logs() -> impl IntoResponse {
    crate::config::clear_logs();
    Json(json!({ "success": true }))
}

// -----------------------------------------------------------------------------
// 7. Sessions Service
// -----------------------------------------------------------------------------
#[derive(Deserialize)]
pub(crate) struct ClaimPayload {
    pub(crate) token: String,
    #[serde(default, alias = "activeTab")]
    pub(crate) active_tab: Option<String>,
}

pub(crate) async fn axum_session_claim(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(payload): Json<ClaimPayload>,
) -> impl IntoResponse {
    let now = std::time::Instant::now();
    let current_tab = payload.active_tab.unwrap_or_else(|| "menu".to_string());
    if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
        *lock = Some(ActiveWebSession {
            token: payload.token.clone(),
            active_tab: current_tab,
            last_seen: now,
        });
    }
    if let Ok(mut s) = shared.lock() {
        s.state_version = s.state_version.wrapping_add(1);
    }
    crate::app_state::push_client_action("session:claimed", Some(&payload.token));
    Json(json!({ "success": true }))
}

#[derive(Deserialize)]
pub(crate) struct TabPayload {
    #[serde(alias = "activeTab")]
    pub(crate) active_tab: String,
}

pub(crate) async fn axum_session_tab(Json(payload): Json<TabPayload>) -> impl IntoResponse {
    if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
        if let Some(session) = lock.as_mut() {
            session.active_tab = payload.active_tab;
            session.last_seen = std::time::Instant::now();
        }
    }
    Json(json!({ "success": true }))
}
