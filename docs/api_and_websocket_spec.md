# Dokumentasi API, WebSocket, dan SSE (Backend & Frontend Web)

Dokumen ini menjelaskan arsitektur komunikasi real-time (**WebSocket** & **Server-Sent Events**) serta seluruh rute **REST API** antara backend Rust (Axum) dan frontend Web (SolidJS).

> **Panduan Optimasi & Efisiensi SolidJS:**
> * Gunakan fine-grained reactivity SolidJS (`createSignal` / `createStore` dengan `reconcile`) agar render UI tidak ter-trigger berlebihan saat menerima stream WebSocket/SSE.
> * Pisahkan subscription berat (seperti log sistem) agar hanya aktif ketika tab UI yang bersangkutan terbuka.
> * Kirim delta updates (hanya field yang berubah) saat sinkronisasi frekuensi tinggi (misal 3D Pose Tuner 60 FPS).

---

## 1. WebSocket (Bi-directional Real-Time)

* **Endpoint:** `GET /api/ws`
* **Backend Handler:** `crate::services::events_service::axum_ws_handler`
* **Frontend Service:** `ai-voice/web/src/services/eventsService.ts` (`startWsStream()`)

Koneksi WebSocket menangani komunikasi real-time dua arah untuk sinkronisasi status AI, trigger action, streaming log sistem, dan live 3D pose tuning.

### A. Server to Client Events (Push)

Semua pesan yang dikirim dari server berformat JSON dengan struktur umum:
```json
{
  "event": "<event_name>",
  "data": { ... }
}
```

1. **`event: "state"`**
   * Dikirim saat ada perubahan status AI atau saat koneksi pertama kali terbuka (*initial connection*).
   * **Optimasi SolidJS:** Gunakan signal terpisah per atribut atau `reconcile` store agar perubahan `emotion` tidak memicu re-render avatar state bila nilainya sama.
   * **Payload (`CoreStatePayload`):**
     ```json
     {
       "event": "state",
       "data": {
         "state": "idle" | "listening" | "speaking" | "thinking" | "shutdown",
         "emotion": "neutral" | "happy" | "sad" | "angry" | string,
         "voice_muted": false,
         "mic_muted": false,
         "is_running": true
       }
     }
     ```

2. **`event: "action"`**
   * Push notification ke client browser untuk memicu aksi tertentu di UI atau media player.
   * **Catatan Optimasi Payload:** Gunakan native JSON object pada `payload` (hindari *double stringified JSON* seperti `"\"file\": ...\""` agar tidak memerlukan `JSON.parse()` ganda di SolidJS).
   * **Contoh trigger:**
     * `assistant:speak` — Memutar suara / TTS.
     * `music:play` — Memutar trek lagu di frontend player.
     * `antigravity:oauth_success` — Notifikasi OAuth Google sukses diterima backend.
     * `codex:oauth_success` — Notifikasi OAuth OpenAI Codex sukses diterima backend.
   * **Payload (`ClientAction`):**
     ```json
     {
       "event": "action",
       "data": {
         "id": 123,
         "action": "music:play",
         "payload": {
           "file": "lagu.mp3"
         }
       }
     }
     ```

3. **`event: "log"`**
   * Streaming baris log konsol dari backend ke UI Logs tab.
   * **Catatan Efisiensi & Rekomendasi:**
     * Menyemburkan log berfrekuensi tinggi ke WebSocket umum dapat membebani main thread JavaScript SolidJS dan memori browser saat agent sedang kompilasi/run task besar.
     * *Rekomendasi implementasi:* Frontend hanya memproses atau me-retain buffer log jika tab `Logs` aktif (`logsContainerRef` terpasang). Maksimal simpan 250 log di memori frontend untuk menjaga performa DOM SolidJS.
   * **Payload:**
     ```json
     {
       "event": "log",
       "data": {
         "id": 45,
         "timestamp": "2026-10-28T12:00:00Z",
         "level": "INFO" | "WARN" | "ERROR",
         "message": "Model loaded successfully"
       }
     }
     ```

