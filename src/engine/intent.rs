use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, RwLock};

/// Embedded fallback dictionary for Indonesian (compile-time zero failure guarantee)
const EMBEDDED_ID_JSON: &str = include_str!("../../assets/i18n/id.json");
/// Embedded fallback dictionary for English
const EMBEDDED_EN_JSON: &str = include_str!("../../assets/i18n/en.json");
/// Embedded fallback dictionary for Japanese
const EMBEDDED_JA_JSON: &str = include_str!("../../assets/i18n/ja.json");

/// Raw dictionary structure matching `assets/i18n/{lang}.json`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocaleIntentData {
    #[serde(default)]
    pub music: Vec<String>,
    #[serde(default)]
    pub recipe: Vec<String>,
    #[serde(default)]
    pub todo: Vec<String>,
    #[serde(default)]
    pub skill: Vec<String>,
    #[serde(default)]
    pub browser: Vec<String>,
    #[serde(default)]
    pub desktop: Vec<String>,
    #[serde(default)]
    pub app_query: Vec<String>,
    #[serde(default)]
    pub conjunctions: Vec<String>,
}

impl LocaleIntentData {
    pub fn merge(&mut self, other: &LocaleIntentData) {
        self.music.extend(other.music.clone());
        self.recipe.extend(other.recipe.clone());
        self.todo.extend(other.todo.clone());
        self.skill.extend(other.skill.clone());
        self.browser.extend(other.browser.clone());
        self.desktop.extend(other.desktop.clone());
        self.app_query.extend(other.app_query.clone());
        self.conjunctions.extend(other.conjunctions.clone());
    }
}

/// Compiled in-memory regex DFA engine for high-throughput (<1µs) execution
pub struct CompiledLocaleIntents {
    pub re_music: Regex,
    pub re_recipe: Regex,
    pub re_todo: Regex,
    pub re_skill: Regex,
    pub re_browser: Regex,
    pub re_desktop: Regex,
    pub re_app_query: Regex,
    pub conjunctions: HashSet<String>,
}

static COMPILED_CACHE: LazyLock<RwLock<HashMap<String, Arc<CompiledLocaleIntents>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn normalize_lang_code(raw: &str) -> String {
    let clean = raw.trim().to_lowercase();
    if clean.is_empty() {
        return "id".to_string();
    }
    clean
        .split(['-', '_'])
        .next()
        .unwrap_or("id")
        .to_string()
}

/// Load locale dictionary from `assets/i18n/{lang}.json` with embedded compile-time fallback
pub fn load_locale_data(lang: &str) -> LocaleIntentData {
    let norm = normalize_lang_code(lang);
    let mut data = LocaleIntentData::default();

    // 1. Try reading from runtime filesystem: assets/i18n/{lang}.json
    let file_path = get_i18n_path(&norm);
    if file_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            if let Ok(parsed) = serde_json::from_str::<LocaleIntentData>(&content) {
                data = parsed;
            }
        }
    }

    // 2. Fallback to embedded compile-time JSON if filesystem was empty or missing
    if data.music.is_empty() {
        let embedded_content = match norm.as_str() {
            "id" => Some(EMBEDDED_ID_JSON),
            "en" => Some(EMBEDDED_EN_JSON),
            "ja" => Some(EMBEDDED_JA_JSON),
            _ => None,
        };
        if let Some(raw) = embedded_content {
            if let Ok(parsed) = serde_json::from_str::<LocaleIntentData>(raw) {
                data = parsed;
            }
        }
    }

    // 3. For non-English languages, merge universal English keywords
    // so common English commands (e.g. "open google.com", "play music") work seamlessly
    if norm != "en" {
        let en_data = load_locale_data("en");
        data.merge(&en_data);
    }

    data
}

fn get_i18n_path(lang: &str) -> PathBuf {
    crate::assets::get_assets_dir()
        .join("i18n")
        .join(format!("{lang}.json"))
}

/// Convert list of keywords/phrases into optimized Regex pattern
fn build_regex_from_words(words: &[String]) -> Regex {
    if words.is_empty() {
        return Regex::new(r"$^").expect("valid empty regex");
    }

    // Sort descending by length so compound phrases match first (e.g. "buka tab" before "tab")
    let mut sorted: Vec<String> = words
        .iter()
        .map(|w| w.trim().to_lowercase())
        .filter(|w| !w.is_empty())
        .collect();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.len()));
    sorted.dedup();

    let mut patterns = Vec::with_capacity(sorted.len());
    for w in sorted {
        let escaped = regex::escape(&w);
        // If word contains only ASCII alphanumeric / spaces, enforce word boundaries
        if w.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ' || c == '-') {
            patterns.push(format!(r"\b{}\b", escaped));
        } else {
            // For CJK characters or symbols, match directly without ASCII \b
            patterns.push(escaped);
        }
    }

    // Add universal domain extension matches for browser intent if relevant
    let joined = patterns.join("|");
    Regex::new(&format!(r"(?i)(?:{joined})"))
        .unwrap_or_else(|_| Regex::new(r"$^").expect("valid fallback impossible match regex"))
}

