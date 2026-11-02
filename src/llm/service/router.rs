use std::sync::{Arc, Mutex};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::app_state::ServerSharedState;

pub fn register_llm_routes(
    router: Router<Arc<Mutex<ServerSharedState>>>,
) -> Router<Arc<Mutex<ServerSharedState>>> {
    router
        // User-defined OpenAI-compatible providers
        .route("/api/llm/custom/accounts", get(axum_custom_accounts).post(axum_custom_save_accounts))
        .route("/api/llm/custom/accounts/delete", post(axum_custom_delete_account))
        // LLM Providers - Antigravity
        .route("/api/llm/antigravity/models", get(axum_ag_models))
        .route("/api/llm/antigravity/accounts", get(axum_ag_accounts).post(axum_ag_add_account))
        .route("/api/llm/antigravity/accounts/delete", post(axum_ag_delete_account))
        .route("/api/llm/antigravity/accounts/refresh", post(axum_ag_refresh_account))
        .route("/api/llm/antigravity/accounts/toggle", post(axum_ag_toggle_account))
        .route("/api/llm/antigravity/accounts/quota", get(axum_ag_quota_get).post(axum_ag_quota_post))
        .route("/api/llm/antigravity/oauth/url", get(axum_ag_oauth_url))
        .route("/api/llm/antigravity/oauth/callback", get(axum_ag_oauth_callback))
        .route("/api/llm/antigravity/oauth/exchange", post(axum_ag_oauth_exchange))
        // LLM Providers - Codex
        .route("/api/llm/codex/models", get(axum_cx_models))
        .route("/api/llm/codex/accounts", get(axum_cx_accounts).post(axum_cx_add_account))
        .route("/api/llm/codex/accounts/delete", post(axum_cx_delete_account))
        .route("/api/llm/codex/accounts/refresh", post(axum_cx_refresh_account))
        .route("/api/llm/codex/accounts/toggle", post(axum_cx_toggle_account))
        .route("/api/llm/codex/accounts/quota", get(axum_cx_quota_get).post(axum_cx_quota_post))
        .route("/api/llm/codex/oauth/url", get(axum_cx_oauth_url))
        .route("/api/llm/codex/oauth/callback", get(super::codex_service::axum_codex_oauth_callback))
        .route("/api/llm/codex/oauth/exchange", post(axum_cx_oauth_exchange))
        // LLM Providers - Claude (Anthropic)
        .route("/api/llm/claude/models", get(axum_cc_models))
        .route("/api/llm/claude/accounts", get(axum_cc_accounts).post(axum_cc_add_account))
        .route("/api/llm/claude/accounts/delete", post(axum_cc_delete_account))
        .route("/api/llm/claude/accounts/refresh", post(axum_cc_refresh_account))
        .route("/api/llm/claude/accounts/toggle", post(axum_cc_toggle_account))
        .route("/api/llm/claude/accounts/quota", get(axum_cc_quota_get).post(axum_cc_quota_post))
        .route("/api/llm/claude/oauth/url", get(axum_cc_oauth_url))
        .route("/api/llm/claude/oauth/callback", get(super::claude_service::axum_claude_oauth_callback))
        .route("/api/llm/claude/oauth/exchange", post(axum_cc_oauth_exchange))
        // LLM Token Usage
        .route("/api/llm/usage/summary", get(axum_usage_summary))
        .route("/api/llm/usage/records", get(axum_usage_records))
        .route("/api/llm/usage/clear", post(axum_usage_clear))
}

// -----------------------------------------------------------------------------
// User-defined OpenAI-compatible providers
// -----------------------------------------------------------------------------
#[derive(Deserialize)]
pub(crate) struct CustomAccountsPayload {
    accounts: Vec<crate::llm::providers::custom::CustomLlmAccount>,
}

#[derive(Deserialize)]
pub(crate) struct CustomPrefixPayload {
    prefix: String,
}

pub(crate) async fn axum_custom_accounts() -> impl IntoResponse {
    let accounts = crate::llm::providers::custom::load_accounts();
    Json(accounts)
}

