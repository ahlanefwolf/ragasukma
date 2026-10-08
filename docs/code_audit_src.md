# Code Audit: Memory Leak, Race Condition, Dead Code, dan Redundant Code

Dokumen ini berisi hasil audit menyeluruh terhadap kode sumber di `src/` terkait:
1. Indikasi Memory Leak / Unbounded Memory Growth
2. Potensi Race Condition & Concurrency Issue
3. Dead Code (Unused items)
4. Redundant Code (Duplikasi / Kode Tidak Efisien)

---

## 1. Analisis Memory Leak & Growth

| Lokasi | Jenis / Pola | Analisis & Status | Rekomendasi Mitigasi |
|---|---|---|---|
| `src/audio/player.rs:192` | `Box::leak` | Digunakan untuk mempertahankan `rodio::OutputStream` agar tidak drop selama aplikasi hidup dalam singleton `AUDIO_HANDLE` (`OnceLock`). Bukan kebocoran berulang (*controlled singleton leak*). | Biarkan tetap seperti ini karena stream audio OS memang harus berumur `'static`. |
| `src/audio/supertonic.rs:494-515` | Global Audio Queue | `LAST_AUDIO_WAV_LIST` dan `LAST_AUDIO_OPUS_LIST` dibatasi maksimum 3 entri (`while list.len() >= 3 { list.pop_front(); }`). Tidak ada pertumbuhan unbounded. | Aman / Tidak ada leak. |
| `src/app_state.rs:353` | Action Buffer | `CLIENT_ACTIONS_BUFFER` otomatis membuang antrean lama jika sudah mencapai 50 entri (`pop_front()`). | Aman / Terkontrol. |
| `src/config.rs:33` | In-memory Log Buffer | `LOG_BUFFER` dibatasi maksimal 150 baris (`buf.pop_front()`). | Aman / Terkontrol. |
| `src/llm/providers/antigravity/v1_adapter.rs:88` | Signature Cache | `THOUGHT_SIGNATURE_STORE` menggunakan batas kapasitas 5000 entri dan pembersihan berbasis TTL (2 jam). | Aman / Terkontrol. |
| `src/services/browser_session_service.rs:353` | Browser Command Queue | `PENDING_BROWSER_COMMANDS` dibatasi maksimal 20 entri (`lock.pop_front()`). | Aman / Terkontrol. |
| `src/audio/tts.rs:160` | Dynamic Thread Spawning | Setiap kali sintesis full WAV selesai, di-spawn `std::thread::spawn` baru untuk encode Opus cadangan tanpa threadpool/rate-limiter. | Gunakan threadpool atau worker queue statis jika request TTS berurutan tinggi untuk menghindari lonjakan alokasi stack thread OS. |

---

## 2. Analisis Race Condition & Concurrency Issues

### 2.1 Lost Update pada File `todo.md`
* **File**: `src/engine/todo.rs:110-148`
* **Gejala**: `add_task` dan `complete_task` membaca file dengan `fs::read_to_string`, memodifikasi baris di memori, lalu menulis kembali dengan `fs::write` tanpa sinkronisasi mutex atau file locking.
* **Risiko**: Jika dua instruksi (misalnya dari Web UI REST API dan tool call LLM/voice agent) memodifikasi daftar todo secara hampir bersamaan, instruksi yang selesai terakhir akan menimpa seluruh file, menghilangkan task dari instruksi pertama (*lost update*).
* **Solusi**: Tambahkan mutex sinkronisasi internal di `todo.rs` (seperti `CONFIG_MUTEX` di `config.rs`) saat membaca dan menulis `todo.md`.

