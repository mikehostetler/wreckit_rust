# Research: Build TUI (Terminal User Interface)

**Date**: 2026-01-15
**Item**: 003-build-tui-terminal-user-interface

## Research Question
Users need to interact with the tool primarily through the command line, requiring a proper TUI.

**Motivation:** The primary way to interact with this tool will be on the command line, so a good TUI is essential for usability.

**Success criteria:**
- Primary interaction happens through command line TUI

## Summary

The TypeScript version of Wreckit (`/Users/mhostetler/Source/MikeHostetler/wreckit`) uses **Ink** (React for CLI) to build a sophisticated TUI with real-time updates, multi-pane layouts, and interactive keyboard controls. The Rust port needs equivalent functionality using Rust-native TUI libraries.

The TypeScript TUI provides:
- **Real-time dashboard** showing item progress, workflow phases, and agent activity
- **Multi-pane layout** with item list, active item details, agent output, and activity tracking
- **Keyboard controls** for navigation (quit, toggle views, scroll logs)
- **Live updates** as agents run, showing thoughts, tool executions, and results
- **Responsive design** that adapts to terminal dimensions

For Rust, the recommended approach is to use **ratatui** (formerly tui-rs), the most mature and actively maintained TUI library for Rust. It provides widget primitives, layout management, and event handling similar to Ink's capabilities.

## Current State Analysis

### Existing Implementation

**TypeScript version** (`/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/`):

The TypeScript implementation uses **Ink** (React for CLI) with these key components:

1. **Core Architecture**:
   - `TuiRunner` class (`runner.ts:31-234`) - Manages TUI lifecycle, state updates, and event subscriptions
   - `InkApp` component (`InkApp.tsx:12-156`) - Main React component handling keyboard input and layout
   - `TuiState` interface (`dashboard.ts:18-36`) - Immutable state structure with items, logs, activity tracking

2. **State Management**:
   ```typescript
   interface TuiState {
     currentItem: string | null;
     currentPhase: string | null;
     currentIteration: number;
     maxIterations: number;
     items: Array<{ id, state, title, currentStoryId }>;
     completedCount: number;
     totalCount: number;
     startTime: Date;
     logs: string[];
     showLogs: boolean;
     activityByItem: Record<string, AgentActivityForItem>;
   }
   ```
   - Uses functional updates: `updateTuiState(state, update)` (`dashboard.ts:62-67`)
   - Subscriber pattern for reactive updates (`runner.ts:44-54`)

3. **Layout Components**:
   - `Header.tsx` - Shows current item, phase, iteration, story
   - `ItemsPane.tsx` - Lists all items with state icons (✓ → ○)
   - `ActiveItemPane.tsx` - Details of currently running item
   - `AgentActivityPane.tsx` - Shows agent thoughts and tool executions
   - `LogsPane.tsx` - Scrollable log output
   - `Footer.tsx` - Progress, runtime, keyboard shortcuts

4. **Keyboard Controls**:
   - `q` or `Ctrl+C` - Quit
   - `l` - Toggle between items view and logs view
   - `j/k` or `↑/↓` - Scroll logs
   - `PageUp/PageDown` - Scroll by page
   - `g/G` - Jump to top/bottom of logs

5. **Agent Event Processing**:
   - `appendAgentEvent()` (`runner.ts:140-211`) processes agent events:
     - `assistant_text` - Agent thoughts (sanitized, max 50)
     - `tool_started` - Tool execution tracking (max 20)
     - `tool_result` - Tool completion with results
     - `tool_error` - Tool errors
     - `error` - General errors

**Rust port current state** (`/Users/mhostetler/Source/wreckit_rust/`):

The Rust port has:
- Basic CLI structure using `clap` (`src/cli/mod.rs:1-147`)
- Command implementations (mostly `todo!` placeholders) (`src/cli/commands/`)
- Schema definitions for `Item`, `Config`, `WorkflowState` (`src/schemas/`)
- Agent runner with process spawning (`src/agent/runner.rs:74-191`)
- Domain logic for state machine (`src/domain/states.rs`)
- Immutable data patterns already established (`src/schemas/item.rs:169-199`)

**No TUI implementation exists yet** - all commands are currently `todo!` stubs.

### Key Files