4. **`event: "model_status"`**
   * Mengabarkan status dan progress download model LLM, STT, atau TTS.
   * **Catatan Frekuensi Server:** Server membroadcast event ini secara debounce/throttled saat progress download berubah agar tidak membombardir event loop browser.
   * **Payload (`ModelDownloadStatus`):**
     ```json
     {
       "event": "model_status",
       "data": {
         "status": "ready" | "checking" | "downloading" | "extracting" | "error",
         "downloaded_bytes": 104857600,
         "total_bytes": 524288000,
         "percent": 20.0,
         "speed_mbps": 12.5,
         "message": "Downloading model weights...",
         "model_name": "supertonic-tts"
       }
     }
     ```

---

### B. Client to Server Messages (Send)

Pesan dikirim dari browser dalam format JSON dengan tag `"type"`:

1. **`type: "tuner_live"` (High Frequency Pose Tuning)**
   * Menyesuaikan posisi/rotasi tulang VRM 3D secara live dari Pose Tuner.
   * **Optimasi & Penghilangan Attribute Redundan:**
     * Pada pembaruan 60 FPS slider, kirim **hanya tulang yang berubah (`bones`)**.
     * Attribute `emotion` dan `state` bersifat **opsional**. Jangan kirim setiap tick slider jika nilainya tidak berubah agar mengurangi alokasi memori dan parsing JSON di server backend.
     * Gunakan `requestAnimationFrame` di frontend SolidJS untuk membatasi pengiriman pesan (throttling ke 30–60 FPS).
   ```json
   {
     "type": "tuner_live",
     "active": true,
     "bones": {
       "head": [0.0, 1.2, 0.1],
       "leftArm": [0.3, 0.8, -0.2]
     },
     "emotion": "happy",   // Opsional: sertakan hanya jika berubah
     "state": "speaking"   // Opsional: sertakan hanya jika berubah
   }
   ```

2. **`type: "tuner_preview"`**
   * Menampilkan preview konfigurasi pose penuh secara instan tanpa menyimpan ke disk.
   ```json
   {
     "type": "tuner_preview",
     "poses": { ... },
     "emotion": "neutral",
     "state": "idle"
   }
   ```

3. **`type: "tab_change"` & `type: "claim"` (Direkomendasikan via WebSocket)**
   * Daripada membuat request HTTP POST terpisah (`/api/session/tab` dan `/api/session/claim`), pengiriman status tab aktif dan klaim sesi tab dapat dilakukan langsung melalui WebSocket:
   ```json
   {
     "type": "tab",
     "active_tab": "agent"
   }
   ```

4. **`type: "ping"` (Keep-Alive)**
   * Keep-alive ping dari client browser untuk menjaga koneksi tetap aktif di jaringan mobile / proxy tunnel (dapat menggunakan application-level ping JSON atau WebSocket standard ping frames).
   ```json
   {
     "type": "ping"
   }
   ```

---

## 2. Server-Sent Events (SSE / `text/event-stream`)

Backend menyediakan 3 endpoint SSE:

### 1. `GET /api/chat/events`
* **Backend Handler:** `axum_chat_events_sse` di `services/api_router/system.rs`
* **Frontend Service:** `ai-voice/web/src/services/historyService.ts` (`subscribeChatEvents()`)
* **Fungsi:** Streaming percakapan dialog suara / chat secara langsung saat user dan asisten sedang berinteraksi.
* **Format Payload:**
  * Initial ping: `data: {"type": "connected"}`
  * Turn event:
    ```json
    data: {
      "type": "chat_turn",
      "item": {
        "id": "turn_123",
        "user_text": "Halo Ragasukma",
        "assistant_text": "Halo! Ada yang bisa saya bantu?",
        "timestamp": 1730100000
      }
    }
    ```

