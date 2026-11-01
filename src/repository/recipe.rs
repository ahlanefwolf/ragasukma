use crate::engine::actions::recipe::types::AutomationRecipe;
use std::fs;
use std::path::PathBuf;

/// Gets storage directory for recipes: assets/recipes/ or ~/.config/ragasukma/recipes/
pub fn get_recipes_dir() -> PathBuf {
    if PathBuf::from("Cargo.toml").exists() {
        let local = PathBuf::from("assets/recipes");
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
    let data_recipes = crate::config::get_app_data_dir().join("recipes");
    if data_recipes.exists() {
        return data_recipes;
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
    recipes
        .into_iter()
        .find(|r| r.recipe_id.eq_ignore_ascii_case(clean_id))
}

/// Save a recipe to the designated recipe directory
pub fn save_recipe(recipe: &AutomationRecipe) -> Result<PathBuf, String> {
    let dir = get_recipes_dir();
    let filename = format!("{}.json", recipe.recipe_id.trim().to_lowercase());
    let target_path = dir.join(filename);
    let json_data = serde_json::to_string_pretty(recipe)
        .map_err(|e| format!("Failed to serialize recipe: {e}"))?;
    fs::write(&target_path, json_data)
        .map_err(|e| format!("Failed to write recipe file: {e}"))?;
    Ok(target_path)
}

/// Delete a recipe by recipe_id
pub fn delete_recipe(recipe_id: &str) -> Result<bool, String> {
    let dir = get_recipes_dir();
    let filename = format!("{}.json", recipe_id.trim().to_lowercase());
    let target_path = dir.join(filename);
    if target_path.exists() {
        fs::remove_file(target_path).map_err(|e| format!("Failed to delete recipe file: {e}"))?;
        Ok(true)
    } else {
        Ok(false)
    }
}
