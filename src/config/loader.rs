//! Configuration loading with defaults

use std::path::Path;

use crate::errors::Result;
use crate::fs;
use crate::schemas::Config;

/// Load configuration from the repository, falling back to defaults.
///
/// If config.json exists, it will be read and merged with defaults.
/// If it doesn't exist, default configuration is returned.
///
/// # Arguments
/// * `root` - Path to the repository root
///
/// # Returns
/// The resolved configuration
pub fn load_config(root: &Path) -> Result<Config> {
    fs::read_config(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as std_fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_defaults() {
        let temp = TempDir::new().unwrap();
        std_fs::create_dir(temp.path().join(".wreckit")).unwrap();

        let config = load_config(temp.path()).unwrap();
        assert_eq!(config.base_branch, "main");
        assert_eq!(config.branch_prefix, "wreckit/");
        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.timeout_seconds, 3600);
    }

    #[test]
    fn test_load_config_from_file() {
        let temp = TempDir::new().unwrap();
        let wreckit_dir = temp.path().join(".wreckit");
        std_fs::create_dir(&wreckit_dir).unwrap();

        let config_content = r#"{
            "base_branch": "develop",
            "branch_prefix": "feature/",
            "max_iterations": 50
        }"#;
        std_fs::write(wreckit_dir.join("config.json"), config_content).unwrap();

        let config = load_config(temp.path()).unwrap();
        assert_eq!(config.base_branch, "develop");
        assert_eq!(config.branch_prefix, "feature/");
        assert_eq!(config.max_iterations, 50);
        // Default for unspecified field
        assert_eq!(config.timeout_seconds, 3600);
    }
}
