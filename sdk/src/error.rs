use crate::organization::{CHOOSE_WORKSPACE_HELP, Workspace, describe_workspaces};
use thiserror::Error;

/// Result type for SDK operations
pub type Result<T> = std::result::Result<T, LangstarError>;

/// Errors that can occur when using the Langstar SDK
#[derive(Error, Debug)]
pub enum LangstarError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// API returned an error response
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    /// JSON serialization/deserialization failed
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    /// Invalid schema format
    #[error("Invalid JSON Schema: {0}")]
    InvalidSchemaError(String),

    /// Invalid structured output method
    #[error("Invalid structured output method: {0}. Valid methods: json_schema, function_calling")]
    InvalidMethodError(String),

    /// Every GitHub integration listed its repositories, and none includes this one
    #[error("No GitHub integration has access to {owner}/{repo}")]
    NoGitHubIntegrationForRepo { owner: String, repo: String },

    /// A request with no workspace got 403, and the API key reaches several workspaces
    #[error(
        "The server answered 403 Forbidden. The API key reaches more than one workspace, \
         and no workspace was chosen, so langstar cannot tell which one to use.\n\
         Workspaces this key reaches:\n{}\n{}",
        describe_workspaces(.reachable),
        CHOOSE_WORKSPACE_HELP
    )]
    WorkspaceRequired { reachable: Vec<Workspace> },

    /// A request for a workspace got 403, and the API key does not reach that workspace
    #[error(
        "The server answered 403 Forbidden for workspace {workspace_id}, \
         and the API key does not reach that workspace.\n\
         Workspaces this key reaches:\n{}\n{}",
        describe_workspaces(.reachable),
        CHOOSE_WORKSPACE_HELP
    )]
    WorkspaceNotReachable {
        workspace_id: String,
        reachable: Vec<Workspace>,
    },

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}
