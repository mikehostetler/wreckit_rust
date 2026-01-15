//! Status command - Show status of all items

use crate::errors::Result;
use std::path::Path;

/// Show status of all items
pub async fn run(_cwd: Option<&Path>, _json: bool) -> Result<()> {
    todo!("Implement status command")
}
