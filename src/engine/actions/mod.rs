pub mod browser;
pub mod catalog;
pub mod desktop;
pub mod media;
pub mod os_apps;
pub mod todo_actions;

pub use browser::execute_browser_action;
pub use catalog::{get_builtin_system_skills, open_url_by_os};
pub use desktop::execute_desktop_action;
pub use media::execute_media_action;
pub use os_apps::{execute_app_action, execute_os_action, execute_os_or_app_action};
pub use todo_actions::execute_todo_action;

use crate::engine::skills::{execute_command_by_os, parse_skills_markdown};

/// Eksekusi skill berdasarkan format canonical `namespace:action|payload` atau `namespace:action`
pub fn execute_skill_by_identifier(identifier: &str) -> Option<(String, String)> {
    let raw = identifier.trim();
    if raw.is_empty() {
        return None;
    }

    let (prefix_part, payload) = if let Some((p, arg)) = raw.split_once('|') {
        (p.trim(), arg.trim())
    } else {
        (raw, "")
    };

    let (namespace, action) = if let Some((ns, act)) = prefix_part.split_once(':') {
        (ns.trim().to_lowercase(), act.trim().to_lowercase())
    } else {
        (String::new(), prefix_part.trim().to_lowercase())
    };

    match (namespace.as_str(), action.as_str()) {
        ("browser", act) => browser::execute_browser_action(act, payload),
        ("media", act) => media::execute_media_action(act, payload),
        ("todo", act) => todo_actions::execute_todo_action(act, payload),
        ("desktop" | "vision", act) => desktop::execute_desktop_action(act, payload),
        ("os" | "app", act) => os_apps::execute_os_or_app_action(namespace.as_str(), act, payload),
        ("internal", "open_extension_installer") => browser::execute_browser_action("open_ext_folder", ""),
        _ => {
            // Cek Custom Skills di skill-command.md
            let skills = parse_skills_markdown();
            for skill in skills {
                if skill.name.eq_ignore_ascii_case(raw) {
                    match execute_command_by_os(&skill.command) {
                        Ok(_) => return Some((skill.name, "Berhasil dijalankan".to_string())),
                        Err(e) => return Some((skill.name, format!("Error: {e}"))),
                    }
                }
            }
            None
        }
    }
}
