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
    /// Name file relatif in folder music (e.g. "Song Title [abc123].opus")
    pub filename: String,
}

/// Install yt-dlp automatically sesuai OS
pub fn install_ytdlp() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let mut winget_cmd = Command::new("winget");
        winget_cmd.args([
            "install",
            "--id",
            "yt-dlp.yt-dlp",
            "-e",
            "--accept-source-agreements",
            "--accept-package-agreements",
        ]);
        {
            use std::os::windows::process::CommandExt;
            winget_cmd.creation_flags(0x08000000);
        }
        let out = winget_cmd
            .output()
            .map_err(|e| format!("Failed to run winget: {}", e))?;
        if out.status.success() {
            if let Ok(mut guard) = CACHED_YTDLP.lock() {
                *guard = None;
            }
            Ok("yt-dlp successfully installed via winget".to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("brew")
            .args(["install", "yt-dlp"])
            .output()
            .map_err(|e| format!("Failed to run brew: {}", e))?;
        if out.status.success() {
            Ok("yt-dlp successfully installed via brew".to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut cmd = Command::new("pip");
        cmd.args(["install", "-U", "yt-dlp"])
            .env_remove("PYTHONHOME")
            .env_remove("PYTHONPATH")
            .env_remove("LD_LIBRARY_PATH")
            .env_remove("LD_PRELOAD");
        let out = cmd
            .output()
            .map_err(|e| format!("Failed to run pip: {}", e))?;
        if out.status.success() {
            Ok("yt-dlp successfully installed via pip".to_string())
        } else {
            // fallback pip3
            let mut cmd2 = Command::new("pip3");
            cmd2.args(["install", "-U", "yt-dlp"])
                .env_remove("PYTHONHOME")
                .env_remove("PYTHONPATH")
                .env_remove("LD_LIBRARY_PATH")
                .env_remove("LD_PRELOAD");
            let out2 = cmd2
                .output()
                .map_err(|e| format!("Failed to run pip3: {}", e))?;
            if out2.status.success() {
                Ok("yt-dlp successfully installed via pip3".to_string())
            } else {
                Err(String::from_utf8_lossy(&out2.stderr).trim().to_string())
            }
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("OS not supported for auto-install".to_string())
    }
}

/// Search path executable yt-dlp in Windows (termasuk WinGet Links & Packages, Scoop, Python Script)
fn find_ytdlp_executable() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // 1. Check WinGet standard directory: %LOCALAPPDATA%\Microsoft\WinGet\Links\yt-dlp.exe
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

        // 2. Check %USERPROFILE%\scoop\shims\yt-dlp.exe or %USERPROFILE%\.cargo\bin\yt-dlp.exe
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

        let program_files = [
            "C:\\Program Files\\yt-dlp\\yt-dlp.exe",
            "C:\\Program Files (x86)\\yt-dlp\\yt-dlp.exe",
            "C:\\yt-dlp\\yt-dlp.exe",
        ];
        for path_str in program_files {
            let p = std::path::PathBuf::from(path_str);
            if p.exists() {
                return Some(p);
            }
        }

        // 3. Fallback: Search in system PATH directories directly without spawning probe process
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join("yt-dlp.exe");
                if candidate.exists() {
                    return Some(candidate);
                }
                let candidate_no_ext = dir.join("yt-dlp");
                if candidate_no_ext.exists() {
                    return Some(candidate_no_ext);
                }
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
fn ytdlp_command() -> Option<Command> {
    let exe_path = find_ytdlp_executable()?;
    let mut cmd = Command::new(exe_path);

    // Clean environment variables hijacked by Linux AppImage wrapper
    // so that yt-dlp / Python using dynamic library asli milik OS host
    cmd.env_remove("PYTHONHOME");
    cmd.env_remove("PYTHONPATH");
    cmd.env_remove("LD_LIBRARY_PATH");
    cmd.env_remove("LD_PRELOAD");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.stdin(std::process::Stdio::null());
    Some(cmd)
}

static CACHED_YTDLP: std::sync::Mutex<Option<YtDlpInfo>> = std::sync::Mutex::new(None);

/// Check whether yt-dlp is installed and return its version
pub fn check_ytdlp() -> YtDlpInfo {
    if let Ok(guard) = CACHED_YTDLP.lock() {
        if let Some(info) = guard.as_ref() {
            return info.clone();
        }
    }

    // 1. If yt-dlp binary path is known on disk, run --version directly with full detachment
    if let Some(exe_path) = find_ytdlp_executable() {
        let mut cmd = Command::new(exe_path);
        cmd.arg("--version");
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::null());
        cmd.env_remove("PYTHONHOME");
        cmd.env_remove("PYTHONPATH");
        cmd.env_remove("LD_LIBRARY_PATH");
        cmd.env_remove("LD_PRELOAD");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let info = YtDlpInfo {
                    version,
                    available: true,
                };
                if let Ok(mut guard) = CACHED_YTDLP.lock() {
                    *guard = Some(info.clone());
                }
                return info;
            }
        }
    }

    // 2. If not found on disk, report unavailable immediately without spawning fallback cmd.exe
    let info = YtDlpInfo {
        version: String::new(),
        available: false,
    };
    if let Ok(mut guard) = CACHED_YTDLP.lock() {
        *guard = Some(info.clone());
    }
    info
}

