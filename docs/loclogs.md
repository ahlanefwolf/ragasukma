# 📜 Peta Lokasi Log Sistem Ragasukma (`loclogs.md`)

Dokumentasi lengkap titik-titik keluaran log (*console output / terminal logs / ring buffer*) di seluruh codebase Ragasukma AI.

---

## 🏛️ Arsitektur Log Terpusat (`term_log!` & `term_err!`)

Seluruh log sistem Ragasukma diatur oleh dua makro terpusat di `src/config.rs`:
1. `term_log!(...)`: Mencetak ke `stdout` (Terminal/CLI) sekaligus memasukkan pesan ke dalam **Ring Buffer Memori (kapasitas 1.000 entri)** dengan level `"info"`.
2. `term_err!(...)`: Mencetak ke `stderr` (Terminal/CLI) sekaligus memasukkan pesan ke dalam **Ring Buffer Memori** dengan level `"error"`.

Buffer memori ini langsung dapat diakses secara live oleh Web UI via:
* `GET /api/logs` (`src/services/logs_service.rs`)
* `POST /api/logs/clear` (Membersihkan buffer log)

---

## 📍 Rincian Peta Lokasi File Log di Codebase

### 1. 🔄 Session & Chat Loop (`src/engine/chat_loop/`)
* **`mod.rs`**:
  * Banner sesi baru: `✨ SESI PERCAKAPAN RAGASUKMA DIMULAI`.
  * Transkrip ucapan pengguna: `\n[User] Kamu [5.00ms]: "..."`.
  * Penghentian sesi: `\n[SELESAI] Sesi obrolan dihentikan.`.
  * Status interupsi suara: `⚡ [Interupsi Diterima] Memproses giliran bicara baru...`.
  * Suara hening: `[INFO] (Suara tidak terdengar jelas / terlalu pelan)`.
* **`llm_responder.rs`**:
  * Mulai berpikir: `\n[LLM] <Asisten> sedang merespons...`.
  * Token streaming kalimat LLM ke stdout.
  * Teks respon utuh: `[<Asisten>] "..."`.
  * Deteksi 8 emosi kanonikal (`happy`, `shy`, `excited`, `surprised`, `thinking`, `sad`, `angry`, `neutral`) via `detect_emotion`.
  * Error jaringan / LLM: `⚠️ [LLM Error]: ...`.
* **`input_listener.rs`**:
  * Status jeda: `\n[DIJEDA] <Asisten> berhenti mendengarkan sementara.`.
  * Status lanjut: `▶️ [DILANJUTKAN] <Asisten> siap mendengarkan lagi!`.
  * Reset memori: `🔄 [Reset Session] Riwayat memori obrolan AI berhasil di-reset!`.
* **`skill_runner.rs`**:
  * Eksekusi skill: `\n[MEMPROSES SKILL]: Memanggil ID '...'...`.
  * Status sukses: `[SKILL BERHASIL: <Name> (<ID>)] -> <Output>`.
  * Status gagal: `[SKILL GAGAL]: Identifier '...' tidak cocok di skill-command.md`.
  * Evaluasi agentic: `🔄 [Agentic Loop Step 2]: Menjalankan evaluasi lanjutan...`.
  * Fallback terminal: `\n[SMART FALLBACK]: Terdeteksi permintaan buka terminal.`.

---

### 2. ⚙️ Session Manager & Reactive Config (`src/engine/session.rs`)
* Inisialisasi asisten: `🎀 Karakter <Asisten> (Imut & Pemalu) siap menemani <User>.`.
* Inisialisasi TTS: `🔊 Engine TTS: Google Chrome Web Speech API siap.`.
* Reactive update voice interruption: `🔄 [Reactive Update] Voice Interruption diupdate realtime!`.
* Reactive update STT: `🔄 [Reactive Update] STT Lang dialihkan ke: <lang>`.
* Reactive update TTS & Gender: `🔄 [Reactive Update] Voice / Gender (<gender>) diperbarui.`.
* Reactive update LLM: `🔄 [Reactive Update] LLM Provider / Model / Persona dialihkan ke: ...`.
* Shutdown aplikasi: `\n👋 Program selesai. Sampai jumpa!`.

---

