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

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}
