use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static REQUEST_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn generate_pseudo_uuid(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let count = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}-{:x}-{:x}", prefix, now, count)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityAccount {
    #[serde(default = "default_account_id")]
    pub id: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(alias = "refreshToken", alias = "token")]
    pub refresh_token: String,
    #[serde(default, alias = "accessToken")]
    pub access_token: Option<String>,
    #[serde(default, alias = "expiresAt")]
    pub expires_at: u64,
    #[serde(default, alias = "projectId")]
    pub project_id: Option<String>,
    #[serde(default, alias = "tierId")]
    pub tier_id: Option<String>,
    #[serde(default, alias = "rateLimitedUntil")]
    pub rate_limited_until: u64,
    #[serde(default, alias = "consecutiveErrors")]
    pub consecutive_errors: u32,
    #[serde(default = "default_true", alias = "isActive", alias = "enabled")]
    pub is_active: bool,
}

fn default_account_id() -> String {
    generate_pseudo_uuid("ag-acc")
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityAccountInfo {
    pub id: String,
    pub email: Option<String>,
    pub project_id: Option<String>,
    pub is_active: bool,
    pub is_rate_limited: bool,
    pub rate_limited_seconds_left: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    #[serde(default)]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQuotaInfo {
    pub model_id: String,
    pub display_name: String,
    pub remaining_fraction: f64,
    pub remaining_percentage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityQuotaResponse {
    pub status: String,
    pub account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    pub models: Vec<ModelQuotaInfo>,
    pub weekly: Vec<ModelQuotaInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
