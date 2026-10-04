# Language Audit: `llm` Module

- **Directory**: `src/llm`
- **Total Files**: 16
- **Total Comments**: 103
- **Total String Literals**: 1147

## Table of Contents
- [llm/cleaner.rs](#llmcleanerrs) (22 comments, 138 strings)
- [llm/client.rs](#llmclientrs) (10 comments, 52 strings)
- [llm/history.rs](#llmhistoryrs) (0 comments, 11 strings)
- [llm/mod.rs](#llmmodrs) (1 comments, 0 strings)
- [llm/prompt.rs](#llmpromptrs) (17 comments, 261 strings)
- [llm/providers/antigravity/constants.rs](#llmprovidersantigravityconstantsrs) (1 comments, 17 strings)
- [llm/providers/antigravity/mod.rs](#llmprovidersantigravitymodrs) (0 comments, 0 strings)
- [llm/providers/antigravity/oauth.rs](#llmprovidersantigravityoauthrs) (1 comments, 55 strings)
- [llm/providers/antigravity/provider.rs](#llmprovidersantigravityproviderrs) (0 comments, 95 strings)
- [llm/providers/antigravity/quota.rs](#llmprovidersantigravityquotars) (4 comments, 67 strings)
- [llm/providers/antigravity/types.rs](#llmprovidersantigravitytypesrs) (0 comments, 19 strings)
- [llm/providers/antigravity/v1_adapter.rs](#llmprovidersantigravityv1-adapterrs) (29 comments, 310 strings)
- [llm/providers/antigravity/v1_adapter_tests.rs](#llmprovidersantigravityv1-adapter-testsrs) (15 comments, 107 strings)
- [llm/providers/mod.rs](#llmprovidersmodrs) (0 comments, 0 strings)
- [llm/types.rs](#llmtypesrs) (1 comments, 2 strings)
- [llm/v1_engine.rs](#llmv1-enginers) (2 comments, 13 strings)

---

## `llm/cleaner.rs`

- **Path**: `src/llm/cleaner.rs`
- **Comments**: 22
- **Strings**: 138

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 7 | `//` | Matches Claude-style and tool-call XML blocks the model may emit instead of [CMD:...]: |
| 8 | `//` | <dots_function_call>...</dots_function_call> |
| 9 | `//` | <tool_call>...</tool_call> |
| 10 | `//` | <invoke ...>...</invoke> |
| 28 | `///` | Translates tool-call XML blocks (e.g. from Dots Studio, Claude, or tool-fine-tuned LLMs) |
| 29 | `///` | into canonical Ragasukma \`[CMD:...]\` action tags so they execute seamlessly. |
| 39 | `//` | Extract parameters into a map |
| 62 | `//` | 1. Direct canonical tag: if name is already formatted like "desktop:click_text" or "cmd:desktop:click_text" |
| 65 | `//` | Get primary text parameter if available |
| 77 | `//` | 2. Dots / generic tool mapping |
| 80 | `//` | E.g. <invoke name="command"><parameter name="text">Providers</parameter></invoke> |
| 81 | `//` | or <parameter name="text">[CMD:desktop:click_text\|Providers]</parameter> |
| 87 | `//` | If it's a bare word/target without namespace, default to desktop text click |
| 151 | `//` | Check if name has namespace:action format directly |
| 175 | `///` | Clean AI text: strip action tags ([CMD:...], [SKILL:...]), XML tool-call blocks, |
| 176 | `///` | bold markdown asterisks (**bold** -> bold), single-asterisk emotion tags, and backticks. |
| 178 | `//` | 1. Strip any XML tool-call blocks Claude may have emitted |
| 181 | `//` | 2. Strip [CMD:...] / [SKILL:...] tags |
| 184 | `//` | 3. Normalize markdown bold: \`**text**\` -> \`text\` |
| 187 | `//` | 4. Buang tag bintang tunggal emotion (*malu-malu*, *happy*, dsb.) |
| 190 | `//` | 5. Buang backtick (\`\` \` \`\`) and header markdown (\`#\`) |
| 199 | `//` | 7. Rapikan multiple spaces |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 5 | `\[(?:SKILL\|CMD):[^\]]+\]` |
| 5 | `valid regex` |
| 13 | `(?s)<(?:dots_function_call\|tool_call)[^>]*>.*?</(?:dots_function_call\|tool_call)>\|<invoke[^>]*>.*?</invoke>` |
| 15 | `valid xml tool call regex` |
| 19 | `(?s)<invoke\s+name=["']([^"']+)["'][^>]*>(.*?)</invoke>` |
| 20 | `valid invoke block regex` |
| 24 | `(?s)<parameter\s+name=["']([^"']+)["'][^>]*>(.*?)</parameter>` |
| 25 | `valid param block regex` |
| 49 | ` {cmd} ` |
| 63 | `cmd:` |
| 67 | `text` |
| 68 | `query` |
| 69 | `target` |
| 70 | `value` |
| 71 | `url` |
| 72 | `label` |
| 73 | `title` |
| 74 | `name` |
| 79 | `command` |
| 79 | `cmd` |
| 84 | `[CMD:` |
| 84 | `[SKILL:` |
| 88 | `[CMD:desktop:click_text\|{trimmed}]` |
| 92 | `desktop_scan_visual` |
| 92 | `desktop:scan_visual` |
| 92 | `scan_visual` |
| 93 | `[CMD:desktop:scan_visual]` |
| 95 | `desktop_click_text` |
| 95 | `desktop:click_text` |
| 95 | `click_text` |
| 96 | `[CMD:desktop:click_text\|{val}]` |
| 98 | `desktop_click_button` |
| 98 | `desktop:click_button` |
| 98 | `click_button` |
| 99 | `[CMD:desktop:click_button\|{val}]` |
| 101 | `desktop_click_at` |
| 101 | `desktop:click_at` |
| 101 | `click_at` |
| 102 | `x` |
| 102 | `0` |
| 103 | `y` |
| 103 | `0` |
| 104 | `[CMD:desktop:click_at\|{x}\|{y}]` |
| 106 | `desktop_type` |
| 106 | `desktop:type` |
| 107 | `[CMD:desktop:type\|{val}]` |
| 109 | `desktop_key` |
| 109 | `desktop:key` |
| 110 | `[CMD:desktop:key\|{val}]` |
| 112 | `desktop_focus` |
| 112 | `desktop:focus` |
| 113 | `[CMD:desktop:focus\|{val}]` |
| 115 | `desktop_minimize` |
| 115 | `desktop:minimize` |
| 115 | `[CMD:desktop:minimize]` |
| 116 | `desktop_maximize` |
| 116 | `desktop:maximize` |
| 116 | `[CMD:desktop:maximize]` |
| 117 | `desktop_close` |
| 117 | `desktop:close` |
| 117 | `[CMD:desktop:close]` |
| 118 | `browser_open` |
| 118 | `browser:open` |
| 119 | `[CMD:browser:open\|{val}]` |
| 121 | `browser_search` |
| 121 | `browser:search` |
| 122 | `[CMD:browser:search\|{val}]` |
| 124 | `browser_switch` |
| 124 | `browser:switch` |
| 125 | `[CMD:browser:switch\|{val}]` |
| 127 | `browser_close` |
| 127 | `browser:close` |
| 128 | `[CMD:browser:close\|{val}]` |
| 130 | `browser_scroll` |
| 130 | `browser:scroll` |
| 131 | `400` |
| 132 | `[CMD:browser:scroll\|{dist}]` |
| 134 | `browser_type` |
| 134 | `browser:type` |
| 135 | `[CMD:browser:type\|{val}]` |
| 137 | `browser_enter` |
| 137 | `browser:enter` |
| 137 | `[CMD:browser:enter]` |
| 138 | `app_open` |
| 138 | `app:open` |
| 138 | `[CMD:app:open\|{val}]` |
| 139 | `media_play` |
| 139 | `media:play` |
| 139 | `[CMD:media:play\|{val}]` |
| 140 | `media_pause` |
| 140 | `media:pause` |
| 140 | `[CMD:media:pause]` |
| 141 | `media_resume` |
| 141 | `media:resume` |
| 141 | `[CMD:media:resume]` |
| 142 | `media_stop` |
| 142 | `media:stop` |
| 142 | `[CMD:media:stop]` |
| 143 | `os_volume_up` |
| 143 | `os:volume_up` |
| 143 | `[CMD:os:volume_up]` |
| 144 | `os_volume_down` |
| 144 | `os:volume_down` |
| 144 | `[CMD:os:volume_down]` |
| 145 | `os_mute` |
| 145 | `os:mute` |
| 145 | `[CMD:os:mute]` |
| 146 | `os_screenshot` |
| 146 | `os:screenshot` |
| 146 | `[CMD:os:screenshot]` |
| 147 | `os_lock` |
| 147 | `os:lock` |
| 147 | `[CMD:os:lock]` |
| 148 | `todo_add` |
| 148 | `todo:add` |
| 148 | `[CMD:todo:add\|{val}]` |
| 149 | `todo_done` |
| 149 | `todo:done` |
| 149 | `[CMD:todo:done\|{val}]` |
| 154 | `[CMD:{ns}:{act}\|{param}]` |
| 156 | `[CMD:{ns}:{act}]` |
| 159 | `[CMD:{clean_name}\|{param}]` |
| 161 | `[SKILL:{clean_name}]` |
| 168 | `\*\*([^\*\n]+?)\*\*` |
| 168 | `valid bold regex` |
| 172 | `\*[^\*\n]{1,60}\*` |
| 172 | `valid single star emote regex` |
| 185 | `$1` |
| 223 | `<dots_function_call>\n<invoke name="command">\n<parameter name="text">Providers</parameter>\n</invoke>\n</dots_functi...` |
| 225 | `[CMD:desktop:click_text\|Providers]` |
| 230 | `<dots_function_call>\n<invoke name="desktop_scan_visual">\n</invoke>\n</dots_function_call>` |
| 232 | `[CMD:desktop:scan_visual]` |
| 237 | `*happy* Oke! <tool_call><invoke name="browser:open"><parameter name="url">youtube.com</parameter></invoke></tool_call>` |
| 239 | `[CMD:browser:open\|youtube.com]` |
| 244 | `*malu-malu* Halo Kak! Ini **sangat penting** lho! \`kode_rahasia\`` |
| 246 | `Halo Kak! Ini sangat penting lho! kode_rahasia` |
| 251 | `*happy* Siap! [CMD:desktop:click_text\|Save] *tersenyum manis*` |
| 253 | `Siap!` |

---

## `llm/client.rs`

- **Path**: `src/llm/client.rs`
- **Comments**: 10
- **Strings**: 52

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 62 | `//` | Hemat token in RAM/API: save system prompt + maksimal 6 conversation terakhir |
| 79 | `//` | Dynamic Prompt Indexing: suntikkan only Ground of Truth which relevan with ucapan user! |
| 92 | `//` | 1. Local Mode: Using internal Ragasukma engine (currently Antigravity pool, extensible) |
| 137 | `//` | 2. Mode Custom: AI Assistant kustom (OpenAI-compatible, Ollama, LM Studio, vLLM, Groq, dll) |
| 138 | `//` | Custom endpoint strictly obeys base_url configuration and user-selected model |
| 279 | `//` | 3. Fallback if custom endpoint Ollama native /api/chat used |
| 340 | `///` | Complete a prompt or message sequence silently without triggering audio playback or modifying chat history. |
| 350 | `//` | Low temperature for deterministic planning |
| 358 | `//` | Custom OpenAI-compatible endpoint |
| 395 | `//` | Fallback to Ollama native format |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 37 | `system` |
| 56 | `system` |
| 82 | `system` |
| 88 | `user` |
| 93 | `local` |
| 94 | `antigravity` |
| 108 | `assistant` |
| 126 | `[LLM Client] Local Engine error: {e}. Attempting fallback...` |
| 132 | `[LLM Client] Provider set to 'local', but no local engine account is connected to the pool.` |
| 139 | `/chat/completions` |
| 142 | `{}/chat/completions` |
| 157 | `Content-Type` |
| 157 | `application/json` |
| 161 | `Authorization` |
| 161 | `Bearer {key}` |
| 172 | `neutral` |
| 177 | `data: ` |
| 178 | `[DONE]` |
| 182 | `/choices/0/delta/reasoning_content` |
| 187 | `/choices/0/delta/content` |
| 188 | `/choices/0/message/content` |
| 195 | `[SKILL:` |
| 196 | `[CMD:` |
| 205 | `[SKILL:` |
| 206 | `[CMD:` |
| 247 | `assistant` |
| 265 | `[LLM Client] Custom endpoint status {}: {}` |
| 273 | `[LLM Client] Failed streaming request to {}: {e}` |
| 287 | `/api/chat` |
| 290 | `{}/api/chat` |
| 305 | `Failed to connect to Ollama ({ollama_endpoint}): {e}` |
| 309 | `Ollama error: {err_text}` |
| 312 | `Failed to parse Ollama response` |
| 323 | `assistant` |
| 342 | `local` |
| 343 | `antigravity` |
| 359 | `/chat/completions` |
| 362 | `{}/chat/completions` |
| 371 | `type` |
| 371 | `json_object` |
| 377 | `Content-Type` |
| 377 | `application/json` |
| 381 | `Authorization` |
| 381 | `Bearer {key}` |
| 388 | `choices` |
| 388 | `message` |
| 388 | `content` |
| 396 | `/api/chat` |
| 399 | `{}/api/chat` |
| 406 | `json` |
| 414 | `Failed to connect to Ollama ({ollama_endpoint}): {e}` |
| 416 | `Failed to parse Ollama response` |

---

## `llm/history.rs`

- **Path**: `src/llm/history.rs`
- **Comments**: 0
- **Strings**: 11

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 17 | `system` |
| 32 | `{:04}-{:02}-{:02}` |
| 33 | `{:02}:{:02}:{:02}` |
| 63 | `{:04}-{:02}-{:02}` |
| 64 | `{:02}:{:02}:{:02}` |
| 76 | `histories` |
| 78 | `[History] Failed to create histories directory: {e}` |
| 82 | `{date_str}.md` |
| 93 | `# Conversation History — {date_str}\n> Assistant: {assistant_name} ({role}) \| User: {callsign}\n\n---\n\n` |
| 99 | `### [{time_str}] {callsign}\n{user_msg}\n\n### [{time_str}] {assistant_name}\n{assistant_msg}\n\n---\n\n` |
| 105 | `[History] Failed to write chat history to {:?}: {e}` |

---

## `llm/mod.rs`

- **Path**: `src/llm/mod.rs`
- **Comments**: 1
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 9 | `//` | Re-export komponen utama |


### String Literals (`"..."`)

_No string literals found._

---

## `llm/prompt.rs`

- **Path**: `src/llm/prompt.rs`
- **Comments**: 17
- **Strings**: 261

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 3 | `///` | Resolves any BCP-47 / ISO-639 language tag to a (canonical_code, human_readable_name) pair. |
| 4 | `///` | Source of truth for supported languages: SUPPORTED_LANGUAGES in AudioTtsSection.tsx. |
| 5 | `///` | Kept in sync with clean_lang_code() in src/audio/supertonic.rs. |
| 69 | `//` | User custom skills from assets/skill-command.md |
| 91 | `//` | Load registered automation recipes from assets/recipes/*.json |
| 201 | `//` | 1. List of all currently open tabs |
| 207 | `//` | 2. Snapshot of active web page DOM with Ground of Truth Context |
| 264 | `//` | Buttons on Web Page |
| 300 | `//` | Text Content / Selected Elements |
| 362 | `//` | Desktop Buttons / Controls |
| 393 | `//` | Text Content on Screen |
| 425 | `//` | Living State: Provide holistic computer status context to the LLM |
| 433 | `//` | Persona & Character Dynamics |
| 521 | `//` | BCP-47 tags (Chrome STT format) |
| 530 | `//` | Bare ISO-639-1 codes |
| 563 | `//` | Ambiguous / unknown → fallback to Indonesian |
| 565 | `//` | not a code → fallback |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 10 | `id` |
| 10 | `id` |
| 10 | `Indonesian (Bahasa Indonesia)` |
| 11 | `en` |
| 11 | `en` |
| 11 | `English` |
| 12 | `ja` |
| 12 | `ja` |
| 12 | `Japanese (日本語)` |
| 13 | `ko` |
| 13 | `ko` |
| 13 | `Korean (한국어)` |
| 14 | `ar` |
| 14 | `ar` |
| 14 | `Arabic (العربية)` |
| 15 | `bg` |
| 15 | `bg` |
| 15 | `Bulgarian (Български)` |
| 16 | `cs` |
| 16 | `cs` |
| 16 | `Czech (Čeština)` |
| 17 | `da` |
| 17 | `da` |
| 17 | `Danish (Dansk)` |
| 18 | `de` |
| 18 | `de` |
| 18 | `German (Deutsch)` |
| 19 | `el` |
| 19 | `el` |
| 19 | `Greek (Ελληνικά)` |
| 20 | `es` |
| 20 | `es` |
| 20 | `Spanish (Español)` |
| 21 | `et` |
| 21 | `et` |
| 21 | `Estonian (Eesti)` |
| 22 | `fi` |
| 22 | `fi` |
| 22 | `Finnish (Suomi)` |
| 23 | `fr` |
| 23 | `fr` |
| 23 | `French (Français)` |
| 24 | `hi` |
| 24 | `hi` |
| 24 | `Hindi (हिन्दी)` |
| 25 | `hr` |
| 25 | `hr` |
| 25 | `Croatian (Hrvatski)` |
| 26 | `hu` |
| 26 | `hu` |
| 26 | `Hungarian (Magyar)` |
| 27 | `it` |
| 27 | `it` |
| 27 | `Italian (Italiano)` |
| 28 | `lt` |
| 28 | `lt` |
| 28 | `Lithuanian (Lietuvių)` |
| 29 | `lv` |
| 29 | `lv` |
| 29 | `Latvian (Latviešu)` |
| 30 | `nl` |
| 30 | `nl` |
| 30 | `Dutch (Nederlands)` |
| 31 | `pl` |
| 31 | `pl` |
| 31 | `Polish (Polski)` |
| 32 | `pt` |
| 32 | `pt` |
| 32 | `Portuguese (Português)` |
| 33 | `ro` |
| 33 | `ro` |
| 33 | `Romanian (Română)` |
| 34 | `ru` |
| 34 | `ru` |
| 34 | `Russian (Русский)` |
| 35 | `sk` |
| 35 | `sk` |
| 35 | `Slovak (Slovenčina)` |
| 36 | `sl` |
| 36 | `sl` |
| 36 | `Slovenian (Slovenščina)` |
| 37 | `sv` |
| 37 | `sv` |
| 37 | `Swedish (Svenska)` |
| 38 | `tr` |
| 38 | `tr` |
| 38 | `Turkish (Türkçe)` |
| 39 | `uk` |
| 39 | `uk` |
| 39 | `Ukrainian (Українська)` |
| 40 | `vi` |
| 40 | `vi` |
| 40 | `Vietnamese (Tiếng Việt)` |
| 41 | `zh` |
| 41 | `zh` |
| 41 | `Chinese (中文)` |
| 42 | `id` |
| 42 | `Indonesian (Bahasa Indonesia)` |
| 55 | `{}. {} ({})\n` |
| 60 | `\nLOCAL MUSIC COLLECTION (Total {} tracks):\n{}And more. Play any track using \`[CMD:media:play\|title]\` (automatically...` |
| 76 | `\| \`[SKILL:{}]\` \| As defined in script \| {} \|` |
| 83 | `*(No additional custom skills registered)*` |
| 86 | `\| Custom Skill Tag \| Parameter \| Description \|\n\| :--- \| :--- \| :--- \|\n{}` |
| 87 | `\n` |
| 96 | `[CMD:recipe:run\|{}]` |
| 101 | `value_{p}` |
| 104 | `[CMD:recipe:run\|{}\|{}]` |
| 107 | `\| \`{}\` \| \`{}\` \| {} \|` |
| 113 | `*(No recipes registered in assets/recipes/)*` |
| 116 | `\| Recipe Name \| Action Tag (Run Directly) \| Description \|\n\| :--- \| :--- \| :--- \|\n{}` |
| 117 | `\n` |
| 122 | `\nACTION TAG FORMAT (MANDATORY — NEVER DEVIATE): - Use ONLY the bracket format: [CMD:category:action\|param] or [SKILL...` |
| 188 | `todo.md` |
| 192 | `\nUSER TO-DO / NOTES:\n{}\n` |
| 210 | `\n###  ACTIVE BROWSER PAGE CONTEXT (GROUND OF TRUTH):\n\             * **Page Title:** {}\n\             * **URL:** {...` |
| 223 | `{}...` |
| 228 | `* **Page Content / Summary (Ground of Truth):**\n{}\n\                 (Use this content whenever answering questions...` |
| 248 | `\`{}\` (placeholder: "{}", value: "{}")` |
| 252 | `\`{}\` (placeholder: "{}")` |
| 254 | `\`{}\`` |
| 259 | `* **Form / Input Fields:** {}\n` |
| 260 | `, ` |
| 291 | `"{}"` |
| 294 | `* **Buttons on Web Page:** {}\n` |
| 295 | `, ` |
| 319 | `* **Text Content / Selected Elements:**\n  - {}\n` |
| 320 | `\n  - ` |
| 327 | `\n` |
| 337 | `\n###  ACTIVE DESKTOP WINDOW CONTEXT (GROUND OF TRUTH):\n\             * **Currently Active Window:** {} ({})\n\     ...` |
| 345 | `Program Manager` |
| 348 | `"{}" ({}) [ACTIVE]` |
| 350 | `"{}" ({})` |
| 356 | `* **List of Open Windows in OS:** {}\n` |
| 357 | `, ` |
| 370 | `#{}: "{}" [center: {},{}]` |
| 372 | `#{}: "{}"` |
| 378 | `* **Desktop Buttons / Controls:** {}\n` |
| 379 | `, ` |
| 388 | `"{}"` |
| 390 | `* **Input Fields:** {}\n` |
| 390 | `, ` |
| 398 | `* **Text Content on Screen:** {}\n` |
| 399 | ` • ` |
| 422 | `male` |
| 436 | `partner` |
| 437 | `You are an anime husbando / loving male partner who is caring, protective, warm, mature, and deeply devoted to {calls...` |
| 439 | `assistant` |
| 440 | `You are a professional male personal assistant / butler who is intelligent, courteous, efficient, and always ready to...` |
| 442 | `friend` |
| 443 | `You are a close male anime buddy / best friend (bro/bestie) who is fun, casual, supportive, and energetic with {calls...` |
| 445 | `tsundere` |
| 446 | `You are a male anime Tsundere character for {callsign}. Outwardly aloof, feigning indifference, and easily flustered,...` |
| 448 | `maid` |
| 449 | `You are a loyal male butler / servant who respectfully and devotedly attends to all needs of {callsign} with utmost c...` |
| 452 | `You are a smart, cool, and friendly male anime AI voice assistant for {callsign}.` |
| 457 | `partner` |
| 458 | `You are an anime waifu / loving female partner who is sweet, caring, playful, and deeply devoted to {callsign}. Speak...` |
| 460 | `assistant` |
| 461 | `You are a professional female personal assistant who is intelligent, courteous, efficient, and always ready to serve ...` |
| 463 | `friend` |
| 464 | `You are a close female anime buddy / best friend (bestie) who is fun, casual, lively, and warm with {callsign}.` |
| 466 | `tsundere` |
| 467 | `You are a female anime Tsundere character for {callsign}. Outwardly feisty, aloof, and easily flustered, but inwardly...` |
| 469 | `maid` |
| 470 | `You are a cute, polite, and loyal anime maid who cheerfully and devotedly serves {callsign} with great warmth.` |
| 473 | `You are a smart, sweet, and friendly female anime AI voice assistant for {callsign}.` |
| 478 | `male` |
| 478 | `female` |
| 481 | `\n[SYSTEM STATUS INFO: {}]\n` |
| 487 | `Your name is {assistant_name}. You are an AI with {gender_tag} gender. {persona_desc}{task_context}\n\ {music_prompt}...` |
| 522 | `id-ID` |
| 522 | `id` |
| 523 | `en-US` |
| 523 | `en` |
| 524 | `ja-JP` |
| 524 | `ja` |
| 525 | `ko-KR` |
| 525 | `ko` |
| 526 | `ar-SA` |
| 526 | `ar` |
| 527 | `zh-TW` |
| 527 | `zh` |
| 528 | `pt-PT` |
| 528 | `pt` |
| 529 | `pt-BR` |
| 529 | `pt` |
| 531 | `id` |
| 531 | `id` |
| 532 | `en` |
| 532 | `en` |
| 533 | `ja` |
| 533 | `ja` |
| 534 | `ko` |
| 534 | `ko` |
| 535 | `ar` |
| 535 | `ar` |
| 536 | `bg` |
| 536 | `bg` |
| 537 | `cs` |
| 537 | `cs` |
| 538 | `da` |
| 538 | `da` |
| 539 | `de` |
| 539 | `de` |
| 540 | `el` |
| 540 | `el` |
| 541 | `es` |
| 541 | `es` |
| 542 | `et` |
| 542 | `et` |
| 543 | `fi` |
| 543 | `fi` |
| 544 | `fr` |
| 544 | `fr` |
| 545 | `hi` |
| 545 | `hi` |
| 546 | `hr` |
| 546 | `hr` |
| 547 | `hu` |
| 547 | `hu` |
| 548 | `it` |
| 548 | `it` |
| 549 | `lt` |
| 549 | `lt` |
| 550 | `lv` |
| 550 | `lv` |
| 551 | `nl` |
| 551 | `nl` |
| 552 | `pl` |
| 552 | `pl` |
| 553 | `pt` |
| 553 | `pt` |
| 554 | `ro` |
| 554 | `ro` |
| 555 | `ru` |
| 555 | `ru` |
| 556 | `sk` |
| 556 | `sk` |
| 557 | `sl` |
| 557 | `sl` |
| 558 | `sv` |
| 558 | `sv` |
| 559 | `tr` |
| 559 | `tr` |
| 560 | `uk` |
| 560 | `uk` |
| 561 | `vi` |
| 561 | `vi` |
| 562 | `zh` |
| 562 | `zh` |
| 564 | `id` |
| 565 | `hindi` |
| 565 | `id` |
| 566 | `xx` |
| 566 | `id` |
| 572 | `TEXT-TO-SPEECH (TTS) SPOKEN FORMAT RULES:` |
| 573 | `ACTION TAG FORMAT` |
| 574 | `AUTOMATION RECIPES` |
| 575 | `[CMD:recipe:run\|` |
| 576 | `[CMD:recipe:train\|` |
| 577 | `[CMD:desktop:focus\|` |

---

## `llm/providers/antigravity/constants.rs`

- **Path**: `src/llm/providers/antigravity/constants.rs`
- **Comments**: 1
- **Strings**: 17

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 26 | `///` | Default thinking signature fallback for Gemini 3+ / Antigravity thinking models (valid Base64) |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 2 | `1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com` |
| 3 | `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` |
| 4 | `https://accounts.google.com/o/oauth2/v2/auth` |
| 5 | `https://oauth2.googleapis.com/token` |
| 6 | `https://www.googleapis.com/oauth2/v1/userinfo` |
| 7 | `https://daily-cloudcode-pa.googleapis.com` |
| 9 | `https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist` |
| 11 | `https://cloudcode-pa.googleapis.com/v1internal:onboardUser` |
| 13 | `https://daily-cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels` |
| 15 | `https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary` |
| 16 | `antigravity/ide/2.11.0 darwin/arm64` |
| 19 | `https://www.googleapis.com/auth/cloud-platform` |
| 20 | `https://www.googleapis.com/auth/userinfo.email` |
| 21 | `https://www.googleapis.com/auth/userinfo.profile` |
| 22 | `https://www.googleapis.com/auth/cclog` |
| 23 | `https://www.googleapis.com/auth/experimentsandconfigs` |
| 27 | `dGhvdWdodF9zaWduYXR1cmU=` |

---

## `llm/providers/antigravity/mod.rs`

- **Path**: `src/llm/providers/antigravity/mod.rs`
- **Comments**: 0
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

_No string literals found._

---

## `llm/providers/antigravity/oauth.rs`

- **Path**: `src/llm/providers/antigravity/oauth.rs`
- **Comments**: 1
- **Strings**: 55

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 223 | `//` | Fire onboarding call |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 38 | `{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&access_type=offline&prompt=consent` |
| 53 | `grant_type` |
| 53 | `authorization_code` |
| 54 | `client_id` |
| 55 | `client_secret` |
| 56 | `code` |
| 57 | `redirect_uri` |
| 64 | `Failed to contact Google OAuth token endpoint for code exchange` |
| 68 | `OAuth authorization code exchange failed: {err}` |
| 72 | `Failed to parse OAuth token response` |
| 78 | `{}?alt=json` |
| 79 | `Authorization` |
| 79 | `Bearer {access_token}` |
| 81 | `Failed to fetch Google user info` |
| 85 | `User info fetch failed: {err}` |
| 88 | `Failed to parse user info` |
| 101 | `Google OAuth did not return a refresh_token. Please revoke app access or ensure prompt=consent was used.` |
| 110 | `ag-acc` |
| 131 | `client_id` |
| 132 | `client_secret` |
| 133 | `grant_type` |
| 133 | `refresh_token` |
| 134 | `refresh_token` |
| 141 | `Failed to contact Google OAuth token endpoint` |
| 145 | `Google OAuth token refresh failed: {err}` |
| 148 | `Failed to parse token response` |
| 177 | `ideType` |
| 178 | `platform` |
| 179 | `pluginType` |
| 183 | `metadata` |
| 188 | `Authorization` |
| 188 | `Bearer {access_token}` |
| 189 | `Content-Type` |
| 189 | `application/json` |
| 190 | `User-Agent` |
| 191 | `x-request-source` |
| 191 | `local` |
| 199 | `/cloudaicompanionProject/id` |
| 200 | `/cloudaicompanionProject` |
| 203 | `cloudaicompanion-default` |
| 205 | `legacy-tier` |
| 206 | `/allowedTiers` |
| 208 | `isDefault` |
| 212 | `id` |
| 226 | `Authorization` |
| 226 | `Bearer {access_token}` |
| 227 | `Content-Type` |
| 227 | `application/json` |
| 228 | `User-Agent` |
| 229 | `x-request-source` |
| 229 | `local` |
| 231 | `tierId` |
| 232 | `metadata` |
| 241 | `project-{}` |
| 241 | `p` |

---

## `llm/providers/antigravity/provider.rs`

- **Path**: `src/llm/providers/antigravity/provider.rs`
- **Comments**: 0
- **Strings**: 95

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 26 | `antigravity_accounts.json` |
| 31 | `assets` |
| 32 | `antigravity_accounts.json` |
| 52 | `[Antigravity] Initialized provider with {} account(s) loaded.` |
| 84 | `ag-acc-{}` |
| 103 | `accounts` |
| 104 | `tokens` |
| 105 | `connections` |
| 114 | `/credentials/refreshToken` |
| 115 | `/credentials/refresh_token` |
| 116 | `/refreshToken` |
| 117 | `/refresh_token` |
| 121 | `/credentials/email` |
| 122 | `/email` |
| 126 | `/credentials/projectId` |
| 127 | `/projectId` |
| 132 | `ag-acc-{}` |
| 164 | `tmp` |
| 208 | `Refresh token cannot be empty` |
| 228 | `ag-acc-{}` |
| 295 | `Account not found` |
| 306 | `Account not found` |
| 315 | `Account with ID '{id}' not found` |
| 352 | `Antigravity has no configured accounts. Please add an account first.` |
| 371 | `All Antigravity accounts are rate-limited or inactive. Please wait {} seconds.` |
| 390 | `[Antigravity] Failed to refresh token for account {}: {}` |
| 400 | `cloudaicompanion-default` |
| 426 | `429` |
| 427 | `resource_exhausted` |
| 428 | `quota` |
| 435 | `[Antigravity] Account {} hit 429 quota limit. Rotating to next account...` |
| 440 | `401` |
| 440 | `403` |
| 444 | `[Antigravity] Account {} auth error: {}. Retrying next...` |
| 457 | `All Antigravity accounts failed during execution.` |
| 475 | `{}/v1internal:streamGenerateContent?alt=sse` |
| 482 | `assistant` |
| 482 | `model` |
| 483 | `user` |
| 486 | `role` |
| 487 | `parts` |
| 487 | `text` |
| 491 | `ag-sess` |
| 492 | `ag-conv` |
| 493 | `ag-traj` |
| 499 | `agent/{conversation_id}/{now_ms}/{trajectory_id}/{step}` |
| 501 | `ag/` |
| 508 | `project` |
| 509 | `model` |
| 510 | `userAgent` |
| 510 | `antigravity` |
| 511 | `requestType` |
| 511 | `agent` |
| 512 | `requestId` |
| 513 | `request` |
| 514 | `contents` |
| 515 | `generationConfig` |
| 516 | `temperature` |
| 517 | `topP` |
| 518 | `topK` |
| 519 | `maxOutputTokens` |
| 521 | `sessionId` |
| 528 | `Authorization` |
| 528 | `Bearer {access_token}` |
| 529 | `Content-Type` |
| 529 | `application/json` |
| 530 | `User-Agent` |
| 531 | `X-Client-Name` |
| 531 | `antigravity` |
| 532 | `X-Client-Version` |
| 532 | `2.11.0` |
| 535 | `Failed to send request to Antigravity stream endpoint` |
| 540 | `Antigravity HTTP error {status}: {err_text}` |
| 546 | `neutral` |
| 551 | `data: ` |
| 552 | `[DONE]` |
| 557 | `/response/candidates` |
| 558 | `/candidates` |
| 564 | `/content/parts` |
| 568 | `thought` |
| 575 | `text` |
| 579 | `[SKILL:` |
| 588 | `[SKILL:` |
| 642 | `Antigravity has no configured accounts. Please add an account first.` |
| 661 | `All Antigravity accounts are rate-limited or inactive. Please wait {} seconds.` |
| 680 | `[Antigravity] Failed to refresh token for account {}: {}` |
| 690 | `cloudaicompanion-default` |
| 714 | `429` |
| 715 | `resource_exhausted` |
| 716 | `quota` |
| 723 | `[Antigravity] Account {} hit 429 quota limit. Rotating to next account...` |
| 728 | `401` |
| 728 | `403` |
| 732 | `[Antigravity] Account {} auth error: {}. Retrying next...` |
| 745 | `All Antigravity accounts failed during execution.` |

---

## `llm/providers/antigravity/quota.rs`

- **Path**: `src/llm/providers/antigravity/quota.rs`
- **Comments**: 4
- **Strings**: 67

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 31 | `//` | 1. Fetch available models with quota info |
| 97 | `//` | Sort models_quota to put priority models first |
| 128 | `//` | 2. Fetch weekly quota summary |
| 197 | `//` | Tier / Plan name |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 26 | `project` |
| 37 | `Authorization` |
| 37 | `Bearer {access_token}` |
| 38 | `Content-Type` |
| 38 | `application/json` |
| 39 | `User-Agent` |
| 40 | `X-Client-Name` |
| 40 | `antigravity` |
| 41 | `X-Client-Version` |
| 41 | `2.11.0` |
| 50 | `models` |
| 53 | `isInternal` |
| 60 | `quotaInfo` |
| 62 | `remainingFraction` |
| 67 | `resetTime` |
| 71 | `displayName` |
| 89 | `HTTP {status}: {err_txt}` |
| 93 | `Request failed: {e}` |
| 99 | `gemini-3.8-flash-high` |
| 100 | `gemini-3.8-flash-medium` |
| 101 | `gemini-3.8-flash-low` |
| 102 | `gemini-3.7-flash-high` |
| 103 | `gemini-3.7-flash-medium` |
| 104 | `gemini-3.7-flash-low` |
| 105 | `gemini-3.6-flash-high` |
| 106 | `gemini-3.6-flash-medium` |
| 107 | `gemini-3.6-flash-low` |
| 108 | `gemini-3.5-flash-high` |
| 109 | `gemini-3.5-flash-low` |
| 110 | `gemini-pro-agent` |
| 111 | `claude-sonnet-4-6` |
| 112 | `claude-opus-4-6-thinking` |
| 113 | `gpt-oss-120b-medium` |
| 114 | `gemini-3.1-flash-image` |
| 132 | `Authorization` |
| 132 | `Bearer {access_token}` |
| 133 | `Content-Type` |
| 133 | `application/json` |
| 134 | `User-Agent` |
| 135 | `X-Client-Name` |
| 135 | `antigravity` |
| 136 | `X-Client-Version` |
| 136 | `2.11.0` |
| 144 | `groups` |
| 145 | `/quotaSummary/groups` |
| 150 | `displayName` |
| 151 | `buckets` |
| 154 | `bucketId` |
| 156 | `displayName` |
| 157 | `{b_id} {b_name}` |
| 158 | `weekly` |
| 161 | `disabled` |
| 165 | `remainingFraction` |
| 167 | `gemini` |
| 168 | `Gemini (Weekly)` |
| 169 | `claude` |
| 170 | `gpt` |
| 172 | `Claude & GPT (Weekly)` |
| 174 | `{g_name} (Weekly)` |
| 177 | `resetTime` |
| 199 | `g1-pro-tier` |
| 199 | `Google AI Pro` |
| 200 | `free-tier` |
| 200 | `starter` |
| 200 | `Starter (Free)` |
| 205 | `error` |
| 207 | `success` |

---

## `llm/providers/antigravity/types.rs`

- **Path**: `src/llm/providers/antigravity/types.rs`
- **Comments**: 0
- **Strings**: 19

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 20 | `{}-{:x}-{:x}` |
| 25 | `default_account_id` |
| 29 | `refreshToken` |
| 29 | `token` |
| 31 | `accessToken` |
| 33 | `expiresAt` |
| 35 | `projectId` |
| 37 | `tierId` |
| 39 | `rateLimitedUntil` |
| 41 | `consecutiveErrors` |
| 43 | `default_true` |
| 43 | `isActive` |
| 43 | `enabled` |
| 48 | `ag-acc` |
| 79 | `Option::is_none` |
| 87 | `Option::is_none` |
| 89 | `Option::is_none` |
| 91 | `Option::is_none` |
| 95 | `Option::is_none` |

---

## `llm/providers/antigravity/v1_adapter.rs`

- **Path**: `src/llm/providers/antigravity/v1_adapter.rs`
- **Comments**: 29
- **Strings**: 310

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 16 | `///` | Validate if a string is a valid standard or URL-safe Base64 encoded string |
| 56 | `///` | Sanitize and validate thought signature, returning a strictly valid padded base64 string |
| 77 | `///` | Store a Gemini 3+ thoughtSignature for a tool call ID with a 2-hour TTL |
| 103 | `///` | Retrieve a cached thoughtSignature by tool call ID |
| 119 | `///` | Resolve thoughtSignature from embedded ID (e.g. \`call_123__thought__<sig>\`) or cache |
| 191 | `//` | 1. Convert const to enum |
| 198 | `//` | 2. Convert enum values to strings & ensure type is string |
| 214 | `//` | 3. Flatten allOf |
| 260 | `//` | 4. Flatten anyOf / oneOf |
| 309 | `//` | 5. Flatten type array: e.g. ["string", "null"] -> "string" + nullable: true |
| 331 | `//` | Normalize type string aliases if any (e.g. "int" -> "integer", "bool" -> "boolean") |
| 345 | `//` | 6. Infer missing type=object when properties exist |
| 350 | `//` | 7. Ensure array has items |
| 365 | `//` | 8. Remove unsupported JSON Schema keywords for Gemini Protobuf |
| 411 | `//` | 9. Recurse into properties |
| 418 | `//` | 10. Recurse into items |
| 423 | `//` | 11. Cleanup required fields: only keep fields that exist in properties |
| 463 | `//` | Gemini strictly requires the root parameters schema to have type: "object" |
| 466 | `//` | If properties is missing or empty, add placeholder |
| 490 | `//` | Rewrites competing client branding to avoid 429 quota exhaustion flag from Google |
| 507 | `//` | First pass: index tool_calls IDs to function names |
| 629 | `//` | "user" or other |
| 640 | `//` | Turn Normalization: Merge adjacent same-role messages |
| 670 | `//` | Ensure all model turns adhere to Gemini 3+ thoughtSignature rules (ported from 9router) |
| 675 | `//` | Strip thought-only parts, keep thoughtSignature on functionCall parts |
| 736 | `//` | Ensure the first turn is user |
| 752 | `//` | Tools conversion |
| 772 | `//` | Claude style tool definition |
| 935 | `//` | Keep client connection active during thinking phase |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 24 | `==` |
| 37 | `==` |
| 39 | `=` |
| 64 | `==` |
| 125 | `__thought__` |
| 137 | `ag/` |
| 141 | `gemini-3.8-flash-high` |
| 142 | `gemini-3.8-flash-medium` |
| 143 | `gemini-3.8-flash-low` |
| 144 | `gemini-3.8-pro-high` |
| 145 | `gemini-3.7-flash-high` |
| 146 | `gemini-3.7-flash-medium` |
| 147 | `gemini-3.7-flash-low` |
| 148 | `gemini-3.6-flash-high` |
| 149 | `gemini-3.6-flash-medium` |
| 150 | `gemini-3.6-flash-low` |
| 151 | `gemini-3.5-flash-high` |
| 152 | `gemini-3.5-flash-low` |
| 153 | `gemini-2.5-pro` |
| 154 | `gemini-2.5-flash` |
| 155 | `gemini-pro-agent` |
| 156 | `claude-sonnet-4-6` |
| 157 | `claude-opus-4-6-thinking` |
| 158 | `gpt-oss-120b-medium` |
| 159 | `gemini-3.1-flash-image` |
| 167 | `opus` |
| 167 | `3-7` |
| 167 | `3.7` |
| 168 | `claude-opus-4-6-thinking` |
| 169 | `sonnet` |
| 170 | `claude` |
| 171 | `3-5` |
| 172 | `3.5` |
| 174 | `claude-sonnet-4-6` |
| 175 | `gpt-4` |
| 175 | `gpt-o` |
| 176 | `gemini-3.8-flash-high` |
| 177 | `pro` |
| 178 | `gemini-2.5-pro` |
| 179 | `flash` |
| 180 | `gemini-3.8-flash-high` |
| 182 | `claude-sonnet-4-6` |
| 192 | `const` |
| 193 | `enum` |
| 194 | `enum` |
| 199 | `enum` |
| 209 | `type` |
| 210 | `type` |
| 210 | `string` |
| 215 | `allOf` |
| 221 | `properties` |
| 226 | `required` |
| 237 | `properties` |
| 247 | `required` |
| 261 | `anyOf` |
| 261 | `oneOf` |
| 269 | `type` |
| 269 | `null` |
| 270 | `null` |
| 276 | `type` |
| 277 | `object` |
| 277 | `properties` |
| 279 | `array` |
| 279 | `items` |
| 294 | `nullable` |
| 310 | `type` |
| 316 | `null` |
| 324 | `nullable` |
| 326 | `string` |
| 327 | `type` |
| 332 | `type` |
| 335 | `int` |
| 335 | `integer` |
| 336 | `bool` |
| 336 | `boolean` |
| 337 | `float` |
| 337 | `double` |
| 337 | `number` |
| 341 | `type` |
| 346 | `properties` |
| 346 | `type` |
| 347 | `type` |
| 347 | `object` |
| 351 | `type` |
| 351 | `array` |
| 352 | `items` |
| 355 | `items` |
| 357 | `items` |
| 357 | `type` |
| 357 | `string` |
| 361 | `items` |
| 361 | `type` |
| 361 | `string` |
| 367 | `$schema` |
| 368 | `$id` |
| 369 | `$ref` |
| 370 | `$comment` |
| 371 | `$defs` |
| 372 | `definitions` |
| 373 | `title` |
| 374 | `default` |
| 375 | `examples` |
| 376 | `additionalProperties` |
| 377 | `propertyNames` |
| 378 | `patternProperties` |
| 379 | `unevaluatedProperties` |
| 380 | `unevaluatedItems` |
| 381 | `minLength` |
| 382 | `maxLength` |
| 383 | `exclusiveMinimum` |
| 384 | `exclusiveMaximum` |
| 385 | `minItems` |
| 386 | `maxItems` |
| 387 | `uniqueItems` |
| 388 | `multipleOf` |
| 389 | `contains` |
| 390 | `prefixItems` |
| 391 | `additionalItems` |
| 392 | `format` |
| 393 | `contentSchema` |
| 394 | `contentMediaType` |
| 395 | `contentEncoding` |
| 396 | `deprecated` |
| 397 | `readOnly` |
| 398 | `writeOnly` |
| 399 | `dependencies` |
| 400 | `dependentSchemas` |
| 401 | `dependentRequired` |
| 402 | `not` |
| 403 | `if` |
| 404 | `then` |
| 405 | `else` |
| 412 | `properties` |
| 419 | `items` |
| 425 | `properties` |
| 431 | `required` |
| 440 | `required` |
| 449 | `type` |
| 449 | `object` |
| 450 | `properties` |
| 451 | `reason` |
| 452 | `type` |
| 452 | `string` |
| 453 | `description` |
| 453 | `Brief explanation of why you are calling this tool` |
| 456 | `required` |
| 456 | `reason` |
| 464 | `type` |
| 464 | `object` |
| 467 | `properties` |
| 475 | `reason` |
| 477 | `type` |
| 477 | `string` |
| 478 | `description` |
| 478 | `Brief explanation of why you are calling this tool` |
| 481 | `properties` |
| 482 | `required` |
| 482 | `reason` |
| 492 | `You are a Claude agent, built on Anthropic's Claude Agent SDK.` |
| 495 | `OpenCode` |
| 495 | `Antigravity` |
| 496 | `opencode` |
| 496 | `antigravity` |
| 497 | `OPENCODE` |
| 497 | `ANTIGRAVITY` |
| 521 | `system` |
| 526 | `assistant` |
| 526 | `model` |
| 529 | `text` |
| 547 | `/google/thought_signature` |
| 548 | `/google/thoughtSignature` |
| 557 | `thought_signature` |
| 558 | `thoughtSignature` |
| 575 | `functionCall` |
| 576 | `id` |
| 577 | `name` |
| 578 | `args` |
| 584 | `thoughtSignature` |
| 592 | `role` |
| 592 | `model` |
| 593 | `parts` |
| 597 | `tool` |
| 603 | `tool` |
| 606 | `result` |
| 610 | `result` |
| 614 | `name` |
| 615 | `response` |
| 618 | `id` |
| 622 | `role` |
| 622 | `user` |
| 623 | `parts` |
| 624 | `functionResponse` |
| 632 | `role` |
| 632 | `user` |
| 633 | `parts` |
| 633 | `text` |
| 643 | `role` |
| 643 | `user` |
| 645 | `parts` |
| 655 | `role` |
| 657 | `parts` |
| 665 | `role` |
| 666 | `parts` |
| 672 | `role` |
| 672 | `model` |
| 674 | `parts` |
| 677 | `thought` |
| 678 | `functionCall` |
| 680 | `text` |
| 685 | `thoughtSignature` |
| 685 | `thought_signature` |
| 698 | `functionCall` |
| 699 | `id` |
| 701 | `thoughtSignature` |
| 702 | `thought_signature` |
| 725 | `thoughtSignature` |
| 739 | `role` |
| 739 | `user` |
| 740 | `parts` |
| 740 | `text` |
| 740 | `Hello` |
| 742 | `role` |
| 742 | `user` |
| 746 | `role` |
| 746 | `user` |
| 747 | `parts` |
| 747 | `text` |
| 747 | `...` |
| 758 | `function` |
| 759 | `name` |
| 761 | `description` |
| 764 | `parameters` |
| 767 | `name` |
| 768 | `description` |
| 769 | `parameters` |
| 771 | `name` |
| 773 | `description` |
| 774 | `input_schema` |
| 777 | `name` |
| 778 | `description` |
| 779 | `parameters` |
| 786 | `functionDeclarations` |
| 789 | `functionCallingConfig` |
| 790 | `mode` |
| 790 | `AUTO` |
| 796 | `ag-v1-sess` |
| 797 | `ag-v1-conv` |
| 798 | `ag-v1-traj` |
| 804 | `agent/{conversation_id}/{now_ms}/{trajectory_id}/{step}` |
| 807 | `contents` |
| 808 | `generationConfig` |
| 809 | `temperature` |
| 810 | `topP` |
| 811 | `topK` |
| 812 | `maxOutputTokens` |
| 814 | `sessionId` |
| 818 | `\n\n` |
| 819 | `systemInstruction` |
| 820 | `parts` |
| 820 | `text` |
| 825 | `tools` |
| 828 | `toolConfig` |
| 832 | `project` |
| 833 | `model` |
| 834 | `userAgent` |
| 834 | `antigravity` |
| 835 | `requestType` |
| 835 | `agent` |
| 836 | `requestId` |
| 837 | `request` |
| 853 | `{}/v1internal:streamGenerateContent?alt=sse` |
| 860 | `Authorization` |
| 860 | `Bearer {access_token}` |
| 861 | `Content-Type` |
| 861 | `application/json` |
| 862 | `User-Agent` |
| 863 | `X-Client-Name` |
| 863 | `antigravity` |
| 864 | `X-Client-Version` |
| 864 | `2.11.0` |
| 867 | `Failed to send request to Antigravity stream endpoint` |
| 872 | `Antigravity HTTP error {status}: {err_text}` |
| 879 | `stop` |
| 885 | `data: ` |
| 886 | `[DONE]` |
| 891 | `/response/candidates` |
| 892 | `/candidates` |
| 897 | `finishReason` |
| 898 | `MAX_TOKENS` |
| 899 | `length` |
| 904 | `/content/parts` |
| 908 | `thoughtSignature` |
| 909 | `thought_signature` |
| 919 | `thought` |
| 923 | `functionCall` |
| 925 | `text` |
| 940 | `text` |
| 947 | `functionCall` |
| 949 | `thoughtSignature` |
| 950 | `thought_signature` |
| 960 | `name` |
| 965 | `args` |
| 967 | `{}` |
| 969 | `id` |
| 974 | `call_{}_{}` |
| 975 | `fc` |
| 986 | `function` |
| 1004 | `tool_calls` |
| 1015 | `tool_calls` |
| 1034 | `v1_adapter_tests.rs` |

---

## `llm/providers/antigravity/v1_adapter_tests.rs`

- **Path**: `src/llm/providers/antigravity/v1_adapter_tests.rs`
- **Comments**: 15
- **Strings**: 107

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 33 | `//` | Assert root properties |
| 39 | `//` | Assert path |
| 42 | `//` | Assert offset flattened from ["integer", "null"] to "integer" with nullable: true |
| 46 | `//` | Assert options flattened from anyOf to "string" with nullable: true |
| 50 | `//` | Assert required pruned non-existent field |
| 62 | `//` | Retrieve from cache |
| 66 | `//` | Non-existent ID returns None |
| 69 | `//` | Embedded signature in ID takes priority |
| 127 | `//` | Model turn must have functionCall with thoughtSignature |
| 208 | `//` | Valid standard and padded base64 |
| 216 | `//` | Auto-padded unpadded valid base64 |
| 217 | `//` | len 6 -> + "==" -> len 8 |
| 223 | `//` | Corrupted / impossible length (len % 4 == 1) |
| 227 | `//` | Corrupted non-base64 characters |
| 231 | `//` | Empty strings |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 8 | `$schema` |
| 8 | `http://json-schema.org/draft-07/schema#` |
| 9 | `title` |
| 9 | `ZedToolParameters` |
| 10 | `type` |
| 10 | `object` |
| 11 | `additionalProperties` |
| 12 | `properties` |
| 13 | `path` |
| 14 | `type` |
| 14 | `string` |
| 15 | `description` |
| 15 | `File path` |
| 17 | `offset` |
| 18 | `type` |
| 18 | `integer` |
| 18 | `null` |
| 19 | `description` |
| 19 | `Offset in bytes` |
| 21 | `options` |
| 22 | `anyOf` |
| 23 | `type` |
| 23 | `string` |
| 24 | `type` |
| 24 | `null` |
| 28 | `required` |
| 28 | `path` |
| 28 | `nonexistent` |
| 34 | `type` |
| 34 | `object` |
| 35 | `$schema` |
| 36 | `title` |
| 37 | `additionalProperties` |
| 40 | `properties` |
| 40 | `path` |
| 40 | `type` |
| 40 | `string` |
| 43 | `properties` |
| 43 | `offset` |
| 43 | `type` |
| 43 | `integer` |
| 44 | `properties` |
| 44 | `offset` |
| 44 | `nullable` |
| 47 | `properties` |
| 47 | `options` |
| 47 | `type` |
| 47 | `string` |
| 48 | `properties` |
| 48 | `options` |
| 48 | `nullable` |
| 51 | `required` |
| 53 | `path` |
| 58 | `call_test_12345` |
| 59 | `test_signature_abc_xyz==` |
| 67 | `call_non_existent` |
| 70 | `call_test_12345__thought__embedded_sig_999` |
| 73 | `embedded_sig_999` |
| 80 | `ag/gemini-3.8-flash-high` |
| 83 | `user` |
| 84 | `list directory contents` |
| 91 | `assistant` |
| 94 | `call_list_dir_1` |
| 95 | `function` |
| 97 | `default_api:list_directory` |
| 98 | `{"dir_path":"/workspace"}` |
| 109 | `tool` |
| 110 | `["file1.txt","file2.txt"]` |
| 112 | `call_list_dir_1` |
| 124 | `test-project` |
| 125 | `request` |
| 125 | `contents` |
| 128 | `role` |
| 128 | `model` |
| 129 | `parts` |
| 132 | `functionCall` |
| 136 | `thoughtSignature` |
| 140 | `functionCall` |
| 140 | `name` |
| 141 | `default_api:list_directory` |
| 147 | `call_cached_sig_test` |
| 148 | `EuwGCukGAXLI2custom_stored_signature` |
| 152 | `gemini-3.8-flash-high` |
| 155 | `user` |
| 156 | `hi` |
| 163 | `assistant` |
| 167 | `function` |
| 169 | `test_tool` |
| 170 | `{}` |
| 188 | `test-project` |
| 189 | `request` |
| 189 | `contents` |
| 190 | `role` |
| 190 | `model` |
| 191 | `parts` |
| 193 | `thoughtSignature` |
| 209 | `EuwGCuk=` |
| 210 | `EuwGCukG` |
| 212 | `EuwGCuk=` |
| 213 | `EuwGCuk=` |
| 217 | `EuwGCg` |
| 219 | `EuwGCg` |
| 220 | `EuwGCg==` |
| 224 | `EuwG1` |
| 225 | `EuwG1` |
| 228 | `EuwG!@#$%^&*()` |
| 229 | `EuwG!@#$%^&*()` |

---

## `llm/providers/mod.rs`

- **Path**: `src/llm/providers/mod.rs`
- **Comments**: 0
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

_No string literals found._

---

## `llm/types.rs`

- **Path**: `src/llm/types.rs`
- **Comments**: 1
- **Strings**: 2

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 20 | `//` | Ollama native API fallback format |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 16 | `Option::is_none` |
| 26 | `Option::is_none` |

---

## `llm/v1_engine.rs`

- **Path**: `src/llm/v1_engine.rs`
- **Comments**: 2
- **Strings**: 13

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 98 | `///` | Helper function to extract text content from OpenAI message content |
| 115 | `///` | Sanitize function name to conform to common tool calling standards |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 13 | `{}-{:x}-{:x}` |
| 24 | `default_tool_call_id` |
| 26 | `default_function_type` |
| 26 | `type` |
| 29 | `thought_signature` |
| 29 | `thoughtSignature` |
| 38 | `call_{}` |
| 38 | `tc` |
| 42 | `function` |
| 56 | `thought_signature` |
| 56 | `thoughtSignature` |
| 105 | `text` |
| 118 | `_unknown` |

---
