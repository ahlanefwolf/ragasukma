# 📡 Spesifikasi API Request/Response & Server-Sent Events (SSE) — Ragasukma

Dokumen ini mendokumentasikan spesifikasi seluruh endpoint REST HTTP dan **Server-Sent Events (SSE)** yang aktif pada backend Rust (`127.0.0.1:42128`) beserta struktur payload, interval waktu, ping keepalive, dan arsitektur pemutar musik lokal.

---

## ⚡ 1. Server-Sent Events (SSE) Streams

Semua koneksi SSE menggunakan header HTTP standar:
```http
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-cache, no-transform
Connection: keep-alive
Access-Control-Allow-Origin: *
X-Accel-Buffering: no
```

> **Catatan Socket Timeout**: Begitu route SSE terdeteksi (`url.starts_with("/api/events")`), TCP stream di backend disetel `read_timeout: None` dan `write_timeout: None` agar koneksi long-lived tidak diputus oleh timeout socket TCP normal.

---

### 🟢 A. App Lifecycle & Action Stream (`/api/events/state`)
- **Tujuan**: Memantau status visual asisten AI (`idle` $\leftrightarrow$ `listening` $\leftrightarrow$ `thinking` $\leftrightarrow$ `speaking` $\leftrightarrow$ `shutdown`) serta memancarkan instruksi aksi AI (*Client Actions* seperti pemicu musik atau sinyal takeover session).
- **Klien yang Mendengarkan**: Global dashboard (aktif di semua tab sepanjang session hidup).
- **Loop Tick Rate**: `100 ms` (`thread::sleep(Duration::from_millis(100))`).
- **Heartbeat / Ping**: Mengirim `: ping\n\n` (8 byte) setiap **3.000 ms (3 detik)** jika tidak ada perubahan state/aksi.
- **Client Reconnect Delay**: `1.500 ms (1.5 detik)` jika koneksi TCP socket terputus mendadak.

#### 1. Event: `state` (Murni Status Visual AI)
*Hanya terkirim saat status visual AI berganti*:
```http
event: state
data: {"state":"idle"}
```
*Nilai status*: `"idle"` | `"listening"` | `"thinking"` | `"speaking"` | `"shutdown"`

#### 2. Event: `action` (Perintah Aksi AI & Session Claim)
*Hanya terkirim saat AI mengeksekusi skill atau ada tab lain yang mengambil alih session*:
```http
event: action
data: {"action":"music:play","payload":"Setia Band - Asmara ｜ Official Music Video"}
```
*Daftar Aksi*:
- `music:play` $\rightarrow$ `payload`: Judul lagu / nama file yang siap diputar.
- `music:pause` $\rightarrow$ Jeda lagu.
- `music:resume` $\rightarrow$ Lanjutkan lagu.
- `music:next` $\rightarrow$ Beralih ke lagu berikutnya.
- `music:stop` $\rightarrow$ Hentikan pemutaran musik.
- `session:claimed` $\rightarrow$ `payload`: Token session baru (mematikan tab lama secara instan).

---

### 🎙️ B. Menu Realtime Audio Spectrum Stream (`/api/events/menu`)
- **Tujuan**: Visualisasi spektrum mikrofon realtime (`mic_level`) dan VU meter.
- **Klien yang Mendengarkan**: **Hanya `MenuTab`**.
  - Saat tab berpindah (Music, Tuner, Settings, Skills, Logs), frontend memanggil `api.closeMenuSSE()` dan backend mendeteksi `active_tab != "menu"`, langsung memutuskan loop dan mematikan thread socket untuk menghemat CPU & RAM.
- **Loop Tick Rate**: `60 ms` (~16.6 FPS) di backend Rust.
- **Client-Side Smoothing**: Frontend SolidJS menginterpolasi target level menggunakan loop `requestAnimationFrame` (**60 FPS Lerp**: *Attack 0.35*, *Decay 0.18*) agar animasi VU meter dan bar spektrum bergerak sangat mulus tanpa membebani network/CPU.
- **Heartbeat / Ping**: Mengirim `: ping\n\n` setiap **3.000 ms (3 detik)** jika mic hening/level stagnan.

#### Event: `mic` (Realtime Mic Level 0–100%)
*Hanya terkirim saat level mic berubah atau bernilai > 0*:
```http
event: mic
data: {"level":42}
```

---

## 📋 Ringkasan Interval & Timeout SSE

| Stream Endpoint | Target Tab / Komponen | Backend Tick Rate | Client Render | Ping Keepalive | Idle Reconnect |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`/api/events/state`** | Global (Semua Tab) | `100 ms` | Event-Driven | `3.000 ms` (3 detik) | `1.500 ms` |
| **`/api/events/menu`** | Tab Menu Saja *(auto-close on unmount)* | `60 ms` (~16.6 FPS) | **60 FPS** (RAF Lerp) | `3.000 ms` (3 detik) | Managed by Mount |

---

## 🎵 2. Arsitektur Pemutar Musik (Local Web Player & Range Streaming)

Dashboard web berjalan di browser lokal dan memegang daftar library lagu dari `GET /api/music/list`. Seluruh pemutaran audio dikelola **100% secara mandiri di browser**:
1. File audio diputar langsung oleh browser via HTML5 `<audio src="/music/nama_file.m4a">`.
2. **HTTP 206 Partial Content (Range Request)**: Backend menyajikan potongan buffer audio (~256 KB - 1 MB) murni saat browser meminta chunk. Tidak ada thread/loop streaming audio yang membebani server.
3. **Pencarian Cerdas & Auto Background Download**:
   - Saat user memberi perintah suara (contoh *"Putar Setia Band Asmara"*), backend mengecek metadata lokal terlebih dahulu.
   - **Jika ada di lokal**: Backend langsung mengirim event SSE `music:play` dengan judul pasti.
   - **Jika belum ada**: Backend mendaftarkan proses ke Background Task Worker `yt-dlp`. Begitu selesai diunduh, backend otomatis mengirim event SSE `music:play` sehingga lagu baru langsung diputar seketika tanpa memainkan lagu sembarangan.

