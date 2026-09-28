# Dokumentasi Sistem Ekspresi & Lip-Sync VRM (Ragasukma)

Dokumen ini menjelaskan standar ekspresi wajah (facial expressions), kedipan mata alami (natural eye blinking), dan sinkronisasi bibir prosedural (procedural lip-sync) yang digunakan pada model avatar VRM 1.0, baik di **Native Desktop (Rust WGPU)** maupun **Mobile Web Client (Three-VRM)**.

---

## 1. Standar 8 Emosi Kanonikal

Sistem membatasi emosi ke **8 emosi kanonikal resmi** yang dipetakan langsung ke blend shape / morph target standar VRM 1.0 (`char.vrm`):

| No | Tag Emosi (Prompt LLM) | Kode Internal | Bobot Morph Target (Blend Shapes) | Visual Wajah Avatar |
|:---|:-----------------------|:--------------|:-----------------------------------|:---------------------|
| 1 | `*senang*` / `*happy*` | `happy` | `happy: 0.85` | Tersenyum manis, sudut mulut terangkat, mata menyipit ramah dan ceria. |
| 2 | `*malu*` / `*shy*` | `shy` | `happy: 0.40`<br>`blink: 0.30` | Tersenyum malu-malu dengan mata sedikit sayu/setengah terpejam (*blushing look*). |
| 3 | `*semangat*` / `*excited*` | `excited` | `blinkLeft: 0.85`<br>`happy: 0.70` | Mengedipkan satu mata (*wink*) sebelah kiri dengan senyum lebar penuh antusiasme. |
| 4 | `*kaget*` / `*surprised*` | `surprised` | `surprised: 0.85` | Mata terbuka lebar/membulat dan mulut sedikit ternganga terkejut. |
| 5 | `*mikir*` / `*thinking*` | `thinking` | `ou: 0.35`<br>`sad: 0.20` | Alis sedikit mengernyit bingung/penasaran dengan bibir membentuk huruf "U" (*pout/mikir*). |
| 6 | `*sedih*` / `*sad*` | `sad` | `sad: 0.85` | Alis turun ke arah luar, sudut bibir melengkung ke bawah, tatapan murung. |
| 7 | `*marah*` / `*angry*` | `angry` | `angry: 0.85` | Alis menukik tajam ke tengah bawah, mata menyipit tajam, ekspresi kesal/merengut. |
| 8 | `*netral*` / `*neutral*` | `neutral` | *(Semua morph: 0.0)* | Wajah dasar asli model (*100% base mesh*), mata segar terbuka penuh, tatapan santai. |

---

## 2. Alur Deteksi & Ekstraksi Emosi

1. **Prompt LLM yang Ketat (`src/llm/prompt.rs`):**
   - AI diwajibkan selalu mengawali kata pertama responnya dengan salah satu dari 8 tag bintang resmi di atas (contoh: `*senang* Halo kak, ada yang bisa kubantu?`).
2. **Pencocokan Deterministik (`src/ui/renderer/expressions.rs`):**
   - Fungsi `detect_emotion(text)` mengekstrak tag format bintang (`*tag*`) atau tanda kurung siku (`[tag]`).
   - Keyword heuristik bebas percakapan (seperti *makasih*, *maaf*, *sayang*) sudah dihapus untuk mencegah *false-positive* yang tidak diinginkan.
3. **Pembersihan Teks TTS (`src/audio/tts.rs`):**
   - Filter `clean_text_for_speech` otomatis menghapus tag emosi sebelum teks dikirim ke mesin suara (Web Speech API / Sherpa-ONNX / Kokoro), sehingga tag tidak terbaca sebagai suara.

---

## 3. Natural Eye Blinking (Kedipan Mata Otomatis)

Avatar memiliki sistem kedipan mata prosedural agar tidak terlihat kaku seperti patung:
- **Interval Kedip:** 
  - Saat `Idle`: Kedipan terjadi setiap **~5.2 detik**.
  - Saat `Speaking`: Kedipan terjadi setiap **~6.2 detik** (agar tidak bertabrakan dengan artikulasi bicara).
- **Durasi Kedip:** ~0.24 detik (240 milidetik) menggunakan kurva sinus halus $\sin^{1.5}(t \times \pi)$.
- **Peredaman Dinamis (*Emotion Damping*):** 
  - Jika avatar sedang dalam emosi kuat (seperti *kaget*, *marah*, *semangat*), intensitas kedipan otomatis diredam sebesar **40%** agar tidak merusak ekspresi utama mata.

---

## 4. Procedural Lip-Sync Vokal (A-I-U-E-O)

Saat avatar berada dalam status visual `Speaking`:
- Sistem secara prosedural menggerakkan blend shape vokal vokal standar:
  - `aa` (Bukaan mulut lebar vokal A)
  - `ih` (Bukaan mulut menyamping vokal I)
  - `ou` (Mulut membulat maju vokal U)
  - `ee` (Mulut terbuka sedang vokal E)
  - `oh` (Mulut membulat lebar vokal O)
- **Modulasi Waktu:** Pergantian vokal terjadi dengan variasi frekuensi halus dan amplitudo yang responsif, menghasilkan gerakan mulut yang hidup dan alami saat suara TTS berbunyi.

---

## 5. Konsistensi 100% Native Desktop vs Mobile

Implementasi di native desktop Rust WGPU ([`expressions.rs`](file:///home/headway/MyWork/project/node/ai-voice/src/ui/renderer/expressions.rs)) dan Three.js web mobile client ([`AvatarViewer.tsx`](file:///home/headway/MyWork/project/node/ai-voice/web/src/mobile/AvatarViewer.tsx)) menggunakan parameter bobot, nama morph target, dan formula animasi yang **identik satu sama lain**.
