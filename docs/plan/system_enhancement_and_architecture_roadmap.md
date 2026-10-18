# Rencana Peningkatan Sistem & Roadmap Arsitektur: Ragasukma

Dokumen ini merangkum hasil evaluasi arsitektur sistem saat ini, analisis kesenjangan (*gap analysis*) terhadap klaim `README.md`, serta rencana langkah-langkah teknis konkret untuk menyempurnakan Ragasukma menjadi sistem asisten AI desktop yang benar-benar solid, mandiri, dan berlatensi rendah.

---

## 1. Ringkasan Eksekutif & Status Sistem

### 1.1. Komponen yang Sudah Terbukti Solid
- **Supertonic 3 & OpenVoice V2 (Pure ONNX di Rust)**:
  - Inferensi neural TTS berjalan murni di CPU tanpa runtime Python via `ort`.
  - Dukungan 31 bahasa, voice morphing (Slerp blending), dan ekstraksi forman frekuensi untuk sinkronisasi blendshape bibir AIUEO.
- **3D VRM Native Overlay (WGPU + MToon)**:
  - Rendering cel-shaded MToon langsung di WGPU/WGSL dengan *Verlet Spring Bone physics* (VRM 0.x dan VRM 1.0) dengan hit collider.
- **Multi-Account OAuth Pooling (Antigravity & Codex)**:
  - Rotasi akun otomatis, refresh token latar belakang, dan quota failover.
- **OS Native Integration**:
  - `mimalloc`, DPI Awareness V2 di Windows, single-instance handshake protection, dan accessibility scanner (`atspi` / Windows UI Automation).

### 1.2. Masalah & Kesenjangan Kritis yang Ditemukan
1. **Paradoks Ketergantungan STT**: Fitur input suara (Speech-to-Text) saat ini **100% bergantung pada Chrome Web Speech API** di browser (`web/src/services/voiceSession.ts`). Jika browser ditutup atau offline, asisten tidak bisa mendengar suara pengguna sama sekali.
2. **Latensi Tinggi pada Suara Respon**: Pemanggilan `tts.speak_with_emotion` baru dieksekusi **setelah seluruh respon LLM selesai dibuat**, menyebabkan jeda respon suara 5–8 detik.
3. **Streaming Palsu pada `/v1/chat/completions`**: Request OpenAI streaming diarahkan ke `handle_axum_fallback` yang menampung seluruh teks ke `VirtualHttpStream` (RAM buffer). Akibatnya, Cursor/Zed/Cline tidak menerima token per token secara real-time.
4. **Alur Pseudo-Agentic**: Eksekusi tool/skill menyuntikkan teks buatan ke antrean input user (`inject_remote_text`), mencemari riwayat percakapan.
5. **CORS & Tunnel Tanpa Autentikasi**: Port `42128` dan tunnel `cloudflared` terbuka tanpa token proteksi, membuka celah eksekusi aksi OS dari situs web lain via fetch lokal (CSRF).
6. **Keterikatan Wajib GUI**: Program langsung berhenti jika inisialisasi window WGPU gagal, sehingga tidak bisa dijalankan sebagai daemon API headless.

---

## 2. Roadmap Perbaikan & Arsitektur Solusi

