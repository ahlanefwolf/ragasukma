use crate::llm::providers::codex::CodexProvider;
use crate::llm::providers::codex::constants::{
    CODEX_CLI_VERSION, CODEX_DEFAULT_MODELS, CODEX_MODELS_URL, CODEX_ORIGINATOR,
};
use crate::llm::providers::codex::oauth::{
    build_auth_url, complete_oauth_flow, generate_pkce, store_oauth_state, take_oauth_state,
};
use crate::server::send_json_response;
use serde::Deserialize;
use std::io::{Read, Write};

pub const CODEX_OAUTH_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";

/// Ensure background listener on port 1455 is running to catch OpenAI's callback
pub fn ensure_port_1455_listener() {
    static LISTENER_STARTED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if LISTENER_STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(|| {
        let listener = match std::net::TcpListener::bind("127.0.0.1:1455") {
            Ok(l) => {
                crate::term_log!(" [Codex OAuth Callback Listener] Running on http://127.0.0.1:1455/auth/callback");
                l
            }
            Err(e) => {
                eprintln!("[Codex OAuth] Notice: Could not bind 127.0.0.1:1455: {}", e);
                LISTENER_STARTED.store(false, std::sync::atomic::Ordering::SeqCst);
                return;
            }
        };

        for mut stream in listener.incoming().flatten() {
            std::thread::spawn(move || {
                let mut buffer = [0u8; 4096];
                let n = stream.read(&mut buffer).unwrap_or(0);
                let req_str = String::from_utf8_lossy(&buffer[..n]);
                if let Some(first_line) = req_str.lines().next() {
                    let parts: Vec<&str> = first_line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let url_path = parts[1];
                        if url_path.starts_with("/auth/callback") || url_path.starts_with("/callback") {
                            handle_codex_oauth_callback(url_path, &mut stream);
                            let _ = stream.flush();
                            return;
                        }
                    }
                }
                let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            });
        }
    });
}

#[derive(Deserialize)]
struct AddCodexAccountPayload {
    #[serde(alias = "refreshToken", alias = "token")]
    refresh_token: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default, alias = "chatgptAccountId", alias = "accountId")]
    chatgpt_account_id: Option<String>,
}

#[derive(Deserialize)]
struct IdPayload {
    id: String,
}

#[derive(Deserialize)]
struct ExchangeCodePayload {
    code: String,
    state: Option<String>,
    #[serde(default, alias = "codeVerifier")]
    code_verifier: Option<String>,
    #[serde(default, alias = "redirectUri")]
    redirect_uri: Option<String>,
}

fn send_html_response(stream: &mut dyn Write, status: u16, html: &str) {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        _ => "Internal Server Error",
    };
    let resp = format!(
        "HTTP/1.1 {status} {reason}\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        Content-Length: {}\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Connection: close\r\n\r\n\
        {}",
        html.len(),
        html
    );
    let _ = stream.write_all(resp.as_bytes());
}

pub fn handle_get_codex_auth_url(full_url: &str, stream: &mut dyn Write) {
    ensure_port_1455_listener();

    let (code_verifier, code_challenge) = generate_pkce();
    let state = format!(
        "cx_state_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        std::process::id()
    );

    store_oauth_state(&state, &code_verifier);

    let auth_url = build_auth_url(CODEX_OAUTH_REDIRECT_URI, &state, &code_challenge);

    if full_url.contains("open=1") {
        crate::assets::open_browser(&auth_url);
    }

    let json = serde_json::json!({
        "status": "success",
        "auth_url": auth_url,
        "redirect_uri": CODEX_OAUTH_REDIRECT_URI,
        "state": state
    });
    send_json_response(stream, 200, &json.to_string());
}

