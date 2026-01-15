//! File system utilities for wreckit
//!
//! Provides path resolution and JSON file operations.

mod json;
mod paths;

pub use json::{
    read_config, read_item, read_json, read_prd, write_item, write_json, write_prd,
};
pub use paths::{
    find_repo_root, get_config_path, get_item_dir, get_items_dir, get_plan_path,
    get_progress_log_path, get_prompts_dir, get_prd_path, get_research_path, get_wreckit_dir,
    resolve_cwd,
};
