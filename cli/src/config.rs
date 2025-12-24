use crate::error::{CliError, Result};
use langstar_sdk::AuthConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the Langstar CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LangSmith API key (used for both LangSmith and LangGraph APIs)
    pub langsmith_api_key: Option<String>,
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
    /// Timezone for displaying timestamps in CLI output
    ///
    /// Accepts IANA timezone names (e.g., "America/New_York", "Europe/London")
    /// or special values: "local" (system timezone), "UTC"
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Suppress warning when both organization_id and workspace_id are set
    #[serde(default)]
    pub hide_workspace_and_org_id_message: bool,
}

fn default_output_format() -> String {
    "table".to_string()
}

fn default_timezone() -> String {
    "local".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            langsmith_api_key: None,
            organization_id: None,
            workspace_id: None,
            github_integration_id: None,
            output_format: default_output_format(),
            timezone: default_timezone(),
            hide_workspace_and_org_id_message: false,
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
        if let Ok(tz) = std::env::var("LANGSTAR_TIMEZONE") {
            config.timezone = tz;
        }
        if let Ok(hide) = std::env::var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE") {
            config.hide_workspace_and_org_id_message =
                hide == "1" || hide.eq_ignore_ascii_case("true");
        }

        // Log warning if both organization and workspace IDs are set
        if !config.hide_workspace_and_org_id_message
            && config.organization_id.is_some()
            && config.workspace_id.is_some()
        {
            use crate::commands::config::messages;
            eprintln!(
                "Warning: Both organization_id and workspace_id are set. Workspace ID takes precedence for narrower scoping."
            );
            eprintln!("  {}", messages::SUPPRESS_WORKSPACE_ORG_WARNING);
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
    ///
    /// Uses ~/.config/langstar/config.toml on macOS and Linux for consistency.
    /// On Windows, uses the platform-specific AppData location.
    pub fn config_file_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let config_dir = dirs::config_dir().ok_or_else(|| {
                CliError::Config("Could not determine config directory".to_string())
            })?;
            Ok(config_dir.join("langstar").join("config.toml"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Use ~/.config/langstar/config.toml on macOS and Linux
            let home_dir = dirs::home_dir().ok_or_else(|| {
                CliError::Config("Could not determine home directory".to_string())
            })?;
            Ok(home_dir
                .join(".config")
                .join("langstar")
                .join("config.toml"))
        }
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
        assert_eq!(config.timezone, "local");
        assert!(config.langsmith_api_key.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            langsmith_api_key: Some("test_key".to_string()),
            organization_id: Some("test_org_id".to_string()),
            workspace_id: None,
            github_integration_id: None,
            output_format: "json".to_string(),
            timezone: "America/New_York".to_string(),
            hide_workspace_and_org_id_message: false,
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("langsmith_api_key"));
        assert!(toml.contains("test_key"));
        assert!(toml.contains("organization_id"));
        assert!(toml.contains("test_org_id"));
        assert!(toml.contains("json"));
        assert!(toml.contains("America/New_York"));
    }

    #[test]
    fn test_config_with_workspace() {
        let config = Config {
            langsmith_api_key: Some("test_key".to_string()),
            organization_id: None,
            workspace_id: Some("test_workspace_id".to_string()),
            github_integration_id: None,
            output_format: "table".to_string(),
            timezone: "local".to_string(),
            hide_workspace_and_org_id_message: false,
        };

        let auth = config.to_auth_config();
        assert!(auth.organization_id.is_none());
        assert_eq!(auth.workspace_id, Some("test_workspace_id".to_string()));
    }

    #[test]
    fn test_config_to_auth_config_with_both() {
        let config = Config {
            langsmith_api_key: Some("key".to_string()),
            organization_id: Some("org_123".to_string()),
            workspace_id: Some("workspace_456".to_string()),
            github_integration_id: None,
            output_format: "table".to_string(),
            timezone: "UTC".to_string(),
            hide_workspace_and_org_id_message: false,
        };

        let auth = config.to_auth_config();
        assert_eq!(auth.organization_id, Some("org_123".to_string()));
        assert_eq!(auth.workspace_id, Some("workspace_456".to_string()));
    }

    #[test]
    fn test_hide_warning_flag() {
        let mut config = Config::default();
        assert!(!config.hide_workspace_and_org_id_message);

        config.hide_workspace_and_org_id_message = true;
        assert!(config.hide_workspace_and_org_id_message);
    }

    #[test]
    fn test_config_serde_with_hide_warning() {
        let toml = r#"
            output_format = "json"
            hide_workspace_and_org_id_message = true
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.hide_workspace_and_org_id_message);
    }

    #[test]
    fn test_env_var_hide_warning_parsing() {
        unsafe {
            // Test parsing "1"
            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "1");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            // Test parsing "true" (case-insensitive)
            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "true");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "TRUE");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "True");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            // Test that other values are treated as false
            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "0");
            let config = Config::load().unwrap();
            assert!(!config.hide_workspace_and_org_id_message);

            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "false");
            let config = Config::load().unwrap();
            assert!(!config.hide_workspace_and_org_id_message);

            // Cleanup
            std::env::remove_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE");
        }
    }
}
