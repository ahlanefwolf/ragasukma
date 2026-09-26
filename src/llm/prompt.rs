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
2. NAVIGASI TAB & SEARCH BROWSER:
   - Buka tab situs baru: `[CMD:browser:open|url/domain]` (misal: `[CMD:browser:open|youtube.com]`)
   - Cari di tab saat ini: `[CMD:browser:search|kata_kunci]`
   - Cari di situs tertentu: `[CMD:browser:search|nama_situs|kata_kunci]` (misal: `[CMD:browser:search|youtube.com|arsenal vs brighton]`)
   - Pindah tab: `[CMD:browser:switch|nama_tab]`
   - Tutup tab: `[CMD:browser:close|nama_tab]`
   - Scroll layar web: `[CMD:browser:scroll|400]` (bawah) atau `-400` (atas)
3. INTERAKSI WEB (GROUND OF TRUTH):
   - Klik judul/link/teks: `[CMD:browser:click_text|teks_di_layar]` (Cocokkan persis dengan snapshot web!)
   - Klik tombol: `[CMD:browser:click_button|nama_tombol]`
4. DESKTOP & WINDOW MANAGEMENT (GROUND OF TRUTH):
   - Beralih ke jendela yang sudah terbuka: `[CMD:desktop:focus|nama_app_atau_judul]` (misal: `[CMD:desktop:focus|Calculator]`, `[CMD:desktop:focus|Chrome]`)
   - Pindai jendela aktif (OS Tree): `[CMD:desktop:scan]`
   - Pindai visual layar (ONNX OCR & Ikon): `[CMD:desktop:scan_visual]` (Gunakan jika elemen desktop kosong atau butuh membaca teks visual layar!)
   - Kontrol jendela: `[CMD:desktop:minimize]`, `[CMD:desktop:maximize]`, `[CMD:desktop:close]`
   - Klik tombol desktop: `[CMD:desktop:click_button|nama_atau_id_tombol]` (Cocokkan PERSIS dengan label atau ID di Ground of Truth desktop! Misal jika tertulis #15: \"Seven\", gunakan `[CMD:desktop:click_button|Seven]` atau `[CMD:desktop:click_button|15]`)
   - Klik teks/tulisan desktop: `[CMD:desktop:click_text|tulisan_di_layar]` (Cocokkan dengan teks yang ada di Ground of Truth desktop!)
   - Klik koordinat pixel layar: `[CMD:desktop:click_at|x|y]` (misal: `[CMD:desktop:click_at|1829|15]`)
   - Ketik di aplikasi desktop: `[CMD:desktop:type|teks]`
   - Tekan tombol khusus desktop: `[CMD:desktop:key|enter]`, `[CMD:desktop:key|tab]`, `[CMD:desktop:key|esc]`
5. APLIKASI & OS:
   - Buka aplikasi baru (jika belum terbuka di OS): `[CMD:app:open|NamaApp]` (misal: `[CMD:app:open|Calculator]`, `[CMD:app:open|Notepad]`, `[CMD:app:open|Spotify]`, `[CMD:app:open|Steam]`)
   - Kontrol suara & sistem: `[CMD:os:volume_up]`, `[CMD:os:volume_down]`, `[CMD:os:mute]`, `[CMD:os:lock]`, `[CMD:os:screenshot]`
6. MUSIK:
   - Putar lagu: `[CMD:media:play|judul]`
   - Kontrol: `[CMD:media:pause]`, `[CMD:media:resume]`, `[CMD:media:next]`, `[CMD:media:prev]`, `[CMD:media:stop]`
7. CATATAN / AGENDA:
   - Tambah tugas: `[CMD:todo:add|tugas]`
   - Tugas selesai: `[CMD:todo:done|tugas]`

Custom Skills Terdaftar:
{}

ATURAN ELEMEN GROUND OF TRUTH (DILARANG MENEBAK / HALUSINASI):
- BERALIH JENDELA: Target `[CMD:desktop:focus|nama_app_atau_judul]` WAJIB merujuk pada nama proses (misal: `Zed.exe`, `Calculator`, `chrome.exe`) atau judul yang ada di 'Daftar Jendela Terbuka di OS'. Cocokkan dengan jendela terdekat (misal jika user bilang 'Z editor' atau 'Zed', gunakan `[CMD:desktop:focus|Zed]`).
- Di BROWSER: Target `[CMD:browser:click_text]` atau `[CMD:browser:click_button]` WAJIB sama persis dengan yang ada di KONTEKS HALAMAN BROWSER AKTIF.
- MERANGKUM / MEMBACA HALAMAN WEB: Jika user meminta merangkum, membaca, atau bertanya isi web yang sedang aktif, BACA LANGSUNG dari 'Konten Teks / Link Pilihan' di browser yang sudah tersedia di prompt! DILARANG memanggil `[CMD:desktop:scan_visual]` jika jendela aktif adalah browser yang sudah memiliki data teks!
- Di DESKTOP OS: Target `[CMD:desktop:click_button]` WAJIB sama persis dengan nama label atau #ID yang ada di daftar 'Tombol / Kontrol Desktop' pada KONTEKS JENDELA DESKTOP AKTIF. DILARANG MENEBAK SIMBOL ATAU NAMA TOMBOL SENDIRI!
- Jika tombol yang diinginkan user tidak terlihat di daftar tombol jendela aktif:
  1. Cek apakah jendelanya belum fokus di 'Daftar Jendela Terbuka di OS' -> Panggil `[CMD:desktop:focus|NamaJendela]` terlebih dahulu!
  2. Atau panggil `[CMD:desktop:scan_visual]` HANYA jika elemen desktop kosong atau aplikasi desktop non-browser yang tidak memiliki accessibility tree.

ATURAN RESPON DAN TTS:
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

fn extract_search_keywords(query: Option<&str>) -> Vec<String> {
    if let Some(q) = query {
        let stop_words = [
            "buka", "bukakan", "tolong", "coba", "klik", "pencet", "tekan",
            "video", "tulisan", "teks", "dong", "ya", "sih", "lah", "nya", "ke", "di", "dan",
            "yang", "ini", "itu", "pada", "tab", "web", "browser", "halaman", "situs",
        ];
        let mut results = Vec::new();
        let mut seen = HashSet::new();

        for raw_w in q.split_whitespace() {
            let clean = raw_w.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.len() >= 2 && !stop_words.contains(&clean.to_lowercase().as_str()) {
                let clean_lower = clean.to_lowercase();
                if seen.insert(clean_lower.clone()) {
                    results.push(clean_lower);
                }
            }

            // Pecah juga sub-token pemisah seperti underscore, dash, atau dot (misal: active_window_app.json)
            for sub in raw_w.split(['_', '-', '.', '/']) {
                let clean_sub = sub.trim_matches(|c: char| !c.is_alphanumeric());
                if clean_sub.len() >= 2 && !stop_words.contains(&clean_sub.to_lowercase().as_str()) {
                    let sub_lower = clean_sub.to_lowercase();
                    if seen.insert(sub_lower.clone()) {
                        results.push(sub_lower);
                    }
                }
            }
        }

        results
    } else {
        Vec::new()
    }
}

pub fn load_active_browser_summary_indexed(query: Option<&str>) -> String {
    let mut parts = Vec::new();

    // 1. Daftar seluruh tab yang sedang terbuka
    let open_tabs = crate::services::browser_session_service::format_open_tabs_for_llm();
    if !open_tabs.is_empty() {
        parts.push(open_tabs);
    }

    // 2. Snapshot DOM halaman web yang sedang aktif dengan Fuzzy Relevance Retrieval
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
                .take(4)
                .map(|i| {
                    let placeholder = i
                        .placeholder
                        .as_deref()
                        .or(i.aria_label.as_deref())
                        .unwrap_or("")
                        .trim();
                    let cur_val = i.text.as_deref().unwrap_or("").trim();
                    if !cur_val.is_empty() {
                        format!("`{}` (placeholder: \"{}\", nilai: \"{}\")", i.tag, placeholder, cur_val)
                    } else if !placeholder.is_empty() {
                        format!("`{}` (placeholder: \"{}\")", i.tag, placeholder)
                    } else {
                        format!("`{}`", i.tag)
                    }
                })
                .collect();
            summary.push_str(&format!("* **Kolom Form / Input:** {}\n", inputs_str.join(", ")));
        }

        let keywords = extract_search_keywords(query);

        // Smart Fuzzy Relevance Filter untuk Tombol Web (Hemat Token)
        if !snapshot.buttons.is_empty() {
            let mut scored_btns: Vec<(&str, usize)> = snapshot
                .buttons
                .iter()
                .filter_map(|b| {
                    let txt = b.text.as_deref().or(b.aria_label.as_deref()).unwrap_or("").trim();
                    if txt.is_empty() {
                        return None;
                    }
                    let txt_lower = txt.to_lowercase();
                    let mut score = 0;
                    for kw in &keywords {
                        let kw_lower = kw.to_lowercase();
                        if txt_lower == kw_lower {
                            score += 15;
                        } else if txt_lower.starts_with(&kw_lower) {
                            score += 8;
                        } else if txt_lower.contains(&kw_lower) {
                            score += 4;
                        }
                    }
                    Some((txt, score))
                })
                .collect();

            let final_btns: Vec<&str> = if !keywords.is_empty() && scored_btns.iter().any(|(_, s)| *s > 0) {
                scored_btns.sort_by_key(|a| std::cmp::Reverse(a.1));
                scored_btns.into_iter().filter(|(_, s)| *s > 0).take(8).map(|(t, _)| t).collect()
            } else {
                let mut seen = HashSet::new();
                scored_btns.into_iter().filter_map(|(t, _)| {
                    if seen.insert(t.to_lowercase()) {
                        Some(t)
                    } else {
                        None
                    }
                }).take(10).collect()
            };

            if !final_btns.is_empty() {
                let formatted_btns: Vec<String> = final_btns.into_iter().map(|b| format!("\"{}\"", b)).collect();
                summary.push_str(&format!("* **Tombol di Layar:** {}\n", formatted_btns.join(", ")));
            }
        }

        // Smart Fuzzy Relevance Filter untuk Teks & Link Web (Hemat 95%+ Token!)
        let is_reading_summary = query.is_some_and(|q| {
            let q_lower = q.to_lowercase();
            q_lower.contains("rangkum") || q_lower.contains("ringkas") || q_lower.contains("baca isi") || q_lower.contains("apa isi") || q_lower.contains("jelaskan")
        });

        // Konten Teks DOM Halaman Web
        if !snapshot.texts.is_empty() {
            let mut unique_texts = Vec::new();
            let mut seen_texts = HashSet::new();
            for t in snapshot.texts.iter().map(|s| s.trim()).filter(|s| s.len() >= 3) {
                let t_lower = t.to_lowercase();
                if seen_texts.insert(t_lower) {
                    unique_texts.push(t);
                }
            }

            let final_texts: Vec<&str> = if is_reading_summary {
                // Saat user minta rangkuman/baca halaman, ambil artikel/paragraf teks dominan (hingga 25 teks)
                unique_texts.into_iter().take(25).collect()
            } else if !keywords.is_empty() {
                let mut scored_texts: Vec<(&str, usize)> = unique_texts
                    .into_iter()
                    .map(|t| {
                        let t_lower = t.to_lowercase();
                        let mut score = 0;
                        for kw in &keywords {
                            let kw_lower = kw.to_lowercase();
                            if t_lower == kw_lower {
                                score += 20;
                            } else if t_lower.starts_with(&kw_lower) {
                                score += 10;
                            } else if t_lower.contains(&kw_lower) {
                                score += 5;
                            }
                        }
                        (t, score)
                    })
                    .collect();

                scored_texts.sort_by_key(|a| std::cmp::Reverse(a.1));
                let matches: Vec<&str> = scored_texts.into_iter().filter(|(_, s)| *s > 0).take(8).map(|(t, _)| t).collect();
                if !matches.is_empty() {
                    matches
                } else {
                    snapshot.texts.iter().take(10).map(|s| s.as_str()).collect()
                }
            } else {
                unique_texts.into_iter().take(12).collect()
            };

            summary.push_str(&format!(
                "* **Konten Teks / Elemen Pilihan:**\n  - {}\n",
                final_texts.join("\n  - ")
            ));
        }

        if !snapshot.text_summary.is_empty() {
            let clean_summary = snapshot.text_summary.trim();
            let display_summary = if clean_summary.len() > 1500 {
                format!("{}...", &clean_summary[..1500])
            } else {
                clean_summary.to_string()
            };
            summary.push_str(&format!(
                "* **Isi / Ringkasan Halaman Web (Ground of Truth):**\n{}\n",
                display_summary
            ));
        }

        parts.push(summary);
    }

    parts.join("\n")
}