### 2. `GET /api/agent/sessions/:id/events`
* **Backend Handler:** `session_events_sse` di `services/api_router/agent.rs`
* **Frontend Hook:** `ai-voice/web/src/components/agent/useChatSession.ts` (`connectSessionSse()`)
* **Fungsi:** Streaming progress eksekusi coding agent pada session project tertentu secara real-time.
* **Struktur Event Sebenarnya (Sesuai `AgentSessionEvent` Backend):**
  * `{"type": "connected", "data": {"session_id": "...", "status": "idle" | "running"}}` — Initial connection & status snapshot.
  * `{"type": "delta", "data": {"delta": "teks lanjutan..."}}` — Streaming token teks respon asisten.
  * `{"type": "thought_delta", "data": {"delta": "penalaran..."}}` — Streaming reasoning/thought tokens (misal model Gemini Thinking/CoT).
  * `{"type": "message", "data": AgentMessage}` — Pesan lengkap yang berhasil di-commit (baik pesan teks asisten, tool invocation, maupun tool output hasil eksekusi file/command).
  * `{"type": "step", "data": {"iteration": 1, "action": "tool_call", "details": "running cargo check"}}` — Indikator langkah eksekusi agent yang sedang berjalan.
  * `{"type": "status", "data": {"status": "running" | "idle" | "error"}}` — Perubahan status lifecycle session.
  * `{"type": "done", "data": {"reply": "..."}}` — Task selesai dieksekusi.
  * `{"type": "error", "data": {"message": "..."}}` — Error fatal pada eksekusi session.
  * `{"type": "stream_reset"}` — Reset buffer streaming frontend.
  * `event: ping\ndata: {}` — Periodic keep-alive ping setiap 3 detik untuk mencegah timeout Cloudflare Tunnel.

### 3. `POST /v1/chat/completions` & `/api/v1/chat/completions` (OpenAI-Compatible SSE)
* **Backend Handler:** `axum_v1_chat_completions_handler` di `services/openai_v1_service.rs`
* **Fungsi:** Jika request menyertakan `"stream": true`, backend merespons dengan format standar OpenAI SSE:
  ```text
  data: {"id":"chatcmpl-123","object":"chat.completion.chunk","choices":[{"delta":{"content":"Halo"}}]}

  data: [DONE]
  ```

---

## 3. Daftar REST API Endpoints

### A. System, Configuration, and Logs (`services/api_router/system.rs`)

| Method | Endpoint | Deskripsi & Catatan Optimasi |
|---|---|---|
| `GET` | `/api/ping` | Healthcheck server (`{"app":"ragasukma","status":"running"}`) |
| `GET` | `/api/config` | Mengambil seluruh konfigurasi user (`UserConfig`) |
| `POST` | `/api/config` | Menyimpan perubahan konfigurasi user |
| `GET` | `/api/state` | Mengambil status runtime saat ini |
| `POST` | `/api/state` | Mengubah status runtime (state, emotion, mute) |
| `GET` | `/api/system/apps` | Mendeteksi aplikasi yang terinstall di OS lokal |
| `GET` | `/api/system/extension-path` | Mengambil path folder / file zip ekstensi browser |
| `GET` | `/api/system/model-status` | Cek status file model AI lokal |
| `POST` | `/api/system/model-download` | Memulai download model AI |
| `GET` | `/api/system/python/info` | Cek ketersediaan dan versi runtime Python |
| `POST` | `/api/system/python/install` | Trigger auto-instalasi runtime Python |
| `POST` | `/api/cmd` | Mengirim perintah tray / shortcut menu |
| `GET` | `/api/actions/since?id=<id>` | Polling client action (fallback jika WebSocket mati) |
| `GET` | `/api/history/dates` | Mengambil daftar tanggal riwayat percakapan |
| `GET` | `/api/history/view?date=YYYY-MM-DD` | Mengambil daftar chat turn pada tanggal tertentu |
| `POST` | `/api/history/delete` | Menghapus riwayat percakapan |
| `GET` | `/api/logs?since=<id>` | Mengambil baris log sistem via REST (initial fetch sebelum WS stream) |
| `POST` | `/api/logs/clear` | Mengosongkan buffer log memori |
| `POST` | `/api/session/claim` | Mengklaim ownership browser session (Single-tab claim) |
| `POST` | `/api/session/tab` | Memberitahu backend tab UI yang sedang aktif |

