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
    #[allow(unused_mut)]
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

        apps.sort_by_key(|a| a.name.to_lowercase());
    }

    #[cfg(target_os = "windows")]
    {
        use std::collections::HashSet;
        let mut seen_names = HashSet::new();

        // Scan shortcuts (.lnk dan .url) dari folder Start Menu
        let mut start_menu_dirs: Vec<PathBuf> = Vec::new();

        // Start Menu semua user
        if let Ok(program_data) =
            std::env::var("ProgramData").or_else(|_| std::env::var("ALLUSERSPROFILE"))
        {
            start_menu_dirs.push(
                PathBuf::from(&program_data).join("Microsoft\\Windows\\Start Menu\\Programs"),
            );
        }
        // Start Menu user saat ini
        if let Ok(appdata) = std::env::var("APPDATA") {
            start_menu_dirs
                .push(PathBuf::from(&appdata).join("Microsoft\\Windows\\Start Menu\\Programs"));
        } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
            start_menu_dirs.push(
                PathBuf::from(&userprofile)
                    .join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs"),
            );
        }

        for dir in &start_menu_dirs {
            scan_lnk_dir(dir, &mut apps, &mut seen_names);
        }

        // Aplikasi bawaan Windows esensial (sebagai jaminan selalu ada & bisa dibuka)
        let default_windows_apps = [
            ("Notepad", "notepad.exe", "Aplikasi Catatan Notepad"),
            ("Kalkulator", "calc.exe", "Kalkulator Windows"),
            ("Paint", "mspaint.exe", "Aplikasi Gambar Paint"),
            ("Task Manager", "taskmgr.exe", "Windows Task Manager"),
            ("Pengaturan", "start ms-settings:", "Windows Settings"),
            ("File Explorer", "explorer.exe", "Windows File Explorer"),
            ("Command Prompt", "cmd.exe", "Windows Command Prompt"),
            ("PowerShell", "powershell.exe", "Windows PowerShell"),
            ("Microsoft Edge", "start msedge:", "Browser Microsoft Edge"),
        ];

        for (name, exec, desc) in default_windows_apps {
            let key = name.to_lowercase();
            if !seen_names.contains(&key) {
                seen_names.insert(key);
                apps.push(InstalledApp {
                    name: name.to_string(),
                    exec: exec.to_string(),
                    icon: None,
                    comment: Some(desc.to_string()),
                });
            }
        }

        apps.sort_by_key(|a| a.name.to_lowercase());
    }

    #[cfg(target_os = "macos")]
    {
        use std::collections::HashSet;
        let mut seen_names = HashSet::new();

        let mut dirs = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
            PathBuf::from("/System/Applications/Utilities"),
        ];

        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(&home).join("Applications"));
        }

        for dir in &dirs {
            scan_app_bundle_dir(dir, &mut apps, &mut seen_names);
        }

        apps.sort_by_key(|a| a.name.to_lowercase());
    }

    apps
}

#[cfg(target_os = "macos")]
fn scan_app_bundle_dir(
    dir: &PathBuf,
    apps: &mut Vec<InstalledApp>,
    seen_names: &mut std::collections::HashSet<String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("app"))
            .unwrap_or(false)
        {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let name = stem.to_string();
                let key = name.to_lowercase();
                if seen_names.contains(&key) {
                    continue;
                }
                seen_names.insert(key);
                // `open` adalah cara standar buka .app bundle di macOS
                let exec = format!("open -a \"{}\"", name);
                apps.push(InstalledApp {
                    name,
                    exec,
                    icon: None,
                    comment: None,
                });
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn scan_lnk_dir(
    dir: &PathBuf,
    apps: &mut Vec<InstalledApp>,
    seen_names: &mut std::collections::HashSet<String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Rekursif masuk subfolder (misal "Accessories", "Games", dll.)
            scan_lnk_dir(&path, apps, seen_names);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if ext_lower == "lnk" || ext_lower == "url" {
                // Ambil nama dari filename tanpa ekstensi
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let name = stem.to_string();
                    // Skip uninstaller / entry yang biasanya tidak relevan
                    let lower = name.to_lowercase();
                    if lower.contains("uninstall")
                        || lower.contains("readme")
                        || lower.contains("release note")
                        || lower.contains("help")
                        || lower.contains("documentation")
                    {
                        continue;
                    }
                    let key = lower.clone();
                    if seen_names.contains(&key) {
                        continue;
                    }
                    seen_names.insert(key);
                    // Gunakan start command untuk membuka shortcut
                    let exec = format!("start \"\" \"{}\"", path.display());
                    apps.push(InstalledApp {
                        name,
                        exec,
                        icon: None,
                        comment: None,
                    });
                }
            }
        }
    }
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
                    let field_codes = [
                        "%f",
                        "%F",
                        "%u",
                        "%U",
                        "%d",
                        "%D",
                        "%n",
                        "%N",
                        "%i",
                        "%c",
                        "%k",
                        "%v",
                        "%m",
                        "@@u %U @@",
                        "@@",
                    ];
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
    // Gunakan kandidat standar (Lokal project assets/ -> beside exe -> Resources macOS -> AppData/Roaming)
    let candidate = crate::assets::find_file_in_candidates("skill-command.md");
    if candidate.exists() {
        return candidate;
    }

    // Default target jika belum ada
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

/// Helper utilitas untuk mengonfigurasi subprocess agar terisolasi dari AppImage runtime
#[allow(unused_variables)]
pub fn configure_subprocess(cmd: &mut std::process::Command) {
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
        let mut command = std::process::Command::new("cmd");
        command.args(["/C", cmd]);
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