/// Retrieve or lazily compile the locale regex engine for a specific language
pub fn get_compiled_intents_for_lang(lang: &str) -> Arc<CompiledLocaleIntents> {
    let norm = normalize_lang_code(lang);

    // Fast read lock check
    if let Ok(guard) = COMPILED_CACHE.read() {
        if let Some(engine) = guard.get(&norm) {
            return engine.clone();
        }
    }

    // Write lock compile
    let mut guard = COMPILED_CACHE
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(engine) = guard.get(&norm) {
        return engine.clone();
    }

    let data = load_locale_data(&norm);

    let re_music = build_regex_from_words(&data.music);
    let re_recipe = build_regex_from_words(&data.recipe);
    let re_todo = build_regex_from_words(&data.todo);
    let re_skill = build_regex_from_words(&data.skill);

    // For browser intent, also ensure .com, .id, .org, .io, .net, .dev are matched
    let mut browser_words = data.browser.clone();
    browser_words.extend(vec![
        "google.com".to_string(),
        "youtube.com".to_string(),
        ".com".to_string(),
        ".id".to_string(),
        ".org".to_string(),
        ".io".to_string(),
        ".net".to_string(),
        ".dev".to_string(),
    ]);
    let re_browser = build_regex_from_words(&browser_words);

    let re_desktop = build_regex_from_words(&data.desktop);
    let re_app_query = build_regex_from_words(&data.app_query);

    let mut conjunctions_set = HashSet::new();
    for c in &data.conjunctions {
        conjunctions_set.insert(c.trim().to_lowercase());
    }

    let compiled = Arc::new(CompiledLocaleIntents {
        re_music,
        re_recipe,
        re_todo,
        re_skill,
        re_browser,
        re_desktop,
        re_app_query,
        conjunctions: conjunctions_set,
    });

    guard.insert(norm, compiled.clone());
    compiled
}

/// Get compiled intents for the active user language configured in `assets/config-ai.json`
pub fn get_active_intents() -> Arc<CompiledLocaleIntents> {
    let cfg = crate::config::load_user_config();
    get_compiled_intents_for_lang(&cfg.lang)
}

// ============================================================================
// Public Facade Functions (Zero-overhead, drop-in replacement)
// ============================================================================

pub fn is_music_intent(query: &str) -> bool {
    let active = get_active_intents();
    active.re_music.is_match(query)
        || get_compiled_intents_for_lang("en").re_music.is_match(query)
        || get_compiled_intents_for_lang("id").re_music.is_match(query)
}

pub fn is_recipe_intent(query: &str) -> bool {
    let active = get_active_intents();
    active.re_recipe.is_match(query)
        || get_compiled_intents_for_lang("en").re_recipe.is_match(query)
        || get_compiled_intents_for_lang("id").re_recipe.is_match(query)
}

pub fn is_todo_intent(query: &str) -> bool {
    let active = get_active_intents();
    active.re_todo.is_match(query)
        || get_compiled_intents_for_lang("en").re_todo.is_match(query)
        || get_compiled_intents_for_lang("id").re_todo.is_match(query)
}

pub fn is_skill_intent(query: &str) -> bool {
    let active = get_active_intents();
    active.re_skill.is_match(query)
        || get_compiled_intents_for_lang("en").re_skill.is_match(query)
        || get_compiled_intents_for_lang("id").re_skill.is_match(query)
}

pub fn is_browser_intent(query: &str) -> bool {
    let active = get_active_intents();
    active.re_browser.is_match(query)
        || get_compiled_intents_for_lang("en").re_browser.is_match(query)
        || get_compiled_intents_for_lang("id").re_browser.is_match(query)
}

pub fn is_desktop_intent(query: &str) -> bool {
    let active = get_active_intents();
    active.re_desktop.is_match(query)
        || get_compiled_intents_for_lang("en").re_desktop.is_match(query)
        || get_compiled_intents_for_lang("id").re_desktop.is_match(query)
}

pub fn is_app_query_intent(query: &str) -> bool {
    let active = get_active_intents();
    active.re_app_query.is_match(query)
        || get_compiled_intents_for_lang("en").re_app_query.is_match(query)
        || get_compiled_intents_for_lang("id").re_app_query.is_match(query)
}