---

### B. Agent & Coding Workspace (`services/api_router/agent.rs`)

| Method | Endpoint | Deskripsi & Catatan Optimasi |
|---|---|---|
| `GET` | `/api/agent/projects` | Mengambil daftar semua project agent |
| `POST` | `/api/agent/projects` | Membuat project agent baru |
| `GET` | `/api/agent/projects/:id` | Detail project |
| `POST` | `/api/agent/projects/:id/update` | Memperbarui nama, deskripsi, atau folder project |
| `POST` | `/api/agent/projects/:id/delete` | Menghapus project |
| `GET` | `/api/agent/projects/:id/files` | Mengambil struktur folder & file dalam project *(perlu ignore `node_modules`, `.git`, `target` agar response tidak membesar)* |
| `GET` | `/api/agent/projects/:id/file-content?path=...` | Membaca konten file dari project |
| `POST` | `/api/agent/validate-folder` | Validasi apakah path direktori valid di OS |
| `POST` | `/api/agent/browse-folder` | Menampilkan native file picker OS untuk memilih direktori |
| `POST` | `/api/agent/open-in-file-manager` | Membuka folder project di explorer/finder lokal |
| `GET` | `/api/agent/projects/:id/sessions` | Mengambil daftar session percakapan agent pada project |
| `POST` | `/api/agent/projects/:id/sessions` | Membuat session baru |
| `POST` | `/api/agent/sessions/:id/delete` | Menghapus session agent |
| `POST` | `/api/agent/sessions/:id/run` | Menjalankan task agent pada session (streaming via SSE) |
| `POST` | `/api/agent/sessions/:id/stop` | Menghentikan task agent yang sedang berjalan |
| `POST` | `/api/agent/sessions/from-chat` | Membuat session agent baru dari chat turn history |
| `GET` | `/api/agent/sessions/:id/messages` | Mengambil riwayat pesan dalam session agent *(mendukung pagination untuk performa UI)* |
| `POST` | `/api/agent/sessions/:id/messages` | Menambahkan pesan baru ke session |
| `GET` | `/api/agent/skills` | Mengambil daftar skill coding global |
| `GET` | `/api/agent/projects/:id/skills` | Mengambil skill khusus yang ada di direktori project |
| `GET` | `/api/agent/projects/:id/mcp` | Daftar MCP server aktif untuk project |
| `POST` | `/api/agent/projects/:id/mcp` | Menghubungkan MCP server ke project |
| `GET` | `/api/agent/mcp` | Mengambil daftar server MCP global |
| `POST` | `/api/agent/mcp` | Menyimpan konfigurasi MCP server baru |
| `POST` | `/api/agent/mcp/:id/delete` | Menghapus MCP server |
| `POST` | `/api/agent/mcp/test` | Test koneksi ke MCP server |
| `GET` | `/api/agent/projects/:id/goals` | Mengambil daftar goal dalam project |
| `POST` | `/api/agent/projects/:id/goals` | Membuat goal baru |
| `GET` | `/api/agent/goals/:id` | Detail goal |
| `GET` | `/api/agent/goals/:id/evidence` | Mengambil daftar bukti eksekusi (evidence) goal |
| `GET` | `/api/agent/goals/:id/verifications` | Mengambil hasil verifikasi build/test goal |
| `POST` | `/api/agent/goals/:id/conclude` | Menandai goal selesai / conclude |

---

### C. LLM Providers, Accounts Pool, & Quota (`services/api_router/llm.rs`)

