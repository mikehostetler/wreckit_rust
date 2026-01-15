//! TUI runner - manages TUI lifecycle and rendering

use crate::errors::Result;
use crate::schemas::Item;
use crate::tui::events::{sanitize_assistant_text, AgentEvent};
use crate::tui::state::{AgentActivity, ToolExecution, ToolStatus, TuiState};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::DisableMouseCapture,
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

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

/// State update events
#[derive(Clone)]
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

/// Main TUI runner
pub struct TuiRunner {
    state: Arc<Mutex<TuiState>>,
    options: TuiOptions,
    state_tx: tokio::sync::broadcast::Sender<TuiUpdate>,
    _state_rx: tokio::sync::broadcast::Receiver<TuiUpdate>,
    scroll_offset: usize,
    auto_scroll: bool,
}

impl TuiRunner {
    /// Create a new TUI runner
    pub async fn new(items: Vec<Item>, options: TuiOptions) -> Self {
        let state = Arc::new(Mutex::new(TuiState::new(items)));
        let (state_tx, mut state_rx) = tokio::sync::broadcast::channel(100);

        // Spawn task to process state updates
        let state_clone = state.clone();
        let mut rx = state_tx.subscribe();
        tokio::spawn(async move {
            while let Ok(update) = rx.recv().await {
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
                    TuiUpdate::SetCurrentStory(_story) => {
                        // TODO: Parse story from string in Phase 4
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
            state_tx,
            _state_rx: state_rx,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    fn handle_agent_event(state: &mut TuiState, item_id: String, event: AgentEvent) {
        match event {
            AgentEvent::AssistantText { text } => {
                if let Some(cleaned) = sanitize_assistant_text(&text) {
                    state.append_thought(&item_id, cleaned);
                }
            }
            AgentEvent::ToolStarted {
                tool_use_id,
                tool_name,
                input,
            } => {
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
    pub fn create_update_sender(&self) -> tokio::sync::broadcast::Sender<TuiUpdate> {
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
        use ratatui::layout::{Constraint, Direction, Layout};

        loop {
            let state = self.get_state().await;

            // Draw
            terminal.draw(|f| {
                let size = f.area();

                // Header (5 lines), Main (flex), Footer (4 lines)
                let header_height = 5;
                let footer_height = 4;

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
                        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                        .split(chunks[1]);

                    crate::tui::widgets::render_items_pane(f, main_chunks[0], &state);

                    let right_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(4), Constraint::Min(0)])
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
                            crossterm::event::KeyCode::Char('j')
                            | crossterm::event::KeyCode::Down => {
                                if state.show_logs && self.scroll_offset > 0 {
                                    self.scroll_offset -= 1;
                                    self.auto_scroll = false;
                                }
                            }
                            crossterm::event::KeyCode::Char('k')
                            | crossterm::event::KeyCode::Up => {
                                if state.show_logs {
                                    self.scroll_offset += 1;
                                    self.auto_scroll = false;
                                }
                            }
                            crossterm::event::KeyCode::PageDown => {
                                if state.show_logs {
                                    let logs_height = 15;
                                    self.scroll_offset =
                                        self.scroll_offset.saturating_sub(logs_height);
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
                                if key.modifiers.contains(
                                    crossterm::event::KeyModifiers::CONTROL,
                                ) =>
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

            // Auto-scroll to bottom when new logs arrive
            if self.auto_scroll {
                self.scroll_offset = 0;
            }
        }
    }
}
