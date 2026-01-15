//! JSON file operations with schema validation
//!
//! Provides functions to read and write JSON files with serde validation.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::{Result, WreckitError};
use crate::schemas::{Config, Item, Prd};

use super::paths::{get_config_path, get_item_json_path, get_prd_path};

/// Read and deserialize a JSON file.
///
/// # Arguments
/// * `path` - Path to the JSON file
///
/// # Returns
/// The deserialized value
///
/// # Errors
/// * `FileNotFound` - If the file does not exist
/// * `InvalidJson` - If the file contains invalid JSON
/// * `SchemaValidation` - If the JSON does not match the expected schema
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            WreckitError::FileNotFound(format!("File not found: {}", path.display()))
        } else {
            WreckitError::Io(e)
        }
    })?;

    serde_json::from_str(&content).map_err(|e| {
        WreckitError::InvalidJson(format!("Invalid JSON in file {}: {}", path.display(), e))
    })
}

/// Write a value to a JSON file with pretty formatting.
///
/// Uses atomic write (write to temp file, then rename) to avoid partial writes.
///
/// # Arguments
/// * `path` - Path to the JSON file
/// * `data` - The value to serialize and write
///
/// # Errors
/// * `Io` - If there's an error writing the file
pub fn write_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let content =
        serde_json::to_string_pretty(data).map_err(|e| WreckitError::InvalidJson(e.to_string()))?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write atomically: write to temp file, then rename
    let temp_path = path.with_extension("json.tmp");
    let mut file = fs::File::create(&temp_path)?;
    file.write_all(content.as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    drop(file);

    fs::rename(&temp_path, path)?;

    Ok(())
}

/// Read the config.json file for a repository.
///
/// # Arguments
/// * `root` - Path to the repository root
///
/// # Returns
/// The parsed Config, or default if file doesn't exist
pub fn read_config(root: &Path) -> Result<Config> {
    let path = get_config_path(root);
    if !path.exists() {
        return Ok(Config::default());
    }
    read_json(&path)
}

/// Read an item.json file from an item directory.
///
/// # Arguments
/// * `root` - Path to the repository root
/// * `id` - Item ID
///
/// # Returns
/// The parsed Item
pub fn read_item(root: &Path, id: &str) -> Result<Item> {
    let path = get_item_json_path(root, id);
    read_json(&path)
}

/// Write an item.json file to an item directory.
///
/// # Arguments
/// * `root` - Path to the repository root
/// * `id` - Item ID
/// * `item` - The item to write
pub fn write_item(root: &Path, id: &str, item: &Item) -> Result<()> {
    let path = get_item_json_path(root, id);
    write_json(&path, item)
}

/// Read a prd.json file from an item directory.
///
/// # Arguments
/// * `root` - Path to the repository root
/// * `id` - Item ID
///
/// # Returns
/// The parsed PRD
pub fn read_prd(root: &Path, id: &str) -> Result<Prd> {
    let path = get_prd_path(root, id);
    read_json(&path)
}

/// Write a prd.json file to an item directory.
///
/// # Arguments
/// * `root` - Path to the repository root
/// * `id` - Item ID
/// * `prd` - The PRD to write
pub fn write_prd(root: &Path, id: &str, prd: &Prd) -> Result<()> {
    let path = get_prd_path(root, id);
    write_json(&path, prd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::WorkflowState;
    use tempfile::TempDir;

    #[test]
    fn test_read_json_file_not_found() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nonexistent.json");

        let result: Result<Item> = read_json(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WreckitError::FileNotFound(_)));
    }

    #[test]
    fn test_read_json_invalid_json() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("invalid.json");
        fs::write(&path, "not valid json {").unwrap();

        let result: Result<Item> = read_json(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WreckitError::InvalidJson(_)));
    }

    #[test]
    fn test_write_and_read_json() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.json");

        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        write_json(&path, &item).unwrap();
        assert!(path.exists());

        let read_item: Item = read_json(&path).unwrap();
        assert_eq!(read_item.id, item.id);
        assert_eq!(read_item.title, item.title);
    }

    #[test]
    fn test_write_json_creates_parent_dirs() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nested").join("dir").join("test.json");

        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        write_json(&path, &item).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_read_config_default_when_missing() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(".wreckit")).unwrap();

        let config = read_config(temp.path()).unwrap();
        assert_eq!(config.base_branch, "main");
        assert_eq!(config.branch_prefix, "wreckit/");
    }

    #[test]
    fn test_read_write_item() {
        let temp = TempDir::new().unwrap();
        let items_dir = temp.path().join(".wreckit").join("items").join("test-001");
        fs::create_dir_all(&items_dir).unwrap();

        let item = Item::new(
            "test-001".to_string(),
            "Test Item".to_string(),
            "Test overview".to_string(),
        );

        write_item(temp.path(), "test-001", &item).unwrap();

        let read = read_item(temp.path(), "test-001").unwrap();
        assert_eq!(read.id, "test-001");
        assert_eq!(read.title, "Test Item");
        assert_eq!(read.state, WorkflowState::Idea);
    }

    #[test]
    fn test_read_write_prd() {
        let temp = TempDir::new().unwrap();
        let items_dir = temp.path().join(".wreckit").join("items").join("test-001");
        fs::create_dir_all(&items_dir).unwrap();

        let prd = Prd::new("test-001".to_string(), "wreckit/test-001".to_string());

        write_prd(temp.path(), "test-001", &prd).unwrap();

        let read = read_prd(temp.path(), "test-001").unwrap();
        assert_eq!(read.id, "test-001");
        assert_eq!(read.branch_name, "wreckit/test-001");
    }
}
