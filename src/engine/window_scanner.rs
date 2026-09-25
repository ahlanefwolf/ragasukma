use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesktopElement {
    pub id: usize,
    pub label: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<[i32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<[i32; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowItem {
    pub id: usize,
    pub app_name: String,
    pub title: String,
    pub active: bool,
    pub window_id: String,
}

fn default_source() -> String {
    "native_accessibility".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWindowSnapshot {
    pub app_name: String,
    pub window_title: String,
    pub window_id: String,
    #[serde(default = "default_source")]
    pub source: String,
    pub timestamp: u64,
    pub windows: Vec<WindowItem>,
    pub buttons: Vec<DesktopElement>,
    pub inputs: Vec<DesktopElement>,
    pub texts: Vec<String>,
    pub text_summary: String,
}

impl Default for ActiveWindowSnapshot {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            window_title: String::new(),
            window_id: String::new(),
            source: default_source(),
            timestamp: 0,
            windows: Vec::new(),
            buttons: Vec::new(),
            inputs: Vec::new(),
            texts: Vec::new(),
            text_summary: String::new(),
        }
    }
}

impl ActiveWindowSnapshot {
    pub fn save_to_file(&self) -> Result<(), String> {
        let path = get_snapshot_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Gagal serialize snapshot: {e}"))?;
        fs::write(&path, &json)
            .map_err(|e| format!("Gagal menulis {}: {e}", path.display()))?;

        // Tulis juga ke working directory lokal jika berbeda
        let local = PathBuf::from("assets/active_window_app.json");
        if local != path {
            if let Some(parent) = local.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&local, &json);
        }

        Ok(())
    }

    pub fn load_from_file() -> Result<Self, String> {
        let path = get_snapshot_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Gagal membaca {}: {e}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Gagal parse json snapshot: {e}"))
    }
}

pub fn get_snapshot_path() -> PathBuf {
    let local = PathBuf::from("assets/active_window_app.json");
    if local.exists() || PathBuf::from("Cargo.toml").exists() {
        return local;
    }
    crate::assets::get_assets_dir().join("active_window_app.json")
}

pub fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Versi sinkron untuk dipanggil langsung dari thread sinkron (seperti HTTP API / action runner)
pub fn scan_active_window_sync() -> Result<ActiveWindowSnapshot, String> {
    #[cfg(target_os = "linux")]
    {
        atspi::zbus::block_on(scan_active_window())
    }
    #[cfg(target_os = "windows")]
    {
        scan_active_window_windows()
    }
    #[cfg(target_os = "macos")]
    {
        scan_active_window_macos()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err("Sistem operasi tidak didukung".to_string())
    }
}

/// Pindai aplikasi & jendela aktif menggunakan Accessibility Tree OS (Cross-Platform)
pub async fn scan_active_window() -> Result<ActiveWindowSnapshot, String> {
    #[cfg(target_os = "linux")]
    {
        scan_active_window_linux().await
    }
    #[cfg(target_os = "windows")]
    {
        scan_active_window_windows()
    }
    #[cfg(target_os = "macos")]
    {
        scan_active_window_macos()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err("Sistem operasi tidak didukung untuk scanning accessibility window".to_string())
    }
}

#[cfg(target_os = "linux")]
static ATSPI_CONN: std::sync::Mutex<Option<atspi::connection::AccessibilityConnection>> =
    std::sync::Mutex::new(None);

#[cfg(target_os = "linux")]
async fn get_or_init_atspi_conn() -> Option<atspi::connection::AccessibilityConnection> {
    if let Ok(lock) = ATSPI_CONN.lock() {
        if let Some(ref conn) = *lock {
            return Some(conn.clone());
        }
    }

    match atspi::connection::AccessibilityConnection::new().await {
        Ok(conn) => {
            if let Ok(mut lock) = ATSPI_CONN.lock() {
                *lock = Some(conn.clone());
            }
            Some(conn)
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!("⚠️  [Desktop Scanner] AT-SPI D-Bus init error: {e}");
            None
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
struct CompositorWindow {
    pub id: i64,
    pub app: String,
    pub title: String,
    pub focused: bool,
}

#[cfg(target_os = "linux")]
fn get_sway_windows() -> Option<Vec<CompositorWindow>> {
    if std::env::var("SWAYSOCK").is_err() {
        return None;
    }

    let output = std::process::Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let val = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok()?;

    fn collect_nodes(node: &serde_json::Value, out: &mut Vec<CompositorWindow>) {
        let name = node.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let app = node
            .get("app_id")
            .and_then(|a| a.as_str())
            .or_else(|| {
                node.get("window_properties")
                    .and_then(|wp| wp.get("class"))
                    .and_then(|c| c.as_str())
            })
            .unwrap_or("");

        if !name.is_empty() && !app.is_empty() && !app.eq_ignore_ascii_case("waybar") {
            let id = node.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
            let focused = node.get("focused").and_then(|f| f.as_bool()).unwrap_or(false);
            out.push(CompositorWindow {
                id,
                app: app.to_string(),
                title: name.to_string(),
                focused,
            });
        }

        let empty_vec = Vec::new();
        let nodes = node.get("nodes").and_then(|n| n.as_array()).unwrap_or(&empty_vec);
        for child in nodes {
            collect_nodes(child, out);
        }

        let floating = node.get("floating_nodes").and_then(|n| n.as_array()).unwrap_or(&empty_vec);
        for child in floating {
            collect_nodes(child, out);
        }
    }

    let mut wins = Vec::new();
    collect_nodes(&val, &mut wins);
    if wins.is_empty() {
        None
    } else {
        Some(wins)
    }
}

#[cfg(target_os = "linux")]
async fn scan_active_window_linux() -> Result<ActiveWindowSnapshot, String> {
    use atspi::proxy::accessible::ObjectRefExt;
    use atspi::{Role, State};

    let mut snapshot = ActiveWindowSnapshot {
        timestamp: current_epoch_ms(),
        ..Default::default()
    };

    let sway_windows = get_sway_windows();

    let mut all_windows: Vec<WindowItem> = Vec::new();
    let mut window_counter = 1;

    let mut focused_app = String::new();
    let mut focused_title = String::new();
    let mut focused_win_id = String::new();

    if let Some(ref wins) = sway_windows {
        for w in wins {
            all_windows.push(WindowItem {
                id: window_counter,
                app_name: w.app.clone(),
                title: w.title.clone(),
                active: w.focused,
                window_id: format!("sway:{}", w.id),
            });
            window_counter += 1;

            if w.focused {
                focused_app = w.app.clone();
                focused_title = w.title.clone();
                focused_win_id = format!("sway:{}", w.id);
            }
        }
    }

    if !focused_app.is_empty() {
        snapshot.app_name = focused_app.clone();
        snapshot.window_title = focused_title.clone();
        snapshot.window_id = focused_win_id.clone();
    }

    // Inisialisasi AT-SPI untuk mengekstrak elemen tombol/menu jika didukung
    if let Some(conn) = get_or_init_atspi_conn().await {
        let zbus_conn = conn.connection();
        if let Ok(desktop) = conn.root_accessible_on_registry().await {
            if let Ok(children) = desktop.get_children().await {
                let mut target_win_ref: Option<atspi::ObjectRefOwned> = None;

                for app_ref in children {
                    let Ok(app_proxy) = app_ref.as_accessible_proxy(zbus_conn).await else {
                        continue;
                    };

                    let app_name = app_proxy.name().await.unwrap_or_default();
                    if app_name.is_empty() {
                        continue;
                    }

                    if let Ok(windows) = app_proxy.get_children().await {
                        for win_ref in windows {
                            let Ok(win_proxy) = win_ref.as_accessible_proxy(zbus_conn).await else {
                                continue;
                            };

                            let role = win_proxy.get_role().await.unwrap_or(Role::Invalid);
                            let win_name = win_proxy.name().await.unwrap_or_default();
                            let state = win_proxy.get_state().await.unwrap_or_default();
                            let is_active = state.contains(State::Active) || state.contains(State::Focused);

                            if !matches!(role, Role::Window | Role::Frame | Role::Dialog | Role::Application) || win_name.is_empty() {
                                continue;
                            }

                            // Jika bukan dari Sway, kumpulkan ke all_windows
                            if sway_windows.is_none() {
                                let is_panel = app_name.eq_ignore_ascii_case("waybar") || app_name.eq_ignore_ascii_case("desktop");
                                if !is_panel {
                                    all_windows.push(WindowItem {
                                        id: window_counter,
                                        app_name: app_name.clone(),
                                        title: win_name.clone(),
                                        active: is_active,
                                        window_id: win_ref.path_as_str().to_string(),
                                    });
                                    window_counter += 1;

                                    if is_active && snapshot.app_name.is_empty() {
                                        snapshot.app_name = app_name.clone();
                                        snapshot.window_title = win_name.clone();
                                        snapshot.window_id = win_ref.path_as_str().to_string();
                                        target_win_ref = Some(win_ref.clone());
                                    }
                                }
                            } else {
                                // Cocokkan window AT-SPI dengan focused window dari Sway
                                let app_lower = app_name.to_lowercase();
                                let f_app_lower = focused_app.to_lowercase();
                                let app_matches = app_lower.contains(&f_app_lower) || f_app_lower.contains(&app_lower);
                                let title_matches = !focused_title.is_empty() && (win_name.contains(&focused_title) || focused_title.contains(&win_name));

                                if app_matches || title_matches {
                                    target_win_ref = Some(win_ref.clone());
                                    break;
                                }
                            }
                        }
                    }

                    if target_win_ref.is_some() {
                        break;
                    }
                }

                // Ekstrak elemen tombol & form input dari target window
                if let Some(win_ref) = target_win_ref {
                    if let Ok(win_proxy) = win_ref.as_accessible_proxy(zbus_conn).await {
                        let mut element_id_counter = 1;
                        let mut queue: Vec<(atspi::ObjectRefOwned, usize)> = Vec::new();
                        if let Ok(win_children) = win_proxy.get_children().await {
                            for c in win_children {
                                queue.push((c, 1));
                            }
                        }

                        while let Some((node_ref, depth)) = queue.pop() {
                            let Ok(node_proxy) = node_ref.as_accessible_proxy(zbus_conn).await else {
                                continue;
                            };

                            let role = node_proxy.get_role().await.unwrap_or(Role::Invalid);
                            let name = node_proxy.name().await.unwrap_or_default().trim().to_string();

                            if !name.is_empty() {
                                match role {
                                    Role::Button
                                    | Role::ToggleButton
                                    | Role::RadioButton
                                    | Role::Menu
                                    | Role::MenuItem
                                    | Role::CheckMenuItem
                                    | Role::RadioMenuItem
                                    | Role::PageTab
                                    | Role::ListItem
                                    | Role::TableCell => {
                                        snapshot.buttons.push(DesktopElement {
                                            id: element_id_counter,
                                            label: name.clone(),
                                            role: format!("{:?}", role).to_lowercase(),
                                            ..Default::default()
                                        });
                                        element_id_counter += 1;
                                    }
                                    Role::Text | Role::Entry | Role::PasswordText => {
                                        snapshot.inputs.push(DesktopElement {
                                            id: element_id_counter,
                                            label: name.clone(),
                                            role: "input".to_string(),
                                            ..Default::default()
                                        });
                                        element_id_counter += 1;
                                    }
                                    Role::Label | Role::Static | Role::Heading => {
                                        if !snapshot.texts.contains(&name) {
                                            snapshot.texts.push(name.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if depth < 6 {
                                if let Ok(children) = node_proxy.get_children().await {
                                    for child in children {
                                        queue.push((child, depth + 1));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    snapshot.windows = all_windows;
    if snapshot.app_name.is_empty() {
        snapshot.text_summary = "Tidak ada jendela aplikasi yang terfokus".to_string();
    }

    Ok(snapshot)
}

/// Cek cepat identitas jendela aktif saat ini tanpa melakukan traversal tree mendalam
pub fn get_focused_window_quick_info() -> Option<(String, String)> {
    #[cfg(target_os = "linux")]
    {
        if let Some(wins) = get_sway_windows() {
            if let Some(w) = wins.into_iter().find(|w| w.focused) {
                return Some((format!("sway:{}", w.id), w.title));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW};
            let hwnd = GetForegroundWindow();
            if !hwnd.0.is_null() {
                let len = GetWindowTextLengthW(hwnd);
                let mut buf = vec![0u16; (len + 1) as usize];
                let copied = GetWindowTextW(hwnd, &mut buf);
                let title = String::from_utf16_lossy(&buf[..copied as usize]);
                return Some((format!("win:0x{:x}", hwnd.0 as usize), title));
            }
        }
    }

    None
}

/// Menjalankan watcher background thread yang memantau perpindahan window secara reaktif
pub fn start_window_watcher() {
    std::thread::Builder::new()
        .name("ragasukma-window-watcher".to_string())
        .spawn(|| {
            println!("🚀 [Desktop Worker] Dedicated ActiveWindowWatcher aktif untuk active_window_app.json");
            let mut last_window_id = String::new();
            let mut last_title = String::new();

            loop {
                let target_path = get_snapshot_path();
                let file_is_empty = fs::metadata(&target_path)
                    .map(|m| m.len() == 0)
                    .unwrap_or(true);

                // Quick focus check: jika jendela yang terfokus belum berubah dan file snapshot sudah terisi,
                // lewati scan AT-SPI/D-Bus sepenuhnya untuk menghemat thread dan CPU (0% load).
                if let Some((quick_id, quick_title)) = get_focused_window_quick_info() {
                    let changed = quick_id != last_window_id || quick_title != last_title;
                    if !changed && !file_is_empty {
                        std::thread::sleep(std::time::Duration::from_millis(800));
                        continue;
                    }
                }

                match scan_active_window_sync() {
                    Ok(snap) => {
                        let changed = snap.window_id != last_window_id || snap.window_title != last_title;
                        if file_is_empty || changed {
                            if changed && !last_title.is_empty() {
                                println!(
                                    "🖥️  [Desktop Worker] Jendela aktif berganti: \"{}\" ({})",
                                    snap.window_title, snap.app_name
                                );
                            }
                            last_window_id = snap.window_id.clone();
                            last_title = snap.window_title.clone();
                            let _ = snap.save_to_file();
                        }
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("⚠️  [Desktop Worker] Warning scan window: {e}");
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(800));
            }
        })
        .expect("Gagal spawn window watcher thread");
}

#[cfg(target_os = "windows")]
fn scan_active_window_windows() -> Result<ActiveWindowSnapshot, String> {
    use std::path::Path;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, TreeScope_Descendants,
        UIA_ButtonControlTypeId, UIA_CheckBoxControlTypeId, UIA_EditControlTypeId,
        UIA_HyperlinkControlTypeId, UIA_MenuItemControlTypeId, UIA_RadioButtonControlTypeId,
        UIA_CONTROLTYPE_ID,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible,
    };

    let mut snapshot = ActiveWindowSnapshot {
        timestamp: current_epoch_ms(),
        ..Default::default()
    };

    unsafe {
        let fg_hwnd = GetForegroundWindow();

        // 1. Dapatkan Foreground Window Info
        if !fg_hwnd.0.is_null() {
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(fg_hwnd, &mut title_buf);
            if len > 0 {
                snapshot.window_title = String::from_utf16_lossy(&title_buf[..len as usize]);
            }
            snapshot.window_id = format!("win:0x{:x}", fg_hwnd.0 as usize);

            let mut pid = 0u32;
            GetWindowThreadProcessId(fg_hwnd, Some(&mut pid));
            if pid != 0 {
                if let Ok(process_handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                    let mut path_buf = [0u16; 1024];
                    let mut path_len = path_buf.len() as u32;
                    if QueryFullProcessImageNameW(
                        process_handle,
                        PROCESS_NAME_FORMAT(0),
                        windows::core::PWSTR(path_buf.as_mut_ptr()),
                        &mut path_len,
                    ).is_ok() && path_len > 0 {
                        let full_path = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                        let exe_name = Path::new(&full_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&full_path);
                        snapshot.app_name = exe_name.to_string();
                    }
                }
            }
        }

        // 2. Enum semua jendela yang terlihat di Windows
        struct EnumContext {
            fg: HWND,
            windows: Vec<WindowItem>,
            counter: usize,
        }

        unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let ctx = &mut *(lparam.0 as *mut EnumContext);

                if IsWindowVisible(hwnd).as_bool() {
                    let len = GetWindowTextLengthW(hwnd);
                    if len > 0 {
                        let mut title_buf = vec![0u16; (len + 1) as usize];
                        let read_len = GetWindowTextW(hwnd, &mut title_buf);
                        if read_len > 0 {
                            let title = String::from_utf16_lossy(&title_buf[..read_len as usize]);

                            let mut pid = 0u32;
                            GetWindowThreadProcessId(hwnd, Some(&mut pid));
                            let mut app_name = String::new();
                            if pid != 0 {
                                if let Ok(proc) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                                    let mut path_buf = [0u16; 512];
                                    let mut path_len = path_buf.len() as u32;
                                    if QueryFullProcessImageNameW(
                                        proc,
                                        PROCESS_NAME_FORMAT(0),
                                        windows::core::PWSTR(path_buf.as_mut_ptr()),
                                        &mut path_len,
                                    ).is_ok() && path_len > 0 {
                                        let full = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                                        app_name = Path::new(&full)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                    }
                                }
                            }

                            let is_active = hwnd == ctx.fg;
                            ctx.windows.push(WindowItem {
                                id: ctx.counter,
                                app_name,
                                title,
                                active: is_active,
                                window_id: format!("win:0x{:x}", hwnd.0 as usize),
                            });
                            ctx.counter += 1;
                        }
                    }
                }

                BOOL(1)
            }
        }

        let mut enum_ctx = EnumContext {
            fg: fg_hwnd,
            windows: Vec::new(),
            counter: 1,
        };

        let _ = EnumWindows(Some(enum_windows_callback), LPARAM(&mut enum_ctx as *mut _ as isize));
        snapshot.windows = enum_ctx.windows;

        // 3. UI Automation untuk mengekstrak Buttons & Inputs dari Foreground Window
        if !fg_hwnd.0.is_null() {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            if let Ok(automation) = CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
                if let Ok(root_elem) = automation.ElementFromHandle(fg_hwnd) {
                    let mut element_id = 1;

                    // Buat TrueCondition untuk mencari descendants
                    if let Ok(condition) = automation.CreateTrueCondition() {
                        if let Ok(elements) = root_elem.FindAll(TreeScope_Descendants, &condition) {
                            if let Ok(count) = elements.Length() {
                                for i in 0..count {
                                    if let Ok(elem) = elements.GetElement(i) {
                                        let name: String = elem.CurrentName().map(|b| b.to_string()).unwrap_or_default().trim().to_string();
                                        let control_type = elem.CurrentControlType().unwrap_or(UIA_CONTROLTYPE_ID(0));

                                        if !name.is_empty() {
                                            let (center, bbox) = if let Ok(rect) = elem.CurrentBoundingRectangle() {
                                                let w = rect.right - rect.left;
                                                let h = rect.bottom - rect.top;
                                                if w > 0 && h > 0 {
                                                    let cx = rect.left + w / 2;
                                                    let cy = rect.top + h / 2;
                                                    (Some([cx, cy]), Some([rect.left, rect.top, w, h]))
                                                } else {
                                                    (None, None)
                                                }
                                            } else {
                                                (None, None)
                                            };

                                            #[allow(non_upper_case_globals)]
                                            match control_type {
                                                UIA_ButtonControlTypeId
                                                | UIA_CheckBoxControlTypeId
                                                | UIA_RadioButtonControlTypeId
                                                | UIA_MenuItemControlTypeId
                                                | UIA_HyperlinkControlTypeId => {
                                                    snapshot.buttons.push(DesktopElement {
                                                        id: element_id,
                                                        label: name,
                                                        role: "button".to_string(),
                                                        center,
                                                        bbox,
                                                    });
                                                    element_id += 1;
                                                }
                                                UIA_EditControlTypeId => {
                                                    snapshot.inputs.push(DesktopElement {
                                                        id: element_id,
                                                        label: name,
                                                        role: "input".to_string(),
                                                        center,
                                                        bbox,
                                                    });
                                                    element_id += 1;
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            CoUninitialize();
        }
    }

    if snapshot.app_name.is_empty() {
        snapshot.text_summary = "Tidak ada jendela aplikasi yang terfokus".to_string();
    }

    let _ = snapshot.save_to_file();
    Ok(snapshot)
}

#[cfg(target_os = "macos")]
async fn scan_active_window_macos() -> Result<ActiveWindowSnapshot, String> {
    let script = r#"
    set jsonOut to "{\"app\":\"\",\"title\":\"\",\"windows\":[],\"buttons\":[]}"
    try
        tell application "System Events"
            set frontProc to first process whose frontmost is true
            set procName to name of frontProc
            set winTitle to ""
            set winList to {}
            set btnList to {}

            -- Daftar seluruh proses ber-jendela (open windows)
            set allProcs to every process whose background only is false
            repeat with p in allProcs
                try
                    set pName to name of p
                    repeat with w in windows of p
                        set wTitle to name of w
                        if wTitle is not "" then
                            set end of winList to "{\"app\":\"" & pName & "\",\"title\":\"" & wTitle & "\",\"active\":" & ((frontmost of p) as string) & "}"
                        end if
                    end repeat
                end try
            end repeat

            -- Detail tombol jendela aktif
            if (count of windows of frontProc) > 0 then
                set targetWin to front window of frontProc
                set winTitle to name of targetWin
                set btnIdx to 1
                try
                    repeat with b in (every button of targetWin)
                        set bName to name of b
                        if bName is not "" then
                            set {bx, by} to position of b
                            set {bw, bh} to size of b
                            set cx to (bx + (bw / 2)) as integer
                            set cy to (by + (bh / 2)) as integer
                            set end of btnList to "{\"id\":" & btnIdx & ",\"label\":\"" & bName & "\",\"role\":\"button\",\"center\":[" & cx & "," & cy & "]}"
                            set btnIdx to btnIdx + 1
                        end if
                    end repeat
                end try
            end if

            -- Format output sederhana
            return procName & "|||" & winTitle & "|||[" & (my joinList(winList, ",")) & "]|||[" & (my joinList(btnList, ",")) & "]"
        end tell
    on error errMsg
        return "||||||[]|||[]"
    end try

    on joinList(lst, sep)
        set oldTID to AppleScript's text item delimiters
        set AppleScript's text item delimiters to sep
        set str to lst as string
        set AppleScript's text item delimiters to oldTID
        return str
    end joinList
    "#;

    let output = std::process::Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Gagal memanggil osascript: {e}"))?;

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = raw.split("|||").collect();

    let mut snapshot = ActiveWindowSnapshot {
        timestamp: current_epoch_ms(),
        source: default_source(),
        ..Default::default()
    };

    if parts.len() >= 4 {
        snapshot.app_name = parts[0].trim().to_string();
        snapshot.window_title = parts[1].trim().to_string();
        snapshot.window_id = format!("macos:{}", snapshot.app_name);

        if let Ok(wins) = serde_json::from_str::<Vec<serde_json::Value>>(parts[2]) {
            let mut counter = 1;
            for w in wins {
                let app = w.get("app").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let title = w.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let active = w.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                if !app.is_empty() {
                    snapshot.windows.push(WindowItem {
                        id: counter,
                        app_name: app.clone(),
                        title,
                        active,
                        window_id: format!("macos:{}", app),
                    });
                    counter += 1;
                }
            }
        }

        if let Ok(btns) = serde_json::from_str::<Vec<DesktopElement>>(parts[3]) {
            snapshot.buttons = btns;
        }
    }

    let _ = snapshot.save_to_file();
    Ok(snapshot)
}
