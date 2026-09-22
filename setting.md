# ⚙️ PANDUAN KONFIGURASI SISTEM RAGASUKMA (`setting.md`)

Seluruh konfigurasi sistem dan preferensi pengguna bersumber tunggal dan terhubung langsung ke file **`config-ai.json`**. Tidak ada nilai hardcode ganda—apapun yang tersimpan di `config-ai.json` adalah yang dieksekusi oleh Backend Rust dan dibaca oleh Dashboard Frontend.

---

## 📌 1. Skema Parameter `config-ai.json`

| Parameter | Tipe Data | Deskripsi |
| :--- | :--- | :--- |
| **`assistant_name`** | `String` | Nama panggilan asisten AI |
| **`user_callsign`** | `String` | Panggilan asisten kepada pengguna |
| **`personality_role`** | `String` | Peran persona (`partner`, `assistant`, `friend`, `tsundere`, `maid`) |
| **`assistant_gender`** | `String` | Gender asisten (`female` / `male`) |
| **`llm_provider`** | `String` | Provider otak AI: **`local`** (Engine Lokal Ragasukma) atau **`custom`** (User AI Assistant) |
| **`llm_model`** | `String` | Model yang dipilih untuk AI Assistant |
| **`llm_base_url`** | `String` | Endpoint URL runtime custom (hanya diisi saat mode `custom`) |
| **`llm_api_key`** | `String` | API Key kustom (opsional jika pakai Ollama/LM Studio lokal) |
| **`groq_api_key`** | `String` | API Key Groq untuk ultra-fast Whisper STT |
| **`stt_engine`** | `String` | Engine Speech-to-Text (`google` / `groq`) |
| **`voice_mode`** | `String` | Mode interaksi suara (`auto` / `push_to_talk`) |
| **`mic_sensitivity`** | `String` | Sensitivitas mikrofon (Level 0 - 6) |
| **`vad_silence_limit`** | `String` | Batas jeda hening VAD sebelum AI mulai merespon (dalam detik) |
| **`noise_reduction_level`**| `i32` | Level peredam bising audio |
| **`tts_engine`** | `String` | Engine Text-to-Speech (`google` / `elevenlabs` / `edge`) |
| **`tts_lang`** | `String` | Bahasa suara TTS (`id` = Bahasa Indonesia) |
| **`tts_speed`** | `f32` | Kecepatan bicara asisten suara |
| **`tts_eleven_labs_key`** | `String` | API Key ElevenLabs (jika memilih engine ElevenLabs) |
| **`elevenlabs_voice_id`** | `String` | ID Suara ElevenLabs |
| **`avatar_type`** | `String` | Tipe avatar visual (`vrm` 3D atau `2d` sprite) |
| **`always_on_top`** | `bool` | Jendela overlay 3D selalu berada di atas aplikasi lain |
| **`voice_interruption`** | `bool` | Memotong suara AI saat user mulai berbicara (barge-in) |

---

## 🧠 2. Perbedaan Dua Mode LLM Provider

### A. Mode `local` (Ragasukma Engine)
* Routing, rotasi token, dan endpoint internal dikelola otomatis oleh sistem Ragasukma.
* Parameter `llm_base_url` dan `llm_api_key` tidak dipakai / dibiarkan kosong.
* AI Assistant memilih model langsung dari katalog internal (semua berawalan `ag/...`).

### B. Mode `custom` (User AI Assistant)
* Terhubung ke runtime mandiri pengguna (seperti Ollama, LM Studio, 9router, OpenAI, Groq, dll).
* Sistem membaca `llm_base_url`, `llm_model`, dan `llm_api_key` langsung dari `config-ai.json` tanpa pembajakan atau fallback ke engine internal.

---

## 🌐 3. Endpoint API Bawaan Ragasukma untuk Klien Eksternal

Aplikasi eksternal (Cursor, VS Code Cline/Continue, OpenWebUI) yang ingin memakai engine Ragasukma:
* **Base URL**: `http://127.0.0.1:42128/v1`
* **Chat Endpoint**: `http://127.0.0.1:42128/v1/chat/completions`
* **Models Endpoint**: `http://127.0.0.1:42128/v1/models`
* **Format Model**: Wajib berawalan `ag/...`
