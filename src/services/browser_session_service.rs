use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTabInfo {
    pub id: i64,
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub window_id: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrowserCommand {
    pub action: String, // "SWITCH_OR_NAVIGATE", "CLOSE_TAB", "MEDIA_PLAY", "MEDIA_PAUSE", "MEDIA_TOGGLE", "MEDIA_NEXT", "MEDIA_PREV", "CLICK_ELEMENT", "TYPE_TEXT", "SCROLL"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_to_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub press_enter: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_y: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomElementInfo {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserDomSnapshot {
    pub url: String,
    pub domain: String,
    pub title: String,
    pub timestamp: u64,
    #[serde(default)]
    pub tabs: Vec<BrowserTabInfo>,
    #[serde(default)]
    pub buttons: Vec<DomElementInfo>,
    #[serde(default)]
    pub inputs: Vec<DomElementInfo>,
    #[serde(default)]
    pub texts: Vec<String>,
    #[serde(default)]
    pub text_summary: String,
}

pub enum BrowserSessionEvent {
    NewSnapshot(BrowserDomSnapshot),
    UpdateTabs(Vec<BrowserTabInfo>),
}

pub static LAST_ACTIVE_SNAPSHOT: Mutex<Option<BrowserDomSnapshot>> = Mutex::new(None);
pub static GLOBAL_BROWSER_TABS: Mutex<Vec<BrowserTabInfo>> = Mutex::new(Vec::new());
pub static PENDING_BROWSER_COMMANDS: Mutex<VecDeque<BrowserCommand>> = Mutex::new(VecDeque::new());
static SESSION_WORKER_TX: Mutex<Option<SyncSender<BrowserSessionEvent>>> = Mutex::new(None);

/// Inisialisasi dedicated background worker thread untuk sinkronisasi `active_tab_browser.json`
pub fn init_browser_session_worker() {
    let (tx, rx): (
        SyncSender<BrowserSessionEvent>,
        Receiver<BrowserSessionEvent>,
    ) = sync_channel(128);

    if let Ok(mut lock) = SESSION_WORKER_TX.lock() {
        *lock = Some(tx);
    }

    let _ = std::thread::Builder::new()
        .name("ragasukma-browser-worker".to_string())
        .spawn(move || {
            println!("🚀 [Browser Worker] Dedicated BrowserSessionWorker active for active_tab_browser.json");
            browser_worker_loop(rx);
        });
}

fn write_snapshot_to_disk(snap: &BrowserDomSnapshot) {
    let file_path = get_active_tab_browser_path();
    if let Some(parent) = file_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(json_pretty) = serde_json::to_string_pretty(snap) {
        let _ = fs::write(&file_path, &json_pretty);
        let local = PathBuf::from("assets/active_tab_browser.json");
        if local != file_path {
            if let Some(parent) = local.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&local, &json_pretty);
        }
    }
}

