use super::catalog::get_builtin_system_skills;
use crate::engine::app_scanner::get_installed_applications;
use crate::engine::skills::execute_command_by_os;

/// Eksekusi aksi sistem OS bawaan (Volume, Mute, Lock, Screenshot, Calculator, File Manager, dll.)
pub fn execute_os_action(action: &str) -> Option<(String, String)> {
    match action {
        "volume_up" => execute_by_catalog_name("Naikkan Volume"),
        "volume_down" => execute_by_catalog_name("Turunkan Volume"),
        "mute" => execute_by_catalog_name("Mute Audio"),
        "lock" => execute_by_catalog_name("Kunci Layar"),
        "screenshot" => execute_by_catalog_name("Ambil Screenshot"),
        "calculator" => execute_by_catalog_name("Buka Kalkulator"),
        "file_manager" => execute_by_catalog_name("Buka File Manager"),
        "terminal" => execute_by_catalog_name("Buka Terminal"),
        "notepad" => execute_by_catalog_name("Buka Notepad"),
        _ => None,
    }
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

    let installed_apps = get_installed_applications();

    // 1st Priority: Exact match
    for app in &installed_apps {
        if app.name.eq_ignore_ascii_case(app_query) {
            if execute_command_by_os(&app.exec).is_ok() {
                return Some((format!("Buka {}", app.name), "Berhasil membuka aplikasi".to_string()));
            }
        }
    }

    // 2nd Priority: Starts with / Contains match
    let q_lower = app_query.to_lowercase();
    for app in &installed_apps {
        let app_lower = app.name.to_lowercase();
        if app_lower.starts_with(&q_lower) || app_lower.contains(&q_lower) {
            if execute_command_by_os(&app.exec).is_ok() {
                return Some((format!("Buka {}", app.name), "Berhasil membuka aplikasi".to_string()));
            }
        }
    }

    // 3rd Priority: Fallback eksekusi langsung binary nama aplikasi jika ada di PATH OS
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
