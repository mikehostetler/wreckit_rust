//! CLI module for wreckit
//!
//! Provides the command-line interface using clap.

pub mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Wreckit - A CLI tool for turning ideas into automated PRs through an autonomous agent loop
#[derive(Parser, Debug)]
#[command(name = "wreckit")]
#[command(version)]
#[command(about = "A CLI tool for turning ideas into automated PRs through an autonomous agent loop")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose logging (debug level)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress info-level output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Preview operations without executing them
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Override the working directory
    #[arg(long, global = true)]
    pub cwd: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new wreckit project in the current repository
    Init {
        /// Force initialization even if .wreckit already exists
        #[arg(long)]
        force: bool,
    },

    /// Show status of all items
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List items with optional filtering
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Filter by workflow state (idea, researched, planned, implementing, in_pr, done)
        #[arg(long)]
        state: Option<String>,
    },

    /// Show details of a specific item
    Show {
        /// Item ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run the research phase for an item
    Research {
        /// Item ID
        id: String,

        /// Force re-run even if research.md exists
        #[arg(long)]
        force: bool,
    },

    /// Run the planning phase for an item
    Plan {
        /// Item ID
        id: String,

        /// Force re-run even if plan.md and prd.json exist
        #[arg(long)]
        force: bool,
    },

    /// Run the implementation phase for an item
    Implement {
        /// Item ID
        id: String,

        /// Force re-run implementation
        #[arg(long)]
        force: bool,
    },

    /// Create or update the pull request for an item
    Pr {
        /// Item ID
        id: String,

        /// Force PR creation even if one exists
        #[arg(long)]
        force: bool,
    },

    /// Mark an item as complete (after PR is merged)
    Complete {
        /// Item ID
        id: String,
    },

    /// Run an item through all phases until completion
    Run {
        /// Item ID
        id: String,

        /// Force re-run of all phases
        #[arg(long)]
        force: bool,
    },

    /// Find and run the next incomplete item
    Next,

    /// Validate items and optionally fix issues
    Doctor {
        /// Automatically fix recoverable issues
        #[arg(long)]
        fix: bool,
    },

    /// Ingest ideas from a file or stdin
    Ideas {
        /// Path to file containing ideas (reads from stdin if not provided)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}
