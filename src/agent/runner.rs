//! Process-based agent runner
//!
//! Executes agents via process spawning with stdin/stdout streaming.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use crate::errors::{Result, WreckitError};
use crate::schemas::AgentConfig;

/// Result of an agent execution
#[derive(Debug)]
pub struct AgentResult {
    /// Whether the agent completed successfully
    pub success: bool,

    /// Combined stdout/stderr output
    pub output: String,

    /// Whether the agent timed out
    pub timed_out: bool,

    /// Exit code (if process exited normally)
    pub exit_code: Option<i32>,

    /// Whether the completion signal was detected
    pub completion_detected: bool,
}

/// Options for running an agent
pub struct RunAgentOptions {
    /// Agent configuration
    pub config: AgentConfig,

    /// Working directory for the agent
    pub cwd: PathBuf,

    /// Prompt to send to the agent
    pub prompt: String,

    /// If true, return mock result without spawning
    pub dry_run: bool,

    /// Timeout in seconds
    pub timeout_seconds: u32,

    /// Callback for stdout chunks (optional)
    pub on_stdout: Option<Box<dyn Fn(&str) + Send>>,

    /// Callback for stderr chunks (optional)
    pub on_stderr: Option<Box<dyn Fn(&str) + Send>>,
}

/// Run an agent with the given options.
///
/// This function:
/// 1. Spawns the agent process with the configured command and args
/// 2. Writes the prompt to stdin and closes it
/// 3. Reads stdout/stderr, buffering output
/// 4. Detects the completion signal in output
/// 5. Applies timeout (SIGTERM, then SIGKILL after 5s)
/// 6. Returns result with exit code and completion status
///
/// # Arguments
/// * `options` - Agent execution options
///
/// # Returns
/// The result of the agent execution
pub async fn run_agent(options: RunAgentOptions) -> Result<AgentResult> {
    // Handle dry-run mode
    if options.dry_run {
        return Ok(AgentResult {
            success: true,
            output: "[DRY RUN] Would execute agent".to_string(),
            timed_out: false,
            exit_code: Some(0),
            completion_detected: true,
        });
    }

    let mut cmd = Command::new(&options.config.command);
    cmd.args(&options.config.args)
        .current_dir(&options.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| WreckitError::AgentError(format!("Failed to spawn agent: {}", e)))?;

    // Write prompt to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(options.prompt.as_bytes())
            .await
            .map_err(|e| WreckitError::AgentError(format!("Failed to write to stdin: {}", e)))?;
        // stdin is dropped here, closing it
    }

    let mut output = String::new();
    let mut completion_detected = false;

    // Read stdout
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let timeout_duration = Duration::from_secs(options.timeout_seconds as u64);

    let result = timeout(timeout_duration, async {
        // Read stdout and stderr concurrently
        let stdout_handle = tokio::spawn(async move {
            let mut stdout_output = String::new();
            if let Some(stdout) = stdout {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    stdout_output.push_str(&line);
                    stdout_output.push('\n');
                }
            }
            stdout_output
        });

        let stderr_handle = tokio::spawn(async move {
            let mut stderr_output = String::new();
            if let Some(stderr) = stderr {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    stderr_output.push_str(&line);
                    stderr_output.push('\n');
                }
            }
            stderr_output
        });

        let stdout_output = stdout_handle.await.unwrap_or_default();
        let stderr_output = stderr_handle.await.unwrap_or_default();

        (stdout_output, stderr_output, child.wait().await)
    })
    .await;

    match result {
        Ok((stdout_output, stderr_output, wait_result)) => {
            output.push_str(&stdout_output);
            output.push_str(&stderr_output);

            // Check for completion signal
            completion_detected = output.contains(&options.config.completion_signal);

            // Call callbacks if provided
            if let Some(ref on_stdout) = options.on_stdout {
                on_stdout(&stdout_output);
            }
            if let Some(ref on_stderr) = options.on_stderr {
                on_stderr(&stderr_output);
            }

            match wait_result {
                Ok(status) => Ok(AgentResult {
                    success: status.success() && completion_detected,
                    output,
                    timed_out: false,
                    exit_code: status.code(),
                    completion_detected,
                }),
                Err(e) => Err(WreckitError::AgentError(format!(
                    "Failed to wait for agent: {}",
                    e
                ))),
            }
        }
        Err(_) => {
            // Timeout occurred - kill the process
            let _ = child.kill().await;

            Ok(AgentResult {
                success: false,
                output,
                timed_out: true,
                exit_code: None,
                completion_detected: false,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dry_run() {
        let options = RunAgentOptions {
            config: AgentConfig::default(),
            cwd: PathBuf::from("."),
            prompt: "test prompt".to_string(),
            dry_run: true,
            timeout_seconds: 60,
            on_stdout: None,
            on_stderr: None,
        };

        let result = run_agent(options).await.unwrap();

        assert!(result.success);
        assert!(result.output.contains("DRY RUN"));
        assert!(!result.timed_out);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.completion_detected);
    }

    #[tokio::test]
    async fn test_simple_command() {
        let options = RunAgentOptions {
            config: AgentConfig {
                mode: crate::schemas::AgentMode::Process,
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                completion_signal: "hello".to_string(),
            },
            cwd: PathBuf::from("."),
            prompt: String::new(),
            dry_run: false,
            timeout_seconds: 10,
            on_stdout: None,
            on_stderr: None,
        };

        let result = run_agent(options).await.unwrap();

        assert!(result.success);
        assert!(result.output.contains("hello"));
        assert!(!result.timed_out);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.completion_detected);
    }
}
