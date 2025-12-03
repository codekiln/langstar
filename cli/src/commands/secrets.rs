use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::LangchainClient;
use langstar_sdk::secrets::SecretUpsert;
use rpassword::read_password;
use std::fs;
use std::io::{self, Read};
use tabled::Tabled;

/// Commands for managing LangSmith workspace secrets
#[derive(Debug, Subcommand)]
pub enum SecretsCommands {
    /// List all workspace secret keys (values are never displayed)
    List {
        /// Output format (json or table)
        #[arg(short = 'f', long)]
        format: Option<String>,
    },

    /// Set or update a workspace secret
    Set {
        /// Secret key name (e.g., "ANTHROPIC_API_KEY")
        key: String,

        /// Read secret value from file
        #[arg(long, value_name = "FILE", conflicts_with_all = ["from_env", "interactive"])]
        from_file: Option<std::path::PathBuf>,

        /// Read secret value from environment variable
        #[arg(long, value_name = "VAR", conflicts_with_all = ["from_file", "interactive"])]
        from_env: Option<String>,

        /// Prompt for secret value interactively (masked input)
        #[arg(long, conflicts_with_all = ["from_file", "from_env"])]
        interactive: bool,

        /// Output format (json or table)
        #[arg(short = 'f', long)]
        format: Option<String>,
    },

    /// Delete a workspace secret
    Delete {
        /// Secret key name to delete
        key: String,

        /// Output format (json or table)
        #[arg(short = 'f', long)]
        format: Option<String>,
    },
}

/// Table row for displaying secret keys
#[derive(Debug, Tabled)]
struct SecretKeyRow {
    #[tabled(rename = "Key")]
    key: String,
}

impl SecretsCommands {
    /// Execute the secrets command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;

        match self {
            SecretsCommands::List { format: cmd_format } => {
                let format = if let Some(fmt_str) = cmd_format {
                    OutputFormat::from_str(fmt_str)?
                } else {
                    format
                };
                let formatter = OutputFormatter::new(format);

                formatter.info("Fetching workspace secrets...");

                let keys = client.list_workspace_secrets().await?;

                if format == OutputFormat::Json {
                    formatter.print(&keys)?;
                } else {
                    let rows: Vec<SecretKeyRow> = keys
                        .iter()
                        .map(|k| SecretKeyRow { key: k.key.clone() })
                        .collect();
                    formatter.print_table(&rows)?;
                    println!("\nFound {} secrets", keys.len());
                }

                Ok(())
            }

            SecretsCommands::Set {
                key,
                from_file,
                from_env,
                interactive,
                format: cmd_format,
            } => {
                let format = if let Some(fmt_str) = cmd_format {
                    OutputFormat::from_str(fmt_str)?
                } else {
                    format
                };
                let formatter = OutputFormatter::new(format);

                // Security: Determine how to read the secret value
                let value = if let Some(file_path) = from_file {
                    // Read from file
                    fs::read_to_string(file_path)
                        .map_err(crate::error::CliError::Io)?
                        .trim()
                        .to_string()
                } else if let Some(env_var) = from_env {
                    // Read from environment variable
                    std::env::var(env_var).map_err(|_| {
                        crate::error::CliError::Config(format!(
                            "Environment variable '{}' not found",
                            env_var
                        ))
                    })?
                } else if *interactive {
                    // Interactive prompt with masked input
                    eprintln!("Enter secret value for '{}': ", key);
                    read_password().map_err(crate::error::CliError::Io)?
                } else {
                    // Try reading from stdin
                    let mut buffer = String::new();
                    io::stdin()
                        .read_to_string(&mut buffer)
                        .map_err(crate::error::CliError::Io)?;

                    if buffer.is_empty() {
                        return Err(crate::error::CliError::Config(
                            "No secret value provided. Use --from-file, --from-env, --interactive, or provide value via stdin".to_string()
                        ));
                    }

                    buffer.trim().to_string()
                };

                // Security: Validate that value is not empty
                if value.is_empty() {
                    return Err(crate::error::CliError::Config(
                        "Secret value cannot be empty".to_string(),
                    ));
                }

                formatter.info(&format!("Setting secret '{}'...", key));

                let secrets = vec![SecretUpsert::set(key, value)];
                client.upsert_workspace_secrets(secrets).await?;

                // Security: Never output the secret value
                if format == OutputFormat::Json {
                    formatter.print(&serde_json::json!({
                        "status": "success",
                        "key": key,
                        "message": format!("Secret '{}' set successfully", key)
                    }))?;
                } else {
                    println!("✓ Secret '{}' set successfully", key);
                }

                Ok(())
            }

            SecretsCommands::Delete {
                key,
                format: cmd_format,
            } => {
                let format = if let Some(fmt_str) = cmd_format {
                    OutputFormat::from_str(fmt_str)?
                } else {
                    format
                };
                let formatter = OutputFormatter::new(format);

                formatter.info(&format!("Deleting secret '{}'...", key));

                client.delete_workspace_secret(key).await?;

                if format == OutputFormat::Json {
                    formatter.print(&serde_json::json!({
                        "status": "success",
                        "key": key,
                        "message": format!("Secret '{}' deleted successfully", key)
                    }))?;
                } else {
                    println!("✓ Secret '{}' deleted successfully", key);
                }

                Ok(())
            }
        }
    }
}
