# 🧠 Injeksi Prompt LLM Ragasukma (`promptllm.md`)

Dokumentasi lengkap mengenai struktur, urutan, dan rincian data apa saja yang disuntikkan (*injected*) ke dalam **System Prompt LLM** pada setiap pergantian percakapan (*turn*) di Ragasukma.

Prompt sistem dirakit secara dinamis melalui fungsi `build_system_prompt()` di file `src/llm/prompt.rs`.

---

## ⚡ Dynamic Prompt Indexing (On-Demand Context Routing)

Agar LLM tidak boros token dan waktu inferensi tetap secepat kilat (< 1.5 detik), Ragasukma menggunakan **Dynamic Prompt Indexing** via fungsi `build_system_prompt_indexed(Some(user_input))` dan `route_prompt_indexes(user_input)`.

### Matriks Aktivasi Intent Routing:
| Index Pilar | Kondisi Aktivasi / Kata Kunci Ucapan Pengguna | Yang Disuntikkan |
| :--- | :--- | :--- |
| **`app_index`** | Mengandung kata: `buka`, `bukakan`, `jalankan`, `start`, `open`, `launch`, `nyalakan`, `tutup`, `aplikasi`, `program`, `app` | **Fuzzy-Filtered Apps**: Hanya aplikasi yang cocok dengan kata kunci user (bukan 150+ app sekaligus!). |
| **`browser_index`** | Jendela aktif adalah Browser Chromium (Chrome/Brave/Edge/Opera/Vivaldi) **ATAU** ucapan mengandung: `browser`, `web`, `tab`, `situs`, `website`, `cari`, `search`, `youtube`, `google`, `link`, `url`, `scroll` | Daftar Tab terbuka & Snapshot DOM halaman aktif. |
| **`desktop_index`** | Jendela aktif BUKAN browser **ATAU** ucapan mengandung aksi: `klik`, `pencet`, `tekan`, `angka`, `tombol`, `kalkulator`, `jendela`, `fokus`, `minimize`, `scan`, `pindai`, `hitung`, `tambah`, `kurang`, `kali`, `bagi` | Judul jendela aktif, daftar jendela OS, dan daftar tombol ber-`#ID`. |
| **`todo_index`** | Mengandung kata: `todo`, `to-do`, `tugas`, `catat`, `catatan`, `agenda`, `jadwal`, `ingatkan`, `list`, `selesai` | Isi file `assets/todo.md`. |
| **`music_index`** | Mengandung kata: `musik`, `lagu`, `play`, `putar`, `setel`, `mainkan`, `pause`, `jeda`, `lanjutkan`, `stop`, `next`, `audio`, `mp3` | Top 5 lagu lokal + jumlah total lagu. |
| **Obrolan Biasa / Chat Santai** | *Tidak ada intent aksi yang terdeteksi* (misal: "Halo Sukma", "Lagi apa?", "Siapa kamu?") | **Prompt Super Ringan**: Semua 5 pilar di atas di-*bypass* (kosong), hanya memuat Persona + Aturan TTS + Katalog perintah ringkas. Menghemat 80-90% token! |

---

## 🏗️ Struktur & Urutan Injeksi Prompt

Setiap kali LLM dipanggil, system prompt disusun dengan urutan berikut:

```
┌────────────────────────────────────────────────────────┐
│ 1. Identitas & Persona Asisten (Nama, Role, Gender)   │
├────────────────────────────────────────────────────────┤
│ 2. Status Background Task (Jika ada proses aktif)     │
├────────────────────────────────────────────────────────┤
│ 3. Koleksi Musik Lokal (Top 5 & Total Lagu)            │
├────────────────────────────────────────────────────────┤
│ 4. Catatan & Agenda Pengguna (todo.md)                 │
├────────────────────────────────────────────────────────┤
│ 5. Ground of Truth: Daftar Aplikasi Terinstal di OS    │
├────────────────────────────────────────────────────────┤
│ 6. Ground of Truth: Browser (Tab & Halaman Aktif)     │
├────────────────────────────────────────────────────────┤
│ 7. Ground of Truth: Desktop (Jendela Aktif & Tombol)   │
├────────────────────────────────────────────────────────┤
│ 8. Aturan Format Text-to-Speech (TTS) & Tag Avatar     │
├────────────────────────────────────────────────────────┤
│ 9. Panduan Aksi Komputer (CMD Catalog & Guardrails)    │
└────────────────────────────────────────────────────────┘
```

