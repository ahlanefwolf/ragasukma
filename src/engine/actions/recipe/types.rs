use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialAction {
    pub action: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    #[serde(default)]
    pub step: usize,
    pub action: String, // "click", "type", "enter", "scroll"
    #[serde(default)]
    pub selector: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback_selectors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRecipe {
    pub recipe_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub required_params: Vec<String>,
    pub initial_action: InitialAction,
    pub steps: Vec<RecipeStep>,
}

fn default_version() -> u32 {
    1
}

/// Step-by-step decision structure in Agentic Training Loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainStepDecision {
    #[serde(default)]
    pub reasoning: String,
    pub status: String, // "CONTINUE" or "DONE"
    #[serde(default)]
    pub recipe_id: Option<String>,
    #[serde(default)]
    pub recipe_name: Option<String>,
    #[serde(default)]
    pub required_params: Option<Vec<String>>,
    #[serde(default)]
    pub next_step: Option<RecipeStep>,
}

/// Extracts all parameter names found within `{{param_name}}` templates in steps
pub fn extract_template_params(steps: &[RecipeStep]) -> Vec<String> {
    let mut params = Vec::new();
    for step in steps {
        if let Some(ref tmpl) = step.value {
            let mut remaining = tmpl.as_str();
            while let Some(start) = remaining.find("{{") {
                if let Some(end) = remaining[start + 2..].find("}}") {
                    let key = remaining[start + 2..start + 2 + end].trim().to_string();
                    if !key.is_empty() && !params.contains(&key) {
                        params.push(key);
                    }
                    remaining = &remaining[start + 2 + end + 2..];
                } else {
                    break;
                }
            }
        }
    }
    params
}

#[derive(Debug, Clone)]
pub enum ClickTarget {
    Button(String), // Selector or button identifier
    Text(String),   // Fallback text string found in snapshot.texts
}

/// Gets storage directory for recipes: assets/recipes/ or ~/.config/ragasukma/recipes/
pub fn get_recipes_dir() -> PathBuf {
    let local = PathBuf::from("assets/recipes");
    if local.exists() || PathBuf::from("Cargo.toml").exists() {
        let _ = fs::create_dir_all(&local);
        return local;
    }
    let cfg = crate::config::get_app_config_dir().join("recipes");
    if cfg.exists() {
        return cfg;
    }
    let legacy = crate::config::get_app_config_dir().join("assets").join("recipes");
    if legacy.exists() {
        return legacy;
    }
    let _ = fs::create_dir_all(&cfg);
    cfg
}

/// Load all recipes stored in recipes/*.json
pub fn load_all_recipes() -> Vec<AutomationRecipe> {
    let mut dirs_to_scan = Vec::new();
    let local = PathBuf::from("assets/recipes");
    if local.exists() {
        dirs_to_scan.push(local);
    }
    let cfg = crate::config::get_app_config_dir().join("recipes");
    if cfg.exists() && !dirs_to_scan.contains(&cfg) {
        dirs_to_scan.push(cfg);
    }
    let legacy = crate::config::get_app_config_dir().join("assets").join("recipes");
    if legacy.exists() && !dirs_to_scan.contains(&legacy) {
        dirs_to_scan.push(legacy);
    }

    let mut recipes = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    for dir in dirs_to_scan {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(recipe) = serde_json::from_str::<AutomationRecipe>(&content) {
                            if seen_ids.insert(recipe.recipe_id.clone()) {
                                recipes.push(recipe);
                            }
                        }
                    }
                }
            }
        }
    }

    recipes.sort_by(|a, b| a.recipe_id.cmp(&b.recipe_id));
    recipes
}

/// Search recipe by recipe_id
pub fn find_recipe(recipe_id: &str) -> Option<AutomationRecipe> {
    let clean_id = recipe_id.trim();
    let recipes = load_all_recipes();
    recipes.into_iter().find(|r| r.recipe_id.eq_ignore_ascii_case(clean_id))
}
