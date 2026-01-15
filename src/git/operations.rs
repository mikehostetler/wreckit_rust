//! Git and GitHub CLI operations
//!
//! Wrappers for git and gh commands with proper error handling.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::process::Command;

use crate::errors::{Result, WreckitError};

/// Options for git operations
#[derive(Debug, Clone)]
pub struct GitOptions {
    /// Working directory for git commands
    pub cwd: PathBuf,

    /// If true, log commands without executing
    pub dry_run: bool,
}

/// Result of a branch operation
#[derive(Debug)]
pub struct BranchResult {
    /// Name of the branch
    pub branch_name: String,

    /// Whether the branch was newly created
    pub created: bool,
}

/// Result of a PR operation
#[derive(Debug)]
pub struct PrResult {
    /// PR URL
    pub url: String,

    /// PR number
    pub number: u32,

    /// Whether the PR was newly created
    pub created: bool,
}

/// Result of git preflight checks
#[derive(Debug)]
pub struct GitPreflightResult {
    /// Whether all checks passed
    pub valid: bool,

    /// List of errors found
    pub errors: Vec<String>,
}

/// Execute a git command and return stdout
pub async fn run_git_command(args: &[&str], options: &GitOptions) -> Result<String> {
    if options.dry_run {
        tracing::info!("[DRY RUN] git {}", args.join(" "));
        return Ok(String::new());
    }

    let output = Command::new("git")
        .args(args)
        .current_dir(&options.cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| WreckitError::GitError(format!("Failed to execute git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WreckitError::GitError(format!(
            "git {} failed: {}",
            args.join(" "),
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Execute a gh command and return stdout
pub async fn run_gh_command(args: &[&str], options: &GitOptions) -> Result<String> {
    if options.dry_run {
        tracing::info!("[DRY RUN] gh {}", args.join(" "));
        return Ok(String::new());
    }

    let output = Command::new("gh")
        .args(args)
        .current_dir(&options.cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| WreckitError::GitError(format!("Failed to execute gh: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WreckitError::GitError(format!(
            "gh {} failed: {}",
            args.join(" "),
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Check if a path is inside a git repository
pub async fn is_git_repo(cwd: &Path) -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    matches!(output, Ok(status) if status.success())
}

/// Get the current branch name
pub async fn get_current_branch(options: &GitOptions) -> Result<String> {
    run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"], options).await
}

/// Check if a branch exists locally
pub async fn branch_exists(branch_name: &str, options: &GitOptions) -> bool {
    let result = run_git_command(
        &["rev-parse", "--verify", &format!("refs/heads/{}", branch_name)],
        options,
    )
    .await;
    result.is_ok()
}

/// Check if there are uncommitted changes
pub async fn has_uncommitted_changes(options: &GitOptions) -> bool {
    let result = run_git_command(&["status", "--porcelain"], options).await;
    match result {
        Ok(output) => !output.is_empty(),
        Err(_) => true, // Assume changes if we can't check
    }
}

/// Ensure a branch exists, creating it if necessary
pub async fn ensure_branch(
    base_branch: &str,
    branch_prefix: &str,
    item_slug: &str,
    options: &GitOptions,
) -> Result<BranchResult> {
    let branch_name = format!("{}{}", branch_prefix, item_slug);

    if branch_exists(&branch_name, options).await {
        // Checkout existing branch
        run_git_command(&["checkout", &branch_name], options).await?;
        Ok(BranchResult {
            branch_name,
            created: false,
        })
    } else {
        // Create and checkout new branch from base
        run_git_command(&["checkout", "-b", &branch_name, base_branch], options).await?;
        Ok(BranchResult {
            branch_name,
            created: true,
        })
    }
}

/// Commit all changes with a message
pub async fn commit_all(message: &str, options: &GitOptions) -> Result<()> {
    run_git_command(&["add", "-A"], options).await?;
    run_git_command(&["commit", "-m", message], options).await?;
    Ok(())
}

/// Push branch to origin
pub async fn push_branch(branch_name: &str, options: &GitOptions) -> Result<()> {
    run_git_command(&["push", "-u", "origin", branch_name], options).await?;
    Ok(())
}

/// Get PR info by branch name
pub async fn get_pr_by_branch(branch_name: &str, options: &GitOptions) -> Option<PrResult> {
    let result = run_gh_command(
        &[
            "pr",
            "view",
            branch_name,
            "--json",
            "number,url",
        ],
        options,
    )
    .await;

    match result {
        Ok(json) => {
            // Parse JSON response
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                let number = value["number"].as_u64()? as u32;
                let url = value["url"].as_str()?.to_string();
                Some(PrResult {
                    url,
                    number,
                    created: false,
                })
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Create or update a PR
pub async fn create_or_update_pr(
    base_branch: &str,
    head_branch: &str,
    title: &str,
    body: &str,
    options: &GitOptions,
) -> Result<PrResult> {
    // Check if PR already exists
    if let Some(existing) = get_pr_by_branch(head_branch, options).await {
        return Ok(existing);
    }

    // Create new PR
    let output = run_gh_command(
        &[
            "pr",
            "create",
            "--base",
            base_branch,
            "--head",
            head_branch,
            "--title",
            title,
            "--body",
            body,
        ],
        options,
    )
    .await?;

    // Parse the PR URL from output
    let url = output.trim().to_string();
    let number = url
        .rsplit('/')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(PrResult {
        url,
        number,
        created: true,
    })
}

/// Check if a PR is merged
pub async fn is_pr_merged(pr_number: u32, options: &GitOptions) -> bool {
    let result = run_gh_command(
        &[
            "pr",
            "view",
            &pr_number.to_string(),
            "--json",
            "state",
        ],
        options,
    )
    .await;

    match result {
        Ok(json) => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                value["state"].as_str() == Some("MERGED")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Run preflight checks before git operations
pub async fn check_git_preflight(options: &GitOptions) -> GitPreflightResult {
    let mut errors = Vec::new();

    // Check if in a git repo
    if !is_git_repo(&options.cwd).await {
        errors.push("Not in a git repository".to_string());
        return GitPreflightResult {
            valid: false,
            errors,
        };
    }

    // Check for detached HEAD
    let branch = get_current_branch(options).await;
    if let Ok(ref b) = branch {
        if b == "HEAD" {
            errors.push("HEAD is detached".to_string());
        }
    }

    // Check for uncommitted changes
    if has_uncommitted_changes(options).await {
        errors.push("There are uncommitted changes".to_string());
    }

    GitPreflightResult {
        valid: errors.is_empty(),
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_git_repo() -> TempDir {
        let temp = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(temp.path())
            .output()
            .await
            .unwrap();

        // Configure git
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp.path())
            .output()
            .await
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(temp.path())
            .output()
            .await
            .unwrap();

        // Create initial commit
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();

        Command::new("git")
            .args(["add", "-A"])
            .current_dir(temp.path())
            .output()
            .await
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp.path())
            .output()
            .await
            .unwrap();

        temp
    }

    #[tokio::test]
    async fn test_is_git_repo() {
        let temp = setup_git_repo().await;
        assert!(is_git_repo(temp.path()).await);

        let non_repo = TempDir::new().unwrap();
        assert!(!is_git_repo(non_repo.path()).await);
    }

    #[tokio::test]
    async fn test_get_current_branch() {
        let temp = setup_git_repo().await;
        let options = GitOptions {
            cwd: temp.path().to_path_buf(),
            dry_run: false,
        };

        let branch = get_current_branch(&options).await.unwrap();
        // Could be "main" or "master" depending on git config
        assert!(!branch.is_empty());
    }

    #[tokio::test]
    async fn test_has_uncommitted_changes() {
        let temp = setup_git_repo().await;
        let options = GitOptions {
            cwd: temp.path().to_path_buf(),
            dry_run: false,
        };

        // No uncommitted changes initially
        assert!(!has_uncommitted_changes(&options).await);

        // Create an uncommitted change
        std::fs::write(temp.path().join("new_file.txt"), "content").unwrap();
        assert!(has_uncommitted_changes(&options).await);
    }

    #[tokio::test]
    async fn test_branch_exists() {
        let temp = setup_git_repo().await;
        let options = GitOptions {
            cwd: temp.path().to_path_buf(),
            dry_run: false,
        };

        // Get current branch name
        let current = get_current_branch(&options).await.unwrap();

        // Current branch should exist
        assert!(branch_exists(&current, &options).await);

        // Non-existent branch should not exist
        assert!(!branch_exists("nonexistent-branch", &options).await);
    }

    #[tokio::test]
    async fn test_dry_run_git_command() {
        let temp = TempDir::new().unwrap();
        let options = GitOptions {
            cwd: temp.path().to_path_buf(),
            dry_run: true,
        };

        // Should not fail even if not a git repo
        let result = run_git_command(&["status"], &options).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