#### 1. Antigravity (Google Gemini Provider)
| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET` | `/api/llm/antigravity/models` | List model Gemini yang tersedia |
| `GET` | `/api/llm/antigravity/accounts` | Mengambil daftar akun Google di quota pool |
| `POST` | `/api/llm/antigravity/accounts` | Menambahkan akun Google baru |
| `POST` | `/api/llm/antigravity/accounts/delete` | Menghapus akun dari pool |
| `POST` | `/api/llm/antigravity/accounts/refresh` | Refresh OAuth access token akun |
| `POST` | `/api/llm/antigravity/accounts/toggle` | Mengaktifkan/menonaktifkan akun di pool |
| `GET / POST`| `/api/llm/antigravity/accounts/quota` | Cek kuota limit per menit & per hari akun |
| `GET` | `/api/llm/antigravity/oauth/url` | Mendapatkan URL login Google OAuth |
| `GET` | `/api/llm/antigravity/oauth/callback`| Callback penerima OAuth code dari Google |
| `POST`| `/api/llm/antigravity/oauth/exchange`| Menukar kode otorisasi menjadi token |

#### 2. Codex (OpenAI Codex Provider)
| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET` | `/api/llm/codex/models` | List model Codex yang tersedia |
| `GET` | `/api/llm/codex/accounts` | Mengambil daftar akun OpenAI Codex di pool |
| `POST` | `/api/llm/codex/accounts` | Menambahkan akun Codex baru |
| `POST` | `/api/llm/codex/accounts/delete` | Menghapus akun dari pool |
| `POST` | `/api/llm/codex/accounts/refresh` | Refresh token OpenAI Codex |
| `POST` | `/api/llm/codex/accounts/toggle` | Mengaktifkan/menonaktifkan akun di pool |
| `GET / POST`| `/api/llm/codex/accounts/quota` | Cek sisa kuota akun Codex |
| `GET` | `/api/llm/codex/oauth/url` | Mendapatkan URL login OpenAI OAuth |
| `GET` | `/api/llm/codex/oauth/callback`| Callback penerima OAuth code OpenAI |
| `POST`| `/api/llm/codex/oauth/exchange`| Menukar kode otorisasi menjadi token |
| `GET` | `/auth/callback`, `/callback` | URL callback alias untuk kompatibilitas redirect OAuth |

#### 3. LLM Token Usage Analytics
| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET` | `/api/llm/usage/summary` | Ringkasan total prompt token, output token, dan perkiraan biaya |
| `GET` | `/api/llm/usage/records` | Daftar record penggunaan token per model dan waktu |
| `POST` | `/api/llm/usage/clear` | Mereset data pencatatan token usage |

---

### D. Automation, Browser Extension, & Remote (`services/api_router/automation.rs`)

| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET / POST` | `/api/skills` | Ambil daftar skill / buat skill automasi |
| `POST` | `/api/skills/delete` | Menghapus skill |
| `POST` | `/api/skills/run` | Menjalankan perintah skill |
| `GET / POST` | `/api/todos` | Mengambil daftar to-do / membuat to-do baru |
| `POST` | `/api/todos/complete` | Menandai to-do selesai |
| `POST` | `/api/todos/delete` | Menghapus to-do |
| `GET` | `/api/recipes` | Mengambil daftar recipe otomasi |
| `POST` | `/api/recipes/save` | Menyimpan recipe |
| `POST` | `/api/recipes/delete` | Menghapus recipe |
| `POST` | `/api/recipes/run` | Menjalankan recipe otomasi |
| `POST` | `/api/browser/snapshot` | Menerima snapshot DOM/Accessibility dari ekstensi browser |
| `POST` | `/api/browser/copied_text`| Sinkronisasi teks yang dicopy user di browser |
| `GET / POST` | `/api/browser/tabs` | Mengambil / memperbarui tab aktif di browser |
| `GET` | `/api/browser/command/next` | Long-poll ekstensi browser untuk mengambil perintah automasi berikutnya |
| `GET` | `/api/remote/tunnel/status`| Cek status binary dan koneksi Cloudflare Tunnel |
| `GET` | `/api/remote/tunnel/check` | Cek ketersediaan binary `cloudflared` |
| `POST` | `/api/remote/tunnel/install`| Download dan pasang binary `cloudflared` |
| `POST` | `/api/remote/tunnel/start` | Menjalankan Cloudflare Tunnel untuk akses remote publik |
| `POST` | `/api/remote/tunnel/stop` | Menghentikan Cloudflare Tunnel |
| `POST` | `/api/remote/active` | Touch remote heartbeat |
| `POST` | `/api/remote/text` | Mengirim teks input dari client remote |

