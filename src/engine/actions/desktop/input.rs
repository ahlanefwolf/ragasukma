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
            INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, SendInput,
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
        if std::process::Command::new("wlrctl")
            .args(["keyboard", "type", text])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return true;
        }

        // 2. Coba ydotool (Wayland uinput)
        if std::process::Command::new("ydotool")
            .args(["type", text])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
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
        let script = format!(
            "tell application \"System Events\" to keystroke \"{}\"",
            safe_text
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

pub fn press_key(key_name: &str) -> bool {
    let key = key_name.trim().to_lowercase();

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_BACK, VK_DOWN,
            VK_ESCAPE, VK_LEFT, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
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
                            dwFlags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(
                                0,
                            ),
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
        std::process::Command::new("xdotool")
            .args(["key", mapped])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
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

/// Reads current text content from the system clipboard across Linux, Windows, and macOS.
pub fn read_clipboard_text() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // 1. Wayland clipboard (wl-paste)
        if let Ok(output) = std::process::Command::new("wl-paste")
            .arg("--no-newline")
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        // 2. X11 clipboard via xclip
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        // 3. X11 clipboard via xsel
        if let Ok(output) = std::process::Command::new("xsel")
            .args(["--clipboard", "--output"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("powershell");
        crate::engine::skills::configure_subprocess(&mut cmd);
        cmd.args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", "Get-Clipboard"]);
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("pbpaste").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        None
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        None
    }
}