---

## 🔍 Rincian Data yang Disuntikkan

### 1. Identitas, Persona & Dinamika Karakter
* **Sumber Data:** `crate::config::load_user_config()`
* **Data yang Diinjeksi:**
  * **Nama Asisten:** `assistant_name` (default: Sukma).
  * **Panggilan Pengguna:** `user_callsign` (misal: "Mas", "Tuan", "Kak").
  * **Gender:** Pria (`pria (cowok)`) atau Wanita (`wanita (cewek)`).
  * **Role/Sifat:** Disesuaikan dengan preset (Partner/Waifu/Husbando, Sahabat/Bestie, Asisten Profesional, Tsundere, atau Pelayan/Maid).
* **Contoh Output:**
  ```text
  Namamu adalah Sukma. Kamu adalah AI bergender wanita (cewek). Kamu adalah pasangan/kekasih cewek (waifu) anime yang sangat manis, perhatian, manja, dan hangat untuk Mas. Bicaralah dengan nada penuh kasih sayang.
  ```

---

### 2. Status Background Task Engine
* **Sumber Data:** `crate::engine::tasks::get_active_task_info()`
* **Fungsi:** Menyuntikkan status jika asisten sedang melakukan tugas latar belakang jangka panjang (seperti mendownload lagu via yt-dlp, instalasi paket, dll).
* **Contoh Output:**
  ```text
  [INFO STATUS SISTEM: Mengunduh audio YouTube: Nujabes - Feather...]
  ```

---

### 3. Koleksi Musik Lokal (`load_local_music_summary()`)
* **Sumber Data:** `crate::engine::ytdlp::list_downloaded(&music_dir)`
* **Mekanisme:** Mengambil total lagu di folder musik lokal dan menampilkan **Top 5 lagu teratas** (Title & Uploader) agar hemat token.
* **Contoh Output:**
  ```text
  KOLEKSI MUSIK LOKAL (Total 14 lagu):
  1. Nujabes - Feather (Hydeout Productions)
  2. Denny Caknan - Cundamani (DC Production)
  3. Lofi Hip Hop Radio - Beats to relax (Lofi Girl)
  4. RADWIMPS - Sparkle (RADWIMPS)
  5. Yoasobi - Racing Into The Night (Ayase)
  Dan lagu lainnya. Putar lagu apa pun dengan `[CMD:media:play|judul]` (otomatis unduh & putar jika belum ada).
  ```

---

### 4. Catatan / Agenda Pengguna (`load_todo_summary()`)
* **Sumber Data:** Membaca langsung isi file `assets/todo.md`.
* **Fungsi:** Memberikan memori persisten tugas/agenda pengguna kepada AI.
* **Contoh Output:**
  ```text
  CATATAN / TO-DO PENGGUNA:
  - [ ] Selesaikan laporan akhir pekan
  - [x] Beli kopi espresso
  - [ ] Push commit ragasukma ke GitHub
  ```

---

