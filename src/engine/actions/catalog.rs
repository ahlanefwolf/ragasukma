use crate::engine::skills::SkillItem;

/// Daftar aksi sistem bawaan yang otomatis aktif sesuai OS (Linux / Windows / macOS)
pub fn get_builtin_system_skills() -> Vec<SkillItem> {
    #[cfg(target_os = "windows")]
    {
        vec![
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume suara komputer".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]175)\""
                        .to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume suara komputer".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]174)\""
                        .to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara komputer (mute/unmute)".to_string(),
                command:
                    "powershell -c \"(New-Object -ComObject Wscript.Shell).SendKeys([char]173)\""
                        .to_string(),
            },
            SkillItem {
                name: "Kunci Layar".to_string(),
                description: "Mengunci layar komputer / lock workstation".to_string(),
                command: "rundll32.exe user32.dll,LockWorkStation".to_string(),
            },
            SkillItem {
                name: "Pindai Visual Layar".to_string(),
                description: "Membaca teks dan tombol/ikon di layar via ONNX OCR & Vision".to_string(),
                command: "desktop:scan_visual".to_string(),
            },
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume audio macOS (+10%)".to_string(),
                command: "osascript -e \"set volume output volume ((output volume of (get volume settings)) + 10)\"".to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume audio macOS (-10%)".to_string(),
                command: "osascript -e \"set volume output volume ((output volume of (get volume settings)) - 10)\"".to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara macOS (mute toggle)".to_string(),
                command: "osascript -e \"set volume output muted not (output muted of (get volume settings))\"".to_string(),
            },
            SkillItem {
                name: "Ambil Screenshot".to_string(),
                description: "Mengambil tangkapan layar monitor macOS".to_string(),
                command: "screencapture -i -c".to_string(),
            },
            SkillItem {
                name: "Kunci Layar".to_string(),
                description: "Mengunci sesi layar macOS".to_string(),
                command: "pmset displaysleepnow".to_string(),
            },
            SkillItem {
                name: "Pindai Visual Layar".to_string(),
                description: "Membaca teks dan tombol/ikon di layar via ONNX OCR & Vision".to_string(),
                command: "desktop:scan_visual".to_string(),
            },
        ]
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        vec![
            SkillItem {
                name: "Naikkan Volume".to_string(),
                description: "Menaikkan volume suara laptop/komputer (+10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ +10% || amixer set Master 10%+"
                    .to_string(),
            },
            SkillItem {
                name: "Turunkan Volume".to_string(),
                description: "Menurunkan volume suara laptop/komputer (-10%)".to_string(),
                command: "pactl set-sink-volume @DEFAULT_SINK@ -10% || amixer set Master 10%-"
                    .to_string(),
            },
            SkillItem {
                name: "Mute Audio".to_string(),
                description: "Mematikan atau menyalakan suara audio komputer".to_string(),
                command: "pactl set-sink-mute @DEFAULT_SINK@ toggle || amixer set Master toggle"
                    .to_string(),
            },
            SkillItem {
                name: "Ambil Screenshot".to_string(),
                description: "Mengambil tangkapan layar monitor komputer".to_string(),
                command: "gnome-screenshot || spectacle || flameshot gui || xfce4-screenshooter"
                    .to_string(),
            },
            SkillItem {
                name: "Kunci Layar".to_string(),
                description: "Mengunci sesi layar komputer".to_string(),
                command:
                    "loginctl lock-session || xdg-screensaver lock || gnome-screensaver-command -l"
                        .to_string(),
            },
            SkillItem {
                name: "Pindai Visual Layar".to_string(),
                description: "Membaca teks dan tombol/ikon di layar via ONNX OCR & Vision".to_string(),
                command: "desktop:scan_visual".to_string(),
            },
        ]
    }
}

/// Helper untuk membuka URL secara aman di dedicated browser profile pengguna
pub fn open_url_by_os(url: &str) -> std::io::Result<()> {
    crate::assets::open_browser(url);
    Ok(())
}