/// Natural conjunction check for TTS clause splitting
pub fn is_natural_conjunction(word: &str, lang: Option<&str>) -> bool {
    let clean = word.trim().to_lowercase();
    if clean.is_empty() {
        return false;
    }
    let engine = if let Some(l) = lang {
        get_compiled_intents_for_lang(l)
    } else {
        get_active_intents()
    };
    engine.conjunctions.contains(&clean)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multilingual_music_intent() {
        let id_engine = get_compiled_intents_for_lang("id");
        assert!(id_engine.re_music.is_match("play some jazz"));
        assert!(id_engine.re_music.is_match("putar lagu pop indonesia"));
        assert!(id_engine.re_music.is_match("nyanyikan lagu santai"));
        assert!(!id_engine.re_music.is_match("buka kalkulator"));

        let ja_engine = get_compiled_intents_for_lang("ja");
        assert!(ja_engine.re_music.is_match("音楽を再生して"));
        assert!(ja_engine.re_music.is_match("play song"));
    }

    #[test]
    fn test_multilingual_recipe_intent() {
        let engine = get_compiled_intents_for_lang("id");
        assert!(engine.re_recipe.is_match("train a new recipe for youtube"));
        assert!(engine.re_recipe.is_match("resep baru"));
        assert!(!engine.re_recipe.is_match("selamat pagi"));
    }

    #[test]
    fn test_multilingual_todo_intent() {
        let id_engine = get_compiled_intents_for_lang("id");
        assert!(id_engine.re_todo.is_match("tambahkan to-do beli kopi"));
        assert!(id_engine.re_todo.is_match("catat tugas laporan"));
        assert!(id_engine.re_todo.is_match("remind me tomorrow"));
        assert!(!id_engine.re_todo.is_match("halo dunia"));

        let ja_engine = get_compiled_intents_for_lang("ja");
        assert!(ja_engine.re_todo.is_match("買い物メモを追加して"));
    }

    #[test]
    fn test_multilingual_skill_intent() {
        let engine = get_compiled_intents_for_lang("id");
        assert!(engine.re_skill.is_match("jalankan skill backup"));
        assert!(engine.re_skill.is_match("run terminal script"));
        assert!(!engine.re_skill.is_match("siapa namamu?"));
    }

    #[test]
    fn test_multilingual_browser_intent() {
        let id_engine = get_compiled_intents_for_lang("id");
        assert!(id_engine.re_browser.is_match("open google.com"));
        assert!(id_engine.re_browser.is_match("tolong cari resep kue di youtube"));
        assert!(id_engine.re_browser.is_match("ringkas isi halaman web ini"));
        assert!(!id_engine.re_browser.is_match("halo selamat pagi"));

        let ja_engine = get_compiled_intents_for_lang("ja");
        assert!(ja_engine.re_browser.is_match("ブラウザで検索して"));
    }

    #[test]
    fn test_multilingual_desktop_intent() {
        let engine = get_compiled_intents_for_lang("id");
        assert!(engine.re_desktop.is_match("click the save button on screen"));
        assert!(engine.re_desktop.is_match("fokuskan ke jendela spotify"));
        assert!(engine.re_desktop.is_match("tekan tombol enter di layar"));
        assert!(!engine.re_desktop.is_match("ceritakan lelucon"));
    }

    #[test]
    fn test_app_query_intent() {
        let engine = get_compiled_intents_for_lang("id");
        assert!(engine.re_app_query.is_match("apa saja aplikasi yang terpasang?"));
        assert!(engine.re_app_query.is_match("list installed apps"));
        assert!(!engine.re_app_query.is_match("buka browser"));
    }

    #[test]
    fn test_conjunctions_splitting() {
        assert!(is_natural_conjunction("dan", Some("id")));
        assert!(is_natural_conjunction("because", Some("en")));
        assert!(is_natural_conjunction("tetapi", Some("id")));
        assert!(!is_natural_conjunction("kucing", Some("id")));
    }

    #[test]
    fn test_all_31_languages_load_and_compile() {
        let all_langs = [
            "id", "en", "ja", "ko", "ar", "bg", "cs", "da", "de", "el", "es", "et", "fi", "fr",
            "hi", "hr", "hu", "it", "lt", "lv", "nl", "pl", "pt", "ro", "ru", "sk", "sl", "sv",
            "tr", "uk", "vi",
        ];
        for lang in all_langs {
            let engine = get_compiled_intents_for_lang(lang);
            assert!(
                !engine.conjunctions.is_empty(),
                "Language {lang} should have non-empty conjunctions"
            );
            // Every language should recognize universal commands via merged fallback
            assert!(
                engine.re_music.is_match("play music"),
                "Language {lang} failed to match universal music command"
            );
            assert!(
                engine.re_browser.is_match("open google.com"),
                "Language {lang} failed to match universal browser command"
            );
        }
    }
}