pub(crate) async fn axum_custom_save_accounts(Json(payload): Json<CustomAccountsPayload>) -> impl IntoResponse {
    let mut prefixes = std::collections::HashSet::new();
    for account in &payload.accounts {
        if let Err(error) = crate::llm::providers::custom::validate_account(account) {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": error.to_string()}))).into_response();
        }
        if !prefixes.insert(account.prefix.to_lowercase()) {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Prefixes must be unique"}))).into_response();
        }
    }
    match crate::llm::providers::custom::save_accounts(&payload.accounts) {
        Ok(()) => Json(json!({"status":"success"})).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":error.to_string()}))).into_response(),
    }
}

pub(crate) async fn axum_custom_delete_account(Json(payload): Json<CustomPrefixPayload>) -> impl IntoResponse {
    let mut accounts = crate::llm::providers::custom::load_accounts();
    let previous = accounts.len();
    accounts.retain(|account| !account.prefix.eq_ignore_ascii_case(&payload.prefix));
    if accounts.len() == previous {
        return (StatusCode::NOT_FOUND, Json(json!({"error":"Account not found"}))).into_response();
    }
    match crate::llm::providers::custom::save_accounts(&accounts) {
        Ok(()) => Json(json!({"status":"success"})).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":error.to_string()}))).into_response(),
    }
}

// -----------------------------------------------------------------------------
// 1. LLM Providers - Antigravity
// -----------------------------------------------------------------------------
pub(crate) async fn axum_ag_models() -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(|| {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.get_dynamic_available_models()
    })
    .await;

    match res {
        Ok(Ok(models)) => Json(json!({
            "status": "success",
            "models": models
        }))
        .into_response(),
        Ok(Err(e)) => {
            if e.to_string().contains("No accounts available") {
                Json(json!({
                    "status": "success",
                    "models": []
                }))
                .into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "error",
                        "error": e.to_string()
                    })),
                )
                    .into_response()
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_ag_accounts() -> impl IntoResponse {
    let accounts = tokio::task::spawn_blocking(|| {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        let _ = provider.load_accounts();
        provider.list_accounts()
    })
    .await
    .unwrap_or_default();
    Json(accounts)
}

#[derive(Deserialize)]
pub(crate) struct AddAgAccountPayload {
    #[serde(alias = "refreshToken", alias = "token")]
    pub(crate) refresh_token: String,
    #[serde(default)]
    pub(crate) email: Option<String>,
    #[serde(default, alias = "projectId")]
    pub(crate) project_id: Option<String>,
}

pub(crate) async fn axum_ag_add_account(Json(payload): Json<AddAgAccountPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.add_account(&payload.refresh_token, payload.email, payload.project_id)
    })
    .await;

    match res {
        Ok(Ok(id)) => Json(json!({ "status": "success", "id": id })).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct IdPayload {
    pub(crate) id: String,
}

pub(crate) async fn axum_ag_delete_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let removed = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.remove_account(&payload.id)
    })
    .await
    .unwrap_or(false);

    if removed {
        Json(json!({ "status": "success" })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "error", "error": "Account not found" })),
        )
            .into_response()
    }
}

