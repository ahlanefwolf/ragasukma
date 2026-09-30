use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::providers::antigravity::oauth::build_auth_url;
use crate::server::send_json_response;
use serde::Deserialize;
use std::io::Write;
use std::net::TcpStream;

pub const OAUTH_REDIRECT_URI: &str = "http://localhost:42128/api/llm/antigravity/oauth/callback";

#[derive(Deserialize)]
struct AddAccountPayload {
    #[serde(alias = "refreshToken", alias = "token")]
    refresh_token: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default, alias = "projectId")]
    project_id: Option<String>,
}

#[derive(Deserialize)]
struct IdPayload {
    id: String,
}

#[derive(Deserialize)]
struct ExchangeCodePayload {
    code: String,
    #[serde(default)]
    redirect_uri: Option<String>,
}

pub fn handle_get_antigravity_auth_url(full_url: &str, stream: &mut TcpStream) {
    let state = format!(
        "ragasukma_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );
    let auth_url = build_auth_url(OAUTH_REDIRECT_URI, &state);

    if full_url.contains("open=1") {
        crate::assets::open_browser(&auth_url);
    }

    let json = serde_json::json!({
        "status": "success",
        "auth_url": auth_url,
        "redirect_uri": OAUTH_REDIRECT_URI
    });
    send_json_response(stream, 200, &json.to_string());
}

pub fn handle_antigravity_oauth_callback(full_url: &str, stream: &mut TcpStream) {
    // Parse query string for ?code=...
    let query_str = full_url.split('?').nth(1).unwrap_or("");
    let mut code: Option<String> = None;
    let mut error: Option<String> = None;

    for (k, v) in form_urlencoded::parse(query_str.as_bytes()) {
        if k == "code" {
            code = Some(v.to_string());
        } else if k == "error" {
            error = Some(v.to_string());
        }
    }

    if let Some(err_msg) = error {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><title>OAuth Failed</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2>❌ Otorisasi Google Gagal</h2>
    <p style="color: #94a3b8;">{}</p>
    <a href="http://localhost:42128" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Kembali ke Dashboard</a>
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
    <h2>⚠️ Kode Otorisasi Tidak Ditemukan</h2>
    <p style="color: #94a3b8;">Google tidak mengirimkan authorization code yang valid.</p>
    <a href="http://localhost:42128" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Kembali ke Dashboard</a>
  </div>
</body>
</html>"#;
            send_html_response(stream, 400, html);
            return;
        }
    };

    let mut provider = AntigravityProvider::new();
    match provider.add_oauth_account(&auth_code, OAUTH_REDIRECT_URI) {
        Ok(id) => {
            let email = provider
                .accounts
                .iter()
                .find(|a| a.id == id)
                .and_then(|a| a.email.clone())
                .unwrap_or_else(|| id.clone());

            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
  <title>Login Google Antigravity Berhasil</title>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    body {{
      font-family: system-ui, -apple-system, sans-serif;
      background: #090d16;
      color: #f1f5f9;
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      margin: 0;
    }}
    .card {{
      text-align: center;
      padding: 40px 32px;
      background: #111827;
      border-radius: 28px;
      border: 1px solid #06b6d4;
      box-shadow: 0 20px 40px rgba(0,0,0,0.6), 0 0 20px rgba(6,182,212,0.2);
      max-width: 420px;
      width: 90%;
    }}
    .btn {{
      display: inline-block;
      margin-top: 24px;
      padding: 12px 28px;
      background: #06b6d4;
      color: #090d16;
      font-weight: bold;
      border-radius: 14px;
      text-decoration: none;
      transition: all 0.2s;
    }}
    .btn:hover {{
      background: #22d3ee;
      transform: scale(1.02);
    }}
  </style>
</head>
<body>
  <div class="card">
    <div style="font-size: 48px; margin-bottom: 12px;">🚀</div>
    <h2 style="margin: 0 0 8px 0; color: #38bdf8;">Antigravity Login Successful!</h2>
    <p style="color: #94a3b8; font-size: 14px; margin: 0 0 16px 0;">Akun <b>{}</b> telah terhubung ke pool LLM Ragasukma.</p>
    <a class="btn" href="http://localhost:42128">Kembali ke Dashboard</a>
  </div>
  <script>
    if (window.opener) {{
      window.opener.postMessage({{ type: 'ANTIGRAVITY_OAUTH_SUCCESS' }}, '*');
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
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head><title>OAuth Failed</title><meta charset="utf-8"></head>
<body style="font-family: sans-serif; background: #0f172a; color: #f87171; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
  <div style="text-align: center; padding: 32px; background: #1e293b; border-radius: 24px; border: 1px solid #ef4444;">
    <h2>❌ Gagal Menghubungkan Akun</h2>
    <p style="color: #94a3b8;">{}</p>
    <a href="http://localhost:42128" style="color: #38bdf8; text-decoration: none; font-weight: bold;">Kembali ke Dashboard</a>
  </div>
</body>
</html>"#,
                e
            );
            send_html_response(stream, 500, &html);
        }
    }
}

pub fn handle_post_antigravity_oauth_exchange(body: &str, stream: &mut TcpStream) {
    let payload: ExchangeCodePayload = match serde_json::from_str(body) {
        Ok(p) => p,
        Err(e) => {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Invalid request payload: {e}\"}}"),
            );
            return;
        }
    };

    let redirect_uri = payload
        .redirect_uri
        .unwrap_or_else(|| OAUTH_REDIRECT_URI.to_string());

    let mut provider = AntigravityProvider::new();
    match provider.add_oauth_account(&payload.code, &redirect_uri) {
        Ok(id) => {
            send_json_response(
                stream,
                200,
                &format!("{{\"status\":\"success\",\"id\":\"{id}\"}}"),
            );
        }
        Err(e) => {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Failed to exchange code: {e}\"}}"),
            );
        }
    }
}

