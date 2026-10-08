pub mod runner;
pub mod trainer;
pub mod types;

use std::sync::atomic::AtomicBool;

pub static IS_RECIPE_RUNNING: AtomicBool = AtomicBool::new(false);

pub use runner::{execute_recipe, execute_single_step};
pub use trainer::record_recipe_action;
pub use types::*;
