#[tauri::command]
pub fn check_for_updates() -> serde_json::Value {
    let current_ver = env!("CARGO_PKG_VERSION");
    let repo = "ahlanefwolf/ragasukma";
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    let client = reqwest::blocking::Client::builder()
        .user_agent("ragasukma-updater")
        .timeout(std::time::Duration::from_secs(5))
        .build();

    match client {
        Ok(c) => match c.get(&url).send() {
            Ok(resp) => {
                if resp.status().is_success() {
                    if let Ok(json) = resp.json::<serde_json::Value>() {
                        let latest_tag = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v');
                        let body = json["body"].as_str().unwrap_or("Ada versi pembaruan baru!");
                        let html_url = json["html_url"].as_str().unwrap_or("");

                        let has_update = !latest_tag.is_empty() && latest_tag != current_ver;
                        return serde_json::json!({
                            "status": "success",
                            "has_update": has_update,
                            "current_version": current_ver,
                            "latest_version": latest_tag,
                            "release_notes": body,
                            "url": html_url
                        });
                    }
                }
                serde_json::json!({
                    "status": "up_to_date",
                    "has_update": false,
                    "current_version": current_ver,
                    "message": "Aplikasi sudah versi terbaru!"
                })
            }
            Err(_) => serde_json::json!({
                "status": "offline",
                "has_update": false,
                "current_version": current_ver,
                "message": "Tidak dapat terhubung ke server update."
            })
        },
        Err(_) => serde_json::json!({
            "status": "error",
            "has_update": false,
            "current_version": current_ver,
            "message": "Gagal menyiapkan engine updater."
        })
    }
}
