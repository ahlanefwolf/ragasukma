use crate::llm::providers::claude::ClaudeProvider;
use crate::llm::providers::claude::oauth::{complete_oauth_flow, take_oauth_state};
use axum::response::IntoResponse;
use serde::Deserialize;

pub const CLAUDE_OAUTH_REDIRECT_URI: &str = "http://localhost:54321/callback";

#[derive(Deserialize, Default)]
pub struct ClaudeOAuthQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// Core processing of OAuth redirect callback from Claude (Anthropic)
pub fn process_claude_oauth_callback(
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
) -> (u16, String) {
    if let Some(err_msg) = error {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Claude OAuth Failed</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2>Claude Authorization Failed</h2>
    <p style="color: #94a3b8;">{}</p>
    <a href="http://localhost:42128/#provider-claude" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a>
  </div>
</body>
</html>"#,
            err_msg
        );
        return (400, html);
    }

    let auth_code = match code {
        Some(c) if !c.is_empty() => c,
        _ => {
            let html = r#"<!DOCTYPE html>
<html>
<head><title>Invalid Code</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2>Authorization Code Not Found</h2>
    <p style="color: #94a3b8;">Anthropic did not return a valid authorization code.</p>
    <a href="http://localhost:42128/#provider-claude" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a>
  </div>
</body>
</html>"#;
            return (400, html.to_string());
        }
    };

    let st = state.unwrap_or_default();
    let verifier = take_oauth_state(&st).unwrap_or_default();

    if verifier.is_empty() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>PKCE Expired</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2>PKCE State Expired</h2>
    <p style="color: #94a3b8;">OAuth session expired. Please restart the authorization flow.</p>
    <a href="http://localhost:42128/#provider-claude" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a>
  </div>
</body>
</html>"#;
        return (400, html.to_string());
    }

    let mut provider = ClaudeProvider::new();
    crate::term_log!(
        "[Claude OAuth] Received code from Anthropic, exchanging token with redirect_uri: {}",
        CLAUDE_OAUTH_REDIRECT_URI
    );

    match complete_oauth_flow(&provider.client, &auth_code, CLAUDE_OAUTH_REDIRECT_URI, &verifier, &st) {
        Ok(account) => match provider.add_oauth_account(account) {
            Ok(id) => {
                crate::term_log!("[Claude OAuth] Successfully connected account: {}", id);
                let email = provider
                    .accounts
                    .iter()
                    .find(|a| a.id == id)
                    .and_then(|a| a.email.clone())
                    .unwrap_or_else(|| id.clone());

                crate::app_state::push_client_action("claude:oauth_success", Some(&email));

                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
  <title>Login Success</title>
  <meta charset="utf-8">
  <style>
    body {{
      font-family: system-ui, sans-serif;
      background: #111;
      color: #eee;
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      margin: 0;
    }}
    .box {{
      text-align: center;
      padding: 24px;
    }}
    button {{
      margin-top: 16px;
      padding: 8px 16px;
      cursor: pointer;
    }}
  </style>
</head>
<body>
  <div class="box">
    <p>Login success: <b>{}</b></p>
    <p>You can close this window now.</p>
    <button onclick="window.close()">Close Window</button>
  </div>
  <script>
    if (window.opener) {{
      window.opener.postMessage({{ type: 'CLAUDE_OAUTH_SUCCESS' }}, '*');
      setTimeout(() => window.close(), 1200);
    }}
  </script>
</body>
</html>"#,
                    email
                );
                (200, html)
            }
            Err(e) => {
                let err_html = format!("<!DOCTYPE html><html><body><h3>Save Error</h3><p>{}</p></body></html>", e);
                (500, err_html)
            }
        },
        Err(e) => {
            let err_html = format!("<!DOCTYPE html><html><body><h3>Exchange Error</h3><p>{}</p></body></html>", e);
            (500, err_html)
        }
    }
}

/// Native Axum route handler for OAuth callback
pub async fn axum_claude_oauth_callback(
    axum::extract::Query(params): axum::extract::Query<ClaudeOAuthQuery>,
) -> axum::response::Response {
    let (status, html) = tokio::task::spawn_blocking(move || {
        process_claude_oauth_callback(params.code, params.state, params.error)
    })
    .await
    .unwrap_or((500, "Internal Server Error".to_string()));

    let status_code = axum::http::StatusCode::from_u16(status)
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

    axum::response::Response::builder()
        .status(status_code)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .body(axum::body::Body::from(html))
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Ensure background listener on port 54321 is running with Axum on Tokio
pub fn ensure_port_54321_listener() {
    static LISTENER_STARTED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if LISTENER_STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:54321").await {
            Ok(l) => {
                crate::term_log!(" [Claude OAuth Callback Listener] Running on http://127.0.0.1:54321/callback");
                l
            }
            Err(e) => {
                eprintln!("[Claude OAuth] Notice: Could not bind 127.0.0.1:54321: {}", e);
                LISTENER_STARTED.store(false, std::sync::atomic::Ordering::SeqCst);
                return;
            }
        };

        let cors = tower_http::cors::CorsLayer::permissive();
        let app = axum::Router::new()
            .route("/callback", axum::routing::get(axum_claude_oauth_callback))
            .route("/auth/callback", axum::routing::get(axum_claude_oauth_callback))
            .layer(cors);

        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("[Claude OAuth] Axum listener error on port 54321: {}", e);
            LISTENER_STARTED.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_claude_oauth_callback_error() {
        let (status, html) = process_claude_oauth_callback(None, None, Some("access_denied".to_string()));
        assert_eq!(status, 400);
        assert!(html.contains("Claude Authorization Failed"));
        assert!(html.contains("access_denied"));
    }

    #[test]
    fn test_process_claude_oauth_callback_missing_code() {
        let (status, html) = process_claude_oauth_callback(None, None, None);
        assert_eq!(status, 400);
        assert!(html.contains("Authorization Code Not Found"));
    }

    #[test]
    fn test_process_claude_oauth_callback_expired_pkce() {
        let (status, html) = process_claude_oauth_callback(
            Some("valid_code".to_string()),
            Some("expired_or_invalid_state".to_string()),
            None,
        );
        assert_eq!(status, 400);
        assert!(html.contains("PKCE State Expired"));
    }

    #[tokio::test]
    async fn test_axum_claude_oauth_callback_error_query() {
        let query = ClaudeOAuthQuery {
            code: None,
            state: None,
            error: Some("test_error".to_string()),
        };
        let res = axum_claude_oauth_callback(axum::extract::Query(query)).await;
        assert_eq!(res.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