fn browser_worker_loop(rx: Receiver<BrowserSessionEvent>) {
    let mut current_snapshot: Option<BrowserDomSnapshot> = None;
    let mut current_tabs: Vec<BrowserTabInfo> = Vec::new();

    loop {
        // Tunggu event dengan timeout 500ms agar tetap responsif dan konsisten di disk
        let event = rx.recv_timeout(Duration::from_millis(500));

        match event {
            Ok(BrowserSessionEvent::NewSnapshot(mut snap)) => {
                // Abaikan jika berasal dari dashboard internal
                if snap.url.contains("127.0.0.1:42128") || snap.url.contains("localhost:42128") {
                    continue;
                }

                if snap.timestamp == 0 {
                    snap.timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                }

                // Gunakan daftar tabs terkini jika snapshot dari content script tidak membawa tabs
                if snap.tabs.is_empty() {
                    snap.tabs = current_tabs.clone();
                } else {
                    current_tabs = snap.tabs.clone();
                }

                // Normalisasi Single Active Tab: tab yang URL / Title cocok diberi active: true, sisanya false
                let snap_url = snap.url.to_lowercase();
                let mut found_active = false;
                for t in snap.tabs.iter_mut() {
                    if t.active && !found_active {
                        t.url = snap.url.clone();
                        t.title = snap.title.clone();
                        found_active = true;
                    } else {
                        let t_url = t.url.to_lowercase();
                        if !found_active
                            && !snap_url.is_empty()
                            && (t_url == snap_url
                                || snap_url.starts_with(&t_url)
                                || t_url.starts_with(&snap_url))
                        {
                            t.active = true;
                            t.url = snap.url.clone();
                            t.title = snap.title.clone();
                            found_active = true;
                        } else {
                            t.active = false;
                        }
                    }
                }

                // Simpan ke in-memory cache global
                if let Ok(mut lock) = LAST_ACTIVE_SNAPSHOT.lock() {
                    *lock = Some(snap.clone());
                }
                if let Ok(mut lock) = GLOBAL_BROWSER_TABS.lock() {
                    *lock = snap.tabs.clone();
                }

                // Tulis ke file disk
                write_snapshot_to_disk(&snap);
                current_snapshot = Some(snap);
            }

            Ok(BrowserSessionEvent::UpdateTabs(tabs)) => {
                current_tabs = tabs.clone();
                if let Ok(mut lock) = GLOBAL_BROWSER_TABS.lock() {
                    *lock = tabs.clone();
                }

                let mut snap = current_snapshot.take().unwrap_or_else(|| {
                    LAST_ACTIVE_SNAPSHOT
                        .lock()
                        .ok()
                        .and_then(|l| l.clone())
                        .unwrap_or_else(|| BrowserDomSnapshot {
                            url: String::new(),
                            domain: String::new(),
                            title: String::new(),
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0),
                            tabs: Vec::new(),
                            buttons: Vec::new(),
                            inputs: Vec::new(),
                            texts: Vec::new(),
                            text_summary: String::new(),
                        })
                });

                let mut updated_tabs = tabs;

                // Prioritaskan tab yang dilaporkan aktif oleh browser
                let mut active_idx = updated_tabs.iter().position(|t| t.active);
                if active_idx.is_none() {
                    let snap_url = snap.url.to_lowercase();
                    if !snap_url.is_empty() {
                        active_idx = updated_tabs.iter().position(|t| {
                            t.url.to_lowercase() == snap_url
                                || snap_url.starts_with(&t.url.to_lowercase())
                                || t.url.to_lowercase().starts_with(&snap_url)
                        });
                    }
                }

                for (idx, t) in updated_tabs.iter_mut().enumerate() {
                    let is_act = active_idx == Some(idx);
                    t.active = is_act;
                    if is_act && !t.url.is_empty() {
                        // Jika tab aktif berubah, update URL & Title dasar dan bersihkan elemen DOM tab lama
                        if snap.url != t.url {
                            snap.url = t.url.clone();
                            snap.title = t.title.clone();
                            snap.domain = t
                                .url
                                .split("://")
                                .nth(1)
                                .unwrap_or(&t.url)
                                .split('/')
                                .next()
                                .unwrap_or("")
                                .to_string();
                            // Kosongkan elemen DOM lama agar tidak tercampur dengan halaman baru sebelum snapshot baru masuk
                            snap.buttons.clear();
                            snap.inputs.clear();
                            snap.texts.clear();
                            snap.text_summary.clear();
                        }
                    }
                }

                snap.tabs = updated_tabs;
                snap.timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                if let Ok(mut lock) = LAST_ACTIVE_SNAPSHOT.lock() {
                    *lock = Some(snap.clone());
                }

                write_snapshot_to_disk(&snap);
                current_snapshot = Some(snap);
            }

            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Pastikan file disk tetap ada dan terisi
                let target_path = get_active_tab_browser_path();
                if !target_path.exists() {
                    if let Ok(lock) = LAST_ACTIVE_SNAPSHOT.lock() {
                        if let Some(ref snap) = *lock {
                            write_snapshot_to_disk(snap);
                        }
                    }
                }
            }

            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                println!("🛑 [Browser Worker] Channel disconnected, worker exiting.");
                break;
            }
        }
    }
}

pub fn update_browser_tabs(tabs: Vec<BrowserTabInfo>) {
    if let Ok(lock) = SESSION_WORKER_TX.lock() {
        if let Some(ref tx) = *lock {
            let _ = tx.try_send(BrowserSessionEvent::UpdateTabs(tabs));
            return;
        }
    }

    // Fallback synchronous jika worker belum diinisialisasi
    if let Ok(mut lock) = GLOBAL_BROWSER_TABS.lock() {
        *lock = tabs;
    }
}

