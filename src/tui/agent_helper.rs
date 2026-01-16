//! Helper for running agents with TUI updates

use crate::agent::{run_agent, AgentResult, RunAgentOptions};
use crate::errors::Result;
use crate::tui::events::AgentEvent;
use crate::tui::runner::TuiUpdate;

/// Run an agent with TUI updates
///
/// This helper wraps the `run_agent` function and forwards all agent events
/// to the TUI via the provided channel sender.
///
/// # Arguments
/// * `options` - Agent execution options (will be cloned and modified)
/// * `item_id` - ID of the item being processed (for event tracking)
/// * `tui_tx` - Channel sender for TUI state updates
///
/// # Returns
/// The result of the agent execution
pub async fn run_agent_with_tui(
    options: RunAgentOptions,
    item_id: String,
    tui_tx: tokio::sync::mpsc::Sender<TuiUpdate>,
) -> Result<AgentResult> {
    // Create a channel for agent events
    let (agent_event_tx, mut agent_event_rx) = tokio::sync::mpsc::channel::<AgentEvent>(100);

    // Create a task to forward agent events to TUI updates
    let tui_tx_clone = tui_tx.clone();
    let item_id_clone = item_id.clone();
    let event_forwarder = tokio::spawn(async move {
        while let Some(event) = agent_event_rx.recv().await {
            let _ = tui_tx_clone.send(TuiUpdate::AgentEvent(item_id_clone.clone(), event)).await;
        }
    });

    // Wrap the options with the event channel
    let options_with_events = RunAgentOptions {
        on_tui_event: Some(agent_event_tx),
        ..options
    };

    // Run the agent
    let result = run_agent(options_with_events).await;

    // Abort the event forwarder task
    event_forwarder.abort();

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::AgentConfig;

    #[tokio::test]
    async fn test_run_agent_with_tui_dry_run() {
        let (tui_tx, mut tui_rx) = tokio::sync::mpsc::channel::<TuiUpdate>(100);

        let options = RunAgentOptions {
            config: AgentConfig::default(),
            cwd: std::path::PathBuf::from("."),
            prompt: "test".to_string(),
            dry_run: true,
            timeout_seconds: 60,
            on_stdout: None,
            on_stderr: None,
            on_tui_event: None,
        };

        let result = run_agent_with_tui(options, "test-item".to_string(), tui_tx.clone()).await.unwrap();

        assert!(result.success);
        assert!(result.completion_detected);

        // Verify no TUI updates were sent (dry run doesn't generate events)
        // Drop the sender to close the channel
        drop(tui_tx);
        let updates = tui_rx.recv().await;
        assert!(updates.is_none(), "Dry run should not generate TUI updates");
    }

    #[tokio::test]
    async fn test_run_agent_with_tui_forwards_events() {
        let (tui_tx, mut tui_rx) = tokio::sync::mpsc::channel::<TuiUpdate>(100);

        let options = RunAgentOptions {
            config: AgentConfig {
                mode: crate::schemas::AgentMode::Process,
                command: "echo".to_string(),
                args: vec![
                    "<assistant_text>Thinking about the problem</assistant_text>".to_string()
                ],
                completion_signal: "Thinking".to_string(),
            },
            cwd: std::path::PathBuf::from("."),
            prompt: String::new(),
            dry_run: false,
            timeout_seconds: 10,
            on_stdout: None,
            on_stderr: None,
            on_tui_event: None,
        };

        // Spawn a task to collect TUI updates
        let update_collector = tokio::spawn(async move {
            let mut updates = Vec::new();
            while let Some(update) = tui_rx.recv().await {
                updates.push(update);
                // Collect at most 5 updates
                if updates.len() >= 5 {
                    break;
                }
            }
            updates
        });

        let result = run_agent_with_tui(options, "test-item".to_string(), tui_tx.clone()).await.unwrap();

        assert!(result.success);

        // Give the collector a moment to finish
        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        drop(tui_tx);

        // Verify that some TUI updates were sent
        let updates = update_collector.await.unwrap();
        assert!(!updates.is_empty(), "Should have received TUI updates");
    }
}
