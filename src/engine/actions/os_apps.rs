use super::catalog::get_builtin_system_skills;
use crate::engine::app_scanner::get_installed_applications;
use crate::engine::skills::execute_command_by_os;

/// Handler gabungan untuk kontrol sistem OS dan peluncuran aplikasi
pub fn execute_os_or_app_action(namespace: &str, action: &str, payload: &str) -> Option<(String, String)> {
    let act = action.trim().to_lowercase();
    let query = payload.trim();

    // 1. Aksi utilitas murni sistem OS
    match act.as_str() {
        "volume_up" => return execute_by_catalog_name("Naikkan Volume"),
        "volume_down" => return execute_by_catalog_name("Turunkan Volume"),
        "mute" => return execute_by_catalog_name("Mute Audio"),
        "lock" => return execute_by_catalog_name("Kunci Layar"),
        "screenshot" => return execute_by_catalog_name("Ambil Screenshot"),
        _ => {}
    }

    // 2. Semua permintaan aplikasi (baik payload maupun action) dialirkan ke scanner aplikasi dinamis
    let target = if !query.is_empty() {
        query
    } else if namespace == "app" || namespace == "os" {
        action
    } else {
        ""
    };

    if !target.is_empty() {
        return execute_app_action(target);
    }

    None
}

/// Eksekusi aksi sistem OS bawaan berdasarkan nama aksi
pub fn execute_os_action(action: &str) -> Option<(String, String)> {
    execute_os_or_app_action("os", action, "")
}

fn execute_by_catalog_name(catalog_name: &str) -> Option<(String, String)> {
    for item in get_builtin_system_skills() {
        if item.name.eq_ignore_ascii_case(catalog_name) {
            match execute_command_by_os(&item.command) {
                Ok(_) => return Some((item.name, "Berhasil dijalankan".to_string())),
                Err(e) => return Some((item.name, format!("Error: {e}"))),
            }
        }
    }
    None
}

/// Eksekusi launcher aplikasi OS yang terinstal
/// Format: `app:open|<nama>`
pub fn execute_app_action(payload: &str) -> Option<(String, String)> {
    let app_query = payload.trim();
    if app_query.is_empty() {
        return None;
    }

    let q_lower = app_query.to_lowercase();
    let installed_apps = get_installed_applications();

    // 1st Priority: Exact match dari installed applications
    for app in &installed_apps {
        if app.name.eq_ignore_ascii_case(app_query) {
            if execute_command_by_os(&app.exec).is_ok() {
                return Some((format!("Buka {}", app.name), "Berhasil membuka aplikasi".to_string()));
            }
        }
    }

    // 2nd Priority: Starts with / Contains match dari installed applications
    for app in &installed_apps {
        let app_lower = app.name.to_lowercase();
        if app_lower.starts_with(&q_lower) || app_lower.contains(&q_lower) {
            if execute_command_by_os(&app.exec).is_ok() {
                return Some((format!("Buka {}", app.name), "Berhasil membuka aplikasi".to_string()));
            }
        }
    }

    // 3rd Priority: Fallback eksekusi langsung binary nama aplikasi jika ada di PATH OS
    #[cfg(target_os = "windows")]
    {
        if execute_command_by_os(app_query).is_ok() {
            return Some((
                format!("Buka {app_query}"),
                "Berhasil menjalankan aplikasi dari sistem".to_string(),
            ));
        }
        let explorer_cmd = format!("explorer.exe \"{}\"", app_query);
        if execute_command_by_os(&explorer_cmd).is_ok() {
            return Some((
                format!("Buka {app_query}"),
                "Berhasil menjalankan aplikasi dari sistem".to_string(),
            ));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let binary_candidate = q_lower.replace(' ', "-");
        let cmd_test = format!("command -v {binary_candidate} >/dev/null 2>&1");
        if std::process::Command::new("sh")
            .args(["-c", &cmd_test])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            let launch_cmd = format!("{binary_candidate} >/dev/null 2>&1 &");
            if execute_command_by_os(&launch_cmd).is_ok() {
                return Some((
                    format!("Buka {app_query}"),
                    "Berhasil menjalankan aplikasi dari sistem".to_string(),
                ));
            }
        }
    }

    None
}
