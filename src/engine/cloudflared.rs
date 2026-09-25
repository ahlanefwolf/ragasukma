use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;

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
    let mut lock = TUNNEL_MANAGER.lock().unwrap();
    if lock.is_none() {
        *lock = Some(CloudflaredTunnelManager::new());
    }
    let mgr = lock.as_mut().unwrap();

    // Check if child exited
    if let Some(ref mut child) = mgr.child {
        if let Ok(Some(_status)) = child.try_wait() {
            mgr.child = None;
            mgr.status = "stopped".to_string();
            mgr.public_url = None;
        }
    }

    (mgr.status.clone(), mgr.public_url.clone(), mgr.auth_key.clone())
}

pub fn is_auth_key_valid(key: &str) -> bool {
    let mut lock = TUNNEL_MANAGER.lock().unwrap();
    if lock.is_none() {
        *lock = Some(CloudflaredTunnelManager::new());
    }
    let mgr = lock.as_ref().unwrap();
    mgr.auth_key == key
}

fn resolve_cloudflared_bin() -> std::path::PathBuf {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(if cfg!(windows) { "cloudflared.exe" } else { "cloudflared" });
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

    std::path::PathBuf::from(if cfg!(windows) { "cloudflared.exe" } else { "cloudflared" })
}

pub fn start_cloudflared_tunnel(local_port: u16) -> Result<(String, String), String> {
    let mut lock = TUNNEL_MANAGER.lock().unwrap();
    if lock.is_none() {
        *lock = Some(CloudflaredTunnelManager::new());
    }
    let mgr = lock.as_mut().unwrap();

    if let Some(ref mut child) = mgr.child {
        if let Ok(None) = child.try_wait() {
            // Already running
            return Ok((mgr.public_url.clone().unwrap_or_default(), mgr.auth_key.clone()));
        }
    }

    // Refresh auth key for this tunnel session
    mgr.auth_key = generate_random_auth_key();
    mgr.status = "starting".to_string();
    mgr.public_url = None;

    // Check cloudflared binary
    let bin_path = resolve_cloudflared_bin();
    let mut cmd = Command::new(bin_path);
    cmd.args(["tunnel", "--url", &format!("http://127.0.0.1:{}", local_port), "--no-autoupdate"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn().map_err(|e| {
        format!("Gagal menjalankan cloudflared: {}. Pastikan cloudflared terinstal dan ada di PATH.", e)
    })?;

    let stderr = child.stderr.take().ok_or("Failed to capture cloudflared stderr")?;
    let stdout = child.stdout.take().ok_or("Failed to capture cloudflared stdout")?;
    mgr.child = Some(child);

    // Read stderr & stdout in background threads to find trycloudflare.com URL
    let auth_key = mgr.auth_key.clone();

    // Spawn stderr reader
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let url_regex = regex::Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com").unwrap();

        for line in reader.lines().map_while(Result::ok) {
            crate::term_log!("[Cloudflared] {}", line);
            if let Some(matched) = url_regex.find(&line) {
                let found_url = matched.as_str().to_string();
                if let Ok(mut lock) = TUNNEL_MANAGER.lock() {
                    if let Some(ref mut mgr) = *lock {
                        mgr.public_url = Some(found_url.clone());
                        mgr.status = "running".to_string();
                        crate::term_log!("🌐 [Cloudflared Tunnel Ready] {}", found_url);
                    }
                }
            }
        }
    });

    // Spawn stdout reader as well (in case cloudflared outputs URL to stdout)
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let url_regex = regex::Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com").unwrap();

        for line in reader.lines().map_while(Result::ok) {
            crate::term_log!("[Cloudflared stdout] {}", line);
            if let Some(matched) = url_regex.find(&line) {
                let found_url = matched.as_str().to_string();
                if let Ok(mut lock) = TUNNEL_MANAGER.lock() {
                    if let Some(ref mut mgr) = *lock {
                        mgr.public_url = Some(found_url.clone());
                        mgr.status = "running".to_string();
                        crate::term_log!("🌐 [Cloudflared Tunnel Ready] {}", found_url);
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
    let mut lock = TUNNEL_MANAGER.lock().unwrap();
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
        crate::term_log!("🛑 [Cloudflared Tunnel Stopped]");
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
