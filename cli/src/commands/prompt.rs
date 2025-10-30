use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{CommitRequest, LangchainClient, Prompt};
use serde_json::json;
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

    /// Push/create a prompt in PromptHub
    Push {
        /// Owner of the prompt (username or organization)
        #[arg(short, long)]
        owner: String,

        /// Prompt repository name
        #[arg(short, long)]
        repo: String,

        /// Prompt template text
        #[arg(short, long)]
        template: String,

        /// Input variables (comma-separated, e.g., "context,question")
        #[arg(short, long)]
        input_variables: Option<String>,

        /// Template format (default: f-string)
        #[arg(long, default_value = "f-string")]
        template_format: String,
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
                .unwrap_or_default(),
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

            PromptCommands::Push {
                owner,
                repo,
                template,
                input_variables,
                template_format,
            } => {
                // Get organization info and set organization ID
                formatter.info("Fetching organization information...");
                let mut client = client;
                match client.get_current_organization().await {
                    Ok(org) => {
                        if let Some(org_id) = &org.id {
                            println!("✓ Organization: {}", org.display_name.as_deref().unwrap_or("Unknown"));
                            println!("  ID: {}", org_id);
                            client = client.with_organization_id(org_id.clone());
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠ Warning: Could not fetch organization: {}", e);
                        eprintln!("  Proceeding without X-Organization-Id header");
                    }
                }

                // Try to create repository if it doesn't exist
                let repo_handle = format!("{}/{}", owner, repo);
                formatter.info(&format!("Checking if repository {} exists...", repo_handle));

                match client.prompts().get(&repo_handle).await {
                    Ok(_) => {
                        println!("✓ Repository exists");
                    }
                    Err(_) => {
                        formatter.info(&format!("Repository not found, creating {}...", repo_handle));
                        match client.prompts().create_repo(
                            &repo_handle,
                            Some("Created via langstar CLI".to_string()),
                            None,
                            false, // Private by default
                            Some(vec!["cli".to_string(), "langstar".to_string()]),
                        ).await {
                            Ok(_) => {
                                println!("✓ Repository created successfully");
                            }
                            Err(e) => {
                                eprintln!("⚠ Warning: Could not create repository: {}", e);
                                eprintln!("  Will attempt to push anyway...");
                            }
                        }
                    }
                }

                formatter.info(&format!("Pushing prompt to {}/{}...", owner, repo));

                // Parse input variables
                let vars: Vec<String> = if let Some(vars_str) = input_variables {
                    vars_str.split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    vec![]
                };

                // Create commit request
                let commit_request = CommitRequest {
                    manifest: json!({
                        "type": "prompt",
                        "template": template,
                        "input_variables": vars,
                        "template_format": template_format
                    }),
                    parent_commit: None,
                    example_run_ids: None,
                };

                // Push the commit
                match client.prompts().push(owner, repo, &commit_request).await {
                    Ok(response) => {
                        if format == OutputFormat::Json {
                            formatter.print(&response)?;
                        } else {
                            println!("\n✓ Prompt commit pushed successfully!");
                            println!("  Repository: {}/{}", owner, repo);
                            println!("  Commit hash: {}", response.commit_hash);
                            if let Some(url) = &response.url {
                                println!("  URL: {}", url);
                            }
                        }
                    }
                    Err(e) => {
                        return Err(crate::error::CliError::Sdk(e));
                    }
                }
            }
        }

        Ok(())
    }
}
