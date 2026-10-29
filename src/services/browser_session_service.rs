use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    pub fallback_selectors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_to_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub press_enter: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_y: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallbacks: Vec<String>,
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
    #[serde(default)]
    pub is_loading: bool,
    #[serde(default)]
    pub pending_requests: u32,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub is_network_idle: bool,
}

pub enum BrowserSessionEvent {
    NewSnapshot(BrowserDomSnapshot),
    UpdateTabs(Vec<BrowserTabInfo>),
}

pub static LAST_ACTIVE_SNAPSHOT: Mutex<Option<BrowserDomSnapshot>> = Mutex::new(None);
pub static GLOBAL_BROWSER_TABS: Mutex<Vec<BrowserTabInfo>> = Mutex::new(Vec::new());
pub static PENDING_BROWSER_COMMANDS: Mutex<VecDeque<BrowserCommand>> = Mutex::new(VecDeque::new());
static SESSION_WORKER_TX: Mutex<Option<SyncSender<BrowserSessionEvent>>> = Mutex::new(None);
static SESSION_WORKER_STARTED: AtomicBool = AtomicBool::new(false);
static SNAPSHOT_CONDVAR: Condvar = Condvar::new();
#[derive(Debug, Clone)]
struct CopiedText {
    text: String,
    url: String,
}
static LATEST_COPIED_TEXT: Mutex<Option<CopiedText>> = Mutex::new(None);
static COPIED_TEXT_CONDVAR: Condvar = Condvar::new();
static NEXT_BROWSER_SEARCH_GENERATION: AtomicU64 = AtomicU64::new(0);
static ACTIVE_BROWSER_SEARCH_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Initializes one dedicated background worker thread to synchronize `active_tab_browser.json`.
/// Repeated calls are harmless and retain the sender owned by the original worker.
pub fn init_browser_session_worker() {
    if SESSION_WORKER_STARTED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    let (tx, rx): (
        SyncSender<BrowserSessionEvent>,
        Receiver<BrowserSessionEvent>,
    ) = sync_channel(128);

    match SESSION_WORKER_TX.lock() {
        Ok(mut lock) => *lock = Some(tx),
        Err(error) => {
            eprintln!("[Browser Worker] Failed to store worker sender: {error}");
            SESSION_WORKER_STARTED.store(false, Ordering::Release);
            return;
        }
    }

    if let Err(error) = std::thread::Builder::new()
        .name("ragasukma-browser-worker".to_string())
        .stack_size(512 * 1024)
        .spawn(move || {
            println!("[Browser Worker] Dedicated BrowserSessionWorker active for active_tab_browser.json");
            browser_worker_loop(rx);
        })
    {
        eprintln!("[Browser Worker] Failed to start worker: {error}");
        if let Ok(mut lock) = SESSION_WORKER_TX.lock() {
            *lock = None;
        }
        SESSION_WORKER_STARTED.store(false, Ordering::Release);
    }
}

fn write_snapshot_to_disk(snap: &BrowserDomSnapshot) {
    let file_path = get_active_tab_browser_path();
    let Some(parent) = file_path.parent() else {
        eprintln!("[Browser Worker] Snapshot path has no parent: {}", file_path.display());
        return;
    };

    if let Err(error) = fs::create_dir_all(parent) {
        eprintln!(
            "[Browser Worker] Failed to create snapshot directory {}: {error}",
            parent.display()
        );
        return;
    }

    let json_pretty = match serde_json::to_string_pretty(snap) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("[Browser Worker] Failed to serialize snapshot: {error}");
            return;
        }
    };

    let temporary_path = parent.join(format!(
        ".{}.{}.tmp",
        file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("active_tab_browser.json"),
        std::process::id()
    ));

    if let Err(error) = fs::write(&temporary_path, json_pretty) {
        eprintln!(
            "[Browser Worker] Failed to write temporary snapshot {}: {error}",
            temporary_path.display()
        );
        return;
    }

    if let Err(error) = fs::rename(&temporary_path, &file_path) {
        eprintln!(
            "[Browser Worker] Failed to replace snapshot {}: {error}",
            file_path.display()
        );
        let _ = fs::remove_file(&temporary_path);
    }
}

