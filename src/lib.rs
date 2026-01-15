//! Wreckit - A CLI tool for turning ideas into automated PRs through an autonomous agent loop
//!
//! This library provides the core functionality for the wreckit CLI, including:
//! - Schema definitions for items, configs, PRDs, and stories
//! - Domain logic for workflow states and transitions
//! - File system utilities for reading/writing JSON
//! - Git operations for branch management and PR creation
//! - Agent execution for running the Claude CLI
//! - Workflow phases (research, plan, implement, pr, complete)

pub mod agent;
pub mod cli;
pub mod config;
pub mod domain;
pub mod errors;
pub mod fs;
pub mod git;
pub mod prompts;
pub mod schemas;
pub mod workflow;

// Re-export commonly used types
pub use errors::{Result, WreckitError};
pub use schemas::{Config, Item, Prd, Story, WorkflowState};
