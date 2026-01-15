//! Terminal User Interface (TUI) module
//!
//! Provides real-time visualization of workflow progress and agent activity.

pub mod state;
pub mod runner;
pub mod widgets;
pub mod events;

// Re-export commonly used types
pub use state::{AgentActivity, TuiState, ToolExecution, ToolStatus};
pub use runner::{TuiRunner, TuiOptions};
pub use events::{AgentEvent, sanitize_assistant_text};
