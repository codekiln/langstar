use crate::config::Config;
use crate::error::{CliError, Result};
use crate::time::ConfiguredTimezone;
use clap::{Args, Subcommand};
use std::fs;
use toml_edit::DocumentMut;

/// Message constants for DRY principle
pub mod messages {
    /// Suppression hint message for workspace/org ID warnings
    pub const SUPPRESS_WORKSPACE_ORG_WARNING: &str = "→ To suppress: Run 'langstar config hide_workspace_and_org_id_message set true' or set LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE=1";
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Show configuration file location and values
    Show,

    /// Get or set a configuration value
    #[command(subcommand)]
    Setting(ConfigSetting),
}

#[derive(Debug, Subcommand)]
pub enum ConfigSetting {
    /// Manage hide_workspace_and_org_id_message setting
    #[command(name = "hide_workspace_and_org_id_message")]
    HideWorkspaceAndOrgIdMessage(SettingAction),

    /// Manage output_format setting
    #[command(name = "output_format")]
    OutputFormat(SettingAction),

    /// Manage timezone setting
    #[command(name = "timezone")]
    Timezone(SettingAction),
}

#[derive(Debug, Args)]
pub struct SettingAction {
    #[command(subcommand)]
    action: Option<SettingCommand>,
}

#[derive(Debug, Subcommand)]
pub enum SettingCommand {
    /// Set the configuration value
    Set {
        /// The value to set
        value: String,
    },
}

impl ConfigCommands {
    pub async fn execute(&self) -> Result<()> {
        match self {
            ConfigCommands::Show => {
                Self::show_config().await?;
            }
            ConfigCommands::Setting(setting) => {
                setting.execute().await?;
            }
        }
        Ok(())
    }

    async fn show_config() -> Result<()> {
        let config = Config::load()?;
        let config_path = Config::config_file_path()?;

        println!("Configuration file: {}", config_path.display());
        println!("\nCurrent configuration:");
        println!("  Output format: {}", config.output_format);

        // Parse and display timezone with validation
        let tz_display = match ConfiguredTimezone::parse(&config.timezone) {
            Ok(tz) => tz.description(),
            Err(_) => format!("{} (invalid, using UTC)", config.timezone),
        };
        println!("  Timezone: {}", tz_display);
        println!(
            "  Hide workspace/org ID warnings: {}",
            config.hide_workspace_and_org_id_message
        );
        println!(
            "  LangSmith API key: {}",
            if config.langsmith_api_key.is_some() {
                "configured"
            } else {
                "not configured"
            }
        );
        println!(
            "  LangGraph API key: {}",
            if config.langgraph_api_key.is_some() {
                "configured"
            } else {
                "not configured"
            }
        );

        // Show scoping configuration
        println!("\nScoping configuration:");
        println!(
            "  Organization ID: {}",
            config
                .organization_id
                .as_deref()
                .unwrap_or("not configured")
        );
        println!(
            "  Workspace ID: {}",
            config.workspace_id.as_deref().unwrap_or("not configured")
        );

        // Show active scope
        if config.workspace_id.is_some() {
            println!("\n  Active scope: Workspace (narrower)");
            println!("  → Operations will be scoped to the workspace");
        } else if config.organization_id.is_some() {
            println!("\n  Active scope: Organization");
            println!("  → Operations will be scoped to the organization");
        } else {
            println!("\n  Active scope: None (global)");
            println!("  → Operations will access all available prompts");
        }

        println!("\nEnvironment variables:");
        println!(
            "  LANGSMITH_API_KEY: {}",
            if std::env::var("LANGSMITH_API_KEY").is_ok() {
                "set"
            } else {
                "not set"
            }
        );
        println!(
            "  LANGSMITH_ORGANIZATION_ID: {}",
            std::env::var("LANGSMITH_ORGANIZATION_ID").unwrap_or_else(|_| "not set".to_string())
        );
        println!(
            "  LANGSMITH_WORKSPACE_ID: {}",
            std::env::var("LANGSMITH_WORKSPACE_ID").unwrap_or_else(|_| "not set".to_string())
        );
        println!(
            "  LANGGRAPH_API_KEY: {}",
            if std::env::var("LANGGRAPH_API_KEY").is_ok() {
                "set"
            } else {
                "not set"
            }
        );
        println!(
            "  LANGSTAR_OUTPUT_FORMAT: {}",
            std::env::var("LANGSTAR_OUTPUT_FORMAT").unwrap_or_else(|_| "not set".to_string())
        );
        println!(
            "  LANGSTAR_TIMEZONE: {}",
            std::env::var("LANGSTAR_TIMEZONE").unwrap_or_else(|_| "not set".to_string())
        );
        println!(
            "  LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE: {}",
            std::env::var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE")
                .unwrap_or_else(|_| "not set".to_string())
        );

        Ok(())
    }
}

