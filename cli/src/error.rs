use langstar_sdk::LangstarError;
use std::fmt;

/// CLI-specific error type
pub type Result<T> = std::result::Result<T, CliError>;

/// Errors that can occur in the CLI
#[derive(Debug)]
pub enum CliError {
    /// SDK error
    Sdk(LangstarError),
    /// Configuration error
    Config(String),
    /// IO error
    Io(std::io::Error),
    /// Other errors
    Other(anyhow::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Sdk(e) => write!(f, "{}", e),
            CliError::Config(e) => write!(f, "Configuration error: {}", e),
            CliError::Io(e) => write!(f, "IO error: {}", e),
            CliError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CliError {}

impl From<LangstarError> for CliError {
    fn from(err: LangstarError) -> Self {
        CliError::Sdk(err)
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Other(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Other(err.into())
    }
}
