# 📡 Ragasukma API Request & Response Reference (`resreq.md`)

Dokumentasi lengkap seluruh endpoint HTTP REST API dan Server-Sent Events (SSE) pada backend **Ragasukma**.

- **Default Base URL**: `http://127.0.0.1:42128`
- **Content-Type**: `application/json` (kecuali streaming audio/binary upload)
- **CORS Policy**: `Access-Control-Allow-Origin: *` (Semua endpoint terbuka untuk browser client & companion extension)

---

## 📑 Daftar Isi
1. [🎙️ Chrome Web Speech API (STT & TTS)](#1-🎙️-chrome-web-speech-api-stt--tts)
2. [⚡ Realtime SSE Stream (`/api/events/state`)](#2-⚡-realtime-sse-stream-apieventsstate)
3. [🎮 Menu & Runtime Control State](#3-🎮-menu--runtime-control-state)
4. [⚙️ Config Service](#4-⚙️-config-service)
5. [🎨 3D VRM & 2D Avatar Assets](#5-🎨-3d-vrm--2d-avatar-assets)
6. [🕺 Poses & Live Bone Tuner](#6-🕺-poses--live-bone-tuner)
7. [🧩 Skills & System Applications](#7-🧩-skills--system-applications)
8. [🌐 Browser Companion Extension](#8-🌐-browser-companion-extension)
9. [☁️ Cloudflare Remote Tunnel & Mobile Text](#9-☁️-cloudflare-remote-tunnel--mobile-text)
10. [🎵 Local Music & YouTube Downloader](#10-🎵-local-music--youtube-downloader)
11. [🤖 Antigravity Multi-Account OAuth](#11-🤖-antigravity-multi-account-oauth)
12. [🧠 OpenAI-Compatible `/v1` Endpoints](#12-🧠-openai-compatible-v1-endpoints)
13. [📜 Logs Service & System Handshake](#13-📜-logs-service--system-handshake)

---

## 1. 🎙️ Chrome Web Speech API (STT & TTS)

### `POST /api/stt/transcript`
Menerima hasil pengenalan suara (STT) yang ditangkap langsung oleh Google Chrome (`webkitSpeechRecognition`) untuk disalurkan ke pipeline reasoning LLM backend.

- **Request Body**:
  ```json
  {
    "transcript": "Halo Sukma, tolong buka YouTube"
  }
  ```
  *(Alias payload yang diterima: `{ "text": "..." }`)*

- **Response (200 OK)**:
  ```json
  {
    "success": true,
    "message": "Transcript injected to LLM"
  }
  ```

- **Response Error (503 Service Unavailable)**:
  ```json
  {
    "error": "Voice chat loop is not currently active"
  }
  ```

---

### `POST /api/tts/state`
Menerima status sinkronisasi visual dan vokal dari Google Chrome Speech Recognition/Synthesis (`window.speechSynthesis` & `webkitSpeechRecognition`). Digunakan untuk sinkronisasi state pose avatar 3D (`listening`, `thinking`, `speaking`, `idle`), gerak bibir (morph targets), dan status system tray.

- **Proteksi Standby & Anti-Stuck Guard:**
  * Jika target state adalah `"listening"` namun sesi asisten di backend belum dimulai (`is_running == false`), backend otomatis mengabaikan permintaan ini (`{"success":true,"ignored":true}`) agar avatar tidak salah masuk ke pose mendengarkan saat komputer masih standby.
  * Browser dan backend memiliki *watchdog timer* (3.5 - 4.5 detik) yang otomatis memaksa status kembali ke `"idle"` jika tidak ada kata yang ter-generate.

- **Request Body (Mulai Berbicara / TTS Aktif)**:
  ```json
  {
    "speaking": true,
    "emotion": "happy"
  }
  ```

- **Request Body (Deteksi Suara User / Listening State)**:
  ```json
  {
    "state": "listening",
    "emotion": "neutral"
  }
  ```

- **Request Body (Selesai Bicara / Terpotong Interupsi / Kembali Standby)**:
  ```json
  {
    "speaking": false
  }
  ```
  *(Atau menggunakan `{ "state": "idle", "emotion": "neutral" }`)*

- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

## 2. ⚡ Realtime SSE Stream (`/api/events/state`)

Endpoint Server-Sent Events satu arah dengan reconnect otomatis untuk mengalirkan status avatar dan trigger aksi dari backend ke browser.

### `GET /api/events/state` *(atau `/api/events`)*
- **Header Response**:
  ```http
  HTTP/1.1 200 OK
  Content-Type: text/event-stream
  Cache-Control: no-cache
  Connection: keep-alive
  ```

- **Event: `state`** (Diterbitkan saat visual state, status emosi, mute, atau status sesi `is_running` berubah):
  ```sse
  event: state
  data: {"state":"speaking","emotion":"happy","voice_muted":false,"mic_muted":false,"is_running":true}
  ```
  *Nilai `state`: `"idle"` | `"listening"` | `"speaking"` | `"thinking"` | `"shutdown"`.*
  *Nilai `emotion`: 8 emosi kanonikal (`"happy"`, `"shy"`, `"excited"`, `"surprised"`, `"thinking"`, `"sad"`, `"angry"`, `"neutral"`).*
  *Nilai `is_running`: `true` saat tombol "Mulai Voice Assistant" aktif, `false` saat standby.*

- **Event: `action`** (Trigger perintah aksi/TTS):
  * **Assistant Bicara (TTS)**:
    ```sse
    event: action
    data: {"action":"assistant:speak","payload":"{\"text\":\"Halo Mas! Ada yang bisa kubantu?\",\"emotion\":\"senang\"}"}
    ```
  * **Audio Musik**:
    ```sse
    event: action
    data: {"action":"media:play","payload":"/music/lagu.mp3"}
    ```
  * **Interupsi / Mute**:
    ```sse
    event: action
    data: {"action":"voice:interrupt"}
    ```

---

## 3. 🎮 Menu & Runtime Control State

### `GET /api/state`
Mengambil kondisi sistem saat ini secara instan (1x call saat halaman web pertama kali dimuat).

- **Response (200 OK)**:
  ```json
  {
    "state": "idle",
    "is_running": true,
    "is_paused": false,
    "voice_muted": false,
    "mic_muted": false,
    "active_session_token": "token-tab-12345"
  }
  ```

---

### `POST /api/state`
Memperbarui state sistem dari kontrol tombol Web Dashboard.

- **Request Body**:
  ```json
  {
    "state": "idle",
    "is_running": true,
    "is_paused": false,
    "voice_muted": false,
    "mic_muted": false
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

### `POST /api/cmd`
Mengirim perintah aksi cepat dari UI Dashboard atau Tray menu.

- **Request Body**:
  ```json
  {
    "cmd": "start_chat"
  }
  ```
  *Daftar `cmd` valid: `"start_chat"`, `"stop_chat"`, `"toggle_pause"`, `"voice_mute"`, `"voice_unmute"`, `"mic_mute"`, `"mic_unmute"`, `"quit"`.*

- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

### `POST /api/session/claim`
Memastikan kepemilikan tab tunggal (Single-Tab Ownership) agar STT Chrome tidak berebut mikrofon.

- **Request Body**:
  ```json
  {
    "token": "browser-tab-uuid-xxxx"
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true,
    "claimed": true,
    "owner": "browser-tab-uuid-xxxx"
  }
  ```

---

## 4. ⚙️ Config Service

### `GET /api/config`
Membaca seluruh isi file `assets/config-ai.json`.

- **Response (200 OK)**:
  ```json
  {
    "assistant_name": "Sukma",
    "user_callsign": "Mas",
    "personality_role": "tsundere",
    "assistant_gender": "female",
    "llm_provider": "custom",
    "llm_base_url": "http://localhost:20128/v1",
    "llm_model": "kc/dots-studio/dots-3-note-preview:free",
    "llm_api_key": "sk-xxxx",
    "tts_lang": "id-ID",
    "stt_lang": "id-ID",
    "tts_voice_name": "Google Bahasa Indonesia",
    "tts_speed": 1.15,
    "tts_pitch": 1.25,
    "avatar_type": "vrm",
    "always_on_top": true,
    "voice_interruption": false
  }
  ```

---

### `POST /api/config`
Menyimpan konfigurasi baru ke `assets/config-ai.json` dan menyebarkan sinyal reaktif live ke renderer WGPU dan thread sesi.

- **Request Body**: Objek JSON konfigurasi lengkap (sama seperti GET).
- **Response (200 OK)**:
  ```json
  {
    "success": true,
    "message": "Config updated"
  }
  ```

---

## 5. 🎨 3D VRM & 2D Avatar Assets

### `GET /api/assets/info`
Mendapatkan informasi ketersediaan file model 3D (`char.vrm`) dan daftar gambar avatar yang ada di direktori `assets/`.

- **Response (200 OK)**:
  ```json
  {
    "vrm_exists": true,
    "vrm_size_bytes": 15420120,
    "vrm_size_mb": 14.7,
    "vrm_path": "assets/char.vrm",
    "images": [
      "char-avatar-idle.png",
      "char-avatar-listening.png",
      "char-avatar-speaking.png",
      "char-avatar-thinking.png"
    ]
  }
  ```

---

### `POST /api/upload/vrm`
Mengunggah file model VRM (`char.vrm`) ke server secara biner dan langsung memicu live reload 3D mesh di WGPU window.

- **Header Request**: `Content-Type: application/octet-stream`
- **Request Body**: Buffer biner file `.vrm`.
- **Response (200 OK)**:
  ```json
  {
    "success": true,
    "message": "VRM model uploaded and reloaded successfully"
  }
  ```

---

### `POST /api/upload/avatar?slot={slot_name}`
Mengunggah gambar 2D untuk ekspresi avatar tertentu.

- **Query Parameter**: `slot` (`idle` | `listening` | `speaking` | `thinking`).
- **File Tersimpan**:
  * `idle` -> `assets/char-avatar-idle.png`
  * `listening` -> `assets/char-avatar-listening.png`
  * `speaking` -> `assets/char-avatar-speaking.png`
  * `thinking` -> `assets/char-avatar-thinking.png`
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

## 6. 🕺 Poses & Live Bone Tuner

### `GET /api/poses`
Mengambil konfigurasi orientasi tulang per visual state dari `assets/poses.json`. Mendukung multi-pose dinamis dengan Catmull-Rom spline interpolation (bukan hanya 2 pose kaku A/B).

- **Response (200 OK)**:
  ```json
  {
    "idle": {
      "speed": 1.0,
      "poses": [
        {
          "bones": {
            "head": [0.03, 0.01, -0.14],
            "leftUpperArm": [0.09, 0.06, -1.4],
            "leftIndex": [-0.01, 0.24, -0.08]
          }
        }
      ]
    },
    "listening": { ... },
    "thinking": { ... },
    "speaking": { ... }
  }
  ```

---

### `POST /api/poses`
Menyimpan konfigurasi rotasi pose permanen ke disk (`assets/poses.json`).

- **Request Body**: JSON data struktur multi-pose `PosesConfig`.
- **Response (200 OK)**:
  ```json
  {
    "success": true,
    "message": "Poses saved successfully"
  }
  ```

---

### `POST /api/tuner/preview`
Meninjau animasi/pose state tertentu secara visual tanpa menyimpan ke disk.

- **Request Body**:
  ```json
  {
    "state": "speaking",
    "emotion": "senang",
    "poses": [
      {
        "bones": {
          "head": [0.05, -0.02, 0.0]
        }
      }
    ]
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

### `POST /api/tuner/live`
Mengirimkan sudut rotasi batch sendi tulang secara realtime (saat user menggeser slider tuner di UI) untuk diaplikasikan ke model 3D seketika (< 5ms) tanpa tertimpa siklus animasi otomatis (`is_tuning` active).

- **Request Body**:
  ```json
  {
    "active": true,
    "state": "idle",
    "emotion": "neutral",
    "bones": {
      "head": [0.05, -0.12, 0.0],
      "leftUpperArm": [0.35, 0.2, -0.8],
      "leftIndex": [0.0, 0.0, 1.25]
    }
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

## 7. 🧩 Skills & System Applications

### `GET /api/skills`
Mengambil daftar custom skills yang terpasang di `assets/skills.json`.

- **Response (200 OK)**: Array objek skill (trigger, deskripsi, command/action).

---

### `POST /api/skills`
Menambah atau mengedit definisi skill baru.

- **Request Body**:
  ```json
  {
    "name": "Buka Spotify",
    "trigger": "buka musik",
    "command": "spotify"
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

### `POST /api/skills/delete`
Menghapus skill berdasarkan ID/nama.

- **Request Body**:
  ```json
  {
    "name": "Buka Spotify"
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

### `GET /api/system/apps`
Memindai aplikasi `.desktop` (Linux) atau `Start Menu` (Windows) yang terpasang di komputer pengguna.

- **Response (200 OK)**:
  ```json
  [
    { "name": "Google Chrome", "exec": "google-chrome-stable", "icon": "google-chrome" },
    { "name": "Visual Studio Code", "exec": "code", "icon": "code" }
  ]
  ```

---

## 8. 🌐 Browser Companion Extension

Digunakan oleh ekstensi browser Ragasukma untuk grounding konteks halaman aktif (Active Tab & DOM Snapshot).

### `POST /api/browser/snapshot`
Menerima DOM snapshot teks dari ekstensi browser.

- **Request Body**:
  ```json
  {
    "url": "https://id.wikipedia.org/wiki/Indonesia",
    "title": "Indonesia - Wikipedia",
    "content": "Indonesia adalah negara kepulauan di Asia Tenggara..."
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

### `POST /api/browser/tabs`
Memperbarui daftar tab yang sedang terbuka di browser.

- **Request Body**: Array objek tab `[{ "id": 1, "title": "...", "url": "..." }]`.
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

### `GET /api/browser/command/next`
Ekstensi melakukan long-polling untuk mengambil antrean perintah kontrol browser (misal: scroll, navigasi URL, klik elemen).

- **Response (200 OK)**:
  ```json
  {
    "command": "scroll_down",
    "params": { "pixels": 500 }
  }
  ```

---

## 9. ☁️ Cloudflare Remote Tunnel & Mobile Text

### `GET /api/remote/tunnel/status`
Mengambil status koneksi tunnel Cloudflare dan URL publik untuk remote control HP.

- **Response (200 OK)**:
  ```json
  {
    "status": "running",
    "url": "https://ragasukma-random-subdomain.trycloudflare.com",
    "auth_key": "secret-auth-key-xxxx"
  }
  ```

---

### `POST /api/remote/tunnel/start`
Menyalakan proses binary `cloudflared` bawaan untuk membuat tunnel publik.

- **Response (200 OK)**:
  ```json
  {
    "status": "starting",
    "url": "https://ragasukma-random-subdomain.trycloudflare.com",
    "auth_key": "secret-auth-key-xxxx"
  }
  ```

---

### `POST /api/remote/tunnel/stop`
Mematikan proses tunnel Cloudflare.

- **Response (200 OK)**:
  ```json
  {
    "status": "stopped"
  }
  ```

---

### `POST /api/remote/text?key={auth_key}`
Menerima teks percakapan dari antarmuka Web HP jarak jauh yang diamankan dengan `auth_key`.

- **Request Body**:
  ```json
  {
    "text": "Halo Sukma, matikan PC dalam 10 menit"
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true,
    "message": "Text sent to LLM"
  }
  ```

---

### `POST /api/remote/audio-action?key={auth_key}`
Mengatur perutean audio (Audio Routing) dan sinkronisasi playback antara **Mode Remote Desktop** vs **Mode Full Mobile**:
- **Mode Remote Desktop:** TTS dan musik tetap bersuara dari speaker PC desktop; HP hanya berfungsi sebagai remote kontrol.
- **Mode Full Mobile:** Suara TTS dan musik di desktop PC otomatis ditiadakan (mute di PC), suara dialihkan penuh ke smartphone pengguna. Mic di browser desktop otomatis di-mute saat remote aktif untuk mencegah *feedback loop*.

- **Request Body**:
  ```json
  {
    "mode": "full_mobile",
    "action": "music:play",
    "payload": "/music/chill_lofi.mp3"
  }
  ```
- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```

---

## 10. 🎵 Local Music & YouTube Downloader

### `GET /api/music/list`
Mengambil daftar file audio lokal yang ada di folder `music/`.

- **Response (200 OK)**:
  ```json
  [
    {
      "name": "chill_lofi.mp3",
      "url": "/music/chill_lofi.mp3",
      "size": 4210512
    }
  ]
  ```

---

### `GET /music/{filename}` *(atau `/api/music/stream?file=...`)*
Streaming file audio musik dengan dukungan `HTTP 206 Partial Content` (Range Header) untuk seeking durasi lagu.

---

### `GET /api/music/check`
Memeriksa apakah `yt-dlp` dan `ffmpeg` tersedia di sistem untuk pengunduhan musik otomatis.

- **Response (200 OK)**:
  ```json
  {
    "ytdlp_installed": true,
    "ffmpeg_installed": true
  }
  ```

---

## 11. 🤖 Antigravity Multi-Account OAuth

### `GET /api/llm/antigravity/accounts`
Mengambil daftar akun Google Cloud / Gemini yang terautentikasi.

- **Response (200 OK)**:
  ```json
  [
    {
      "email": "user@gmail.com",
      "active": true,
      "quota_remaining": 98.5
    }
  ]
  ```

---

### `GET /api/llm/antigravity/oauth/url`
Mendapatkan URL login Google OAuth 2.0 untuk menambahkan akun LLM baru.

- **Response (200 OK)**:
  ```json
  {
    "url": "https://accounts.google.com/o/oauth2/v2/auth?client_id=..."
  }
  ```

---

## 12. 🧠 OpenAI-Compatible `/v1` Endpoints

Mengizinkan Ragasukma bertindak sebagai gateway proxy OpenAI standard untuk klien eksternal.

### `POST /v1/chat/completions` *(atau `/api/v1/chat/completions`)*
- **Header Request**: `Authorization: Bearer sk-xxxx`
- **Request Body**:
  ```json
  {
    "model": "ragasukma",
    "messages": [
      { "role": "user", "content": "Halo apa kabar?" }
    ],
    "stream": false
  }
  ```
- **Response (200 OK)**: Format standar respons chat completion OpenAI.

---

### `GET /v1/models`
- **Response (200 OK)**:
  ```json
  {
    "object": "list",
    "data": [
      { "id": "ragasukma", "object": "model", "owned_by": "ragasukma" }
    ]
  }
  ```

---

## 13. 📜 Logs Service & System Handshake

### `GET /api/ping`
Handshake ping cepat untuk verifikasi kesiapan server backend.

- **Response (200 OK)**:
  ```json
  {
    "app": "ragasukma",
    "status": "running"
  }
  ```

---

### `GET /api/logs`
Mengambil riwayat buffer log sistem (hingga 1.000 log terbaru).

- **Response (200 OK)**:
  ```json
  [
    {
      "id": 1,
      "timestamp": "17:11:54",
      "level": "info",
      "message": "🚀 [Task Worker] Dedicated Background Task Engine siap."
    }
  ]
  ```

---

### `POST /api/logs/clear`
Mengosongkan buffer log di memori backend.

- **Response (200 OK)**:
  ```json
  {
    "success": true
  }
  ```
