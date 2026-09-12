use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: String,
    pub category: String,
}

fn get_skills_path() -> std::path::PathBuf {
    let local = std::path::PathBuf::from("skill-command.md");
    if local.exists() {
        return local;
    }
    let p = crate::config::get_app_config_dir().join("skill-command.md");
    if !p.exists() {
        // Buat file skill default jika belum ada
        let default_content = "# ⚡ Skill Command Registry\n\n\
Daftar perintah cepat dan otomasi sistem yang dapat dijalankan langsung melalui suara, menu popover, atau terminal.\n\n\
| ID | Nama Skill | Deskripsi | Command / Script | Kategori |\n\
| :--- | :--- | :--- | :--- | :--- |\n\
| `open_browser` | Buka Browser | Membuka web browser bawaan sistem | `xdg-open https://google.com` | Web |\n\
| `system_calc` | Buka Kalkulator | Membuka aplikasi kalkulator sistem | `gnome-calculator` | Sistem |\n";
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
        // parts[0] is empty, parts[1]=ID, parts[2]=Name, parts[3]=Desc, parts[4]=Cmd, parts[5]=Cat
        if parts.len() >= 6 {
            let id = parts[1].replace('`', "");
            let name = parts[2].to_string();
            let description = parts[3].to_string();
            let command = parts[4].replace('`', "");
            let category = parts[5].to_string();

            if !id.is_empty() && !command.is_empty() {
                skills.push(SkillItem {
                    id,
                    name,
                    description,
                    command,
                    category,
                });
            }
        }
    }
    skills
}

pub fn save_skills_markdown(skills: &[SkillItem]) -> std::io::Result<()> {
    let mut out = String::from(
        "# ⚡ Skill Command Registry\n\n\
Daftar perintah cepat dan otomasi sistem yang dapat dijalankan langsung melalui suara, menu popover, atau terminal.\n\
File ini dapat diedit langsung oleh manusia atau dikelola melalui UI Skill Command.\n\n\
| ID | Nama Skill | Deskripsi | Command / Script | Kategori |\n\
| :--- | :--- | :--- | :--- | :--- |\n",
    );

    for s in skills {
        out.push_str(&format!(
            "| `{}` | {} | {} | `{}` | {} |\n",
            s.id.trim(),
            s.name.trim(),
            s.description.trim(),
            s.command.trim(),
            s.category.trim()
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
    if let Some(pos) = skills.iter().position(|s| s.id == skill.id) {
        skills[pos] = skill;
    } else {
        skills.push(skill);
    }
    save_skills_markdown(&skills).map_err(|e| e.to_string())?;
    Ok(skills)
}

#[tauri::command]
pub fn delete_skill(id: String) -> Result<Vec<SkillItem>, String> {
    let mut skills = parse_skills_markdown();
    skills.retain(|s| s.id != id);
    save_skills_markdown(&skills).map_err(|e| e.to_string())?;
    Ok(skills)
}

#[tauri::command]
pub fn execute_skill(id: String) -> Result<String, String> {
    let skills = parse_skills_markdown();
    if let Some(skill) = skills.iter().find(|s| s.id == id) {
        crate::menu::execute_command_by_os(&skill.command)
            .map_err(|e| format!("Gagal menjalankan command: {e}"))?;

        Ok("Perintah berhasil dijalankan.".to_string())
    } else {
        Err(format!("Skill dengan ID '{id}' tidak ditemukan"))
    }
}
