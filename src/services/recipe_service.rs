use serde::Deserialize;
use std::fs;
use std::net::TcpStream;

use crate::engine::actions::recipe::{
    AutomationRecipe, execute_recipe, get_recipes_dir, load_all_recipes,
};
use crate::server::send_json_response;

/// GET /api/recipes
pub fn handle_get_recipes(stream: &mut TcpStream) {
    let recipes = load_all_recipes();
    let json = serde_json::to_string(&recipes).unwrap_or_else(|_| "[]".to_string());
    send_json_response(stream, 200, &json);
}

/// POST /api/recipes/save
pub fn handle_save_recipe(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct SaveRecipePayload {
        #[serde(flatten)]
        recipe: AutomationRecipe,
        #[serde(default)]
        previous_id: Option<String>,
    }

    match serde_json::from_str::<SaveRecipePayload>(body_str) {
        Ok(payload) => {
            let mut recipe = payload.recipe;
            let recipe_id = recipe.recipe_id.trim().to_string();
            if recipe_id.is_empty() {
                send_json_response(stream, 400, "{\"error\":\"Recipe ID cannot be empty\"}");
                return;
            }

            // Auto-reindex step property sequentially (1..=N)
            for (idx, step) in recipe.steps.iter_mut().enumerate() {
                step.step = idx + 1;
            }

            let dir = get_recipes_dir();
            let _ = fs::create_dir_all(&dir);
            let file_path = dir.join(format!("{recipe_id}.json"));

            // If renamed from a previous slug, remove the old json file
            if let Some(prev) = payload.previous_id {
                let prev_clean = prev.trim();
                if !prev_clean.is_empty() && prev_clean != recipe_id {
                    let old_path = dir.join(format!("{prev_clean}.json"));
                    if old_path.exists() {
                        let _ = fs::remove_file(old_path);
                        crate::term_log!("[Recipe Service] Renamed recipe from '{}' -> '{}'", prev_clean, recipe_id);
                    }
                }
            }

            match serde_json::to_string_pretty(&recipe) {
                Ok(formatted) => {
                    if let Err(e) = fs::write(&file_path, formatted) {
                        send_json_response(stream, 500, &format!("{{\"error\":\"Failed to save recipe: {e}\"}}"));
                    } else {
                        crate::term_log!("[Recipe Service] Saved recipe '{}' with {} steps", recipe_id, recipe.steps.len());
                        send_json_response(stream, 200, "{\"success\":true}");
                    }
                }
                Err(e) => {
                    send_json_response(stream, 500, &format!("{{\"error\":\"Failed to serialize recipe: {e}\"}}"));
                }
            }
        }
        Err(e) => {
            send_json_response(stream, 400, &format!("{{\"error\":\"Invalid recipe JSON: {e}\"}}"));
        }
    }
}

/// POST /api/recipes/delete
pub fn handle_delete_recipe(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct DeleteRecipePayload {
        recipe_id: String,
    }

    if let Ok(item) = serde_json::from_str::<DeleteRecipePayload>(body_str) {
        let clean_id = item.recipe_id.trim();
        let file_path = get_recipes_dir().join(format!("{clean_id}.json"));
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
            crate::term_log!("[Recipe Service] Deleted recipe '{}'", clean_id);
            send_json_response(stream, 200, "{\"success\":true}");
        } else {
            send_json_response(stream, 404, "{\"error\":\"Recipe not found\"}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid delete request payload\"}");
    }
}

/// POST /api/recipes/run
pub fn handle_run_recipe(body_str: &str, stream: &mut TcpStream) {
    #[derive(Deserialize)]
    struct RunRecipePayload {
        recipe_id: String,
        params: Option<serde_json::Value>,
    }

    if let Ok(item) = serde_json::from_str::<RunRecipePayload>(body_str) {
        let clean_id = item.recipe_id.trim();
        if clean_id.is_empty() {
            send_json_response(stream, 400, "{\"error\":\"Recipe ID cannot be empty\"}");
            return;
        }

        let payload_str = if let Some(params) = item.params {
            if params.is_object() && !params.as_object().unwrap().is_empty() {
                format!("{clean_id}|{}", params)
            } else {
                clean_id.to_string()
            }
        } else {
            clean_id.to_string()
        };

        if let Some((title, msg)) = execute_recipe(&payload_str) {
            crate::term_log!("[Recipe Service] Run result: {} — {}", title, msg);
            send_json_response(stream, 200, &format!("{{\"success\":true,\"message\":{}}}", serde_json::to_string(&msg).unwrap_or_default()));
        } else {
            send_json_response(stream, 400, "{\"error\":\"Failed to execute recipe\"}");
        }
    } else {
        send_json_response(stream, 400, "{\"error\":\"Invalid run payload\"}");
    }
}