pub(crate) async fn axum_ag_refresh_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.refresh_account_by_id(&payload.id)
    })
    .await;

    match res {
        Ok(Ok(_)) => Json(json!({ "status": "success" })).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_ag_toggle_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.toggle_account_by_id(&payload.id)
    })
    .await;

    match res {
        Ok(Ok(active)) => Json(json!({ "status": "success", "is_active": active })).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct QuotaQuery {
    #[serde(default, alias = "accountId", alias = "account_id")]
    pub(crate) id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct QuotaPayload {
    #[serde(default, alias = "accountId", alias = "account_id")]
    pub(crate) id: Option<String>,
}

pub(crate) async fn axum_ag_quota_get(Query(q): Query<QuotaQuery>) -> impl IntoResponse {
    let account_id = q.id.unwrap_or_default();
    if account_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing account id" })),
        )
            .into_response();
    }
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.get_account_quota(&account_id)
    })
    .await;

    match res {
        Ok(Ok(val)) => Json(val).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_ag_quota_post(
    Query(q): Query<QuotaQuery>,
    body: Option<Json<QuotaPayload>>,
) -> impl IntoResponse {
    let account_id = q.id.or_else(|| body.and_then(|b| b.id.clone())).unwrap_or_default();
    if account_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing account id" })),
        )
            .into_response();
    }
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.get_account_quota(&account_id)
    })
    .await;

    match res {
        Ok(Ok(val)) => Json(val).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct OpenBrowserQuery {
    #[serde(default)]
    pub(crate) open: Option<String>,
}

pub(crate) async fn axum_ag_oauth_url(Query(q): Query<OpenBrowserQuery>) -> impl IntoResponse {
    let state = format!(
        "ragasukma_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    let auth_url = crate::llm::providers::antigravity::oauth::build_auth_url(
        crate::llm::providers::antigravity::constants::ANTIGRAVITY_OAUTH_REDIRECT_URI,
        &state,
    );
    if q.open.as_deref() == Some("1") {
        crate::assets::open_browser(&auth_url);
    }
    Json(json!({
        "status": "success",
        "auth_url": auth_url,
        "redirect_uri": crate::llm::providers::antigravity::constants::ANTIGRAVITY_OAUTH_REDIRECT_URI
    }))
}

#[derive(Deserialize)]
pub(crate) struct AgOAuthQuery {
    #[serde(default)]
    pub(crate) code: Option<String>,
    #[serde(default)]
    pub(crate) error: Option<String>,
}

pub(crate) async fn axum_ag_oauth_callback(Query(q): Query<AgOAuthQuery>) -> impl IntoResponse {
    if let Some(err_msg) = q.error {
        let html = format!(
            r#"<!DOCTYPE html><html><head><title>OAuth Failed</title><meta charset="utf-8"></head><body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;"><div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;"><h2>Google Authorization Failed</h2><p style="color: #94a3b8;">{}</p><a href="http://localhost:42128" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a></div></body></html>"#,
            err_msg
        );
        return (StatusCode::BAD_REQUEST, Html(html)).into_response();
    }

    let auth_code = match q.code {
        Some(c) if !c.is_empty() => c,
        _ => {
            let html = r#"<!DOCTYPE html><html><head><title>Invalid Code</title><meta charset="utf-8"></head><body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;"><div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;"><h2>Authorization Code Not Found</h2><p style="color: #94a3b8;">Google did not return a valid authorization code.</p><a href="http://localhost:42128" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a></div></body></html>"#;
            return (StatusCode::BAD_REQUEST, Html(html)).into_response();
        }
    };

    let redirect_uri = crate::llm::providers::antigravity::constants::ANTIGRAVITY_OAUTH_REDIRECT_URI;
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        let result = provider.add_oauth_account(&auth_code, redirect_uri);
        match result {
            Ok(id) => {
                let email = provider
                    .accounts
                    .iter()
                    .find(|a| a.id == id)
                    .and_then(|a| a.email.clone())
                    .unwrap_or_else(|| id.clone());
                Ok(email)
            }
            Err(e) => Err(e.to_string()),
        }
    })
    .await;

    match res {
        Ok(Ok(email)) => {
            crate::app_state::push_client_action("antigravity:oauth_success", Some(&email));
            let html = format!(
                r#"<!DOCTYPE html><html><head><title>Login Success</title><meta charset="utf-8"><style>body {{ font-family: system-ui, sans-serif; background: #111; color: #eee; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }} .box {{ text-align: center; padding: 24px; }} button {{ margin-top: 16px; padding: 8px 16px; cursor: pointer; }}</style></head><body><div class="box"><p>Login success: <b>{}</b></p><button onclick="window.close()">Close</button></div><script>try {{ window.close(); }} catch (e) {{}} setTimeout(() => {{ try {{ window.close(); }} catch (e) {{}} }}, 300);</script></body></html>"#,
                email
            );
            (StatusCode::OK, Html(html)).into_response()
        }
        Ok(Err(e)) => {
            let html = format!(
                r#"<!DOCTYPE html><html><head><title>OAuth Failed</title><meta charset="utf-8"></head><body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;"><div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;"><h2>Failed to Connect Account</h2><p style="color: #94a3b8;">{}</p><a href="http://localhost:42128" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a></div></body></html>"#,
                e
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
        }
        Err(e) => {
            let html = format!(
                r#"<!DOCTYPE html><html><head><title>OAuth Failed</title><meta charset="utf-8"></head><body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;"><div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;"><h2>Failed to Connect Account</h2><p style="color: #94a3b8;">{}</p><a href="http://localhost:42128" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a></div></body></html>"#,
                e
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct ExchangeAgCodePayload {
    pub(crate) code: String,
    #[serde(default)]
    pub(crate) redirect_uri: Option<String>,
}

pub(crate) async fn axum_ag_oauth_exchange(Json(payload): Json<ExchangeAgCodePayload>) -> impl IntoResponse {
    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| crate::llm::providers::antigravity::constants::ANTIGRAVITY_OAUTH_REDIRECT_URI.to_string());
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::antigravity::AntigravityProvider::new();
        provider.add_oauth_account(&payload.code, &redirect_uri)
    })
    .await;

    match res {
        Ok(Ok(id)) => Json(json!({ "status": "success", "id": id })).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("Failed to exchange code: {e}") })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Task join error: {e}") })),
        )
            .into_response(),
    }
}

