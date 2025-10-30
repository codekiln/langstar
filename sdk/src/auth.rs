use crate::error::{LangstarError, Result};
use std::env;

/// Authentication configuration for LangChain services
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// LangSmith API key
    pub langsmith_api_key: Option<String>,
    /// LangGraph Cloud API key
    pub langgraph_api_key: Option<String>,
    /// Optional organization ID for scoping API requests
    pub organization_id: Option<String>,
    /// Optional workspace ID for narrower scoping of API requests
    pub workspace_id: Option<String>,
}

impl AuthConfig {
    /// Create a new AuthConfig by loading from environment variables
    ///
    /// Loads the following environment variables:
    /// - `LANGSMITH_API_KEY` - LangSmith API key
    /// - `LANGGRAPH_API_KEY` - LangGraph Cloud API key
    /// - `LANGSMITH_ORGANIZATION_ID` - Optional organization ID for scoping
    /// - `LANGSMITH_WORKSPACE_ID` - Optional workspace ID for narrower scoping
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            langsmith_api_key: env::var("LANGSMITH_API_KEY").ok(),
            langgraph_api_key: env::var("LANGGRAPH_API_KEY").ok(),
            organization_id: env::var("LANGSMITH_ORGANIZATION_ID").ok(),
            workspace_id: env::var("LANGSMITH_WORKSPACE_ID").ok(),
        })
    }

    /// Create a new AuthConfig with explicit values
    pub fn new(
        langsmith_api_key: Option<String>,
        langgraph_api_key: Option<String>,
        organization_id: Option<String>,
        workspace_id: Option<String>,
    ) -> Self {
        Self {
            langsmith_api_key,
            langgraph_api_key,
            organization_id,
            workspace_id,
        }
    }

    /// Get LangSmith API key, returning error if not configured
    pub fn require_langsmith_key(&self) -> Result<&str> {
        self.langsmith_api_key.as_deref().ok_or_else(|| {
            LangstarError::AuthError(
                "LANGSMITH_API_KEY not configured. Set it via environment variable or config file."
                    .to_string(),
            )
        })
    }

    /// Get LangGraph API key, returning error if not configured
    pub fn require_langgraph_key(&self) -> Result<&str> {
        self.langgraph_api_key.as_deref().ok_or_else(|| {
            LangstarError::AuthError(
                "LANGGRAPH_API_KEY not configured. Set it via environment variable or config file."
                    .to_string(),
            )
        })
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
            Some("test_org_id".to_string()),
            Some("test_workspace_id".to_string()),
        );
        assert_eq!(
            config.require_langsmith_key().unwrap(),
            "test_langsmith_key"
        );
        assert_eq!(
            config.require_langgraph_key().unwrap(),
            "test_langgraph_key"
        );
        assert_eq!(
            config.organization_id.as_deref().unwrap(),
            "test_org_id"
        );
        assert_eq!(
            config.workspace_id.as_deref().unwrap(),
            "test_workspace_id"
        );
    }

    #[test]
    fn test_auth_config_missing_key() {
        let config = AuthConfig::new(None, None, None, None);
        assert!(config.require_langsmith_key().is_err());
        assert!(config.require_langgraph_key().is_err());
    }

    #[test]
    fn test_auth_config_with_org_only() {
        let config = AuthConfig::new(
            Some("test_key".to_string()),
            None,
            Some("org_123".to_string()),
            None,
        );
        assert_eq!(config.organization_id.as_deref().unwrap(), "org_123");
        assert!(config.workspace_id.is_none());
    }

    #[test]
    fn test_auth_config_with_workspace_only() {
        let config = AuthConfig::new(
            Some("test_key".to_string()),
            None,
            None,
            Some("workspace_456".to_string()),
        );
        assert!(config.organization_id.is_none());
        assert_eq!(
            config.workspace_id.as_deref().unwrap(),
            "workspace_456"
        );
    }
}
