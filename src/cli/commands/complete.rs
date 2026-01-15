//! Complete command - Mark an item as complete after PR is merged

use crate::errors::Result;
use std::path::Path;

/// Mark an item as complete (after PR is merged)
pub async fn run(_cwd: Option<&Path>, _id: &str, _dry_run: bool) -> Result<()> {
    todo!("Implement complete command")
}