// -----------------------------------------------------------------------------
// 2. LLM Providers - Codex
// -----------------------------------------------------------------------------
pub(crate) async fn axum_cx_models() -> impl IntoResponse {
    let list = tokio::task::spawn_blocking(|| {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        provider.get_dynamic_available_models().unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    Json(list)
}

pub(crate) async fn axum_cx_accounts() -> impl IntoResponse {
    let accounts = tokio::task::spawn_blocking(|| {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        let _ = provider.load_accounts();
        provider.list_account_infos()
    })
    .await
    .unwrap_or_default();
    Json(accounts)
}

#[derive(Deserialize)]
pub(crate) struct AddCodexAccountPayload {
    #[serde(alias = "refreshToken", alias = "token")]
    pub(crate) refresh_token: String,
    #[serde(default)]
    pub(crate) email: Option<String>,
    #[serde(default, alias = "accountId", alias = "chatgptAccountId")]
    pub(crate) chatgpt_account_id: Option<String>,
}

pub(crate) async fn axum_cx_add_account(Json(payload): Json<AddCodexAccountPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        provider.add_account(&payload.refresh_token, payload.email, payload.chatgpt_account_id)
    })
    .await;

    match res {
        Ok(Ok(id)) => Json(json!({ "status": "success", "id": id })).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cx_delete_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let removed = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        provider.remove_account(&payload.id)
    })
    .await
    .unwrap_or(false);

    if removed {
        Json(json!({ "status": "success" })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "error", "error": "Account not found" })),
        )
            .into_response()
    }
}

