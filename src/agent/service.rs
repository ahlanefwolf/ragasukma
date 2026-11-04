use std::sync::{Arc, Mutex};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::agent::repository;
use crate::app_state::ServerSharedState;

pub fn register_agent_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    router
        // Projects
        .route("/api/agent/projects", get(list_projects).post(create_project))
        .route("/api/agent/projects/:id", get(get_project))
        .route("/api/agent/projects/:id/update", post(update_project))
        .route("/api/agent/projects/:id/delete", post(delete_project))
        .route("/api/agent/projects/:id/files", get(get_project_files))
        .route("/api/agent/projects/:id/file-content", get(get_project_file_content))
        .route("/api/agent/validate-folder", post(validate_folder))
        .route("/api/agent/browse-folder", post(browse_folder))
        .route("/api/agent/open-in-file-manager", post(open_in_file_manager))
        // Sessions
        .route("/api/agent/projects/:id/sessions", get(list_sessions).post(create_session))
        .route("/api/agent/sessions/:id/delete", post(delete_session))
        .route("/api/agent/sessions/:id/run", post(run_session))
        .route("/api/agent/sessions/:id/stop", post(stop_session))
        .route("/api/agent/sessions/:id/events", get(session_events_sse))
        .route("/api/agent/sessions/from-chat", post(create_session_from_chat))
        // Messages
        .route("/api/agent/sessions/:id/messages", get(list_messages).post(create_message))
        // Agent Skills (Auto-detected from project .skills / .skill folders)
        .route("/api/agent/skills", get(list_skills))
        .route("/api/agent/projects/:id/skills", get(get_project_skills))
        .route("/api/agent/projects/:id/mcp", get(get_project_mcp_servers_handler).post(set_project_mcp_servers_handler))
        // MCP Servers
        .route("/api/agent/mcp", get(list_mcp_servers).post(save_mcp_server))
        .route("/api/agent/mcp/:id/delete", post(delete_mcp_server))
        .route("/api/agent/mcp/test", post(test_mcp_server_handler))
        // Goals & Evidence
        .route("/api/agent/projects/:id/goals", get(list_project_goals).post(create_project_goal))
        .route("/api/agent/goals/:id", get(get_goal_details))
        .route("/api/agent/goals/:id/evidence", get(get_goal_evidences))
        .route("/api/agent/goals/:id/verifications", get(get_goal_verifications_list))
        .route("/api/agent/goals/:id/audit", get(get_goal_audit_logs))
        .route("/api/agent/sessions/:id/audit", get(get_session_audit_logs))
        .route("/api/agent/goals/:id/conclude", post(conclude_goal_handler))
}

// -----------------------------------------------------------------------------
// Projects Endpoints
// -----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateProjectPayload {
    pub name: String,
    pub description: Option<String>,
    pub root_path: Option<String>,
    pub folders: Option<Vec<String>>,
    pub custom_instructions: Option<String>,
}

#[derive(Deserialize)]
pub struct ValidateFolderPayload {
    pub path: String,
}

async fn validate_folder(Json(payload): Json<ValidateFolderPayload>) -> impl IntoResponse {
    let raw_path = payload.path.trim();
    if raw_path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"valid": false, "error": "Folder path cannot be empty"})),
        );
    }

    let p = std::path::Path::new(raw_path);
    if !p.exists() {
        return (
            StatusCode::OK,
            Json(json!({"valid": false, "error": format!("Folder not found on disk: {}", raw_path)})),
        );
    }
    if !p.is_dir() {
        return (
            StatusCode::OK,
            Json(json!({"valid": false, "error": format!("Path is not a directory/folder: {}", raw_path)})),
        );
    }

    let canonical = p.canonicalize().ok().map(|c| c.to_string_lossy().to_string()).unwrap_or_else(|| raw_path.to_string());
    (
        StatusCode::OK,
        Json(json!({"valid": true, "path": canonical})),
    )
}

#[derive(Deserialize, Default)]
pub struct BrowseFolderPayload {
    pub initial_path: Option<String>,
}

async fn browse_folder(body: axum::body::Bytes) -> impl IntoResponse {
    let payload = if !body.is_empty() {
        serde_json::from_slice::<BrowseFolderPayload>(&body).ok()
    } else {
        None
    };
    let initial_path = payload.and_then(|p| p.initial_path);

    let res = tokio::task::spawn_blocking(move || {
        crate::agent::tools::pick_folder_dialog(initial_path.as_deref())
    })
    .await;

    match res {
        Ok(Ok(Some(path))) => (
            StatusCode::OK,
            Json(json!({ "ok": true, "path": path })),
        ),
        Ok(Ok(None)) => (
            StatusCode::OK,
            Json(json!({ "ok": false, "cancelled": true })),
        ),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": e.to_string() })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": format!("Task panicked: {}", e) })),
        ),
    }
}