/// Download audio pakai yt-dlp to out_dir, return metadata track
pub fn download_audio(query: &str, out_dir: &Path) -> anyhow::Result<YtTrackInfo> {
    std::fs::create_dir_all(out_dir)?;

    let abs_out_dir = if out_dir.is_relative() {
        std::env::current_dir()?.join(out_dir)
    } else {
        out_dir.to_path_buf()
    };

    // If query is not a URL, wrap as ytsearch
    let target = if query.starts_with("http://") || query.starts_with("https://") {
        query.to_string()
    } else {
        format!("ytsearch1:{}", query)
    };

    println!(
        "[yt-dlp] Starting audio download: {} to {:?}",
        target, abs_out_dir
    );

    // Record existing audio files before download
    let existing_files: std::collections::HashSet<String> = std::fs::read_dir(&abs_out_dir)
        .ok()
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();

    // Extract audio directly to MP3 using yt-dlp / ffmpeg for maximum stability and compatibility across all platforms
    // Use --print "after_move:..." so that yt-dlp prints metadata after file is completely written
    let mut cmd = ytdlp_command()
        .ok_or_else(|| anyhow::anyhow!("yt-dlp executable not found on system"))?;
    let mut child = cmd
        .current_dir(&abs_out_dir)
        .args([
            "--no-playlist",
            "-x",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "0",
            "-o",
            "%(title)s [%(id)s].%(ext)s",
            "--print",
            "after_move:%(id)s\t%(title)s\t%(uploader)s\t%(duration)s\t%(filename)s",
            "--no-warnings",
            &target,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Limit timeout: 25 detik
    let timeout = std::time::Duration::from_secs(25);
    let start_time = std::time::Instant::now();
    let poll_interval = std::time::Duration::from_millis(250);

    let output = loop {
        match child.try_wait()? {
            Some(_status) => {
                // Process finished before timeout
                break child.wait_with_output()?;
            }
            None => {
                if start_time.elapsed() >= timeout {
                    let _ = child.kill();
                    clean_json_sidecars(&abs_out_dir);
                    anyhow::bail!(
                        "Download timed out (25 second limit). Slow internet connection."
                    );
                }
                std::thread::sleep(poll_interval);
            }
        }
    };

    // Clean leftover JSON files if any
    clean_json_sidecars(&abs_out_dir);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp download error: {}", stderr);
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // 1. Coba parse metadata persis from stdout yt-dlp (after_move)
    for line in stdout_str.lines().rev() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 5 {
            let id = parts[0].trim().to_string();
            let title = parts[1].trim().to_string();
            let uploader = parts[2].trim().to_string();
            let duration: u64 = parts[3].trim().parse().unwrap_or(0);
            let raw_filename = parts[4].trim().to_string();

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

                let real_dur = if duration > 0 {
                    duration
                } else {
                    estimate_audio_duration_secs(&abs_out_dir.join(&filename))
                };

                println!(
                    "[yt-dlp] Audio file successfully saved & loaded: {} ({}, dur: {}s)",
                    title, filename, real_dur
                );
                return Ok(YtTrackInfo {
                    id,
                    title,
                    uploader: if uploader.is_empty() {
                        "YouTube Audio".to_string()
                    } else {
                        uploader
                    },
                    duration: real_dur,
                    thumbnail,
                    filename,
                });
            }
        } else if parts.len() >= 4 {
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

                let real_dur = estimate_audio_duration_secs(&abs_out_dir.join(&filename));

                println!(
                    "[yt-dlp] Audio file successfully saved & loaded: {} ({}, dur: {}s)",
                    title, filename, real_dur
                );
                return Ok(YtTrackInfo {
                    id,
                    title,
                    uploader: if uploader.is_empty() {
                        "YouTube Audio".to_string()
                    } else {
                        uploader
                    },
                    duration: real_dur,
                    thumbnail,
                    filename,
                });
            }
        }
    }

    // 2. Detect newly added files in folder (before vs after comparison)
    if let Ok(entries) = std::fs::read_dir(&abs_out_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !existing_files.contains(&name) {
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if ["m4a", "opus", "mp3", "ogg", "wav", "webm", "flac"].contains(&ext.as_str()) {
                    let (title, id) = parse_filename_title_id(&name);
                    let thumbnail = if !id.is_empty() {
                        Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id))
                    } else {
                        None
                    };
                    println!("[yt-dlp] New file detected: {} ({})", title, name);
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

    // 2. Fallback: Search for the most recently modified audio file (latest mtime) in folder
    if let Ok(entries) = std::fs::read_dir(&abs_out_dir) {
        let mut audio_files = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ["m4a", "opus", "mp3", "ogg", "wav", "webm", "flac"]
                    .contains(&ext_lower.as_str())
                {
                    let mtime = entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    audio_files.push((path, mtime));
                }
            }
        }

        // Sort by latest modification time
        audio_files.sort_by_key(|a| std::cmp::Reverse(a.1));

        if let Some((latest_path, _)) = audio_files.first() {
            let filename = latest_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let (title, id) = parse_filename_title_id(&filename);
            let thumbnail = if !id.is_empty() {
                Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id))
            } else {
                None
            };
            println!(
                "[yt-dlp] Fallback to latest audio file: {} ({})",
                title, filename
            );
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

    anyhow::bail!(
        "Download finished but audio file not found at {:?}",
        abs_out_dir
    )
}

