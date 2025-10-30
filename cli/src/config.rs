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
        if let Ok(format) = std::env::var("LANGSTAR_OUTPUT_FORMAT") {
            config.output_format = format;
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
            output_format: "json".to_string(),
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("langsmith_api_key"));
        assert!(toml.contains("test_key"));
        assert!(toml.contains("json"));
    }
}