fn browser_worker_loop(rx: Receiver<BrowserSessionEvent>) {
    let mut current_snapshot: Option<BrowserDomSnapshot> = None;
    let mut current_tabs: Vec<BrowserTabInfo> = Vec::new();

    // Disk writes are coalesced: bursts of events only mark the snapshot dirty,
    // and a single write happens once the queue is quiet or the max delay elapsed.
    const FLUSH_QUIET: Duration = Duration::from_millis(50);
    const FLUSH_MAX_DELAY: Duration = Duration::from_millis(250);
    let mut dirty_since: Option<Instant> = None;

    loop {
        let wait = if dirty_since.is_some() {
            FLUSH_QUIET
        } else {
            Duration::from_millis(500)
        };
        let event = rx.recv_timeout(wait);

        // Force a flush if a continuous event flood keeps postponing it.
        if let Some(since) = dirty_since {
            if event.is_ok() && since.elapsed() >= FLUSH_MAX_DELAY {
                if let Some(ref snap) = current_snapshot {
                    write_snapshot_to_disk(snap);
                }
                dirty_since = Some(Instant::now());
            }
        }

        match event {
            Ok(BrowserSessionEvent::NewSnapshot(mut snap)) => {
                // Abaikan if berasal from dashboard internal
                if snap.url.contains("127.0.0.1:42128") || snap.url.contains("localhost:42128") {
                    continue;
                }

                if snap.timestamp == 0 {
                    snap.timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                }

                // Use latest tabs list if content script snapshot does not carry tabs
                if snap.tabs.is_empty() {
                    snap.tabs = current_tabs.clone();
                } else {
                    current_tabs = snap.tabs.clone();
                }

                // Single Active Tab normalization: tab with matching URL / Title is marked active: true, others false
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

                // Save to in-memory cache global
                if let Ok(mut lock) = LAST_ACTIVE_SNAPSHOT.lock() {
                    *lock = Some(snap.clone());
                }
                SNAPSHOT_CONDVAR.notify_all();
                if let Ok(mut lock) = GLOBAL_BROWSER_TABS.lock() {
                    *lock = snap.tabs.clone();
                }

                // Disk write is coalesced (flushed after a quiet period)
                current_snapshot = Some(snap);
                dirty_since.get_or_insert_with(Instant::now);
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
                            revision: 0,
                            is_loading: false,
                            pending_requests: 0,
                            is_network_idle: true,
                        })
                });

                let mut updated_tabs = tabs;

                // Prioritize tab reported active by browser
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
                        // If active tab changed, update URL & base Title and clean old tab DOM elements
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
                            // Clear old DOM elements to prevent mixing with new page before new snapshot arrives
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
                SNAPSHOT_CONDVAR.notify_all();

                current_snapshot = Some(snap);
                dirty_since.get_or_insert_with(Instant::now);
            }

            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if dirty_since.take().is_some() {
                    if let Some(ref snap) = current_snapshot {
                        write_snapshot_to_disk(snap);
                    }
                }
                // Ensure disk file remains present and populated
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
                println!("[Browser Worker] Channel disconnected, worker exiting.");
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

    // Fallback synchronous if worker not yet initialized
    if let Ok(mut lock) = GLOBAL_BROWSER_TABS.lock() {
        *lock = tabs;
    }
}

