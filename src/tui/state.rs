//! TUI state management

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::schemas::Item;

/// Tool execution tracking
#[derive(Debug, Clone)]
pub struct ToolExecution {
    pub tool_use_id: String,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub status: ToolStatus,
    pub result: Option<serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

/// Tool status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolStatus {
    Running,
    Completed,
    Error,
}

/// Agent activity for a specific item
#[derive(Debug, Clone)]
pub struct AgentActivity {
    pub thoughts: Vec<String>,
    pub tools: Vec<ToolExecution>,
}

impl Default for AgentActivity {
    fn default() -> Self {
        Self {
            thoughts: Vec::new(),
            tools: Vec::new(),
        }
    }
}

/// Item state for TUI display
#[derive(Debug, Clone)]
pub struct ItemState {
    pub id: String,
    pub state: String,
    pub title: String,
    pub current_story_id: Option<String>,
}

impl From<Item> for ItemState {
    fn from(item: Item) -> Self {
        Self {
            id: item.id,
            state: item.state.to_string(),
            title: item.title,
            current_story_id: None,
        }
    }
}

/// Story tracking
#[derive(Debug, Clone)]
pub struct CurrentStory {
    pub id: String,
    pub title: String,
}

/// Main TUI state
#[derive(Debug, Clone)]
pub struct TuiState {
    pub current_item: Option<String>,
    pub current_phase: Option<String>,
    pub current_iteration: u32,
    pub max_iterations: u32,
    pub current_story: Option<CurrentStory>,
    pub items: Vec<ItemState>,
    pub completed_count: usize,
    pub total_count: usize,
    pub start_time: DateTime<Utc>,
    pub logs: Vec<String>,
    pub show_logs: bool,
    pub activity_by_item: HashMap<String, AgentActivity>,
}

impl TuiState {
    const MAX_THOUGHTS: usize = 50;
    const MAX_TOOLS: usize = 20;
    const MAX_LOGS: usize = 500;

    /// Create new TUI state from items
    pub fn new(items: Vec<Item>) -> Self {
        let total_count = items.len();
        let completed_count = items
            .iter()
            .filter(|i| i.state == crate::schemas::WorkflowState::Done)
            .count();

        let item_states: Vec<ItemState> = items.into_iter().map(ItemState::from).collect();
        let activity_by_item: HashMap<String, AgentActivity> = item_states
            .iter()
            .map(|item| (item.id.clone(), AgentActivity::default()))
            .collect();

        Self {
            current_item: None,
            current_phase: None,
            current_iteration: 0,
            max_iterations: 100,
            current_story: None,
            items: item_states,
            completed_count,
            total_count,
            start_time: Utc::now(),
            logs: Vec::new(),
            show_logs: false,
            activity_by_item,
        }
    }

    // ===== IMMUTABLE BUILDER METHODS =====

    /// Return a new TuiState with the current item updated
    pub fn with_current_item(mut self, item: Option<String>) -> Self {
        self.current_item = item;
        self
    }

    /// Return a new TuiState with the current phase updated
    pub fn with_current_phase(mut self, phase: Option<String>) -> Self {
        self.current_phase = phase;
        self
    }

    /// Return a new TuiState with iteration counter updated
    pub fn with_iteration(mut self, iteration: u32) -> Self {
        self.current_iteration = iteration;
        self
    }

    /// Return a new TuiState with the current story updated
    pub fn with_current_story(mut self, story: Option<CurrentStory>) -> Self {
        self.current_story = story;
        self
    }

    /// Return a new TuiState with an item state updated
    pub fn with_item_state(mut self, item_id: String, state: String) -> Self {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == item_id) {
            item.state = state;
        }
        self
    }

    /// Return a new TuiState with completed count updated
    pub fn with_completed_count(mut self, count: usize) -> Self {
        self.completed_count = count;
        self
    }

    /// Return a new TuiState with logs appended
    pub fn with_logs(mut self, mut logs: Vec<String>) -> Self {
        self.logs.append(&mut logs);
        if self.logs.len() > Self::MAX_LOGS {
            let excess = self.logs.len() - Self::MAX_LOGS;
            self.logs.drain(0..excess);
        }
        self
    }

    /// Return a new TuiState with a single log appended
    pub fn with_log(mut self, log: String) -> Self {
        self.logs.push(log);
        if self.logs.len() > Self::MAX_LOGS {
            self.logs.remove(0);
        }
        self
    }

    /// Return a new TuiState with show_logs toggled
    pub fn with_show_logs(mut self, show: bool) -> Self {
        self.show_logs = show;
        self
    }

    /// Return a new TuiState with agent activity updated
    pub fn with_agent_activity(mut self, item_id: String, activity: AgentActivity) -> Self {
        self.activity_by_item.insert(item_id, activity);
        self
    }

    /// Append a thought to an item's activity
    pub fn append_thought(&mut self, item_id: &str, thought: String) {
        if let Some(activity) = self.activity_by_item.get_mut(item_id) {
            // Merge with last thought if short
            if let Some(last) = activity.thoughts.last() {
                if last.len() < 120 {
                    let merged = format!("{} {}", last, thought);
                    activity.thoughts.pop();
                    activity.thoughts.push(merged);
                } else {
                    activity.thoughts.push(thought);
                }
            } else {
                activity.thoughts.push(thought);
            }

            // Limit thoughts
            if activity.thoughts.len() > Self::MAX_THOUGHTS {
                activity.thoughts.remove(0);
            }
        }
    }

    /// Append a tool execution to an item's activity
    pub fn append_tool(&mut self, item_id: &str, tool: ToolExecution) {
        if let Some(activity) = self.activity_by_item.get_mut(item_id) {
            activity.tools.push(tool);
            if activity.tools.len() > Self::MAX_TOOLS {
                activity.tools.remove(0);
            }
        }
    }

    /// Update a tool execution status
    pub fn update_tool_status(
        &mut self,
        item_id: &str,
        tool_use_id: &str,
        status: ToolStatus,
        result: Option<serde_json::Value>,
    ) {
        if let Some(activity) = self.activity_by_item.get_mut(item_id) {
            if let Some(tool) = activity.tools.iter_mut().find(|t| t.tool_use_id == tool_use_id) {
                tool.status = status;
                tool.result = result;
                if status != ToolStatus::Running {
                    tool.finished_at = Some(Utc::now());
                }
            }
        }
    }
}