/// Clean *.json files in music directory so it cleanly contains only pure audio files
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

/// Scan folder music, parse judul & ID direct from name file, return list track
pub fn list_downloaded(music_dir: &Path) -> Vec<YtTrackInfo> {
    clean_json_sidecars(music_dir);
    let mut tracks = Vec::new();

    let entries = match std::fs::read_dir(music_dir) {
        Ok(e) => e,
        Err(_) => return tracks,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
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

        let duration = estimate_audio_duration_secs(&path);

        tracks.push(YtTrackInfo {
            id,
            title,
            uploader,
            duration,
            thumbnail,
            filename,
        });
    }

    tracks.sort_by(|a, b| a.title.cmp(&b.title));
    tracks
}

/// Probe precise audio duration from MP3 Xing/Info/VBRI headers or ID3 TLEN
fn probe_mp3_info_frame_duration(path: &Path) -> Option<u64> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; 65536];
    let bytes_read = file.read(&mut buf).ok()?;
    if bytes_read < 40 {
        return None;
    }
    buf.truncate(bytes_read);

    // 1. Check ID3v2 tag
    let mut audio_offset = 0usize;
    if buf.starts_with(b"ID3") && buf.len() >= 10 {
        let size = ((buf[6] as usize & 0x7F) << 21)
            | ((buf[7] as usize & 0x7F) << 14)
            | ((buf[8] as usize & 0x7F) << 7)
            | (buf[9] as usize & 0x7F);
        let id3_end = 10 + size;

        // Try extracting TLEN (length in milliseconds) frame from ID3 tag
        let scan_limit = id3_end.min(buf.len());
        if let Some(pos) = buf[..scan_limit].windows(4).position(|w| w == b"TLEN") {
            if pos + 10 + 4 <= buf.len() {
                let frame_size = ((buf[pos + 4] as usize) << 24)
                    | ((buf[pos + 5] as usize) << 16)
                    | ((buf[pos + 6] as usize) << 8)
                    | (buf[pos + 7] as usize);
                let content_start = pos + 10;
                let content_end = (content_start + frame_size).min(buf.len());
                if content_end > content_start {
                    let content_slice = &buf[content_start + 1..content_end];
                    if let Ok(s) = std::str::from_utf8(content_slice) {
                        let clean_s = s.trim_matches('\0').trim();
                        if let Ok(ms) = clean_s.parse::<u64>() {
                            if ms > 0 {
                                return Some(ms / 1000);
                            }
                        }
                    }
                }
            }
        }

        if id3_end < buf.len() {
            audio_offset = id3_end;
        } else if file.seek(SeekFrom::Start(id3_end as u64)).is_ok() {
            let mut new_buf = vec![0u8; 32768];
            if let Ok(n) = file.read(&mut new_buf) {
                new_buf.truncate(n);
                buf = new_buf;
                audio_offset = 0;
            }
        }
    }

    // Sample rate lookup table: [MPEG version][index]
    // version index: 0 = MPEG 2.5, 1 = reserved, 2 = MPEG 2, 3 = MPEG 1
    const SAMPLE_RATES: [[u32; 4]; 4] = [
        [11025, 12000, 8000, 0],
        [0, 0, 0, 0],
        [22050, 24000, 16000, 0],
        [44100, 48000, 32000, 0],
    ];

    let max_search = buf.len().saturating_sub(40);
    for i in audio_offset..max_search {
        if buf[i] == 0xFF && (buf[i + 1] & 0xE0) == 0xE0 {
            let b1 = buf[i + 1];
            let b2 = buf[i + 2];
            let b3 = buf[i + 3];

            let version_id = ((b1 >> 3) & 0x03) as usize;
            let layer_id = ((b1 >> 1) & 0x03) as usize;
            let protection = b1 & 0x01;
            let sr_idx = ((b2 >> 2) & 0x03) as usize;
            let channel_mode = (b3 >> 6) & 0x03;

            // Must be Layer III (MP3)
            if version_id == 1 || layer_id != 1 || sr_idx == 3 {
                continue;
            }

            let sample_rate = SAMPLE_RATES[version_id][sr_idx];
            if sample_rate == 0 {
                continue;
            }

            let is_mono = channel_mode == 3;
            let is_mpeg1 = version_id == 3;
            let crc_offset = if protection == 0 { 2 } else { 0 };

            let xing_offset = if is_mpeg1 {
                if is_mono { 21 } else { 36 }
            } else if is_mono {
                13
            } else {
                21
            } + crc_offset;

            // Check Xing / Info frame
            let check_pos = i + xing_offset;
            if check_pos + 12 <= buf.len() {
                let magic = &buf[check_pos..check_pos + 4];
                if magic == b"Xing" || magic == b"Info" {
                    let flags = u32::from_be_bytes(buf[check_pos + 4..check_pos + 8].try_into().ok()?);
                    if flags & 0x0001 != 0 {
                        let total_frames =
                            u32::from_be_bytes(buf[check_pos + 8..check_pos + 12].try_into().ok()?);
                        let samples_per_frame = if is_mpeg1 { 1152u64 } else { 576u64 };
                        let total_samples = total_frames as u64 * samples_per_frame;
                        let dur_secs = total_samples / sample_rate as u64;
                        if dur_secs > 0 {
                            return Some(dur_secs);
                        }
                    }
                }
            }

            // Check VBRI frame
            let vbri_pos = i + 36;
            if vbri_pos + 18 <= buf.len() && &buf[vbri_pos..vbri_pos + 4] == b"VBRI" {
                let total_frames =
                    u32::from_be_bytes(buf[vbri_pos + 14..vbri_pos + 18].try_into().ok()?);
                let dur_secs = (total_frames as u64 * 1152) / sample_rate as u64;
                if dur_secs > 0 {
                    return Some(dur_secs);
                }
            }
        }
    }

    None
}