---

### E. Media, Audio, TTS/STT, & Assets (`services/api_router/media.rs` & `server.rs`)

| Method | Endpoint | Deskripsi & Catatan Optimasi |
|---|---|---|
| `GET / POST` | `/api/poses` | Ambil atau simpan konfigurasi pose VRM |
| `POST` | `/api/tuner/preview` | Preview pose via HTTP (fallback jika WS belum siap) |
| `POST` | `/api/tuner/live` | Update live tuning bones via HTTP *(Gunakan WebSocket `tuner_live` sebagai jalur utama untuk menghindari ratusan HTTP handshake per menit)* |
| `GET` | `/api/music/check` | Cek ketersediaan yt-dlp & player musik |
| `POST` | `/api/music/install-ytdlp` | Download & instalasi binary yt-dlp |
| `GET` | `/api/music/list` | Mengambil daftar file musik yang ada di folder musik |
| `GET` | `/api/music/status` | Mengambil status pemutaran musik saat ini |
| `POST` | `/api/music/control` | Kontrol musik: `play`, `pause`, `resume`, `stop`, `set_volume` |
| `POST` | `/api/music/download` | Download musik dari URL YouTube |
| `POST` | `/api/music/delete` | Menghapus file musik |
| `GET` | `/api/music/stream?filename=...` | Stream file audio musik (mendukung HTTP Range header `bytes=start-end`) |
| `GET` | `/music/*path` | Path alias untuk streaming file audio musik |
| `POST` | `/api/stt/transcript` | Menerima transkripsi audio STT |
| `POST` | `/api/tts/state` | Update status audio player TTS di frontend |
| `POST` | `/api/tts/synthesize` | Meminta backend mensintesis teks menjadi audio WAV |
| `GET` | `/api/tts/audio?id=...` | Mengambil file WAV audio TTS hasil sintesis |
| `POST` | `/api/tts/evict` | Menghapus buffer audio TTS yang sudah selesai diputar |
| `POST` | `/api/tts/ack` | Alias evict audio *(identik dengan `/api/tts/evict`)* |
| `GET` | `/api/assets/info` | Informasi avatar aktif, model VRM, dan reference voice |
| `GET` | `/api/assets/file/*path` | Mengambil asset statis lokal |
| `POST` | `/api/upload/vrm` | Upload model 3D `.vrm` |
| `POST` | `/api/upload/avatar` | Upload gambar avatar AI |
| `POST` | `/api/upload/refvoice` | Upload audio referensi suara untuk clone/TTS |
| `POST` | `/api/delete/refvoice` | Menghapus file audio referensi suara |

---

### F. OpenAI-Compatible Standard Proxy (`server.rs`)

Backend bertindak sebagai proxy kompatibel dengan OpenAI API sehingga tools eksternal (seperti Claude Dev, Continue, Aider, atau Zed) dapat langsung mengakses model Antigravity atau Codex:

| Method | Endpoint | Deskripsi |
|---|---|---|
| `POST` | `/v1/chat/completions` | Chat completions (mendukung mode regular JSON dan SSE streaming) |
| `POST` | `/api/v1/chat/completions` | Alias endpoint chat completions |
| `POST` | `/v1/completions` | Text completions |
| `POST` | `/api/v1/completions` | Alias text completions |
| `GET` | `/v1/models` | List model yang aktif |
| `GET` | `/api/v1/models` | Alias list model |
| `GET` | `/v1`, `/api/v1` | Root endpoint OpenAI discovery |

---

