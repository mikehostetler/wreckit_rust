# Build TUI (Terminal User Interface) Implementation Plan

## Overview

This plan implements a Terminal User Interface (TUI) for the Wreckit Rust port, providing real-time visualization of agent workflow progress, item states, and activity tracking. The TUI will be the primary interactive interface for `wreckit run` and other commands, matching the functionality of the TypeScript version's Ink-based UI.

The implementation uses **ratatui** (v0.30+), the most mature Rust TUI framework, which provides widgets, layout management, and cross-platform terminal handling via crossterm.

## Current State Analysis

### What Exists Now

**TypeScript Reference Implementation** (`/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/`):
- Full-featured TUI using Ink (React for CLI)
- State management with immutable updates (`TuiState` in `dashboard.ts:18-36`)
- Multi-pane layout: Header, ItemsPane, ActiveItemPane, AgentActivityPane, LogsPane, Footer
- Keyboard controls: quit (q/Ctrl+C), toggle logs (l), scroll (j/k, arrows, page up/down, g/G)
- Agent event processing: assistant_text, tool_started, tool_result, tool_error, error
- Real-time updates via subscriber pattern (`runner.ts:44-54`)
- Buffer limits: 50 thoughts, 20 tools, 500 logs

**Rust Port Current State** (`/Users/mhostetler/Source/wreckit_rust/`):
- ✅ CLI structure using clap (`src/cli/mod.rs:1-147`)
- ✅ Item schema with immutable builder methods (`src/schemas/item.rs:169-199`)
- ✅ Workflow state machine (`src/domain/states.rs:1-122`)
- ✅ Agent runner with process spawning (`src/agent/runner.rs:74-191`)
- ✅ Error handling framework (`src/errors.rs`)
- ❌ **No TUI implementation** - all commands are `todo!()` stubs
- ❌ **No TUI dependencies** in `Cargo.toml`

### Key Constraints & Patterns

1. **Follow existing immutable pattern**: Use builder methods (`with_*()`) for state updates
2. **Leverage existing schemas**: `Item`, `Config`, `WorkflowState` are already defined
3. **Use tokio for async**: TUI must work with async agent execution
4. **Channel-based communication**: Use `tokio::sync::mpsc/broadcast` for TUI updates
5. **Cross-platform**: Ratatui + crossterm handle macOS/Linux/Windows

### Key Discoveries

From analyzing the TypeScript implementation:

1. **State Structure** (`dashboard.ts:18-36`):
   ```typescript
   interface TuiState {
     currentItem: string | null;
     currentPhase: string | null;
     currentIteration: number;
     maxIterations: number;
     currentStory: { id: string; title: string } | null;
     items: Array<{ id, state, title, currentStoryId }>;
     completedCount: number;
     totalCount: number;
     startTime: Date;
     logs: string[];
     showLogs: boolean;
     activityByItem: Record<string, AgentActivityForItem>;
   }
   ```

2. **Agent Event Types** (`runner.ts:140-211`):
   - `assistant_text`: Agent thoughts (sanitized, max 50)
   - `tool_started`: Tool execution (max 20)
   - `tool_result`: Tool completion
   - `tool_error`: Tool errors
   - `error`: General errors

3. **Layout Dimensions** (`InkApp.tsx:102-104`):
   - Left pane: 40% width (items list)
   - Right pane: 60% width (activity + details)
   - Header: 5 lines
   - Footer: 4 lines

4. **Color Scheme**:
   - Cyan for borders and chrome
   - Green for completed items (✓)
   - Yellow for active items (→)
   - Dim for secondary text