---

## 🌐 3. Seluruh Endpoint REST API (Request / Response)

### A. Session & Single-Instance Mutex
- **`POST /api/session/claim`** (Dipanggil murni **1x** saat tab web pertama kali dibuka)
  - Request: `{"token": "sess_xxx", "active_tab": "menu"}`
  - Response: `{"success": true}`
  - *Efek*: Mendaftarkan tab aktif baru dan mematikan tab lama via broadcast `session:claimed`.
- **`POST /api/session/tab`** (Dipanggil saat user mengklik menu tab baru)
  - Request: `{"active_tab": "skills"}`
  - Response: `{"success": true}`
  - *Efek*: Memberitahu backend tab yang aktif agar stream `/api/events/menu` otomatis diputus jika bukan tab menu.

### B. Control & App State
- **`GET /api/state`** (Dipanggil 1x saat `MenuTab` mount untuk inisialisasi tombol)
  - Response:
    ```json
    {
      "state": "idle",
      "is_running": true,
      "is_paused": false,
      "voice_muted": false,
      "mic_muted": false,
      "mic_level": 0
    }
    ```
- **`POST /api/state`**
  - Request: `{"state": "listening", "is_running": true, "is_paused": false, "voice_muted": false, "mic_muted": false}`
  - Response: `{"status": "ok"}`
- **`POST /api/cmd`**
  - Request: `{"cmd": "toggle_run" | "toggle_pause" | "toggle_voice_mute" | "toggle_mic_mute" | "start_chat" | "stop_chat" | "reset_session" | "open_browser" | "exit"}`
  - Response: `{"status": "ok"}`

### C. Music Storage & YouTube Downloader
- **`GET /music/<filename>`** $\rightarrow$ Menyajikan file audio lokal untuk elemen `<audio>` (Mendukung HTTP 206 Partial Content / Range Request).
- **`GET /api/music/list`** $\rightarrow$ Objek metadata lokasi folder musik dan daftar trek lagu:
  ```json
  {
    "music_dir": "C:\\Project\\ragasukma\\assets\\music",
    "tracks": [
      {
        "id": "kygIGzfuxEE",
        "title": "Setia Band - Asmara ｜ Official Music Video",
        "uploader": "Trinity Optima Production",
        "duration": 250,
        "filename": "Setia Band - Asmara ｜ Official Music Video [kygIGzfuxEE].m4a",
        "thumbnail": "https://i.ytimg.com/vi/kygIGzfuxEE/hqdefault.jpg"
      }
    ]
  }
  ```
- **`POST /api/music/download`** $\rightarrow$ Request: `{"query": "judul lagu atau url youtube"}` $\rightarrow$ Memicu background worker `yt-dlp`.
- **`POST /api/music/delete`** $\rightarrow$ Request: `{"filename": "lagu.m4a"}` $\rightarrow$ Menghapus file audio dari disk.
- **`GET /api/music/check`** $\rightarrow$ Cek ketersediaan dan versi binary `yt-dlp`.
- **`POST /api/music/install-ytdlp`** $\rightarrow$ Auto-install binary `yt-dlp` portable.

### D. Config, Poses & 3D Avatar Tuner
- **`GET /api/config`** $\rightarrow$ Mengambil isi data `config.json`.
- **`POST /api/config`** $\rightarrow$ Request: `UserConfig` JSON $\rightarrow$ Menyimpan konfigurasi ke disk.
- **`GET /api/poses`** $\rightarrow$ Membaca konfigurasi pose tulang avatar 3D `poses.json`.
- **`POST /api/poses`** $\rightarrow$ Menyimpan perubahan tulang avatar ke disk.
- **`POST /api/tuner/preview`** $\rightarrow$ Update transformasi tulang in-memory tanpa menulis ke disk.
- **`POST /api/tuner/live`** $\rightarrow$ Update transformasi live saat slider digeser (menggunakan `AbortController` di frontend untuk drop frame usang).

### E. Skills & Automation
- **`GET /api/skills`** $\rightarrow$ Mengambil daftar skill kustom dari `skill-command.md`.
- **`POST /api/skills`** $\rightarrow$ Request: `{"content": "markdown string"}` $\rightarrow$ Menyimpan pembaruan skill ke file markdown.
- **`POST /api/skills/delete`** $\rightarrow$ Request: `{"index": 0}` $\rightarrow$ Menghapus entri skill.
- **`POST /api/skills/run`** $\rightarrow$ Request: `{"command": "calc.exe"}` $\rightarrow$ Menjalankan perintah shell OS secara aman.
- **`GET /api/system/apps`** $\rightarrow$ Mengambil daftar aplikasi yang terpasang di sistem operasi.

### F. Assets & Uploads
- **`GET /api/assets/info`** $\rightarrow$ Informasi file aset aktif (model VRM aktif, 2D avatar idle/talking/thinking/listening).
- **`POST /api/upload/vrm`** $\rightarrow$ Upload file `.vrm` / `.glb` baru.
- **`POST /api/upload/avatar`** $\rightarrow$ Upload gambar avatar PNG baru per slot (`?slot=idle|listening|thinking|talking`).

### G. Terminal Logs
- **`GET /api/logs`** $\rightarrow$ Mengambil riwayat log terminal.
- **`POST /api/logs/clear`** $\rightarrow$ Membersihkan buffer log terminal.
