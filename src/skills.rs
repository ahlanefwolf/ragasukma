use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledApp {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
}

pub fn get_installed_applications() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    #[cfg(target_os = "linux")]
    {
        use std::collections::HashSet;
        let mut dirs = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        ];

        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(&home);
            dirs.push(p.join(".local/share/applications"));
            dirs.push(p.join(".local/share/flatpak/exports/share/applications"));
        }

        let mut seen_names = HashSet::new();

        for dir in dirs {
            if !dir.exists() || !dir.is_dir() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                        if let Some(app) = parse_desktop_file(&path) {
                            let key = app.name.to_lowercase();
                            if !seen_names.contains(&key) {
                                seen_names.insert(key);
                                apps.push(app);
                            }
                        }
                    }
                }
            }
        }

        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    apps
}

#[cfg(target_os = "linux")]
fn parse_desktop_file(path: &std::path::Path) -> Option<InstalledApp> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut in_desktop_entry = false;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut comment = None;
    let mut no_display = false;
    let mut is_application = true;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_desktop_entry = trimmed == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim();
            let val = v.trim();
            match key {
                "Type" => {
                    if val != "Application" {
                        is_application = false;
                    }
                }
                "NoDisplay" | "Hidden" => {
                    if val.eq_ignore_ascii_case("true") {
                        no_display = true;
                    }
                }
                "Name" if name.is_none() => {
                    name = Some(val.to_string());
                }
                "Exec" if exec.is_none() => {
                    let mut cleaned = val.to_string();
                    let field_codes = ["%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m", "@@u %U @@", "@@"];
                    for fc in field_codes {
                        cleaned = cleaned.replace(fc, "");
                    }
                    cleaned = cleaned.trim().to_string();
                    if !cleaned.is_empty() {
                        exec = Some(cleaned);
                    }
                }
                "Icon" if icon.is_none() => {
                    icon = Some(val.to_string());
                }
                "Comment" if comment.is_none() => {
                    comment = Some(val.to_string());
                }
                _ => {}
            }
        }
    }

    if no_display || !is_application {
        return None;
    }

    let name = name?;
    let exec = exec?;
    if name.is_empty() || exec.is_empty() {
        return None;
    }

    Some(InstalledApp {
        name,
        exec,
        icon,
        comment,
    })
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
        if parts.len() >= 6 {
            // Jika ada 4 kolom (dengan kolom Tipe lama): | Nama | Tipe | Deskripsi | Command |
            let name = parts[1].to_string();
            let description = parts[3].to_string();
            let command = parts[4].replace('`', "");

            if !name.is_empty() && !command.is_empty() {
                skills.push(SkillItem {
                    name,
                    description,
                    command,
                });
            }
        } else if parts.len() >= 4 {
            // Standar 3 kolom: | Nama Skill | Deskripsi | Command |
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