5. **Agent Event Sanitization** (`runner.ts:10-22`):
   - Remove code blocks (`````)
   - Remove tool calls
   - Trim and collapse whitespace
   - Max 120 chars per thought

## Desired End State

### Functional Specification

The TUI should provide:

1. **Real-time Dashboard View**:
   - Header showing current item, phase, iteration, story
   - Left pane: Scrollable list of all items with state icons
   - Right pane top: Active item details
   - Right pane bottom: Agent thoughts and tool executions
   - Footer: Progress counter, runtime, keyboard shortcuts

2. **Logs View** (toggle with 'l'):
   - Full-screen scrollable log output
   - Last 15 lines visible by default
   - Auto-scroll to bottom when new logs arrive

3. **Keyboard Controls**:
   - `q` or `Ctrl+C`: Quit TUI
   - `l`: Toggle between dashboard and logs view
   - `j/k` or `↑/↓`: Scroll logs by line
   - `PageUp/PageDown`: Scroll by page
   - `g/G`: Jump to top/bottom of logs

4. **State Icons**:
   - `✓` for done
   - `→` for implementing/in_pr
   - `○` for idea/researched/planned

5. **Responsive Layout**:
   - Recalculate on terminal resize
   - Graceful handling of small terminals (min 80x24)

### Verification

1. **Automated**:
   - Unit tests for state management (immutable updates)
   - Integration tests for keyboard handling
   - Property tests for state transitions

2. **Manual**:
   - Run `wreckit run <id>` and verify TUI appears
   - Test keyboard controls
   - Verify agent activity updates in real-time
   - Test terminal resize handling
   - Verify colors match TypeScript version

## What We're NOT Doing

- ❌ **NOT** implementing web UI (out of scope)
- ❌ **NOT** supporting multiple TUI modes (only full dashboard)
- ❌ **NOT** implementing progress bar mode (deferred)
- ❌ **NOT** adding mouse support (keyboard-only, like TypeScript)
- ❌ **NOT** customizing color schemes (hardcoded cyan/green/yellow)
- ❌ **NOT** persisting TUI state (ephemeral only)
- ❌ **NOT** supporting non-TTY output (use `--json` flag instead)

## Implementation Approach

### High-Level Strategy

The implementation follows a **incremental, phased approach**:

1. **Phase 1**: Core infrastructure (ratatui setup, terminal management)
2. **Phase 2**: State management (port TuiState, immutable updates, channels)
3. **Phase 3**: Layout and rendering (all panes, colors, responsive sizing)
4. **Phase 4**: Agent integration (parse agent output, stream events to TUI)
5. **Phase 5**: Command integration (replace `todo!()` in commands)

Each phase is independently testable and builds on the previous one. The approach minimizes risk by:
- Starting with isolated components (no agent dependency yet)
- Using mock data for rendering tests
- Adding async integration last (most complex)
- Ensuring each phase has working deliverables

---

## Phase 1: Core TUI Infrastructure

### Overview

Set up the foundational TUI infrastructure: add ratatui dependency, create module structure, implement terminal setup/teardown, and create a basic "hello world" TUI to verify everything works.

### Changes Required

#### 1. Add Dependencies to Cargo.toml

**File**: `/Users/mhostetler/Source/wreckit_rust/Cargo.toml`

**Changes**: Add ratatui and crossterm dependencies

```toml
[dependencies]
# ... existing dependencies ...

# TUI framework
ratatui = "0.30"
crossterm = "0.28"

# ... dev-dependencies ...
```

**Why**: Ratatui is the core TUI library. Crossterm handles cross-platform terminal control (keyboard events, terminal size).

---

#### 2. Create TUI Module

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/mod.rs` (new)

**Changes**: Create the TUI module with basic structure

```rust
//! Terminal User Interface (TUI) module
//!
//! Provides real-time visualization of workflow progress and agent activity.

pub mod state;
pub mod runner;
pub mod widgets;

// Re-export commonly used types
pub use state::TuiState;
pub use runner::{TuiRunner, TuiOptions};
```

---

#### 3. Create Basic TUI Runner

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/runner.rs` (new)

**Changes**: Implement basic TUI lifecycle (setup, teardown, event loop)

```rust
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

use crate::errors::{Result, WreckitError};

/// Main TUI runner
pub struct TuiRunner {
    // TODO: Add state management in Phase 2
}

impl TuiRunner {
    /// Create a new TUI runner
    pub fn new() -> Self {
        Self {}
    }

    /// Run the TUI (blocking call)
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run TUI loop
        let result = self.run_tui_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_tui_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            // Draw
            terminal.draw(|f| {
                // TODO: Replace with actual rendering in Phase 3
                let size = f.size();
                let paragraph = ratatui::widgets::Paragraph::new("Wreckit TUI - Phase 1")
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(paragraph, size);
            })?;

            // Handle events (with timeout)
            if crossterm::event::poll(Duration::from_millis(100))? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(key) => {
                        // Handle 'q' or Ctrl+C to quit
                        if key.code == crossterm::event::KeyCode::Char('q')
                            || (key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                && key.code == crossterm::event::KeyCode::Char('c'))
                        {
                            return Ok(());
                        }
                    }
                    crossterm::event::Event::Resize(_, _) => {
                        // Force redraw on resize
                    }
                    _ => {}
                }
            }
        }
    }
}
```

---

#### 4. Create Stub State Module

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/state.rs` (new)

**Changes**: Create placeholder state module (full implementation in Phase 2)

```rust
//! TUI state management

/// TUI state (placeholder - will be expanded in Phase 2)
#[derive(Debug, Clone)]
pub struct TuiState {
    // TODO: Add state fields in Phase 2
}

impl TuiState {
    pub fn new() -> Self {
        Self {}
    }
}
```

---

#### 5. Create Stub Widgets Module

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/widgets.rs` (new)

**Changes**: Create placeholder widgets module (full implementation in Phase 3)

```rust
//! TUI widget rendering

// TODO: Implement widgets in Phase 3
```

---

#### 6. Add TUI Module to Lib

**File**: `/Users/mhostetler/Source/wreckit_rust/src/lib.rs`

**Changes**: Add TUI module to library exports

```rust
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
pub mod tui; // NEW