### C. LLM Providers, Accounts Pool, & Quota (`services/api_router/llm.rs`)

#### 1. Antigravity (Google Gemini Provider)
| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET` | `/api/llm/antigravity/models` | List model Gemini yang tersedia |
| `GET` | `/api/llm/antigravity/accounts` | Mengambil daftar akun Google di quota pool |
| `POST` | `/api/llm/antigravity/accounts` | Menambahkan akun Google baru |
| `POST` | `/api/llm/antigravity/accounts/delete` | Menghapus akun dari pool |
| `POST` | `/api/llm/antigravity/accounts/refresh` | Refresh OAuth access token akun |
| `POST` | `/api/llm/antigravity/accounts/toggle` | Mengaktifkan/menonaktifkan akun di pool |
| `GET / POST`| `/api/llm/antigravity/accounts/quota` | Cek kuota limit per menit & per hari akun |
| `GET` | `/api/llm/antigravity/oauth/url` | Mendapatkan URL login Google OAuth |
| `GET` | `/api/llm/antigravity/oauth/callback`| Callback penerima OAuth code dari Google |
| `POST`| `/api/llm/antigravity/oauth/exchange`| Menukar kode otorisasi menjadi token |

#### 2. Codex (OpenAI Codex Provider)
| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET` | `/api/llm/codex/models` | List model Codex yang tersedia |
| `GET` | `/api/llm/codex/accounts` | Mengambil daftar akun OpenAI Codex di pool |
| `POST` | `/api/llm/codex/accounts` | Menambahkan akun Codex baru |
| `POST` | `/api/llm/codex/accounts/delete` | Menghapus akun dari pool |
| `POST` | `/api/llm/codex/accounts/refresh` | Refresh token OpenAI Codex |
| `POST` | `/api/llm/codex/accounts/toggle` | Mengaktifkan/menonaktifkan akun di pool |
| `GET / POST`| `/api/llm/codex/accounts/quota` | Cek sisa kuota akun Codex |
| `GET` | `/api/llm/codex/oauth/url` | Mendapatkan URL login OpenAI OAuth |
| `GET` | `/api/llm/codex/oauth/callback`| Callback penerima OAuth code OpenAI |
| `POST`| `/api/llm/codex/oauth/exchange`| Menukar kode otorisasi menjadi token |
| `GET` | `/auth/callback`, `/callback` | URL callback alias untuk kompatibilitas redirect OAuth |

#### 3. LLM Token Usage Analytics
| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET` | `/api/llm/usage/summary` | Ringkasan total prompt token, output token, dan perkiraan biaya |
| `GET` | `/api/llm/usage/records` | Daftar record penggunaan token per model dan waktu |
| `POST` | `/api/llm/usage/clear` | Mereset data pencatatan token usage |

---

### D. Automation, Browser Extension, & Remote (`services/api_router/automation.rs`)

| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET / POST` | `/api/skills` | Ambil daftar skill / buat skill automasi |
| `POST` | `/api/skills/delete` | Menghapus skill |
| `POST` | `/api/skills/run` | Menjalankan perintah skill |
| `GET / POST` | `/api/todos` | Mengambil daftar to-do / membuat to-do baru |
| `POST` | `/api/todos/complete` | Menandai to-do selesai |
| `POST` | `/api/todos/delete` | Menghapus to-do |
| `GET` | `/api/recipes` | Mengambil daftar recipe otomasi |
| `POST` | `/api/recipes/save` | Menyimpan recipe |
| `POST` | `/api/recipes/delete` | Menghapus recipe |
| `POST` | `/api/recipes/run` | Menjalankan recipe otomasi |
| `POST` | `/api/browser/snapshot` | Menerima snapshot DOM/Accessibility dari ekstensi browser |
| `POST` | `/api/browser/copied_text`| Sinkronisasi teks yang dicopy user di browser |
| `GET / POST` | `/api/browser/tabs` | Mengambil / memperbarui tab aktif di browser |
| `GET` | `/api/browser/command/next` | Long-poll ekstensi browser untuk mengambil perintah automasi berikutnya |
| `GET` | `/api/remote/tunnel/status`| Cek status binary dan koneksi Cloudflare Tunnel |
| `GET` | `/api/remote/tunnel/check` | Cek ketersediaan binary `cloudflared` |
| `POST` | `/api/remote/tunnel/install`| Download dan pasang binary `cloudflared` |
| `POST` | `/api/remote/tunnel/start` | Menjalankan Cloudflare Tunnel untuk akses remote publik |
| `POST` | `/api/remote/tunnel/stop` | Menghentikan Cloudflare Tunnel |
| `POST` | `/api/remote/active` | Touch remote heartbeat |
| `POST` | `/api/remote/text` | Mengirim teks input dari client remote |

