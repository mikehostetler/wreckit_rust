//! Config schema - Configuration for wreckit

use serde::{Deserialize, Serialize};

/// Agent execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    /// Execute agent via process spawning
    #[default]
    Process,
    /// Execute agent via SDK (not implemented in Rust port)
    Sdk,
}

/// Merge mode for completed work
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MergeMode {
    /// Create a pull request
    #[default]
    Pr,
    /// Direct merge to base branch (YOLO mode)
    Direct,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent execution mode
    #[serde(default)]
    pub mode: AgentMode,

    /// Command to execute (e.g., "claude")
    pub command: String,

    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,

    /// Signal that indicates agent completion
    pub completion_signal: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        AgentConfig {
            mode: AgentMode::Process,
            command: "claude".to_string(),
            args: vec![
                "--dangerously-skip-permissions".to_string(),
                "--print".to_string(),
            ],
            completion_signal: "<promise>COMPLETE</promise>".to_string(),
        }
    }
}

/// Main configuration for wreckit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Schema version for forward compatibility
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Base branch for PRs (e.g., "main")
    #[serde(default = "default_base_branch")]
    pub base_branch: String,

    /// Prefix for feature branches (e.g., "wreckit/")
    #[serde(default = "default_branch_prefix")]
    pub branch_prefix: String,

    /// Merge mode for completed work
    #[serde(default)]
    pub merge_mode: MergeMode,

    /// Agent configuration
    #[serde(default)]
    pub agent: AgentConfig,

    /// Maximum iterations for implementation phase
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,

    /// Timeout in seconds for agent execution
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,
}

fn default_schema_version() -> u32 {
    1
}

fn default_base_branch() -> String {
    "main".to_string()
}

fn default_branch_prefix() -> String {
    "wreckit/".to_string()
}

fn default_max_iterations() -> u32 {
    100
}

fn default_timeout_seconds() -> u32 {
    3600
}

impl Default for Config {
    fn default() -> Self {
        Config {
            schema_version: 1,
            base_branch: "main".to_string(),
            branch_prefix: "wreckit/".to_string(),
            merge_mode: MergeMode::Pr,
            agent: AgentConfig::default(),
            max_iterations: 100,
            timeout_seconds: 3600,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.schema_version, 1);
        assert_eq!(config.base_branch, "main");
        assert_eq!(config.branch_prefix, "wreckit/");
        assert_eq!(config.merge_mode, MergeMode::Pr);
        assert_eq!(config.max_iterations, 100);
        assert_eq!(config.timeout_seconds, 3600);
    }

    #[test]
    fn test_agent_config_default() {
        let agent = AgentConfig::default();
        assert_eq!(agent.mode, AgentMode::Process);
        assert_eq!(agent.command, "claude");
        assert_eq!(agent.args, vec!["--dangerously-skip-permissions", "--print"]);
        assert_eq!(agent.completion_signal, "<promise>COMPLETE</promise>");
    }

    #[test]
    fn test_config_json_round_trip() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.base_branch, config.base_branch);
        assert_eq!(parsed.branch_prefix, config.branch_prefix);
        assert_eq!(parsed.agent.command, config.agent.command);
    }

    #[test]
    fn test_config_partial_json() {
        // Simulate a config file with only some fields set
        let json = r#"{"base_branch": "develop"}"#;
        let parsed: Config = serde_json::from_str(json).unwrap();

        assert_eq!(parsed.base_branch, "develop");
        // Other fields should have defaults
        assert_eq!(parsed.branch_prefix, "wreckit/");
        assert_eq!(parsed.max_iterations, 100);
    }

    #[test]
    fn test_merge_mode_serialization() {
        assert_eq!(serde_json::to_string(&MergeMode::Pr).unwrap(), "\"pr\"");
        assert_eq!(serde_json::to_string(&MergeMode::Direct).unwrap(), "\"direct\"");
    }

    #[test]
    fn test_agent_mode_serialization() {
        assert_eq!(serde_json::to_string(&AgentMode::Process).unwrap(), "\"process\"");
        assert_eq!(serde_json::to_string(&AgentMode::Sdk).unwrap(), "\"sdk\"");
    }
}
