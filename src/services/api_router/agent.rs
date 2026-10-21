use std::sync::{Arc, Mutex};
use axum::{
    extract::Path,
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
        .route("/api/agent/sessions/:id/events", get(session_events_sse))
        .route("/api/agent/sessions/from-chat", post(create_session_from_chat))
        // Messages
        .route("/api/agent/sessions/:id/messages", get(list_messages).post(create_message))
        // Prompt Skills
        .route("/api/agent/skills", get(list_skills).post(create_skill))
        .route("/api/agent/skills/:id/delete", post(delete_skill))
        .route("/api/agent/projects/:id/skills", get(get_project_skills).post(set_project_skills))
        // MCP Servers
        .route("/api/agent/mcp", get(list_mcp_servers).post(save_mcp_server))
        .route("/api/agent/mcp/:id/delete", post(delete_mcp_server))
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
            Json(json!({"valid": false, "error": "Path folder tidak boleh kosong"})),
        );
    }

    let p = std::path::Path::new(raw_path);
    if !p.exists() {
        return (
            StatusCode::OK,
            Json(json!({"valid": false, "error": format!("Folder tidak ditemukan di disk: {}", raw_path)})),
        );
    }
    if !p.is_dir() {
        return (
            StatusCode::OK,
            Json(json!({"valid": false, "error": format!("Path bukan merupakan direktori/folder: {}", raw_path)})),
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
            Json(json!({ "ok": false, "error": "Path tidak boleh kosong" })),
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
            return Err(format!("Folder tidak ditemukan di disk: {}", path_str));
        }
        if !p.is_dir() {
            return Err(format!("Path bukan direktori/folder: {}", path_str));
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
                Json(json!({"error": "Project tidak ditemukan"})),
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
                Json(json!({"error": "Project tidak ditemukan"})),
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
                Json(json!({"error": format!("Akses path ditolak: {}", e)})),
            )
        }
    };

    if !resolved.exists() || !resolved.is_file() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "File tidak ditemukan"})),
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
            Json(json!({"error": format!("Gagal membaca file: {}", e)})),
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
    // Spawn detached task in Tokio runtime, emitting real-time events via SSE
    tokio::spawn(async move {
        if let Err(e) = crate::agent::executor::execute_session_turn(&session_id, model_override.as_deref()).await {
            eprintln!("[Agent] Execution error for session {}: {}", session_id, e);
        }
    });

    (StatusCode::OK, Json(json!({"status": "started", "session_id": id})))
}

async fn session_events_sse(
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let sender = crate::agent::executor::get_session_sender(&session_id);
    let mut rx = sender.subscribe();

    let stream = async_stream::stream! {
        // Initial connection confirmation event
        let init_json = serde_json::json!({
            "type": "connected",
            "data": {
                "session_id": session_id,
            }
        });
        yield Ok::<_, std::convert::Infallible>(format!("data: {}\n\n", init_json));

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(evt) => {
                            if let Ok(json_str) = serde_json::to_string(&evt) {
                                yield Ok(format!("data: {}\n\n", json_str));
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
                    yield Ok(":\n\n".to_string());
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
            Json(json!({"error": "Percakapan chat tidak ditemukan"})),
        );
    };

    let title = payload.title.unwrap_or_else(|| {
        let first_line = user_msg.lines().next().unwrap_or("Task dari Chat").trim();
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
        "Task diturunkan dari Main Chat:\n\n[User]: {}\n\n[Assistant]: {}",
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

async fn list_messages(Path(session_id): Path<String>) -> impl IntoResponse {
    match repository::list_messages(&session_id).await {
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
// Prompt Skills Endpoints
// -----------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateSkillPayload {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub tags: Option<Vec<String>>,
}

async fn list_skills() -> impl IntoResponse {
    match repository::list_prompt_skills().await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn create_skill(Json(payload): Json<CreateSkillPayload>) -> impl IntoResponse {
    let desc = payload.description.unwrap_or_default();
    let tags_json = serde_json::to_string(&payload.tags.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());

    match repository::create_prompt_skill(
        &payload.name,
        &payload.slug,
        &desc,
        &payload.system_prompt,
        &tags_json,
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

async fn delete_skill(Path(id): Path<String>) -> impl IntoResponse {
    match repository::delete_prompt_skill(&id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "deleted"}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

#[derive(Deserialize)]
pub struct SetProjectSkillsPayload {
    pub skill_ids: Vec<String>,
}

async fn get_project_skills(Path(project_id): Path<String>) -> impl IntoResponse {
    match repository::get_project_skills(&project_id).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn set_project_skills(
    Path(project_id): Path<String>,
    Json(payload): Json<SetProjectSkillsPayload>,
) -> impl IntoResponse {
    match repository::set_project_skills(&project_id, &payload.skill_ids).await {
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
    pub is_enabled: Option<bool>,
}

async fn list_mcp_servers() -> impl IntoResponse {
    match repository::list_mcp_servers().await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn save_mcp_server(Json(payload): Json<SaveMcpServerPayload>) -> impl IntoResponse {
    let transport = payload.transport.unwrap_or_else(|| "stdio".to_string());
    let args_str = serde_json::to_string(&payload.args.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
    let env_str = serde_json::to_string(&payload.env.unwrap_or_else(|| json!({}))).unwrap_or_else(|_| "{}".to_string());
    let is_enabled = payload.is_enabled.unwrap_or(true);

    match repository::create_or_update_mcp_server(
        &payload.id,
        &payload.name,
        &transport,
        payload.command.as_deref(),
        &args_str,
        &env_str,
        payload.url.as_deref(),
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
