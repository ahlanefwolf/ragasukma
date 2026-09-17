use super::skills::{SkillItem, execute_command_by_os, get_installed_applications};

fn append_common_skills(skills: &mut Vec<SkillItem>) {
    skills.push(SkillItem {
        name: "Catat Tugas".to_string(),
        description: "Mencatat tugas atau agenda baru ke daftar to-do (assets/todo.md)".to_string(),
        command: "internal:todo_add".to_string(),
    });
    skills.push(SkillItem {
        name: "Selesai Tugas".to_string(),
        description: "Menandai tugas di daftar to-do sebagai selesai".to_string(),
        command: "internal:todo_done".to_string(),
    });
}

/// Daftar aksi sistem bawaan yang otomatis aktif sesuai OS (Linux / Windows)
pub fn get_builtin_system_skills() -> Vec<SkillItem> {
    #[cfg(target_os = "windows")]
    {
        let mut list = vec![
            SkillItem {
                name: "Buka File Manager".to_string(),
                description: "Membuka File Explorer Windows".to_string(),
                command: "explorer".to_string(),
            },
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume suara komputer".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]175)\""
                        .to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume suara komputer".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]174)\""
                        .to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara komputer (mute/unmute)".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]173)\""
                        .to_string(),
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
        ];
        append_common_skills(&mut list);
        list
    }

    #[cfg(target_os = "macos")]
    {
        let mut list = vec![
            SkillItem {
                name: "Buka File Manager".to_string(),
                description: "Membuka Finder macOS".to_string(),
                command: "open ~".to_string(),
            },
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume audio macOS (+10%)".to_string(),
                command: "osascript -e \"set volume output volume ((output volume of (get volume settings)) + 10)\"".to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume audio macOS (-10%)".to_string(),
                command: "osascript -e \"set volume output volume ((output volume of (get volume settings)) - 10)\"".to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara macOS (mute toggle)".to_string(),
                command: "osascript -e \"set volume output muted not (output muted of (get volume settings))\"".to_string(),
            },
            SkillItem {
                name: "Ambil Screenshot".to_string(),
                description: "Mengambil tangkapan layar monitor macOS".to_string(),
                command: "screencapture -i -c".to_string(),
            },
            SkillItem {
                name: "Kunci Layar".to_string(),
                description: "Mengunci sesi layar macOS".to_string(),
                command: "pmset displaysleepnow".to_string(),
            },
            SkillItem {
                name: "Buka Terminal".to_string(),
                description: "Membuka aplikasi Terminal macOS".to_string(),
                command: "open -a Terminal".to_string(),
            },
            SkillItem {
                name: "Buka Kalkulator".to_string(),
                description: "Membuka aplikasi Calculator macOS".to_string(),
                command: "open -a Calculator".to_string(),
            },
        ];
        append_common_skills(&mut list);
        list
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let mut list = vec![
            SkillItem {
                name: "Buka File Manager".to_string(),
                description: "Membuka pengelola berkas / file manager utama".to_string(),
                command: "xdg-open ~ || nautilus ~ || thunar ~ || dolphin ~".to_string(),
            },
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume suara laptop/komputer (+10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ +10% || amixer set Master 10%+"
                    .to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume suara laptop/komputer (-10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ -10% || amixer set Master 10%-"
                    .to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara audio komputer".to_string(),
                command: "pactl set-sink-mute @DEFAULT_SINK@ toggle || amixer set Master toggle"
                    .to_string(),
            },
            SkillItem {
                name: "Ambil Screenshot".to_string(),
                description: "Mengambil tangkapan layar monitor komputer".to_string(),
                command: "gnome-screenshot || spectacle || flameshot gui || xfce4-screenshooter"
                    .to_string(),
            },
            SkillItem {
                name: "Kunci Layar".to_string(),
                description: "Mengunci sesi layar komputer".to_string(),
                command:
                    "loginctl lock-session || xdg-screensaver lock || gnome-screensaver-command -l"
                        .to_string(),
            },
            SkillItem {
                name: "Buka Terminal".to_string(),
                description: "Membuka aplikasi terminal/konsol sistem".to_string(),
                command: "x-terminal-emulator || gnome-terminal || konsole || alacritty || xterm"
                    .to_string(),
            },
            SkillItem {
                name: "Buka Kalkulator".to_string(),
                description: "Membuka aplikasi kalkulator sistem".to_string(),
                command: "gnome-calculator || kcalc || galculator || xcalc".to_string(),
            },
        ];
        append_common_skills(&mut list);
        list
    }
}

