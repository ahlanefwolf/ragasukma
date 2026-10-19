use std::sync::{Arc, Mutex};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::ServerSharedState;

pub fn register_automation_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    router
        // Skills
        .route("/api/skills", get(axum_get_skills).post(axum_post_skills))
        .route("/api/skills/delete", post(axum_delete_skill))
        .route("/api/skills/run", post(axum_run_skill))
        // Todos
        .route("/api/todos", get(axum_get_todos).post(axum_post_todo))
        .route("/api/todos/complete", post(axum_complete_todo))
        .route("/api/todos/delete", post(axum_delete_todo))
        // Recipes
        .route("/api/recipes", get(axum_get_recipes))
        .route("/api/recipes/save", post(axum_save_recipe))
        .route("/api/recipes/delete", post(axum_delete_recipe))
        .route("/api/recipes/run", post(axum_run_recipe))
        // Browser Automation & DOM
        .route("/api/browser/snapshot", post(axum_browser_snapshot))
        .route("/api/browser/copied_text", post(axum_browser_copied_text))
        .route("/api/browser/tabs", get(axum_get_browser_tabs).post(axum_post_browser_tabs))
        .route("/api/browser/command/next", get(axum_get_browser_command_next))
        // Cloudflared Tunnel & Remote
        .route("/api/remote/tunnel/status", get(axum_tunnel_status))
        .route("/api/remote/tunnel/check", get(axum_tunnel_check))
        .route("/api/remote/tunnel/install", post(axum_tunnel_install))
        .route("/api/remote/tunnel/start", post(axum_tunnel_start))
        .route("/api/remote/tunnel/stop", post(axum_tunnel_stop))
        .route("/api/remote/active", post(axum_remote_active))
        .route("/api/remote/text", post(axum_remote_text))
}

// -----------------------------------------------------------------------------
// 1. Skills Service
// -----------------------------------------------------------------------------
pub(crate) async fn axum_get_skills() -> impl IntoResponse {
    let skills = crate::engine::skills::sync_skills_from_db().await;
    Json(skills)
}

#[derive(Deserialize)]
pub(crate) struct AddSkillPayload {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    pub(crate) command: String,
}

