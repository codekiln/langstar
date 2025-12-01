use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::LangchainClient;
use langstar_sdk::playground_settings::{
    ListPlaygroundSettingsParams, PlaygroundSettingsCreateRequest, PlaygroundSettingsResponse,
    PlaygroundSettingsUpdateRequest,
};
use serde_json::Value;
use std::fs;
use tabled::Tabled;
use uuid::Uuid;

/// Commands for managing LangSmith model configurations (playground settings)
#[derive(Debug, Subcommand)]
pub enum ModelConfigCommands {
    /// List all model configurations
    List {
        /// Maximum number of items to return
        #[arg(short, long, default_value = "20")]
        limit: i64,

        /// Number of items to skip
        #[arg(short, long, default_value = "0")]
        offset: i64,
    },

    /// Get details of a specific model configuration
    Get {
        /// Configuration ID (UUID)
        id: Uuid,
    },

    /// Create a new model configuration
    Create {
        /// Path to JSON file containing configuration
        #[arg(short, long)]
        file: std::path::PathBuf,
    },

    /// Update an existing model configuration
    Update {
        /// Configuration ID (UUID)
        id: Uuid,

        /// Path to JSON file containing updates
        #[arg(short, long, conflicts_with_all = ["name", "description"])]
        file: Option<std::path::PathBuf>,

        /// Update only the name
        #[arg(long, conflicts_with = "file")]
        name: Option<String>,

        /// Update only the description
        #[arg(long, conflicts_with = "file")]
        description: Option<String>,
    },

    /// Delete a model configuration
    Delete {
        /// Configuration ID (UUID)
        id: Uuid,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

/// Simplified model config info for table display
#[derive(Debug, Tabled)]
struct ModelConfigRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PROVIDER")]
    provider: String,
    #[tabled(rename = "MODEL")]
    model: String,
}

impl ModelConfigRow {
    fn from_response(response: &PlaygroundSettingsResponse) -> Self {
        let (provider, model) = extract_provider_and_model(&response.settings);

        Self {
            id: response.id.to_string(),
            name: response.name.clone().unwrap_or_else(|| "-".to_string()),
            provider,
            model,
        }
    }
}

/// Extract provider and model from settings JSON
fn extract_provider_and_model(settings: &Value) -> (String, String) {
    let provider = settings
        .get("id")
        .and_then(|id| id.as_array())
        .and_then(|arr| {
            // Try to get provider from id array (e.g., ["langchain", "chat_models", "anthropic", "ChatAnthropic"])
            arr.get(2).and_then(|v| v.as_str())
        })
        .unwrap_or("-")
        .to_string();

    let model = settings
        .get("kwargs")
        .and_then(|kwargs| kwargs.get("model"))
        .and_then(|m| m.as_str())
        .unwrap_or("-")
        .to_string();

    (provider, model)
}

impl ModelConfigCommands {
    /// Execute the model-config command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;
        let formatter = OutputFormatter::new(format);

        match self {
            ModelConfigCommands::List { limit, offset } => {
                let params = ListPlaygroundSettingsParams {
                    limit: Some(*limit),
                    offset: Some(*offset),
                };

                let configs = client.list_playground_settings(params).await?;

                if configs.is_empty() {
                    println!("No model configurations found");
                    return Ok(());
                }

                match format {
                    OutputFormat::Json => {
                        formatter.print(&configs)?;
                    }
                    OutputFormat::Table => {
                        let rows: Vec<ModelConfigRow> =
                            configs.iter().map(ModelConfigRow::from_response).collect();
                        formatter.print_table(&rows)?;
                    }
                }

                Ok(())
            }

            ModelConfigCommands::Get { id } => {
                // Get a specific config by listing and filtering
                // TODO: Add dedicated get_playground_settings method to SDK
                let params = ListPlaygroundSettingsParams {
                    limit: Some(100),
                    offset: Some(0),
                };
                let configs = client.list_playground_settings(params).await?;

                let config = configs
                    .into_iter()
                    .find(|c| c.id == *id)
                    .ok_or_else(|| {
                        crate::error::CliError::Other(anyhow::anyhow!(
                            "Model configuration with ID {} not found",
                            id
                        ))
                    })?;

                match format {
                    OutputFormat::Json => {
                        formatter.print(&config)?;
                    }
                    OutputFormat::Table => {
                        let row = ModelConfigRow::from_response(&config);
                        formatter.print_table(&[row])?;
                    }
                }

                Ok(())
            }

            ModelConfigCommands::Create { file } => {
                let content = fs::read_to_string(file)?;
                let request: PlaygroundSettingsCreateRequest = serde_json::from_str(&content)?;

                let response = client.create_playground_settings(request).await?;

                match format {
                    OutputFormat::Json => {
                        formatter.print(&response)?;
                    }
                    OutputFormat::Table => {
                        let row = ModelConfigRow::from_response(&response);
                        formatter.print_table(&[row])?;
                    }
                }

                eprintln!("✓ Model configuration created: {}", response.id);
                Ok(())
            }

            ModelConfigCommands::Update {
                id,
                file,
                name,
                description,
            } => {
                let request = if let Some(file_path) = file {
                    // Load full update from file
                    let content = fs::read_to_string(file_path)?;
                    serde_json::from_str(&content)?
                } else {
                    // Build partial update from flags
                    PlaygroundSettingsUpdateRequest {
                        name: name.clone(),
                        description: description.clone(),
                        settings: None,
                        options: None,
                    }
                };

                let response = client.update_playground_settings(*id, request).await?;

                match format {
                    OutputFormat::Json => {
                        formatter.print(&response)?;
                    }
                    OutputFormat::Table => {
                        let row = ModelConfigRow::from_response(&response);
                        formatter.print_table(&[row])?;
                    }
                }

                eprintln!("✓ Model configuration updated: {}", response.id);
                Ok(())
            }

            ModelConfigCommands::Delete { id, yes } => {
                if !yes {
                    // Prompt for confirmation
                    eprint!("Delete model configuration {}? [y/N]: ", id);
                    use std::io::{self, BufRead};
                    let stdin = io::stdin();
                    let mut line = String::new();
                    stdin.lock().read_line(&mut line)?;

                    let answer = line.trim().to_lowercase();
                    if answer != "y" && answer != "yes" {
                        println!("Cancelled");
                        return Ok(());
                    }
                }

                client.delete_playground_settings(*id).await?;
                eprintln!("✓ Model configuration deleted: {}", id);
                Ok(())
            }
        }
    }
}
