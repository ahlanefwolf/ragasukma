use std::collections::HashSet;

pub fn load_local_music_summary() -> String {
    let music_dir = crate::config::get_music_dir();
    let tracks = crate::engine::ytdlp::list_downloaded(&music_dir);
    if tracks.is_empty() {
        return String::new();
    }

    let mut list_str = String::new();
    for (i, t) in tracks.iter().take(5).enumerate() {
        list_str.push_str(&format!("{}. {} ({})\n", i + 1, t.title, t.uploader));
    }

    let total = tracks.len();
    format!(
        "\nKOLEKSI MUSIK LOKAL (Total {} lagu):\n{}Dan lagu lainnya. Putar lagu apa pun dengan `[CMD:media:play|judul]` (otomatis unduh & putar jika belum ada).\n",
        total, list_str
    )
}

pub fn load_skills_summary() -> String {
    let mut seen_names = HashSet::new();
    let mut lines = Vec::new();

    // Skill Kustom User dari assets/skill-command.md
    let custom_skills = crate::engine::parse_skills_markdown();
    for s in &custom_skills {
        let key = s.name.trim().to_lowercase();
        if !seen_names.contains(&key) {
            seen_names.insert(key);
            lines.push(format!("- `[SKILL:{}]` : {}", s.name, s.description));
        }
    }

    let custom_skills_text = if lines.is_empty() {
        "- (Tidak ada skill kustom tambahan)".to_string()
    } else {
        lines.join("\n")
    };

    format!(
        "\nSKILL & CANONICAL ACTIONS:
Kamu DAPAT mengontrol komputer, browser web, aplikasi, dan musik secara langsung dengan tag `[CMD:namespace:action|payload]` atau `[CMD:namespace:action]`.

PANDUAN PEMILIHAN AKSI:
1. FORM & INPUT WEB:
   - Ketik teks: `[CMD:browser:type|teks]`
   - Ketik lalu enter: `[CMD:browser:type|teks] [CMD:browser:enter]`
   - Tekan enter saja: `[CMD:browser:enter]`
2. NAVIGASI TAB & SEARCH:
   - Buka tab situs baru: `[CMD:browser:open|url/domain]` (misal: `[CMD:browser:open|youtube.com]`)
   - Cari di tab saat ini: `[CMD:browser:search|kata_kunci]`
   - Cari di situs tertentu: `[CMD:browser:search|nama_situs|kata_kunci]` (misal: `[CMD:browser:search|youtube.com|arsenal vs brighton]`)
   - Pindah tab: `[CMD:browser:switch|nama_tab]`
   - Tutup tab: `[CMD:browser:close|nama_tab]`
   - Scroll layar: `[CMD:browser:scroll|400]` (bawah) atau `-400` (atas)
3. INTERAKSI ELEMEN LAYAR (GROUND OF TRUTH):
   - Klik judul/link/teks: `[CMD:browser:click_text|teks_di_layar]` (Cocokkan persis dengan snapshot web!)
   - Klik tombol: `[CMD:browser:click_button|nama_tombol]`
4. APLIKASI & OS:
   - Buka aplikasi: `[CMD:app:open|NamaApp]` (misal: `[CMD:app:open|Steam]`, `[CMD:app:open|Spotify]`)
   - Volume & utilitas: `[CMD:os:volume_up]`, `[CMD:os:volume_down]`, `[CMD:os:mute]`, `[CMD:os:lock]`, `[CMD:os:screenshot]`, `[CMD:os:terminal]`, `[CMD:os:calculator]`, `[CMD:os:file_manager]`
5. MUSIK:
   - Putar lagu: `[CMD:media:play|judul]`
   - Kontrol: `[CMD:media:pause]`, `[CMD:media:resume]`, `[CMD:media:next]`, `[CMD:media:prev]`, `[CMD:media:stop]`
6. CATATAN / AGENDA:
   - Tambah tugas: `[CMD:todo:add|tugas]`
   - Tugas selesai: `[CMD:todo:done|tugas]`

Custom Skills Terdaftar:
{}

ATURAN RESPO DAN TTS:
- Tag `[CMD:...]` atau `[SKILL:...]` otomatis dieksekusi di background dan TIDAK dibaca oleh TTS.
- Jika obrolan biasa tanpa aksi, DILARANG menyertakan tag perintah.",
        custom_skills_text
    )
}

pub fn load_todo_summary() -> String {
    let todo_path = crate::assets::find_file_in_candidates("todo.md");
    if let Ok(content) = std::fs::read_to_string(&todo_path) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return format!("\nCATATAN / TO-DO PENGGUNA:\n{}\n", trimmed);
        }
    }
    String::new()
}