### 3. 🌐 Web Speech, STT Watchdog & Remote Ingestion (`src/services/remote_service.rs` & `voiceSession.ts`)
* Transkrip STT dari browser Chrome: `🎙️ [Chrome Web STT] Transkrip diterima: "..."`.
* Peringatan loop nonaktif: `⚠️ [Chrome Web STT] Chat loop belum berjalan aktif!`.
* Input teks dari smartphone: `📱 [Mobile Remote] Menerima teks dari HP: "..."`.
* STT Watchdog Timeout & Fallback (Browser):
  * `[Chrome Web STT] Reset listening ke idle (watchdog timeout 3.5s)`
  * `[Chrome Web STT] Reset listening ke idle (speech ended without transcript)`
  * `[Chrome Web STT] Reset listening ke idle (no speech match)`
* Remote Audio Routing: `🔊 [Remote Audio Sync] Mode: Remote Desktop vs Full Mobile`.

---

### 4. 🍃 VRM SpringBone & 3D Renderer (`src/ui/spring_bone.rs` & `app.rs`)
* Inisialisasi pegas Verlet Physics: `🍃 [SpringBone] Diinisialisasi: X joints pegas dinamis, Y colliders tabrakan`.
* Watchdog fail-safe desktop: otomatis reset visual state `Listening -> Idle` jika waktu listening > 4.5 detik.

---

### 4. 🧠 LLM Client & Providers (`src/llm/`)
* **`client.rs`**:
  * Mode streaming: `🌊 Menggunakan Streaming Response untuk kecepatan maksimal...`.
  * Mode non-streaming: `⚡ Menggunakan Non-Streaming Response (Fallback).`.
  * Error HTTP/Request: `❌ [LLM Client] Request error: ...`.
* **`providers/antigravity.rs`**:
  * Status auth & rotasi akun AI Antigravity: `✨ [Antigravity] Initialized provider with X account(s) loaded.`.
  * Error pemanggilan Antigravity API.
* **`providers/openai.rs` / `anthropic.rs` / `gemini.rs`**:
  * Status inisialisasi API key & payload streaming error.

---

### 5. 🛠️ Action Router & Desktop Automation (`src/engine/actions/`)
* **`mod.rs`**: Log router pemetaan identifier ke skill.
* **`desktop.rs`**:
  * Pindah jendela: `🪟 [Desktop Focus] Berpindah ke jendela: ...`.
  * Klik mouse & ketik keyboard: `🖱️ [Desktop Click] Klik mouse di (x, y)`.
  * Error OS API: `❌ [Desktop Click] Gagal simulasi klik mouse di Linux/Windows`.
* **`browser.rs`**:
  * Navigasi tab: `🌐 [Browser Action] Membuka URL: ...`.
  * Switch tab / search: `🔍 [Browser Action] Mencari: ...`.
* **`media.rs` / `ytdlp.rs`**:
  * Download audio YouTube: `📥 [YT-DLP] Mengunduh audio: ...`.
  * Error download atau konversi MP3.

---

### 6. 🎵 Audio Player (`src/audio/player.rs`)
* Pemutaran musik: `🎵 [Music Controller] Memutar trek '<Judul>' via Web Dashboard & PiP stream`.
* Kontrol playback:
  * `⏸️ [Music Controller] Musik dijeda.`
  * `▶️ [Music Controller] Musik dilanjutkan.`
  * `⏹️ [Music Controller] Musik dihentikan.`

---

### 7. ☁️ Cloudflared Tunnel Worker (`src/engine/cloudflared.rs`)
* Status tunnel Cloudflare:
  * `🌐 [Cloudflared] Menjalankan tunnel untuk remote Web UI...`.
  * `🔗 [Cloudflared] URL Publik Aktif: https://...trycloudflare.com`.
  * `🛑 [Cloudflared] Tunnel dihentikan.`.

---

### 8. 🌐 HTTP Server & API Dispatcher (`src/server.rs`)
* Server listen port: `🚀 [Server] Dashboard HTTP Server berjalan di http://127.0.0.1:42128`.
* File config fallback: `⚠️ [Config Load] File config-ai.json sedang dikunci/tidak terbaca, menggunakan fallback memori.` (`src/config.rs`).