pub fn handle_codex_oauth_callback(full_url: &str, stream: &mut dyn Write) {
    let query_str = full_url.split('?').nth(1).unwrap_or("");
    let mut code: Option<String> = None;
    let mut state: Option<String> = None;
    let mut error: Option<String> = None;

    for (k, v) in form_urlencoded::parse(query_str.as_bytes()) {
        if k == "code" {
            code = Some(v.to_string());
        } else if k == "state" {
            state = Some(v.to_string());
        } else if k == "error" {
            error = Some(v.to_string());
        }
    }

    if let Some(err_msg) = error {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Codex OAuth Failed</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2> OpenAI Authorization Failed</h2>
    <p style="color: #94a3b8;">{}</p>
    <a href="http://localhost:42128/#provider-codex" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a>
  </div>
</body>
</html>"#,
            err_msg
        );
        send_html_response(stream, 400, &html);
        return;
    }

    let auth_code = match code {
        Some(c) if !c.is_empty() => c,
        _ => {
            let html = r#"<!DOCTYPE html>
<html>
<head><title>Invalid Code</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2> Authorization Code Not Found</h2>
    <p style="color: #94a3b8;">OpenAI did not return a valid authorization code.</p>
    <a href="http://localhost:42128/#provider-codex" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a>
  </div>
</body>
</html>"#;
            send_html_response(stream, 400, html);
            return;
        }
    };

    let verifier = state
        .as_deref()
        .and_then(take_oauth_state)
        .unwrap_or_default();

    if verifier.is_empty() {
        let html = r#"<!DOCTYPE html>
<html>
<head><title>PKCE Expired</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2> PKCE State Expired</h2>
    <p style="color: #94a3b8;">OAuth session expired. Please restart the authorization flow.</p>
    <a href="http://localhost:42128/#provider-codex" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Back to Dashboard</a>
  </div>
</body>
</html>"#;
        send_html_response(stream, 400, html);
        return;
    }

    let mut provider = CodexProvider::new();
    crate::term_log!(
        "[Codex OAuth] Received code from OpenAI, exchanging token with redirect_uri: {}",
        CODEX_OAUTH_REDIRECT_URI
    );

    match complete_oauth_flow(&provider.client, &auth_code, CODEX_OAUTH_REDIRECT_URI, &verifier) {
        Ok(account) => match provider.add_oauth_account(account) {
            Ok(id) => {
                crate::term_log!("[Codex OAuth] Successfully connected account: {}", id);
                let email = provider
                    .accounts
                    .iter()
                    .find(|a| a.id == id)
                    .and_then(|a| a.email.clone())
                    .unwrap_or_else(|| id.clone());

                crate::app_state::push_client_action("codex:oauth_success", Some(&email));

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
      window.opener.postMessage({{ type: 'CODEX_OAUTH_SUCCESS' }}, '*');
      setTimeout(() => window.close(), 1200);
    }}
  </script>
</body>
</html>"#,
                    email
                );
                send_html_response(stream, 200, &html);
            }
            Err(e) => {
                let err_html = format!("<!DOCTYPE html><html><body><h3>Save Error</h3><p>{}</p></body></html>", e);
                send_html_response(stream, 500, &err_html);
            }
        },
        Err(e) => {
            let err_html = format!("<!DOCTYPE html><html><body><h3>Exchange Error</h3><p>{}</p></body></html>", e);
            send_html_response(stream, 500, &err_html);
        }
    }
}

pub fn handle_post_codex_oauth_exchange(body_str: &str, stream: &mut dyn Write) {
    let payload: ExchangeCodePayload = match serde_json::from_str(body_str) {
        Ok(p) => p,
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"Invalid JSON: {}\"}}", e);
            send_json_response(stream, 400, &err);
            return;
        }
    };

    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| CODEX_OAUTH_REDIRECT_URI.to_string());

    let verifier = if let Some(v) = payload.code_verifier {
        v
    } else if let Some(st) = payload.state.as_deref() {
        take_oauth_state(st).unwrap_or_default()
    } else {
        String::new()
    };

    if verifier.is_empty() {
        let err = "{\"status\":\"error\",\"error\":\"Missing code_verifier or state not found\"}";
        send_json_response(stream, 400, err);
        return;
    }

    let mut provider = CodexProvider::new();
    match complete_oauth_flow(&provider.client, &payload.code, &redirect_uri, &verifier) {
        Ok(account) => match provider.add_oauth_account(account) {
            Ok(id) => {
                crate::app_state::push_client_action("codex:oauth_success", Some(&id));
                let json = serde_json::json!({
                    "status": "success",
                    "id": id
                });
                send_json_response(stream, 200, &json.to_string());
            }
            Err(e) => {
                let err = format!("{{\"status\":\"error\",\"error\":\"{}\"}}", e);
                send_json_response(stream, 500, &err);
            }
        },
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"{}\"}}", e);
            send_json_response(stream, 500, &err);
        }
    }
}

pub fn handle_get_codex_accounts(stream: &mut dyn Write) {
    let provider = CodexProvider::new();
    let infos = provider.list_account_infos();
    let json = serde_json::to_string(&infos).unwrap_or_else(|_| "[]".to_string());
    send_json_response(stream, 200, &json);
}

