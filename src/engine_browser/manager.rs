use super::types::{BrowserCommand, BrowserTabInfo};
use super::ws::make_websocket_text_frame;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct BrowserSessionManager {
    pub active_tabs: HashMap<String, BrowserTabInfo>,
    pub active_connections: Vec<TcpStream>,
}

impl BrowserSessionManager {
    pub fn new() -> Self {
        Self {
            active_tabs: HashMap::new(),
            active_connections: Vec::new(),
        }
    }
}

impl Default for BrowserSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_BROWSER_MANAGER: Mutex<Option<Arc<Mutex<BrowserSessionManager>>>> = Mutex::new(None);

pub fn get_browser_manager() -> Arc<Mutex<BrowserSessionManager>> {
    let mut lock = GLOBAL_BROWSER_MANAGER.lock().unwrap();
    if let Some(mgr) = lock.as_ref() {
        return Arc::clone(mgr);
    }
    let mgr = Arc::new(Mutex::new(BrowserSessionManager::new()));
    *lock = Some(Arc::clone(&mgr));
    mgr
}

/// Kirim perintah aksi ke semua tab browser yang terhubung
pub fn broadcast_command(command: &BrowserCommand) {
    let json = match serde_json::to_string(command) {
        Ok(j) => j,
        Err(_) => return,
    };

    let mgr = get_browser_manager();
    let mut mgr_lock = mgr.lock().unwrap();
    let frame = make_websocket_text_frame(&json);

    mgr_lock.active_connections.retain_mut(|stream| {
        stream.write_all(&frame).is_ok()
    });
}

/// Dapatkan ringkasan tab browser yang sedang aktif dibuka pengguna
pub fn get_active_tabs_summary() -> String {
    let mgr = get_browser_manager();
    let lock = match mgr.lock() {
        Ok(l) => l,
        Err(_) => return String::new(),
    };

    if lock.active_tabs.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for tab in lock.active_tabs.values() {
        if !tab.title.trim().is_empty() {
            lines.push(format!("- Domain: `{}` | Judul Halaman/Tab: \"{}\"", tab.domain, tab.title.trim()));
        }
    }

    if lines.is_empty() {
        return String::new();
    }

    format!(
        "\n5. TAB BROWSER YANG SEDANG DIBUKA PENGGUNA SAAT INI:\n\
Kamu bisa melihat halaman / tab browser yang sedang aktif dibuka pengguna di bawah ini:\n{}\n\
Gunakan informasi ini jika pengguna bertanya tentang apa yang sedang ia tonton, dengarkan, atau baca!\n",
        lines.join("\n")
    )
}