// Re-export commonly used types
pub use errors::{Result, WreckitError};
pub use schemas::{Config, Item, Prd, Story, WorkflowState};
```

---

### Success Criteria

#### Automated Verification:
- [ ] Tests pass: `cargo test`
- [ ] Build succeeds: `cargo build`
- [ ] No clippy warnings: `cargo clippy`

#### Manual Verification:
- [ ] Run `cargo run` and verify TUI starts
- [ ] Verify "Wreckit TUI - Phase 1" appears centered
- [ ] Press `q` and verify TUI exits cleanly
- [ ] Press `Ctrl+C` and verify TUI exits cleanly
- [ ] Verify terminal is restored to normal state after exit
- [ ] Test on terminal sizes: 80x24, 120x40, 200x60

**Note**: Complete all automated verification, then pause for manual confirmation before proceeding to Phase 2.

---

## Phase 2: State Management

### Overview

Port the TypeScript `TuiState` structure to Rust, implement immutable update functions (following existing patterns), create agent event types, and add channel-based communication for async state updates.

### Changes Required

#### 1. Implement TuiState Structure

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/state.rs`

**Changes**: Replace stub with full TuiState implementation

```rust
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

#[derive(Debug, Clone, PartialEq)]
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
        let completed_count = items.iter().filter(|i| i.state == crate::schemas::WorkflowState::Done).count();

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
            self.logs = self.logs.split_off(self.logs.len() - Self::MAX_LOGS);
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
                    activity.thoughts.push(format!("{} {}", last, thought));
                    activity.thoughts.remove(activity.thoughts.len() - 2);
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
    pub fn update_tool_status(&mut self, item_id: &str, tool_use_id: &str, status: ToolStatus, result: Option<serde_json::Value>) {
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
```

---

#### 2. Create Agent Event Types

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/events.rs` (new)

**Changes**: Define agent event types for streaming

```rust
//! Agent event types for TUI updates

use serde::{Deserialize, Serialize};

/// Events from agent execution that update the TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let cleaned = regex::Regex::new(r"```[\s\S]*?```")
        .unwrap()
        .replace_all(text, "")
        .to_string();

    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        return None;
    }

    // Collapse whitespace
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    // Remove tool calls
    if regex::Regex::new(r"^tool\s*:", regex::Regex::new("").unwrap().UNICODE_CASE)
        .unwrap()
        .is_match(&cleaned)
    {
        return None;
    }

    Some(cleaned)
}
```

---

#### 3. Update TuiRunner with State Management

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/runner.rs`

**Changes**: Add state management and channel communication

