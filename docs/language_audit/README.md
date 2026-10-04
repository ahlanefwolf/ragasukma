# Codebase Language & String Audit (`src/`)

Dokumentasi audit menyeluruh untuk seluruh komentar (`//`, `///`, `/* */`) dan string literals (`"..."`) di dalam backend Rust (`src/`), dipecah per modul untuk mempermudah pengecekan dan verifikasi bahasa.

## 📊 Ringkasan Statistik per Modul

| Modul | File Audit | Total File `.rs` | Total Baris Komentar | Total String Literals | Status Bahasa |
|:---|:---|:---:|:---:|:---:|:---:|
| **AUDIO** | [audio.md](./audio.md) | 4 | 100 | 277 | 100% English Verified |
| **ENGINE** | [engine.md](./engine.md) | 37 | 460 | 1777 | 100% English Verified |
| **LLM** | [llm.md](./llm.md) | 16 | 103 | 1147 | 100% English Verified |
| **ROOT** | [root.md](./root.md) | 5 | 66 | 361 | 100% English Verified |
| **SERVICES** | [services.md](./services.md) | 17 | 56 | 391 | 100% English Verified |
| **UI** | [ui.md](./ui.md) | 12 | 136 | 506 | 100% English Verified |
| **TOTAL** | **6 Modul** | **91** | **921** | **4459** | **100% English Verified** |

## 📁 Struktur Dokumen Audit

- **[audio.md](./audio.md)**: Berisi daftar rinci baris per baris untuk modul `src/audio` (4 file, 100 komentar, 277 string).
- **[engine.md](./engine.md)**: Berisi daftar rinci baris per baris untuk modul `src/engine` (37 file, 460 komentar, 1777 string).
- **[llm.md](./llm.md)**: Berisi daftar rinci baris per baris untuk modul `src/llm` (16 file, 103 komentar, 1147 string).
- **[root.md](./root.md)**: Berisi daftar rinci baris per baris untuk modul `src/root` (5 file, 66 komentar, 361 string).
- **[services.md](./services.md)**: Berisi daftar rinci baris per baris untuk modul `src/services` (17 file, 56 komentar, 391 string).
- **[ui.md](./ui.md)**: Berisi daftar rinci baris per baris untuk modul `src/ui` (12 file, 136 komentar, 506 string).

## 🔍 Catatan Pengecualian Khusus (Non-Issue / Intentional)

1. **Kamus Multibahasa OS App (`src/engine/app_scanner.rs`)**:
   - Mengandung kata benda generik seperti `"peramban"`, `"kode"`, `"teks"`, `"berkas"` bersama bahasa dunia lainnya sebagai dictionary matcher perintah suara pengguna.
2. **Unit Test Regex & Typo Matching (`src/llm/cleaner.rs`, `src/engine/perception/utils.rs`, `src/audio/supertonic.rs`)**:
   - Berada dalam blok `#[cfg(test)]` untuk memverifikasi parser pembersih tanda baca, similarity phonetic, dan sintesis ONNX lokal.

---
_Audit generated & verified automatically._
