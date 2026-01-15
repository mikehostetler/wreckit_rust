//! Agent execution module
//!
//! Provides the agent runner for executing Claude CLI or other agents.

mod runner;

pub use runner::{run_agent, AgentResult, RunAgentOptions};