#[derive(Deserialize, Default)]
pub struct OpenFileManagerPayload {
    pub path: String,
}

async fn open_in_file_manager(body: axum::body::Bytes) -> impl IntoResponse {
    let payload = match serde_json::from_slice::<OpenFileManagerPayload>(&body) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": format!("Invalid JSON: {}", e) })),
            );
        }
    };

    let path = payload.path.trim().to_string();
    if path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "Path cannot be empty" })),
        );
    }

    let res = tokio::task::spawn_blocking(move || {
        crate::agent::tools::open_path_in_file_manager(&path)
    })
    .await;

    match res {
        Ok(Ok(())) => (
            StatusCode::OK,
            Json(json!({ "ok": true })),
        ),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": e.to_string() })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": format!("Task error: {}", e) })),
        ),
    }
}

fn resolve_and_validate_folders(
    folders_opt: Option<Vec<String>>,
    root_path_opt: Option<String>,
) -> Result<Option<String>, String> {
    let mut collected: Vec<String> = Vec::new();

    if let Some(folders) = folders_opt {
        for f in folders {
            let trimmed = f.trim();
            if !trimmed.is_empty() {
                collected.push(trimmed.to_string());
            }
        }
    } else if let Some(rp) = root_path_opt {
        let trimmed = rp.trim();
        if !trimmed.is_empty() {
            // Check if it's already a JSON array
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
                for item in arr {
                    let item_trim = item.trim();
                    if !item_trim.is_empty() {
                        collected.push(item_trim.to_string());
                    }
                }
            } else {
                collected.push(trimmed.to_string());
            }
        }
    }

    if collected.is_empty() {
        return Ok(None);
    }

    // Validate that each folder actually exists and is a directory
    for path_str in &collected {
        let p = std::path::Path::new(path_str);
        if !p.exists() {
            return Err(format!("Folder not found on disk: {}", path_str));
        }
        if !p.is_dir() {
            return Err(format!("Path is not a directory/folder: {}", path_str));
        }
    }

    let json_str = serde_json::to_string(&collected).unwrap_or_else(|_| "[]".to_string());
    Ok(Some(json_str))
}

async fn list_projects() -> impl IntoResponse {
    match repository::list_projects().await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_project(Path(id): Path<String>) -> impl IntoResponse {
    match repository::get_project(&id).await {
        Ok(Some(p)) => (StatusCode::OK, Json(json!(p))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Project not found"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_project(Json(payload): Json<CreateProjectPayload>) -> impl IntoResponse {
    let desc = payload.description.unwrap_or_default();
    let validated_root = match resolve_and_validate_folders(payload.folders, payload.root_path) {
        Ok(v) => v,
        Err(err_msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": err_msg})),
            );
        }
    };

    if let Some(ref vr) = validated_root {
        for folder in crate::agent::executor::parse_project_folders(Some(vr)) {
            let _ = crate::agent::tools::ensure_git_and_gitignore(std::path::Path::new(&folder));
        }
    }

    match repository::create_project(
        &payload.name,
        &desc,
        validated_root.as_deref(),
        payload.custom_instructions.as_deref(),
    )
    .await
    {
        Ok(p) => (StatusCode::CREATED, Json(json!(p))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn update_project(
    Path(id): Path<String>,
    Json(payload): Json<CreateProjectPayload>,
) -> impl IntoResponse {
    let desc = payload.description.unwrap_or_default();
    let validated_root = match resolve_and_validate_folders(payload.folders, payload.root_path) {
        Ok(v) => v,
        Err(err_msg) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": err_msg})),
            );
        }
    };

    if let Some(ref vr) = validated_root {
        for folder in crate::agent::executor::parse_project_folders(Some(vr)) {
            let _ = crate::agent::tools::ensure_git_and_gitignore(std::path::Path::new(&folder));
        }
    }

    match repository::update_project(
        &id,
        &payload.name,
        &desc,
        validated_root.as_deref(),
        payload.custom_instructions.as_deref(),
    )
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "ok"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn delete_project(Path(id): Path<String>) -> impl IntoResponse {
    match repository::delete_project(&id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "deleted"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
pub struct ReadFileQueryParams {
    pub path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

async fn get_project_files(Path(id): Path<String>) -> impl IntoResponse {
    let project = match repository::get_project(&id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Project not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        }
    };

    let allowed = crate::agent::executor::parse_project_folders(project.root_path.as_deref());
    for folder in &allowed {
        let _ = crate::agent::tools::ensure_git_and_gitignore(std::path::Path::new(folder));
    }
    match crate::agent::tools::list_workspace_files(&allowed) {
        Ok(files) => (StatusCode::OK, Json(json!({"status": "ok", "files": files}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_project_file_content(
    Path(id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<ReadFileQueryParams>,
) -> impl IntoResponse {
    let project = match repository::get_project(&id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Project not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        }
    };

    let allowed = crate::agent::executor::parse_project_folders(project.root_path.as_deref());
    let resolved = match crate::agent::tools::resolve_allowed_path(&params.path, &allowed) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"error": format!("Path access forbidden: {}", e)})),
            )
        }
    };

    if !resolved.exists() || !resolved.is_file() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "File not found"})),
        );
    }

    match std::fs::read_to_string(&resolved) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();

            let s = params.start_line.unwrap_or(1);
            let e = params.end_line.unwrap_or(total_lines).min(total_lines);

            let s_idx = s.saturating_sub(1);
            let sliced = if s_idx < e {
                lines[s_idx..e].join("\n")
            } else {
                String::new()
            };

            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "path": params.path,
                    "total_lines": total_lines,
                    "start_line": s,
                    "end_line": e,
                    "content": sliced,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to read file: {}", e)})),
        ),
    }
}

