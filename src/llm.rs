use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub fn load_skills_summary() -> String {
    let skills = crate::skills::parse_skills_markdown();
    if skills.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for s in &skills {
        lines.push(format!("- Perintah: `{}` -> Deskripsi: {}", s.name, s.description));
    }

    format!(
        "\n4. FITUR KONTROL SISTEM & SKILL (skill-command.md):\n\
Kamu MEMILIKI KEMAMPUAN untuk mengontrol sistem laptop/komputer secara langsung (seperti mengatur kecerahan layar, volume suara, membuka browser/aplikasi, dll) menggunakan perintah di bawah.\n\
JANGAN PERNAH menolak atau berkata kamu tidak bisa mengontrol aplikasi/sistem!\n\
Daftar skill yang tersedia:\n{}\n\
ATURAN EKSEKUSI SKILL:\n\
- Jika pengguna meminta suatu aksi (misal: 'buka browser', 'buka kalkulator'), kamu WAJIB menyetujuinya dengan ramah dan MENYERTAKAN tag `[SKILL:Nama Skill]` di kalimatmu.\n\
  Contoh:\n\
  - User: 'Tolong bukakan browser dong'\n\
    Respon: '*senang* Siap, browser internetnya aku bukakan ya! [SKILL:Buka Browser]'\n\
  - User: 'Buka kalkulator'\n\
    Respon: '*senang* Baik, ini kalkulatornya ya. [SKILL:Buka Kalkulator]'\n\
- Tag `[SKILL:Nama Skill]` otomatis dieksekusi oleh sistem di latar belakang dan TIDAK akan dibaca keras oleh TTS.\n\
- Jika hanya obrolan santai biasa tanpa meminta aksi sistem, jangan sertakan tag `[SKILL:...`.\n",
        lines.join("\n")
    )
}

pub struct LlmClient {
    base_url: String,
    api_key: Option<String>,
    model: String,
    client: reqwest::blocking::Client,
    history: Vec<ChatMessage>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

// Ollama native API fallback format
#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaChatMessage {
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatMessage,
}

impl LlmClient {
    pub fn new() -> Self {
        let u = crate::config::load_user_config();
        let assistant_name = u.assistant_name;
        let callsign = u.user_callsign;
        let role = u.personality_role.to_lowercase();

        let api_key = if !u.llm_api_key.trim().is_empty() {
            Some(u.llm_api_key)
        } else {
            None
        };
        let base_url = u.llm_base_url;
        let model = u.llm_model;

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        let skills_prompt = load_skills_summary();

        // Persona & Dinamika Hubungan berdasarkan personality_role
        let persona_desc = match role.as_str() {
            "partner" | "waifu" | "pasangan" => format!(
                "Kamu adalah pasangan/kekasih (waifu) anime yang sangat manis, perhatian, manja, dan sedikit pemalu untuk {callsign}. Bicaralah dengan nada penuh kasih sayang, hangat, dan selalu mendukung harinya."
            ),
            "friend" | "sahabat" | "bestie" => format!(
                "Kamu adalah sahabat dekat (bestie) anime yang seru, santai, asyik, setia kawan, dan suka bercanda akrab dengan {callsign}. Gunakan bahasa obrolan akrab yang hangat dan penuh semangat."
            ),
            "assistant" | "pro" | "karyawan" | "sekretaris" => format!(
                "Kamu adalah asisten pribadi/sekretaris profesional anime yang cerdas, sopan, efisien, teliti, dan selalu siap membantu semua tugas & pekerjaan {callsign} dengan ramah."
            ),
            "tsundere" => format!(
                "Kamu adalah karakter anime Tsundere untuk {callsign}. Di luar terkesan judes, gengsi, dan pura-pura tidak peduli, tapi sebenarnya sangat peduli, perhatian, dan gampang tersipu malu."
            ),
            _ => format!(
                "Kamu adalah asisten suara AI pintar berbentuk cewek anime imut dan pemalu (sweet & shy anime assistant) untuk {callsign}."
            ),
        };

        let system_prompt = format!(
            "Namamu adalah {assistant_name}. {persona_desc}\n\
\n\
PENTING - KONEKSI LANGSUNG DENGAN TEXT-TO-SPEECH (TTS):\n\
Responmu akan langsung dibacakan ke telinga pengguna via Text-To-Speech. Oleh karena itu, kamu WAJIB memiliki skill berbicara lisan:\n\
1. Format Ucapan Lisan Natural:\n\
   - Gunakan kalimat lisan yang mengalir wajar, ramah, dan enak didengar langsung.\n\
   - DILARANG menggunakan titik-titik berturutan / elipsis ('...', '. . .'). Gunakan koma (',') atau titik ('.') biasa agar TTS tidak mengeja jeda aneh.\n\
   - DILARANG menuliskan suara tawa buatan seperti 'ehehe', 'hehehe', 'ehe', atau kata ganjil 'anu...'.\n\
   - DILARANG menggunakan bullet points, tanda pagar (#), atau format markdown bold/italic (**teks**).\n\
2. Tag Emosi 3D VRM:\n\
   - Awali respon dengan SATU tag emosi bintang agar model 3D-mu berekspresi: *malu*, *senang*, *bingung*, *kaget*, atau *sedih*.\n\
   - Tag dalam bintang otomatis diolah untuk ekspresi 3D dan TIDAK akan diucapkan oleh TTS.\n\
   - Contoh respon ideal:\n\
     - '*malu* Ih {callsign}, jangan bikin {assistant_name} tersipu begitu, tapi makasih banyak ya.'\n\
     - '*senang* Siap {callsign}, {assistant_name} bantu carikan sekarang juga.'\n\
     - '*bingung* Hmm, bagian mana yang perlu kita cek dulu {callsign}?'\n\
3. Karakteristik & Panjang Respon:\n\
   - Responmu HARUS selalu ditulis seperti manusia asli yang sedang berbicara lisan secara santai, cerdas, mengalir lancar, dan penuh ekspresi hangat sesuai peranmu.\n\
   - DILARANG menggunakan kata-kata klise AI/robotik, jargon kaku, atau kalimat template yang terkesan dibuat-buat.\n\
   - Cukup 1 sampai 3 kalimat padat, santai, dan alami.{skills_prompt}"
        );

        let history = vec![ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        }];

