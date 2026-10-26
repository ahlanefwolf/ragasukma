use std::sync::{Arc, Mutex};
use axum::{
    extract::{Query, State},
    http::StatusCode,
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
        // Config Service
        .route("/api/config", get(axum_get_config).post(axum_post_config))
        // System & Model Manager
        .route("/api/system/apps", get(axum_get_apps))
        .route("/api/system/extension-path", get(axum_get_extension_path))
        .route("/api/system/model-status", get(axum_get_model_status))
        .route("/api/system/model-download", post(axum_post_model_download))
        .route("/api/system/bun/info", get(axum_bun_info))
        .route("/api/system/bun/install", post(axum_bun_install))
        // Menu & State
        .route("/api/cmd", post(axum_menu_cmd))
        .route("/api/state", get(axum_get_state).post(axum_post_state))
        // Actions Polling
        .route("/api/actions/since", get(axum_get_actions_since))
        // History
        .route("/api/history/dates", get(axum_get_history_dates))
        .route("/api/history/view", get(axum_get_history_view))
        .route("/api/history/delete", post(axum_delete_history))
        .route("/api/chat/events", get(axum_chat_events_sse))
        // Logs
        .route("/api/logs", get(axum_get_logs))
        .route("/api/logs/clear", post(axum_clear_logs))
        // Sessions
        .route("/api/session/claim", post(axum_session_claim))
        .route("/api/session/tab", post(axum_session_tab))
}

// -----------------------------------------------------------------------------
// 1. Config Service
// -----------------------------------------------------------------------------
pub(crate) async fn axum_get_config() -> impl IntoResponse {
    Json(crate::assets::load_config())
}

pub(crate) async fn axum_post_config(
    State(shared): State<Arc<Mutex<ServerSharedState>>>,
    Json(cfg): Json<crate::config::UserConfig>,
) -> impl IntoResponse {
    if let Err(e) = crate::assets::save_config(&cfg) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response()
    } else {
        if let Ok(mut lock) = shared.lock() {
            lock.config_updated = true;
        }
        Json(json!({ "success": true })).into_response()
    }
}

// -----------------------------------------------------------------------------
// 2. System & Model Manager Service
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

pub(crate) async fn axum_bun_info() -> impl IntoResponse {
    Json(crate::engine::bun::get_bun_info())
}

pub(crate) async fn axum_bun_install() -> impl IntoResponse {
    crate::engine::bun::ensure_bun_installed_background();
    Json(crate::engine::bun::get_bun_info())
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
// 5. History Service
// -----------------------------------------------------------------------------
// 5. History Service (SQLite-Powered with Legacy Markdown Fallback)
// -----------------------------------------------------------------------------
pub(crate) async fn axum_get_history_dates() -> impl IntoResponse {
    let dates = crate::llm::history::get_history_dates().await;
    Json(dates)
}

#[derive(Deserialize)]
pub(crate) struct HistoryViewQuery {
    #[serde(default)]
    pub(crate) date: Option<String>,
    #[serde(default)]
    pub(crate) page: Option<usize>,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
}

pub(crate) async fn axum_get_history_view(Query(q): Query<HistoryViewQuery>) -> impl IntoResponse {
    let target_date = match q.date {
        Some(ref d) if !d.trim().is_empty() => d.trim().to_string(),
        _ => {
            let dates = crate::llm::history::get_history_dates().await;
            dates.first().cloned().unwrap_or_default()
        }
    };

    if target_date.is_empty() {
        return Json(json!({
            "date": "",
            "turns": [],
            "total_turns": 0,
            "page": 1,
            "limit": 20,
            "total_pages": 1
        }))
        .into_response();
    }

    let page = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(20);
    let paginated = crate::llm::history::get_history_by_date_paginated(&target_date, page, limit).await;
    Json(paginated).into_response()
}

#[derive(Deserialize)]
pub(crate) struct DeleteHistoryPayload {
    pub(crate) date: String,
}

pub(crate) async fn axum_delete_history(Json(payload): Json<DeleteHistoryPayload>) -> impl IntoResponse {
    let clean_date = payload.date.trim();
    if clean_date.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing date parameter" })),
        )
            .into_response();
    }

    match crate::llm::history::delete_history_by_date(clean_date).await {
        Ok(true) => Json(json!({ "success": true })).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "History record not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to delete history: {e}") })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_chat_events_sse() -> impl IntoResponse {
    let sender = crate::llm::history::get_chat_event_sender();
    let mut rx = sender.subscribe();

    let stream = async_stream::stream! {
        let init_json = serde_json::json!({
            "type": "connected"
        });
        yield Ok::<_, std::convert::Infallible>(format!("data: {}\n\n", init_json));

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(item) => {
                            let evt = serde_json::json!({
                                "type": "chat_turn",
                                "item": item
                            });
                            if let Ok(json_str) = serde_json::to_string(&evt) {
                                yield Ok(format!("data: {}\n\n", json_str));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                            eprintln!("[Chat SSE] Recv lagged by {} messages", missed);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                    // Send periodic keep-alive event every 3s to flush Cloudflare & proxy buffers
                    yield Ok("event: ping\ndata: {}\n\n".to_string());
                }
            }
        }
    };

    axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream; charset=utf-8")
        .header("Cache-Control", "no-cache, no-transform")
        .header("Connection", "keep-alive")
        .header("X-Accel-Buffering", "no")
        .header("Access-Control-Allow-Origin", "*")
        .body(axum::body::Body::from_stream(stream))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create SSE stream").into_response())
}

// -----------------------------------------------------------------------------
// 6. Logs Service
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
