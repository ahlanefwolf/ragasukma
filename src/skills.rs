use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    pub command: String,
}

fn get_skills_path() -> std::path::PathBuf {
    let local = std::path::PathBuf::from("skill-command.md");
    if local.exists() {
        return local;
    }
    let p = crate::config::get_app_config_dir().join("skill-command.md");
    if !p.exists() {
        let default_content = "# ⚡ Skill Command Registry\n\n\
Daftar perintah cepat dan otomasi sistem yang dapat dijalankan langsung melalui suara, menu, atau otomatisasi AI.\n\n\
| Nama Skill | Deskripsi | Command / Script |\n\
| :--- | :--- | :--- |\n\
| Buka Browser | Membuka web browser bawaan sistem | `xdg-open https://google.com` |\n\
| Buka Kalkulator | Membuka aplikasi kalkulator sistem | `gnome-calculator` |\n";
        let _ = std::fs::write(&p, default_content);
    }
    p
}

pub fn parse_skills_markdown() -> Vec<SkillItem> {
    let path = get_skills_path();
    if !path.exists() {
        return Vec::new();
    }
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
        // Format baru 3 kolom: parts[0]="", parts[1]=Name, parts[2]=Desc, parts[3]=Cmd
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
        // Kompatibilitas mundur jika ada file format lama 5 kolom (ID | Name | Desc | Cmd | Cat)
        else if parts.len() >= 5 {
            let name = parts[2].to_string();
            let description = parts[3].to_string();
            let command = parts[4].replace('`', "");

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

#[tauri::command]
pub fn get_skills() -> Vec<SkillItem> {
    parse_skills_markdown()
}

#[tauri::command]
pub fn save_skill(skill: SkillItem) -> Result<Vec<SkillItem>, String> {
    let mut skills = parse_skills_markdown();
    if let Some(pos) = skills.iter().position(|s| s.name.eq_ignore_ascii_case(&skill.name)) {
        skills[pos] = skill;
    } else {
        skills.push(skill);
    }
    save_skills_markdown(&skills).map_err(|e| e.to_string())?;
    crate::tray::TrayState::emit_event("assistant:skills-updated", serde_json::to_value(&skills).unwrap_or_default());
    Ok(skills)
}

#[tauri::command]
pub fn delete_skill(name: String) -> Result<Vec<SkillItem>, String> {
    let mut skills = parse_skills_markdown();
    skills.retain(|s| !s.name.eq_ignore_ascii_case(&name));
    save_skills_markdown(&skills).map_err(|e| e.to_string())?;
    crate::tray::TrayState::emit_event("assistant:skills-updated", serde_json::to_value(&skills).unwrap_or_default());
    Ok(skills)
}

#[tauri::command]
pub fn execute_skill(name: String) -> Result<String, String> {
    let skills = parse_skills_markdown();
    if let Some(skill) = skills.iter().find(|s| s.name.eq_ignore_ascii_case(&name)) {
        crate::menu::execute_command_by_os(&skill.command)
            .map_err(|e| format!("Gagal menjalankan command: {e}"))?;

        Ok("Perintah berhasil dijalankan.".to_string())
    } else {
        Err(format!("Skill '{name}' tidak ditemukan"))
    }
}