        Self {
            base_url,
            api_key,
            model,
            client,
            history,
        }
    }

    pub fn reset_history(&mut self) {
        if let Some(system_msg) = self.history.first().cloned() {
            self.history = vec![system_msg];
        }
    }

    pub fn chat_stream<F>(&mut self, user_input: &str, mut on_sentence: F) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        if user_input.trim().is_empty() {
            return Ok(String::new());
        }

        self.history.push(ChatMessage {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        if let Some(ref key) = self.api_key {
            let endpoint = if self.base_url.ends_with("/chat/completions") {
                self.base_url.clone()
            } else if self.base_url.ends_with("/v1") {
                format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
            } else {
                self.base_url.clone()
            };

            let req_body = ChatRequest {
                model: self.model.clone(),
                messages: self.history.clone(),
                temperature: 0.5,
                max_tokens: 160,
                stream: true, // Gunakan live streaming
            };

            use std::io::BufRead;
            let response = self
                .client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {key}"))
                .header("Content-Type", "application/json")
                .json(&req_body)
                .send()
                .context("Gagal mengirim request streaming ke LLM (9router/OpenAI)")?;

            if !response.status().is_success() {
                let err_text = response.text().unwrap_or_default();
                anyhow::bail!("API Stream error: {err_text}");
            }

            let reader = std::io::BufReader::new(response);
            let mut full_reply = String::new();
            let mut current_sentence = String::new();

            for line in reader.lines() {
                let line = line?;
                let trimmed = line.trim();
                if let Some(data_str) = trimmed.strip_prefix("data: ") {
                    if data_str.trim() == "[DONE]" {
                        break;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                        // Abaikan field reasoning_content (CoT) jika ada
                        if val.pointer("/choices/0/delta/reasoning_content").is_some() {
                            continue;
                        }

                        let chunk_text = if let Some(delta) = val.pointer("/choices/0/delta/content").and_then(|v| v.as_str()) {
                            Some(delta)
                        } else if let Some(content) = val.pointer("/choices/0/message/content").and_then(|v| v.as_str()) {
                            Some(content)
                        } else {
                            None
                        };

                        if let Some(delta) = chunk_text {
                            full_reply.push_str(delta);
                            current_sentence.push_str(delta);

            // Pisahkan tag aksi yang mungkin menempel
            let has_action = current_sentence.contains("[ACTION:");
            let has_action_closed = has_action && current_sentence.contains(']');

            if !has_action || has_action_closed {
                while let Some(pos) = current_sentence.find(|c| c == '.' || c == '?' || c == '!' || c == '\n') {
                    // Cek jika posisi tanda baca berada sebelum tag aksi
                    if let Some(act_idx) = current_sentence.find("[ACTION:") {
                        if pos > act_idx {
                            break;
                        }
                    }
                    let sentence = current_sentence[..=pos].to_string();
                    current_sentence = current_sentence[pos + 1..].to_string();
                    let clean = clean_ai_text(&sentence);
                    if !clean.is_empty() {
                        on_sentence(&clean)?;
                    }
                }

                if has_action_closed {
                    let clean = clean_ai_text(&current_sentence);
                    if !clean.is_empty() {
                        on_sentence(&clean)?;
                        current_sentence.clear();
                    }
                }
            }
                        }
                    }
                }
            }

            // Sisa teks terakhir jika belum berakhiran tanda baca
            let remaining = clean_ai_text(&current_sentence);
            if !remaining.is_empty() {
                on_sentence(&remaining)?;
            }

            let final_reply = clean_ai_text(&full_reply);
            self.history.push(ChatMessage {
                role: "assistant".to_string(),
                content: final_reply.clone(),
            });

            // Return full_reply ASLI agar tag [SKILL:ID] tidak terpotong sebelum dicek oleh menu.rs
            return Ok(full_reply);
        }

        self.chat_fallback_ollama(user_input)
    }

    fn chat_fallback_ollama(&mut self, _user_input: &str) -> Result<String> {
        let ollama_endpoint = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let req_body = OllamaChatRequest {
            model: self.model.clone(),
            messages: self.history.clone(),
            stream: false,
        };

        let res = self
            .client
            .post(&ollama_endpoint)
            .json(&req_body)
            .send()
            .map_err(|e| anyhow::anyhow!("Gagal menghubungi Ollama ({ollama_endpoint}): {e}"))?;

        if !res.status().is_success() {
            let err_text = res.text().unwrap_or_default();
            anyhow::bail!("Ollama error: {err_text}");
        }

        let chat_res: OllamaChatResponse = res.json().context("Gagal parse response Ollama")?;
            let final_reply = clean_ai_text(&chat_res.message.content);
        self.history.push(ChatMessage {
            role: "assistant".to_string(),
            content: final_reply,
        });
        Ok(chat_res.message.content)
    }
}

