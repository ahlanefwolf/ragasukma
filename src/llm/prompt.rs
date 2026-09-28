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
            lines.push(format!(
                "| `[SKILL:{}]` | Sesuai skrip | {} |",
                s.name, s.description
            ));
        }
    }

    let custom_skills_text = if lines.is_empty() {
        "*(Tidak ada skill kustom tambahan terdaftar)*".to_string()
    } else {
        format!(
            "| Tag Custom Skill | Parameter | Keterangan |\n| :--- | :--- | :--- |\n{}",
            lines.join("\n")
        )
    };

    format!(
        "\nKATALOG AKSI & SKILL (CANONICAL ACTIONS TABLE):
Gunakan tabel di bawah ini sebagai rujukan mutlak pemilihan tag aksi `[CMD:...]` atau `[SKILL:...]`.

| Kategori | Tag Aksi | Parameter | Sumber Data Ground of Truth | Kapan Digunakan & Batasan Mutlak | Contoh Ucapan Pendek Asisten |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Browser Web** | `[CMD:browser:open|url]` | `url/domain` | Masukan user | Buka alamat situs / domain baru di Chromium | '*senang* Siap, kubuka webnya! [CMD:browser:open&#124;youtube.com]' |
| **Browser Web** | `[CMD:browser:search|query]` | `kata_kunci` | Masukan user | Cari di Google / tab pencarian aktif saat ini | '*semangat* Oke, langsung kucari! [CMD:browser:search&#124;berita terkini]' |
| **Browser Web** | `[CMD:browser:search|situs|query]` | `situs&#124;kata_kunci` | Masukan user | Cari langsung di situs web tertentu | '*semangat* Oke, kucari di YouTube! [CMD:browser:search&#124;youtube.com&#124;denny caknan]' |
| **Browser Web** | `[CMD:browser:switch|tab]` | `nama_tab` | Daftar Seluruh Tab Terbuka | Pindah tab web. DILARANG pakai jika targetnya aplikasi OS | '*senang* Pindah ke tab YouTube! [CMD:browser:switch&#124;YouTube]' |
| **Browser Web** | `[CMD:browser:close|tab]` | `nama_tab` | Daftar Seluruh Tab Terbuka | Tutup salah satu tab web di browser | '*senang* Tabnya sudah kututup! [CMD:browser:close&#124;YouTube]' |
| **Browser Web** | `[CMD:browser:scroll|jarak]` | `400` / `-400` | Nilai angka pixel | Gulir halaman web ke bawah atau ke atas | '*senang* Halaman digulir ke bawah! [CMD:browser:scroll&#124;400]' |
| **Elemen Website**| `[CMD:browser:click_text|teks]` | `teks_link_web` | Konten Teks / Elemen Pilihan | Klik tautan/artikel di dalam web. DILARANG jika user bilang 'di layar' | '*senang* Siap, kuklik linknya! [CMD:browser:click_text&#124;Play]' |
| **Elemen Website**| `[CMD:browser:click_button|nama]`| `nama_tombol_web` | Tombol di Halaman Web | Klik tombol form di web. DILARANG jika user bilang 'tombol di layar' | '*senang* Tombol web kuklik! [CMD:browser:click_button&#124;Submit]' |
| **Elemen Website**| `[CMD:browser:type|teks]` | `teks_input` | Kolom Form / Input Web | Ketik teks ke kotak input di halaman web | '*semangat* Siap, kuketik sekarang! [CMD:browser:type&#124;resep masakan]' |
| **Elemen Website**| `[CMD:browser:enter]` | *(Tanpa parameter)* | *(Tidak ada)* | Kirim tombol Enter pada halaman web | '*semangat* Tombol enter ditekan! [CMD:browser:enter]' |
| **Desktop / Layar**| `[CMD:desktop:focus|jendela]` | `app_name&#124;judul` | Daftar Jendela Terbuka di OS | Fokus ke jendela aplikasi komputer (foot, Zed, Chrome, dll) | '*senang* Siap, pindah jendela! [CMD:desktop:focus&#124;Spotify]' |
| **Desktop / Layar**| `[CMD:desktop:click_button|id]`| `label_atau_id_tombol` | Tombol / Kontrol Desktop | WAJIB saat user sebut 'tombol di layar/desktop/window'. Ambil persis! | '*senang* Tombol di layar kuklik! [CMD:desktop:click_button&#124;Simpan]' |
| **Desktop / Layar**| `[CMD:desktop:click_text|teks]` | `tulisan_di_layar` | Konten Teks di Layar | WAJIB saat user sebut 'tulisan/teks di layar/desktop'. Ambil persis! | '*senang* Tulisan di layar kuklik! [CMD:desktop:click_text&#124;Skill Custom]' |
| **Desktop / Layar**| `[CMD:desktop:click_at|x|y]` | `x&#124;y` | Koordinat visual pixel | Klik posisi pixel spesifik layar | '*senang* Koordinat layar kuklik! [CMD:desktop:click_at&#124;1829&#124;15]' |
| **Desktop / Layar**| `[CMD:desktop:type|teks]` | `teks_ketikan` | Kolom Input | Ketik ke jendela aplikasi OS yang sedang aktif | '*semangat* Teks sudah kuketikkan! [CMD:desktop:type&#124;cargo build]' |
| **Desktop / Layar**| `[CMD:desktop:key|nama_key]` | `enter` / `tab` / `esc` | *(Tidak ada)* | Tekan tombol keyboard di aplikasi OS aktif | '*semangat* Tombol keyboard ditekan! [CMD:desktop:key&#124;enter]' |
| **Desktop / Layar**| `[CMD:desktop:scan_visual]` | *(Tanpa parameter)* | Layar monitor via OCR | Scan visual monitor jika user minta scan/lihat layar | '*semangat* Siap kupindai layarnya! [CMD:desktop:scan_visual]' |
| **Desktop / Layar**| `[CMD:desktop:scan]` | *(Tanpa parameter)* | Accessibility tree OS | Perbarui pohon elemen jendela aplikasi aktif | '*senang* Memeriksa elemen jendela! [CMD:desktop:scan]' |
| **Desktop / Layar**| `[CMD:desktop:minimize]` | *(Tanpa parameter)* | Jendela Aktif Saat Ini | Perkecil jendela aplikasi aktif ke panel taskbar | '*senang* Jendelanya sudah diminimalkan! [CMD:desktop:minimize]' |
| **Desktop / Layar**| `[CMD:desktop:maximize]` | *(Tanpa parameter)* | Jendela Aktif Saat Ini | Perbesar / pulihkan jendela aplikasi aktif | '*senang* Ukuran jendela dimaksimalkan! [CMD:desktop:maximize]' |
| **Desktop / Layar**| `[CMD:desktop:close]` | *(Tanpa parameter)* | Jendela Aktif Saat Ini | Tutup jendela aplikasi komputer yang aktif | '*senang* Jendelanya sudah kututup! [CMD:desktop:close]' |
| **Aplikasi OS** | `[CMD:app:open|nama_app]` | `NamaAplikasi` | Daftar Aplikasi Terinstal | Buka aplikasi baru di OS jika belum ada di jendela terbuka | '*senang* Siap, kubuka sekarang! [CMD:app:open&#124;Spotify]' |
| **Kontrol Sistem**| `[CMD:os:volume_up]` | *(Tanpa parameter)* | Audio OS | Naikkan volume audio komputer | '*semangat* Volume suara dinaikkan! [CMD:os:volume_up]' |
| **Kontrol Sistem**| `[CMD:os:volume_down]` | *(Tanpa parameter)* | Audio OS | Turunkan volume audio komputer | '*senang* Volume suara diturunkan! [CMD:os:volume_down]' |
| **Kontrol Sistem**| `[CMD:os:mute]` | *(Tanpa parameter)* | Audio OS | Matikan / bisukan suara speaker komputer | '*senang* Suara komputer dibisukan! [CMD:os:mute]' |
| **Kontrol Sistem**| `[CMD:os:screenshot]` | *(Tanpa parameter)* | Monitor OS | Ambil gambar screenshot layar desktop | '*senang* Tangkapan layar tersimpan! [CMD:os:screenshot]' |
| **Kontrol Sistem**| `[CMD:os:lock]` | *(Tanpa parameter)* | Sesi Pengguna OS | Kunci layar sistem operasi | '*senang* Layar komputer dikunci! [CMD:os:lock]' |
| **Media Musik** | `[CMD:media:play|judul]` | `judul_lagu` | Koleksi Musik / yt-dlp | Putar musik audio lokal (atau unduh otomatis jika belum ada) | '*semangat* Memutar lagunya sekarang! [CMD:media:play&#124;Nemen]' |
| **Media Musik** | `[CMD:media:pause]` | *(Tanpa parameter)* | Pemutar Media | Jeda sementara pemutaran musik | '*senang* Pemutaran musik dijeda! [CMD:media:pause]' |
| **Media Musik** | `[CMD:media:resume]` | *(Tanpa parameter)* | Pemutar Media | Lanjutkan lagu yang sedang dijeda | '*semangat* Melanjutkan putar musik! [CMD:media:resume]' |
| **Media Musik** | `[CMD:media:next]` / `prev` | *(Tanpa parameter)* | Antrean Musik | Putar lagu berikutnya atau kembali ke lagu sebelumnya | '*senang* Ganti ke lagu berikutnya! [CMD:media:next]' |
| **Media Musik** | `[CMD:media:stop]` | *(Tanpa parameter)* | Pemutar Media | Hentikan pemutaran musik sepenuhnya | '*senang* Musik dimatikan! [CMD:media:stop]' |
| **Agenda & Tugas**| `[CMD:todo:add|tugas]` | `deskripsi_tugas` | File Catatan To-Do | Tambah agenda atau tugas baru ke daftar to-do | '*senang* Catatan tugas ditambahkan! [CMD:todo:add&#124;Beli kopi]' |
| **Agenda & Tugas**| `[CMD:todo:done|tugas]` | `kata_kunci_tugas` | File Catatan To-Do | Tandai tugas to-do yang sudah selesai dikerjakan | '*senang* Tugas ditandai selesai! [CMD:todo:done&#124;Beli kopi]' |

TABEL SKILL KUSTOM TERDAFTAR:
{}

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
        // Partikel pengisi / basa-basi yang mutlak dibuang
        let filler_particles = [
            "dong", "ya", "sih", "lah", "nya", "ke", "di", "dan", "yang", "ini", "itu",
            "pada", "tolong", "coba", "kan", "deh", "tuh", "kah",
        ];

        // Kata kerja / label GUI yang biasanya merupakan instruksi, tapi bisa jadi nama elemen di layar
        let action_labels = [
            "buka", "bukakan", "klik", "pencet", "tekan", "video", "tulisan", "teks",
            "tab", "web", "browser", "halaman", "situs", "tombol",
        ];

        let mut results = Vec::new();
        let mut seen = HashSet::new();

        for raw_w in q.split_whitespace() {
            let clean = raw_w.trim_matches(|c: char| !c.is_alphanumeric());
            let clean_lower = clean.to_lowercase();
            if clean.len() >= 2
                && !filler_particles.contains(&clean_lower.as_str())
                && !action_labels.contains(&clean_lower.as_str())
            {
                if seen.insert(clean_lower.clone()) {
                    results.push(clean_lower);
                }
            }

            // Pecah juga sub-token pemisah seperti underscore, dash, atau dot (misal: active_window_app.json)
            for sub in raw_w.split(['_', '-', '.', '/']) {
                let clean_sub = sub.trim_matches(|c: char| !c.is_alphanumeric());
                let sub_lower = clean_sub.to_lowercase();
                if clean_sub.len() >= 2
                    && !filler_particles.contains(&sub_lower.as_str())
                    && !action_labels.contains(&sub_lower.as_str())
                {
                    if seen.insert(sub_lower.clone()) {
                        results.push(sub_lower);
                    }
                }
            }
        }

        // Safety Fallback Cerdas:
        // Jika hasil kosong (misal user hanya berkata "klik Buka", "pilih Tab Video", atau "tekan tombol"),
        // jangan biarkan kosong! Ambil kembali kata non-partikel agar elemen "Buka" atau "Video" tetap ditemukan.
        if results.is_empty() {
            for raw_w in q.split_whitespace() {
                let clean = raw_w.trim_matches(|c: char| !c.is_alphanumeric());
                let clean_lower = clean.to_lowercase();
                if clean.len() >= 2 && !filler_particles.contains(&clean_lower.as_str()) {
                    if seen.insert(clean_lower.clone()) {
                        results.push(clean_lower);
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
                summary.push_str(&format!("* **Tombol di Halaman Web:** {}\n", formatted_btns.join(", ")));
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
                // Saat user minta rangkuman/baca halaman, ambil teks metadata penting (harga, varian, judul, status)
                // hingga 20 item pertama dari DOM texts agar informasi esensial tidak terlewat
                unique_texts.into_iter().take(20).collect()
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
                "* **Isi / Ringkasan Naratif Halaman (Ground of Truth):**\n{}\n",
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
   - JIKA MENJALANKAN AKSI / PERINTAH (selain merangkum, membaca, atau melihat): Jawablah SANGAT SINGKAT (cukup 3 hingga 6 kata saja), to-the-point, jangan mengulang instruksi panjang, langsung jalankan aksi! Ikuti format contoh di kolom 'Contoh Ucapan Pendek Asisten' pada tabel aksi di atas.
   - JIKA MERANGKUM / MEMBACA WEB / MELIHAT / MENJELASKAN PENGETAHUAN: Jelaskan dengan lengkap, jelas, dan informatif sesuai yang diminta.
   - JIKA OBROLAN SANTAI BIASA / KONFIRMASI: Jawab maksimal 1-2 kalimat pendek yang padat dan hangat. DILARANG BERTELE-TELE ATAU PANJANG LEBAR! Dilarang menyusun esai atau paragraf panjang jika tidak diminta secara eksplisit.{skills_prompt}"
    )
}