pub(crate) async fn axum_post_skills(Json(item): Json<AddSkillPayload>) -> impl IntoResponse {
    let skill = crate::engine::SkillItem::new(
        item.name,
        item.description,
        item.command,
    );
    match crate::engine::skills::save_skill_to_db(&skill).await {
        Ok(_) => Json(json!({ "success": true })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct DeleteSkillPayload {
    pub(crate) name: String,
}

pub(crate) async fn axum_delete_skill(Json(item): Json<DeleteSkillPayload>) -> impl IntoResponse {
    match crate::engine::skills::delete_skill_from_db(&item.name).await {
        Ok(_) => Json(json!({ "success": true })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

// -----------------------------------------------------------------------------
// 1b. Todos Service
// -----------------------------------------------------------------------------
pub(crate) async fn axum_get_todos() -> impl IntoResponse {
    let todos = crate::engine::todo::sync_todos_from_db().await;
    Json(todos)
}

#[derive(Deserialize)]
pub(crate) struct AddTodoPayload {
    pub(crate) task: String,
}

pub(crate) async fn axum_post_todo(Json(item): Json<AddTodoPayload>) -> impl IntoResponse {
    let task = item.task.trim();
    if task.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Task must not be empty" })),
        )
            .into_response();
    }
    match crate::engine::todo::add_task(task) {
        Ok(msg) => Json(json!({ "success": true, "message": msg })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct CompleteTodoPayload {
    #[serde(default)]
    pub(crate) query: String,
}

pub(crate) async fn axum_complete_todo(
    Json(payload): Json<CompleteTodoPayload>,
) -> impl IntoResponse {
    match crate::engine::todo::complete_task(&payload.query) {
        Ok(msg) => Json(json!({ "success": true, "message": msg })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct DeleteTodoPayload {
    pub(crate) id: i64,
}

pub(crate) async fn axum_delete_todo(Json(payload): Json<DeleteTodoPayload>) -> impl IntoResponse {
    let removed = crate::engine::todo::delete_task(payload.id);
    Json(json!({ "success": removed })).into_response()
}

#[derive(Deserialize)]
pub(crate) struct RunSkillPayload {
    #[serde(alias = "cmd")]
    pub(crate) command: String,
}

pub(crate) async fn axum_run_skill(Json(payload): Json<RunSkillPayload>) -> impl IntoResponse {
    let cmd = payload.command.trim();
    if let Some((name, res)) = crate::engine::execute_skill_by_identifier(cmd) {
        println!("[Skill Run UI] Executing internal skill '{}' -> {}", name, res);
        return Json(json!({ "success": true, "message": res })).into_response();
    }
    if let Err(e) = crate::engine::execute_command_by_os(cmd) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response()
    } else {
        Json(json!({ "success": true })).into_response()
    }
}

// -----------------------------------------------------------------------------
// 2. Recipes Service
// -----------------------------------------------------------------------------
pub(crate) async fn axum_get_recipes() -> impl IntoResponse {
    Json(crate::engine::actions::recipe::load_all_recipes())
}

#[derive(Deserialize)]
pub(crate) struct SaveRecipePayload {
    #[serde(flatten)]
    pub(crate) recipe: crate::engine::actions::recipe::AutomationRecipe,
    #[serde(default, alias = "previousId")]
    pub(crate) previous_id: Option<String>,
}

pub(crate) async fn axum_save_recipe(Json(mut payload): Json<SaveRecipePayload>) -> impl IntoResponse {
    let recipe_id = payload.recipe.recipe_id.trim().to_string();
    if recipe_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Recipe ID cannot be empty" })),
        )
            .into_response();
    }

    for (idx, step) in payload.recipe.steps.iter_mut().enumerate() {
        step.step = idx + 1;
    }

    let dir = crate::engine::actions::recipe::get_recipes_dir();
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join(format!("{recipe_id}.json"));

    if let Some(prev) = payload.previous_id {
        let prev_clean = prev.trim();
        if !prev_clean.is_empty() && prev_clean != recipe_id {
            let old_path = dir.join(format!("{prev_clean}.json"));
            if old_path.exists() {
                let _ = std::fs::remove_file(old_path);
                crate::term_log!(
                    "[Recipe Service] Renamed recipe from '{}' -> '{}'",
                    prev_clean,
                    recipe_id
                );
            }
        }
    }

    match serde_json::to_string_pretty(&payload.recipe) {
        Ok(formatted) => {
            if let Err(e) = std::fs::write(&file_path, formatted) {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Failed to save recipe: {e}") })),
                )
                    .into_response()
            } else {
                crate::term_log!(
                    "[Recipe Service] Saved recipe '{}' with {} steps",
                    recipe_id,
                    payload.recipe.steps.len()
                );
                Json(json!({ "success": true })).into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to serialize recipe: {e}") })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct DeleteRecipePayload {
    #[serde(alias = "recipeId")]
    pub(crate) recipe_id: String,
}

pub(crate) async fn axum_delete_recipe(Json(item): Json<DeleteRecipePayload>) -> impl IntoResponse {
    let clean_id = item.recipe_id.trim();
    let file_path = crate::engine::actions::recipe::get_recipes_dir().join(format!("{clean_id}.json"));
    if file_path.exists() {
        let _ = std::fs::remove_file(&file_path);
        crate::term_log!("[Recipe Service] Deleted recipe '{}'", clean_id);
        Json(json!({ "success": true })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Recipe not found" })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
pub(crate) struct RunRecipePayload {
    #[serde(alias = "recipeId")]
    pub(crate) recipe_id: String,
    #[serde(default)]
    pub(crate) params: Option<serde_json::Value>,
}

pub(crate) async fn axum_run_recipe(Json(payload): Json<RunRecipePayload>) -> impl IntoResponse {
    let clean_id = payload.recipe_id.trim();
    if clean_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Recipe ID cannot be empty" })),
        )
            .into_response();
    }

    let payload_str = if let Some(params) = payload.params {
        if params.as_object().is_some_and(|obj| !obj.is_empty()) {
            format!("{clean_id}|{}", params)
        } else {
            clean_id.to_string()
        }
    } else {
        clean_id.to_string()
    };

    if let Some((title, msg)) = crate::engine::actions::recipe::execute_recipe(&payload_str) {
        crate::term_log!("[Recipe Service] Run result: {} — {}", title, msg);
        Json(json!({
            "success": true,
            "title": title,
            "message": msg
        }))
        .into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Failed to execute recipe or recipe not found" })),
        )
            .into_response()
    }
}

// -----------------------------------------------------------------------------
// 3. Browser Automation & DOM
// -----------------------------------------------------------------------------
pub(crate) async fn axum_browser_snapshot(body: String) -> impl IntoResponse {
    let (resp_body, _) = crate::services::browser_session_service::handle_post_browser_snapshot(&body);
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(resp_body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub(crate) async fn axum_browser_copied_text(body: String) -> impl IntoResponse {
    let (resp_body, _) = crate::services::browser_session_service::handle_post_browser_copied_text(&body);
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(resp_body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub(crate) async fn axum_get_browser_tabs() -> impl IntoResponse {
    Json(crate::services::browser_session_service::get_browser_tabs())
}

pub(crate) async fn axum_post_browser_tabs(body: String) -> impl IntoResponse {
    let (resp_body, _) = crate::services::browser_session_service::handle_post_browser_tabs(&body);
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(resp_body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub(crate) async fn axum_get_browser_command_next() -> impl IntoResponse {
    if let Some(cmd) = crate::services::browser_session_service::get_next_browser_command() {
        Json(json!({
            "has_command": true,
            "command": cmd
        }))
        .into_response()
    } else {
        Json(json!({ "has_command": false })).into_response()
    }
}

// -----------------------------------------------------------------------------
// 4. Cloudflared Tunnel & Remote
// -----------------------------------------------------------------------------
pub(crate) async fn axum_tunnel_status() -> impl IntoResponse {
    let (status, url, auth_key) = crate::engine::cloudflared::get_tunnel_status();
    let cf_info = crate::engine::cloudflared::check_cloudflared();
    Json(json!({
        "status": status,
        "url": url,
        "auth_key": auth_key,
        "cloudflared": cf_info
    }))
}

pub(crate) async fn axum_tunnel_check() -> impl IntoResponse {
    Json(crate::engine::cloudflared::check_cloudflared())
}

pub(crate) async fn axum_tunnel_install() -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(crate::engine::cloudflared::install_cloudflared).await;
    match res {
        Ok(Ok(msg)) => {
            let info = crate::engine::cloudflared::check_cloudflared();
            Json(json!({
                "success": true,
                "message": msg,
                "info": info
            }))
            .into_response()
        }
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": err })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_tunnel_start() -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(|| {
        crate::engine::cloudflared::start_cloudflared_tunnel(42128)
    })
    .await;

    match res {
        Ok(Ok((url, auth_key))) => {
            let status = if url.is_empty() { "starting" } else { "running" };
            Json(json!({
                "status": status,
                "url": if url.is_empty() { None } else { Some(url) },
                "auth_key": auth_key
            }))
            .into_response()
        }
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_tunnel_stop() -> impl IntoResponse {
    let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
    Json(json!({ "status": "stopped" }))
}

pub(crate) async fn axum_remote_active() -> impl IntoResponse {
    crate::engine::os_efficiency::notify_active();
    Json(json!({ "success": true, "message": "Remote heartbeat updated" }))
}

#[derive(Deserialize)]
pub(crate) struct RemoteTextQuery {
    #[serde(default)]
    pub(crate) key: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct RemoteTextPayload {
    pub(crate) text: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) device_id: Option<String>,
}

pub(crate) async fn axum_remote_text(
    Query(q): Query<RemoteTextQuery>,
    Json(payload): Json<RemoteTextPayload>,
) -> impl IntoResponse {
    if !crate::engine::cloudflared::is_auth_key_valid(q.key.as_deref().unwrap_or("")) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Unauthorized: invalid auth_key" })),
        )
            .into_response();
    }

    let text = payload.text.trim().to_string();
    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Empty text" })),
        )
            .into_response();
    }

    crate::engine::remote_text::inject_remote_text(text);
    Json(json!({ "success": true }))
        .into_response()
}