pub(crate) async fn axum_cx_refresh_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        provider.refresh_account_by_id(&payload.id)
    })
    .await;

    match res {
        Ok(Ok(_)) => Json(json!({ "status": "success" })).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cx_toggle_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        provider.toggle_account_by_id(&payload.id)
    })
    .await;

    match res {
        Ok(Ok(active)) => Json(json!({ "status": "success", "is_active": active })).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cx_quota_get(Query(q): Query<QuotaQuery>) -> impl IntoResponse {
    let account_id = q.id.unwrap_or_default();
    if account_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing account id" })),
        )
            .into_response();
    }
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        provider.get_account_quota(&account_id)
    })
    .await;

    match res {
        Ok(Ok(val)) => Json(val).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cx_quota_post(
    Query(q): Query<QuotaQuery>,
    body: Option<Json<QuotaPayload>>,
) -> impl IntoResponse {
    let account_id = q.id.or_else(|| body.and_then(|b| b.id.clone())).unwrap_or_default();
    if account_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing account id" })),
        )
            .into_response();
    }
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        provider.get_account_quota(&account_id)
    })
    .await;

    match res {
        Ok(Ok(val)) => Json(val).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cx_oauth_url(Query(q): Query<OpenBrowserQuery>) -> impl IntoResponse {
    super::codex_service::ensure_port_1455_listener();

    let (code_verifier, code_challenge) = crate::llm::providers::codex::oauth::generate_pkce();
    let state = format!(
        "cx_state_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        std::process::id()
    );

    crate::llm::providers::codex::oauth::store_oauth_state(&state, &code_verifier);

    let auth_url = crate::llm::providers::codex::oauth::build_auth_url(
        super::codex_service::CODEX_OAUTH_REDIRECT_URI,
        &state,
        &code_challenge,
    );

    if q.open.as_deref() == Some("1") {
        crate::assets::open_browser(&auth_url);
    }

    Json(json!({
        "status": "success",
        "auth_url": auth_url,
        "redirect_uri": super::codex_service::CODEX_OAUTH_REDIRECT_URI,
        "state": state
    }))
}

#[derive(Deserialize)]
pub(crate) struct ExchangeCodexCodePayload {
    pub(crate) code: String,
    #[serde(default)]
    pub(crate) code_verifier: Option<String>,
    #[serde(default)]
    pub(crate) redirect_uri: Option<String>,
    #[serde(default)]
    pub(crate) state: Option<String>,
}

pub(crate) async fn axum_cx_oauth_exchange(Json(payload): Json<ExchangeCodexCodePayload>) -> impl IntoResponse {
    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| super::codex_service::CODEX_OAUTH_REDIRECT_URI.to_string());

    let verifier = if let Some(v) = payload.code_verifier {
        v
    } else if let Some(st) = payload.state.as_deref() {
        crate::llm::providers::codex::oauth::take_oauth_state(st).unwrap_or_default()
    } else {
        String::new()
    };

    if verifier.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing code_verifier or state not found" })),
        )
            .into_response();
    }

    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::codex::CodexProvider::new();
        match crate::llm::providers::codex::oauth::complete_oauth_flow(&provider.client, &payload.code, &redirect_uri, &verifier) {
            Ok(account) => match provider.add_oauth_account(account) {
                Ok(id) => {
                    crate::app_state::push_client_action("codex:oauth_success", Some(&id));
                    Ok(id)
                }
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            },
            Err(e) => Err((StatusCode::BAD_REQUEST, format!("OAuth exchange failed: {e}"))),
        }
    })
    .await;

    match res {
        Ok(Ok(id)) => Json(json!({ "status": "success", "id": id })).into_response(),
        Ok(Err((status, msg))) => (
            status,
            Json(json!({ "status": "error", "error": msg })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

// -----------------------------------------------------------------------------
// 3. LLM Providers - Claude (Anthropic)
// -----------------------------------------------------------------------------

pub(crate) async fn axum_cc_models() -> impl IntoResponse {
    let list = tokio::task::spawn_blocking(|| {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        provider.get_dynamic_available_models().unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    Json(list)
}

pub(crate) async fn axum_cc_accounts() -> impl IntoResponse {
    let accounts = tokio::task::spawn_blocking(|| {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        let _ = provider.load_accounts();
        provider.list_account_infos()
    })
    .await
    .unwrap_or_default();
    Json(accounts)
}

#[derive(Deserialize)]
pub(crate) struct AddClaudeAccountPayload {
    pub(crate) refresh_token: String,
    #[serde(default)]
    pub(crate) email: Option<String>,
    #[serde(default)]
    pub(crate) org_id: Option<String>,
}

pub(crate) async fn axum_cc_add_account(Json(payload): Json<AddClaudeAccountPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        provider.add_account(&payload.refresh_token, payload.email, payload.org_id)
    })
    .await;

    match res {
        Ok(Ok(id)) => Json(json!({ "status": "success", "id": id })).into_response(),
        Ok(Err(e)) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cc_delete_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let removed = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        provider.remove_account(&payload.id)
    })
    .await
    .unwrap_or(false);

    if removed {
        Json(json!({ "status": "success" })).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "status": "error", "error": "Account not found" })),
        )
            .into_response()
    }
}