**TypeScript reference implementation:**
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/runner.ts:31-234` - TuiRunner class with state management
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/InkApp.tsx:12-156` - Main Ink app component
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/dashboard.ts:18-177` - State structure and rendering
- `/Users/mhostetler/Source/MikeHostetler/wreckit/src/tui/components/` - UI components

**Rust port files to integrate with:**
- `/Users/mhostetler/Source/wreckit_rust/src/schemas/item.rs:64-139` - Item schema (immutable builders)
- `/Users/mhostetler/Source/wreckit_rust/src/domain/states.rs:1-122` - Workflow state machine
- `/Users/mhostetler/Source/wreckit_rust/src/agent/runner.rs:74-191` - Agent execution
- `/Users/mhostetler/Source/wreckit_rust/src/cli/mod.rs:1-147` - CLI command definitions
- `/Users/mhostetler/Source/wreckit_rust/src/main.rs:30-79` - Main command dispatcher

## Technical Considerations

### Dependencies

**Required external crates:**
1. **ratatui** (v0.30.0) - Core TUI library
   - Most mature Rust TUI framework
   - Provides widgets: Paragraph, List, Table, Block, etc.
   - Layout system with Flex and Rect
   - Terminal management and event handling

2. **crossterm** (dependency of ratatui) - Terminal control
   - Cross-platform terminal handling
   - Keyboard event capture
   - Terminal size detection

3. **tokio** (already in dependencies) - Async runtime
   - TUI updates will be async
   - Channel communication for state updates

4. **Existing dependencies to leverage:**
   - `serde` / `serde_json` - State serialization
   - `chrono` - Timestamp formatting
   - `thiserror` - Error handling
   - `tracing` - Logging integration

**Internal modules to integrate with:**
- `crate::schemas::{Item, Config, WorkflowState}` - Data models
- `crate::domain::states` - State machine logic
- `crate::agent::runner` - Agent execution events
- `crate::cli::commands` - Command implementations that will use TUI

### Patterns to Follow

**From TypeScript implementation:**

1. **Immutable state updates** (already matches Rust pattern):
   ```typescript
   export function updateTuiState(state: TuiState, update: Partial<TuiState>): TuiState {
     return { ...state, ...update };
   }
   ```
   Rust equivalent already exists in `Item::with_*()` methods (`src/schemas/item.rs:172-199`)

2. **Subscriber pattern for reactive updates** (`runner.ts:44-54`):
   ```typescript
   subscribe(cb: (state: TuiState) => void): () => void {
     this.subscribers.add(cb);
     cb(this.state);
     return () => this.subscribers.delete(cb);
   }
   ```
   Rust implementation should use `tokio::sync::broadcast` channels

3. **Agent event processing** (`runner.ts:140-211`):
   - Sanitize assistant text (remove code blocks, tool calls)
   - Track tool executions with status
   - Limit buffer sizes (max 50 thoughts, 20 tools)

4. **Responsive layout** (`InkApp.tsx:102-104`):
   ```typescript
   const leftPaneWidth = Math.floor(width * 0.4);
   const rightPaneWidth = width - leftPaneWidth - 3;
   ```
   Ratatui provides `Layout::default()` with flex sizing

**Rust-specific patterns:**

1. **Use existing error types** (`src/errors.rs:1-130`):
   - Add `WreckitError::TuiError` variant if needed
   - Use `Result<T>` for fallible TUI operations

2. **Follow existing module structure** (`src/lib.rs:1-25`):
   - Create new `pub mod tui;` module
   - Re-export TUI types if needed

3. **Maintain immutability** (established pattern in `src/schemas/item.rs:169-199`):
   - State updates return new instances
   - Use builder methods for state transitions

4. **Async integration**:
   - TUI runs on main thread with terminal control
   - Agent execution runs in background tasks
   - Communication via channels (mpsc/broadcast)

## Recommended Approach

### High-Level Strategy

1. **Phase 1: Core TUI Infrastructure**
   - Add `ratatui` and `crossterm` to `Cargo.toml`
   - Create `src/tui/` module with state management
   - Implement basic terminal setup/teardown
   - Create channel-based state updates (async→sync bridge)

2. **Phase 2: State Management**
   - Port `TuiState` structure to Rust with proper types
   - Implement immutable update functions (matching existing patterns)
   - Create agent event processor (translate TypeScript event types)
   - Add subscriber notification system using `tokio::sync::broadcast`

3. **Phase 3: Layout & Rendering**
   - Implement main layout with Header, ItemsPane, ActivityPane, LogsPane, Footer
   - Add keyboard event handling (quit, toggle, scroll)
   - Implement responsive sizing based on terminal dimensions
   - Add color scheme (match cyan/green/yellow from TypeScript)

4. **Phase 4: Integration with Agent Runner**
   - Modify `agent::runner::run_agent()` to accept TUI update callbacks
   - Parse agent output for events (assistant_text, tool_started, etc.)
   - Stream events to TUI state via channels

5. **Phase 5: Command Integration**
   - Replace `todo!()` in commands with TUI calls
   - Start with `wreckit run` command as primary TUI entry point
   - Add `--json` flag support for non-TUI output (scripting)

### Implementation Notes

**Ratatui Architecture Pattern:**
```rust
// Main TUI loop
fn run_tui(mut terminal: Terminal<...>) -> Result<()> {
    loop {
        // Draw
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(4)])
                .split(f.size());
            render_header(f, chunks[0], &state);
            render_main_panes(f, chunks[1], &state);
            render_footer(f, chunks[2], &state);
        })?;

        // Handle events (with timeout)
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => handle_key(key),
                Event::Resize(_, _) => {}, // Force redraw
                _ => {}
            }
        }

        // Check for state updates from channel
        if let Ok(update) = state_rx.try_recv() {
            state = apply_update(state, update);
        }
    }
}
```

**State Management Pattern:**
```rust
// src/tui/state.rs
pub struct TuiState {
    pub current_item: Option<String>,
    pub current_phase: Option<String>,
    pub items: Vec<ItemState>,
    pub logs: Vec<String>,
    pub activity_by_item: HashMap<String, AgentActivity>,
    // ...
}