/// Computes audio duration in seconds using precise Info/Xing frame parsing,
/// native container decoding via rodio/symphonia, and bitrate estimation fallback.
pub fn estimate_audio_duration_secs(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    // 1. Precise MP3 frame parsing (Xing/Info/VBRI or ID3 TLEN)
    if let Some(dur) = probe_mp3_info_frame_duration(path) {
        if dur > 0 {
            return dur;
        }
    }

    // 2. Precise container decoder probing via rodio/symphonia (FLAC, M4A, WAV, OGG, etc.)
    if let Ok(file) = std::fs::File::open(path) {
        use rodio::Source;
        if let Ok(decoder) = rodio::Decoder::new(std::io::BufReader::new(file)) {
            if let Some(dur) = decoder.total_duration() {
                let secs = dur.as_secs();
                if secs > 0 {
                    return secs;
                }
            }
        }
    }

    // 3. Fallback to file size estimation if headers are absent or stripped
    if let Ok(meta) = std::fs::metadata(path) {
        let size_bytes = meta.len();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let bytes_per_sec = match ext.as_str() {
            "wav" => 176400,
            "flac" => 90000,
            "opus" => 16000,
            "ogg" => 20000,
            "m4a" => 16000,
            _ => 24000,
        };

        if let Some(duration) = size_bytes.checked_div(bytes_per_sec) {
            return duration.max(1);
        }
    }

    0
}

/// Parse "(title) [(id)].ext" from filename if no sidecar exists
fn parse_filename_title_id(filename: &str) -> (String, String) {
    // Strip any extension (.webm, .opus, .mp3, etc.)
    let stem = if let Some((name_without_ext, _)) = filename.rsplit_once('.') {
        name_without_ext
    } else {
        filename
    };
    // Coba ekstrak [id] from end
    if let (Some(open), Some(close)) = (stem.rfind('['), stem.rfind(']')) {
        if open < close {
            let id = stem[open + 1..close].to_string();
            let title = stem[..open].trim().to_string();
            return (title, id);
        }
    }
    (stem.to_string(), String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precise_audio_duration_on_refvoice() {
        let candidate = Path::new("assets/refvoice.mp3");
        if candidate.exists() {
            let dur = estimate_audio_duration_secs(candidate);
            assert!(dur > 0, "Duration of refvoice.mp3 should be positive");
        }
    }
}
