//! Error types for the wreckit CLI
//!
//! Each error type has a corresponding error code for programmatic handling.

use thiserror::Error;

/// Result type alias for wreckit operations
pub type Result<T> = std::result::Result<T, WreckitError>;

/// Main error type for all wreckit operations
#[derive(Debug, Error)]
pub enum WreckitError {
    /// Repository not found - no .git and .wreckit directories
    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    /// Invalid JSON format
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    /// Schema validation failed
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Agent execution error
    #[error("Agent error: {0}")]
    AgentError(String),

    /// Git operation error
    #[error("Git error: {0}")]
    GitError(String),

    /// Operation timed out
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Operation was interrupted (e.g., by SIGINT)
    #[error("Operation interrupted")]
    Interrupted,

    /// Workflow state transition error
    #[error("State transition error: {0}")]
    StateTransition(String),

    /// IO error wrapper
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic wrapped error with context
    #[error("{context}: {message}")]
    Wrapped { context: String, message: String },
}

impl WreckitError {
    /// Get the error code for this error type
    pub fn code(&self) -> &'static str {
        match self {
            WreckitError::RepoNotFound(_) => "REPO_NOT_FOUND",
            WreckitError::InvalidJson(_) => "INVALID_JSON",
            WreckitError::SchemaValidation(_) => "SCHEMA_VALIDATION",
            WreckitError::FileNotFound(_) => "FILE_NOT_FOUND",
            WreckitError::ConfigError(_) => "CONFIG_ERROR",
            WreckitError::AgentError(_) => "AGENT_ERROR",
            WreckitError::GitError(_) => "GIT_ERROR",
            WreckitError::Timeout(_) => "TIMEOUT",
            WreckitError::Interrupted => "INTERRUPTED",
            WreckitError::StateTransition(_) => "STATE_TRANSITION",
            WreckitError::Io(_) => "IO_ERROR",
            WreckitError::Wrapped { .. } => "WRAPPED_ERROR",
        }
    }

    /// Wrap an error with additional context
    pub fn wrap<E: std::fmt::Display>(error: E, context: impl Into<String>) -> Self {
        WreckitError::Wrapped {
            context: context.into(),
            message: error.to_string(),
        }
    }
}

/// Convert an error to an appropriate exit code
pub fn to_exit_code(error: &WreckitError) -> i32 {
    match error {
        WreckitError::Interrupted => 130, // Standard Unix exit code for SIGINT
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(WreckitError::RepoNotFound("test".into()).code(), "REPO_NOT_FOUND");
        assert_eq!(WreckitError::InvalidJson("test".into()).code(), "INVALID_JSON");
        assert_eq!(WreckitError::SchemaValidation("test".into()).code(), "SCHEMA_VALIDATION");
        assert_eq!(WreckitError::FileNotFound("test".into()).code(), "FILE_NOT_FOUND");
        assert_eq!(WreckitError::ConfigError("test".into()).code(), "CONFIG_ERROR");
        assert_eq!(WreckitError::AgentError("test".into()).code(), "AGENT_ERROR");
        assert_eq!(WreckitError::GitError("test".into()).code(), "GIT_ERROR");
        assert_eq!(WreckitError::Timeout("test".into()).code(), "TIMEOUT");
        assert_eq!(WreckitError::Interrupted.code(), "INTERRUPTED");
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(to_exit_code(&WreckitError::Interrupted), 130);
        assert_eq!(to_exit_code(&WreckitError::RepoNotFound("test".into())), 1);
        assert_eq!(to_exit_code(&WreckitError::GitError("test".into())), 1);
    }

    #[test]
    fn test_wrap_error() {
        let wrapped = WreckitError::wrap("inner error", "outer context");
        assert_eq!(wrapped.code(), "WRAPPED_ERROR");
        assert!(wrapped.to_string().contains("outer context"));
        assert!(wrapped.to_string().contains("inner error"));
    }
}