// -----------------------------------------------------------------------------
// Sessions Endpoints
// -----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateSessionPayload {
    pub title: String,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub source_chat_id: Option<String>,
}

async fn list_sessions(Path(project_id): Path<String>) -> impl IntoResponse {
    match repository::list_sessions(&project_id).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_session(
    Path(project_id): Path<String>,
    Json(payload): Json<CreateSessionPayload>,
) -> impl IntoResponse {
    match repository::create_session(
        &project_id,
        &payload.title,
        payload.model.as_deref(),
        payload.temperature,
        payload.source_chat_id.as_deref(),
    )
    .await
    {
        Ok(s) => (StatusCode::CREATED, Json(json!(s))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn delete_session(Path(id): Path<String>) -> impl IntoResponse {
    crate::agent::executor::cancel_session_task(&id);
    crate::agent::executor::remove_session_broadcaster(&id);
    crate::agent::diagnostics::clear_session_diagnostic_queue(&id);
    match repository::delete_session(&id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "deleted"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize, Default)]
pub struct RunSessionPayload {
    pub model: Option<String>,
}

async fn run_session(
    Path(id): Path<String>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let model_override = if !body.is_empty() {
        serde_json::from_slice::<RunSessionPayload>(&body)
            .ok()
            .and_then(|p| p.model)
    } else {
        None
    };

    let session_id = id.clone();
    // Cancel any previous task still running for this session to avoid multiple workers
    crate::agent::executor::cancel_session_task(&session_id);

    let sid_for_task = session_id.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = crate::agent::executor::execute_session_turn(&sid_for_task, model_override.as_deref()).await {
            eprintln!("[Agent] Execution error for session {}: {}", sid_for_task, e);
        }
        crate::agent::executor::unregister_session_task(&sid_for_task);
    });

    crate::agent::executor::register_session_task(&session_id, handle.abort_handle());

    (StatusCode::OK, Json(json!({"status": "started", "session_id": id})))
}

async fn stop_session(
    Path(id): Path<String>,
) -> impl IntoResponse {
    // 1. Immediately abort the running Tokio background task (and kill any active child process)
    let was_running = crate::agent::executor::cancel_session_task(&id);
    if was_running {
        println!("[Agent] Successfully aborted background task for session {}", id);
    }

    // 2. Persist idle status to database
    let _ = crate::agent::repository::update_session_status(&id, "idle").await;

    // 3. Notify subscribers via SSE
    crate::agent::executor::broadcast_session_event(
        &id,
        crate::agent::models::AgentSessionEvent::StatusChange {
            status: "idle".to_string(),
        },
    );
    crate::agent::executor::broadcast_session_event(
        &id,
        crate::agent::models::AgentSessionEvent::Done {
            reply: "Task execution stopped by user.".to_string(),
        },
    );
    (StatusCode::OK, Json(json!({"status": "stopped", "session_id": id, "aborted_active_task": was_running})))
}

async fn session_events_sse(
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let sender = crate::agent::executor::get_session_sender(&session_id);
    let mut rx = sender.subscribe();

    // Query current session status from database to send an immediate snapshot
    let current_status = crate::agent::repository::get_session(&session_id)
        .await
        .ok()
        .flatten()
        .map(|s| s.status)
        .unwrap_or_else(|| "idle".to_string());

    let stream = async_stream::stream! {
        // Initial connection confirmation event with status snapshot
        let init_json = serde_json::json!({
            "type": "connected",
            "data": {
                "session_id": session_id,
                "status": current_status,
            }
        });
        yield Ok::<_, std::convert::Infallible>(format!("data: {}\n\n", init_json));

        // Persistent keep-alive ping interval (prevents reallocating timer futures on every token)
        let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(3));
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(evt) => {
                            if let Ok(json_str) = serde_json::to_string(&evt) {
                                yield Ok(format!("data: {}\n\n", json_str));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                            eprintln!("[Agent SSE] Recv lagged by {} messages for session {}", missed, session_id);
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                _ = ping_interval.tick() => {
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

#[derive(Deserialize)]
pub struct CreateSessionFromChatPayload {
    pub chat_id: String,
    pub project_id: String,
    pub title: Option<String>,
}

async fn create_session_from_chat(
    Json(payload): Json<CreateSessionFromChatPayload>,
) -> impl IntoResponse {
    let pool = crate::llm::usage::get_pool().await;
    let turn: Option<(String, String)> = match sqlx::query_as(
        "SELECT user_msg, assistant_msg FROM chat_history WHERE id = ?1",
    )
    .bind(&payload.chat_id)
    .fetch_optional(pool)
    .await
    {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            );
        }
    };

    let Some((user_msg, assistant_msg)) = turn else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Chat conversation turn not found"})),
        );
    };

    let title = payload.title.unwrap_or_else(|| {
        let first_line = user_msg.lines().next().unwrap_or("Task from Chat").trim();
        if first_line.chars().count() > 40 {
            format!("{}...", first_line.chars().take(40).collect::<String>())
        } else {
            first_line.to_string()
        }
    });

    // Create session in project
    let session = match repository::create_session(
        &payload.project_id,
        &title,
        None,
        None,
        Some(&payload.chat_id),
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    // Insert initial context message derived from main chat turn
    let initial_content = format!(
        "Task derived from Main Chat:\n\n[User]: {}\n\n[Assistant]: {}",
        user_msg, assistant_msg
    );

    let _ = repository::create_message(
        &session.id,
        "user",
        Some(&initial_content),
        None,
        None,
        None,
        "success",
        None,
        None,
    )
    .await;

    (StatusCode::CREATED, Json(json!(session)))
}

// -----------------------------------------------------------------------------
// Messages Endpoints
// -----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateMessagePayload {
    pub role: String,
    pub content: Option<String>,
    pub thought: Option<String>,
    pub tool_calls: Option<serde_json::Value>,
    pub tool_call_id: Option<String>,
    pub status: Option<String>,
    pub token_usage_input: Option<i64>,
    pub token_usage_output: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListMessagesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub before_sequence_num: Option<i64>,
}

async fn list_messages(
    Path(session_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> impl IntoResponse {
    match repository::list_messages_paginated(
        &session_id,
        query.limit,
        query.offset,
        query.before_sequence_num,
    )
    .await
    {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_message(
    Path(session_id): Path<String>,
    Json(payload): Json<CreateMessagePayload>,
) -> impl IntoResponse {
    let tool_calls_str = payload
        .tool_calls
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    let status_str = payload.status.unwrap_or_else(|| "success".to_string());

    match repository::create_message(
        &session_id,
        &payload.role,
        payload.content.as_deref(),
        payload.thought.as_deref(),
        tool_calls_str.as_deref(),
        payload.tool_call_id.as_deref(),
        &status_str,
        payload.token_usage_input,
        payload.token_usage_output,
    )
    .await
    {
        Ok(m) => (StatusCode::CREATED, Json(json!(m))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

// -----------------------------------------------------------------------------
// Agent Skills Endpoints (Filesystem Dynamic Scanner: .skills / .skill)
// -----------------------------------------------------------------------------

async fn list_skills() -> impl IntoResponse {
    let list = crate::agent::skills::scan_project_skills(None, &[]);
    (StatusCode::OK, Json(json!(list)))
}

async fn get_project_skills(Path(project_id): Path<String>) -> impl IntoResponse {
    match repository::get_project(&project_id).await {
        Ok(Some(proj)) => {
            let folders = crate::agent::executor::parse_project_folders(proj.root_path.as_deref());
            let list = crate::agent::skills::scan_project_skills(proj.root_path.as_deref(), &folders);
            (StatusCode::OK, Json(json!(list)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Project not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
pub struct SetProjectMcpPayload {
    pub server_ids: Vec<String>,
}

async fn get_project_mcp_servers_handler(Path(project_id): Path<String>) -> impl IntoResponse {
    match repository::get_project_mcp_server_ids(&project_id).await {
        Ok(ids) => (StatusCode::OK, Json(json!(ids))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn set_project_mcp_servers_handler(
    Path(project_id): Path<String>,
    Json(payload): Json<SetProjectMcpPayload>,
) -> impl IntoResponse {
    match repository::set_project_mcp_servers(&project_id, &payload.server_ids).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "ok"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

// -----------------------------------------------------------------------------
// MCP Servers Endpoints
// -----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SaveMcpServerPayload {
    pub id: String,
    pub name: String,
    pub transport: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<serde_json::Value>,
    pub url: Option<String>,
    pub raw_json: Option<serde_json::Value>,
    pub is_enabled: Option<bool>,
}

async fn list_mcp_servers() -> impl IntoResponse {
    match repository::list_mcp_servers().await {
        Ok(list) => {
            let dtos: Vec<serde_json::Value> = list
                .into_iter()
                .map(|s| {
                    json!({
                        "id": s.id,
                        "name": s.name,
                        "config": s.config,
                        "transport": s.transport(),
                        "command": s.command(),
                        "args": s.args_json(),
                        "env": s.env_json(),
                        "url": s.url(),
                        "is_enabled": s.is_enabled,
                        "created_at": s.created_at,
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!(dtos)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn save_mcp_server(Json(payload): Json<SaveMcpServerPayload>) -> impl IntoResponse {
    let mut config_val = payload.raw_json.unwrap_or_else(|| json!({}));

    // If config_val is empty or not an object, build it from the other fields if provided
    if !config_val.is_object() || config_val.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        let mut obj = serde_json::Map::new();
        obj.insert("name".to_string(), json!(payload.name));
        if let Some(ref t) = payload.transport {
            obj.insert("transport".to_string(), json!(t));
        }
        if let Some(ref c) = payload.command {
            obj.insert("command".to_string(), json!(c));
        }
        if let Some(ref a) = payload.args {
            obj.insert("args".to_string(), json!(a));
        }
        if let Some(ref e) = payload.env {
            obj.insert("env".to_string(), e.clone());
        }
        if let Some(ref u) = payload.url {
            obj.insert("url".to_string(), json!(u));
        }
        config_val = serde_json::Value::Object(obj);
    } else if let Some(obj) = config_val.as_object_mut() {
        if !obj.contains_key("name") {
            obj.insert("name".to_string(), json!(payload.name));
        }
    }

    let config_str = serde_json::to_string(&config_val).unwrap_or_else(|_| "{}".to_string());
    let is_enabled = payload.is_enabled.unwrap_or(true);

    match repository::create_or_update_mcp_server(
        &payload.id,
        &payload.name,
        &config_str,
        is_enabled,
    )
    .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "saved"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn delete_mcp_server(Path(id): Path<String>) -> impl IntoResponse {
    match repository::delete_mcp_server(&id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "deleted"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn test_mcp_server_handler(Json(payload): Json<SaveMcpServerPayload>) -> impl IntoResponse {
    let mut config_val = payload.raw_json.unwrap_or_else(|| json!({}));
    if !config_val.is_object() || config_val.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        let mut obj = serde_json::Map::new();
        obj.insert("name".to_string(), json!(payload.name));
        if let Some(ref t) = payload.transport {
            obj.insert("transport".to_string(), json!(t));
        }
        if let Some(ref c) = payload.command {
            obj.insert("command".to_string(), json!(c));
        }
        if let Some(ref a) = payload.args {
            obj.insert("args".to_string(), json!(a));
        }
        if let Some(ref e) = payload.env {
            obj.insert("env".to_string(), e.clone());
        }
        if let Some(ref u) = payload.url {
            obj.insert("url".to_string(), json!(u));
        }
        config_val = serde_json::Value::Object(obj);
    }
    let config_str = serde_json::to_string(&config_val).unwrap_or_else(|_| "{}".to_string());
    let server = crate::agent::models::McpServer {
        id: payload.id.clone(),
        name: payload.name.clone(),
        config: config_str,
        is_enabled: 1,
        created_at: 0,
    };
    let config: crate::agent::mcp::McpServerConfig = (&server).into();
    let client = crate::agent::mcp::McpClient::new(config);
    match client.list_tools().await {
        Ok(tools) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "tools_count": tools.len(),
                "tools": tools
            })),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(json!({
                "ok": false,
                "error": e
            })),
        ),
    }
}

// -----------------------------------------------------------------------------
// Goals & Evidence Cache Endpoints
// -----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateGoalPayload {
    pub id: Option<String>,
    pub session_id: Option<String>,
    pub parent_goal_id: Option<String>,
    pub title: String,
    pub demand: String,
    pub success_criteria: Option<Vec<String>>,
    pub constraints: Option<Vec<String>>,
    pub reuse_keywords: Option<Vec<String>>,
}

async fn list_project_goals(Path(project_id): Path<String>) -> impl IntoResponse {
    match repository::list_project_goals(&project_id).await {
        Ok(goals) => (StatusCode::OK, Json(json!(goals))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_project_goal(
    Path(project_id): Path<String>,
    Json(payload): Json<CreateGoalPayload>,
) -> impl IntoResponse {
    let goal_id = payload.id.unwrap_or_else(|| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("G_{}", now)
    });

    let sc = payload.success_criteria.unwrap_or_default();
    let constraints = payload.constraints.unwrap_or_default();
    let kw = payload.reuse_keywords.unwrap_or_default();
    let kw_refs: Vec<&str> = kw.iter().map(|s| s.as_str()).collect();

    // Check project workspace roots for file revalidation
    let workspace_roots = match repository::get_project(&project_id).await {
        Ok(Some(p)) => crate::agent::executor::parse_project_folders(p.root_path.as_deref()),
        _ => Vec::new(),
    };

    match crate::agent::orchestrator::GoalOrchestrator::initialize_demand(
        &goal_id,
        &project_id,
        payload.session_id.as_deref(),
        payload.parent_goal_id.as_deref(),
        &payload.title,
        &payload.demand,
        &sc,
        &constraints,
        &kw_refs,
        &workspace_roots,
    )
    .await
    {
        Ok(goal) => (StatusCode::CREATED, Json(json!(goal))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_goal_details(Path(id): Path<String>) -> impl IntoResponse {
    match repository::get_goal(&id).await {
        Ok(Some(goal)) => (StatusCode::OK, Json(json!(goal))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Goal not found"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_goal_evidences(Path(id): Path<String>) -> impl IntoResponse {
    match repository::get_active_evidences_for_goal(&id).await {
        Ok(evidences) => (StatusCode::OK, Json(json!(evidences))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_goal_verifications_list(Path(id): Path<String>) -> impl IntoResponse {
    match repository::list_goal_verifications(&id).await {
        Ok(verifs) => (StatusCode::OK, Json(json!(verifs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
pub struct AuditQuery {
    #[serde(default = "default_audit_limit")]
    pub limit: i64,
}

fn default_audit_limit() -> i64 {
    100
}

async fn get_goal_audit_logs(
    Path(id): Path<String>,
    Query(q): Query<AuditQuery>,
) -> impl IntoResponse {
    match repository::list_goal_audit_logs(&id, q.limit).await {
        Ok(logs) => (StatusCode::OK, Json(json!(logs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn get_session_audit_logs(
    Path(id): Path<String>,
    Query(q): Query<AuditQuery>,
) -> impl IntoResponse {
    match repository::list_session_audit_logs(&id, q.limit).await {
        Ok(logs) => (StatusCode::OK, Json(json!(logs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
pub struct ConcludeGoalPayload {
    pub conclusion: String,
}

async fn conclude_goal_handler(
    Path(id): Path<String>,
    Json(payload): Json<ConcludeGoalPayload>,
) -> impl IntoResponse {
    let goal = match repository::get_goal(&id).await {
        Ok(Some(g)) => g,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Goal not found"}))),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    let orch = crate::agent::orchestrator::GoalOrchestrator::new(&goal.id, &goal.project_id);
    match orch.conclude(&payload.conclusion).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "completed"}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}
