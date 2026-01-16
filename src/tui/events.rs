//! Agent event types for TUI updates

use serde::{Deserialize, Serialize};

/// Events from agent execution that update the TUI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentEvent {
    /// Assistant text (thought)
    AssistantText { text: String },
    /// Tool execution started
    ToolStarted {
        tool_use_id: String,
        tool_name: String,
        input: serde_json::Value,
    },
    /// Tool execution result
    ToolResult {
        tool_use_id: String,
        result: serde_json::Value,
    },
    /// Tool execution error
    ToolError {
        tool_use_id: String,
        error: String,
    },
    /// General error
    Error { message: String },
    /// Run completed
    RunResult,
}

/// Sanitize assistant text (remove code blocks, tool calls)
pub fn sanitize_assistant_text(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    // Remove code blocks
    let re = regex::Regex::new(r"```[\s\S]*?```").unwrap();
    let cleaned = re.replace_all(text, "").to_string();

    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        return None;
    }

    // Collapse whitespace
    let cleaned: String = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    // Remove tool calls (lines starting with "tool:")
    if cleaned.starts_with("tool:") {
        return None;
    }

    Some(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_assistant_text_removes_code_blocks() {
        let text = "Thinking about stuff\n```\ncode here\n```\nMore thoughts";
        let result = sanitize_assistant_text(text);
        assert_eq!(result, Some("Thinking about stuff More thoughts".to_string()));
    }

    #[test]
    fn test_sanitize_assistant_text_removes_tool_calls() {
        let text = "tool: some_tool_call";
        let result = sanitize_assistant_text(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_sanitize_assistant_text_collapses_whitespace() {
        let text = "Thinking    about\n\n\n    stuff";
        let result = sanitize_assistant_text(text);
        assert_eq!(result, Some("Thinking about stuff".to_string()));
    }

    #[test]
    fn test_sanitize_assistant_text_empty_returns_none() {
        let text = "   \n\n   ";
        let result = sanitize_assistant_text(text);
        assert_eq!(result, None);
    }

    #[test]
    fn test_sanitize_assistant_text_normal_text() {
        let text = "This is normal text about implementation";
        let result = sanitize_assistant_text(text);
        assert_eq!(result, Some("This is normal text about implementation".to_string()));
    }
}
