//! Run command - Run an item through all phases until completion

use crate::errors::Result;
use std::path::Path;

/// Run an item through all phases until completion
pub async fn run(_cwd: Option<&Path>, _id: &str, _force: bool, _dry_run: bool) -> Result<()> {
    todo!("Implement run command")
}