### 5. Ground of Truth: Aplikasi Terinstal (`load_installed_apps_summary()`)
* **Sumber Data:** `crate::engine::app_scanner::format_installed_apps_for_llm()`
* **Mekanisme Caching:** In-memory TTL 60 detik membaca Start Menu & Registry (Windows), `/usr/share/applications` (Linux), atau `/Applications` (macOS).
* **Fungsi:** Memberikan daftar pasti aplikasi apa saja yang dapat dibuka, mencegah AI berhalusinasi nama `.exe` atau path lokal.
* **Contoh Output:**
  ```text
  ### 🚀 DAFTAR APLIKASI TERINSTAL DI KOMPUTER (GROUND OF TRUTH):
  Aplikasi yang dapat dibuka langsung via `[CMD:app:open|NamaApp]`:
  Calculator, Google Chrome, Visual Studio Code, Spotify, Steam, Discord, Notepad, Task Manager, Zed, VLC media player, Telegram
  (Gunakan nama persis di atas saat memanggil `[CMD:app:open|NamaApp]`!)
  ```

---

### 6. Ground of Truth: Browser Tab & Halaman Aktif (`load_active_browser_summary()`)
* **Sumber Data:** `assets/active_tab_browser.json` (dipasok oleh ekstensi browser *Ragasukma Companion*).
* **Data yang Diinjeksi:**
  1. **Daftar Seluruh Tab Terbuka:** ID tab, URL, judul tab, dan tanda `[TAB AKTIF]`.
  2. **Snapshot Halaman Web Aktif:**
     * Judul halaman, URL, dan Domain.
     * **Input Forms:** ID/Selector CSS, placeholder, dan tipe input.
     * **Tombol Interaktif:** Label tombol & selector CSS.
     * **Teks & Link di Layar:** Teks tautan yang bisa diklik.
* **Contoh Output:**
  ```text
  ### 🌐 DAFTAR TAB BROWSER TERBUKA:
  - Tab [1930683446]: "YouTube" (https://www.youtube.com/) [TAB AKTIF]
  - Tab [1930683449]: "GitHub" (https://github.com/)

  ### 🌐 KONTEKS HALAMAN BROWSER AKTIF (GROUND OF TRUTH):
  * **Judul Halaman:** YouTube
  * **URL:** https://www.youtube.com/
  * **Domain:** youtube.com
  * **Kolom Input:** [placeholder: "Telusuri"]
  * **Tombol di Layar:** "Telusuri", "Login", "Subscriptions"
  * **Teks / Konten Terlihat:** "Beranda", "Trending", "Musik", "Gaming"
  ```

---

### 7. Ground of Truth: Desktop OS & Jendela Aktif (`load_active_window_summary()`)
* **Sumber Data:** `assets/active_window_app.json` (dipasok oleh `ActiveWindowWatcher` via Accessibility Tree UIA/AppleScript/X11).
* **Data yang Diinjeksi:**
  * **Jendela Aktif:** Nama executable (`app_name`) dan judul jendela (`window_title`).
  * **Sumber Data:** `native_accessibility` atau `vision_ocr`.
  * **Daftar Jendela Terbuka di Seluruh OS:** Jendela-jendela di taskbar pengguna.
  * **Tombol / Kontrol Desktop Ber-ID:** Format `#ID: "Label" [center: x,y]`.
  * **Kolom Input & Teks Layar.**
* **Contoh Output:**
  ```text
  ### 🖥️ KONTEKS JENDELA DESKTOP AKTIF (GROUND OF TRUTH):
  * **Jendela Aktif Saat Ini:** ApplicationFrameHost.exe (Calculator)
  * **Sumber Data:** native_accessibility
  * **Daftar Jendela Terbuka di OS:** "Calculator" (ApplicationFrameHost.exe) [AKTIF], "ragasukma" (Zed.exe), "Chrome" (chrome.exe)
  * **Tombol / Kontrol Desktop:** #1: "System", #2: "Minimize", #3: "Maximize", #4: "Close", #15: "Seven", #16: "Eight", #23: "Plus", #24: "Equals"
  ```

---