impl ConfigSetting {
    async fn execute(&self) -> Result<()> {
        match self {
            ConfigSetting::HideWorkspaceAndOrgIdMessage(action) => {
                Self::handle_hide_workspace_and_org_id_message(action).await?;
            }
            ConfigSetting::OutputFormat(action) => {
                Self::handle_output_format(action).await?;
            }
            ConfigSetting::Timezone(action) => {
                Self::handle_timezone(action).await?;
            }
        }
        Ok(())
    }

    async fn handle_hide_workspace_and_org_id_message(action: &SettingAction) -> Result<()> {
        match &action.action {
            Some(SettingCommand::Set { value }) => {
                // Parse boolean value
                let bool_value = match value.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => {
                        return Err(CliError::Config(format!(
                            "Invalid boolean value '{}'. Use: true, false, 1, 0, yes, no, on, or off",
                            value
                        )));
                    }
                };

                Self::set_config_value(
                    "hide_workspace_and_org_id_message",
                    &bool_value.to_string(),
                )?;
                println!("✓ Set hide_workspace_and_org_id_message = {}", bool_value);
                println!(
                    "  Workspace/org ID warnings will {} be displayed",
                    if bool_value { "NOT" } else { "still" }
                );
            }
            None => {
                // Show help for this setting
                Self::show_setting_help(
                    "hide_workspace_and_org_id_message",
                    "Suppress warnings when both organization and workspace IDs are configured",
                    "boolean (true/false)",
                    &["true", "false"],
                )?;
            }
        }
        Ok(())
    }

    async fn handle_output_format(action: &SettingAction) -> Result<()> {
        match &action.action {
            Some(SettingCommand::Set { value }) => {
                // Validate output format
                let valid_formats = ["json", "table"];
                if !valid_formats.contains(&value.to_lowercase().as_str()) {
                    return Err(CliError::Config(format!(
                        "Invalid output format '{}'. Use: json or table",
                        value
                    )));
                }

                Self::set_config_value("output_format", value)?;
                println!("✓ Set output_format = {}", value);
            }
            None => {
                Self::show_setting_help(
                    "output_format",
                    "Default output format for command results",
                    "string (json or table)",
                    &["json", "table"],
                )?;
            }
        }
        Ok(())
    }

    async fn handle_timezone(action: &SettingAction) -> Result<()> {
        match &action.action {
            Some(SettingCommand::Set { value }) => {
                // Validate timezone
                if let Err(e) = ConfiguredTimezone::parse(value) {
                    return Err(CliError::Config(format!(
                        "Invalid timezone '{}': {}",
                        value, e
                    )));
                }

                Self::set_config_value("timezone", value)?;
                println!("✓ Set timezone = {}", value);
            }
            None => {
                Self::show_setting_help(
                    "timezone",
                    "Default timezone for timestamp display",
                    "string (IANA timezone name or 'local'/'utc')",
                    &["UTC", "local", "America/New_York", "Europe/London"],
                )?;
            }
        }
        Ok(())
    }

    fn show_setting_help(
        name: &str,
        description: &str,
        value_type: &str,
        examples: &[&str],
    ) -> Result<()> {
        let config = Config::load()?;
        let config_path = Config::config_file_path()?;

        // Get current value using reflection or manual matching
        let current_value = match name {
            "hide_workspace_and_org_id_message" => {
                config.hide_workspace_and_org_id_message.to_string()
            }
            "output_format" => config.output_format.clone(),
            "timezone" => config.timezone.clone(),
            _ => "unknown".to_string(),
        };

        println!("Setting: {}", name);
        println!("Description: {}", description);
        println!("Type: {}", value_type);
        println!("Current value: {}", current_value);
        println!("\nExamples:");
        for example in examples {
            println!("  langstar config {} set {}", name, example);
        }
        println!("\nConfiguration file: {}", config_path.display());

        Ok(())
    }

    fn set_config_value(key: &str, new_value: &str) -> Result<()> {
        let config_path = Config::config_file_path()?;

        // Create config file if it doesn't exist
        if !config_path.exists() {
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&config_path, "")?;
        }

        // Read and parse the config file
        let config_content = fs::read_to_string(&config_path)?;
        let mut doc = config_content
            .parse::<DocumentMut>()
            .map_err(|e| CliError::Config(format!("Failed to parse config file: {}", e)))?;

        // Set the value based on type
        match key {
            "hide_workspace_and_org_id_message" => {
                let bool_value = new_value
                    .parse::<bool>()
                    .map_err(|e| CliError::Config(format!("Failed to parse boolean: {}", e)))?;
                doc[key] = toml_edit::value(bool_value);
            }
            _ => {
                doc[key] = toml_edit::value(new_value);
            }
        }

        // Write the updated config
        fs::write(&config_path, doc.to_string())?;

        Ok(())
    }
}
