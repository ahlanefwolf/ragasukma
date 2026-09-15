use super::manager::broadcast_command;
use super::types::BrowserCommand;

pub struct YouTubeEngine;

impl YouTubeEngine {
    /// Buka beranda YouTube utama
    pub fn open_main() -> std::io::Result<()> {
        let url = "https://www.youtube.com";
        crate::engine::open_url_by_os(url)
    }

    /// Cari video di YouTube
    pub fn search_or_play(query: &str) -> std::io::Result<()> {
        let encoded: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!("https://www.youtube.com/results?search_query={encoded}");
        crate::engine::open_url_by_os(&url)
    }

    /// Jeda pemutaran video YouTube
    pub fn pause() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["pause"]).spawn();
        }
        Self::send_action("pause")
    }

    /// Lanjutkan pemutaran video YouTube
    pub fn play() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["play"]).spawn();
        }
        Self::send_action("play")
    }

    /// Toggle Play/Pause video YouTube
    pub fn toggle() -> bool {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("playerctl").args(["play-pause"]).spawn();
        }
        Self::send_action("toggle")
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
