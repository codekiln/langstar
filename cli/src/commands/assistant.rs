use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{Assistant, CreateAssistantRequest, LangchainClient, UpdateAssistantRequest};
use serde_json::json;
use tabled::Tabled;

/// Commands for interacting with LangGraph Assistants
#[derive(Debug, Subcommand)]
pub enum AssistantCommands {
    /// List all assistants
    List {
        /// Maximum number of assistants to return
        #[arg(short, long, default_value = "20")]
        limit: u32,

        /// Number of assistants to skip
        #[arg(short, long, default_value = "0")]
        offset: u32,
    },

    /// Search for assistants by name
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },

    /// Get details of a specific assistant
    Get {
        /// Assistant ID
        assistant_id: String,
    },

    /// Create a new assistant
    Create {
        /// Graph ID to base the assistant on
        #[arg(short, long)]
        graph_id: String,

        /// Name for the assistant
        #[arg(short, long)]
        name: String,

        /// Configuration JSON file path
        #[arg(long)]
        config_file: Option<String>,

        /// Configuration JSON (inline)
        #[arg(long, conflicts_with = "config_file")]
        config: Option<String>,
    },

    /// Update an existing assistant
    Update {
        /// Assistant ID to update
        assistant_id: String,

        /// Updated name
        #[arg(short, long)]
        name: Option<String>,

        /// Configuration JSON file path
        #[arg(long)]
        config_file: Option<String>,

        /// Configuration JSON (inline)
        #[arg(long, conflicts_with = "config_file")]
        config: Option<String>,
    },

    /// Delete an assistant
    Delete {
        /// Assistant ID to delete
        assistant_id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

/// Simplified assistant info for table display
#[derive(Debug, Tabled)]
struct AssistantRow {
    #[tabled(rename = "ID")]
    assistant_id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Graph ID")]
    graph_id: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

impl From<&Assistant> for AssistantRow {
    fn from(assistant: &Assistant) -> Self {
        Self {
            assistant_id: if assistant.assistant_id.len() > 16 {
                format!("{}...", &assistant.assistant_id[..13])
            } else {
                assistant.assistant_id.clone()
            },
            name: if assistant.name.len() > 30 {
                format!("{}...", &assistant.name[..27])
            } else {
                assistant.name.clone()
            },
            graph_id: if assistant.graph_id.len() > 16 {
                format!("{}...", &assistant.graph_id[..13])
            } else {
                assistant.graph_id.clone()
            },
            created_at: assistant
                .created_at
                .as_ref()
                .and_then(|s| s.split('T').next())
                .unwrap_or("N/A")
                .to_string(),
        }
    }
}

impl AssistantCommands {
    /// Execute the assistant command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;
        let formatter = OutputFormatter::new(format);

        match self {
            AssistantCommands::List { limit, offset } => {
                formatter.info(&format!(
                    "Fetching assistants (limit: {}, offset: {})...",
                    limit, offset
                ));

                let assistants = client
                    .assistants()
                    .list(Some(*limit), Some(*offset))
                    .await?;

                if format == OutputFormat::Json {
                    formatter.print(&json!({ "assistants": assistants }))?;
                } else {
                    let rows: Vec<AssistantRow> = assistants.iter().map(|a| a.into()).collect();
                    formatter.print_table(&rows)?;

                    if assistants.is_empty() {
                        eprintln!("\nℹ No assistants found");
                    } else {
                        eprintln!(
                            "\nℹ Found {} assistant{}",
                            assistants.len(),
                            if assistants.len() == 1 { "" } else { "s" }
                        );
                    }
                }

                Ok(())
            }

            AssistantCommands::Search { query, limit } => {
                formatter.info(&format!("Searching for assistants matching '{}'...", query));

                let assistants = client.assistants().search(query, Some(*limit)).await?;

                if format == OutputFormat::Json {
                    formatter.print(&json!({ "assistants": assistants }))?;
                } else {
                    let rows: Vec<AssistantRow> = assistants.iter().map(|a| a.into()).collect();
                    formatter.print_table(&rows)?;

                    if assistants.is_empty() {
                        eprintln!("\nℹ No assistants found matching '{}'", query);
                    } else {
                        eprintln!(
                            "\nℹ Found {} matching assistant{}",
                            assistants.len(),
                            if assistants.len() == 1 { "" } else { "s" }
                        );
                    }
                }

                Ok(())
            }

            AssistantCommands::Get { assistant_id } => {
                formatter.info(&format!("Fetching assistant '{}'...", assistant_id));

                let assistant = client.assistants().get(assistant_id).await?;

                if format == OutputFormat::Json {
                    formatter.print(&assistant)?;
                } else {
                    let row = AssistantRow::from(&assistant);
                    formatter.print_table(&[row])?;

                    // Show additional details in table mode
                    eprintln!("\nℹ Assistant Details:");
                    eprintln!("  ID: {}", assistant.assistant_id);
                    eprintln!("  Name: {}", assistant.name);
                    eprintln!("  Graph ID: {}", assistant.graph_id);
                    if let Some(created) = &assistant.created_at {
                        eprintln!("  Created: {}", created);
                    }
                    if let Some(updated) = &assistant.updated_at {
                        eprintln!("  Updated: {}", updated);
                    }
                    if let Some(config) = &assistant.config {
                        eprintln!("  Config: {}", serde_json::to_string_pretty(config)?);
                    }
                    if let Some(metadata) = &assistant.metadata {
                        eprintln!("  Metadata: {}", serde_json::to_string_pretty(metadata)?);
                    }
                }

                Ok(())
            }

            AssistantCommands::Create {
                graph_id,
                name,
                config_file,
                config,
            } => {
                formatter.info(&format!("Creating assistant '{}'...", name));

                // Parse config from file or inline JSON
                let config_value = if let Some(file_path) = config_file {
                    let content = std::fs::read_to_string(file_path)?;
                    Some(serde_json::from_str(&content)?)
                } else if let Some(json_str) = config {
                    Some(serde_json::from_str(json_str)?)
                } else {
                    None
                };

                let request = CreateAssistantRequest {
                    graph_id: graph_id.clone(),
                    name: name.clone(),
                    config: config_value,
                    metadata: None,
                };

                let assistant = client.assistants().create(&request).await?;

                if format == OutputFormat::Json {
                    formatter.print(&assistant)?;
                } else {
                    formatter.print_table(&[AssistantRow::from(&assistant)])?;
                    eprintln!("\n✓ Successfully created assistant '{}'", assistant.name);
                    eprintln!("  ID: {}", assistant.assistant_id);
                }

                Ok(())
            }

            AssistantCommands::Update {
                assistant_id,
                name,
                config_file,
                config,
            } => {
                formatter.info(&format!("Updating assistant '{}'...", assistant_id));

                // Parse config from file or inline JSON
                let config_value = if let Some(file_path) = config_file {
                    let content = std::fs::read_to_string(file_path)?;
                    Some(serde_json::from_str(&content)?)
                } else if let Some(json_str) = config {
                    Some(serde_json::from_str(json_str)?)
                } else {
                    None
                };

                let request = UpdateAssistantRequest {
                    name: name.clone(),
                    config: config_value,
                    metadata: None,
                };

                let assistant = client.assistants().update(assistant_id, &request).await?;

                if format == OutputFormat::Json {
                    formatter.print(&assistant)?;
                } else {
                    formatter.print_table(&[AssistantRow::from(&assistant)])?;
                    eprintln!("\n✓ Successfully updated assistant '{}'", assistant.name);
                }

                Ok(())
            }

            AssistantCommands::Delete {
                assistant_id,
                force,
            } => {
                if !force {
                    eprintln!(
                        "⚠ This will permanently delete assistant '{}'",
                        assistant_id
                    );
                    eprint!("Continue? [y/N]: ");

                    use std::io::{self, Write};
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;

                    let input = input.trim().to_lowercase();
                    if input != "y" && input != "yes" {
                        eprintln!("❌ Cancelled");
                        return Ok(());
                    }
                }

                formatter.info(&format!("Deleting assistant '{}'...", assistant_id));

                client.assistants().delete(assistant_id).await?;

                if format == OutputFormat::Json {
                    formatter.print(&json!({ "deleted": assistant_id }))?;
                } else {
                    eprintln!("✓ Successfully deleted assistant '{}'", assistant_id);
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_row_truncation() {
        let assistant = Assistant {
            assistant_id: "very-long-assistant-id-that-should-be-truncated".to_string(),
            graph_id: "graph-123".to_string(),
            name: "Test Assistant".to_string(),
            config: None,
            metadata: None,
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            updated_at: None,
        };

        let row = AssistantRow::from(&assistant);
        assert!(row.assistant_id.len() <= 16);
        assert_eq!(row.name, "Test Assistant");
    }
}