fn send_html_response(stream: &mut TcpStream, status: u16, html: &str) {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let resp = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: *\r\n\r\n{}",
        status,
        status_text,
        html.len(),
        html
    );
    let _ = stream.write_all(resp.as_bytes());
}

pub fn handle_get_antigravity_accounts(stream: &mut TcpStream) {
    let mut provider = AntigravityProvider::new();
    let _ = provider.load_accounts();
    let accounts = provider.list_accounts();

    match serde_json::to_string(&accounts) {
        Ok(json) => send_json_response(stream, 200, &json),
        Err(e) => send_json_response(
            stream,
            500,
            &format!("{{\"error\":\"Failed to serialize accounts: {e}\"}}"),
        ),
    }
}

pub fn handle_post_antigravity_account(body: &str, stream: &mut TcpStream) {
    let payload: AddAccountPayload = match serde_json::from_str(body) {
        Ok(p) => p,
        Err(e) => {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Invalid request payload: {e}\"}}"),
            );
            return;
        }
    };

    let mut provider = AntigravityProvider::new();
    match provider.add_account(&payload.refresh_token, payload.email, payload.project_id) {
        Ok(id) => {
            send_json_response(
                stream,
                200,
                &format!("{{\"status\":\"success\",\"id\":\"{id}\"}}"),
            );
        }
        Err(e) => {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Failed to add account: {e}\"}}"),
            );
        }
    }
}

pub fn handle_delete_antigravity_account(body: &str, stream: &mut TcpStream) {
    let payload: IdPayload = match serde_json::from_str(body) {
        Ok(p) => p,
        Err(e) => {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Invalid request payload: {e}\"}}"),
            );
            return;
        }
    };

    let mut provider = AntigravityProvider::new();
    let deleted = provider.remove_account(&payload.id);

    if deleted {
        send_json_response(stream, 200, "{\"status\":\"success\"}");
    } else {
        send_json_response(stream, 404, "{\"error\":\"Account not found\"}");
    }
}

pub fn handle_refresh_antigravity_account(body: &str, stream: &mut TcpStream) {
    let payload: IdPayload = match serde_json::from_str(body) {
        Ok(p) => p,
        Err(e) => {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Invalid request payload: {e}\"}}"),
            );
            return;
        }
    };

    let mut provider = AntigravityProvider::new();
    match provider.refresh_account_by_id(&payload.id) {
        Ok(pid) => {
            send_json_response(
                stream,
                200,
                &format!("{{\"status\":\"success\",\"project_id\":\"{pid}\"}}"),
            );
        }
        Err(e) => {
            send_json_response(
                stream,
                500,
                &format!("{{\"error\":\"Token refresh failed: {e}\"}}"),
            );
        }
    }
}

pub fn handle_toggle_antigravity_account(body: &str, stream: &mut TcpStream) {
    let payload: IdPayload = match serde_json::from_str(body) {
        Ok(p) => p,
        Err(e) => {
            send_json_response(
                stream,
                400,
                &format!("{{\"error\":\"Invalid request payload: {e}\"}}"),
            );
            return;
        }
    };

    let mut provider = AntigravityProvider::new();
    match provider.toggle_account_by_id(&payload.id) {
        Ok(active) => {
            send_json_response(
                stream,
                200,
                &format!("{{\"status\":\"success\",\"is_active\":{active}}}"),
            );
        }
        Err(e) => {
            send_json_response(stream, 404, &format!("{{\"error\":\"{e}\"}}"));
        }
    }
}

pub fn handle_get_antigravity_account_quota(body: &str, full_url: &str, stream: &mut TcpStream) {
    let mut id = None;
    if !body.trim().is_empty() {
        if let Ok(payload) = serde_json::from_str::<IdPayload>(body) {
            id = Some(payload.id);
        }
    }
    if id.is_none() {
        let query_str = full_url.split('?').nth(1).unwrap_or("");
        for (k, v) in form_urlencoded::parse(query_str.as_bytes()) {
            if k == "id" {
                id = Some(v.to_string());
                break;
            }
        }
    }

    let acc_id = match id {
        Some(i) if !i.is_empty() => i,
        _ => {
            send_json_response(
                stream,
                400,
                "{\"status\":\"error\",\"error\":\"Account ID is required\"}",
            );
            return;
        }
    };

    let mut provider = AntigravityProvider::new();
    match provider.get_account_quota(&acc_id) {
        Ok(quota) => {
            if let Ok(json) = serde_json::to_string(&quota) {
                send_json_response(stream, 200, &json);
            } else {
                send_json_response(
                    stream,
                    500,
                    "{\"status\":\"error\",\"error\":\"Serialization error\"}",
                );
            }
        }
        Err(e) => {
            let err_json = serde_json::json!({
                "status": "error",
                "account_id": acc_id,
                "error": e.to_string(),
                "models": [],
                "weekly": []
            });
            send_json_response(stream, 200, &err_json.to_string());
        }
    }
}