/// Eksekusi aksi sistem, buka URL/web dinamis, atau cari aplikasi terinstal secara dinamis
pub fn try_execute_builtin_or_app(target: &str) -> Option<(String, String)> {
    let target_trim = target.trim();

    // 1. Deteksi Perintah Buka Website atau URL Dinamis
    // Format: "Buka Website|https://..." atau "Open Website|https://..." atau langsung format URL
    let lower_trim = target_trim.to_lowercase();
    if lower_trim.starts_with("buka website|") || lower_trim.starts_with("open website|") {
        if let Some((_, raw_url)) = target_trim.split_once('|') {
            let url = raw_url.trim();
            let final_url = if !url.starts_with("http://") && !url.starts_with("https://") {
                format!("https://{url}")
            } else {
                url.to_string()
            };
            return match open_url_by_os(&final_url) {
                Ok(_) => Some((
                    format!("Buka Website: {final_url}"),
                    "Berhasil membuka link web".to_string(),
                )),
                Err(e) => Some((format!("Buka Website: {final_url}"), format!("Error: {e}"))),
            };
        }
    }

    // 2. Deteksi Perintah Pencarian Google Dinamis
    // Format: "Cari Google|kata kunci" atau "Search Google|query"
    if lower_trim.starts_with("cari google|") || lower_trim.starts_with("search google|") {
        if let Some((_, query)) = target_trim.split_once('|') {
            let encoded_query = query.trim().replace(' ', "+");
            let final_url = format!("https://www.google.com/search?q={encoded_query}");
            return match open_url_by_os(&final_url) {
                Ok(_) => Some((
                    format!("Cari Google: {}", query.trim()),
                    "Berhasil membuka pencarian Google".to_string(),
                )),
                Err(e) => Some((
                    format!("Cari Google: {}", query.trim()),
                    format!("Error: {e}"),
                )),
            };
        }
    }

    // 3. Deteksi Perintah To-Do / Catat Tugas
    if lower_trim.starts_with("catat tugas|")
        || lower_trim.starts_with("tambah todo|")
        || lower_trim.starts_with("catat todo|")
        || lower_trim.starts_with("tambah tugas|")
    {
        if let Some((_, task)) = target_trim.split_once('|') {
            match super::todo::add_task(task.trim()) {
                Ok(msg) => return Some(("Catat Tugas".to_string(), msg)),
                Err(e) => return Some(("Catat Tugas".to_string(), format!("Error: {e}"))),
            }
        }
    }

    if lower_trim.starts_with("selesai tugas|")
        || lower_trim.starts_with("selesai todo|")
        || lower_trim.starts_with("selesaikan tugas|")
    {
        if let Some((_, query)) = target_trim.split_once('|') {
            match super::todo::complete_task(query.trim()) {
                Ok(msg) => return Some(("Selesai Tugas".to_string(), msg)),
                Err(e) => return Some(("Selesai Tugas".to_string(), format!("Error: {e}"))),
            }
        }
    }

    let target_clean = target_trim.to_lowercase().replace(['_', '-'], " ");

    // Putar musik (download via yt-dlp lalu putar)
    // Format: "putar musik|<query>" atau "play music|<query>"
    if (lower_trim.starts_with("putar musik|") || lower_trim.starts_with("play music|"))
        && let Some((_, query)) = target_trim.split_once('|')
    {
        let query = query.trim();
        if query.is_empty()
            || ["musik", "lagu", "music", "song", "audio"].contains(&query.to_lowercase().as_str())
        {
            return Some((
                "Putar Musik".to_string(),
                "Judul lagu belum ditentukan, silakan sebutkan judul lagunya.".to_string(),
            ));
        }

        let music_dir = crate::config::get_music_dir();

        // Cek dulu apakah sudah ada di library (cari by title/filename fuzzy)
        let existing = crate::engine::ytdlp::list_downloaded(&music_dir);
        let q_lower = query.to_lowercase();
        let query_words: Vec<&str> = query.split_whitespace().collect();

        let found = existing.iter().find(|t| {
            let t_title = t.title.to_lowercase();
            let t_filename = t.filename.to_lowercase();
            t_title.contains(&q_lower)
                || t_filename.contains(&q_lower)
                || (!query_words.is_empty()
                    && query_words.iter().all(|w| {
                        let w_lower = w.to_lowercase();
                        t_title.contains(&w_lower) || t_filename.contains(&w_lower)
                    }))
        });

        if let Some(track) = found {
            crate::app_state::push_client_action("music:play", Some(&track.title));
            return Some((
                format!("Putar: {}", track.title),
                "Memutar musik dari library lokal".to_string(),
            ));
        }

        // Belum ada di library lokal — delegasikan ke Background Task Worker Thread
        // AI langsung merespons percakapan tanpa freeze/menunggu, lagu akan otomatis diputar begitu selesai diunduh
        crate::engine::tasks::queue_download_and_play(query);

        return Some((
            format!("Unduh & Putar: {}", query),
            "Sedang memproses dan mengunduh lagu di background worker".to_string(),
        ));
    }

    // Kontrol player musik
    if target_clean == "pause musik"
        || target_clean == "jeda musik"
        || target_clean == "pause lagu"
        || target_clean == "jeda lagu"
        || target_clean == "pause"
        || target_clean == "jeda"
    {
        crate::app_state::push_client_action("music:pause", None);
        return Some(("Jeda Musik".to_string(), "Musik dijeda".to_string()));
    }
    if target_clean == "resume musik"
        || target_clean == "lanjut musik"
        || target_clean == "lanjutkan musik"
        || target_clean == "resume lagu"
        || target_clean == "lanjut lagu"
        || target_clean == "resume"
        || target_clean == "lanjut"
    {
        crate::app_state::push_client_action("music:resume", None);
        return Some(("Lanjut Musik".to_string(), "Musik dilanjutkan".to_string()));
    }
    if target_clean == "stop musik"
        || target_clean == "berhenti musik"
        || target_clean == "hentikan musik"
        || target_clean == "stop lagu"
        || target_clean == "hentikan lagu"
        || target_clean == "matikan musik"
        || target_clean == "stop"
    {
        crate::app_state::push_client_action("music:stop", None);
        return Some(("Stop Musik".to_string(), "Musik dihentikan".to_string()));
    }
    if target_clean == "next musik"
        || target_clean == "next lagu"
        || target_clean == "lagu berikutnya"
        || target_clean == "musik berikutnya"
        || target_clean == "next"
        || target_clean == "ganti lagu"
        || target_clean == "putar lagu lain"
    {
        crate::app_state::push_client_action("music:next", None);
        return Some((
            "Next Musik".to_string(),
            "Memutar lagu berikutnya".to_string(),
        ));
    }
    if target_clean == "prev musik"
        || target_clean == "previous musik"
        || target_clean == "lagu sebelumnya"
        || target_clean == "musik sebelumnya"
        || target_clean == "prev"
    {
        crate::app_state::push_client_action("music:prev", None);
        return Some((
            "Prev Musik".to_string(),
            "Memutar lagu sebelumnya".to_string(),
        ));
    }

    // 5. Cek dari daftar built-in system skills
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
                    ));
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
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}
