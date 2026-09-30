// ==========================================
// WINDOW CONTROL (CROSS-PLATFORM)
// ==========================================

pub fn focus_window(target: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_MENU, keybd_event,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, SW_RESTORE,
            SW_SHOW, SetForegroundWindow, ShowWindow,
        };

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
                keybd_event(
                    VK_MENU.0 as u8,
                    0,
                    KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                    0,
                );

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
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
            && std::process::Command::new("hyprctl")
                .args(["dispatch", "focuswindow", sway_target])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        {
            return true;
        }

        // 3. Coba wmctrl (X11 / XWayland)
        if std::process::Command::new("wmctrl")
            .args(["-a", target])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        // 4. Coba xdotool (X11 / XWayland)
        if std::process::Command::new("xdotool")
            .args(["search", "--name", target, "windowactivate"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
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
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
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
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, SW_MINIMIZE, ShowWindow,
        };
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
        let _ = std::process::Command::new("xdotool")
            .args(["getactivewindow", "windowminimize"])
            .status();
        true
    }

    #[cfg(target_os = "macos")]
    {
        let script = "tell application \"System Events\" to set miniaturized of window 1 of (first process whose frontmost is true) to true";
        std::process::Command::new("osascript")
            .args(["-e", script])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
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
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, SW_MAXIMIZE, ShowWindow,
        };
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
        if std::process::Command::new("swaymsg")
            .args(["fullscreen", "toggle"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        // 2. Coba Hyprland fullscreen toggle
        if std::process::Command::new("hyprctl")
            .args(["dispatch", "fullscreen", "1"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        // 3. Coba wmctrl (X11)
        let _ = std::process::Command::new("wmctrl")
            .args([
                "-r",
                ":ACTIVE:",
                "-b",
                "toggle,maximized_vert,maximized_horz",
            ])
            .status();
        true
    }

    #[cfg(target_os = "macos")]
    {
        let script = "tell application \"System Events\" to set zoomed of window 1 of (first process whose frontmost is true) to true";
        std::process::Command::new("osascript")
            .args(["-e", script])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

pub fn close_window(target: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, PostMessageW, WM_CLOSE,
        };
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
            let _ = std::process::Command::new("hyprctl")
                .args(["dispatch", "closewindow", sway_target])
                .status();
            let _ = std::process::Command::new("wmctrl")
                .args(["-c", target])
                .status();
        } else {
            let _ = std::process::Command::new("swaymsg").arg("kill").status();
            let _ = std::process::Command::new("hyprctl")
                .args(["dispatch", "killactive"])
                .status();
            let _ = std::process::Command::new("xdotool")
                .args(["getactivewindow", "windowclose"])
                .status();
        }
        true
    }

    #[cfg(target_os = "macos")]
    {
        let script = "tell application \"System Events\" to click button 1 of window 1 of (first process whose frontmost is true)";
        std::process::Command::new("osascript")
            .args(["-e", script])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
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
    if let Some(snap) = super::load_snapshot() {
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
                || query_words
                    .iter()
                    .any(|&word| app_lower.contains(word) || title_lower.contains(word));

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
                    if title.contains(&ctx.query)
                        || ctx.words.iter().any(|w| title.contains(w.as_str()))
                    {
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
