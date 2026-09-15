use crate::skills::{execute_command_by_os, parse_skills_markdown};
use crate::system_actions::try_execute_builtin_or_app;

pub fn execute_skill_by_identifier(identifier: &str) -> Option<(String, String)> {
    let target = identifier.trim().to_lowercase().replace(['_', '-'], " ");

    // 1. Cek skill kustom di skill-command.md terlebih dahulu
    let skills = parse_skills_markdown();
    for skill in skills {
        let skill_name_clean = skill.name.trim().to_lowercase();
        if skill_name_clean == target || skill_name_clean.replace(['_', '-'], " ") == target {
            match execute_command_by_os(&skill.command) {
                Ok(_) => return Some((skill.name, "Berhasil dijalankan".to_string())),
                Err(e) => return Some((skill.name, format!("Error: {e}"))),
            }
        }
    }

    // 2. Cek aksi sistem paten / aplikasi OS bawaan
    if let Some(res) = try_execute_builtin_or_app(&target) {
        return Some(res);
    }

    None
}
