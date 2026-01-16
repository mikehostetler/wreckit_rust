//! Git operations module
//!
//! Provides wrappers for git and gh CLI commands.

mod operations;

pub use operations::{
    branch_exists, check_git_preflight, commit_all, create_or_update_pr, ensure_branch,
    get_current_branch, get_pr_by_branch, has_uncommitted_changes, is_git_repo, is_pr_merged,
    push_branch, run_gh_command, run_git_command, BranchResult, GitOptions, GitPreflightResult,
    PrResult,
};
