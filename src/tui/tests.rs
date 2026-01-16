//! Comprehensive unit tests for TUI state management

use crate::schemas::{Item, WorkflowState};
use crate::tui::state::{AgentActivity, ToolExecution, ToolStatus, TuiState};
use crate::tui::events::AgentEvent;
use chrono;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_item(id: &str, state: WorkflowState, title: &str) -> Item {
        let now = chrono::Utc::now().to_rfc3339();
        Item {
            schema_version: 1,
            id: id.to_string(),
            title: title.to_string(),
            section: None,
            state,
            overview: String::new(),
            branch: None,
            pr_url: None,
            pr_number: None,
            last_error: None,
            created_at: now.clone(),
            updated_at: now,
            problem_statement: None,
            motivation: None,
            success_criteria: None,
            technical_constraints: None,
            scope_in_scope: None,
            scope_out_of_scope: None,
            priority_hint: None,
            urgency_hint: None,
        }
    }

    #[test]
    fn test_tui_state_creation() {
        let items = vec![
            create_test_item("item1", WorkflowState::Idea, "First Item"),
            create_test_item("item2", WorkflowState::Done, "Second Item"),
        ];

        let state = TuiState::new(items.clone());

        assert_eq!(state.items.len(), 2);
        assert_eq!(state.total_count, 2);
        assert_eq!(state.completed_count, 1); // Only item2 is Done
        assert_eq!(state.current_item, None);
        assert_eq!(state.current_phase, None);
        assert!(state.start_time <= Utc::now());
        assert!(state.logs.is_empty());
        assert!(!state.show_logs);
    }

    #[test]
    fn test_tui_state_with_current_item() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        let updated = state.clone().with_current_item(Some("item1".to_string()));

        assert_eq!(state.current_item, None); // Original unchanged
        assert_eq!(updated.current_item, Some("item1".to_string()));
    }

    #[test]
    fn test_tui_state_with_current_phase() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        let updated = state.clone().with_current_phase(Some("research".to_string()));

        assert_eq!(state.current_phase, None); // Original unchanged
        assert_eq!(updated.current_phase, Some("research".to_string()));
    }

    #[test]
    fn test_tui_state_with_iteration() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        let updated = state.clone().with_iteration(5);

        assert_eq!(state.current_iteration, 0); // Original unchanged
        assert_eq!(updated.current_iteration, 5);
    }

    #[test]
    fn test_tui_state_with_completed_count() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        let updated = state.clone().with_completed_count(10);

        assert_eq!(state.completed_count, 0); // Original unchanged
        assert_eq!(updated.completed_count, 10);
    }

    #[test]
    fn test_tui_state_with_single_log() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        let updated = state.clone().with_log("First log line".to_string());

        assert!(state.logs.is_empty()); // Original unchanged
        assert_eq!(updated.logs.len(), 1);
        assert_eq!(updated.logs[0], "First log line");
    }

    #[test]
    fn test_tui_state_with_logs_enforces_max_limit() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        // Add more logs than MAX_LOGS
        let logs: Vec<String> = (0..600).map(|i| format!("Log line {}", i)).collect();
        let updated = state.clone().with_logs(logs);

        // Should be limited to MAX_LOGS
        assert_eq!(updated.logs.len(), TuiState::MAX_LOGS);
        // Should keep the most recent logs (100-599)
        assert!(updated.logs[0].contains("100"));
        assert!(updated.logs[499].contains("599"));
    }

    #[test]
    fn test_tui_state_with_show_logs() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        let updated = state.clone().with_show_logs(true);

        assert!(!state.show_logs); // Original unchanged
        assert!(updated.show_logs);
    }

    #[test]
    fn test_append_thought_adds_to_activity() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let mut state = TuiState::new(items);

        state.append_thought("item1", "First thought".to_string());

        assert_eq!(state.activity_by_item.get("item1").unwrap().thoughts.len(), 1);
        assert_eq!(state.activity_by_item.get("item1").unwrap().thoughts[0], "First thought");
    }

    #[test]
    fn test_append_thought_merges_short_thoughts() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let mut state = TuiState::new(items);

        state.append_thought("item1", "Short thought".to_string());
        state.append_thought("item1", "more text".to_string());

        let thoughts = &state.activity_by_item.get("item1").unwrap().thoughts;
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0], "Short thought more text");
    }

    #[test]
    fn test_append_thought_enforces_max_limit() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let mut state = TuiState::new(items);

        // Add more thoughts than MAX_THOUGHTS
        // Use longer thoughts to avoid merging (merge happens when last thought < 120 chars)
        for i in 0..100 {
            let thought = format!("This is a longer thought number {} with enough text to exceed the merge threshold of 120 characters easily", i);
            state.append_thought("item1", thought);
        }

        let thoughts = &state.activity_by_item.get("item1").unwrap().thoughts;
        // Should be limited to MAX_THOUGHTS
        assert_eq!(thoughts.len(), TuiState::MAX_THOUGHTS);
        // Verify the buffer is actually limiting by checking we don't have all 100 thoughts
        assert!(thoughts.len() < 100, "Should have fewer thoughts than added due to buffer limit");
    }

    #[test]
    fn test_append_tool_adds_to_activity() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let mut state = TuiState::new(items);

        let tool = ToolExecution {
            tool_use_id: "tool123".to_string(),
            tool_name: "test_tool".to_string(),
            input: serde_json::json!({"arg": "value"}),
            status: ToolStatus::Running,
            result: None,
            started_at: Utc::now(),
            finished_at: None,
        };

        state.append_tool("item1", tool);

        let tools = &state.activity_by_item.get("item1").unwrap().tools;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].tool_name, "test_tool");
    }

    #[test]
    fn test_append_tool_enforces_max_limit() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let mut state = TuiState::new(items);

        // Add more tools than MAX_TOOLS
        for i in 0..30 {
            let tool = ToolExecution {
                tool_use_id: format!("tool{}", i),
                tool_name: format!("tool_{}", i),
                input: serde_json::json!({}),
                status: ToolStatus::Running,
                result: None,
                started_at: Utc::now(),
                finished_at: None,
            };
            state.append_tool("item1", tool);
        }

        let tools = &state.activity_by_item.get("item1").unwrap().tools;
        assert_eq!(tools.len(), TuiState::MAX_TOOLS);
    }

    #[test]
    fn test_update_tool_status() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let mut state = TuiState::new(items);

        // Add a tool
        let tool = ToolExecution {
            tool_use_id: "tool123".to_string(),
            tool_name: "test_tool".to_string(),
            input: serde_json::json!({}),
            status: ToolStatus::Running,
            result: None,
            started_at: Utc::now(),
            finished_at: None,
        };
        state.append_tool("item1", tool);

        // Update the tool status
        let result = serde_json::json!({"output": "success"});
        state.update_tool_status("item1", "tool123", ToolStatus::Completed, Some(result.clone()));

        let tools = &state.activity_by_item.get("item1").unwrap().tools;
        assert_eq!(tools[0].status, ToolStatus::Completed);
        assert_eq!(tools[0].result, Some(result));
        assert!(tools[0].finished_at.is_some());
    }

    #[test]
    fn test_with_item_state() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items.clone());

        let updated = state.clone().with_item_state("item1".to_string(), "done".to_string());

        assert_eq!(state.items[0].state, "idea"); // Original unchanged
        assert_eq!(updated.items[0].state, "done");
    }

    #[test]
    fn test_multiple_immutable_updates_chain() {
        let items = vec![create_test_item("item1", WorkflowState::Idea, "First Item")];
        let state = TuiState::new(items);

        let updated = state
            .clone()
            .with_current_item(Some("item1".to_string()))
            .with_current_phase(Some("implementing".to_string()))
            .with_iteration(3)
            .with_show_logs(true);

        assert_eq!(state.current_item, None);
        assert_eq!(state.current_phase, None);
        assert_eq!(state.current_iteration, 0);
        assert!(!state.show_logs);

        assert_eq!(updated.current_item, Some("item1".to_string()));
        assert_eq!(updated.current_phase, Some("implementing".to_string()));
        assert_eq!(updated.current_iteration, 3);
        assert!(updated.show_logs);
    }
}