pub fn save_browser_snapshot(snapshot: BrowserDomSnapshot) -> Result<String, String> {
    // Abaikan snapshot jika berasal dari internal dashboard Ragasukma sendiri
    if snapshot.url.contains("127.0.0.1:42128") || snapshot.url.contains("localhost:42128") {
        return Ok("Ignored internal dashboard snapshot".to_string());
    }

    let file_path = get_active_tab_browser_path();

    if let Ok(lock) = SESSION_WORKER_TX.lock() {
        if let Some(ref tx) = *lock {
            let _ = tx.try_send(BrowserSessionEvent::NewSnapshot(snapshot));
            return Ok(file_path.to_string_lossy().to_string());
        }
    }

    // Fallback synchronous jika worker belum aktif
    write_snapshot_to_disk(&snapshot);
    if let Ok(mut lock) = LAST_ACTIVE_SNAPSHOT.lock() {
        *lock = Some(snapshot);
    }
    Ok(file_path.to_string_lossy().to_string())
}

pub fn get_browser_tabs() -> Vec<BrowserTabInfo> {
    if let Ok(lock) = GLOBAL_BROWSER_TABS.lock() {
        lock.clone()
    } else {
        Vec::new()
    }
}

pub fn send_browser_command(cmd: BrowserCommand) {
    if let Ok(mut lock) = PENDING_BROWSER_COMMANDS.lock() {
        // Batasi antrean agar tidak menumpuk
        if lock.len() > 20 {
            lock.pop_front();
        }
        lock.push_back(cmd);
    }
}

pub fn get_next_browser_command() -> Option<BrowserCommand> {
    if let Ok(mut lock) = PENDING_BROWSER_COMMANDS.lock() {
        lock.pop_front()
    } else {
        None
    }
}

/// Smart Browser Tab Navigator:
/// Cek apakah tab sudah ada di browser. Jika sudah ada, switch & navigate tab tersebut tanpa membuka tab/jendela baru.
pub fn smart_open_or_switch_tab(url: &str, query: Option<&str>, domain_hint: Option<&str>) {
    let tabs = get_browser_tabs();
    let auto_domain = url
        .split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .replace("www.", "");

    let d_hint = domain_hint.unwrap_or(&auto_domain);
    let q_hint = query.unwrap_or("");

    let existing_tab = tabs.iter().find(|t| {
        let t_url = t.url.to_lowercase();
        let t_title = t.title.to_lowercase();
        if !d_hint.is_empty() && t_url.contains(&d_hint.to_lowercase()) {
            return true;
        }
        if !q_hint.is_empty()
            && (t_url.contains(&q_hint.to_lowercase()) || t_title.contains(&q_hint.to_lowercase()))
        {
            return true;
        }
        false
    });

    if let Some(tab) = existing_tab {
        println!(
            "🔄 [Smart Tab Reuse] Found existing open tab (ID: {}, Title: '{}'), activating & navigating without opening a new tab.",
            tab.id, tab.title
        );
        send_browser_command(BrowserCommand {
            action: "SWITCH_OR_NAVIGATE".to_string(),
            query: if q_hint.is_empty() {
                None
            } else {
                Some(q_hint.to_string())
            },
            domain: if d_hint.is_empty() {
                None
            } else {
                Some(d_hint.to_string())
            },
            new_url: Some(url.to_string()),
            ..Default::default()
        });
    } else {
        // Belum ada tab yang cocok atau ekstensi belum sinkron, buka tab baru via OS
        crate::assets::open_browser(url);
    }
}

pub fn format_open_tabs_for_llm() -> String {
    let tabs = get_browser_tabs();
    if tabs.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push("\n### 📑 DAFTAR TAB BROWSER TERBUKA:".to_string());
    for (i, t) in tabs.iter().take(8).enumerate() {
        let active_mark = if t.active { " `[ACTIVE_TAB]`" } else { "" };
        lines.push(format!(
            "{}. **{}**{} (URL: `{}`) [Tab ID: `{}`]",
            i + 1,
            if t.title.is_empty() {
                "Untitled"
            } else {
                &t.title
            },
            active_mark,
            t.url,
            t.id
        ));
    }
    lines.join("\n")
}

