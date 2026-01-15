//! Path resolution utilities for wreckit
//!
//! Provides functions to locate the repository root and construct paths
//! to various wreckit files and directories.

use std::path::{Path, PathBuf};

use crate::errors::{Result, WreckitError};

/// Find the repository root containing both .git and .wreckit directories.
///
/// Walks up the directory tree from the starting directory looking for
/// a directory that contains both .git and .wreckit.
///
/// # Arguments
/// * `start_cwd` - The directory to start searching from
///
/// # Returns
/// The path to the repository root
///
/// # Errors
/// * `RepoNotFound` - If no repository root is found
/// * `RepoNotFound` - If .wreckit exists without .git
pub fn find_repo_root(start_cwd: &Path) -> Result<PathBuf> {
    let mut current = start_cwd
        .canonicalize()
        .map_err(|e| WreckitError::RepoNotFound(format!("Cannot resolve path: {}", e)))?;

    loop {
        let git_dir = current.join(".git");
        let wreckit_dir = current.join(".wreckit");

        let has_git = git_dir.exists();
        let has_wreckit = wreckit_dir.exists();

        if has_git && has_wreckit {
            return Ok(current);
        }

        if has_wreckit && !has_git {
            return Err(WreckitError::RepoNotFound(format!(
                "Found .wreckit at {} but no .git directory",
                current.display()
            )));
        }

        match current.parent() {
            Some(parent) if parent != current => {
                current = parent.to_path_buf();
            }
            _ => {
                return Err(WreckitError::RepoNotFound(
                    "Could not find repository root with .git and .wreckit directories".to_string(),
                ));
            }
        }
    }
}

/// Resolve the current working directory, optionally using an override.
///
/// # Arguments
/// * `cwd_option` - Optional override for the working directory
///
/// # Returns
/// The resolved working directory path
pub fn resolve_cwd(cwd_option: Option<&Path>) -> PathBuf {
    match cwd_option {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

/// Get the path to the .wreckit directory.
pub fn get_wreckit_dir(root: &Path) -> PathBuf {
    root.join(".wreckit")
}

/// Get the path to the config.json file.
pub fn get_config_path(root: &Path) -> PathBuf {
    get_wreckit_dir(root).join("config.json")
}

/// Get the path to the index.json file.
pub fn get_index_path(root: &Path) -> PathBuf {
    get_wreckit_dir(root).join("index.json")
}

/// Get the path to the prompts directory.
pub fn get_prompts_dir(root: &Path) -> PathBuf {
    get_wreckit_dir(root).join("prompts")
}

/// Get the path to the items directory.
pub fn get_items_dir(root: &Path) -> PathBuf {
    get_wreckit_dir(root).join("items")
}

/// Get the path to a specific item's directory.
pub fn get_item_dir(root: &Path, id: &str) -> PathBuf {
    get_items_dir(root).join(id)
}

/// Get the path to an item's item.json file.
pub fn get_item_json_path(root: &Path, id: &str) -> PathBuf {
    get_item_dir(root, id).join("item.json")
}

/// Get the path to an item's prd.json file.
pub fn get_prd_path(root: &Path, id: &str) -> PathBuf {
    get_item_dir(root, id).join("prd.json")
}

/// Get the path to an item's research.md file.
pub fn get_research_path(root: &Path, id: &str) -> PathBuf {
    get_item_dir(root, id).join("research.md")
}

/// Get the path to an item's plan.md file.
pub fn get_plan_path(root: &Path, id: &str) -> PathBuf {
    get_item_dir(root, id).join("plan.md")
}

/// Get the path to an item's progress.log file.
pub fn get_progress_log_path(root: &Path, id: &str) -> PathBuf {
    get_item_dir(root, id).join("progress.log")
}

/// Get the path to an item's prompt.md file.
pub fn get_prompt_path(root: &Path, id: &str) -> PathBuf {
    get_item_dir(root, id).join("prompt.md")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        std::fs::create_dir(temp.path().join(".wreckit")).unwrap();
        temp
    }

    #[test]
    fn test_find_repo_root_from_root() {
        let temp = setup_repo();
        let root = find_repo_root(temp.path()).unwrap();
        assert_eq!(root.canonicalize().unwrap(), temp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_find_repo_root_from_subdir() {
        let temp = setup_repo();
        let subdir = temp.path().join("src").join("deep");
        std::fs::create_dir_all(&subdir).unwrap();

        let root = find_repo_root(&subdir).unwrap();
        assert_eq!(root.canonicalize().unwrap(), temp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_find_repo_root_wreckit_without_git() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(".wreckit")).unwrap();

        let result = find_repo_root(temp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no .git directory"));
    }

    #[test]
    fn test_find_repo_root_not_found() {
        let temp = TempDir::new().unwrap();
        // No .git or .wreckit

        let result = find_repo_root(temp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not find"));
    }

    #[test]
    fn test_get_wreckit_dir() {
        let root = PathBuf::from("/repo");
        assert_eq!(get_wreckit_dir(&root), PathBuf::from("/repo/.wreckit"));
    }

    #[test]
    fn test_get_config_path() {
        let root = PathBuf::from("/repo");
        assert_eq!(get_config_path(&root), PathBuf::from("/repo/.wreckit/config.json"));
    }

    #[test]
    fn test_get_item_paths() {
        let root = PathBuf::from("/repo");
        let id = "test-001";

        assert_eq!(get_item_dir(&root, id), PathBuf::from("/repo/.wreckit/items/test-001"));
        assert_eq!(get_item_json_path(&root, id), PathBuf::from("/repo/.wreckit/items/test-001/item.json"));
        assert_eq!(get_prd_path(&root, id), PathBuf::from("/repo/.wreckit/items/test-001/prd.json"));
        assert_eq!(get_research_path(&root, id), PathBuf::from("/repo/.wreckit/items/test-001/research.md"));
        assert_eq!(get_plan_path(&root, id), PathBuf::from("/repo/.wreckit/items/test-001/plan.md"));
        assert_eq!(get_progress_log_path(&root, id), PathBuf::from("/repo/.wreckit/items/test-001/progress.log"));
    }

    #[test]
    fn test_resolve_cwd_with_override() {
        let path = PathBuf::from("/custom/path");
        let resolved = resolve_cwd(Some(&path));
        assert_eq!(resolved, path);
    }

    #[test]
    fn test_resolve_cwd_without_override() {
        let resolved = resolve_cwd(None);
        assert!(!resolved.as_os_str().is_empty());
    }
}
