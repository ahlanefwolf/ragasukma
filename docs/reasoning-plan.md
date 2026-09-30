# 🧠 Rencana Pengembangan: Reasoning & Multi-Step Automation (Browser & Desktop)

Dokumen ini merangkum analisis arsitektur, komparasi industri, dan rencana implementasi kapabilitas **Reasoning & Multi-Step Action Chaining** pada **Ragasukma AI Voice Assistant**.

---

## 1. Analisis Komparasi: Ragasukma vs JEV (Browser-Use)

| Aspek | Browser-Use / JEV (`browser-use/jev-ultrafast`) | Ragasukma Browser Engine |
| :--- | :--- | :--- |
| **Lingkungan Eksekusi** | Browser terisolasi / headless via Playwright di server/sandbox. | Browser Chromium harian asli pengguna di desktop lokal. |
| **Mekanisme Persepsi** | VLM berbasis screenshot visual piksel (Vision Model) yang memprediksi koordinat `(x, y)` elemen. | Snapshot Semantic DOM lokal (*Ground of Truth* tree: inputs, buttons, texts, form state). |
| **Kecepatan / Latensi** | Bergantung pada kompresi gambar, upload ke cloud VLM, dan inferensi koordinat (~1–3 detik/langkah). | Sangat cepat (< 100ms) karena representasi teks DOM dievaluasi langsung di backend Rust lokal. |
| **Keandalan Klik/Input** | Rentan meleset jika skala resolusi, scroll, atau rendering kanvas berubah. | 100% deterministik via shadow-DOM traversal, label matching, dan trigger event native browser. |
| **Integrasi Asisten** | Standalone web automation agent (text/code-driven). | Terhubung langsung ke suara (STT/TTS 24kHz), 3D VRM Avatar lipsync, dan OS desktop control. |

---

## 2. Karakteristik Kanal Automasi: Browser vs Desktop

### A. Web Browser (High-Precision DOM Chaining)
- **Kelebihan**: Akses langsung ke struktur DOM aktif (metadata elemen, ID, placeholder, teks tombol, nilai input).
- **Kapabilitas Chaining**: Dapat langsung mengeksekusi multi-step reasoning dalam 1 turn tanpa jeda, misalnya:
  ```text
  [CMD:browser:click_text|Masuk] [CMD:browser:type|user@email.com] [CMD:browser:enter]
  ```
- **Kestabilan**: Tidak terpengaruh perubahan tata letak layar atau window focus OS.

### B. OS Desktop (OCR & Coordinate Perception)
- **Tantangan**: Bergantung pada visual screenshot OCR / bounding box OS.
- **Karakteristik**: Setelah satu klik dilakukan, tampilan layar (window baru/dialog) bisa berubah seketika.
- **Strategi**: Membutuhkan jeda re-scan visual antar langkah (`desktop:scan_visual`) sebelum melangkah ke tindakan berikutnya.

---

## 3. Rencana Arsitektur Reasoning Multi-Step

### 1. Silent Thought Preservation
- Memanfaatkan token pemikiran model reasoning (misal `<think>...</think>` pada DeepSeek-R1 / Qwen-2.5 atau `reasoning_content` pada API).
- Blok reasoning tetap dianalisis oleh LLM untuk menentukan urutan langkah, namun otomatis disaring agar **tidak dibacakan oleh suara TTS**.
- Suara TTS hanya mengonfirmasi intisari tindakan dengan gaya percakapan yang hangat dan singkat.

### 2. Sequential Action Chaining di Backend
- Backend Rust (`skill_runner.rs`) telah mendukung chaining multi-command via regex iteratif `RE_SKILL.captures_iter()`.
- Menambahkan jeda mikro dinamis (misal 50–150ms) antar aksi browser berantai agar event DOM (`input`, `change`, `focus`) tersinkronisasi sempurna dengan JavaScript frontend situs tujuan.

### 3. Two-Step Feedback Loop (Action -> State Verification)
- Untuk alur yang kompleks (misal pencarian lalu pemilihan link hasil):
  1. **Langkah 1**: AI mengirim aksi browser / desktop.
  2. **Langkah 2**: Snapshot DOM / window diperbarui seketika.
  3. **Langkah 3**: Sistem memasukkan snapshot state baru ke memory konteks berikutnya untuk verifikasi keberhasilan aksi.
