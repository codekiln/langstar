use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{LangchainClient, Prompt};
use tabled::Tabled;

/// Commands for interacting with LangSmith Prompts
#[derive(Debug, Subcommand)]
pub enum PromptCommands {
    /// List all prompts
    List {
        /// Maximum number of prompts to return
        #[arg(short, long, default_value = "20")]
        limit: u32,

        /// Number of prompts to skip
        #[arg(short, long, default_value = "0")]
        offset: u32,
    },

    /// Get details of a specific prompt
    Get {
        /// Prompt handle (e.g., "owner/prompt-name")
        handle: String,
    },

    /// Search for prompts
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },
}

/// Simplified prompt info for table display
#[derive(Debug, Tabled)]
struct PromptRow {
    #[tabled(rename = "Handle")]
    repo_handle: String,
    #[tabled(rename = "Likes")]
    num_likes: u32,
    #[tabled(rename = "Downloads")]
    num_downloads: u32,
    #[tabled(rename = "Public")]
    is_public: String,
    #[tabled(rename = "Description")]
    description: String,
}

impl From<&Prompt> for PromptRow {
    fn from(prompt: &Prompt) -> Self {
        Self {
            repo_handle: prompt.repo_handle.clone(),
            num_likes: prompt.num_likes,
            num_downloads: prompt.num_downloads,
            is_public: if prompt.is_public { "yes" } else { "no" }.to_string(),
            description: prompt
                .description
                .as_ref()
                .map(|d| {
                    if d.len() > 50 {
                        format!("{}...", &d[..47])
                    } else {
                        d.clone()
                    }
                })
                .unwrap_or_else(|| "".to_string()),
        }
    }
}

impl PromptCommands {
    /// Execute the prompt command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;
        let formatter = OutputFormatter::new(format);

        match self {
            PromptCommands::List { limit, offset } => {
                formatter.info(&format!("Fetching prompts (limit: {}, offset: {})...", limit, offset));

                let prompts = client.prompts().list(Some(*limit), Some(*offset)).await?;

                if format == OutputFormat::Json {
                    formatter.print(&prompts)?;
                } else {
                    let rows: Vec<PromptRow> = prompts.iter().map(PromptRow::from).collect();
                    formatter.print_table(&rows)?;
                    println!("\nFound {} prompts", prompts.len());
                }
            }

            PromptCommands::Get { handle } => {
                formatter.info(&format!("Fetching prompt '{}'...", handle));

                let prompt = client.prompts().get(handle).await?;

                if format == OutputFormat::Json {
                    formatter.print(&prompt)?;
                } else {
                    println!("\n{}", "Prompt Details".to_uppercase());
                    println!("─────────────────────────────────────────");
                    println!("Handle:      {}", prompt.repo_handle);
                    println!("Likes:       {}", prompt.num_likes);
                    println!("Downloads:   {}", prompt.num_downloads);
                    println!("Public:      {}", if prompt.is_public { "yes" } else { "no" });
                    if let Some(desc) = &prompt.description {
                        println!("Description: {}", desc);
                    }
                    if let Some(created) = &prompt.created_at {
                        println!("Created:     {}", created);
                    }
                    if let Some(updated) = &prompt.updated_at {
                        println!("Updated:     {}", updated);
                    }
                }
            }

            PromptCommands::Search { query, limit } => {
                formatter.info(&format!("Searching for '{}'...", query));

                let prompts = client.prompts().search(query, Some(*limit)).await?;

                if format == OutputFormat::Json {
                    formatter.print(&prompts)?;
                } else {
                    let rows: Vec<PromptRow> = prompts.iter().map(PromptRow::from).collect();
                    formatter.print_table(&rows)?;
                    println!("\nFound {} prompts", prompts.len());
                }
            }
        }

        Ok(())
    }
}