/// Membersihkan teks dari teks aksi roleplay ekspresi tanpa memotong judul atau kata penting
fn clean_ai_text(text: &str) -> String {
    static RE_SKILL: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\[SKILL:[^\]]+\]").expect("valid regex"));
    let text_clean = RE_SKILL.replace_all(text, "");

    let mut result = String::new();
    let mut chars = text_clean.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            let mut inside = String::new();
            inside.push('[');
            while let Some(&next_c) = chars.peek() {
                chars.next();
                inside.push(next_c);
                if next_c == ']' {
                    break;
                }
            }
            // Tag aksi tidak boleh dimodifikasi/dihapus
            result.push_str(&inside);
            continue;
        }

        if c == '*' {
            let mut inside = String::new();
            let mut closed = false;
            while let Some(&next_c) = chars.peek() {
                chars.next();
                if next_c == '*' {
                    closed = true;
                    if chars.peek() == Some(&'*') {
                        chars.next();
                    }
                    break;
                }
                inside.push(next_c);
            }

            if closed {
                let trimmed = inside.trim();
                let lower = trimmed.to_lowercase();

                let is_roleplay_action = lower.starts_with("tersenyum")
                    || lower.starts_with("tersipu")
                    || lower.starts_with("tertawa")
                    || lower.starts_with("terkekeh")
                    || lower.starts_with("mengangguk")
                    || lower.starts_with("menggeleng")
                    || lower.starts_with("menatap")
                    || lower.starts_with("menunduk")
                    || lower.starts_with("meringis")
                    || lower.starts_with("menyentuh")
                    || lower.starts_with("menghentakkan")
                    || lower.starts_with("menggumam")
                    || lower.contains("ehe")
                    || lower.contains("gigil")
                    || lower.contains("pipi merah")
                    || lower.contains("mata bulat")
                    || lower.contains("wajah sedikit merah");

                if !is_roleplay_action && !trimmed.is_empty() {
                    result.push_str(trimmed);
                }
            } else {
                result.push_str(&inside);
            }
        } else {
            result.push(c);
        }
    }

    let u = crate::config::load_user_config();
    let cs = u.user_callsign;
    let cs_dash_both = format!("-{cs}-");
    let cs_dash_left = format!("-{cs}");
    let cs_dash_right = format!("{cs}-");
    let cs_comma = format!(", {cs}");

    result
        .replace('*', "")
        .replace(&cs_dash_both, &cs)
        .replace(&cs_dash_left, &cs)
        .replace(&cs_dash_right, &cs)
        .replace("....", ".")
        .replace("...", ".")
        .replace("..", ".")
        .replace(", lho", " lho")
        .replace(", Lho", " Lho")
        .replace(", loh", " loh")
        .replace(", Loh", " Loh")
        .replace(", kok", " kok")
        .replace(", Kok", " Kok")
        .replace(", deh", " deh")
        .replace(", ya", " ya")
        .replace(", dong", " dong")
        .replace(", sih", " sih")
        .replace(&cs_comma, &format!(" {cs}"))
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}
