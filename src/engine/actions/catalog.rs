use crate::engine::skills::SkillItem;

/// Built-in system actions, auto-activated per OS (Linux / Windows / macOS)
pub fn get_builtin_system_skills() -> Vec<SkillItem> {
    #[cfg(target_os = "windows")]
    {
        vec![
            SkillItem {
                name: "Volume Up".to_string(),
                description: "Increase computer audio volume".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]175)\""
                        .to_string(),
            },
            SkillItem {
                name: "Volume Down".to_string(),
                description: "Decrease computer audio volume".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]174)\""
                        .to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Toggle mute/unmute computer audio".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]173)\""
                        .to_string(),
            },
            SkillItem {
                name: "Lock Screen".to_string(),
                description: "Lock the computer screen / workstation".to_string(),
                command: "rundll32.exe user32.dll,LockWorkStation".to_string(),
            },
            SkillItem {
                name: "Scan Screen".to_string(),
                description: "Read text and buttons/icons on screen via ONNX OCR & Vision"
                    .to_string(),
                command: "desktop:scan_visual".to_string(),
            },
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            SkillItem {
                name: "Volume Up".to_string(),
                description: "Increase macOS audio volume (+10%)".to_string(),
                command: "osascript -e \"set volume output volume ((output volume of (get volume settings)) + 10)\"".to_string(),
            },
            SkillItem {
                name: "Volume Down".to_string(),
                description: "Decrease macOS audio volume (-10%)".to_string(),
                command: "osascript -e \"set volume output volume ((output volume of (get volume settings)) - 10)\"".to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Toggle mute/unmute macOS audio".to_string(),
                command: "osascript -e \"set volume output muted not (output muted of (get volume settings))\"".to_string(),
            },
            SkillItem {
                name: "Screenshot".to_string(),
                description: "Capture a screenshot on macOS".to_string(),
                command: "screencapture -i -c".to_string(),
            },
            SkillItem {
                name: "Lock Screen".to_string(),
                description: "Lock the macOS screen session".to_string(),
                command: "pmset displaysleepnow".to_string(),
            },
            SkillItem {
                name: "Scan Screen".to_string(),
                description: "Read text and buttons/icons on screen via ONNX OCR & Vision".to_string(),
                command: "desktop:scan_visual".to_string(),
            },
        ]
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        vec![
            SkillItem {
                name: "Volume Up".to_string(),
                description: "Increase system audio volume (+10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ +10% || amixer set Master 10%+"
                    .to_string(),
            },
            SkillItem {
                name: "Volume Down".to_string(),
                description: "Decrease system audio volume (-10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ -10% || amixer set Master 10%-"
                    .to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Toggle mute/unmute system audio".to_string(),
                command: "pactl set-sink-mute @DEFAULT_SINK@ toggle || amixer set Master toggle"
                    .to_string(),
            },
            SkillItem {
                name: "Screenshot".to_string(),
                description: "Capture a screenshot of the monitor".to_string(),
                command: "gnome-screenshot || spectacle || flameshot gui || xfce4-screenshooter"
                    .to_string(),
            },
            SkillItem {
                name: "Lock Screen".to_string(),
                description: "Lock the computer screen session".to_string(),
                command:
                    "loginctl lock-session || xdg-screensaver lock || gnome-screensaver-command -l"
                        .to_string(),
            },
            SkillItem {
                name: "Scan Screen".to_string(),
                description: "Read text and buttons/icons on screen via ONNX OCR & Vision"
                    .to_string(),
                command: "desktop:scan_visual".to_string(),
            },
        ]
    }
}

/// Helper to open URL safely in dedicated user browser profile
pub fn open_url_by_os(url: &str) -> std::io::Result<()> {
    crate::assets::open_browser(url);
    Ok(())
}
