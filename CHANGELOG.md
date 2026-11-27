# 📋 Ragasukma Changelog

All notable updates, new features, and bug fixes for the **Ragasukma AI Voice Assistant** application are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and adheres to [Semantic Versioning](https://semver.org/).

---

## [v0.5.3] - 2026-11-27

### ✨ New Features
- **Auto-naming tasks from LLM response** (`ragasukma-core`, `ragasukma-tui`, `ragasukma`): Automatically generates a concise 3–6 word task title from the LLM's first response when the session still has a default placeholder title (`Task #...` / `New Task`), and persists it via `update_session_title`.

### 🎨 UI Improvements (`ragasukma-tui`)
- **Provider View account table**: Separated account email and status into distinct `Email` and `Status` table columns matching `custom_llm_view`.
- **Agent View sidebar**: Removed redundant `[Running]` / `[Idle]` text labels, simplified token counts to `[<count>]`, and truncated overflowing task titles with `...`.

## [v0.5.2] - 2026-11-25

### 🚀 Ecosystem Version Bump
- Unified version bump to `0.5.2` across `ragasukma-core`, `ragasukma`, `ragasukma-tui`, the web dashboard, and the landing page.

### 🐛 Bug Fixes & Provider Reliability (`ragasukma-core`)
- **OAuth refresh parity (`43aedc6`)**: Aligned Codex, Claude, and Antigravity token refresh requests and stream terminal events with `9router-0.5.123` (JSON body for Codex/Claude, form encoding for Antigravity, and refined permanent `refresh_status` error classification).
- **Antigravity 503 capacity resilience (`0b0f782`)**:
  - Honoured Google's `RetryInfo.retryDelay` on `MODEL_CAPACITY_EXHAUSTED` (HTTP 503) instead of falling back to sub-second backoff.
  - Raised `MAX_STREAM_ATTEMPTS` (3 → 4) and `BASE_RETRY_DELAY_MS` (800 → 1500), and treated `"capacity"` / `"temporarily unavailable"` errors as retryable.
  - Preserved 503 errors across account rotation and removed the IDE-only `requestType: "agent"` field from the chat path.
  - Cleaned up `sanitize_ag_function_name` delegation and payload forwarding tests.

### 🔧 Chores (`ragasukma-core`)
- **Dependency & version cleanup (`431d7b1`)**: Dropped unused dependencies, read version dynamically from `Cargo.toml`, and fixed cross-platform test behavior.

## [v0.5.1] - 2026-11-21

### 🚀 MCP Runtime & Agent Harness Reliability
- **MCP stdio session lifecycle** (`ragasukma-core`): MCP subprocesses are now pooled per session/server, reused safely, and terminated cleanly during shutdown or replacement.
- **Concurrent MCP request safety**: stdio requests use serialized session access with per-session request IDs, preventing response mix-ups and stale process replacement races.
- **MCP notifications**: server notifications are retained in a bounded queue and can be drained by the host application.
- **Structured MCP errors**: transport, timeout, malformed JSON, JSON-RPC, process-exit, and database PID update failures are classified with stable error codes.
- **MCP HTTP transport reuse**: HTTP clients are shared and request timeouts are applied consistently.
- **Session MCP state refresh**: discovered tools and live process PIDs are synchronized with session state more reliably.

### 🔒 Agent Safety & Reliability
- Hardened allowed-workspace path validation, including canonical path checks and symlink-aware containment handling.
- Improved MCP server lifecycle cleanup and PID bookkeeping for dead or replaced processes.
- Added an autonomous agent harness improvement roadmap covering recovery, failure classification, command policy, auditability, observability, and concurrency testing.

### 🧪 Tests & Maintenance
- Expanded MCP and tool-related regression coverage.
- Applied repository-wide Rust formatting and normalized missing trailing newlines.

## [v0.5.0] - 2026-11-20

### 🚀 Ecosystem Version Bump
- Unified version bump to `0.5.0` across `ragasukma-core`, `ragasukma`, `ragasukma-tui`, the web dashboard, and the landing page.

### ✨ New Features & Enhancements
- **Single-Source Changelog Service** (`web/src/services/changelog.ts`): the dashboard now parses the root `CHANGELOG.md` at build time, so the version list and release notes can never drift from the file maintainers edit.
- **Changelog-Driven Update Notes** (`MenuTab.tsx`): the update modal renders release notes as Markdown straight from the changelog, with the GitHub API kept only as a fallback.
- **Dynamic App Version** (`MenuTab.tsx`): the displayed version is now derived from `web/package.json` instead of a hardcoded string.
- **Mobile Dashboard Refinement** (`MobileTopBar`, `MobileInteractionDock`, `MobileFloatingMusic`): mobile chrome reworked for a cleaner, less cluttered layout.

### 🐛 Bug Fixes
- **TTS / OpenVoice stability** (`src/audio/openvoice.rs`, `src/audio/tts.rs`): hardened audio synthesis error handling and recovery.
- **Perception utilities** (`src/engine/perception/utils.rs`): fixed edge cases in perception input handling.
- **Skill runner** (`src/engine/chat_loop/skill_runner.rs`): corrected skill directory hint resolution.

### 🔧 Chores
- `web/tsconfig.json`: enabled `resolveJsonModule` for `package.json` imports.

## [v0.4.0] - 2026-11-16

### 🚀 Ecosystem Version Bump
- Unified version bump to `0.4.0` across `ragasukma-core`, `ragasukma`, and `ragasukma-tui`.

### ✨ New Features & Enhancements
- **Inline Image Paste (`[image:N]`)** (`750de60`): Simplified web chat image paste into inline `[image:N]` tokens for cleaner prompt composition and multimodal agent submission.
- **Auto-Detected Skill Directory Hints** (`c86dfcc`): Updated skill directory hints in the web dashboard to reflect auto-detected workspace skill paths.

## [v0.3.2] - 2026-11-15

### 🚀 Ecosystem Version Bump
- Unified version bump to `0.3.2` across `ragasukma-core`, `ragasukma`, and `ragasukma-tui`.
- Local OpenAI-compatible endpoint (`openai_v1.rs`) now reports its version from `CARGO_PKG_VERSION` instead of a hardcoded string, preventing version drift.

### ✨ New Features & Enhancements
- **Richer message rendering** (`message_renderer`): unified diff rendering in tool cards, smart tool-call summaries, and inline Markdown support.
- **Reactive per-session MCP updates**: the MCP server list refreshes live during tool execution without requiring a restart.
- **MCP process hygiene**: registered MCP server processes are cleaned up on TUI startup and shutdown.

### 🐛 Bug Fixes
- **Streaming no longer hijacks manual scroll**: added a `follow_stream` flag so incoming deltas stop forcing the viewport back to the bottom once the user scrolls up mid-stream; auto-follow resumes when returning to the bottom or on a new turn.
- **No more phase-leak into tool-call bubbles**: stream deltas now target only the open assistant phase (one without tool calls), creating a fresh bubble after a tool-call phase instead of appending into a closed one.
- **Incomplete `<think>` blocks** no longer blank the assistant bubble while thinking is still streaming.
- **Duplicate final summaries** are reconciled against already-streamed content, avoiding repeated text.
- Added regression tests covering phase leaking and manual scroll-away during streaming.

## [v0.3.1] - 2026-11-12

### 🚀 Ecosystem Version Bump
- Unified version bump to `0.3.1` across `ragasukma-core`, `ragasukma`, and `ragasukma-tui`.

### 🔧 CI / Release Infrastructure
- **Resilient core update path** (`baeb14e7`): improved `ragasukma-core` git update resilience in the release workflow so a stale or diverged core checkout no longer breaks the build.
- **Release pipeline bump** (`f0137036`): version bump wired through the CI release workflow.

## [v0.3.0] - 2026-11-11

### 🚀 Major Features & Enhancements
- Unified ecosystem version bump across `ragasukma-core`, `ragasukma`, and `ragasukma-tui`.
- Full parity between TUI and GUI interfaces with shared configuration, session history, and local router capabilities.
- Native multiplatform packaging and release CI/CD automation.

## [v0.2.0] - 2026-11-03

### 🚀 Major Features & Enhancements
- **Supertonic 3 & OpenVoice V2 Native Pipeline (Zero-Shot Voice Cloning)**:
  - High-speed neural voice synthesis powered by Supertonic 3 in pure Rust with ONNX Runtime (`ort`).
  - Integrated OpenVoice V2 Tone Color Converter: effortlessly clone any reference audio sample (`assets/refvoice.*`) uploaded directly via the web dashboard.
  - Automatic fallback to native Supertonic voices when reference audio is empty.
  - Slerp-based geometric voice morphing and context-aware prosody calculation (dynamic tempo, punctuation pauses, and sentence density adaptation).
  - Multi-language coverage across 31 world languages (with automatic Hangul Jamo decomposition and full STT synchronization).
  - Automated background model downloader fetching model checkpoints seamlessly from HuggingFace.
- **Three-VRM Spring Bone Physics & 3D Visual Companion**:
  - Transparent desktop overlay window with real-time 3D VRM MToon cel-shaded rendering.
  - Full Verlet Spring Bone physics (gravity, stiffness, drag, and sphere colliders for hair and clothing).
  - Real-time vocal lip-sync and dynamic expressions across 8 canonical emotions (*happy*, *shy*, *excited*, *surprised*, *thinking*, *sad*, *angry*, *neutral*).
- **In-Page Browser DOM Automation & Precision Action Engine**:
  - Live DOM element analysis for hallucination-free website interaction.
  - Voice-driven web navigation: tab management, searching, form filling, text clicking, and button pressing.
- **Retina HiDPI Picture-in-Picture (PiP) Music Player**:
  - High-quality local and YouTube audio streaming and background caching.
  - 2x HiDPI floating mini-player (680×210 px) with persistent dedicated Web Worker timing for stutter-free playback.
- **Desktop Automation & Operating System Controls**:
  - Multiplatform precision desktop clicking with native accessibility tree integration and OCR fallback.
  - Voice-controlled application launcher, volume management, screenshots, and lock screen commands.
  - Integrated to-do list manager with voice task tracking.
- **Antigravity & OpenAI Codex Multi-Provider Pool & Local Router (`:42128/v1`)**:
  - Multi-account OAuth pooling for both Google Antigravity (`ag/`) and OpenAI Codex (`cx/`) with automatic quota failover.
  - Native 1-click browser OAuth login flow eliminating error-prone manual token inputs.
  - Standard OpenAI-compatible local endpoints (`/v1/chat/completions` & `/v1/models`) for drop-in use with Cursor, Zed, VS Code, and Cline.
  - Reactive in-memory synchronization: account additions, deletions, toggles, and token refreshes take effect instantly without restarting.
  - Clean `model_id`-centric quota inspection and model dropdowns, removing duplicate or confusing display labels.
- **Autonomous Coding & Workspace Agent Engine (`src/agent`)**:
  - Resilient self-healing code editor (`workspace_edit_file`) with Git-style 3-way LCS hunk alignment, indentation tolerance, and anchor matching.
  - Strict sequential tool execution enforcement (one tool per turn) preventing parallel tool call failures and empty argument errors.
  - KV-cache friendly context compressor: automatic symbol outline generation (>250 lines) and dynamic historical tool call compaction.
  - Evidence cache and goal verification loop: tracks file mtime/hash for targeted discovery and runs automated test verifications (`cargo test`, `npm test`) prior to goal conclusion.
  - Full Model Context Protocol (MCP) support for external stdio, SSE, and HTTP tools.
- **Memory & Resource Efficiency**:
  - Disabled memory pattern cache on ONNX models and enabled safe OS working-set trimming (`malloc_trim`).
  - Thread safety and resource-friendly CPU budgeting preventing OOM issues alongside heavy apps like OBS.

---

## [v0.1.4] - 2026-09-16

### ✨ New Features & Enhancements
- **Native yt-dlp Music Engine & Offline Playback**:
  - Automated high-quality audio downloads (m4a/aac) via integrated `yt-dlp`.
  - Smart caching and local collection detection in `assets/music/`.
- **Embedded Web Music Player & Floating Bottom Bar**:
  - Dedicated Music Player tab with track listings, artwork, and full playback controls.
- **Strict Single-Tab Session Takeover**:
  - Instant session takeover mechanism when a new browser tab opens.

---

## [v0.1.3] - 2026-09-16

### ✨ New Features & Enhancements
- **Modular Architecture Overhaul**: Clean segregation of server, assets, and engine modules.
- **Precision 3D Camera Controls**: Orbit rotation, panning, zoom, and middle-click reset.
- **Universal Browser Companion**: 1-Click web companion integration for YouTube voice control.

---

## [v0.1.2] - 2026-09-14

### ✨ New Features & Enhancements
- Auto-launch default browser dashboard upon application startup.
- Isolated default dashboard port `42128`.
- Integrated update checker querying GitHub Releases API.
- Dynamic facial emotion expressions synchronized with LLM responses.

---

## [v0.1.1] - 2026-09-10

### ✨ Initial Features
- Native WGPU 3D VRM Overlay Engine.
- Continuous Microphone VAD with DSP noise reduction.
- Integrated STT and streaming LLM client.
- Skill command registry system.
