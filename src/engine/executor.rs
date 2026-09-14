use crate::skills::{execute_command_by_os, parse_skills_markdown};

pub fn execute_skill_by_identifier(identifier: &str) -> Option<(String, String)> {
    let skills = parse_skills_markdown();
    let target = identifier.trim().to_lowercase().replace(['_', '-'], " ");

    for skill in skills {
        let skill_name_clean = skill.name.trim().to_lowercase();
        if skill_name_clean == target || skill_name_clean.replace(['_', '-'], " ") == target {
            match execute_command_by_os(&skill.command) {
                Ok(_) => return Some((skill.name, "Berhasil dijalankan".to_string())),
                Err(e) => return Some((skill.name, format!("Error: {e}"))),
            }
        }
    }
    None
}