```rust
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::errors::{Result, WreckitError};
use crate::schemas::Item;
use crate::tui::events::AgentEvent;
use crate::tui::state::TuiState;

/// Options for TUI initialization
#[derive(Clone)]
pub struct TuiOptions {
    pub on_quit: Option<Arc<dyn Fn() + Send + Sync>>,
    pub debug: bool,
}

impl Default for TuiOptions {
    fn default() -> Self {
        Self {
            on_quit: None,
            debug: false,
        }
    }
}

/// Main TUI runner
pub struct TuiRunner {
    state: Arc<Mutex<TuiState>>,
    options: TuiOptions,
    state_rx: tokio::sync::mpsc::Receiver<TuiUpdate>,
}

/// State update events
pub enum TuiUpdate {
    SetCurrentItem(Option<String>),
    SetCurrentPhase(Option<String>),
    SetIteration(u32),
    SetCurrentStory(Option<String>),
    SetItemState(String, String),
    SetCompletedCount(usize),
    AppendLogs(Vec<String>),
    ToggleLogs(bool),
    AgentEvent(String, AgentEvent),
}

impl TuiRunner {
    /// Create a new TUI runner
    pub async fn new(items: Vec<Item>, options: TuiOptions) -> Self {
        let state = Arc::new(Mutex::new(TuiState::new(items)));
        let (state_tx, state_rx) = tokio::sync::mpsc::channel(100);

        // Spawn task to process state updates
        let state_clone = state.clone();
        tokio::spawn(async move {
            while let Some(update) = state_tx.recv().await {
                let mut state = state_clone.lock().await;
                match update {
                    TuiUpdate::SetCurrentItem(item) => {
                        *state = state.clone().with_current_item(item);
                    }
                    TuiUpdate::SetCurrentPhase(phase) => {
                        *state = state.clone().with_current_phase(phase);
                    }
                    TuiUpdate::SetIteration(iter) => {
                        *state = state.clone().with_iteration(iter);
                    }
                    TuiUpdate::SetCurrentStory(story) => {
                        // TODO: Parse story from string
                    }
                    TuiUpdate::SetItemState(item_id, item_state) => {
                        *state = state.clone().with_item_state(item_id, item_state);
                    }
                    TuiUpdate::SetCompletedCount(count) => {
                        *state = state.clone().with_completed_count(count);
                    }
                    TuiUpdate::AppendLogs(logs) => {
                        *state = state.clone().with_logs(logs);
                    }
                    TuiUpdate::ToggleLogs(show) => {
                        *state = state.clone().with_show_logs(show);
                    }
                    TuiUpdate::AgentEvent(item_id, event) => {
                        Self::handle_agent_event(&mut state, item_id, event);
                    }
                }
            }
        });

        Self {
            state,
            options,
            state_rx,
        }
    }

    fn handle_agent_event(state: &mut TuiState, item_id: String, event: AgentEvent) {
        use crate::tui::events;
        use crate::tui::state::{ToolExecution, ToolStatus};

        match event {
            AgentEvent::AssistantText { text } => {
                if let Some(cleaned) = events::sanitize_assistant_text(&text) {
                    state.append_thought(&item_id, cleaned);
                }
            }
            AgentEvent::ToolStarted { tool_use_id, tool_name, input } => {
                let tool = ToolExecution {
                    tool_use_id,
                    tool_name,
                    input,
                    status: ToolStatus::Running,
                    result: None,
                    started_at: chrono::Utc::now(),
                    finished_at: None,
                };
                state.append_tool(&item_id, tool);
            }
            AgentEvent::ToolResult { tool_use_id, result } => {
                state.update_tool_status(&item_id, &tool_use_id, ToolStatus::Completed, Some(result));
            }
            AgentEvent::ToolError { tool_use_id, error } => {
                state.update_tool_status(&item_id, &tool_use_id, ToolStatus::Error, None);
                state.append_thought(&item_id, format!("[ERROR] {}", error));
            }
            AgentEvent::Error { message } => {
                state.append_thought(&item_id, format!("[ERROR] {}", message));
            }
            AgentEvent::RunResult => {
                // No state update needed
            }
        }
    }

    /// Get current state (for rendering)
    pub async fn get_state(&self) -> TuiState {
        self.state.lock().await.clone()
    }

    /// Create a sender for state updates
    pub fn create_update_sender(&self) -> tokio::sync::mpsc::Sender<TuiUpdate> {
        self.state_tx.clone()
    }

    /// Run the TUI (blocking call)
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run TUI loop
        let result = self.run_tui_loop(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        // Call quit callback
        if let Some(ref on_quit) = self.options.on_quit {
            on_quit();
        }

        result
    }

    async fn run_tui_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            // Draw
            let state = self.get_state().await;
            terminal.draw(|f| {
                // TODO: Replace with actual rendering in Phase 3
                let size = f.size();
                let text = format!(
                    "Wreckit TUI - Phase 2\nItems: {}\nCurrent: {:?}",
                    state.items.len(),
                    state.current_item
                );
                let paragraph = ratatui::widgets::Paragraph::new(text)
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(paragraph, size);
            })?;

            // Handle events (with timeout)
            if crossterm::event::poll(Duration::from_millis(100))? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(key) => {
                        if key.code == crossterm::event::KeyCode::Char('q')
                            || (key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                                && key.code == crossterm::event::KeyCode::Char('c'))
                        {
                            return Ok(());
                        }
                    }
                    crossterm::event::Event::Resize(_, _) => {
                        // Force redraw
                    }
                    _ => {}
                }
            }

            // Check for state updates
            if let Ok(_update) = self.state_rx.try_recv() {
                // State is already updated by the background task
            }
        }
    }
}
```

---

### Success Criteria

#### Automated Verification:
- [ ] Tests pass: `cargo test`
- [ ] Build succeeds: `cargo build`
- [ ] No clippy warnings: `cargo clippy`

#### Manual Verification:
- [ ] Create a test that creates TuiState from items
- [ ] Verify immutable builder methods work correctly
- [ ] Test channel communication with mock updates
- [ ] Verify agent event parsing works
- [ ] Check that buffer limits are enforced (max 50 thoughts, 20 tools, 500 logs)

**Note**: Complete all automated verification, then pause for manual confirmation before proceeding to Phase 3.

---

## Phase 3: Layout and Rendering

### Overview

Implement all TUI widgets (Header, ItemsPane, ActiveItemPane, AgentActivityPane, LogsPane, Footer), keyboard controls, responsive layout, and color scheme to match the TypeScript implementation.

### Changes Required

#### 1. Create Widgets Module

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/widgets.rs`

**Changes**: Implement all TUI widgets

```rust
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::state::{AgentActivity, TuiState};

