use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub command: String,
}

pub fn get_skills_path() -> PathBuf {
    let in_assets = PathBuf::from("assets/skill-command.md");
    if in_assets.exists() {
        return in_assets;
    }
    let parent_assets = PathBuf::from("../assets/skill-command.md");
    if parent_assets.exists() {
        return parent_assets;
    }
    let local = PathBuf::from("skill-command.md");
    if local.exists() {
        return local;
    }
    let parent = PathBuf::from("../skill-command.md");
    if parent.exists() {
        return parent;
    }
    let cfg_path = crate::config::get_app_config_dir().join("assets").join("skill-command.md");
    if cfg_path.exists() {
        return cfg_path;
    }
    PathBuf::from("assets/skill-command.md")
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
        if !trimmed.starts_with('|') || trimmed.contains(":---") || trimmed.contains("Nama Skill") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 4 {
            let name = parts[1].to_string();
            let description = parts[2].to_string();
            let command = parts[3].replace('`', "");

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
        "# ⚡ Skill Command Registry\n\n\
Daftar perintah cepat dan otomasi sistem yang dapat dijalankan langsung melalui suara, menu, atau otomatisasi AI.\n\
File ini dapat diedit langsung atau dikelola melalui UI Skill Commands.\n\n\
| Nama Skill | Deskripsi | Command / Script |\n\
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

pub fn execute_command_by_os(cmd: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", "cmd", "/C", cmd])
            .spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("sh")
            .args(["-c", cmd])
            .spawn()?;
    }
    Ok(())
}
