use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub command: String,
}

pub fn get_skills_path() -> PathBuf {
    // Use kandidat standar (Lokal project assets/ -> beside exe -> Resources macOS -> AppData/Roaming)
    let candidate = crate::assets::find_file_in_candidates("skill-command.md");
    if candidate.exists() {
        return candidate;
    }

    // Default target if none exists yet
    crate::config::get_app_config_dir()
        .join("assets")
        .join("skill-command.md")
}

pub fn parse_skills_markdown() -> Vec<SkillItem> {
    let path = get_skills_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut skills = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|')
            || trimmed.contains(":---")
            || trimmed.contains("Skill Name")
            || trimmed.contains("Nama Skill")
        {
            continue;
        }

        let parts: Vec<&str> = trimmed
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() >= 3 {
            let name = parts[0].to_string();
            let description = parts[1].to_string();
            let command = parts[2].replace('`', "");

            if !name.is_empty() && !command.is_empty() {
                skills.push(SkillItem {
                    name,
                    description,
                    command,
                });
            }
        }
    }
    skills
}

pub fn save_skills_markdown(skills: &[SkillItem]) -> std::io::Result<()> {
    let mut out = String::from(
        "#  Skill Command Registry\n\n\
List of quick commands and system automations that can be triggered directly via voice, menu, or AI automation.\n\
This file can be edited directly or managed via the Skill Commands UI.\n\n\
| Skill Name | Description | Command / Script |\n\
| :--- | :--- | :--- |\n",
    );

    for s in skills {
        out.push_str(&format!(
            "| {} | {} | `{}` |\n",
            s.name.trim(),
            s.description.trim(),
            s.command.trim()
        ));
    }

    let path = get_skills_path();
    std::fs::write(path, out)
}

/// Utility helper to configure subprocess to be isolated from AppImage runtime and prevent console flash on Windows
#[allow(unused_variables)]
pub fn configure_subprocess(cmd: &mut std::process::Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW (0x08000000)
        cmd.creation_flags(0x08000000);
    }
    #[cfg(target_os = "linux")]
    {
        cmd.env_remove("PYTHONHOME");
        cmd.env_remove("PYTHONPATH");
        cmd.env_remove("LD_LIBRARY_PATH");
        cmd.env_remove("LD_PRELOAD");
    }
}

pub fn execute_command_by_os(cmd: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let trimmed = cmd.trim();
        // If perintah is executable direct or file (for example calc.exe, notepad.exe, explorer.exe)
        // and not mengandung shell pipe/redirect, run direct without cmd /C
        if !trimmed.contains('|')
            && !trimmed.contains('&')
            && !trimmed.contains('>')
            && !trimmed.contains('<')
        {
            if let Some((prog, args)) = trimmed.split_once(' ') {
                let clean_prog = prog.trim_matches('"');
                let clean_args = args.trim_matches('"');
                let mut direct_cmd = std::process::Command::new(clean_prog);
                direct_cmd.arg(clean_args);
                configure_subprocess(&mut direct_cmd);
                if direct_cmd.spawn().is_ok() {
                    return Ok(());
                }
            } else {
                let clean_prog = trimmed.trim_matches('"');
                let mut direct_cmd = std::process::Command::new(clean_prog);
                configure_subprocess(&mut direct_cmd);
                if direct_cmd.spawn().is_ok() {
                    return Ok(());
                }
            }
        }

        let mut command = std::process::Command::new("cmd");
        // Using /S /C "<cmd>" so that parser cmd.exe not memotong tanda kutip ganda internal
        command.args(["/S", "/C", &format!("\"{trimmed}\"")]);
        configure_subprocess(&mut command);
        command.spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut command = std::process::Command::new("sh");
        command.args(["-c", cmd]);
        configure_subprocess(&mut command);
        command.spawn()?;
    }
    Ok(())
}