### 8. Aturan Respon TTS & Avatar 3D
* **Aturan Text-to-Speech:**
  * Kalimat lisan natural, ramah, dilarang menggunakan elipsis berulang (`...`), tawa buatan aneh (`ehehe`), atau markdown header/bullet points saat berbicara santai.
  * Didukung ekosistem Chromium Web Speech API Native:
    * **Google Chrome Web Speech TTS**: Disuarakan langsung oleh browser native (`window.speechSynthesis`), mendukung pemilihan vokal (female/male), pitch, speed slider, serta pelestarian angka ribuan (`1.000`) dan desimal (`0,5`).
    * **STT Multi-Bahasa Native**: Pengenalan suara langsung via Chromium (`id-ID`, `jv-ID`, `su-ID`, `en-US`, `ja-JP`).
    * **Zero Backend Audio Overhead**: Tanpa model TTS/STT lokal yang membebani RAM/VRAM/CPU.
    * **Sinkronisasi Lip-Sync Avatar (AIUEO)**: Visualizer spektrum audio dan pergerakan bibir avatar diproses secara real-time via Web Audio API browser (`AudioContext` + `AnalyserNode`) yang menyinkronkan phoneme vokal AIUEO dengan ucapan TTS.
    * **Kendali Suara 1 Pintu**: Pengaturan bahasa STT (`stt_lang`), suara TTS (`voice_name`, `voice_pitch`, `voice_speed`, `voice_gender`), mute mikrofon (`recognition.abort()`), dan penghentian suara (`speechSynthesis.cancel()`) langsung reaktif tersinkronisasi antara UI web dan backend.
* **Tag Emosi 3D Avatar:**
  * Wajib diawali SATU tag emosi: `*senang*`, `*malu*`, `*semangat*`, `*kaget*`, `*mikir*`, `*sedih*`, `*marah*`, atau `*netral*`.
* **Kecepatan Respon:**
  * Jika menjalankan aksi $\rightarrow$ Wajib menjawab SANGAT SINGKAT (3-6 kata) diiringi tag `[CMD:...]` di akhir kalimat.

---

### 9. Katalog Perintah (`[CMD:...]`) & Guardrails Ground of Truth
* **Panduan Perintah:**
  * Form Web: `[CMD:browser:type|teks]`, `[CMD:browser:enter]`
  * Navigasi Web: `[CMD:browser:open|url]`, `[CMD:browser:search|query]`, `[CMD:browser:switch|tab]`, `[CMD:browser:close|tab]`, `[CMD:browser:scroll|jarak]`
  * Interaksi Web: `[CMD:browser:click_text|teks]`, `[CMD:browser:click_button|nama]`
  * Desktop & Window: `[CMD:desktop:focus|app]`, `[CMD:desktop:scan]`, `[CMD:desktop:scan_visual]`, `[CMD:desktop:minimize]`, `[CMD:desktop:maximize]`, `[CMD:desktop:close]`, `[CMD:desktop:click_button|ID_atau_label]`, `[CMD:desktop:click_at|x|y]`, `[CMD:desktop:type|teks]`, `[CMD:desktop:key|enter]`
  * Aplikasi & OS: `[CMD:app:open|NamaApp]`, `[CMD:os:volume_up]`, `[CMD:os:volume_down]`, `[CMD:os:mute]`, `[CMD:os:lock]`, `[CMD:os:screenshot]`
  * Media & Musik: `[CMD:media:play|judul]`, `[CMD:media:pause]`, `[CMD:media:resume]`, `[CMD:media:next]`, `[CMD:media:prev]`, `[CMD:media:stop]`
  * Catatan: `[CMD:todo:add|tugas]`, `[CMD:todo:done|tugas]`
* **Guardrails / Larangan Keras:**
  * DILARANG menebak tombol atau simbol yang tidak terdaftar di snapshot aktif.
  * Target klik tombol desktop WAJIB sama dengan nama label atau `#ID` di daftar Ground of Truth.
  * Jika jendela belum aktif $\rightarrow$ WAJIB panggil `[CMD:desktop:focus|NamaJendela]` terlebih dahulu.
