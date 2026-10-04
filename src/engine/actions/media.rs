/// Eksekusi kontrol media lokal & background audio player (yt-dlp & internal player)
pub fn execute_media_action(action: &str, payload: &str) -> Option<(String, String)> {
    match action {
        "play" => {
            let query = payload.trim();
            if query.is_empty() {
                crate::audio::player::resume();
                crate::app_state::push_client_action("music:play", None);
                return Some((
                    "Play Music".to_string(),
                    "Playing internal music".to_string(),
                ));
            }

            // Check track in local directory
            let music_dir = crate::config::get_music_dir();
            let existing = crate::engine::ytdlp::list_downloaded(&music_dir);
            let q_lower = query.to_lowercase();
            let q_words: Vec<&str> = q_lower.split_whitespace().collect();

            let found = existing.iter().find(|t| {
                let t_title = t.title.to_lowercase();
                let t_filename = t.filename.to_lowercase();
                t_title.contains(&q_lower)
                    || t_filename.contains(&q_lower)
                    || (!q_words.is_empty()
                        && q_words
                            .iter()
                            .all(|w| t_title.contains(w) || t_filename.contains(w)))
            });

            // Fallback: Fuzzy Similarity Matching for judul track lokal
            let found = found.or_else(|| {
                existing
                    .iter()
                    .filter_map(|t| {
                        let sim = crate::engine::perception::string_similarity(query, &t.title)
                            .max(crate::engine::perception::string_similarity(query, &t.filename));
                        if sim >= 0.70 {
                            Some((t, sim))
                        } else {
                            None
                        }
                    })
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(t, sim)| {
                        crate::term_log!(
                            "[Fuzzy Match Music] Found similar song '{}' -> '{}' (score: {:.1}%)",
                            query,
                            t.title,
                            sim * 100.0
                        );
                        t
                    })
            });

            if let Some(track) = found {
                let file_path = music_dir.join(&track.filename);
                // 1. Play direct lewat speaker desktop (Native Audio Player)
                crate::audio::player::play_file(&file_path, &track.title);

                // 2. Synchronize event to Web Dashboard & Smartphone Client
                // Always emit music:play event so mobile/web audio resumes playback if previously interrupted
                crate::app_state::push_client_action("music:play", Some(&track.title));
                return Some((
                    "Play Music".to_string(),
                    format!("Playing local music: {}", track.title),
                ));
            }

            // Not available locally yet -> queue download & play via background worker
            crate::engine::tasks::queue_download_and_play(query);
            Some((
                "Play Music".to_string(),
                format!("Downloading & playing song '{}' in the background", query),
            ))
        }

        "pause" => {
            crate::audio::player::pause();
            crate::app_state::push_client_action("music:pause", None);
            Some((
                "Pause Music".to_string(),
                "Internal music paused".to_string(),
            ))
        }

        "resume" => {
            crate::audio::player::resume();
            crate::app_state::push_client_action("music:resume", None);
            Some((
                "Resume Music".to_string(),
                "Resuming internal music".to_string(),
            ))
        }

        "next" => {
            // Play next track from local list
            let music_dir = crate::config::get_music_dir();
            let tracks = crate::engine::ytdlp::list_downloaded(&music_dir);
            if !tracks.is_empty() {
                let current_title = crate::audio::player::get_current_title().unwrap_or_default();
                let curr_idx = tracks
                    .iter()
                    .position(|t| t.title == current_title)
                    .unwrap_or(0);
                let next_idx = (curr_idx + 1) % tracks.len();
                let next_track = &tracks[next_idx];
                let file_path = music_dir.join(&next_track.filename);
                crate::audio::player::play_file(&file_path, &next_track.title);
                crate::app_state::push_client_action("music:play", Some(&next_track.title));
            } else {
                crate::app_state::push_client_action("music:next", None);
            }
            Some((
                "Next Song".to_string(),
                "Switching to next track".to_string(),
            ))
        }

        "prev" => {
            let music_dir = crate::config::get_music_dir();
            let tracks = crate::engine::ytdlp::list_downloaded(&music_dir);
            if !tracks.is_empty() {
                let current_title = crate::audio::player::get_current_title().unwrap_or_default();
                let curr_idx = tracks
                    .iter()
                    .position(|t| t.title == current_title)
                    .unwrap_or(0);
                let prev_idx = if curr_idx == 0 {
                    tracks.len() - 1
                } else {
                    curr_idx - 1
                };
                let prev_track = &tracks[prev_idx];
                let file_path = music_dir.join(&prev_track.filename);
                crate::audio::player::play_file(&file_path, &prev_track.title);
                crate::app_state::push_client_action("music:play", Some(&prev_track.title));
            } else {
                crate::app_state::push_client_action("music:prev", None);
            }
            Some((
                "Previous Song".to_string(),
                "Switching to previous track".to_string(),
            ))
        }

        "stop" => {
            crate::audio::player::stop();
            crate::app_state::push_client_action("music:stop", None);
            Some((
                "Stop Music".to_string(),
                "Internal music stopped".to_string(),
            ))
        }

        _ => None,
    }
}
