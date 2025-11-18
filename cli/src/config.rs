use crate::error::{CliError, Result};
use langstar_sdk::AuthConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the Langstar CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LangSmith API key
    pub langsmith_api_key: Option<String>,
    /// LangGraph Cloud API key
    pub langgraph_api_key: Option<String>,
    /// Optional organization ID for scoping LangSmith operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Optional workspace ID for narrower scoping of LangSmith operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Optional GitHub integration ID for deployment creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_integration_id: Option<String>,
    /// Default output format (json or table)
    #[serde(default = "default_output_format")]
    pub output_format: String,
}

fn default_output_format() -> String {
    "table".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            langsmith_api_key: None,
            langgraph_api_key: None,
            organization_id: None,
            workspace_id: None,
            github_integration_id: None,
            output_format: default_output_format(),
        }
    }
}

impl Config {
    /// Load configuration from file and environment variables
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables
    /// 2. Config file (~/.config/langstar/config.toml)
    /// 3. Default values
    pub fn load() -> Result<Self> {
        // Start with file config if it exists
        let mut config = Self::load_from_file().unwrap_or_default();

        // Override with environment variables
        if let Ok(key) = std::env::var("LANGSMITH_API_KEY") {
            config.langsmith_api_key = Some(key);
        }
        if let Ok(key) = std::env::var("LANGGRAPH_API_KEY") {
            config.langgraph_api_key = Some(key);
        }
        if let Ok(org_id) = std::env::var("LANGSMITH_ORGANIZATION_ID") {
            config.organization_id = Some(org_id);
        }
        if let Ok(workspace_id) = std::env::var("LANGSMITH_WORKSPACE_ID") {
            config.workspace_id = Some(workspace_id);
        }
        if let Ok(integration_id) = std::env::var("LANGGRAPH_GITHUB_INTEGRATION_ID") {
            config.github_integration_id = Some(integration_id);
        }
        if let Ok(format) = std::env::var("LANGSTAR_OUTPUT_FORMAT") {
            config.output_format = format;
        }

        // Log warning if both organization and workspace IDs are set
        if config.organization_id.is_some() && config.workspace_id.is_some() {
            eprintln!(
                "Warning: Both organization_id and workspace_id are set. Workspace ID takes precedence for narrower scoping."
            );
        }

        Ok(config)
    }

    /// Load configuration from the config file
    fn load_from_file() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| CliError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| CliError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Get the path to the config file
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CliError::Config("Could not determine config directory".to_string()))?;

        Ok(config_dir.join("langstar").join("config.toml"))
    }

    /// Save the current configuration to file
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Convert to AuthConfig for the SDK
    pub fn to_auth_config(&self) -> AuthConfig {
        AuthConfig::new(
            self.langsmith_api_key.clone(),
            self.langgraph_api_key.clone(),
            self.organization_id.clone(),
            self.workspace_id.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.output_format, "table");
        assert!(config.langsmith_api_key.is_none());
        assert!(config.langgraph_api_key.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            langsmith_api_key: Some("test_key".to_string()),
            langgraph_api_key: None,
            organization_id: Some("test_org_id".to_string()),
            workspace_id: None,
            github_integration_id: None,
            output_format: "json".to_string(),
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("langsmith_api_key"));
        assert!(toml.contains("test_key"));
        assert!(toml.contains("organization_id"));
        assert!(toml.contains("test_org_id"));
        assert!(toml.contains("json"));
    }

    #[test]
    fn test_config_with_workspace() {
        let config = Config {
            langsmith_api_key: Some("test_key".to_string()),
            langgraph_api_key: None,
            organization_id: None,
            workspace_id: Some("test_workspace_id".to_string()),
            github_integration_id: None,
            output_format: "table".to_string(),
        };

        let auth = config.to_auth_config();
        assert!(auth.organization_id.is_none());
        assert_eq!(auth.workspace_id, Some("test_workspace_id".to_string()));
    }

    #[test]
    fn test_config_to_auth_config_with_both() {
        let config = Config {
            langsmith_api_key: Some("key".to_string()),
            langgraph_api_key: None,
            organization_id: Some("org_123".to_string()),
            workspace_id: Some("workspace_456".to_string()),
            github_integration_id: None,
            output_format: "table".to_string(),
        };

        let auth = config.to_auth_config();
        assert_eq!(auth.organization_id, Some("org_123".to_string()));
        assert_eq!(auth.workspace_id, Some("workspace_456".to_string()));
    }
}