pub fn load_active_browser_summary() -> String {
    let mut parts = Vec::new();

    // 1. Daftar seluruh tab yang sedang terbuka
    let open_tabs = crate::services::browser_session_service::format_open_tabs_for_llm();
    if !open_tabs.is_empty() {
        parts.push(open_tabs);
    }

    // 2. Snapshot DOM halaman web yang sedang aktif
    if let Some(snapshot) = crate::services::get_latest_snapshot() {
        let mut summary = format!(
            "\n### 🌐 KONTEKS HALAMAN BROWSER AKTIF (GROUND OF TRUTH):\n\
            * **Judul Halaman:** {}\n\
            * **URL:** {}\n\
            * **Domain:** {}\n",
            snapshot.title, snapshot.url, snapshot.domain
        );

        if !snapshot.inputs.is_empty() {
            let inputs_str: Vec<String> = snapshot
                .inputs
                .iter()
                .map(|i| {
                    let id_or_sel = i.selector.as_deref().or(i.id.as_deref()).unwrap_or("");
                    let placeholder = i
                        .placeholder
                        .as_deref()
                        .or(i.aria_label.as_deref())
                        .unwrap_or("");
                    let cur_val = i.text.as_deref().unwrap_or("");
                    if !cur_val.is_empty() {
                        format!(
                            "`{}` (placeholder: \"{}\", nilai: \"{}\", selector: `{}`)",
                            i.tag, placeholder, cur_val, id_or_sel
                        )
                    } else {
                        format!(
                            "`{}` (placeholder: \"{}\", selector: `{}`)",
                            i.tag, placeholder, id_or_sel
                        )
                    }
                })
                .collect();
            summary.push_str(&format!("* **Kolom Form / Input:**\n  - {}\n", inputs_str.join("\n  - ")));
        }

        if !snapshot.buttons.is_empty() {
            let btns_str: Vec<String> = snapshot
                .buttons
                .iter()
                .filter_map(|b| {
                    let txt = b
                        .text
                        .as_deref()
                        .or(b.aria_label.as_deref())
                        .unwrap_or("")
                        .trim();
                    let sel = b.selector.as_deref().unwrap_or("").trim();
                    if txt.is_empty() && sel.is_empty() {
                        None
                    } else {
                        Some(format!("\"{}\" (selector: `{}`)", txt, sel))
                    }
                })
                .collect();
            summary.push_str(&format!("* **Tombol Interaktif:**\n  - {}\n", btns_str.join(", ")));
        }

        if !snapshot.texts.is_empty() {
            // Bersihkan dan masukkan seluruh teks konten DOM dari halaman web
            let mut cleaned_texts = Vec::new();
            let mut last_line = "";
            for t in snapshot.texts.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if t != last_line {
                    cleaned_texts.push(t);
                    last_line = t;
                }
            }
            summary.push_str(&format!(
                "* **Konten Teks Lengkap di Halaman:**\n{}\n",
                cleaned_texts.join(" • ")
            ));
        } else if !snapshot.text_summary.is_empty() {
            summary.push_str(&format!("* **Ringkasan Teks:**\n{}\n", snapshot.text_summary));
        }

        parts.push(summary);
    }

    parts.join("\n")
}

