use super::manager::broadcast_command;
use super::types::BrowserCommand;

pub struct YouTubeMusicEngine;

impl YouTubeMusicEngine {
    /// Buka tab YouTube Music
    pub fn open_main() -> std::io::Result<()> {
        let url = "https://music.youtube.com";
        crate::engine::open_url_by_os(url)
    }

    /// Cari dan putar lagu atau artis di YouTube Music
    pub fn search_or_play(query: &str) -> std::io::Result<()> {
        let encoded: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!("https://music.youtube.com/search?q={encoded}");
        crate::engine::open_url_by_os(&url)
    }

    /// Jeda lagu di YouTube Music
    pub fn pause() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["pause"]).spawn();
        }
        Self::send_action("pause")
    }

    /// Putar / lanjutkan lagu di YouTube Music
    pub fn play() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["play"]).spawn();
        }
        Self::send_action("play")
    }

    /// Toggle Play/Pause lagu
    pub fn toggle() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["play-pause"]).spawn();
        }
        Self::send_action("toggle")
    }

    /// Putar lagu berikutnya (Next Track)
    pub fn next() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["next"]).spawn();
        }
        Self::send_action("next")
    }

    /// Putar lagu sebelumnya (Previous Track)
    pub fn prev() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["previous"]).spawn();
        }
        Self::send_action("prev")
    }

    fn send_action(action: &str) -> bool {
        let cmd = BrowserCommand {
            action: action.to_string(),
            text: None,
            url: None,
        };
        broadcast_command(&cmd);
        true
    }
}
