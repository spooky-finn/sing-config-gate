//! Centralized error types for the application.

use thiserror::Error;

/// Errors related to environment variable loading and validation.
#[derive(Error, Debug)]
pub enum EnvError {
    #[error("Missing required environment variable: {0}")]
    Missing(String),

    #[error("Invalid environment variable {0}: {1}")]
    Invalid(String, String),
}

/// Repository operation errors.
#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<diesel::result::Error> for RepoError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => RepoError::NotFound("Resource not found".to_string()),
            _ => RepoError::Database(err.to_string()),
        }
    }
}

/// Configuration generation errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to generate config: {0}")]
    Generation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Deployment errors.
#[derive(Error, Debug)]
pub enum DeployError {
    #[error("SSH connection failed: {0}")]
    Ssh(String),

    #[error("Command execution failed (exit code {0}): {1}")]
    CommandFailed(i32, String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<ssh2::Error> for DeployError {
    fn from(err: ssh2::Error) -> Self {
        DeployError::Ssh(err.to_string())
    }
}
