//! Langstar SDK - Rust SDK for LangChain ecosystem
//!
//! This SDK provides ergonomic access to LangSmith and LangGraph Cloud APIs.
//! It wraps OpenAPI-generated clients with authentication, error handling,
//! and convenience methods.
//!
//! # Examples
//!
//! ```no_run
//! use langstar_sdk::{AuthConfig, LangchainClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load authentication from environment variables
//!     let auth = AuthConfig::from_env()?;
//!
//!     // Create client
//!     let client = LangchainClient::new(auth)?;
//!
//!     // Make API calls...
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod prompts;

// Re-export commonly used types
pub use auth::AuthConfig;
pub use client::{LangchainClient, ListResponse};
pub use error::{LangstarError, Result};
pub use prompts::{Prompt, PromptClient};
