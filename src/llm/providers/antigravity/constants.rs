pub const ANTIGRAVITY_CLIENT_ID: &str =
    "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
pub const ANTIGRAVITY_CLIENT_SECRET: &str = "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";
pub const ANTIGRAVITY_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const ANTIGRAVITY_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const ANTIGRAVITY_USER_INFO_URL: &str = "https://www.googleapis.com/oauth2/v1/userinfo";
pub const ANTIGRAVITY_BASE_URL: &str = "https://daily-cloudcode-pa.googleapis.com";
pub const ANTIGRAVITY_OAUTH_REDIRECT_URI: &str =
    "http://localhost:42128/api/llm/antigravity/oauth/callback";
pub const ANTIGRAVITY_LOAD_CODE_ASSIST_URL: &str =
    "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
pub const ANTIGRAVITY_ONBOARD_USER_URL: &str =
    "https://cloudcode-pa.googleapis.com/v1internal:onboardUser";
pub const ANTIGRAVITY_FETCH_MODELS_URL: &str =
    "https://daily-cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels";
pub const ANTIGRAVITY_QUOTA_SUMMARY_URL: &str =
    "https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary";
pub const ANTIGRAVITY_USER_AGENT: &str = "antigravity/ide/2.11.0 darwin/arm64";

pub const ANTIGRAVITY_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "https://www.googleapis.com/auth/cclog",
    "https://www.googleapis.com/auth/experimentsandconfigs",
];

/// Default thinking signature fallback for Gemini 3+ / Antigravity thinking models (valid Base64)
pub const DEFAULT_THINKING_AG_SIGNATURE: &str = "dGhvdWdodF9zaWduYXR1cmU=";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_antigravity_constants_validity() {
        assert!(ANTIGRAVITY_CLIENT_ID.contains(".apps.googleusercontent.com"));
        assert!(ANTIGRAVITY_CLIENT_SECRET.starts_with("GOCSPX-"));
        assert!(ANTIGRAVITY_AUTH_URL.starts_with("https://accounts.google.com"));
        assert!(ANTIGRAVITY_TOKEN_URL.starts_with("https://oauth2.googleapis.com"));
        assert!(ANTIGRAVITY_BASE_URL.starts_with("https://daily-cloudcode-pa.googleapis.com"));
        assert!(ANTIGRAVITY_FETCH_MODELS_URL.contains("fetchAvailableModels"));
        assert!(ANTIGRAVITY_QUOTA_SUMMARY_URL.contains("retrieveUserQuotaSummary"));
        assert!(ANTIGRAVITY_SCOPES.len() >= 5);
        assert!(ANTIGRAVITY_USER_AGENT.contains("antigravity/ide/"));

        // Verify base64 signature format
        assert!(!DEFAULT_THINKING_AG_SIGNATURE.is_empty());
        assert!(DEFAULT_THINKING_AG_SIGNATURE.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }
}