/// Render the header section (5 lines)
pub fn render_header(f: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Title line
    let title = Line::from(vec![
        Span::styled("┌─ Wreckit ", Style::default().fg(Color::Cyan)),
        Span::styled("─".repeat(area.width as usize - 12), Style::default().fg(Color::Cyan)),
        Span::styled("┐", Style::default().fg(Color::Cyan)),
    ]);
    let title_paragraph = Paragraph::new(Text::from(title))
        .alignment(Alignment::Left);
    f.render_widget(title_paragraph, chunks[0]);

    // Current item line
    let current_item_text = state.current_item
        .as_ref()
        .map(|id| format!("Running: {}", id))
        .unwrap_or_else(|| "Waiting...".to_string());
    let item_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(pad_to_width(&current_item_text, area.width as usize - 4), Style::default()),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let item_paragraph = Paragraph::new(Text::from(item_line));
    f.render_widget(item_paragraph, chunks[1]);

    // Phase line
    let phase_text = state.current_phase
        .as_ref()
        .map(|phase| format!("Phase: {} (iteration {}/{})", phase, state.current_iteration, state.max_iterations))
        .unwrap_or_else(|| "Phase: idle".to_string());
    let phase_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(pad_to_width(&phase_text, area.width as usize - 4), Style::default()),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let phase_paragraph = Paragraph::new(Text::from(phase_line));
    f.render_widget(phase_paragraph, chunks[2]);

    // Story line
    let story_text = state.current_story
        .as_ref()
        .map(|story| format!("Story: {} - {}", story.id, story.title))
        .unwrap_or_else(|| "Story: none".to_string());
    let story_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(pad_to_width(&story_text, area.width as usize - 4), Style::default()),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let story_paragraph = Paragraph::new(Text::from(story_line));
    f.render_widget(story_paragraph, chunks[3]);

    // Separator line
    let separator = Line::from(vec![
        Span::styled("├", Style::default().fg(Color::Cyan)),
        Span::styled("─".repeat(area.width as usize - 2), Style::default().fg(Color::Cyan)),
        Span::styled("┤", Style::default().fg(Color::Cyan)),
    ]);
    let separator_paragraph = Paragraph::new(Text::from(separator));
    f.render_widget(separator_paragraph, chunks[4]);
}