---

### E. Media, Audio, TTS/STT, & Assets (`services/api_router/media.rs` & `server.rs`)

| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET / POST` | `/api/poses` | Ambil atau simpan konfigurasi pose VRM |
| `POST` | `/api/tuner/preview` | Preview pose via HTTP payload |
| `POST` | `/api/tuner/live` | Update live tuning bones via HTTP |
| `GET` | `/api/music/check` | Cek ketersediaan yt-dlp & player musik |
| `POST` | `/api/music/install-ytdlp` | Download & instalasi binary yt-dlp |
| `GET` | `/api/music/list` | Mengambil daftar file musik yang ada di folder musik |
| `GET` | `/api/music/status` | Mengambil status pemutaran musik saat ini |
| `POST` | `/api/music/control` | Kontrol musik: `play`, `pause`, `resume`, `stop`, `set_volume` |
| `POST` | `/api/music/download` | Download musik dari URL YouTube |
| `POST` | `/api/music/delete` | Menghapus file musik |
| `GET` | `/api/music/stream?filename=...` | Stream file audio musik (mendukung HTTP Range header `bytes=start-end`) |
| `GET` | `/music/*path` | Path alias untuk streaming file audio musik |
| `POST` | `/api/stt/transcript` | Menerima transkripsi audio STT |
| `POST` | `/api/tts/state` | Update status audio player TTS di frontend |
| `POST` | `/api/tts/synthesize` | Meminta backend mensintesis teks menjadi audio WAV |
| `GET` | `/api/tts/audio?id=...` | Mengambil file WAV audio TTS hasil sintesis |
| `POST` | `/api/tts/evict` | Menghapus buffer audio TTS yang sudah selesai diputar |
| `POST` | `/api/tts/ack` | Alias evict audio |
| `GET` | `/api/assets/info` | Informasi avatar aktif, model VRM, dan reference voice |
| `GET` | `/api/assets/file/*path` | Mengambil asset statis lokal |
| `POST` | `/api/upload/vrm` | Upload model 3D `.vrm` |
| `POST` | `/api/upload/avatar` | Upload gambar avatar AI |
| `POST` | `/api/upload/refvoice` | Upload audio referensi suara untuk clone/TTS |
| `POST` | `/api/delete/refvoice` | Menghapus file audio referensi suara |

---

### F. OpenAI-Compatible Standard Proxy (`server.rs`)

Backend bertindak sebagai proxy kompatibel dengan OpenAI API sehingga tools eksternal (seperti Claude Dev, Continue, Aider, atau Zed) dapat langsung mengakses model Antigravity atau Codex:

| Method | Endpoint | Deskripsi |
|---|---|---|
| `POST` | `/v1/chat/completions` | Chat completions (mendukung mode regular JSON dan SSE streaming) |
| `POST` | `/api/v1/chat/completions` | Alias endpoint chat completions |
| `POST` | `/v1/completions` | Text completions |
| `POST` | `/api/v1/completions` | Alias text completions |
| `GET` | `/v1/models` | List model yang aktif |
| `GET` | `/api/v1/models` | Alias list model |
| `GET` | `/v1`, `/api/v1` | Root endpoint OpenAI discovery |
