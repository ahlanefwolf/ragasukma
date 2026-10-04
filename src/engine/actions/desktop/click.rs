// ==========================================
// MOUSE & COORDINATE ACTIONS
// ==========================================

pub fn click_at_coordinates(x: i32, y: i32) -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT, SendInput,
        };
        use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

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
        // 1. If in Sway Wayland: use native Sway IPC seat cursor (highest precision and integration)
        let is_sway = std::env::var("SWAYSOCK").is_ok()
            || std::env::var("XDG_CURRENT_DESKTOP")
                .map(|d| d.eq_ignore_ascii_case("sway"))
                .unwrap_or(false);

        if is_sway {
            let cmd_press = format!("seat * cursor set {} {}, seat * cursor press button1", x, y);
            if std::process::Command::new("swaymsg")
                .arg(&cmd_press)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                std::thread::sleep(std::time::Duration::from_millis(35));
                let _ = std::process::Command::new("swaymsg")
                    .args(["seat", "*", "cursor", "release", "button1"])
                    .status();
                return true;
            }
        }

        // 2. If in Hyprland: set cursor via hyprctl movecursor
        let is_hyprland = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
            || std::env::var("XDG_CURRENT_DESKTOP")
                .map(|d| d.eq_ignore_ascii_case("hyprland"))
                .unwrap_or(false);

        if is_hyprland {
            let _ = std::process::Command::new("hyprctl")
                .args(["dispatch", "movecursor", &x.to_string(), &y.to_string()])
                .status();
            std::thread::sleep(std::time::Duration::from_millis(30));
        }

        // 3. Fallback: ydotool (Wayland uinput daemon - bekerja in GNOME, KDE, Hyprland, Sway)
        let move_ok = std::process::Command::new("ydotool")
            .args(["mousemove", "-a", &x.to_string(), &y.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if move_ok {
            std::thread::sleep(std::time::Duration::from_millis(30));
            let _ = std::process::Command::new("ydotool")
                .args(["click", "0xC0"])
                .status();
            return true;
        }

        // 4. Fallback: wlrctl (standar Wayland wlroots / Hyprland)
        let wlr_click = std::process::Command::new("wlrctl")
            .args(["pointer", "click", "left"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if wlr_click {
            return true;
        }

        // 5. Fallback: xdotool (Standar X11 & XWayland)
        let status = std::process::Command::new("xdotool")
            .args(["mousemove", &x.to_string(), &y.to_string(), "click", "1"])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        crate::term_err!("[Desktop Click] Failed to simulate mouse click on Linux.");
        false
    }

    #[cfg(target_os = "macos")]
    {
        // 1. Try popular cliclick utility if available
        let target_arg = format!("c:{},{}", x, y);
        let status = std::process::Command::new("cliclick")
            .arg(&target_arg)
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return true;
        }

        // 2. Fallback: AppleScript System Events native (without perlu install binary eksternal)
        let script = format!(
            "tell application \"System Events\" to click at {{{}, {}}}",
            x, y
        );
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

pub fn find_target_button<'a>(
    needle: &str,
    buttons: &'a [crate::engine::window_scanner::DesktopElement],
) -> Option<&'a crate::engine::window_scanner::DesktopElement> {
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }
    let needle_lower = needle.to_lowercase();

    // Priority 1: Exact Button Number ID Match
    let by_id = buttons.iter().find(|b| b.id.to_string() == needle);

    // Priority 2: Exact Button Label Match (case-insensitive)
    let by_exact = buttons
        .iter()
        .find(|b| b.label.trim().eq_ignore_ascii_case(needle));

    // Priority 3: Starts-with match (for example "Save" matches "Save Changes")
    let by_starts = buttons
        .iter()
        .find(|b| b.label.trim().to_lowercase().starts_with(&needle_lower));

    // Priority 4: Substring match ONLY if search length is greater than 1 character
    let by_contains = if needle.len() > 1 {
        buttons
            .iter()
            .find(|b| b.label.trim().to_lowercase().contains(&needle_lower))
    } else {
        None
    };

    // Priority 5: Fuzzy Similarity Match (Typo tolerance / phonetic variation)
    let by_fuzzy = if by_id.is_none()
        && by_exact.is_none()
        && by_starts.is_none()
        && by_contains.is_none()
        && needle.len() >= 3
    {
        buttons
            .iter()
            .filter_map(|b| {
                let sim = crate::engine::perception::string_similarity(needle, &b.label);
                if sim >= 0.70 { Some((b, sim)) } else { None }
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(b, sim)| {
                crate::term_log!(
                    "[Fuzzy Match UI] Found similar element '{}' -> '{}' (score: {:.1}%)",
                    needle,
                    b.label,
                    sim * 100.0
                );
                b
            })
    } else {
        None
    };

    by_id
        .or(by_exact)
        .or(by_starts)
        .or(by_contains)
        .or(by_fuzzy)
}

pub fn click_desktop_button(label_or_id: &str) -> bool {
    let needle = label_or_id.trim();
    if needle.is_empty() {
        return false;
    }

    // 1. Try searching existing window scanner snapshot (OS Accessibility Tree)
    if let Some(snap) = super::load_snapshot() {
        if let Some(btn) = find_target_button(needle, &snap.buttons) {
            if let Some(center) = btn.center {
                if !snap.app_name.is_empty() {
                    let _ = super::window::focus_window(&snap.app_name);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                crate::term_log!(
                    "[Desktop Click] Found via accessibility tree: '{}' at [{}, {}]",
                    btn.label,
                    center[0],
                    center[1]
                );
                return click_at_coordinates(center[0], center[1]);
            }
        }
    }

    // 2. Multiplatform Fallback: Run Visual OCR Perception Scan aktual
    // Useful if accessibility tree has no coordinates (e.g. Linux AT-SPI without extents),
    // canvas kustom, game, aplikasi electron without bridge accessibility, dll.)
    crate::term_log!(
        "[Desktop Click] Element '{}' not found with coordinates in accessibility tree. Running visual OCR scan...",
        needle
    );

    let perception_res = crate::engine::perception::request_vision_scan_sync();

    // Map VisualElement to DesktopElement for consistent search
    let ocr_elements: Vec<crate::engine::window_scanner::DesktopElement> = perception_res
        .elements
        .iter()
        .map(|el| crate::engine::window_scanner::DesktopElement {
            id: el.id,
            label: el.label.clone(),
            role: el.role.clone(),
            center: Some(el.center),
            bbox: Some(el.bbox),
        })
        .collect();

    if let Some(found) = find_target_button(needle, &ocr_elements) {
        if let Some(center) = found.center {
            crate::term_log!(
                "[Desktop Click] Found via visual OCR: '{}' at [{}, {}]",
                found.label,
                center[0],
                center[1]
            );
            return click_at_coordinates(center[0], center[1]);
        }
    }

    crate::term_err!(
        "[Desktop Click] Element or text '{}' not found on screen (checked accessibility + visual OCR).",
        needle
    );
    false
}
