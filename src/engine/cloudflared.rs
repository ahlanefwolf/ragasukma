use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudflaredInfo {
    pub available: bool,
    pub version: String,
    pub path: Option<String>,
}

pub struct CloudflaredTunnelManager {
    child: Option<Child>,
    public_url: Option<String>,
    auth_key: String,
    status: String,
}

impl Default for CloudflaredTunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CloudflaredTunnelManager {
    pub fn new() -> Self {
        Self {
            child: None,
            public_url: None,
            auth_key: generate_random_auth_key(),
            status: "stopped".to_string(),
        }
    }
}

static CLOUDFLARED_URL_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| {
        regex::Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com")
            .unwrap_or_else(|_| regex::Regex::new("trycloudflare").expect("valid fallback cloudflare regex"))
    });

pub static TUNNEL_MANAGER: Mutex<Option<CloudflaredTunnelManager>> = Mutex::new(None);

fn generate_random_auth_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(123456789);
    format!("{:x}{:x}", seed, std::process::id())
}

pub fn get_tunnel_status() -> (String, Option<String>, String) {
    let mut lock = TUNNEL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    let mgr = lock.get_or_insert_with(CloudflaredTunnelManager::new);

    // Check if child exited
    if let Some(ref mut child) = mgr.child {
        if let Ok(Some(_status)) = child.try_wait() {
            mgr.child = None;
            mgr.status = "stopped".to_string();
            mgr.public_url = None;
        }
    }

    (
        mgr.status.clone(),
        mgr.public_url.clone(),
        mgr.auth_key.clone(),
    )
}

pub fn is_auth_key_valid(key: &str) -> bool {
    let mut lock = TUNNEL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    let mgr = lock.get_or_insert_with(CloudflaredTunnelManager::new);
    mgr.auth_key == key
}

fn resolve_cloudflared_bin() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let app_bin = std::path::PathBuf::from(local_app_data)
                .join("ragasukma")
                .join("bin")
                .join("cloudflared.exe");
            if app_bin.is_file() {
                return app_bin;
            }
        }
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let local_bin = std::path::PathBuf::from(user_profile)
                .join(".local")
                .join("bin")
                .join("cloudflared.exe");
            if local_bin.is_file() {
                return local_bin;
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let local_bin = std::path::PathBuf::from(home)
                .join(".local")
                .join("bin")
                .join("cloudflared");
            if local_bin.is_file() {
                return local_bin;
            }
        }
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(if cfg!(windows) {
                "cloudflared.exe"
            } else {
                "cloudflared"
            });
            if candidate.is_file() {
                return candidate;
            }
        }
    }

    let candidates = [
        "/usr/local/bin/cloudflared",
        "/usr/bin/cloudflared",
        "/bin/cloudflared",
        "/opt/homebrew/bin/cloudflared",
    ];

    for c in &candidates {
        let p = std::path::PathBuf::from(c);
        if p.is_file() {
            return p;
        }
    }

    std::path::PathBuf::from(if cfg!(windows) {
        "cloudflared.exe"
    } else {
        "cloudflared"
    })
}

pub fn parse_cloudflared_version(stdout: &str) -> String {
    let first_line = stdout.lines().next().unwrap_or("").trim();
    let tokens: Vec<&str> = first_line.split_whitespace().collect();
    let raw_version = if let Some(pos) = tokens.iter().position(|&t| t.eq_ignore_ascii_case("version")) {
        tokens.get(pos + 1).copied().unwrap_or("")
    } else {
        tokens
            .iter()
            .find(|&&t| t.chars().any(|c| c.is_ascii_digit()))
            .copied()
            .unwrap_or("")
    };
    raw_version
        .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-')
        .to_string()
}

pub fn check_cloudflared() -> CloudflaredInfo {
    let bin_path = resolve_cloudflared_bin();
    let mut cmd = Command::new(&bin_path);
    cmd.arg("--version");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    match cmd.output() {
        Ok(out) if out.status.success() => {
            let full_out = String::from_utf8_lossy(&out.stdout);
            let version = parse_cloudflared_version(&full_out);
            CloudflaredInfo {
                available: true,
                version: if version.is_empty() {
                    full_out.trim().to_string()
                } else {
                    version
                },
                path: Some(bin_path.to_string_lossy().to_string()),
            }
        }
        _ => CloudflaredInfo {
            available: false,
            version: String::new(),
            path: None,
        },
    }
}

