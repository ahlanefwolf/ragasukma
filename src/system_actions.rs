use crate::skills::{execute_command_by_os, get_installed_applications, SkillItem};

/// Daftar aksi sistem bawaan yang otomatis aktif sesuai OS (Linux / Windows)
pub fn get_builtin_system_skills() -> Vec<SkillItem> {
    #[cfg(target_os = "windows")]
    {
        vec![
            SkillItem {
                name: "Buka Browser".to_string(),
                description: "Membuka browser web default untuk internet".to_string(),
                command: "start https://www.google.com".to_string(),
            },
            SkillItem {
                name: "Buka File Manager".to_string(),
                description: "Membuka File Explorer Windows".to_string(),
                command: "explorer".to_string(),
            },
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume suara komputer".to_string(),
                command: "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]175)\"".to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume suara komputer".to_string(),
                command: "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]174)\"".to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara komputer (mute/unmute)".to_string(),
                command: "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]173)\"".to_string(),
            },
            SkillItem {
                name: "Kunci Layar".to_string(),
                description: "Mengunci layar komputer / lock workstation".to_string(),
                command: "rundll32.exe user32.dll,LockWorkStation".to_string(),
            },
            SkillItem {
                name: "Buka Kalkulator".to_string(),
                description: "Membuka aplikasi kalkulator sistem".to_string(),
                command: "calc.exe".to_string(),
            },
            SkillItem {
                name: "Buka Notepad".to_string(),
                description: "Membuka aplikasi catatan Notepad".to_string(),
                command: "notepad.exe".to_string(),
            },
        ]
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec![
            SkillItem {
                name: "Buka Browser".to_string(),
                description: "Membuka browser internet utama".to_string(),
                command: "xdg-open https://www.google.com || google-chrome || firefox".to_string(),
            },
            SkillItem {
                name: "Buka File Manager".to_string(),
                description: "Membuka pengelola berkas / file manager utama".to_string(),
                command: "xdg-open ~ || nautilus ~ || thunar ~ || dolphin ~".to_string(),
            },
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume suara laptop/komputer (+10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ +10% || amixer set Master 10%+".to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume suara laptop/komputer (-10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ -10% || amixer set Master 10%-".to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara audio komputer".to_string(),
                command: "pactl set-sink-mute @DEFAULT_SINK@ toggle || amixer set Master toggle".to_string(),
            },
            SkillItem {
                name: "Ambil Screenshot".to_string(),
                description: "Mengambil tangkapan layar monitor komputer".to_string(),
                command: "gnome-screenshot || spectacle || flameshot gui || xfce4-screenshooter".to_string(),
            },
            SkillItem {
                name: "Kunci Layar".to_string(),
                description: "Mengunci sesi layar komputer".to_string(),
                command: "loginctl lock-session || xdg-screensaver lock || gnome-screensaver-command -l".to_string(),
            },
            SkillItem {
                name: "Buka Terminal".to_string(),
                description: "Membuka aplikasi terminal/konsol sistem".to_string(),
                command: "x-terminal-emulator || gnome-terminal || konsole || alacritty || xterm".to_string(),
            },
            SkillItem {
                name: "Buka Kalkulator".to_string(),
                description: "Membuka aplikasi kalkulator sistem".to_string(),
                command: "gnome-calculator || kcalc || galculator || xcalc".to_string(),
            },
        ]
    }
}

/// Eksekusi aksi sistem, buka URL/web dinamis, atau cari aplikasi terinstal secara dinamis
pub fn try_execute_builtin_or_app(target: &str) -> Option<(String, String)> {
    let target_trim = target.trim();

    // 1. Deteksi Perintah Buka Website atau URL Dinamis
    // Format: "Buka Website|https://..." atau "Open Website|https://..." atau langsung format URL
    if target_trim.to_lowercase().starts_with("buka website|")
        || target_trim.to_lowercase().starts_with("open website|")
    {
        if let Some((_, raw_url)) = target_trim.split_once('|') {
            let url = raw_url.trim();
            let final_url = if !url.starts_with("http://") && !url.starts_with("https://") {
                format!("https://{url}")
            } else {
                url.to_string()
            };
            return match open_url_by_os(&final_url) {
                Ok(_) => Some((format!("Buka Website: {final_url}"), "Berhasil membuka link web".to_string())),
                Err(e) => Some((format!("Buka Website: {final_url}"), format!("Error: {e}"))),
            };
        }
    }

    // 2. Deteksi Perintah Pencarian Google Dinamis
    // Format: "Cari Google|kata kunci" atau "Search Google|query"
    if target_trim.to_lowercase().starts_with("cari google|")
        || target_trim.to_lowercase().starts_with("search google|")
    {
        if let Some((_, query)) = target_trim.split_once('|') {
            let encoded_query = query.trim().replace(' ', "+");
            let final_url = format!("https://www.google.com/search?q={encoded_query}");
            return match open_url_by_os(&final_url) {
                Ok(_) => Some((format!("Cari Google: {}", query.trim()), "Berhasil membuka pencarian Google".to_string())),
                Err(e) => Some((format!("Cari Google: {}", query.trim()), format!("Error: {e}"))),
            };
        }
    }

    let target_clean = target_trim.to_lowercase().replace(['_', '-'], " ");

    // 3. Cek dari daftar built-in system skills
    for item in get_builtin_system_skills() {
        let name_clean = item.name.trim().to_lowercase();
        if name_clean == target_clean || name_clean.replace(['_', '-'], " ") == target_clean {
            match execute_command_by_os(&item.command) {
                Ok(_) => return Some((item.name, "Berhasil dijalankan".to_string())),
                Err(e) => return Some((item.name, format!("Error: {e}"))),
            }
        }
    }

    // 4. Cek apakah ini permintaan buka aplikasi (misal "Buka Chrome", "Buka Steam", atau "Steam")
    let app_query = if target_clean.starts_with("buka ") {
        target_clean.trim_start_matches("buka ").trim()
    } else if target_clean.starts_with("open ") {
        target_clean.trim_start_matches("open ").trim()
    } else {
        &target_clean
    };

    let installed_apps = get_installed_applications();
    for app in installed_apps {
        let app_name_clean = app.name.trim().to_lowercase();
        if app_name_clean == app_query
            || app_name_clean == target_clean
            || app_name_clean.replace(['_', '-'], " ") == app_query
        {
            match execute_command_by_os(&app.exec) {
                Ok(_) => {
                    return Some((
                        format!("Buka {}", app.name),
                        "Berhasil membuka aplikasi".to_string(),
                    ))
                }
                Err(e) => return Some((format!("Buka {}", app.name), format!("Error: {e}"))),
            }
        }
    }

    None
}

/// Helper untuk membuka URL secara aman di browser default pengguna
pub fn open_url_by_os(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }
    Ok(())
}