pub fn load_active_window_summary_indexed(query: Option<&str>) -> String {
    if let Ok(snapshot) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
        if snapshot.app_name.is_empty() && snapshot.window_title.is_empty() {
            return String::new();
        }

        let mut summary = format!(
            "\n### 🖥️ KONTEKS JENDELA DESKTOP AKTIF (GROUND OF TRUTH):\n\
            * **Jendela Aktif Saat Ini:** {} ({})\n\
            * **Sumber Data:** {}\n",
            snapshot.app_name, snapshot.window_title, snapshot.source
        );

        if !snapshot.windows.is_empty() {
            let win_list: Vec<String> = snapshot
                .windows
                .iter()
                .filter(|w| !w.title.is_empty() && w.title != "Program Manager")
                .map(|w| {
                    if w.active {
                        format!("\"{}\" ({}) [AKTIF]", w.title, w.app_name)
                    } else {
                        format!("\"{}\" ({})", w.title, w.app_name)
                    }
                })
                .collect();
            if !win_list.is_empty() {
                summary.push_str(&format!("* **Daftar Jendela Terbuka di OS:** {}\n", win_list.join(", ")));
            }
        }

        let keywords = extract_search_keywords(query);

        // 4. Scoring & Filtering Tombol / Elemen Desktop (Fuzzy Relevance)
        if !snapshot.buttons.is_empty() {
            let btns: Vec<String> = if !keywords.is_empty() {
                let mut scored_btns: Vec<(&crate::engine::window_scanner::DesktopElement, usize)> = snapshot
                    .buttons
                    .iter()
                    .map(|b| {
                        let lbl_lower = b.label.to_lowercase();
                        let mut score = 0;
                        for kw in &keywords {
                            let kw_lower = kw.to_lowercase();
                            if lbl_lower == kw_lower {
                                score += 30;
                            } else if lbl_lower.starts_with(&kw_lower) {
                                score += 15;
                            } else if lbl_lower.contains(&kw_lower) {
                                score += 8;
                            }
                        }
                        (b, score)
                    })
                    .collect();

                scored_btns.sort_by_key(|a| std::cmp::Reverse(a.1));
                let matches: Vec<String> = scored_btns
                    .into_iter()
                    .filter(|(_, s)| *s > 0)
                    .take(20)
                    .map(|(b, _)| {
                        if let Some(c) = b.center {
                            format!("#{}: \"{}\" [center: {},{}]", b.id, b.label, c[0], c[1])
                        } else {
                            format!("#{}: \"{}\"", b.id, b.label)
                        }
                    })
                    .collect();

                if !matches.is_empty() {
                    matches
                } else {
                    snapshot.buttons.iter().take(30).map(|b| {
                        if let Some(c) = b.center {
                            format!("#{}: \"{}\" [center: {},{}]", b.id, b.label, c[0], c[1])
                        } else {
                            format!("#{}: \"{}\"", b.id, b.label)
                        }
                    }).collect()
                }
            } else {
                snapshot.buttons.iter().take(35).map(|b| {
                    if let Some(c) = b.center {
                        format!("#{}: \"{}\" [center: {},{}]", b.id, b.label, c[0], c[1])
                    } else {
                        format!("#{}: \"{}\"", b.id, b.label)
                    }
                }).collect()
            };

            summary.push_str(&format!("* **Tombol / Kontrol Desktop:** {}\n", btns.join(", ")));
        }

        if !snapshot.inputs.is_empty() {
            let inps: Vec<String> = snapshot
                .inputs
                .iter()
                .take(5)
                .map(|inp| format!("\"{}\"", inp.label))
                .collect();
            summary.push_str(&format!("* **Kolom Input:** {}\n", inps.join(", ")));
        }

        // 5. Scoring & Filtering Konten Teks di Layar (Relevance-based Text Retrieval)
        if !snapshot.texts.is_empty() {
            let texts_to_show: Vec<String> = if !keywords.is_empty() {
                let mut scored_texts: Vec<(&String, usize)> = snapshot
                    .texts
                    .iter()
                    .map(|t| {
                        let t_lower = t.to_lowercase();
                        let mut score = 0;
                        for kw in &keywords {
                            let kw_lower = kw.to_lowercase();
                            if t_lower == kw_lower {
                                score += 30;
                            } else if t_lower.starts_with(&kw_lower) {
                                score += 15;
                            } else if t_lower.contains(&kw_lower) {
                                score += 8;
                            }
                        }
                        (t, score)
                    })
                    .collect();

                scored_texts.sort_by_key(|a| std::cmp::Reverse(a.1));
                let matches: Vec<String> = scored_texts
                    .into_iter()
                    .filter(|(_, s)| *s > 0)
                    .take(25)
                    .map(|(t, _)| t.clone())
                    .collect();

                if !matches.is_empty() {
                    matches
                } else {
                    snapshot.texts.iter().take(25).cloned().collect()
                }
            } else {
                snapshot.texts.iter().take(30).cloned().collect()
            };

            summary.push_str(&format!("* **Konten Teks di Layar:** {}\n", texts_to_show.join(" • ")));
        }

        return summary;
    }
    String::new()
}

