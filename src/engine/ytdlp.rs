use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YtDlpInfo {
    pub version: String,
    pub available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YtTrackInfo {
    pub id: String,
    pub title: String,
    pub uploader: String,
    pub duration: u64,
    pub thumbnail: Option<String>,
    /// Nama file relatif di folder music (e.g. "Song Title [abc123].opus")
    pub filename: String,
}

/// Install yt-dlp otomatis sesuai OS
pub fn install_ytdlp() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let out = Command::new("winget")
            .args(["install", "--id", "yt-dlp.yt-dlp", "-e", "--accept-source-agreements", "--accept-package-agreements"])
            .output()
            .map_err(|e| format!("Gagal jalankan winget: {}", e))?;
        if out.status.success() {
            Ok("yt-dlp berhasil diinstall via winget".to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("brew")
            .args(["install", "yt-dlp"])
            .output()
            .map_err(|e| format!("Gagal jalankan brew: {}", e))?;
        if out.status.success() {
            Ok("yt-dlp berhasil diinstall via brew".to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        let out = Command::new("pip")
            .args(["install", "-U", "yt-dlp"])
            .output()
            .map_err(|e| format!("Gagal jalankan pip: {}", e))?;
        if out.status.success() {
            Ok("yt-dlp berhasil diinstall via pip".to_string())
        } else {
            // fallback pip3
            let out2 = Command::new("pip3")
                .args(["install", "-U", "yt-dlp"])
                .output()
                .map_err(|e| format!("Gagal jalankan pip3: {}", e))?;
            if out2.status.success() {
                Ok("yt-dlp berhasil diinstall via pip3".to_string())
            } else {
                Err(String::from_utf8_lossy(&out2.stderr).trim().to_string())
            }
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("OS tidak didukung untuk auto-install".to_string())
    }
}

/// Cari path executable yt-dlp di Windows (termasuk WinGet Links & Packages, Scoop, Python Script)
fn find_ytdlp_executable() -> Option<std::path::PathBuf> {
    // 1. Cek langsung jika bisa dipanggil di PATH
    if let Ok(out) = Command::new("yt-dlp").arg("--version").output() {
        if out.status.success() {
            return Some(std::path::PathBuf::from("yt-dlp"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 2. Cek WinGet standard directory: %LOCALAPPDATA%\Microsoft\WinGet\Links\yt-dlp.exe
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let winget_link = std::path::Path::new(&local_app_data)
                .join("Microsoft")
                .join("WinGet")
                .join("Links")
                .join("yt-dlp.exe");
            if winget_link.exists() {
                return Some(winget_link);
            }

            // WinGet Packages folder scan
            let winget_packages = std::path::Path::new(&local_app_data)
                .join("Microsoft")
                .join("WinGet")
                .join("Packages");
            if winget_packages.exists() {
                if let Ok(entries) = std::fs::read_dir(&winget_packages) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let candidate = path.join("yt-dlp.exe");
                            if candidate.exists() {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }

            // Programs folder
            let prog_cand = std::path::Path::new(&local_app_data)
                .join("Programs")
                .join("yt-dlp")
                .join("yt-dlp.exe");
            if prog_cand.exists() {
                return Some(prog_cand);
            }
        }

        // 3. Cek %USERPROFILE%\scoop\shims\yt-dlp.exe atau %USERPROFILE%\.cargo\bin\yt-dlp.exe
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let scoop_cand = std::path::Path::new(&user_profile)
                .join("scoop")
                .join("shims")
                .join("yt-dlp.exe");
            if scoop_cand.exists() {
                return Some(scoop_cand);
            }

            // Python Scripts
            let py_cand = std::path::Path::new(&user_profile)
                .join("AppData")
                .join("Roaming")
                .join("Python")
                .join("Scripts")
                .join("yt-dlp.exe");
            if py_cand.exists() {
                return Some(py_cand);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Homebrew on Apple Silicon (/opt/homebrew/bin) and Intel (/usr/local/bin)
        let candidates = [
            "/opt/homebrew/bin/yt-dlp",
            "/usr/local/bin/yt-dlp",
            "/usr/bin/yt-dlp",
        ];
        for cand in candidates {
            let p = std::path::PathBuf::from(cand);
            if p.exists() {
                return Some(p);
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let user_cand = std::path::Path::new(&home).join(".local/bin/yt-dlp");
            if user_cand.exists() {
                return Some(user_cand);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let candidates = [
            "/usr/bin/yt-dlp",
            "/usr/local/bin/yt-dlp",
            "/snap/bin/yt-dlp",
        ];
        for cand in candidates {
            let p = std::path::PathBuf::from(cand);
            if p.exists() {
                return Some(p);
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let user_cand = std::path::Path::new(&home).join(".local/bin/yt-dlp");
            if user_cand.exists() {
                return Some(user_cand);
            }
        }
    }

    None
}

/// Resolve command yt-dlp
fn ytdlp_command() -> Command {
    if let Some(exe_path) = find_ytdlp_executable() {
        Command::new(exe_path)
    } else {
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", "yt-dlp"]);
            cmd
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("yt-dlp")
        }
    }
}

/// Cek apakah yt-dlp terinstall dan return versinya
pub fn check_ytdlp() -> YtDlpInfo {
    let result = ytdlp_command().arg("--version").output();
    match result {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            YtDlpInfo { version, available: true }
        }
        _ => YtDlpInfo { version: String::new(), available: false },
    }
}

/// Download audio pakai yt-dlp ke out_dir, return metadata track
pub fn download_audio(query: &str, out_dir: &Path) -> anyhow::Result<YtTrackInfo> {
    std::fs::create_dir_all(out_dir)?;

    let abs_out_dir = if out_dir.is_relative() {
        std::env::current_dir()?.join(out_dir)
    } else {
        out_dir.to_path_buf()
    };

    // Jika query bukan URL, bungkus sebagai ytsearch
    let target = if query.starts_with("http://") || query.starts_with("https://") {
        query.to_string()
    } else {
        format!("ytsearch1:{}", query)
    };

    println!("[yt-dlp] Memulai download audio: {} ke {:?}", target, abs_out_dir);

    // Catat file audio yang sudah ada sebelum download
    let existing_files: std::collections::HashSet<String> = std::fs::read_dir(&abs_out_dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();

    // Gunakan -f "ba[ext=m4a]/ba" (best audio M4A/AAC atau best audio)
    // Gunakan --print "after_move:..." agar yt-dlp BENAR-BENAR mendownload file ke disk
    // lalu mencetak metadata setelah file selesai ditulis
    let mut child = ytdlp_command()
        .current_dir(&abs_out_dir)
        .args([
            "--no-playlist",
            "-f", "ba[ext=m4a]/ba",
            "-o", "%(title)s [%(id)s].%(ext)s",
            "--print", "after_move:%(id)s\t%(title)s\t%(uploader)s\t%(filename)s",
            "--no-warnings",
            &target,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Batas timeout: 25 detik
    let timeout = std::time::Duration::from_secs(25);
    let start_time = std::time::Instant::now();
    let poll_interval = std::time::Duration::from_millis(250);

    let output = loop {
        match child.try_wait()? {
            Some(_status) => {
                // Proses selesai sebelum timeout
                break child.wait_with_output()?;
            }
            None => {
                if start_time.elapsed() >= timeout {
                    let _ = child.kill();
                    clean_json_sidecars(&abs_out_dir);
                    anyhow::bail!("Waktu download lagu habis (timeout 25 detik). Koneksi internet lambat.");
                }
                std::thread::sleep(poll_interval);
            }
        }
    };

    // Bersihkan file json sisa jika ada
    clean_json_sidecars(&abs_out_dir);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp download error: {}", stderr);
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // 1. Coba parse metadata persis dari stdout yt-dlp (after_move)
    for line in stdout_str.lines().rev() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let id = parts[0].trim().to_string();
            let title = parts[1].trim().to_string();
            let uploader = parts[2].trim().to_string();
            let raw_filename = parts[3].trim().to_string();

            let filename = std::path::Path::new(&raw_filename)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&raw_filename)
                .to_string();

            if !filename.is_empty() && abs_out_dir.join(&filename).exists() {
                let thumbnail = if !id.is_empty() {
                    Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id))
                } else {
                    None
                };

                println!("[yt-dlp] File berhasil disimpan & dimuat: {} ({})", title, filename);
                return Ok(YtTrackInfo {
                    id,
                    title,
                    uploader: if uploader.is_empty() { "YouTube Audio".to_string() } else { uploader },
                    duration: 0,
                    thumbnail,
                    filename,
                });
            }
        }
    }

    // 2. Deteksi file baru yang bertambah di folder (perbandingan sebelum vs sesudah)
    if let Ok(entries) = std::fs::read_dir(&abs_out_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !existing_files.contains(&name) {
                let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ["m4a", "opus", "mp3", "ogg", "wav", "webm", "flac"].contains(&ext.as_str()) {
                    let (title, id) = parse_filename_title_id(&name);
                    let thumbnail = if !id.is_empty() {
                        Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id))
                    } else {
                        None
                    };
                    println!("[yt-dlp] File baru terdeteksi: {} ({})", title, name);
                    return Ok(YtTrackInfo {
                        id,
                        title,
                        uploader: "YouTube Audio".to_string(),
                        duration: 0,
                        thumbnail,
                        filename: name,
                    });
                }
            }
        }
    }

    // 2. Fallback: Cari file audio yang paling baru dimodifikasi (latest mtime) di folder
    if let Ok(entries) = std::fs::read_dir(&abs_out_dir) {
        let mut audio_files = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ["m4a", "opus", "mp3", "ogg", "wav", "webm", "flac"].contains(&ext_lower.as_str()) {
                    let mtime = entry.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    audio_files.push((path, mtime));
                }
            }
        }

        // Urutkan berdasarkan waktu modifikasi terbaru
        audio_files.sort_by_key(|a| std::cmp::Reverse(a.1));

        if let Some((latest_path, _)) = audio_files.first() {
            let filename = latest_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let (title, id) = parse_filename_title_id(&filename);
            let thumbnail = if !id.is_empty() {
                Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id))
            } else {
                None
            };
            println!("[yt-dlp] Fallback ke file audio terbaru: {} ({})", title, filename);
            return Ok(YtTrackInfo {
                id,
                title,
                uploader: "YouTube Audio".to_string(),
                duration: 0,
                thumbnail,
                filename,
            });
        }
    }

    anyhow::bail!("Download selesai tapi file audio tidak ditemukan di {:?}", abs_out_dir)
}

/// Bersihkan file *.json di folder music agar rapi hanya berisi audio murni
pub fn clean_json_sidecars(music_dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(music_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("json") {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
    }
}

/// Scan folder music, parse judul & ID langsung dari nama file, return list track
pub fn list_downloaded(music_dir: &Path) -> Vec<YtTrackInfo> {
    clean_json_sidecars(music_dir);
    let mut tracks = Vec::new();

    let entries = match std::fs::read_dir(music_dir) {
        Ok(e) => e,
        Err(_) => return tracks,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !["mp3", "opus", "ogg", "wav", "m4a", "flac", "webm"].contains(&ext.as_str()) {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let (title, id) = parse_filename_title_id(&filename);
        let thumbnail = if !id.is_empty() {
            Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id))
        } else {
            None
        };

        let uploader = if let Some((artist, _)) = title.split_once(" - ") {
            artist.trim().to_string()
        } else {
            "YouTube Audio".to_string()
        };

        tracks.push(YtTrackInfo {
            id,
            title,
            uploader,
            duration: 0,
            thumbnail,
            filename,
        });
    }

    tracks.sort_by(|a, b| a.title.cmp(&b.title));
    tracks
}

/// Parse "(title) [(id)].ext" dari nama file jika tidak ada sidecar
fn parse_filename_title_id(filename: &str) -> (String, String) {
    // Hapus ekstensi apapun (.webm, .opus, .mp3, dsb)
    let stem = if let Some((name_without_ext, _)) = filename.rsplit_once('.') {
        name_without_ext
    } else {
        filename
    };
    // Coba ekstrak [id] dari akhir
    if let (Some(open), Some(close)) = (stem.rfind('['), stem.rfind(']')) {
        if open < close {
            let id = stem[open + 1..close].to_string();
            let title = stem[..open].trim().to_string();
            return (title, id);
        }
    }
    (stem.to_string(), String::new())
}
