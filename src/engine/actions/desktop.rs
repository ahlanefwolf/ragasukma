use std::fs;
use crate::engine::window_scanner::ActiveWindowSnapshot;

pub fn execute_desktop_action(action: &str, payload: &str) -> Option<(String, String)> {
    match action {
        "scan" => {
            match crate::engine::window_scanner::scan_active_window_sync() {
                Ok(snap) => {
                    let _ = snap.save_to_file();
                    Some((
                        format!("Pindai Jendela {}", snap.app_name),
                        format!(
                            "Terdeteksi aplikasi {} ({}) dengan {} tombol",
                            snap.app_name,
                            snap.window_title,
                            snap.buttons.len()
                        ),
                    ))
                }
                Err(e) => Some(("Pindai Jendela".to_string(), format!("Gagal memindai jendela: {e}"))),
            }
        }
        "scan_visual" | "ocr" => {
            let res = crate::engine::perception::request_vision_scan_sync();
            Some((
                "Pindai Visual Layar".to_string(),
                format!(
                    "Berhasil memindai layar: {} tombol/ikon dan {} teks terdeteksi.",
                    res.buttons.len(),
                    res.texts.len()
                ),
            ))
        }
        "focus" => {
            let target = payload.trim();
            if focus_window(target) {
                Some(("Fokus Jendela".to_string(), format!("Jendela '{target}' difokuskan")))
            } else {
                Some(("Fokus Jendela".to_string(), format!("Gagal menemukan atau memfokuskan jendela '{target}'")))
            }
        }
        "minimize" => {
            let target = payload.trim();
            if minimize_window(target) {
                Some(("Minimize Jendela".to_string(), "Jendela berhasil diminimalkan".to_string()))
            } else {
                Some(("Minimize Jendela".to_string(), "Gagal me-minimize jendela".to_string()))
            }
        }
        "maximize" => {
            let target = payload.trim();
            if maximize_window(target) {
                Some(("Maksimalkan Jendela".to_string(), "Ukuran jendela dimaksimalkan/restore".to_string()))
            } else {
                Some(("Maksimalkan Jendela".to_string(), "Gagal memaksimalkan jendela".to_string()))
            }
        }
        "close" => {
            let target = payload.trim();
            if close_window(target) {
                Some(("Tutup Jendela".to_string(), format!("Jendela '{target}' ditutup")))
            } else {
                Some(("Tutup Jendela".to_string(), format!("Gagal menutup jendela '{target}'")))
            }
        }
        "click_button" | "click_text" => {
            let target = payload.trim();
            if click_desktop_button(target) {
                Some(("Klik Elemen Desktop".to_string(), format!("Elemen/teks '{target}' berhasil diklik")))
            } else {
                Some(("Klik Elemen Desktop".to_string(), format!("Elemen/teks '{target}' tidak ditemukan di layar")))
            }
        }
        "click_at" => {
            let parts: Vec<&str> = payload.split('|').collect();
            if parts.len() >= 2 {
                let x = parts[0].trim().parse::<i32>().unwrap_or(0);
                let y = parts[1].trim().parse::<i32>().unwrap_or(0);
                if click_at_coordinates(x, y) {
                    Some(("Klik Koordinat".to_string(), format!("Klik di titik ({x}, {y}) berhasil")))
                } else {
                    Some(("Klik Koordinat".to_string(), format!("Gagal mengklik di titik ({x}, {y})")))
                }
            } else {
                Some(("Klik Koordinat".to_string(), "Format salah, gunakan desktop:click_at|x|y".to_string()))
            }
        }
        "type" => {
            let text = payload;
            if type_text(text) {
                Some(("Ketik Teks".to_string(), format!("Mengetik teks: {text}")))
            } else {
                Some(("Ketik Teks".to_string(), "Gagal mengetikkan teks".to_string()))
            }
        }
        "key" => {
            let key_name = payload.trim();
            if press_key(key_name) {
                Some(("Tekan Tombol".to_string(), format!("Tombol '{key_name}' ditekan")))
            } else {
                Some(("Tekan Tombol".to_string(), format!("Tombol '{key_name}' tidak didukung")))
            }
        }
        _ => None,
    }
}

fn load_snapshot() -> Option<ActiveWindowSnapshot> {
    let path = crate::assets::find_file_in_candidates("active_window_app.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(snap) = serde_json::from_str::<ActiveWindowSnapshot>(&content) {
            return Some(snap);
        }
    }
    None
}