pub fn save_browser_snapshot(snapshot: BrowserDomSnapshot) -> Result<String, String> {
    // Abaikan snapshot if berasal from internal dashboard Ragasukma sendiri
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

    // Fallback synchronous if worker not yet active
    write_snapshot_to_disk(&snapshot);
    if let Ok(mut lock) = LAST_ACTIVE_SNAPSHOT.lock() {
        *lock = Some(snapshot);
    }
    SNAPSHOT_CONDVAR.notify_all();
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
        // Limit queue to prevent build-up
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
/// Check if tab already exists in browser. If it exists, switch & navigate to it without opening a new window/tab.
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
            "[Smart Tab Freshness] Found existing tab for domain '{}' (ID: {}, Title: '{}'). Closing stale tab to ensure clean DOM state.",
            d_hint, tab.id, tab.title
        );
        send_browser_command(BrowserCommand {
            action: "CLOSE_TAB".to_string(),
            query: Some(tab.id.to_string()),
            ..Default::default()
        });
        // Non-blocking delay to let browser close stale tab cleanly before opening fresh one
        // crate::assets::open_browser internally already spawns a background thread
        // with a 150ms sleep delay, so we can call it directly without double-spawning.
        crate::assets::open_browser(url);
        return;
    }

    crate::assets::open_browser(url);
}

pub fn format_open_tabs_for_llm() -> String {
    let tabs = get_browser_tabs();
    if tabs.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    lines.push("\n###  OPEN BROWSER TABS:".to_string());
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
        revision: 0,
        is_loading: false,
        pending_requests: 0,
        is_network_idle: true,
    };
    write_snapshot_to_disk(&initial_snap);
    println!("[Active Tab] Initializing active tab snapshot at: {:?}", file);
}

pub fn get_active_tab_browser_path() -> PathBuf {
    if PathBuf::from("Cargo.toml").exists() {
        return PathBuf::from("assets/active_tab_browser.json");
    }
    let cache_dir = crate::config::get_app_cache_dir();
    let _ = fs::create_dir_all(&cache_dir);
    cache_dir.join("active_tab_browser.json")
}

pub fn get_latest_snapshot() -> Option<BrowserDomSnapshot> {
    if let Ok(lock) = LAST_ACTIVE_SNAPSHOT.lock() {
        if let Some(ref snap) = *lock {
            return Some(snap.clone());
        }
    }
    // Fallback: read directly from assets/active_tab_browser.json on disk
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

pub fn begin_browser_search() -> u64 {
    let generation = NEXT_BROWSER_SEARCH_GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
    ACTIVE_BROWSER_SEARCH_GENERATION.store(generation, Ordering::Release);
    reset_copied_text();
    SNAPSHOT_CONDVAR.notify_all();
    COPIED_TEXT_CONDVAR.notify_all();
    generation
}

pub fn is_active_browser_search(generation: u64) -> bool {
    ACTIVE_BROWSER_SEARCH_GENERATION.load(Ordering::Acquire) == generation
}

pub fn reset_copied_text() {
    if let Ok(mut lock) = LATEST_COPIED_TEXT.lock() {
        *lock = None;
    }
}

pub fn notify_copied_text(text: String, url: String) {
    if let Ok(mut lock) = LATEST_COPIED_TEXT.lock() {
        *lock = Some(CopiedText { text, url });
        COPIED_TEXT_CONDVAR.notify_all();
    }
}

pub fn wait_for_copied_text_matching<F>(
    timeout: Duration,
    mut predicate: F,
) -> Option<String>
where
    F: FnMut(&str) -> bool,
{
    let start = Instant::now();
    let mut lock = LATEST_COPIED_TEXT.lock().ok()?;
    loop {
        if let Some(copied) = lock.as_ref() {
            if predicate(&copied.url) {
                return Some(copied.text.clone());
            }
        }
        let remaining = timeout.checked_sub(start.elapsed())?;
        let (new_lock, result) = COPIED_TEXT_CONDVAR.wait_timeout(lock, remaining).ok()?;
        lock = new_lock;
        if result.timed_out() {
            return None;
        }
    }
}

pub fn wait_for_copied_text_matching_for_search<F>(
    generation: u64,
    timeout: Duration,
    mut predicate: F,
) -> Option<String>
where
    F: FnMut(&str) -> bool,
{
    let start = Instant::now();
    let mut lock = LATEST_COPIED_TEXT.lock().ok()?;
    loop {
        if !is_active_browser_search(generation) {
            return None;
        }
        if let Some(copied) = lock.as_ref() {
            if predicate(&copied.url) {
                return Some(copied.text.clone());
            }
        }
        let remaining = timeout.checked_sub(start.elapsed())?;
        // A short wait bounds cancellation latency even if a notifier is missed.
        let wait_for = remaining.min(Duration::from_millis(100));
        let (new_lock, result) = COPIED_TEXT_CONDVAR.wait_timeout(lock, wait_for).ok()?;
        lock = new_lock;
        if result.timed_out() && start.elapsed() >= timeout {
            return None;
        }
    }
}

pub fn wait_for_copied_text(timeout: Duration) -> Option<String> {
    wait_for_copied_text_matching(timeout, |_| true)
}

/// Updates a summary only when the selected snapshot still identifies the same page revision.
pub fn update_snapshot_summary(expected_url: &str, expected_revision: u64, text_summary: String) -> bool {
    let Ok(mut lock) = LAST_ACTIVE_SNAPSHOT.lock() else {
        return false;
    };
    let Some(snapshot) = lock.as_mut() else {
        return false;
    };
    if snapshot.url != expected_url || snapshot.revision != expected_revision {
        return false;
    }
    snapshot.text_summary = text_summary;
    snapshot.revision = snapshot.revision.saturating_add(1);
    let snapshot = snapshot.clone();
    SNAPSHOT_CONDVAR.notify_all();
    drop(lock);
    write_snapshot_to_disk(&snapshot);
    true
}

pub fn wait_for_snapshot_matching<F>(timeout: Duration, mut predicate: F) -> Option<BrowserDomSnapshot>
where
    F: FnMut(&BrowserDomSnapshot) -> bool,
{
    let start = Instant::now();
    let mut lock = LAST_ACTIVE_SNAPSHOT.lock().ok()?;

    while start.elapsed() < timeout {
        if let Some(ref snap) = *lock {
            if predicate(snap) {
                return Some(snap.clone());
            }
        }
        let remaining = timeout.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            break;
        }
        let (new_lock, timeout_res) = SNAPSHOT_CONDVAR.wait_timeout(lock, remaining).ok()?;
        lock = new_lock;
        if timeout_res.timed_out() {
            if let Some(ref snap) = *lock {
                if predicate(snap) {
                    return Some(snap.clone());
                }
            }
            break;
        }
    }
    None
}