```
┌────────────────────────────────────────────────────────────────────────┐
│                        RAGASUKMA DESKTOP RUNTIME                       │
├────────────────────────────────────────────────────────────────────────┤
│  [Audio Input Subsystem]            [Agentic Core & Orchestrator]      │
│  - cpal Native Capture (Rust)  ──►  - Sentence-level Chunking          │
│  - Silero-VAD ONNX (Barge-in)       - ReAct Multi-Step Tool Loop       │
│  - Sherpa/Whisper-ONNX STT          - History Manager (Token Pruning)  │
│                                                     │                  │
│                                                     ▼                  │
│  [Speech Output Subsystem]          [API & Network Gateway (Axum)]     │
│  - Supertonic-3 Streaming TTS  ◄──  - Real SSE /v1/chat/completions    │
│  - Native Formant Lip-Sync          - Local Auth Token Protection      │
│  - Rodio Chunk Streaming            - Headless/Daemon Mode Flag        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Detail Rencana Implementasi

### FASE 1: Optimasi Latensi & Real-Time Streaming (Prioritas P0) [SELESAI]

#### 1.1. Sentence-Level Streaming TTS [SELESAI]
* **Status**: **Selesai Diimplementasikan**.
* **File Terkait**:
  - `src/audio/tts.rs`: Implementasi `StreamingVoiceSession`, `StreamingVoiceMsg`, dan `TextToSpeech::start_streaming_session`.
  - `src/engine/chat_loop/llm_responder.rs`: Terhubung langsung ke `voice_session.push_sentence(&clean_sentence, emotion)` pada setiap chunk kalimat LLM, `voice_session.abort()` untuk barge-in/interupsi seketika, dan `voice_session.finish()` saat LLM selesai.
* **Hasil**:
  - *Time-to-first-voice* terpangkas drastis dari 5–8 detik menjadi < 1 detik.
  - Sintesis neural Supertonic pada kalimat #1 berjalan simultan saat token kalimat #2 sedang di-stream oleh LLM.
  - Tidak ada lagi duplikasi audio atau blocking sinkron di akhir respon.

#### 1.2. Native Axum Server-Sent Events (SSE) untuk `/v1/chat/completions` [SELESAI]
* **Status**: **Selesai Diimplementasikan**.
* **File Terkait**:
  - `src/services/openai_v1_service.rs`: `axum_v1_chat_completions_handler`, `axum_v1_models_handler`, `axum_v1_options_handler`, dan `axum_v1_root_handler`.
  - `src/server.rs`: Pendaftaran route langsung pada `axum::Router` untuk `/v1/chat/completions`, `/api/v1/chat/completions`, `/v1/completions`, `/api/v1/completions`, `/v1/models`, dan `/v1`.
* **Hasil**:
  - Streaming SSE murni tanpa buffering memori (`VirtualHttpStream`) ke IDE eksternal (Cursor, Zed, Cline).
  - Menggunakan `tokio::sync::mpsc::channel` dan `async_stream::stream!` dengan header `text/event-stream; charset=utf-8` serta `X-Accel-Buffering: no`.
  - Mendeteksi *client disconnect* secara reaktif; jika user menekan tombol "Stop" di Cursor/Zed, proses streaming LLM langsung dihentikan untuk menghemat kuota.

#### 1.3. Eliminasi Total VirtualHttpStream & Migrasi Native Axum Router [SELESAI]
* **Status**: **Selesai Diimplementasikan**.
* **File Terkait**:
  - `src/services/api_router.rs`: Implementasi `register_api_routes` untuk ~50 endpoint `/api/*` secara murni asinkron.
  - `src/server.rs`: Penghapusan `VirtualHttpResponse`, `VirtualHttpStream`, dan `handle_api_route` (>500 baris boilerplate lama terhapus).
  - `src/services/llm_providers/codex_service.rs`: Migrasi listener OAuth port 1455 ke Tokio + Axum.
* **Hasil**:
  - Eliminasi 3x overhead re-formatting dan parsing string HTTP/1.1 per request.
  - Bypassing alokasi `tokio::task::spawn_blocking` pada setiap request API web dashboard.
  - Zero-overhead static asset serving langsung dari memori/disk.
  - Seluruh 57 unit tests lulus verifikasi.

#### 1.4. Lazy On-Demand AT-SPI D-Bus Connection & Idle Timeout Cleanup [SELESAI]
* **Masalah**: Pada Linux, koneksi D-Bus AT-SPI (`atspi` / `zbus`) selalu diaktifkan sejak awal startup melalui `start_window_watcher()`, memicu ~12 thread persisten (`zbus::Connection`, `async-io`, `blocking-1`) meskipun pengguna tidak sedang menjalankan automasi UI.
* **Solusi Arsitektur**:
  - `start_window_watcher()` beralih menggunakan `scan_window_list_quick_linux()`: hanya membaca pohon jendela Sway/i3/X11 secara instan tanpa menyentuh D-Bus atau AT-SPI sama sekali.
  - Koneksi AT-SPI (`get_or_init_atspi_conn()`) diubah menjadi murni **lazy on-demand**: baru diinisialisasi saat aksi automasi desktop (misal `click_desktop_button`) benar-benar membutuhkan pencarian tombol/elemen form.
  - Implementasi **Idle Timeout (2 menit)**: `cleanup_idle_atspi_connection()` otomatis mendrop koneksi D-Bus jika tidak ada perintah automasi baru selama 120 detik, sehingga ke-12 thread AT-SPI ditutup dan memori dibebaskan.
* **Hasil**: Saat idle, Ragasukma menghemat 12 thread OS. Seluruh 65 unit tests lulus verifikasi.

---

### FASE 2: Kemandirian Penuh Audio Native (Prioritas P1)

#### 2.1. Native Audio Capture & Silero-VAD di Rust
* **Tujuan**: Mengeliminasi ketergantungan mikrofon pada browser dashboard.
* **Teknologi**: Crate `cpal` + `silero-vad` (ONNX via `ort`).
* **Langkah**:
  1. Tambahkan worker thread `NativeAudioInputWorker` menggunakan `cpal::traits::StreamTrait`.
  2. Alirkan buffer PCM 16kHz mono ke Silero-VAD ONNX model (~2 MB).
  3. Saat VAD mendeteksi sinyal bicara manusia di atas ambang batas:
     - Jika asisten sedang berbicara (`is_ai_speaking == true`), langsung panggil `stop_native_voice()` (Instant Barge-in).
     - Kumpulkan frame suara hingga hening terdeteksi (silence threshold 500–800ms).

#### 2.2. On-Device Speech-to-Text (STT) Engine
* **Pilihan Model**:
  - **Opsi A (Paling Ringan & Cepat)**: `sherpa-onnx` (model Zipformer atau SenseVoice Small ~60MB, latensi < 100ms di CPU).
  - **Opsi B (Paling Akurat Multilingual)**: `whisper-rs` (Whisper Tiny / Base Q5/Q8).
* **Hasil**: Ragasukma bisa mendengarkan dan merespons suara pengguna saat browser ditutup, saat aplikasi berjalan di background, atau saat offline.

---

### FASE 3: Keamanan & Hardening API (Prioritas P1)

#### 3.1. Proteksi Autentikasi Lokal (Session Secret Token)
* **Masalah**: Endpoint `/api/cmd`, `/api/desktop/*`, dan `/api/assets/*` dapat diakses oleh situs web lain yang dibuka di browser melalui script `fetch()` lokal (CSRF).
* **Solusi**:
  1. Buat token acak (*cryptographically secure session token*) saat aplikasi start up: disimpan di memory dan di-expose hanya via file socket / parameter handshake ke dashboard.
  2. Middleware Axum memvalidasi header `Authorization: Bearer <token>` atau `X-Ragasukma-Token`.
  3. Permintaan tanpa token yang valid ditolak dengan HTTP 401 Unauthorized.

#### 3.2. Proteksi Cloudflare Tunnel
* Jika tunnel Cloudflare diaktifkan untuk mobile client:
  - Terapkan OTP / QR-code pairing yang menyematkan bearer token ke client mobile.
  - Blokir akses publik ke rute berbahaya (seperti eksekusi bash/skill command OS) dari jaringan eksternal kecuali diizinkan secara eksplisit oleh pengguna.

---

### FASE 4: Agentic Loop & Multi-Step Execution (Prioritas P2)

#### 4.1. ReAct Tool Loop (Reasoning + Acting) — [SELESAI / COMPLETED]
* **Implementasi**:
  - Siklus multi-turn internal ReAct (`handle_skill_execution_with_depth`):
    1. User berucap: *"Tolong cari harga laptop X di Tokopedia dan simpulkan"*.
    2. LLM Turn 1: Memanggil `[CMD:recipe:run|tokopedia_search|{"query":"laptop X"}]`.
    3. Engine mengeksekusi tool dan menangkap output terstruktur (`skill_results`).
    4. Engine menyuntikkan hasil tersebut ke dalam konteks percakapan sebagai message role `tool` (bukan role `user`).
    5. LLM Turn 2: Membaca hasil tool, lalu merespons dengan suara alami via Supertonic TTS: *"Berdasarkan pencarian, harga laptop X mulai dari Rp...*.
  - Event background/system status (`[SYSTEM STATUS: ...]` & `[TOOL RESULT: ...]`) otomatis diisolasi sebagai role `tool` di chat loop tanpa mencemari riwayat transcript user (`assistant:transcript`).

---

### FASE 5: Fleksibilitas Platform & Pengalaman Pengguna (Prioritas P2)

#### 5.1. Dukungan Mode `--headless` / `--no-gui`
* Izinkan parameter command-line: `ragasukma --headless` atau `ragasukma --no-gui`.
* Pada mode ini:
  - Melewati pembuatan window Winit/WGPU.
  - Langsung menyalakan HTTP server Axum dan LLM session worker.
  - Berguna untuk pengguna yang hanya ingin memanfaatkan Ragasukma sebagai server local LLM router `/v1` di headless Linux VPS/server.

#### 5.2. Indikator Koneksi Ekstensi Browser di Web Dashboard
* Tambahkan deteksi heartbeat aktif antara ekstensi browser dan dashboard:
  - Tampilkan lencana status: `Extension Connected: Active (v1.0.0)` atau `Extension: Not Detected (Click to Install)`.
  - Mencegah kebingungan pengguna saat fitur web click/DOM automation tidak berjalan karena ekstensi belum diaktifkan di Chrome.

---

## 4. Matriks Jadwal & Milestone

| Fase | Target Milestone | Estimasi Dampak |
|---|---|---|
| **Fase 1** | Streaming TTS per kalimat + SSE `/v1/chat/completions` native Axum | Latensi suara < 1s, Cursor/Zed streaming mulus |
| **Fase 2** | Native Audio Capture (cpal) + Silero VAD + Whisper/Sherpa ONNX | 100% mandiri, bebas cloud, tab browser bebas ditutup |
| **Fase 3** | Token Auth Middleware di port 42128 + Secure Cloudflare Pairing | Perlindungan CSRF & keamanan akses desktop jarak jauh |
| **Fase 4** | Siklus ReAct Tool Calling multi-step tanpa injeksi teks palsu | Kecerdasan agen meningkat untuk multi-step workflow |
| **Fase 5** | CLI `--headless` flag + Indikator status ekstensi di UI | Fleksibel di server/remote + UX lebih transparan |
