use crate::error::{LangstarError, Result};
use std::env;

/// Authentication configuration for LangChain services
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// LangSmith API key
    pub langsmith_api_key: Option<String>,
    /// LangGraph Cloud API key
    pub langgraph_api_key: Option<String>,
}

impl AuthConfig {
    /// Create a new AuthConfig by loading from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            langsmith_api_key: env::var("LANGSMITH_API_KEY").ok(),
            langgraph_api_key: env::var("LANGGRAPH_API_KEY").ok(),
        })
    }

    /// Create a new AuthConfig with explicit API keys
    pub fn new(langsmith_api_key: Option<String>, langgraph_api_key: Option<String>) -> Self {
        Self {
            langsmith_api_key,
            langgraph_api_key,
        }
    }

    /// Get LangSmith API key, returning error if not configured
    pub fn require_langsmith_key(&self) -> Result<&str> {
        self.langsmith_api_key
            .as_deref()
            .ok_or_else(|| LangstarError::AuthError(
                "LANGSMITH_API_KEY not configured. Set it via environment variable or config file.".to_string()
            ))
    }

    /// Get LangGraph API key, returning error if not configured
    pub fn require_langgraph_key(&self) -> Result<&str> {
        self.langgraph_api_key
            .as_deref()
            .ok_or_else(|| LangstarError::AuthError(
                "LANGGRAPH_API_KEY not configured. Set it via environment variable or config file.".to_string()
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_new() {
        let config = AuthConfig::new(
            Some("test_langsmith_key".to_string()),
            Some("test_langgraph_key".to_string()),
        );
        assert_eq!(config.require_langsmith_key().unwrap(), "test_langsmith_key");
        assert_eq!(config.require_langgraph_key().unwrap(), "test_langgraph_key");
    }

    #[test]
    fn test_auth_config_missing_key() {
        let config = AuthConfig::new(None, None);
        assert!(config.require_langsmith_key().is_err());
        assert!(config.require_langgraph_key().is_err());
    }
}