pub(crate) async fn axum_cc_refresh_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        provider.refresh_account_by_id(&payload.id)
    })
    .await;

    match res {
        Ok(Ok(_)) => Json(json!({ "status": "success" })).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cc_toggle_account(Json(payload): Json<IdPayload>) -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        provider.toggle_account_by_id(&payload.id)
    })
    .await;

    match res {
        Ok(Ok(active)) => Json(json!({ "status": "success", "is_active": active })).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cc_quota_get(Query(q): Query<QuotaQuery>) -> impl IntoResponse {
    let account_id = q.id.unwrap_or_default();
    if account_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing account id" })),
        )
            .into_response();
    }
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        provider.get_account_quota(&account_id)
    })
    .await;

    match res {
        Ok(Ok(val)) => Json(val).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cc_quota_post(
    Query(q): Query<QuotaQuery>,
    body: Option<Json<QuotaPayload>>,
) -> impl IntoResponse {
    let account_id = q.id.or_else(|| body.and_then(|b| b.id.clone())).unwrap_or_default();
    if account_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing account id" })),
        )
            .into_response();
    }
    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        provider.get_account_quota(&account_id)
    })
    .await;

    match res {
        Ok(Ok(val)) => Json(val).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub(crate) async fn axum_cc_oauth_url(Query(q): Query<OpenBrowserQuery>) -> impl IntoResponse {
    super::claude_service::ensure_port_54321_listener();

    let (code_verifier, code_challenge) = crate::llm::providers::claude::oauth::generate_pkce();
    let state = format!(
        "cc_state_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        std::process::id()
    );

    crate::llm::providers::claude::oauth::store_oauth_state(&state, &code_verifier);

    let auth_url = crate::llm::providers::claude::oauth::build_auth_url(
        super::claude_service::CLAUDE_OAUTH_REDIRECT_URI,
        &state,
        &code_challenge,
    );

    if q.open.as_deref() == Some("1") {
        crate::assets::open_browser(&auth_url);
    }

    Json(json!({
        "status": "success",
        "auth_url": auth_url,
        "redirect_uri": super::claude_service::CLAUDE_OAUTH_REDIRECT_URI,
        "state": state
    }))
}

#[derive(Deserialize)]
pub(crate) struct ExchangeClaudeCodePayload {
    pub(crate) code: String,
    #[serde(default)]
    pub(crate) code_verifier: Option<String>,
    #[serde(default)]
    pub(crate) redirect_uri: Option<String>,
    #[serde(default)]
    pub(crate) state: Option<String>,
}

pub(crate) async fn axum_cc_oauth_exchange(Json(payload): Json<ExchangeClaudeCodePayload>) -> impl IntoResponse {
    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| super::claude_service::CLAUDE_OAUTH_REDIRECT_URI.to_string());

    let state = payload.state.clone().unwrap_or_default();
    let verifier = if let Some(v) = payload.code_verifier {
        v
    } else if !state.is_empty() {
        crate::llm::providers::claude::oauth::take_oauth_state(&state).unwrap_or_default()
    } else {
        String::new()
    };

    if verifier.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "status": "error", "error": "Missing code_verifier or state not found" })),
        )
            .into_response();
    }

    let res = tokio::task::spawn_blocking(move || {
        let mut provider = crate::llm::providers::claude::ClaudeProvider::new();
        match crate::llm::providers::claude::oauth::complete_oauth_flow(&provider.client, &payload.code, &redirect_uri, &verifier, &state) {
            Ok(account) => match provider.add_oauth_account(account) {
                Ok(id) => {
                    crate::app_state::push_client_action("claude:oauth_success", Some(&id));
                    Ok(id)
                }
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            },
            Err(e) => Err((StatusCode::BAD_REQUEST, format!("OAuth exchange failed: {e}"))),
        }
    })
    .await;

    match res {
        Ok(Ok(id)) => Json(json!({ "status": "success", "id": id })).into_response(),
        Ok(Err((status, msg))) => (
            status,
            Json(json!({ "status": "error", "error": msg })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}

// -----------------------------------------------------------------------------
// 3. LLM Token Usage Service
// -----------------------------------------------------------------------------
#[derive(Deserialize)]
pub(crate) struct UsageQuery {
    pub(crate) provider: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) page: Option<usize>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<usize>,
}

pub(crate) async fn axum_usage_summary(Query(q): Query<UsageQuery>) -> impl IntoResponse {
    let summary = crate::llm::usage::get_usage_summary(q.provider.as_deref(), q.date.as_deref()).await;
    Json(summary)
}

pub(crate) async fn axum_usage_records(Query(q): Query<UsageQuery>) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(25).min(200);
    let page = q.page.unwrap_or_else(|| {
        if let Some(off) = q.offset {
            (off / limit) + 1
        } else {
            1
        }
    });

    let paginated = crate::llm::usage::get_usage_records_paginated(
        q.provider.as_deref(),
        q.date.as_deref(),
        page,
        limit,
    )
    .await;
    Json(paginated)
}

pub(crate) async fn axum_usage_clear() -> impl IntoResponse {
    match crate::llm::usage::clear_usage().await {
        Ok(_) => Json(json!({ "status": "success", "success": true })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "status": "error", "error": e.to_string() })),
        )
            .into_response(),
    }
}
