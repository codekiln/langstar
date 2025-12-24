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

    /// Create a new config file with all available options and defaults
    Create {
        /// Overwrite the config file if it already exists
        #[arg(long)]
        force: bool,
    },

    /// Validate the config file and check for errors
    Validate,

    /// Show environment variable mappings for all config keys
    Env,

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
    pub fn execute(&self) -> Result<()> {
        match self {
            ConfigCommands::Show => {
                Self::show_config()?;
            }
            ConfigCommands::Create { force } => {
                Self::handle_create(*force)?;
            }
            ConfigCommands::Validate => {
                Self::handle_validate()?;
            }
            ConfigCommands::Env => {
                Self::handle_env()?;
            }
            ConfigCommands::HideWorkspaceAndOrgIdMessage(action) => {
                Self::handle_hide_workspace_and_org_id_message(action)?;
            }
            ConfigCommands::OutputFormat(action) => {
                Self::handle_output_format(action)?;
            }
            ConfigCommands::Timezone(action) => {
                Self::handle_timezone(action)?;
            }
        }
        Ok(())
    }

    fn show_config() -> Result<()> {
        let config = Config::load()?;
        let config_path = Config::config_file_path()?;

        println!("Configuration file: {}", config_path.display());
        println!("  File exists: {}", config_path.exists());

        println!("\nCurrent configuration:");

        // Show each setting with its value and source
        Self::show_setting_with_source(
            "output_format",
            &config.output_format,
            &std::env::var("LANGSTAR_OUTPUT_FORMAT"),
            "LANGSTAR_OUTPUT_FORMAT",
        );

        // Parse and display timezone with validation
        let tz_display = match ConfiguredTimezone::parse(&config.timezone) {
            Ok(tz) => tz.description(),
            Err(_) => format!("{} (invalid, using UTC)", config.timezone),
        };
        Self::show_setting_with_source(
            "timezone",
            &tz_display,
            &std::env::var("LANGSTAR_TIMEZONE"),
            "LANGSTAR_TIMEZONE",
        );

        Self::show_setting_with_source(
            "hide_workspace_and_org_id_message",
            &config.hide_workspace_and_org_id_message.to_string(),
            &std::env::var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE"),
            "LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE",
        );

        println!("\nAuthentication and scoping:");

        // LangSmith API key
        let api_key_value = if let Some(ref key) = config.langsmith_api_key {
            format!("{}...", &key[..key.len().min(10)])
        } else {
            "not set".to_string()
        };
        Self::show_setting_with_source(
            "langsmith_api_key",
            &api_key_value,
            &std::env::var("LANGSMITH_API_KEY").map(|k| format!("{}...", &k[..k.len().min(10)])),
            "LANGSMITH_API_KEY",
        );

        // Organization ID
        Self::show_setting_with_source(
            "organization_id",
            config.organization_id.as_deref().unwrap_or("not set"),
            &std::env::var("LANGSMITH_ORGANIZATION_ID"),
            "LANGSMITH_ORGANIZATION_ID",
        );

        // Workspace ID
        Self::show_setting_with_source(
            "workspace_id",
            config.workspace_id.as_deref().unwrap_or("not set"),
            &std::env::var("LANGSMITH_WORKSPACE_ID"),
            "LANGSMITH_WORKSPACE_ID",
        );

        // GitHub Integration ID
        Self::show_setting_with_source(
            "github_integration_id",
            config.github_integration_id.as_deref().unwrap_or("not set"),
            &std::env::var("LANGGRAPH_GITHUB_INTEGRATION_ID"),
            "LANGGRAPH_GITHUB_INTEGRATION_ID",
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
            println!("  → Operations will access all available resources");
        }

        println!("\n💡 Tip: Run 'langstar config env' to see all environment variable mappings");
        println!("💡 Tip: Run 'langstar config validate' to check for config file errors");

        Ok(())
    }

    fn show_setting_with_source(
        key: &str,
        current_value: &str,
        env_var_result: &std::result::Result<String, std::env::VarError>,
        env_var_name: &str,
    ) {
        let source = match env_var_result {
            // Env var is set; check whether its value matches the effective configuration value.
            Ok(env_value) if env_value == current_value => {
                format!("(from env: {})", env_var_name)
            }
            // Env var is set but the effective value differs, so the value must be coming
            // from the config file or a default, and the env var is not actually in use.
            Ok(_) => "(from config file or default)".to_string(),
            // Env var is not set; the value is coming from the config file or a built-in default.
            Err(_) => "(from config file or default)".to_string(),
        };
        println!("  {}: {} {}", key, current_value, source);
    }

    fn handle_create(force: bool) -> Result<()> {
        let config_path = Config::config_file_path()?;

        // Check if config file already exists
        if config_path.exists() && !force {
            return Err(CliError::Config(format!(
                "Config file already exists at {}\nUse --force to overwrite",
                config_path.display()
            )));
        }

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create config file content with comments showing environment variable mappings
        let config_content = r#"# Langstar Configuration File
#
# This file provides default values for the Langstar CLI.
# Environment variables take precedence over values in this file.
#
# For a complete mapping of config keys to environment variables, run:
#   langstar config env

# Authentication and Scoping
# --------------------------

# LangSmith API key (used for both LangSmith and LangGraph APIs)
# Environment variable: LANGSMITH_API_KEY
# langsmith_api_key = "your-api-key-here"

# Optional organization ID for scoping LangSmith operations
# Environment variable: LANGSMITH_ORGANIZATION_ID
# organization_id = "your-org-id"

# Optional workspace ID for narrower scoping of LangSmith operations
# Environment variable: LANGSMITH_WORKSPACE_ID
# workspace_id = "your-workspace-id"

# Optional GitHub integration ID for deployment creation
# Environment variable: LANGGRAPH_GITHUB_INTEGRATION_ID
# github_integration_id = "your-integration-id"

# CLI Display Settings
# --------------------

# Default output format for command results (json or table)
# Environment variable: LANGSTAR_OUTPUT_FORMAT
output_format = "table"

# Timezone for displaying timestamps in CLI output
# Accepts IANA timezone names (e.g., "America/New_York", "Europe/London")
# or special values: "local" (system timezone), "UTC"
# Environment variable: LANGSTAR_TIMEZONE
timezone = "local"

# Suppress warning when both organization_id and workspace_id are set
# Environment variable: LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE
hide_workspace_and_org_id_message = false
"#;

        // Write config file
        fs::write(&config_path, config_content)?;

        println!("✓ Created config file at: {}", config_path.display());
        println!("\nNext steps:");
        println!("  1. Edit the file to set your API key and preferences");
        println!("  2. Run 'langstar config show' to verify configuration");
        println!("  3. Run 'langstar config validate' to check for errors");

        Ok(())
    }

    fn handle_validate() -> Result<()> {
        let config_path = Config::config_file_path()?;

        println!("Validating configuration...");
        println!("  Config file: {}", config_path.display());

        // Check if config file exists
        if !config_path.exists() {
            println!("  ⚠ Config file does not exist");
            println!("\n💡 Tip: Run 'langstar config create' to create one");
            return Ok(());
        }

        // Try to load and parse the config file
        let config = match Config::load() {
            Ok(c) => c,
            Err(e) => {
                println!("  ✗ Config file validation FAILED");
                println!("\nError: {}", e);
                return Err(e);
            }
        };

        // Validate output_format
        let valid_formats = ["json", "table"];
        if !valid_formats.contains(&config.output_format.as_str()) {
            println!("  ✗ Invalid output_format: {}", config.output_format);
            println!("    Valid values: json, table");
            return Err(CliError::Config(format!(
                "Invalid output_format: {}. Must be 'json' or 'table'",
                config.output_format
            )));
        }

        // Validate timezone
        if let Err(e) = ConfiguredTimezone::parse(&config.timezone) {
            println!("  ✗ Invalid timezone: {}", config.timezone);
            println!("    Error: {}", e);
            return Err(CliError::Config(format!(
                "Invalid timezone: {}. {}",
                config.timezone, e
            )));
        }

        // Check for API key
        if config.langsmith_api_key.is_none() {
            println!("  ⚠ LangSmith API key is not configured");
            println!("    Set LANGSMITH_API_KEY environment variable or add to config file");
        }

        println!("\n✓ Configuration is valid");
        println!("\nConfiguration summary:");
        println!("  Output format: {}", config.output_format);
        println!("  Timezone: {}", config.timezone);
        println!(
            "  API key: {}",
            if config.langsmith_api_key.is_some() {
                "configured"
            } else {
                "not configured"
            }
        );

        Ok(())
    }

    fn handle_env() -> Result<()> {
        println!("Config Key to Environment Variable Mapping");
        println!("==========================================\n");

        // Define the mappings
        let mappings = vec![
            (
                "langsmith_api_key",
                "LANGSMITH_API_KEY",
                "API key for LangSmith and LangGraph services",
            ),
            (
                "organization_id",
                "LANGSMITH_ORGANIZATION_ID",
                "Organization ID for scoping operations",
            ),
            (
                "workspace_id",
                "LANGSMITH_WORKSPACE_ID",
                "Workspace ID for narrower scoping",
            ),
            (
                "github_integration_id",
                "LANGGRAPH_GITHUB_INTEGRATION_ID",
                "GitHub integration ID for deployments",
            ),
            (
                "output_format",
                "LANGSTAR_OUTPUT_FORMAT",
                "Default output format (json or table)",
            ),
            (
                "timezone",
                "LANGSTAR_TIMEZONE",
                "Timezone for timestamp display",
            ),
            (
                "hide_workspace_and_org_id_message",
                "LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE",
                "Suppress workspace/org ID warnings",
            ),
        ];

        // Display the table
        println!(
            "{:<35} {:<45} Description",
            "Config File Key", "Environment Variable"
        );
        println!("{}", "-".repeat(110));

        for (config_key, env_var, description) in mappings {
            println!("{:<35} {:<45} {}", config_key, env_var, description);
        }

        println!("\nNotes:");
        println!("  • Environment variables take precedence over config file values");
        println!("  • Most environment variables use LANGSMITH_ or LANGSTAR_ prefixes");
        println!(
            "  • Config file keys use snake_case, environment variables use SCREAMING_SNAKE_CASE"
        );

        println!("\nRelated commands:");
        println!("  langstar config show      - Show current configuration with sources");
        println!("  langstar config create    - Create a config file with all options");
        println!("  langstar config validate  - Validate the config file");

        Ok(())
    }

    fn handle_hide_workspace_and_org_id_message(action: &SettingAction) -> Result<()> {
        match &action.action {
            Some(SettingCommand::Set { value }) => {
                // Parse boolean value - accepts various formats but normalizes to "true"/"false"
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

    fn handle_output_format(action: &SettingAction) -> Result<()> {
        match &action.action {
            Some(SettingCommand::Set { value }) => {
                // Normalize to lowercase for consistency
                let normalized_value = value.to_lowercase();
                let valid_formats = ["json", "table"];
                if !valid_formats.contains(&normalized_value.as_str()) {
                    return Err(CliError::Config(format!(
                        "Invalid output format '{}'. Use: json or table",
                        value
                    )));
                }

                Self::set_config_value("output_format", &normalized_value)?;
                println!("✓ Set output_format = {}", normalized_value);
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

    fn handle_timezone(action: &SettingAction) -> Result<()> {
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
            // Create empty config file - toml_edit handles empty documents
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