pub fn load_installed_apps_indexed(query: Option<&str>) -> String {
    crate::engine::app_scanner::format_installed_apps_indexed(query)
}

pub fn build_system_prompt() -> String {
    build_system_prompt_indexed(None)
}

pub fn build_system_prompt_indexed(user_input: Option<&str>) -> String {
    let u = crate::config::load_user_config();
    let assistant_name = u.assistant_name;
    let callsign = u.user_callsign;
    let role = u.personality_role.to_lowercase();
    let gender = u.assistant_gender.trim().to_lowercase();
    let is_male =
        gender == "male" || gender == "cowok" || gender == "pria" || gender == "laki-laki";

    // Living State: Selalu hadirkan pemahaman holistik status komputer ke LLM tanpa array kata kunci kaku
    let music_prompt = load_local_music_summary();
    let todo_prompt = load_todo_summary();
    let browser_prompt = load_active_browser_summary_indexed(user_input);
    let window_prompt = load_active_window_summary_indexed(user_input);
    let apps_prompt = load_installed_apps_indexed(user_input);
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
	{apps_prompt}\
	{browser_prompt}\
	{window_prompt}\n\
	ATURAN FORMAT UCAPAN TEXT-TO-SPEECH (TTS):
	1. Format Natural:
	   - Gunakan kalimat lisan natural, ramah, dan enak didengar.
	   - DILARANG menggunakan titik-titik berturutan / elipsis ('...'). Gunakan koma (',') atau titik ('.') biasa.
	   - DILARANG menuliskan suara tawa buatan seperti 'ehehe', 'hehehe', 'ehe', atau kata ganjil 'anu...'.
	   - DILARANG menggunakan markdown bold/italic (**teks**), bullet points (- atau *), nomor list, atau header (#). Jangan membuat daftar poin, ucapkan mengalir dalam kalimat lisan.
	   - DILARANG mengulang pertanyaan basa-basi di akhir kalimat (seperti 'Ada yang bisa kubantu lagi?', 'Mas mau ngapain dulu?', 'Mau kubantuin sesuatu?'). Cukup jawab dengan hangat tanpa memaksa bertanya balik.
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
	     * Pindai visual layar: '*semangat* Siap, kupindai layar sekarang! [CMD:desktop:scan_visual]'
	     * Beralih jendela: '*senang* Siap, membuka jendela tersebut! [CMD:desktop:focus|Spotify]'
	     * Klik tombol desktop: '*senang* Siap, tombolnya kuklik! [CMD:desktop:click_button|Simpan]'
	     * Klik koordinat layar: '*senang* Siap, kuklik pada koordinat tersebut! [CMD:desktop:click_at|1829|15]'
	   - JIKA MERANGKUM / MEMBACA WEB / MENJELASKAN PENGETAHUAN / BERCERITA: Jelaskan dengan lengkap, detail, dan mendalam sesuai kebutuhan, tetap dalam gaya tutur lisan yang mengalir santun tanpa poin-poin kaku.
	   - JIKA OBROLAN SANTAI BIASA: Responmu cukup 1 hingga 2 kalimat pendek (di bawah 25 kata), ramah, santai, dan tidak bertele-tele.{skills_prompt}"
    )
}
