use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

static APPS_CACHE: Mutex<Option<(Instant, Vec<InstalledApp>)>> = Mutex::new(None);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledApp {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
}

/// Mengambil daftar aplikasi terpasang dengan cache memori 60 detik agar tidak membebani I/O disk
pub fn get_installed_applications_cached() -> Vec<InstalledApp> {
    if let Ok(mut guard) = APPS_CACHE.lock() {
        if let Some((instant, ref apps)) = *guard {
            if instant.elapsed().as_secs() < 60 {
                return apps.clone();
            }
        }
        let fresh_apps = get_installed_applications();
        *guard = Some((Instant::now(), fresh_apps.clone()));
        return fresh_apps;
    }
    get_installed_applications()
}

/// Format daftar aplikasi terpasang dengan filtering terindeks sesuai konteks input pengguna
pub fn format_installed_apps_indexed(query: Option<&str>) -> String {
    let apps = get_installed_applications_cached();
    if apps.is_empty() {
        return String::new();
    }

    let q_clean = query.unwrap_or("").trim().to_lowercase();
    let words: Vec<&str> = q_clean.split_whitespace().collect();

    // Jika ada query spesifik, cari yang cocok dengan salah satu kata dalam ucapan pengguna
    let matched_apps: Vec<&str> = if !words.is_empty() {
        let stop_words = ["buka", "bukakan", "tolong", "coba", "jalankan", "start", "open", "launch", "nyalakan", "aplikasi", "dong", "ya", "sih", "lah", "nya", "ke", "di"];
        let target_keywords: Vec<&str> = words.into_iter().filter(|w| !stop_words.contains(w) && w.len() >= 2).collect();

        if !target_keywords.is_empty() {
            apps.iter()
                .filter(|a| {
                    let name_lower = a.name.to_lowercase();
                    target_keywords.iter().any(|k| name_lower.contains(k))
                })
                .map(|a| a.name.as_str())
                .collect()
        } else {
            apps.iter().take(30).map(|a| a.name.as_str()).collect()
        }
    } else {
        apps.iter().take(30).map(|a| a.name.as_str()).collect()
    };

    if matched_apps.is_empty() {
        // Fallback: tampilkan sebagian aplikasi umum
        let sample: Vec<&str> = apps.iter().take(20).map(|a| a.name.as_str()).collect();
        return format!(
            "\n### 📦 APLIKASI TERPASANG DI SISTEM (GROUND OF TRUTH):\n\
            * Bisa langsung dibuka dengan `[CMD:app:open|NamaApp]`:\n  {}\n",
            sample.join(", ")
        );
    }

    format!(
        "\n### 📦 APLIKASI TERPASANG DI SISTEM (GROUND OF TRUTH):\n\
        * Bisa langsung dibuka dengan `[CMD:app:open|NamaApp]`:\n  {}\n",
        matched_apps.join(", ")
    )
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

        // Aplikasi bawaan Windows esensial (sebagai jaminan selalu ada & bisa dibuka langsung)
        let default_windows_apps = [
            ("Calculator", "calc.exe", "Windows Calculator"),
            ("Kalkulator", "calc.exe", "Kalkulator Windows"),
            ("Notepad", "notepad.exe", "Aplikasi Catatan Notepad"),
            ("Paint", "mspaint.exe", "Aplikasi Gambar Paint"),
            ("Task Manager", "taskmgr.exe", "Windows Task Manager"),
            ("Settings", "explorer.exe ms-settings:", "Windows Settings"),
            ("Pengaturan", "explorer.exe ms-settings:", "Pengaturan Windows"),
            ("File Explorer", "explorer.exe", "Windows File Explorer"),
            ("Command Prompt", "cmd.exe", "Windows Command Prompt"),
            ("PowerShell", "powershell.exe", "Windows PowerShell"),
            ("Microsoft Edge", "explorer.exe microsoft-edge:", "Browser Microsoft Edge"),
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
            scan_lnk_dir(&path, apps, seen_names);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if ext_lower == "lnk" || ext_lower == "url" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let name = stem.to_string();
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
                    let exec = format!("explorer.exe \"{}\"", path.display());
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
