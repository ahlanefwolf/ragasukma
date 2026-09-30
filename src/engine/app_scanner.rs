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
    pub generic_name: Option<String>,
    pub keywords: Vec<String>,
    pub desktop_id: Option<String>,
    pub desktop_path: Option<String>,
}

impl InstalledApp {
    pub fn display_label(&self) -> String {
        if let Some(ref generic) = self.generic_name {
            // Unicode-safe comparison: covers non-ASCII names (Cyrillic, CJK, etc.)
            if !generic.is_empty() && generic.to_lowercase() != self.name.to_lowercase() {
                return format!("{} ({})", self.name, generic);
            }
        }
        self.name.clone()
    }
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

// ── Category word helpers ─────────────────────────────────────────────────────
// Each fn returns true when the given word (already lowercased) belongs to that
// category in any of the 30 languages supported by Chrome STT / SUPPORTED_LANGUAGES.

fn is_terminal_word(w: &str) -> bool {
    matches!(
        w,
        // Latin-script languages
        "terminal" | "console" | "shell" | "bash"
        // id
        | "konsol"
        // de, fr, es, pt, pl, nl, sv, da, cs, sk, sl, ro, hr, hu, lt, lv, et, fi, vi — all use "terminal"
        // fi
        | "pääte"
        // cs, sk, hu
        | "terminál"
        // Non-Latin scripts
        | "ターミナル"          // ja
        | "터미널"              // ko
        | "طرفية" | "طرفي"    // ar
        | "терминал"           // ru, uk, bg
        | "终端" | "終端"       // zh
        | "टर्मिनल"            // hi
        | "τερματικό"          // el
    )
}

fn is_browser_word(w: &str) -> bool {
    matches!(
        w,
        "browser" | "web" | "internet"
        | "peramban"                    // id
        | "navigateur"                  // fr
        | "navegador"                   // es, pt
        | "webbläsare"                  // sv
        | "selain"                      // fi
        | "prohlížeč"                   // cs
        | "prehliadač"                  // sk
        | "brskalnik"                   // sl
        | "preglednik"                  // hr
        | "böngésző"                    // hu
        | "naršyklė"                    // lt
        | "pārlūks"                     // lv
        | "przeglądarka"                // pl
        | "trình duyệt"                 // vi
        | "ブラウザ" | "ブラウザー"      // ja
        | "브라우저"                    // ko
        | "متصفح"                       // ar
        | "браузер"                     // ru, uk
        | "браузър"                     // bg
        | "浏览器" | "瀏覽器"           // zh
        | "ब्राउज़र"                    // hi
        | "πρόγραμμα περιήγησης"        // el
    )
}

fn is_editor_word(w: &str) -> bool {
    matches!(
        w,
        "editor" | "code" | "text"
        | "kode" | "teks"               // id
        | "éditeur"                     // fr
        | "textredigerare"              // sv
        | "tekstieditori"               // fi
        | "urejevalnik"                 // sl
        | "uređivač"                    // hr
        | "szövegszerkesztő"            // hu
        | "redaktorius"                 // lt
        | "redaktors"                   // lv
        | "trình soạn thảo"             // vi
        | "エディタ" | "エディター"      // ja
        | "편집기"                      // ko
        | "محرر"                        // ar
        | "редактор"                    // ru, uk, bg
        | "编辑器" | "編輯器"           // zh
        | "संपादक"                      // hi
        | "επεξεργαστής"                // el
    )
}

fn is_filemanager_word(w: &str) -> bool {
    matches!(
        w,
        "file" | "folder" | "explorer" | "manager"
        | "berkas"                      // id
        | "datei" | "dateien"           // de
        | "fichier" | "fichiers"        // fr
        | "archivo" | "archivos"        // es
        | "pliki" | "plik"              // pl
        | "bestand" | "bestanden"       // nl
        | "filer" | "fil"               // sv, da, no
        | "tiedosto" | "tiedostot"      // fi
        | "soubor" | "soubory"          // cs
        | "súbor" | "súbory"            // sk
        | "datoteka" | "datoteke"       // sl, hr
        | "fișier" | "fișiere"          // ro
        | "fájl" | "fájlok"             // hu
        | "failas" | "failai"           // lt
        | "fails" | "faili"             // lv
        | "fail" | "failid"             // et
        | "tệp"                         // vi
        | "ファイル"                    // ja
        | "파일"                        // ko
        | "ملفات" | "ملف"               // ar
        | "файлы" | "файл"              // ru, uk, bg
        | "文件" | "檔案"               // zh
        | "फ़ाइल"                       // hi
        | "αρχεία" | "αρχείο"           // el
    )
}

// ── App matching logic ─────────────────────────────────────────────────────────

fn app_is_terminal(app: &InstalledApp, gen_lower: &str, id_lower: &str) -> bool {
    gen_lower.contains("terminal")
        || id_lower.contains("terminal")
        || id_lower == "foot"
        || id_lower == "alacritty"
        || id_lower == "kitty"
        || id_lower == "wezterm"
        || id_lower == "konsole"
        || id_lower == "xterm"
        || app
            .keywords
            .iter()
            .any(|kw| kw.contains("terminal") || kw.eq_ignore_ascii_case("terminalemulator"))
}

fn app_is_browser(app: &InstalledApp, gen_lower: &str, id_lower: &str) -> bool {
    gen_lower.contains("browser")
        || id_lower.contains("chrome")
        || id_lower.contains("firefox")
        || id_lower.contains("chromium")
        || id_lower.contains("brave")
        || id_lower.contains("edge")
        || app
            .keywords
            .iter()
            .any(|kw| kw.eq_ignore_ascii_case("webbrowser"))
}

fn app_is_editor(app: &InstalledApp, gen_lower: &str, id_lower: &str) -> bool {
    gen_lower.contains("editor")
        || id_lower.contains("code")
        || id_lower.contains("zed")
        || id_lower.contains("gedit")
        || id_lower.contains("kate")
        || id_lower.contains("sublime")
        || app
            .keywords
            .iter()
            .any(|kw| kw.eq_ignore_ascii_case("texteditor"))
}

fn app_is_filemanager(app: &InstalledApp, gen_lower: &str, id_lower: &str) -> bool {
    gen_lower.contains("file manager")
        || id_lower.contains("thunar")
        || id_lower.contains("nautilus")
        || id_lower.contains("dolphin")
        || id_lower.contains("pcmanfm")
        || id_lower.contains("nemo")
        || id_lower.contains("explorer")
        || app
            .keywords
            .iter()
            .any(|kw| kw.eq_ignore_ascii_case("filemanager"))
}

/// Mengecek apakah aplikasi cocok dengan kata kunci pencarian termasuk alias kategori generik
pub fn is_app_matching_keywords(app: &InstalledApp, keywords: &[&str]) -> bool {
    let name_lower = app.name.to_lowercase();
    let gen_lower = app.generic_name.as_deref().unwrap_or("").to_lowercase();
    let id_lower = app.desktop_id.as_deref().unwrap_or("").to_lowercase();
    let comment_lower = app.comment.as_deref().unwrap_or("").to_lowercase();

    for k in keywords {
        if name_lower.contains(k)
            || gen_lower.contains(k)
            || id_lower.contains(k)
            || comment_lower.contains(k)
            || app.keywords.iter().any(|kw| kw.contains(k))
        {
            return true;
        }

        // Generic category aliases across all supported languages
        if is_terminal_word(k) && app_is_terminal(app, &gen_lower, &id_lower) {
            return true;
        }
        if is_browser_word(k) && app_is_browser(app, &gen_lower, &id_lower) {
            return true;
        }
        if is_editor_word(k) && app_is_editor(app, &gen_lower, &id_lower) {
            return true;
        }
        if is_filemanager_word(k) && app_is_filemanager(app, &gen_lower, &id_lower) {
            return true;
        }
    }
    false
}

/// Mengecek apakah suatu aplikasi disebut di dalam kalimat input pengguna secara agnostik bahasa
pub fn is_app_mentioned_in_query(app: &InstalledApp, query: &str) -> bool {
    let name_lower = app.name.trim().to_lowercase();
    if name_lower.is_empty() {
        return false;
    }

    // 1. Nama lengkap aplikasi muncul di query (misal: "google chrome", "visual studio code", "zed", "spotify")
    if name_lower.len() >= 2 && query.contains(&name_lower) {
        return true;
    }

    // 2. Token utama nama aplikasi (panjang >= 3, bukan kata umum/noise)
    for part in name_lower.split_whitespace() {
        let clean_part = part.trim_matches(|c: char| !c.is_alphanumeric());
        // Noise: common short words across supported languages that appear in app names
        let is_noise = matches!(
            clean_part,
            "the" | "app" | "pro" | "for" | "and" // en
            | "de" | "von" | "für"                 // de
            | "le" | "la" | "les" | "et"           // fr
            | "el" | "los" | "las" | "por"         // es
            | "van" | "voor"                        // nl
            | "för"                                 // sv
        );
        if !is_noise && clean_part.len() >= 3 && query.contains(clean_part) {
            return true;
        }
    }

    // 3. Desktop ID (misal: "dev.zed.Zed" -> "zed", "google-chrome" -> "chrome")
    if let Some(ref id) = app.desktop_id {
        let id_clean = id.trim_end_matches(".desktop").to_lowercase();
        let base_id = id_clean.rsplit('.').next().unwrap_or(&id_clean);
        if base_id.len() >= 3 && query.contains(base_id) {
            return true;
        }
    }

    // 4. Kategori generik umum lintas bahasa (terminal, browser, editor, file manager)
    let gen_lower = app.generic_name.as_deref().unwrap_or("").to_lowercase();
    let id_lower = app.desktop_id.as_deref().unwrap_or("").to_lowercase();

    let query_has_terminal = query.split_whitespace().any(is_terminal_word);
    let query_has_browser = query.split_whitespace().any(is_browser_word);
    let query_has_editor = query.split_whitespace().any(is_editor_word);
    let query_has_filemanager = query.split_whitespace().any(is_filemanager_word);

    if query_has_terminal && app_is_terminal(app, &gen_lower, &id_lower) {
        return true;
    }
    if query_has_browser && app_is_browser(app, &gen_lower, &id_lower) {
        return true;
    }
    if query_has_editor && app_is_editor(app, &gen_lower, &id_lower) {
        return true;
    }
    if query_has_filemanager && app_is_filemanager(app, &gen_lower, &id_lower) {
        return true;
    }

    false
}

/// Format daftar aplikasi terpasang dengan filtering terindeks sesuai konteks input pengguna
pub fn format_installed_apps_indexed(query: Option<&str>) -> String {
    let apps = get_installed_applications_cached();
    if apps.is_empty() {
        return String::new();
    }

    let q_clean = query.unwrap_or("").trim().to_lowercase();

    let matched_apps: Vec<String> = if !q_clean.is_empty() {
        let matches: Vec<String> = apps
            .iter()
            .filter(|a| is_app_mentioned_in_query(a, &q_clean))
            .map(|a| a.display_label())
            .collect();

        if matches.is_empty() {
            apps.iter().take(30).map(|a| a.display_label()).collect()
        } else {
            matches
        }
    } else {
        apps.iter().take(30).map(|a| a.display_label()).collect()
    };

    format!(
        "\n### 📦 INSTALLED SYSTEM APPLICATIONS (GROUND OF TRUTH):\n\
        * Can be opened directly with `[CMD:app:open|AppName]`:\n  {}\n",
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
            ("Calculator",    "calc.exe"),
            ("Notepad",       "notepad.exe"),
            ("Paint",         "mspaint.exe"),
            ("Task Manager",  "taskmgr.exe"),
            ("Settings",      "explorer.exe ms-settings:"),
            ("File Explorer", "explorer.exe"),
            ("Command Prompt","cmd.exe"),
            ("PowerShell",    "powershell.exe"),
            ("Microsoft Edge","explorer.exe microsoft-edge:"),
        ];

        for (name, exec) in default_windows_apps {
            let key = name.to_lowercase();
            if !seen_names.contains(&key) {
                seen_names.insert(key);
                apps.push(InstalledApp {
                    name: name.to_string(),
                    exec: exec.to_string(),
                    icon: None,
                    comment: None,
                    generic_name: Some(name.to_string()),
                    keywords: Vec::new(),
                    desktop_id: Some(name.to_lowercase()),
                    desktop_path: None,
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
                    name: name.clone(),
                    exec,
                    icon: None,
                    comment: None,
                    generic_name: None,
                    keywords: Vec::new(),
                    desktop_id: Some(name.to_lowercase()),
                    desktop_path: Some(path.display().to_string()),
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
                        generic_name: None,
                        keywords: Vec::new(),
                        desktop_id: Some(lower),
                        desktop_path: Some(path.display().to_string()),
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
    let mut generic_name = None;
    let mut exec = None;
    let mut icon = None;
    let mut comment = None;
    let mut keywords = Vec::new();
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
                "GenericName" if generic_name.is_none() => {
                    generic_name = Some(val.to_string());
                }
                "Keywords" => {
                    for kw in val.split(';') {
                        let t = kw.trim();
                        if !t.is_empty() {
                            keywords.push(t.to_lowercase());
                        }
                    }
                }
                "Categories" => {
                    for cat in val.split(';') {
                        let t = cat.trim();
                        if !t.is_empty() {
                            keywords.push(t.to_lowercase());
                        }
                    }
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

    let desktop_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());
    let desktop_path = Some(path.to_string_lossy().to_string());

    Some(InstalledApp {
        name,
        exec,
        icon,
        comment,
        generic_name,
        keywords,
        desktop_id,
        desktop_path,
    })
}