pub fn wait_for_snapshot_matching_for_search<F>(
    generation: u64,
    timeout: Duration,
    mut predicate: F,
) -> Option<BrowserDomSnapshot>
where
    F: FnMut(&BrowserDomSnapshot) -> bool,
{
    let start = Instant::now();
    let mut lock = LAST_ACTIVE_SNAPSHOT.lock().ok()?;
    loop {
        if !is_active_browser_search(generation) {
            return None;
        }
        if let Some(snap) = lock.as_ref() {
            if predicate(snap) {
                return Some(snap.clone());
            }
        }
        let remaining = timeout.checked_sub(start.elapsed())?;
        // Generation changes notify this condvar; the bounded wait is a fallback.
        let wait_for = remaining.min(Duration::from_millis(100));
        let (new_lock, result) = SNAPSHOT_CONDVAR.wait_timeout(lock, wait_for).ok()?;
        lock = new_lock;
        if result.timed_out() && start.elapsed() >= timeout {
            return None;
        }
    }
}

pub fn handle_post_browser_copied_text(body: &str) -> (String, &'static str) {
    #[derive(Deserialize)]
    struct CopiedTextPayload {
        text: String,
        #[serde(default)]
        #[allow(dead_code)]
        url: String,
    }

    match serde_json::from_str::<CopiedTextPayload>(body) {
        Ok(payload) => {
            notify_copied_text(payload.text, payload.url);
            (r#"{"status":"ok"}"#.to_string(), "application/json")
        }
        Err(e) => (
            format!(
                r#"{{"status":"error","message":"Invalid copied text JSON: {}"}}"#,
                e
            ),
            "application/json",
        ),
    }
}