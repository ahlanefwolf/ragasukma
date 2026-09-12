pub fn execute_command_by_os(command: &str) -> std::io::Result<std::process::Child> {
    #[cfg(target_os = "windows")]
    {
        let win_cmd = format!("start \"\" {command}");
        std::process::Command::new("cmd")
            .args(["/C", &win_cmd])
            .spawn()
    }

    #[cfg(target_os = "macos")]
    {
        let spawn_cmd = format!("nohup {command} >/dev/null 2>&1 &");
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&spawn_cmd)
            .spawn()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let spawn_cmd = format!("nohup {command} >/dev/null 2>&1 &");
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&spawn_cmd)
            .spawn()
    }
}

pub fn execute_skill_by_id(identifier: &str) -> Option<(String, String)> {
    let skills = crate::skills::parse_skills_markdown();
    let target = identifier.trim().to_lowercase().replace(['_', '-'], " ");

    for skill in skills {
        let skill_name_clean = skill.name.trim().to_lowercase();
        if skill_name_clean == target || skill_name_clean.replace(['_', '-'], " ") == target {
            let spawn_res = execute_command_by_os(&skill.command);
            match spawn_res {
                Ok(_) => return Some((skill.name, "Berhasil dijalankan".to_string())),
                Err(e) => return Some((skill.name, format!("Error: {e}"))),
            }
        }
    }
    None
}
