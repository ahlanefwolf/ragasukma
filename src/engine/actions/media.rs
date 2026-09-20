/// Eksekusi kontrol media lokal & background audio player (yt-dlp & internal player)
pub fn execute_media_action(action: &str, payload: &str) -> Option<(String, String)> {
    match action {
        "play" => {
            let query = payload.trim();
            if query.is_empty() {
                crate::audio::player::resume();
                crate::app_state::push_client_action("music:play", None);
                return Some(("Putar Musik".to_string(), "Memutar musik internal".to_string()));
            }

            // Cek lagu di direktori lokal
            let music_dir = crate::config::get_music_dir();
            let existing = crate::engine::ytdlp::list_downloaded(&music_dir);
            let q_lower = query.to_lowercase();
            let q_words: Vec<&str> = q_lower.split_whitespace().collect();

            let found = existing.iter().find(|t| {
                let t_title = t.title.to_lowercase();
                let t_filename = t.filename.to_lowercase();
                t_title.contains(&q_lower)
                    || t_filename.contains(&q_lower)
                    || (!q_words.is_empty() && q_words.iter().all(|w| t_title.contains(w) || t_filename.contains(w)))
            });

            if let Some(track) = found {
                let file_path = music_dir.join(&track.filename);
                // 1. Putar langsung lewat speaker desktop (Native Audio Player)
                crate::audio::player::play_file(&file_path, &track.title);

                // 2. Sinkronkan event ke Web Dashboard
                crate::app_state::push_client_action("music:play", Some(&track.title));
                return Some((
                    "Putar Musik".to_string(),
                    format!("Memutar musik lokal: {}", track.title),
                ));
            }

            // Belum ada di lokal -> antrekan download & putar via background worker
            crate::engine::tasks::queue_download_and_play(query);
            Some((
                "Putar Musik".to_string(),
                format!("Mengunduh & memutar lagu '{}' di latar belakang", query),
            ))
        }

        "pause" => {
            crate::audio::player::pause();
            crate::app_state::push_client_action("music:pause", None);
            Some(("Jeda Musik".to_string(), "Musik internal dijeda".to_string()))
        }

        "resume" => {
            crate::audio::player::resume();
            crate::app_state::push_client_action("music:resume", None);
            Some(("Lanjut Musik".to_string(), "Melanjutkan musik internal".to_string()))
        }

        "next" => {
            // Putar lagu berikutnya dari daftar lokal
            let music_dir = crate::config::get_music_dir();
            let tracks = crate::engine::ytdlp::list_downloaded(&music_dir);
            if !tracks.is_empty() {
                let current_title = crate::audio::player::get_current_title().unwrap_or_default();
                let curr_idx = tracks.iter().position(|t| t.title == current_title).unwrap_or(0);
                let next_idx = (curr_idx + 1) % tracks.len();
                let next_track = &tracks[next_idx];
                let file_path = music_dir.join(&next_track.filename);
                crate::audio::player::play_file(&file_path, &next_track.title);
                crate::app_state::push_client_action("music:play", Some(&next_track.title));
            } else {
                crate::app_state::push_client_action("music:next", None);
            }
            Some(("Lagu Berikutnya".to_string(), "Beralih ke trek berikutnya".to_string()))
        }

        "prev" => {
            let music_dir = crate::config::get_music_dir();
            let tracks = crate::engine::ytdlp::list_downloaded(&music_dir);
            if !tracks.is_empty() {
                let current_title = crate::audio::player::get_current_title().unwrap_or_default();
                let curr_idx = tracks.iter().position(|t| t.title == current_title).unwrap_or(0);
                let prev_idx = if curr_idx == 0 { tracks.len() - 1 } else { curr_idx - 1 };
                let prev_track = &tracks[prev_idx];
                let file_path = music_dir.join(&prev_track.filename);
                crate::audio::player::play_file(&file_path, &prev_track.title);
                crate::app_state::push_client_action("music:play", Some(&prev_track.title));
            } else {
                crate::app_state::push_client_action("music:prev", None);
            }
            Some(("Lagu Sebelumnya".to_string(), "Beralih ke trek sebelumnya".to_string()))
        }

        "stop" => {
            crate::audio::player::stop();
            crate::app_state::push_client_action("music:stop", None);
            Some(("Stop Musik".to_string(), "Musik internal dihentikan".to_string()))
        }

        _ => None,
    }
}