pub fn cleanup_browser_sessions_on_startup() {
    let file = get_active_tab_browser_path();
    let initial_snap = BrowserDomSnapshot {
        url: String::new(),
        domain: String::new(),
        title: String::new(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        tabs: Vec::new(),
        buttons: Vec::new(),
        inputs: Vec::new(),
        texts: Vec::new(),
        text_summary: String::new(),
    };
    if let Ok(json_pretty) = serde_json::to_string_pretty(&initial_snap) {
        let _ = fs::write(&file, &json_pretty);
        let local = PathBuf::from("assets/active_tab_browser.json");
        if local != file {
            let _ = fs::write(&local, &json_pretty);
        }
    }
    // Hapus juga direktori legacy browser_sessions jika ada
    let old_dir = PathBuf::from("assets/browser_sessions");
    if old_dir.exists() {
        let _ = fs::remove_dir_all(&old_dir);
    }
    println!("🧹 [Active Tab] Initializing 'assets/active_tab_browser.json' for new session.");
}

pub fn get_active_tab_browser_path() -> PathBuf {
    let local = PathBuf::from("assets/active_tab_browser.json");
    if local.exists() || PathBuf::from("Cargo.toml").exists() {
        return local;
    }
    crate::assets::get_assets_dir().join("active_tab_browser.json")
}

pub fn get_latest_snapshot() -> Option<BrowserDomSnapshot> {
    if let Ok(lock) = LAST_ACTIVE_SNAPSHOT.lock() {
        if let Some(ref snap) = *lock {
            return Some(snap.clone());
        }
    }
    // Fallback: baca langsung file assets/active_tab_browser.json dari disk
    let path = get_active_tab_browser_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(snap) = serde_json::from_str::<BrowserDomSnapshot>(&content) {
                return Some(snap);
            }
        }
    }
    None
}

pub fn handle_post_browser_snapshot(body: &str) -> (String, &'static str) {
    match serde_json::from_str::<BrowserDomSnapshot>(body) {
        Ok(snapshot) => match save_browser_snapshot(snapshot) {
            Ok(path) => (
                format!(r#"{{"status":"ok","path":"{}"}}"#, path.replace('\\', "/")),
                "application/json",
            ),
            Err(e) => (
                format!(r#"{{"status":"error","message":"{}"}}"#, e),
                "application/json",
            ),
        },
        Err(e) => (
            format!(
                r#"{{"status":"error","message":"Invalid snapshot JSON: {}"}}"#,
                e
            ),
            "application/json",
        ),
    }
}

pub fn handle_post_browser_tabs(body: &str) -> (String, &'static str) {
    #[derive(Deserialize)]
    struct TabsPayload {
        tabs: Vec<BrowserTabInfo>,
    }

    match serde_json::from_str::<TabsPayload>(body) {
        Ok(payload) => {
            let count = payload.tabs.len();
            update_browser_tabs(payload.tabs);
            (
                format!(r#"{{"status":"ok","synced_tabs":{}}}"#, count),
                "application/json",
            )
        }
        Err(e) => (
            format!(
                r#"{{"status":"error","message":"Invalid tabs payload: {}"}}"#,
                e
            ),
            "application/json",
        ),
    }
}

pub fn handle_get_browser_tabs() -> (String, &'static str) {
    let tabs = get_browser_tabs();
    match serde_json::to_string(&tabs) {
        Ok(json) => (json, "application/json"),
        Err(e) => (
            format!(r#"{{"status":"error","message":"{}"}}"#, e),
            "application/json",
        ),
    }
}

pub fn handle_get_next_browser_command() -> (String, &'static str) {
    if let Some(cmd) = get_next_browser_command() {
        match serde_json::to_string(&cmd) {
            Ok(cmd_json) => (
                format!(r#"{{"has_command":true,"command":{}}}"#, cmd_json),
                "application/json",
            ),
            Err(_) => (r#"{"has_command":false}"#.to_string(), "application/json"),
        }
    } else {
        (r#"{"has_command":false}"#.to_string(), "application/json")
    }
}