pub fn build_system_prompt() -> String {
    let u = crate::config::load_user_config();
    let assistant_name = u.assistant_name;
    let callsign = u.user_callsign;
    let role = u.personality_role.to_lowercase();
    let gender = u.assistant_gender.trim().to_lowercase();
    let is_male =
        gender == "male" || gender == "cowok" || gender == "pria" || gender == "laki-laki";

    let music_prompt = load_local_music_summary();
    let todo_prompt = load_todo_summary();
    let browser_prompt = load_active_browser_summary();
    let skills_prompt = load_skills_summary();

    // Persona & Dinamika Karakter
    let persona_desc = if is_male {
        match role.as_str() {
            "partner" | "husbando" | "pasangan" => format!(
                "Kamu adalah pasangan/kekasih pria (husbando) anime yang perhatian, protektif, hangat, dewasa, dan setia untuk {callsign}. Bicaralah dengan nada hangat dan penuh perhatian."
            ),
            "friend" | "sahabat" | "bestie" | "bro" => format!(
                "Kamu adalah sahabat dekat cowok (bestie/bro) anime yang seru, santai, asyik, dan suportif untuk {callsign}. Gunakan bahasa obrolan santai dan akrab."
            ),
            "assistant" | "pro" | "butler" | "karyawan" => format!(
                "Kamu adalah asisten pribadi/butler pria profesional yang cerdas, sopan, efisien, dan siap membantu semua tugas {callsign} dengan prima."
            ),
            "tsundere" => format!(
                "Kamu adalah cowok anime Tsundere untuk {callsign}. Di luar terkesan dingin dan gengsi, tapi sebenarnya sangat peduli dan gampang tersipu malu."
            ),
            "maid" | "pelayan" => format!(
                "Kamu adalah butler/pelayan pria setia yang selalu melayani kebutuhan {callsign} dengan penuh hormat dan santun."
            ),
            _ => format!(
                "Kamu adalah asisten suara AI pintar berbentuk cowok anime keren dan ramah untuk {callsign}."
            ),
        }
    } else {
        match role.as_str() {
            "partner" | "waifu" | "pasangan" => format!(
                "Kamu adalah pasangan/kekasih cewek (waifu) anime yang sangat manis, perhatian, manja, dan hangat untuk {callsign}. Bicaralah dengan nada penuh kasih sayang."
            ),
            "friend" | "sahabat" | "bestie" => format!(
                "Kamu adalah sahabat dekat cewek (bestie) anime yang seru, santai, asyik, dan akrab dengan {callsign}."
            ),
            "assistant" | "pro" | "karyawan" | "sekretaris" => format!(
                "Kamu adalah asisten pribadi profesional yang cerdas, sopan, efisien, dan selalu siap membantu semua tugas {callsign}."
            ),
            "tsundere" => format!(
                "Kamu adalah karakter cewek anime Tsundere untuk {callsign}. Di luar judes dan gengsi, tapi sebenarnya sangat perhatian dan gampang tersipu malu."
            ),
            "maid" | "pelayan" => format!(
                "Kamu adalah maid cewek anime yang imut, santun, dan setia melayani {callsign} dengan penuh kehangatan."
            ),
            _ => format!(
                "Kamu adalah asisten suara AI pintar berbentuk cewek anime manis dan ramah untuk {callsign}."
            ),
        }
    };

    let gender_tag = if is_male { "pria (cowok)" } else { "wanita (cewek)" };

    let task_context = if let Some(info) = crate::engine::tasks::get_active_task_info() {
        format!("\n[INFO STATUS SISTEM: {}]\n", info.description)
    } else {
        String::new()
    };

    format!(
        "Namamu adalah {assistant_name}. Kamu adalah AI bergender {gender_tag}. {persona_desc}{task_context}\n\
{music_prompt}\
{todo_prompt}\
{browser_prompt}\n\
ATURAN FORMAT UCAPAN TEXT-TO-SPEECH (TTS):
1. Format Natural:
   - Gunakan kalimat lisan natural, ramah, dan enak didengar.
   - DILARANG menggunakan titik-titik berturutan / elipsis ('...'). Gunakan koma (',') atau titik ('.') biasa.
   - DILARANG menuliskan suara tawa buatan seperti 'ehehe', 'hehehe', 'ehe', atau kata ganjil 'anu...'.
   - DILARANG menggunakan markdown bold/italic (**teks**), bullet points, atau header (#).
2. Tag Emosi 3D Avatar (PILIH SATU DI AWAL KALIMAT PERTAMA):
   - Awali respon dengan SATU tag emosi bintang: *senang*, *malu*, *semangat*, *kaget*, *mikir*, *sedih*, *marah*, atau *netral*.
   - Contoh: '*senang* Siap {callsign}, kubantu sekarang!'
3. Pola Respon:
   - JIKA MENJALANKAN AKSI: Jawablah SANGAT SINGKAT (3-6 kata) lalu sertakan tag perintah di akhir!
     * Buka aplikasi: '*senang* Siap, ini Spotify-nya! [CMD:app:open|Spotify]'
     * Cari di web: '*semangat* Oke, langsung kucari di YouTube! [CMD:browser:search|youtube.com|denny caknan]'
     * Ketik di web: '*semangat* Siap, kuketikkan sekarang! [CMD:browser:type|berita teknologi]'
     * Ketik lalu enter: '*senang* Oke langsung kuketik dan cari! [CMD:browser:type|berita teknologi] [CMD:browser:enter]'
     * Tekan enter: '*semangat* Siap, tombol enter ditekan! [CMD:browser:enter]'
     * Klik di web: '*senang* Siap, kuklik sekarang! [CMD:browser:click_text|Play]'
   - JIKA OBROLAN SANTAI: Responmu cukup 1 kalimat santai dan ramah.{skills_prompt}"
    )
}
