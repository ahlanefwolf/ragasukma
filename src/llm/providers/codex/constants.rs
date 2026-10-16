pub const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const CODEX_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
pub const CODEX_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
pub const CODEX_RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
pub const CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
#[allow(dead_code)]
pub const CODEX_RESET_CREDITS_URL: &str = "https://chatgpt.com/backend-api/wham/rate-limit-reset-credits";
pub const CODEX_CLI_VERSION: &str = "0.159.0";
pub const CODEX_ORIGINATOR: &str = "codex_cli_rs";

pub const CODEX_SCOPES: &[&str] = &[
    "openid",
    "profile",
    "email",
    "offline_access",
];

pub const CODEX_MODELS_URL: &str =
    "https://chatgpt.com/backend-api/codex/models?client_version=0.159.0";

pub const CODEX_DEFAULT_MODELS: &[(&str, &str)] = &[
    ("gpt-6.1-sol", "GPT 6.1 Sol (Fast Reasoning)"),
    ("gpt-6-astra", "GPT 6.0 Astra"),
    ("gpt-6-sol", "GPT 6.0 Sol"),
    ("gpt-6-luna", "GPT 6.0 Luna"),
    ("gpt-5.6-terra", "GPT 5.6 Terra"),
    ("gpt-5.6-luna", "GPT 5.6 Luna"),
    ("gpt-daybreak-blue-latest", "GPT Daybreak Blue"),
    ("gpt-reserve", "GPT Reserve"),
    ("codex-auto-review", "Codex Auto Review"),
];

pub const CODEX_DEFAULT_INSTRUCTIONS: &str =
    "You are an AI assistant in Ragasukma.";
