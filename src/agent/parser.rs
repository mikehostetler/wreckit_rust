//! Parse agent output for structured events

use regex::Regex;
use serde_json::Value;

use crate::tui::events::AgentEvent;

lazy_static::lazy_static! {
    static ref TOOL_USE_REGEX: Regex = Regex::new(
        r"<tool_use>(?P<content>.*?)</tool_use>"
    ).unwrap();

    static ref TOOL_RESULT_REGEX: Regex = Regex::new(
        r"<tool_result>(?P<content>.*?)</tool_result>"
    ).unwrap();

    static ref ASSISTANT_TEXT_REGEX: Regex = Regex::new(
        r"<assistant_text>(?P<content>.*?)</assistant_text>"
    ).unwrap();
}

/// Parse agent output line for events
pub fn parse_agent_line(line: &str) -> Vec<AgentEvent> {
    let mut events = Vec::new();

    // Check for tool_use
    if let Some(caps) = TOOL_USE_REGEX.captures(line) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&caps["content"]) {
            if let Some(tool_use_id) = parsed.get("toolUseId").and_then(|v| v.as_str()) {
                if let Some(tool_name) = parsed.get("name").and_then(|v| v.as_str()) {
                    events.push(AgentEvent::ToolStarted {
                        tool_use_id: tool_use_id.to_string(),
                        tool_name: tool_name.to_string(),
                        input: parsed.get("input").cloned().unwrap_or(Value::Null),
                    });
                }
            }
        }
    }

    // Check for tool_result
    if let Some(caps) = TOOL_RESULT_REGEX.captures(line) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&caps["content"]) {
            if let Some(tool_use_id) = parsed.get("toolUseId").and_then(|v| v.as_str()) {
                events.push(AgentEvent::ToolResult {
                    tool_use_id: tool_use_id.to_string(),
                    result: parsed.get("content").cloned().unwrap_or(Value::Null),
                });
            }
        }
    }

    // Check for assistant text
    if let Some(caps) = ASSISTANT_TEXT_REGEX.captures(line) {
        events.push(AgentEvent::AssistantText {
            text: caps["content"].to_string(),
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_use() {
        let line = r#"<tool_use>{"toolUseId":"123","name":"read_file","input":{"path":"test.rs"}}</tool_use>"#;
        let events = parse_agent_line(line);
        assert_eq!(events.len(), 1);
        match &events[0] {
            AgentEvent::ToolStarted { tool_use_id, tool_name, .. } => {
                assert_eq!(tool_use_id, "123");
                assert_eq!(tool_name, "read_file");
            }
            _ => panic!("Expected ToolStarted event"),
        }
    }

    #[test]
    fn test_parse_tool_result() {
        let line = r#"<tool_result>{"toolUseId":"123","content":"file content"}</tool_result>"#;
        let events = parse_agent_line(line);
        assert_eq!(events.len(), 1);
        match &events[0] {
            AgentEvent::ToolResult { tool_use_id, .. } => {
                assert_eq!(tool_use_id, "123");
            }
            _ => panic!("Expected ToolResult event"),
        }
    }

    #[test]
    fn test_parse_assistant_text() {
        let line = r#"<assistant_text>Thinking about implementation</assistant_text>"#;
        let events = parse_agent_line(line);
        assert_eq!(events.len(), 1);
        match &events[0] {
            AgentEvent::AssistantText { text } => {
                assert_eq!(text, "Thinking about implementation");
            }
            _ => panic!("Expected AssistantText event"),
        }
    }

    #[test]
    fn test_parse_empty_line() {
        let events = parse_agent_line("");
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_parse_invalid_json() {
        let line = r#"<tool_use>invalid json</tool_use>"#;
        let events = parse_agent_line(line);
        assert_eq!(events.len(), 0);
    }
}