### 2.2 Eksekusi Recipe Tidak Terkoordinasi (Concurrent Recipe Collision)
* **File**: `src/engine/actions/recipe/runner.rs:583` dan `src/engine/actions/recipe/trainer.rs:364`
* **Gejala**: Baik `execute_recipe` maupun `record_recipe_action` men-spawn thread baru secara lepas (`thread::spawn(move || { ... })`) tanpa status flag apakah ada otomasi browser yang sedang berjalan.
* **Risiko**: Jika pengguna memicu runner saat recipe lain atau trainer sedang berjalan, kedua thread akan berebut mengirim command ke ekstensi browser (`smart_open_or_switch_tab`, `send_browser_command`), mengakibatkan tab browser berpindah-pindah liar dan input teks tertukar.
* **Solusi**: Sediakan atomic state `IS_RECIPE_RUNNING: AtomicBool` untuk menolak atau mengantre eksekusi recipe baru saat recipe sebelumnya belum selesai.

### 2.3 Dual Sync Fallback pada Browser Snapshot
* **File**: `src/services/browser_session_service.rs:338-340`
* **Gejala**: Jika worker channel penuh, fungsi `save_browser_snapshot` melakukan synchronous fallback dengan menulis langsung ke disk dan lock `LAST_ACTIVE_SNAPSHOT`.
* **Risiko**: Thread worker di latar belakang (`browser_worker_loop`) bisa sedang menulis disk dari antrean sebelumnya, menyebabkan operasi I/O tumpang-tindih dan urutan snapshot di disk menjadi tidak deterministik.

---

## 3. Analisis Dead Code

1. **Struct / Fields Tidak Terpakai yang Ditekan dengan `#[allow(dead_code)]`**:
   * `src/config.rs:288`: `AppConfig` tidak dipakai aktif (digantikan oleh arsitektur `UserConfig`).
   * `src/llm/providers/antigravity/oauth.rs:9, 19`: `OAuthTokenResponse` dan `UserInfo` memiliki atribut `#[allow(dead_code)]`.
2. **Re-export Unused Imports**:
   * `src/llm/mod.rs:10-21`: Banyak item re-export (`clean_ai_text`, `build_system_prompt_indexed`, `V1Response`, dll.) diberi tag `#[allow(unused_imports)]` dan tidak digunakan oleh module consumer.
   * `src/llm/providers/antigravity/mod.rs:9-17`: `constants::*`, `ensure_project_id`, `fetch_account_quota`, `TokenResponse`, dan `execute_v1_stream` di-reexport dengan `#[allow(unused_imports)]`.
3. **Komponen OCR Spesifik OS**:
   * `src/engine/perception/ocr_macos.rs` dan `src/engine/perception/ocr_windows.rs`: Terdapat implementasi OCR native per-OS yang belum terintegrasi penuh atau di-bypass oleh CLI fallback cross-platform.

---

## 4. Analisis Redundant Code (Duplikasi / Tidak Efisien)

1. **Duplikasi Kompilasi Regex di `cloudflared.rs`**:
   * **File**: `src/engine/cloudflared.rs:348` & `src/engine/cloudflared.rs:370`
   * **Masalah**: Closure reader `stderr` dan `stdout` masing-masing mendeklarasikan `static URL_REGEX: std::sync::LazyLock<regex::Regex>` identik secara terpisah.
   * **Rekomendasi**: Satukan deklarasi regex ke level modul/file.
2. **Double File Writing pada Browser Snapshot**:
   * **File**: `src/services/browser_session_service.rs:124-131`
   * **Masalah**: `write_snapshot_to_disk` menulis snapshot ke `get_active_tab_browser_path()`. Jika path tersebut tidak sama dengan `"assets/active_tab_browser.json"`, file yang sama persis ditulis ulang untuk kedua kalinya pada setiap update tab.
3. **Duplikasi Logika Parsing Parameter Recipe**:
   * **File**: `src/engine/actions/recipe/runner.rs`
   * **Masalah**: Pengecekan parameter mandatory dan normalisasi format argumen string pipelined (`id|json`) vs JSON murni diulang-ulang di beberapa titik fungsi runner.