pub fn install_cloudflared() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("USERPROFILE").map(|p| format!("{}\\AppData\\Local", p)))
            .map_err(|_| "Environment variable LOCALAPPDATA / USERPROFILE not found".to_string())?;
        let bin_dir = std::path::PathBuf::from(local_app_data)
            .join("ragasukma")
            .join("bin");
        std::fs::create_dir_all(&bin_dir)
            .map_err(|e| format!("Failed to create folder {}: {}", bin_dir.display(), e))?;
        let target_path = bin_dir.join("cloudflared.exe");

        let arch = if cfg!(target_arch = "x86") {
            "386"
        } else {
            "amd64"
        };
        let url = format!(
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-{}.exe",
            arch
        );

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let resp = client
            .get(&url)
            .send()
            .map_err(|e| format!("Failed to download cloudflared: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Failed to download cloudflared from {}: status {}",
                url,
                resp.status()
            ));
        }

        let bytes = resp
            .bytes()
            .map_err(|e| format!("Failed to read cloudflared payload: {}", e))?;

        std::fs::write(&target_path, bytes)
            .map_err(|e| format!("Failed to write cloudflared binary: {}", e))?;

        Ok(format!(
            "cloudflared successfully downloaded to {}",
            target_path.display()
        ))
    }
    #[cfg(target_os = "macos")]
    {
        let out = Command::new("brew")
            .args(["install", "cloudflared"])
            .output()
            .map_err(|e| format!("Failed to run brew: {}", e))?;
        if out.status.success() {
            Ok("cloudflared successfully installed via brew".to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")
            .map_err(|_| "HOME environment variable not found".to_string())?;
        let bin_dir = std::path::PathBuf::from(home).join(".local").join("bin");
        std::fs::create_dir_all(&bin_dir)
            .map_err(|e| format!("Failed to create folder ~/.local/bin: {}", e))?;
        let target_path = bin_dir.join("cloudflared");

        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "amd64"
        };
        let url = format!(
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-{}",
            arch
        );

        let target_str = target_path.to_string_lossy();
        let out = Command::new("curl")
            .args(["-L", "-o", target_str.as_ref(), &url])
            .output()
            .map_err(|e| format!("Failed to run curl: {}", e))?;

        if !out.status.success() {
            return Err("Failed to download cloudflared binary via curl".to_string());
        }

        let _ = Command::new("chmod")
            .args(["+x", target_str.as_ref()])
            .output();

        Ok(format!(
            "cloudflared successfully downloaded to {}",
            target_path.display()
        ))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Operating system not supported for automatic installation".to_string())
    }
}

pub fn start_cloudflared_tunnel(local_port: u16) -> Result<(String, String), String> {
    let mut lock = TUNNEL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    let mgr = lock.get_or_insert_with(CloudflaredTunnelManager::new);

    if let Some(ref mut child) = mgr.child {
        if let Ok(None) = child.try_wait() {
            // Already running
            return Ok((
                mgr.public_url.clone().unwrap_or_default(),
                mgr.auth_key.clone(),
            ));
        }
    }

    // Refresh auth key for this tunnel session
    mgr.auth_key = generate_random_auth_key();
    mgr.status = "starting".to_string();
    mgr.public_url = None;

    // Check cloudflared binary
    let bin_path = resolve_cloudflared_bin();
    let mut cmd = Command::new(bin_path);
    cmd.args([
        "tunnel",
        "--url",
        &format!("http://127.0.0.1:{}", local_port),
        "--no-autoupdate",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn().map_err(|e| {
        format!(
            "Failed to run cloudflared: {}. Make sure cloudflared is installed and available in PATH.",
            e
        )
    })?;

    let stderr = child
        .stderr
        .take()
        .ok_or("Failed to capture cloudflared stderr")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture cloudflared stdout")?;
    mgr.child = Some(child);

    // Read stderr & stdout in background threads to find trycloudflare.com URL
    let auth_key = mgr.auth_key.clone();

    // Spawn stderr reader
    thread::spawn(move || {
        let reader = BufReader::new(stderr);

        for line in reader.lines().map_while(Result::ok) {
            crate::term_log!("[Cloudflared] {}", line);
            if let Some(matched) = CLOUDFLARED_URL_REGEX.find(&line) {
                let found_url = matched.as_str().to_string();
                if let Ok(mut lock) = TUNNEL_MANAGER.lock() {
                    if let Some(ref mut mgr) = *lock {
                        mgr.public_url = Some(found_url.clone());
                        mgr.status = "running".to_string();
                        crate::term_log!("[Cloudflared Tunnel Ready] {}", found_url);
                    }
                }
            }
        }
    });

    // Spawn stdout reader
    thread::spawn(move || {
        let reader = BufReader::new(stdout);

        for line in reader.lines().map_while(Result::ok) {
            crate::term_log!("[Cloudflared stdout] {}", line);
            if let Some(matched) = CLOUDFLARED_URL_REGEX.find(&line) {
                let found_url = matched.as_str().to_string();
                if let Ok(mut lock) = TUNNEL_MANAGER.lock() {
                    if let Some(ref mut mgr) = *lock {
                        mgr.public_url = Some(found_url.clone());
                        mgr.status = "running".to_string();
                        crate::term_log!("[Cloudflared Tunnel Ready] {}", found_url);
                    }
                }
            }
        }
    });

    // Return immediately with "starting" status and auth_key
    // The frontend will poll /api/remote/tunnel/status until URL is available
    Ok(("".to_string(), auth_key))
}

pub fn stop_cloudflared_tunnel() -> Result<(), String> {
    let mut lock = TUNNEL_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref mut mgr) = *lock {
        if let Some(mut child) = mgr.child.take() {
            #[allow(unused_variables)]
            let pid = child.id();
            let _ = child.kill();
            let _ = child.wait();

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000)
                    .output();
            }
        }
        mgr.public_url = None;
        mgr.status = "stopped".to_string();
        crate::term_log!("[Cloudflared Tunnel Stopped]");
    }
    Ok(())
}

impl Drop for CloudflaredTunnelManager {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            #[allow(unused_variables)]
            let pid = child.id();
            let _ = child.kill();
            let _ = child.wait();

            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000)
                    .output();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cloudflared_version() {
        let sample1 = "cloudflared version 2026.8.3 (Checksum f29324fe934d1e100617484c78deef803c4dc2cd351d645bbde42e96b4fccc5e)";
        assert_eq!(parse_cloudflared_version(sample1), "2026.8.3");

        let sample2 = "cloudflared version 2024.1.5 (built 2024-01-15-1200)";
        assert_eq!(parse_cloudflared_version(sample2), "2024.1.5");

        let sample3 = "cloudflared 2023.10.0";
        assert_eq!(parse_cloudflared_version(sample3), "2023.10.0");
    }
}