pub fn handle_get_codex_models(stream: &mut dyn Write) {
    let mut provider = CodexProvider::new();
    let mut live_models: Vec<serde_json::Value> = Vec::new();

    if let Some(acc) = provider.accounts.iter_mut().find(|a| a.is_active) {
        let _ = crate::llm::providers::codex::oauth::refresh_account_token(&provider.client, acc);
        if let Some(ref access_token) = acc.access_token {
            let mut req = provider
                .client
                .get(CODEX_MODELS_URL)
                .header("Authorization", format!("Bearer {access_token}"))
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .header("originator", CODEX_ORIGINATOR)
                .header("User-Agent", format!("codex_cli_rs/{}", CODEX_CLI_VERSION))
                .header("version", CODEX_CLI_VERSION);

            if let Some(ref act_id) = acc.chatgpt_account_id {
                req = req.header("ChatGPT-Account-ID", act_id);
            }

            if let Ok(res) = req.send() {
                if res.status().is_success() {
                    if let Ok(val) = res.json::<serde_json::Value>() {
                        let items = val
                            .get("models")
                            .or_else(|| val.get("data"))
                            .and_then(|v| v.as_array());

                        if let Some(arr) = items {
                            for item in arr {
                                let id = item
                                    .get("id")
                                    .or_else(|| item.get("slug"))
                                    .or_else(|| item.get("model"))
                                    .or_else(|| item.get("name"))
                                    .and_then(|v| v.as_str());

                                if let Some(m_id) = id {
                                    let name = item
                                        .get("display_name")
                                        .or_else(|| item.get("displayName"))
                                        .or_else(|| item.get("name"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(m_id);

                                    live_models.push(serde_json::json!({
                                        "id": format!("cx/{}", m_id),
                                        "model_id": m_id,
                                        "display_name": name,
                                        "provider": "codex"
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let list = if !live_models.is_empty() {
        live_models
    } else {
        CODEX_DEFAULT_MODELS
            .iter()
            .map(|(id, name)| {
                serde_json::json!({
                    "id": format!("cx/{}", id),
                    "model_id": id,
                    "display_name": name,
                    "provider": "codex"
                })
            })
            .collect()
    };

    let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string());
    send_json_response(stream, 200, &json);
}

pub fn handle_post_codex_account(body_str: &str, stream: &mut dyn Write) {
    let payload: AddCodexAccountPayload = match serde_json::from_str(body_str) {
        Ok(p) => p,
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"Invalid JSON: {}\"}}", e);
            send_json_response(stream, 400, &err);
            return;
        }
    };

    let mut provider = CodexProvider::new();
    match provider.add_account(&payload.refresh_token, payload.email, payload.chatgpt_account_id) {
        Ok(id) => {
            let json = serde_json::json!({
                "status": "success",
                "id": id
            });
            send_json_response(stream, 200, &json.to_string());
        }
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"{}\"}}", e);
            send_json_response(stream, 500, &err);
        }
    }
}

pub fn handle_delete_codex_account(body_str: &str, stream: &mut dyn Write) {
    let payload: IdPayload = match serde_json::from_str(body_str) {
        Ok(p) => p,
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"Invalid JSON: {}\"}}", e);
            send_json_response(stream, 400, &err);
            return;
        }
    };

    let mut provider = CodexProvider::new();
    if provider.remove_account(&payload.id) {
        send_json_response(stream, 200, "{\"status\":\"success\"}");
    } else {
        send_json_response(stream, 404, "{\"status\":\"error\",\"error\":\"Account not found\"}");
    }
}

pub fn handle_refresh_codex_account(body_str: &str, stream: &mut dyn Write) {
    let payload: IdPayload = match serde_json::from_str(body_str) {
        Ok(p) => p,
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"Invalid JSON: {}\"}}", e);
            send_json_response(stream, 400, &err);
            return;
        }
    };

    let mut provider = CodexProvider::new();
    match provider.refresh_account_by_id(&payload.id) {
        Ok(()) => {
            send_json_response(stream, 200, "{\"status\":\"success\"}");
        }
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"{}\"}}", e);
            send_json_response(stream, 500, &err);
        }
    }
}

pub fn handle_toggle_codex_account(body_str: &str, stream: &mut dyn Write) {
    let payload: IdPayload = match serde_json::from_str(body_str) {
        Ok(p) => p,
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"Invalid JSON: {}\"}}", e);
            send_json_response(stream, 400, &err);
            return;
        }
    };

    let mut provider = CodexProvider::new();
    match provider.toggle_account_by_id(&payload.id) {
        Ok(active) => {
            let json = serde_json::json!({
                "status": "success",
                "is_active": active
            });
            send_json_response(stream, 200, &json.to_string());
        }
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"{}\"}}", e);
            send_json_response(stream, 500, &err);
        }
    }
}

pub fn handle_get_codex_account_quota(body_str: &str, full_url: &str, stream: &mut dyn Write) {
    let mut id = None;

    if let Ok(payload) = serde_json::from_str::<IdPayload>(body_str) {
        id = Some(payload.id);
    } else if let Some(query) = full_url.split('?').nth(1) {
        for (k, v) in form_urlencoded::parse(query.as_bytes()) {
            if k == "id" {
                id = Some(v.to_string());
                break;
            }
        }
    }

    let account_id = match id {
        Some(i) => i,
        None => {
            send_json_response(stream, 400, "{\"status\":\"error\",\"error\":\"Missing account id\"}");
            return;
        }
    };

    let mut provider = CodexProvider::new();
    match provider.get_account_quota(&account_id) {
        Ok(res) => {
            let json = serde_json::to_string(&res).unwrap_or_else(|_| "{}".to_string());
            send_json_response(stream, 200, &json);
        }
        Err(e) => {
            let err = format!("{{\"status\":\"error\",\"error\":\"{}\"}}", e);
            send_json_response(stream, 500, &err);
        }
    }
}
