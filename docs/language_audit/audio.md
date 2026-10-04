# Language Audit: `audio` Module

- **Directory**: `src/audio`
- **Total Files**: 4
- **Total Comments**: 100
- **Total String Literals**: 277

## Table of Contents
- [audio/mod.rs](#audiomodrs) (0 comments, 0 strings)
- [audio/player.rs](#audioplayerrs) (33 comments, 10 strings)
- [audio/supertonic.rs](#audiosupertonicrs) (26 comments, 176 strings)
- [audio/tts.rs](#audiottsrs) (41 comments, 91 strings)

---

## `audio/mod.rs`

- **Path**: `src/audio/mod.rs`
- **Comments**: 0
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

_No string literals found._

---

## `audio/player.rs`

- **Path**: `src/audio/player.rs`
- **Comments**: 33
- **Strings**: 10

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 32 | `///` | Extracts PCM samples and sample_rate from WAV format |
| 93 | `///` | Calculates real-time AIUEO lip weights based on formant frequencies of the currently playing audio |
| 110 | `//` | Take 512-sample window (~12-25 ms) |
| 118 | `//` | 1. RMS Energy |
| 125 | `//` | Noise floor: silence / breath pause between sentences |
| 132 | `//` | 2. Goertzel Filter Power on frekuensi formant vokal utama: |
| 133 | `//` | Low: ~450 Hz (U & O) |
| 134 | `//` | Mid: ~950 Hz (A) |
| 135 | `//` | Mid-High: ~1900 Hz (E) |
| 136 | `//` | High: ~2800 Hz (I) |
| 188 | `//` | Keep stream alive throughout application runtime |
| 204 | `///` | Plays Supertonic AI audio response chunks gaplessly via streaming to PC speakers |
| 211 | `//` | First chunk: initialize new playback session |
| 221 | `//` | Extract PCM sample data and sample_rate for precise AIUEO lip synchronization |
| 236 | `//` | Duck native music if currently playing |
| 250 | `//` | Restore music volume after all AI voice chunks finish and return avatar state to Idle |
| 256 | `//` | New voice thread or stop has already taken over |
| 278 | `//` | Playback complete if sink is already empty AND all chunks have been appended, |
| 279 | `//` | Or if stall exceeds the 8-second fail-safe |
| 312 | `//` | Subsequent chunks (> 0): check if still in active playback session |
| 327 | `//` | Append PCM samples to active voice buffer so lip-sync remains synchronized |
| 336 | `//` | Enqueue audio directly to rodio Sink currently playing previous chunk |
| 357 | `///` | Stops native AI voice playback (e.g. during voice interruption or VAD barge-in) |
| 378 | `///` | Check whether native audio voice is currently playing on PC speakers |
| 396 | `///` | Sends audio play command (state synchronization and stream delegation to Web Dashboard & PiP) |
| 408 | `//` | Play to PC hardware speakers in a separate thread isolated with catch_unwind |
| 409 | `//` | so internal rodio format parser bugs never crash the main conversation session thread |
| 413 | `//` | If Phone Only mode is active (voice_muted = true), music only plays on phone, not on PC |
| 416 | `//` | Use in-memory Cursor buffer so symphonia decoder avoids seek errors during initialization |
| 420 | `//` | If AI is currently speaking when music starts, apply ducked volume directly |
| 439 | `///` | Pause music playback |
| 449 | `///` | Resume music playback |
| 459 | `///` | Stops music playback |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 34 | `RIFF` |
| 34 | `WAVE` |
| 50 | `fmt ` |
| 60 | `data` |
| 300 | `Standby` |
| 301 | `neutral` |
| 404 | `[Music Controller] Playing track '{}' via Web Dashboard & PiP stream` |
| 446 | `[Music Controller] Music paused.` |
| 456 | `▶ [Music Controller] Music resumed.` |
| 469 | `[Music Controller] Music stopped.` |

---

## `audio/supertonic.rs`

- **Path**: `src/audio/supertonic.rs`
- **Comments**: 26
- **Strings**: 176

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 88 | `//` | 1. Config |
| 94 | `//` | 2. Unicode Indexer |
| 100 | `//` | 3. ONNX Sessions (use Level1 optimization & bounded threads to minimize memory footprint) |
| 137 | `//` | 4. Preload 10 Voice Styles (M1-M5, F1-F5) |
| 219 | `//` | 1. Direct key match: M1-M5, F1-F5 |
| 224 | `//` | 2. Name mapping from option.html |
| 258 | `//` | Default female voice F4 |
| 289 | `//` | 1. Duration Predictor |
| 304 | `//` | 2. Text Encoder |
| 318 | `//` | 3. Noisy Latent & Flow Matching |
| 319 | `//` | CRUCIAL GUARD: ONNX vector_estimator model has attention tensor dimension limit = 1000. |
| 320 | `//` | Clamping to .min(960) mathematically guarantees never exceeding 1000 ('1000 by 1065' error). |
| 357 | `//` | 4. Vocoder |
| 375 | `///` | Incrementally synthesizes text per chunk and directly sends WAV bytes to channel sender (streaming TTS queue). |
| 376 | `///` | Return total combined WAV audio bytes for caching and mobile client. |
| 386 | `//` | (chunk_idx, total_chunks, wav_bytes) |
| 446 | `///` | Splits long text into natural sentence/clause chunks so it does not exceed ONNX tensor limits (~140 characters) |
| 523 | `//` | Fallback to last spoken |
| 532 | `//` | Normalise any BCP-47 / ISO-639 tag to a bare two-letter base code. |
| 533 | `//` | Must stay in sync with SUPPORTED_LANGUAGES in AudioTtsSection.tsx and |
| 534 | `//` | resolve_language_info() in llm/prompt.rs. |
| 665 | `//` | Peak Normalization (Digital Headroom Scaling): |
| 666 | `//` | Raw Supertonic-3 neural model produces conservative peak amplitudes (~0.25 - 0.35). |
| 667 | `//` | Find maximum waveform peak and scale linearly to target 0.92 (-0.7 dBFS) |
| 668 | `//` | ensures voice is clear, crisp, and free from clipping/distortion on PC and mobile speakers. |
| 678 | `//` | Target 0.92 provides safe headroom against inter-sample peaks on audio DAC |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 70 | `[Supertonic TTS] Failed to initialize model: {e}` |
| 81 | `Supertonic-3 model directory not found` |
| 83 | `[Supertonic TTS] Loading ONNX neural model from {:?}...` |
| 89 | `onnx/tts.json` |
| 95 | `onnx/unicode_indexer.json` |
| 102 | `{e}` |
| 104 | `{e}` |
| 106 | `{e}` |
| 107 | `onnx/text_encoder.onnx` |
| 108 | `{e}` |
| 111 | `{e}` |
| 113 | `{e}` |
| 115 | `{e}` |
| 116 | `onnx/duration_predictor.onnx` |
| 117 | `{e}` |
| 120 | `{e}` |
| 122 | `{e}` |
| 124 | `{e}` |
| 125 | `onnx/vector_estimator.onnx` |
| 126 | `{e}` |
| 129 | `{e}` |
| 131 | `{e}` |
| 133 | `{e}` |
| 134 | `onnx/vocoder.onnx` |
| 135 | `{e}` |
| 139 | `M1` |
| 139 | `M2` |
| 139 | `M3` |
| 139 | `M4` |
| 139 | `M5` |
| 139 | `F1` |
| 139 | `F2` |
| 139 | `F3` |
| 139 | `F4` |
| 139 | `F5` |
| 140 | `voice_styles` |
| 143 | `{key}.json` |
| 184 | `[Supertonic TTS] Initialized in {:.2?}! ({} voice styles ready)` |
| 205 | `models/supertonic-3` |
| 206 | `../models/supertonic-3` |
| 207 | `/home/headway/MyWork/project/node/ai-voice/models/supertonic-3` |
| 211 | `onnx/vocoder.onnx` |
| 226 | `alex` |
| 227 | `M1` |
| 229 | `james` |
| 230 | `M2` |
| 232 | `robert` |
| 233 | `M3` |
| 235 | `sam` |
| 236 | `M4` |
| 238 | `daniel` |
| 239 | `M5` |
| 242 | `sarah` |
| 243 | `F1` |
| 245 | `lily` |
| 246 | `F2` |
| 248 | `jessica` |
| 249 | `F3` |
| 251 | `olivia` |
| 252 | `F4` |
| 254 | `emily` |
| 255 | `F5` |
| 259 | `F4` |
| 284 | `Empty text after preprocessing` |
| 291 | `{e}` |
| 293 | `text_ids` |
| 294 | `style_dp` |
| 295 | `text_mask` |
| 298 | `duration` |
| 306 | `{e}` |
| 308 | `text_ids` |
| 309 | `style_ttl` |
| 310 | `text_mask` |
| 313 | `text_emb` |
| 339 | `{e}` |
| 343 | `noisy_latent` |
| 344 | `text_emb` |
| 345 | `style_ttl` |
| 346 | `latent_mask` |
| 347 | `text_mask` |
| 348 | `current_step` |
| 349 | `total_step` |
| 352 | `denoised_latent` |
| 359 | `{e}` |
| 361 | `latent` |
| 364 | `wav_tts` |
| 397 | `Voice style '{voice_key}' not found` |
| 402 | `Empty text after chunk splitting` |
| 431 | `[Supertonic TTS] Failed to synthesize sub-chunk "{}": {e}` |
| 439 | `All audio sub-chunks failed to synthesize` |
| 539 | `id` |
| 539 | `id` |
| 540 | `en` |
| 540 | `en` |
| 541 | `ja` |
| 541 | `ja` |
| 542 | `ko` |
| 542 | `ko` |
| 543 | `ar` |
| 543 | `ar` |
| 544 | `bg` |
| 544 | `bg` |
| 545 | `cs` |
| 545 | `cs` |
| 546 | `da` |
| 546 | `da` |
| 547 | `de` |
| 547 | `de` |
| 548 | `el` |
| 548 | `el` |
| 549 | `es` |
| 549 | `es` |
| 550 | `et` |
| 550 | `et` |
| 551 | `fi` |
| 551 | `fi` |
| 552 | `fr` |
| 552 | `fr` |
| 553 | `hi` |
| 553 | `hi` |
| 554 | `hr` |
| 554 | `hr` |
| 555 | `hu` |
| 555 | `hu` |
| 556 | `it` |
| 556 | `it` |
| 557 | `lt` |
| 557 | `lt` |
| 558 | `lv` |
| 558 | `lv` |
| 559 | `nl` |
| 559 | `nl` |
| 560 | `pl` |
| 560 | `pl` |
| 561 | `pt` |
| 561 | `pt` |
| 562 | `ro` |
| 562 | `ro` |
| 563 | `ru` |
| 563 | `ru` |
| 564 | `sk` |
| 564 | `sk` |
| 565 | `sl` |
| 565 | `sl` |
| 566 | `sv` |
| 566 | `sv` |
| 567 | `tr` |
| 567 | `tr` |
| 568 | `uk` |
| 568 | `uk` |
| 569 | `vi` |
| 569 | `vi` |
| 570 | `zh` |
| 570 | `zh` |
| 571 | `id` |
| 579 | `-` |
| 581 | `"` |
| 582 | `'` |
| 585 | ` at ` |
| 586 | `e.g.,` |
| 586 | `for example, ` |
| 587 | `i.e.,` |
| 587 | `that is, ` |
| 589 | `\s+` |
| 601 | `<{lang}>{t}</{lang}>` |
| 652 | `RIFF` |
| 654 | `WAVEfmt ` |
| 662 | `data` |
| 700 | `Supertonic engine should initialize` |
| 702 | `Halo Sukma!` |
| 702 | `F4` |
| 702 | `id` |
| 703 | `Synthesis should succeed` |
| 705 | `RIFF` |
| 706 | `WAVE` |
| 711 | `Halo Mas! Hari ini cuacanya panas banget ya. Jangan lupa banyak minum air putih biar tetap segar dan sehat sepanjang ...` |

---

## `audio/tts.rs`

- **Path**: `src/audio/tts.rs`
- **Comments**: 41
- **Strings**: 91

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 11 | `///` | Voice synthesis is processed natively using Supertonic-3 neural model (ONNX Runtime Rust). |
| 12 | `///` | High-fidelity WAV audio (44.1kHz Hi-Fi) is streamed to all clients (Desktop & Mobile). |
| 19 | `//` | Notes: is_voice_muted indicates desktop speakers are muted (redirected to mobile), |
| 20 | `//` | however 'assistant:speak' voice events must still be broadcast so the mobile client can play them! |
| 42 | `//` | Initialize engine if not yet ready, and synthesize Supertonic-3 audio in streaming chunks |
| 49 | `//` | Once first chunk is ready, directly set avatar & tray status to Speaking |
| 89 | `//` | 1. Stream response sentence, emotion, duration, and audio_url to all clients (Desktop & Mobile) via SSE action |
| 102 | `//` | 2. Synchronize avatar state & AIUEO lip sync: |
| 127 | `//` | 3. Precision passive fail-safe watchdog based on actual WAV duration: |
| 128 | `//` | Calculated directly from duration in seconds of synthesized Supertonic-3 WAV audio. |
| 135 | `//` | Network latency and audio player buffering tolerance grace period (~6 seconds) |
| 143 | `//` | Exit if speech finished normally or a new speech epoch began |
| 150 | `//` | If native PC audio is still actively playing, never force idle! |
| 156 | `//` | If client still sent voice activity within the last 3 seconds, extend timeout |
| 161 | `//` | If duration exceeds actual audio duration plus tolerance buffer |
| 262 | `//` | Footnote reference numbers e.g. (1), (2) |
| 266 | `//` | Smiley emoticons |
| 275 | `//` | 1. Normalize special Unicode characters (curly quotes, ellipsis, dash, bullet) |
| 277 | `//` | “ ” « » -> " |
| 278 | `//` | ‘ ’ -> ' |
| 279 | `//` | … -> , |
| 280 | `//` | — – -> - |
| 281 | `//` | • ▪ ● -> space |
| 283 | `//` | 2. Strip markdown code blocks \`\`\`...\`\`\` so TTS does not read code syntax aloud |
| 286 | `//` | 3. Sanitize raw URLs (https://... / http://... / www...) into "web link" |
| 289 | `//` | 4. Use clean_ai_text to remove tool calls [CMD:...], [SKILL:...], bold **text**, star tags *...*, and backticks |
| 292 | `//` | 5. Clean bracket tags [happy], markdown links [label](url), ignorable parentheses (1), and filter punctuation |
| 297 | `//` | A. Clean square bracket tags [happy], links [label](url), etc. |
| 310 | `//` | If square brackets are followed by a markdown link \`(url)\`, preserve label text and omit the url |
| 312 | `//` | consume '(' |
| 333 | `//` | B. Clean parentheses () if they contain footnote reference numbers (1) or emoticons |
| 358 | `//` | If not ignorable parentheses, skip opening parenthesis so inner text remains readable |
| 368 | `//` | C. Normalize non-spoken markdown characters and punctuation |
| 375 | `//` | remove emoji / unknown characters |
| 379 | `//` | 5. Normalize consecutive dots or ellipsis into comma for smooth TTS intonation |
| 390 | `//` | 6. Tangkal bug word emotion mentah in most start sentence (for example: "happy Halo...", "senang, Halo...", "mikir: H... |
| 408 | `//` | Check whether followed by space, colon, comma, or dash |
| 427 | `//` | 7. Collapse repeated whitespace and clean up punctuation |
| 437 | `//` | Avoid whitespace before period/comma (e.g. "Hello , friend" -> "Hello, friend") |
| 449 | `///` | Accurately calculates duration in seconds from a Supertonic-3 WAV audio file |
| 473 | `///` | Global helper for reactive and non-blocking assistant speech |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 26 | `F4` |
| 31 | `id` |
| 39 | `{:x}` |
| 40 | `/api/tts/audio?id={audio_id}` |
| 53 | `{asst_name_str} Speaking` |
| 73 | `[Supertonic TTS] Failed to synthesize audio: {e}` |
| 91 | `text` |
| 92 | `emotion` |
| 93 | `audio_url` |
| 94 | `duration` |
| 95 | `voice` |
| 96 | `lang` |
| 97 | `native_played` |
| 100 | `assistant:speak` |
| 106 | `{asst_name} Speaking` |
| 120 | `{asst_name} Speaking` |
| 164 | `[TTS Watchdog] Audio playback finished ({exact_audio_secs:.1}s), returning avatar to Standby.` |
| 169 | `Standby` |
| 170 | `neutral` |
| 210 | `http://` |
| 211 | `https://` |
| 212 | `www.` |
| 225 | `web link` |
| 239 | `action:` |
| 239 | `cmd:` |
| 239 | `skill:` |
| 244 | `happy` |
| 245 | `shy` |
| 246 | `excited` |
| 247 | `surprised` |
| 248 | `thinking` |
| 249 | `sad` |
| 250 | `angry` |
| 251 | `neutral` |
| 252 | `active tab` |
| 253 | `active` |
| 267 | `:)` |
| 267 | `:d` |
| 267 | `:3` |
| 267 | `^^` |
| 267 | `^_^` |
| 267 | `:o` |
| 267 | `:(` |
| 267 | `;)` |
| 277 | `"` |
| 278 | `'` |
| 279 | `, ` |
| 280 | ` - ` |
| 381 | `. . .` |
| 381 | `, ` |
| 382 | `. .` |
| 382 | `, ` |
| 383 | `....` |
| 383 | `, ` |
| 384 | `...` |
| 384 | `, ` |
| 385 | `..` |
| 385 | `, ` |
| 392 | `happy` |
| 393 | `shy` |
| 394 | `excited` |
| 395 | `surprised` |
| 396 | `thinking` |
| 397 | `sad` |
| 398 | `angry` |
| 399 | `neutral` |
| 451 | `RIFF` |
| 451 | `WAVE` |
| 479 | `Sukma` |
| 492 | `[happy] Halo apa kabar!` |
| 493 | `Halo apa kabar!` |
| 496 | `excited: Halo apa kabar!` |
| 497 | `Halo apa kabar!` |
| 500 | `angry, kenapa begitu?` |
| 501 | `kenapa begitu.` |
| 504 | `*thinking* sebentar ya` |
| 505 | `sebentar ya` |
| 508 | `*malu-malu* halo juga Kak` |
| 509 | `halo juga Kak` |
| 512 | `Iya dong (1) terima kasih` |
| 513 | `Iya dong terima kasih` |
| 520 | `Ini **sangat penting** lho!` |
| 521 | `Ini sangat penting lho!` |
| 523 | `* Poin pertama` |
| 523 | `Poin pertama` |
| 529 | `Kunjungi https://github.com/hlnsptr untuk info.` |
| 530 | `Kunjungi web link untuk info.` |
| 533 | `Markdown [Dokumentasi](https://example.com) bagus.` |
| 534 | `Markdown Dokumentasi bagus.` |
| 537 | `Jalankan \`\`\`bash\ncargo test\n\`\`\` selesai.` |
| 538 | `Jalankan selesai.` |

---