/// Render the items pane (left side)
pub fn render_items_pane(f: &mut Frame, area: Rect, state: &TuiState) {
    let items: Vec<ListItem> = state.items.iter().map(|item| {
        let icon = get_state_icon(&item.state);
        let color = get_state_color(&item.state);

        let story_info = item.current_story_id
            .as_ref()
            .map(|id| format!(" [{}]", id))
            .unwrap_or_default();

        let text = format!("{} {:<30} {:<14}{}", icon, item.id, item.state, story_info);

        ListItem::new(Line::from(vec![
            Span::styled(text, Style::default().fg(color)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .wrap(Wrap { trim: false });

    f.render_widget(list, area);
}

/// Render the active item pane (top right)
pub fn render_active_item_pane(f: &mut Frame, area: Rect, state: &TuiState) {
    let text = if let Some(ref item_id) = state.current_item {
        if let Some(item) = state.items.iter().find(|i| &i.id == item_id) {
            format!("Current Item: {}\nState: {}\n\n{}", item.id, item.state, item.title)
        } else {
            "Item not found".to_string()
        }
    } else {
        "No active item".to_string()
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Active Item"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render the agent activity pane (bottom right)
pub fn render_agent_activity_pane(f: &mut Frame, area: Rect, state: &TuiState) {
    let text = if let Some(ref item_id) = state.current_item {
        if let Some(activity) = state.activity_by_item.get(item_id) {
            let mut lines = Vec::new();

            // Add thoughts
            for thought in &activity.thoughts {
                lines.push(format!("• {}", thought));
            }

            // Add tools
            for tool in &activity.tools {
                let status_symbol = match tool.status {
                    crate::tui::state::ToolStatus::Running => "▶",
                    crate::tui::state::ToolStatus::Completed => "✓",
                    crate::tui::state::ToolStatus::Error => "✗",
                };
                lines.push(format!("{} {}", status_symbol, tool.tool_name));
            }

            if lines.is_empty() {
                "No activity yet".to_string()
            } else {
                lines.join("\n")
            }
        } else {
            "No activity yet".to_string()
        }
    } else {
        "No active item".to_string()
    };

    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Agent Activity"))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render the logs pane (full width when toggled)
pub fn render_logs_pane(f: &mut Frame, area: Rect, state: &TuiState, scroll_offset: usize) {
    let max_log_lines = area.height as usize;

    let logs: Vec<ListItem> = if state.logs.is_empty() {
        vec![ListItem::new("(no output yet)")]

    } else {
        let start = if scroll_offset + max_log_lines > state.logs.len() {
            0.max(state.logs.len() - max_log_lines)
        } else {
            scroll_offset
        };

        let end = (start + max_log_lines).min(state.logs.len());

        state.logs[start..end]
            .iter()
            .map(|log| ListItem::new(log.as_str()))
            .collect()
    };

    let list = List::new(logs)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Agent Output"));

    f.render_widget(list, area);
}

/// Render the footer section (4 lines)
pub fn render_footer(f: &mut Frame, area: Rect, state: &TuiState, show_logs: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // Separator line
    let separator = Line::from(vec![
        Span::styled("├", Style::default().fg(Color::Cyan)),
        Span::styled("─".repeat(area.width as usize - 2), Style::default().fg(Color::Cyan)),
        Span::styled("┤", Style::default().fg(Color::Cyan)),
    ]);
    let separator_paragraph = Paragraph::new(Text::from(separator));
    f.render_widget(separator_paragraph, chunks[0]);

    // Progress line
    let progress_text = format!("Progress: {}/{} complete | Runtime: {}",
        state.completed_count,
        state.total_count,
        format_runtime(state.start_time)
    );
    let progress_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(pad_to_width(&progress_text, area.width as usize - 4), Style::default()),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let progress_paragraph = Paragraph::new(Text::from(progress_line));
    f.render_widget(progress_paragraph, chunks[1]);

    // Empty line
    let empty_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(pad_to_width("", area.width as usize - 4), Style::default()),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let empty_paragraph = Paragraph::new(Text::from(empty_line));
    f.render_widget(empty_paragraph, chunks[2]);

    // Keyboard shortcuts line
    let logs_label = if show_logs { "items" } else { "logs" };
    let keys_text = format!("[q] quit  [l] {}", logs_label);
    let keys_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(pad_to_width(&keys_text, area.width as usize - 4), Style::default()),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let keys_paragraph = Paragraph::new(Text::from(keys_line));
    f.render_widget(keys_paragraph, chunks[3]);
}

// ===== HELPER FUNCTIONS =====

/// Get state icon
fn get_state_icon(state: &str) -> &'static str {
    match state {
        "done" => "✓",
        "implementing" | "in_pr" => "→",
        _ => "○",
    }
}

/// Get state color
fn get_state_color(state: &str) -> Color {
    match state {
        "done" => Color::Green,
        "implementing" | "in_pr" => Color::Yellow,
        _ => Color::White,
    }
}

/// Pad string to width (truncate with ellipsis if too long)
fn pad_to_width(text: &str, width: usize) -> String {
    if text.len() > width {
        format!("{}…", &text[..width - 1])
    } else {
        format!("{:<width$}", text, width = width)
    }
}

/// Format runtime duration
fn format_runtime(start_time: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(start_time);

    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
```

---

#### 2. Update TuiRunner with Full Rendering

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/runner.rs`

**Changes**: Add keyboard controls and full layout rendering

```rust
// Add to TuiRunner struct:
pub struct TuiRunner {
    state: Arc<Mutex<TuiState>>,
    options: TuiOptions,
    state_rx: tokio::sync::mpsc::Receiver<TuiUpdate>,
    scroll_offset: usize,
    auto_scroll: bool,
}

// Update the run_tui_loop method:
async fn run_tui_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    use ratatui::layout::{Constraint, Direction, Layout};

    loop {
        let state = self.get_state().await;

        // Draw
        terminal.draw(|f| {
            let size = f.size();

            // Header (5 lines)
            let header_height = 5;
            let footer_height = 4;
            let main_height = size.height.saturating_sub(header_height + footer_height);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(header_height),
                    Constraint::Min(0),
                    Constraint::Length(footer_height),
                ])
                .split(size);

            // Render header
            crate::tui::widgets::render_header(f, chunks[0], &state);

            // Render main area
            if state.show_logs {
                crate::tui::widgets::render_logs_pane(f, chunks[1], &state, self.scroll_offset);
            } else {
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(40),
                        Constraint::Percentage(60),
                    ])
                    .split(chunks[1]);

                crate::tui::widgets::render_items_pane(f, main_chunks[0], &state);

                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(4),
                        Constraint::Min(0),
                    ])
                    .split(main_chunks[1]);

                crate::tui::widgets::render_active_item_pane(f, right_chunks[0], &state);
                crate::tui::widgets::render_agent_activity_pane(f, right_chunks[1], &state);
            }

            // Render footer
            crate::tui::widgets::render_footer(f, chunks[2], &state, state.show_logs);
        })?;

        // Handle events (with timeout)
        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') => {
                            return Ok(());
                        }
                        crossterm::event::KeyCode::Char('l') => {
                            let mut s = self.state.lock().await;
                            *s = s.clone().with_show_logs(!s.show_logs);
                        }
                        crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                            if state.show_logs {
                                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                                self.auto_scroll = false;
                            }
                        }
                        crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                            if state.show_logs {
                                self.scroll_offset += 1;
                                self.auto_scroll = false;
                            }
                        }
                        crossterm::event::KeyCode::PageDown => {
                            if state.show_logs {
                                let logs_height = 15;
                                self.scroll_offset = self.scroll_offset.saturating_sub(logs_height);
                                self.auto_scroll = false;
                            }
                        }
                        crossterm::event::KeyCode::PageUp => {
                            if state.show_logs {
                                let logs_height = 15;
                                self.scroll_offset += logs_height;
                                self.auto_scroll = false;
                            }
                        }
                        crossterm::event::KeyCode::Char('g') => {
                            if state.show_logs {
                                self.scroll_offset = state.logs.len();
                                self.auto_scroll = false;
                            }
                        }
                        crossterm::event::KeyCode::Char('G') => {
                            if state.show_logs {
                                self.scroll_offset = 0;
                                self.auto_scroll = true;
                            }
                        }
                        crossterm::event::KeyCode::Char('c')
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                crossterm::event::Event::Resize(_, _) => {
                    // Force redraw
                }
                _ => {}
            }
        }

        // Check for state updates
        if let Ok(_update) = self.state_rx.try_recv() {
            if self.auto_scroll {
                self.scroll_offset = 0;
            }
        }
    }
}
```

---

### Success Criteria

#### Automated Verification:
- [ ] Tests pass: `cargo test`
- [ ] Build succeeds: `cargo build`
- [ ] No clippy warnings: `cargo clippy`

#### Manual Verification:
- [ ] Run TUI with sample data and verify all panes render
- [ ] Test keyboard controls (q, l, j/k, page up/down, g/G)
- [ ] Verify colors match TypeScript version (cyan borders, green for done, yellow for active)
- [ ] Test terminal resize and verify layout adapts
- [ ] Toggle between dashboard and logs view
- [ ] Verify text truncation with ellipsis for long content
- [ ] Test with 0 items, 1 item, many items
- [ ] Verify runtime timer updates every second

**Note**: Complete all automated verification, then pause for manual confirmation before proceeding to Phase 4.

---

## Phase 4: Agent Integration

### Overview

Integrate the TUI with the agent runner, parse agent stdout for events, stream events to the TUI via channels, and update the TUI state in real-time as the agent runs.

### Changes Required

#### 1. Update RunAgentOptions

**File**: `/Users/mhostetler/Source/wreckit_rust/src/agent/runner.rs`

**Changes**: Add TUI update callback

```rust
use crate::tui::events::AgentEvent;

/// Options for running an agent
pub struct RunAgentOptions {
    pub config: AgentConfig,
    pub cwd: PathBuf,
    pub prompt: String,
    pub dry_run: bool,
    pub timeout_seconds: u32,
    pub on_stdout: Option<Box<dyn Fn(&str) + Send>>,
    pub on_stderr: Option<Box<dyn Fn(&str) + Send>>,
    pub on_tui_event: Option<Box<dyn Fn(AgentEvent) + Send>>,  // NEW
}
```

---

#### 2. Add Agent Event Parser

**File**: `/Users/mhostetler/Source/wreckit_rust/src/agent/parser.rs` (new)

**Changes**: Parse agent stdout for structured events

```rust
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
```

---

#### 3. Update Agent Runner with Event Streaming

**File**: `/Users/mhostetler/Source/wreckit_rust/src/agent/runner.rs`

**Changes**: Stream agent events to TUI

```rust
use crate::agent::parser;

// In the run_agent function, after reading stdout:
while let Ok(Some(line)) = reader.next_line().await {
    stdout_output.push_str(&line);
    stdout_output.push('\n');

    // Parse line for events
    if let Some(ref on_tui_event) = options.on_tui_event {
        for event in parser::parse_agent_line(&line) {
            on_tui_event(event);
        }
    }
}
```

---

#### 4. Create TUI Helper for Agent Execution

**File**: `/Users/mhostetler/Source/wreckit_rust/src/tui/agent_helper.rs` (new)

**Changes**: Create helper to run agents with TUI updates

```rust
//! Helper for running agents with TUI updates

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent::runner::{run_agent, RunAgentOptions};
use crate::errors::Result;
use crate::tui::events::AgentEvent;
use crate::tui::runner::TuiUpdate;

/// Run an agent with TUI updates
pub async fn run_agent_with_tui(
    options: RunAgentOptions,
    item_id: String,
    tui_tx: tokio::sync::mpsc::Sender<TuiUpdate>,
) -> Result<crate::agent::runner::AgentResult> {
    let tui_tx_clone = tui_tx.clone();

    let options_with_callback = RunAgentOptions {
        on_tui_event: Some(Box::new(move |event| {
            let _ = tui_tx_clone.blocking_send(TuiUpdate::AgentEvent(item_id.clone(), event));
        })),
        ..options
    };

    run_agent(options_with_callback).await
}
```

---

### Success Criteria

#### Automated Verification:
- [ ] Tests pass: `cargo test`
- [ ] Build succeeds: `cargo build`
- [ ] No clippy warnings: `cargo clippy`

#### Manual Verification:
- [ ] Run agent with mock output and verify events are parsed
- [ ] Test with real agent (if available) and verify TUI updates
- [ ] Verify tool executions appear in AgentActivityPane
- [ ] Verify thoughts appear after sanitization
- [ ] Check that buffer limits are enforced

**Note**: Complete all automated verification, then pause for manual confirmation before proceeding to Phase 5.

---

## Phase 5: Command Integration

### Overview

Replace `todo!()` stubs in CLI commands with TUI calls, starting with `wreckit run` as the primary TUI entry point, and add `--no-tui` flag for non-interactive use.

### Changes Required

#### 1. Update Run Command

**File**: `/Users/mhostetler/Source/wreckit_rust/src/cli/commands/run.rs`

**Changes**: Replace `todo!()` with TUI implementation

```rust
use crate::cli::context::CommandContext;
use crate::tui::runner::{TuiRunner, TuiOptions};

/// Run an item through all phases
pub async fn execute(ctx: CommandContext, id: String, force: bool) -> crate::errors::Result<()> {
    // Load item
    let item = ctx.fs.load_item(&id)?;

    // Check if TUI should be used
    let use_tui = !ctx.args.args.contains(&"--no-tui".to_string());

    if use_tui && atty::is(atty::Stream::Stdout) {
        // Run with TUI
        let items = vec![item];
        let options = TuiOptions {
            on_quit: None,
            debug: false,
        };

        let mut runner = TuiRunner::new(items, options).await?;
        runner.run().await?;

    } else {
        // Run without TUI (JSON output)
        // TODO: Implement non-TUI workflow execution
        println!("Running workflow for item: {}", id);
    }

    Ok(())
}
```

---

#### 2. Add --no-tui Flag

**File**: `/Users/mhostetler/Source/wreckit_rust/src/cli/mod.rs`

**Changes**: Add global `--no-tui` flag

```rust
#[derive(Parser, Debug)]
#[command(name = "wreckit")]
#[command(version)]
#[command(about = "A CLI tool for turning ideas into automated PRs through an autonomous agent loop")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[arg(long, global = true)]
    pub dry_run: bool,

    #[arg(long, global = true)]  // NEW
    pub no_tui: bool,

    #[arg(long, global = true)]
    pub cwd: Option<PathBuf>,
}
```

---

#### 3. Add atty Dependency

**File**: `/Users/mhostetler/Source/wreckit_rust/Cargo.toml`

**Changes**: Add atty for terminal detection

```toml
[dependencies]
# ... existing dependencies ...

atty = "0.2"
```

---

### Success Criteria

#### Automated Verification:
- [ ] Tests pass: `cargo test`
- [ ] Build succeeds: `cargo build`
- [ ] No clippy warnings: `cargo clippy`

#### Manual Verification:
- [ ] Run `wreckit run <id>` and verify TUI starts
- [ ] Run `wreckit run --no-tui <id>` and verify no TUI
- [ ] Test with non-TTY (pipe to file) and verify no TUI
- [ ] Verify workflow execution works with TUI
- [ ] Verify all keyboard controls work in the full workflow

**Note**: Complete all automated verification, then pause for final review.

---

## Testing Strategy

### Unit Tests

**State Management** (`src/tui/state.rs`):
- Test TuiState creation from items
- Test immutable builder methods return new instances
- Test buffer limits (max 50 thoughts, 20 tools, 500 logs)
- Test append_thought merges short thoughts
- Test append_tool enforces max tools
- Test update_tool_status finds and updates tools

**Event Parsing** (`src/agent/parser.rs`):
- Test parsing of tool_use events
- Test parsing of tool_result events
- Test parsing of assistant_text events
- Test malformed input doesn't crash
- Test multiple events in one line

**Widgets** (`src/tui/widgets.rs`):
- Test get_state_icon returns correct symbols
- Test get_state_color returns correct colors
- Test pad_to_width truncates with ellipsis
- Test format_runtime calculates duration correctly

### Integration Tests

**TUI Lifecycle**:
- Test TUI starts and exits cleanly
- Test terminal is restored after exit
- Test keyboard events are handled
- Test resize events don't crash

**Channel Communication**:
- Test state updates via channels
- Test concurrent updates are serialized
- Test channel backpressure handling

**Agent Integration**:
- Test agent stdout is parsed for events
- Test events update TUI state
- Test buffer limits are enforced during streaming

### Manual Testing Steps

1. **Basic TUI**:
   - Run `cargo run`
   - Verify header, panes, footer render
   - Press `q` to exit

2. **Keyboard Controls**:
   - Press `l` to toggle logs view
   - Press `j/k` to scroll logs
   - Press `PageUp/PageDown` to scroll by page
   - Press `g/G` to jump to top/bottom
   - Press `Ctrl+C` to quit

3. **Responsive Layout**:
   - Resize terminal window
   - Verify layout adapts to new dimensions
   - Test with 80x24 (minimum)

4. **Agent Execution**:
   - Run `wreckit run <id>` on a real item
   - Verify agent activity appears in real-time
   - Verify tool executions are tracked
   - Verify thoughts are sanitized

5. **Terminal Restoration**:
   - Run TUI and exit
   - Verify terminal prompt is visible
   - Verify terminal settings are restored

## Migration Notes

No data migration required - TUI is purely ephemeral state.

## References

- Research: `/Users/mhostetler/Source/wreckit_rust/.wreckit/items/003-build-tui-terminal-user-interface/research.md`
- TypeScript TUI: `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/`
  - `runner.ts:31-234` - TuiRunner class
  - `InkApp.tsx:12-156` - Main Ink app component
  - `dashboard.ts:18-177` - State structure and rendering
- Rust schemas: `/Users/mhostetler/Source/wreckit_rust/src/schemas/item.rs:64-139`
- Agent runner: `/Users/mhostetler/Source/wreckit_rust/src/agent/runner.rs:74-191`
- CLI structure: `/Users/mhostetler/Source/wreckit_rust/src/cli/mod.rs:1-147`
