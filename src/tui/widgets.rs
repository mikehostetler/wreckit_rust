//! TUI widget rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::state::{AgentActivity, ToolStatus, TuiState};

/// Render the header section (5 lines)
pub fn render_header(f: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Title line
    let border_width = area.width as usize;
    let title = Line::from(vec![
        Span::styled("┌─ Wreckit ", Style::default().fg(Color::Cyan)),
        Span::styled(
            "─".repeat(border_width.saturating_sub(12)),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("┐", Style::default().fg(Color::Cyan)),
    ]);
    let title_paragraph = Paragraph::new(Text::from(title)).alignment(Alignment::Left);
    f.render_widget(title_paragraph, chunks[0]);

    // Current item line
    let current_item_text = state
        .current_item
        .as_ref()
        .map(|id| format!("Running: {}", id))
        .unwrap_or_else(|| "Waiting...".to_string());
    let item_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            pad_to_width(&current_item_text, border_width.saturating_sub(4)),
            Style::default(),
        ),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let item_paragraph = Paragraph::new(Text::from(item_line));
    f.render_widget(item_paragraph, chunks[1]);

    // Phase line
    let phase_text = state.current_phase.as_ref().map(|phase| {
        format!(
            "Phase: {} (iteration {}/{})",
            phase, state.current_iteration, state.max_iterations
        )
    }).unwrap_or_else(|| "Phase: idle".to_string());
    let phase_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            pad_to_width(&phase_text, border_width.saturating_sub(4)),
            Style::default(),
        ),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let phase_paragraph = Paragraph::new(Text::from(phase_line));
    f.render_widget(phase_paragraph, chunks[2]);

    // Story line
    let story_text = state.current_story.as_ref().map(|story| {
        format!("Story: {} - {}", story.id, story.title)
    }).unwrap_or_else(|| "Story: none".to_string());
    let story_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            pad_to_width(&story_text, border_width.saturating_sub(4)),
            Style::default(),
        ),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let story_paragraph = Paragraph::new(Text::from(story_line));
    f.render_widget(story_paragraph, chunks[3]);

    // Separator line
    let separator = Line::from(vec![
        Span::styled("├", Style::default().fg(Color::Cyan)),
        Span::styled(
            "─".repeat(border_width.saturating_sub(2)),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("┤", Style::default().fg(Color::Cyan)),
    ]);
    let separator_paragraph = Paragraph::new(Text::from(separator));
    f.render_widget(separator_paragraph, chunks[4]);
}

/// Render the items pane (left side)
pub fn render_items_pane(f: &mut Frame, area: Rect, state: &TuiState) {
    let items: Vec<ListItem> = state
        .items
        .iter()
        .map(|item| {
            let icon = get_state_icon(&item.state);
            let color = get_state_color(&item.state);

            let story_info = item
                .current_story_id
                .as_ref()
                .map(|id| format!(" [{}]", id))
                .unwrap_or_default();

            let text = format!(
                "{} {:<30} {:<14}{}",
                icon, item.id, item.state, story_info
            );

            ListItem::new(Line::from(vec![Span::styled(text, Style::default().fg(color))]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);
}

/// Render the active item pane (top right)
pub fn render_active_item_pane(f: &mut Frame, area: Rect, state: &TuiState) {
    let text = if let Some(ref item_id) = state.current_item {
        if let Some(item) = state.items.iter().find(|i| &i.id == item_id) {
            format!(
                "Current Item: {}\nState: {}\n\n{}",
                item.id, item.state, item.title
            )
        } else {
            "Item not found".to_string()
        }
    } else {
        "No active item".to_string()
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Active Item"),
        )
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
                    ToolStatus::Running => "▶",
                    ToolStatus::Completed => "✓",
                    ToolStatus::Error => "✗",
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Agent Activity"),
        )
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

    let list = List::new(logs).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Agent Output"),
    );

    f.render_widget(list, area);
}

/// Render the footer section (4 lines)
pub fn render_footer(f: &mut Frame, area: Rect, state: &TuiState, show_logs: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let border_width = area.width as usize;

    // Separator line
    let separator = Line::from(vec![
        Span::styled("├", Style::default().fg(Color::Cyan)),
        Span::styled(
            "─".repeat(border_width.saturating_sub(2)),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("┤", Style::default().fg(Color::Cyan)),
    ]);
    let separator_paragraph = Paragraph::new(Text::from(separator));
    f.render_widget(separator_paragraph, chunks[0]);

    // Progress line
    let progress_text = format!(
        "Progress: {}/{} complete | Runtime: {}",
        state.completed_count,
        state.total_count,
        format_runtime(state.start_time)
    );
    let progress_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            pad_to_width(&progress_text, border_width.saturating_sub(4)),
            Style::default(),
        ),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let progress_paragraph = Paragraph::new(Text::from(progress_line));
    f.render_widget(progress_paragraph, chunks[1]);

    // Empty line
    let empty_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            pad_to_width("", border_width.saturating_sub(4)),
            Style::default(),
        ),
        Span::styled(" │", Style::default().fg(Color::Cyan)),
    ]);
    let empty_paragraph = Paragraph::new(Text::from(empty_line));
    f.render_widget(empty_paragraph, chunks[2]);

    // Keyboard shortcuts line
    let logs_label = if show_logs { "items" } else { "logs" };
    let keys_text = format!("[q] quit  [l] {}", logs_label);
    let keys_line = Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::Cyan)),
        Span::styled(
            pad_to_width(&keys_text, border_width.saturating_sub(4)),
            Style::default(),
        ),
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