impl TuiState {
    pub fn with_current_item(mut self, item: Option<String>) -> Self {
        self.current_item = item;
        self
    }

    pub fn append_log(mut self, log: String) -> Self {
        self.logs.push(log);
        if self.logs.len() > MAX_LOGS {
            self.logs.remove(0);
        }
        self
    }
}
```

**Channel Communication:**
```rust
// Agent runner sends events
pub enum TuiEvent {
    ItemStarted(String),
    PhaseChanged(String),
    LogLine(String),
    AgentEvent(String, AgentEventType),
}

// In agent::runner::run_agent()
if let Some(ref on_tui_event) = options.on_tui_event {
    on_tui_event(TuiEvent::LogLine(log_chunk));
}
```

### Terminal UI Components

**Header** (5 lines):
- Title: "┌─ Wreckit ─┐"
- Line 1: Current item or "Waiting..."
- Line 2: Phase and iteration count
- Line 3: Story ID and title
- Separator line

**Main Panes** (flexible height):
- **Left Pane** (40% width): Item list
  - State icons (✓ → ○)
  - Item ID, state, story info
  - Highlight active item in yellow
- **Right Pane** (60% width):
  - Top 4 lines: Active item details
  - Remaining: Agent activity (thoughts + tools)

**Logs View** (toggle with 'l'):
- Scrollable log output
- Last 15 lines visible
- Auto-scroll to bottom by default

**Footer** (4 lines):
- Progress: "3/10 complete"
- Runtime: "00:45:23"
- Keyboard shortcuts: "[q] quit [l] logs"

### Color Scheme

From TypeScript (`Header.tsx:26`):
- Cyan for borders and UI chrome
- Green for completed items
- Yellow for active items
- Dim color for secondary text

Ratatui equivalents:
- `Color::Cyan`
- `Color::Green`
- `Color::Yellow`
- `Style::default().dim()`

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Ratatui learning curve** | Medium | Start with simple prototype (single pane), add complexity incrementally. Reference existing ratatui examples (tui-rs.github.io/ratatui-book/) |
| **Async/sync boundary complexity** | High | Use `tokio::sync::mpsc` channels for all TUI updates. Keep TUI loop synchronous with channel polling. Run agent execution in separate tasks. |
| **Terminal compatibility** | Medium | Ratatui + crossterm handle cross-platform differences. Test on macOS, Linux, Windows CI. Handle resize events gracefully. |
| **Performance with large logs** | Medium | Limit log buffer (500 lines max as in TypeScript). Implement circular buffer. Lazy rendering (only visible lines). |
| **Agent output parsing** | Medium | Reuse TypeScript sanitization logic (`sanitizeAssistantText()` in `runner.ts:10-22`). Parse completion signals robustly. |
| **State synchronization** | Low | Use immutable state updates (already established pattern). Single source of truth with broadcast channels. |

## Open Questions

1. **Should TUI be default or opt-in?**
   - TypeScript version: TUI is default for `wreckit run`
   - Consider: Always use TUI for interactive, add `--no-tui` flag for CI/scripting
   - **Recommendation**: TUI by default, `--json` flag for machine-readable output

2. **How to handle agent output streaming?**
   - Options: Parse stdout/stderr in real-time, buffer and process, or use structured logging
   - TypeScript uses callback pattern (`on_stdout`, `on_stderr` in `runner.ts:52-56`)
   - **Recommendation**: Extend `RunAgentOptions` with `on_tui_event: mpsc::Sender<TuiEvent>` callback

3. **What about progress indicators for long operations?**
   - Research phase can take minutes
   - **Recommendation**: Show spinner or pulse animation in phase line, update iteration counter

4. **Should we support multiple TUI modes?**
   - Simple progress bar (like `createSimpleProgress()` in `runner.ts:220-233`)
   - Full dashboard
   - **Recommendation**: Start with full dashboard, add simple mode later if needed for CI/CD

5. **How to handle terminal resize during rendering?**
   - Ratatui sends `Event::Resize(width, height)`
   - **Recommendation**: Recalculate layout on resize, clear and redraw

## Next Steps

1. **Create proof-of-concept TUI**
   - Minimal ratatui app with static layout
   - Keyboard handling (quit only)
   - Terminal setup/teardown

2. **Add state management**
   - Port `TuiState` struct
   - Implement update functions
   - Add channel communication

3. **Integrate with existing commands**
   - Start with `wreckit status` as simple TUI test
   - Then `wreckit run` for full agent workflow

4. **Implement agent event parsing**
   - Parse agent stdout for events
   - Update TUI state in real-time

5. **Add polish**
   - Colors and styling
   - Smooth scrolling
   - Error handling and graceful degradation
