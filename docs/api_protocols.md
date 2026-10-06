# Ragasukma API & Protocol Architecture Specification

Dokumen ini merangkum seluruh pemetaan protokol komunikasi antara client (Desktop Web, Mobile Companion, Extension, Tools) dan backend (Axum + Tokio async).

---

## 1. WebSocket Protocol (`/api/ws`)
Protokol duplex real-time sub-millisecond untuk streaming event, status visual, aksi AI, dan interaktivitas tanpa delay / buffering proxy (seperti Cloudflare Tunnel).

| Channel / Event | Tipe Aliran | Deskripsi & Payload | Digunakan Oleh |
| :--- | :---: | :--- | :--- |
| `event: "state"` | Server $\rightarrow$ Client | Status visual avatar AI (`state`, `emotion`, `voice_muted`, `mic_muted`, `is_running`). | Desktop Web, Mobile Companion |
| `event: "action"` | Server $\rightarrow$ Client | Perintah aksi realtime dari AI (`assistant:speak`, `music:play`, `recipe:run`, dll.). | Desktop Web, Mobile Companion |
| `event: "log"` | Server $\rightarrow$ Client | Aliran real-time log terminal / aktivitas console backend (zero polling). | Desktop Web (Logs Tab), Mobile Companion |
| `event: "model_status"` | Server $\rightarrow$ Client | Progres download & validasi model LLM/TTS/STT lokal. | Desktop Web (Settings Tab), Mobile Companion |
| `pose_tuner_update` | Client $\rightarrow$ Server | Update posisi tulang 3D (60 FPS) secara instan ke renderer native wgpu. | Desktop Web (3D Pose Tuner) |

---

## 2. Standard Request-Response (REST HTTP / JSON)
Digunakan untuk operasi sinkron, konfigurasi, upload file, transfer binary media, dan query data state.

### A. State & System Command
| Endpoint | Method | Fungsi |
| :--- | :---: | :--- |
| `/api/state` | `GET` | Mengambil snapshot state visual avatar saat ini. |
| `/api/cmd` | `POST` | Mengirim perintah kontrol (`toggle_mic`, `toggle_voice`, `open_menu`, dll.). |
| `/api/claim` | `POST` | Mengklaim ownership session remote tab/client. |

### B. Konfigurasi & Model
| Endpoint | Method | Fungsi |
| :--- | :---: | :--- |
| `/api/config` | `GET` | Membaca konfigurasi `config-ai.json` (auto-sync). |
| `/api/config` | `POST` | Menyimpan perubahan konfigurasi `config-ai.json` (hot-reload native & web). |
| `/api/model-status` | `GET` | Status kesiapan model AI. |

### C. Voice, Audio & TTS
| Endpoint | Method | Fungsi |
| :--- | :---: | :--- |
| `/api/tts/synthesize` | `POST` | Mensintesis teks menjadi audio (WAV). |
| `/api/tts/audio` | `GET` | Mengambil chunk audio TTS terkompresi (Opus / Ogg) berdasarkan `audio_id`. |
| `/api/music/list` | `GET` | Daftar lagu / audio background yang tersedia. |
| `/api/music/play` | `POST` | Memutar background music. |
| `/api/music/file` | `GET` | Streaming file audio musik via HTTP range request. |

### D. Histori & Logs
| Endpoint | Method | Fungsi |
| :--- | :---: | :--- |
| `/api/histories/dates` | `GET` | Daftar tanggal percakapan yang tersimpan. |
| `/api/histories` | `GET` | Mengambil histori percakapan Markdown berdasarkan tanggal. |
| `/api/histories/delete` | `POST` | Menghapus arsip histori tanggal tertentu. |
| `/api/logs` | `GET` | Snapshot log console terkini saat inisialisasi tab Logs. |
| `/api/logs/clear` | `POST` | Membersihkan log console. |

### E. Browser Integration & Extension
| Endpoint | Method | Fungsi |
| :--- | :---: | :--- |
| `/api/browser/snapshot` | `POST` | Ekstensi browser mengirimkan DOM snapshot & struktur tab aktif ke backend. |
| `/api/browser/tabs` | `POST` | Ekstensi browser memperbarui daftar tab browser yang terbuka. |
| `/api/browser/tabs` | `GET` | Membaca daftar tab browser yang tersinkronisasi. |
| `/api/browser/command/next` | `GET` | Ekstensi mengambil perintah otomatisasi / navigasi browser berikutnya. |

### F. Aset & Media Management
| Endpoint | Method | Fungsi |
| :--- | :---: | :--- |
| `/api/assets/list` | `GET` | Menampilkan daftar aset lokal di folder `assets/`. |
| `/api/assets/upload` | `POST` | Upload file aset baru (avatar image, VRM 3D, background music). |
| `/api/assets/delete` | `POST` | Menghapus file aset. |
| `/api/poses` | `GET` / `POST` | Membaca dan menyimpan konfigurasi pose 3D (`poses.json`). |
| `/api/skills` | `GET` / `POST` | Membaca dan memperbarui instruksi skill (`skill-command.md`). |

### G. Remote Interaction
| Endpoint | Method | Fungsi |
| :--- | :---: | :--- |
| `/api/remote/text` | `POST` | Mengirim pesan teks / prompt dari ponsel / client remote ke LLM pipeline. |
