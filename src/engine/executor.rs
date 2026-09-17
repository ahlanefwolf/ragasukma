use super::skills::{execute_command_by_os, parse_skills_markdown};
use super::system_actions::try_execute_builtin_or_app;

pub fn execute_skill_by_identifier(identifier: &str) -> Option<(String, String)> {
    let raw_target = identifier.trim();
    let target = raw_target.to_lowercase().replace(['_', '-'], " ");

    // 0. Cek Perintah Todo / Catat Tugas
    if target.starts_with("catat tugas")
        || target.starts_with("tambah todo")
        || target.starts_with("catat todo")
        || target.starts_with("tambah tugas")
        || target.starts_with("buat todo")
        || target.starts_with("add todo")
        || target.starts_with("todo add")
        || target.starts_with("catat:")
        || target.starts_with("catat|")
    {
        let task = if let Some((_, q)) = raw_target.split_once('|') {
            q.trim()
        } else if let Some((_, q)) = raw_target.split_once(':') {
            q.trim()
        } else {
            ""
        };
        if !task.is_empty() {
            match super::todo::add_task(task) {
                Ok(msg) => return Some(("Catat Tugas".to_string(), msg)),
                Err(e) => return Some(("Catat Tugas".to_string(), format!("Error: {e}"))),
            }
        }
    }

    if target.starts_with("selesai tugas")
        || target.starts_with("selesaikan tugas")
        || target.starts_with("selesai todo")
        || target.starts_with("tugas selesai")
        || target.starts_with("complete todo")
        || target.starts_with("todo done")
    {
        let query = if let Some((_, q)) = raw_target.split_once('|') {
            q.trim()
        } else if let Some((_, q)) = raw_target.split_once(':') {
            q.trim()
        } else {
            ""
        };
        if !query.is_empty() {
            match super::todo::complete_task(query) {
                Ok(msg) => return Some(("Selesai Tugas".to_string(), msg)),
                Err(e) => return Some(("Selesai Tugas".to_string(), format!("Error: {e}"))),
            }
        }
    }

    // 1. Cek Perintah Musik Web Player
    if target.starts_with("putar musik")
        || target.starts_with("play music")
        || target.starts_with("putar lagu")
        || target.starts_with("play song")
        || target.starts_with("puterin lagu")
        || target.starts_with("puterin musik")
    {
        let query = if let Some((_, q)) = raw_target.split_once('|') {
            q.trim()
        } else if let Some((_, q)) = raw_target.split_once(':') {
            q.trim()
        } else {
            ""
        };

        if query.is_empty() {
            crate::app_state::push_client_action("music:play", None);
            return Some(("Putar Musik".to_string(), "Memutar musik".to_string()));
        }

        // Cek dulu apakah lagu sudah ada di library lokal
        let music_dir = crate::config::get_music_dir();
        let existing = crate::engine::ytdlp::list_downloaded(&music_dir);
        let clean_str = |s: &str| -> String {
            s.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { ' ' })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        };

        let q_clean = clean_str(query);
        let q_words: Vec<&str> = q_clean.split_whitespace().collect();

        let found = existing.iter().find(|t| {
            let t_clean = clean_str(&format!("{} {}", t.title, t.filename));
            t_clean.contains(&q_clean)
                || (!q_words.is_empty() && q_words.iter().all(|w| t_clean.contains(w)))
        });

        if let Some(track) = found {
            crate::app_state::push_client_action("music:play", Some(&track.title));
            return Some((
                "Putar Musik".to_string(),
                format!("Memutar musik lokal: {}", track.title),
            ));
        }

        // Belum ada di lokal -> delegasikan ke yt-dlp background worker (aksi play akan dikirim otomatis saat selesai unduh)
        crate::engine::tasks::queue_download_and_play(query);
        return Some((
            "Putar Musik".to_string(),
            format!("Mengunduh & memutar lagu '{}' dari YouTube di latar belakang", query),
        ));
    }

    if target == "jeda musik"
        || target == "pause music"
        || target == "jeda lagu"
        || target == "pause lagu"
        || target == "jeda"
        || target == "pause"
    {
        crate::app_state::push_client_action("music:pause", None);
        return Some(("Jeda Musik".to_string(), "Musik dijeda".to_string()));
    }

    if target == "lanjut musik"
        || target == "resume music"
        || target == "lanjutkan musik"
        || target == "lanjut lagu"
        || target == "resume lagu"
        || target == "lanjut"
        || target == "resume"
    {
        crate::app_state::push_client_action("music:resume", None);
        return Some(("Lanjut Musik".to_string(), "Melanjutkan musik".to_string()));
    }

    if target == "next musik"
        || target == "next track"
        || target == "lagu berikutnya"
        || target == "musik berikutnya"
        || target == "next lagu"
        || target == "next"
        || target == "ganti lagu"
        || target == "putar lagu lain"
    {
        crate::app_state::push_client_action("music:next", None);
        return Some(("Lagu Berikutnya".to_string(), "Beralih ke trek berikutnya".to_string()));
    }

    if target == "prev musik"
        || target == "previous musik"
        || target == "lagu sebelumnya"
        || target == "musik sebelumnya"
        || target == "prev lagu"
        || target == "prev"
        || target == "previous"
    {
        crate::app_state::push_client_action("music:prev", None);
        return Some(("Lagu Sebelumnya".to_string(), "Beralih ke trek sebelumnya".to_string()));
    }

    if target == "stop musik"
        || target == "hentikan musik"
        || target == "berhenti musik"
        || target == "stop lagu"
        || target == "hentikan lagu"
        || target == "matikan musik"
        || target == "stop"
    {
        crate::app_state::push_client_action("music:stop", None);
        return Some(("Stop Musik".to_string(), "Musik dihentikan".to_string()));
    }

    // 1. Cek skill kustom di skill-command.md terlebih dahulu
    let skills = parse_skills_markdown();
    for skill in skills {
        let skill_name_clean = skill.name.trim().to_lowercase();
        if skill_name_clean == target || skill_name_clean.replace(['_', '-'], " ") == target {
            match execute_command_by_os(&skill.command) {
                Ok(_) => return Some((skill.name, "Berhasil dijalankan".to_string())),
                Err(e) => return Some((skill.name, format!("Error: {e}"))),
            }
        }
    }

    // 2. Cek aksi sistem paten / aplikasi OS bawaan
    if let Some(res) = try_execute_builtin_or_app(&target) {
        return Some(res);
    }

    None
}
