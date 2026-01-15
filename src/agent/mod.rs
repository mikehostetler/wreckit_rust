//! Agent execution module
//!
//! Provides the agent runner for executing Claude CLI or other agents.

mod parser;
mod runner;

pub use parser::parse_agent_line;
pub use runner::{run_agent, AgentResult, RunAgentOptions};
