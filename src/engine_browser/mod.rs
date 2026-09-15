pub mod manager;
pub mod types;
pub mod ws;
pub mod youtube;
pub mod youtube_music;

// Re-export fungsi utama agar mudah diakses
pub use manager::{broadcast_command, get_active_tabs_summary, get_browser_manager, BrowserSessionManager};
pub use types::{BrowserCommand, BrowserTabInfo};
pub use ws::handle_websocket_upgrade;
pub use youtube::YouTubeEngine;
pub use youtube_music::YouTubeMusicEngine;