// ==========================================
// WINDOW CONTROL (CROSS-PLATFORM)
// ==========================================

pub fn focus_window(target: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, IsIconic,
            SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
        };
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            keybd_event, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_MENU,
        };
        use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};

        if let Some(hwnd) = resolve_hwnd(target) {
            unsafe {
                let cur_fg = GetForegroundWindow();
                let cur_thread = GetWindowThreadProcessId(cur_fg, None);
                let target_thread = GetWindowThreadProcessId(hwnd, None);
                let my_thread = GetCurrentThreadId();

                if cur_thread != target_thread {
                    let _ = AttachThreadInput(cur_thread, target_thread, true);
                    let _ = AttachThreadInput(my_thread, target_thread, true);
                }

                // Trik Bypass Windows 10/11 Foreground Lock:
                // Menekan tombol ALT memberitahu kernel Windows bahwa pengguna sedang berinteraksi,
                // sehingga SetForegroundWindow tidak di-block / tidak hanya berkedip oranye di taskbar.
                keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_EXTENDEDKEY, 0);

                if IsIconic(hwnd).as_bool() {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                } else {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                }

                let _ = BringWindowToTop(hwnd);
                let ok = SetForegroundWindow(hwnd).as_bool();

                // Lepas tombol ALT
                keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);

                if cur_thread != target_thread {
                    let _ = AttachThreadInput(cur_thread, target_thread, false);
                    let _ = AttachThreadInput(my_thread, target_thread, false);
                }

                return ok;
            }
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        if target.is_empty() {
            return false;
        }

        let sway_target = target.trim();
        // 1. Coba Sway IPC (swaymsg) jika berada di lingkungan Sway Wayland
        if let Some(id_str) = sway_target.strip_prefix("sway:") {
            if std::process::Command::new("swaymsg")
                .args([&format!("[con_id=\"{}\"] focus", id_str)])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return true;
            }
        }

        let sway_crit = format!("[app_id=\"(?i).*{sway_target}.*\"] focus");
        if std::process::Command::new("swaymsg")
            .arg(&sway_crit)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        let sway_title_crit = format!("[title=\"(?i).*{sway_target}.*\"] focus");
        if std::process::Command::new("swaymsg")
            .arg(&sway_title_crit)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        let sway_class_crit = format!("[class=\"(?i).*{sway_target}.*\"] focus");
        if std::process::Command::new("swaymsg")
            .arg(&sway_class_crit)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        // 2. Coba Hyprland IPC (hyprctl) jika berada di lingkungan Hyprland
        if std::process::Command::new("hyprctl")
            .args(["dispatch", "focuswindow", sway_target])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }
        if std::process::Command::new("hyprctl")
            .args(["dispatch", "focuswindow", &format!("class:{}", sway_target)])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }
        if std::process::Command::new("hyprctl")
            .args(["dispatch", "focuswindow", &format!("title:{}", sway_target)])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        // 3. Coba wmctrl (X11 / XWayland)
        if std::process::Command::new("wmctrl").args(["-a", target]).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // 4. Coba xdotool (X11 / XWayland)
        if std::process::Command::new("xdotool").args(["search", "--name", target, "windowactivate"]).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        false
    }

    #[cfg(target_os = "macos")]
    {
        if target.is_empty() {
            return false;
        }
        let script = format!("tell application \"{}\" to activate", target);
        std::process::Command::new("osascript").args(["-e", &script]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

pub fn minimize_window(target: &str) -> bool {
    let _ = target;
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_MINIMIZE, GetForegroundWindow};
        let hwnd = if target.is_empty() {
            unsafe { GetForegroundWindow() }
        } else {
            resolve_hwnd(target).unwrap_or_else(|| unsafe { GetForegroundWindow() })
        };

        if !hwnd.0.is_null() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_MINIMIZE);
                return true;
            }
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdotool").args(["getactivewindow", "windowminimize"]).status();
        true
    }

    #[cfg(target_os = "macos")]
    {
        let script = "tell application \"System Events\" to set miniaturized of window 1 of (first process whose frontmost is true) to true";
        std::process::Command::new("osascript").args(["-e", script]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

pub fn maximize_window(target: &str) -> bool {
    let _ = target;
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_MAXIMIZE, GetForegroundWindow};
        let hwnd = if target.is_empty() {
            unsafe { GetForegroundWindow() }
        } else {
            resolve_hwnd(target).unwrap_or_else(|| unsafe { GetForegroundWindow() })
        };

        if !hwnd.0.is_null() {
            unsafe {
                let _ = ShowWindow(hwnd, SW_MAXIMIZE);
                return true;
            }
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Coba Sway fullscreen toggle (Sway wayland)
        if std::process::Command::new("swaymsg").args(["fullscreen", "toggle"]).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // 2. Coba Hyprland fullscreen toggle
        if std::process::Command::new("hyprctl").args(["dispatch", "fullscreen", "1"]).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // 3. Coba wmctrl (X11)
        let _ = std::process::Command::new("wmctrl").args(["-r", ":ACTIVE:", "-b", "toggle,maximized_vert,maximized_horz"]).status();
        true
    }

    #[cfg(target_os = "macos")]
    {
        let script = "tell application \"System Events\" to set zoomed of window 1 of (first process whose frontmost is true) to true";
        std::process::Command::new("osascript").args(["-e", script]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

pub fn close_window(target: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::{WPARAM, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE, GetForegroundWindow};
        let hwnd = if target.is_empty() {
            unsafe { GetForegroundWindow() }
        } else {
            resolve_hwnd(target).unwrap_or_else(|| unsafe { GetForegroundWindow() })
        };

        if !hwnd.0.is_null() {
            unsafe {
                let _ = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
                return true;
            }
        }
        false
    }

    #[cfg(target_os = "linux")]
    {
        if !target.is_empty() {
            let sway_target = target.trim();
            if let Some(id_str) = sway_target.strip_prefix("sway:") {
                let _ = std::process::Command::new("swaymsg")
                    .args([&format!("[con_id=\"{}\"] kill", id_str)])
                    .status();
            } else {
                let _ = std::process::Command::new("swaymsg")
                    .args([&format!("[app_id=\"(?i).*{sway_target}.*\"] kill")])
                    .status();
            }
            // Hyprland close window by class/regex
            let _ = std::process::Command::new("hyprctl").args(["dispatch", "closewindow", sway_target]).status();
            let _ = std::process::Command::new("wmctrl").args(["-c", target]).status();
        } else {
            let _ = std::process::Command::new("swaymsg").arg("kill").status();
            let _ = std::process::Command::new("hyprctl").args(["dispatch", "killactive"]).status();
            let _ = std::process::Command::new("xdotool").args(["getactivewindow", "windowclose"]).status();
        }
        true
    }

    #[cfg(target_os = "macos")]
    {
        let script = "tell application \"System Events\" to click button 1 of window 1 of (first process whose frontmost is true)";
        std::process::Command::new("osascript").args(["-e", script]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
fn resolve_hwnd(target: &str) -> Option<windows::Win32::Foundation::HWND> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    };

    let target_clean = target.trim();
    if target_clean.is_empty() {
        return None;
    }

    // 1. Cek format hex window_id (misal "win:0x30354" atau "0x30354")
    let hex_part = target_clean.strip_prefix("win:").unwrap_or(target_clean);
    if let Some(stripped) = hex_part.strip_prefix("0x") {
        if let Ok(val) = usize::from_str_radix(stripped, 16) {
            return Some(HWND(val as *mut std::ffi::c_void));
        }
    }

    let needle = target_clean.to_lowercase();
    let query_words: Vec<&str> = needle.split_whitespace().filter(|w| w.len() >= 2).collect();

    // 2. Cek daftar jendela di snapshot JSON aktif (Ground of Truth)
    if let Some(snap) = load_snapshot() {
        use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
        let mut fallback_hwnd = None;

        for w in &snap.windows {
            let app_lower = w.app_name.to_lowercase();
            let app_base = app_lower.strip_suffix(".exe").unwrap_or(&app_lower);
            let title_lower = w.title.to_lowercase();

            // Pencocokan dinamis murni:
            // 1. Apakah nama app atau title mengandung query?
            // 2. Apakah query mengandung stem nama app (misal query "zed editor" mengandung "zed")?
            // 3. Apakah ada kata dalam query yang cocok dengan nama app atau title?
            let is_match = app_lower.contains(&needle)
                || title_lower.contains(&needle)
                || (!app_base.is_empty() && needle.contains(app_base))
                || query_words.iter().any(|&word| app_lower.contains(word) || title_lower.contains(word));

            if is_match {
                let h_part = w.window_id.strip_prefix("win:").unwrap_or(&w.window_id);
                if let Some(s) = h_part.strip_prefix("0x") {
                    if let Ok(val) = usize::from_str_radix(s, 16) {
                        let candidate = HWND(val as *mut std::ffi::c_void);
                        unsafe {
                            if IsWindowVisible(candidate).as_bool() {
                                return Some(candidate);
                            }
                        }
                        if fallback_hwnd.is_none() {
                            fallback_hwnd = Some(candidate);
                        }
                    }
                }
            }
        }

        if let Some(h) = fallback_hwnd {
            return Some(h);
        }
    }

    // 3. Fallback: Live EnumWindows langsung ke OS
    struct SearchCtx {
        query: String,
        words: Vec<String>,
        found: Option<HWND>,
    }

    unsafe extern "system" fn search_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = unsafe { &mut *(lparam.0 as *mut SearchCtx) };
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return BOOL(1);
            }
            let len = GetWindowTextLengthW(hwnd);
            if len > 0 {
                let mut buf = vec![0u16; (len + 1) as usize];
                let read_len = GetWindowTextW(hwnd, &mut buf);
                if read_len > 0 {
                    let title = String::from_utf16_lossy(&buf[..read_len as usize]).to_lowercase();
                    if title.contains(&ctx.query) || ctx.words.iter().any(|w| title.contains(w.as_str())) {
                        ctx.found = Some(hwnd);
                        return BOOL(0); // Berhenti mencari
                    }
                }
            }
        }
        BOOL(1)
    }

    let mut ctx = SearchCtx {
        query: needle.clone(),
        words: query_words.into_iter().map(|s| s.to_string()).collect(),
        found: None,
    };
    let _ = unsafe { EnumWindows(Some(search_cb), LPARAM(&mut ctx as *mut _ as isize)) };

    ctx.found
}

// ==========================================
// MOUSE & COORDINATE ACTIONS
// ==========================================

pub fn click_at_coordinates(x: i32, y: i32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_MOUSE, MOUSEINPUT, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        };

        unsafe {
            if SetCursorPos(x, y).is_err() {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(30));

            let inputs = [
                INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTDOWN,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];

            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            true
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Jika di Sway Wayland: set koordinat cursor via swaymsg seat
        let _ = std::process::Command::new("swaymsg")
            .args(["seat", "*", "cursor", "set", &x.to_string(), &y.to_string()])
            .status();

        // 2. Jika di Hyprland: set cursor via hyprctl movecursor
        let _ = std::process::Command::new("hyprctl")
            .args(["dispatch", "movecursor", &x.to_string(), &y.to_string()])
            .status();

        // 3. Eksekusi klik via wlrctl (standar Wayland wlroots / Sway / Hyprland)
        let wlr_click = std::process::Command::new("wlrctl")
            .args(["pointer", "click", "left"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if wlr_click {
            return true;
        }

        // 4. Fallback klik via swaymsg seat cursor press/release
        let sway_press = std::process::Command::new("swaymsg")
            .args(["seat", "*", "cursor", "press", "button1"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if sway_press {
            let _ = std::process::Command::new("swaymsg")
                .args(["seat", "*", "cursor", "release", "button1"])
                .status();
            return true;
        }

        // 5. Fallback: ydotool (Wayland uinput daemon - bekerja di GNOME, KDE, Hyprland, Sway)
        let move_ok = std::process::Command::new("ydotool")
            .args(["mousemove", "-a", &x.to_string(), &y.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if move_ok {
            let _ = std::process::Command::new("ydotool").args(["click", "0xC0"]).status();
            return true;
        }

        // 6. Fallback: xdotool (Standar X11 & XWayland)
        let status = std::process::Command::new("xdotool")
            .args(["mousemove", &x.to_string(), &y.to_string(), "click", "1"])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        crate::term_err!("❌ [Desktop Click] Gagal simulasi klik mouse di Linux.");
        false
    }

    #[cfg(target_os = "macos")]
    {
        // 1. Coba utilitas populer cliclick jika tersedia
        let target_arg = format!("c:{},{}", x, y);
        let status = std::process::Command::new("cliclick").arg(&target_arg).status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // 2. Fallback: AppleScript System Events native (tanpa perlu install binary eksternal)
        let script = format!(
            "tell application \"System Events\" to click at {{{}, {}}}",
            x, y
        );
        std::process::Command::new("osascript").args(["-e", &script]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

pub fn click_desktop_button(label_or_id: &str) -> bool {
    let needle = label_or_id.trim();
    if needle.is_empty() {
        return false;
    }

    let needle_lower = needle.to_lowercase();

    if let Some(snap) = load_snapshot() {
        // Prioritas 1: ID Nomor Tombol Sama Persis
        let by_id = snap.buttons.iter().find(|b| b.id.to_string() == needle);

        // Prioritas 2: Label Tombol Sama Persis (Exact Match case-insensitive)
        let by_exact = snap
            .buttons
            .iter()
            .find(|b| b.label.trim().eq_ignore_ascii_case(needle));

        // Prioritas 3: Starts-with match (misal "Simpan" cocok dengan "Simpan Perubahan")
        let by_starts = snap
            .buttons
            .iter()
            .find(|b| b.label.trim().to_lowercase().starts_with(&needle_lower));

        // Prioritas 4: Substring match HANYA jika panjang pencarian lebih dari 1 karakter
        let by_contains = if needle.len() > 1 {
            snap.buttons
                .iter()
                .find(|b| b.label.trim().to_lowercase().contains(&needle_lower))
        } else {
            None
        };

        let target_btn = by_id.or(by_exact).or(by_starts).or(by_contains);

        if let Some(btn) = target_btn {
            if let Some(center) = btn.center {
                // Auto-focus jendela pemilik jika perlu, agar klik diterima oleh aplikasi target
                if !snap.app_name.is_empty() {
                    let _ = focus_window(&snap.app_name);
                }
                return click_at_coordinates(center[0], center[1]);
            }
        }
    }

    false
}

// ==========================================
// KEYBOARD & INPUT ACTIONS
// ==========================================

pub fn type_text(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        };

        let mut inputs = Vec::new();
        for c in text.encode_utf16() {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                        wScan: c,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(0),
                        wScan: c,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            });
        }

        unsafe {
            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            true
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Coba wlrctl (Wayland wlroots / Sway)
        if std::process::Command::new("wlrctl").args(["keyboard", "type", text]).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // 2. Coba ydotool (Wayland uinput)
        if std::process::Command::new("ydotool").args(["type", text]).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // 3. Coba xdotool (X11)
        let status = std::process::Command::new("xdotool")
            .args(["type", "--delay", "20", text])
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(target_os = "macos")]
    {
        let safe_text = text.replace('"', "\\\"");
        let script = format!("tell application \"System Events\" to keystroke \"{}\"", safe_text);
        std::process::Command::new("osascript").args(["-e", &script]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

pub fn press_key(key_name: &str) -> bool {
    let key = key_name.trim().to_lowercase();

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
            VK_RETURN, VK_ESCAPE, VK_TAB, VK_BACK, VK_SPACE, VK_UP, VK_DOWN, VK_LEFT, VK_RIGHT,
        };

        let vk = match key.as_str() {
            "enter" | "return" => VK_RETURN,
            "esc" | "escape" => VK_ESCAPE,
            "tab" => VK_TAB,
            "backspace" | "back" => VK_BACK,
            "space" => VK_SPACE,
            "up" => VK_UP,
            "down" => VK_DOWN,
            "left" => VK_LEFT,
            "right" => VK_RIGHT,
            _ => return false,
        };

        unsafe {
            let inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            wScan: 0,
                            dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];

            SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            true
        }
    }

    #[cfg(target_os = "linux")]
    {
        let mapped = match key.as_str() {
            "enter" | "return" => "Return",
            "esc" | "escape" => "Escape",
            "tab" => "Tab",
            "backspace" | "back" => "BackSpace",
            "space" => "space",
            "up" => "Up",
            "down" => "Down",
            "left" => "Left",
            "right" => "Right",
            _ => return false,
        };
        std::process::Command::new("xdotool").args(["key", mapped]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(target_os = "macos")]
    {
        let code = match key.as_str() {
            "enter" | "return" => 36,
            "esc" | "escape" => 53,
            "tab" => 48,
            "backspace" | "back" => 51,
            "space" => 49,
            "up" => 126,
            "down" => 125,
            "left" => 123,
            "right" => 124,
            _ => return false,
        };
        let script = format!("tell application \"System Events\" to key code {}", code);
        std::process::Command::new("osascript").args(["-e", &script]).status().map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}
