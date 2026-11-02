use crate::config::Config;
use crate::error::Result;
use crate::output::{ColumnMetadata, OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::prompts::{
    LcJson, MessagePromptTemplateKwargs, PromptTemplateKwargs, StructuredOutputKwargs,
    StructuredPrompt, validate_json_schema, validate_method,
};
use langstar_sdk::{CommitRequest, LangchainClient, Prompt, Visibility};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tabled::Tabled;
use uuid::Uuid;

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

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,

        /// Show only public prompts (default: private when scoped, any when not scoped)
        #[arg(long)]
        public: bool,

        /// Show all columns (likes, public) - by default only essential columns are shown
        #[arg(long)]
        full: bool,

        /// Select specific columns for text output (comma-separated)
        /// Available: handle, likes, downloads, public, description, created_at
        #[arg(long, value_delimiter = ',')]
        columns: Option<Vec<String>>,

        /// Show available columns for text output
        #[arg(long)]
        show_columns: bool,
    },

    /// Get details of a specific prompt
    Get {
        /// Prompt handle (e.g., "owner/prompt-name") or UUID
        ///
        /// Can be:
        /// - Full handle: "langchain-ai/rag-prompt"
        /// - Short handle: "prompt-name" (prepends "-/" for private prompts)
        /// - UUID: "8fc72dfc-6213-4048-b025-0156b1b735ff"
        handle: String,

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,
    },

    /// Search for prompts
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: u32,

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,

        /// Show only public prompts (default: private when scoped, any when not scoped)
        #[arg(long)]
        public: bool,

        /// Show all columns (likes, public) - by default only essential columns are shown
        #[arg(long)]
        full: bool,
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

        /// Path to JSON Schema file for structured output
        #[arg(long, value_name = "FILE")]
        schema: Option<std::path::PathBuf>,

        /// Structured output method: json_schema or function_calling
        #[arg(long, default_value = "json_schema")]
        schema_method: String,

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,
    },

    /// Pull a prompt from the PromptHub
    Pull {
        /// Prompt handle (e.g., "owner/prompt-name")
        handle: String,

        /// Commit hash or tag (default: "latest")
        #[arg(long, default_value = "latest")]
        commit: String,

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,
    },

    /// Write one prompt commit to a JSON file that `prompt import` can read
    ///
    /// The file holds the repo details, the commit's manifest with its model,
    /// the tags that point at the commit, and the names of the secrets the model
    /// reads. Export refuses to write a file that would hold a credential, and
    /// names the path of each string the check flagged.
    Export {
        /// Prompt handle: "my-prompt" or "-/my-prompt" for a prompt in your
        /// workspace, or "owner/my-prompt"
        #[arg(allow_hyphen_values = true)]
        handle: String,

        /// Commit hash or tag to export (default: the latest commit)
        #[arg(long)]
        commit: Option<String>,

        /// File to write (default: print the file to stdout)
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,

        /// JSON Pointer of a flagged string you have read and know holds no
        /// credential, such as /commit/manifest/kwargs/first/kwargs/template.
        /// Give it once for each path.
        #[arg(long = "not-a-secret", value_name = "JSON_POINTER")]
        not_a_secret: Vec<String>,

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,
    },
}

/// Compact prompt info for table display (default view)
/// Shows only essential columns: Handle, Downloads, Description
#[derive(Debug, Tabled)]
struct CompactPromptRow {
    #[tabled(rename = "Handle")]
    repo_handle: String,
    #[tabled(rename = "Downloads")]
    num_downloads: u32,
    #[tabled(rename = "Description")]
    description: String,
}

/// Full prompt info for table display (--full view)
/// Shows all columns including Likes and Public status
#[derive(Debug, Tabled)]
struct FullPromptRow {
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

/// Available columns for prompt list text output
const PROMPT_COLUMNS: &[&str] = &[
    "handle",
    "likes",
    "downloads",
    "public",
    "description",
    "created_at",
];

/// Implementation of ColumnMetadata for Prompt
///
/// Enables tab-separated text output with column selection
impl ColumnMetadata for Prompt {
    fn available_columns() -> Vec<&'static str> {
        PROMPT_COLUMNS.to_vec()
    }

    fn render_tsv(&self, columns: &[String]) -> String {
        columns
            .iter()
            .map(|col| match col.as_str() {
                "handle" => self
                    .full_name
                    .clone()
                    .or_else(|| {
                        // Fallback: construct from owner + repo_handle if full_name is missing
                        self.owner
                            .as_ref()
                            .map(|owner| format!("{}/{}", owner, self.repo_handle))
                    })
                    .unwrap_or_else(|| self.repo_handle.clone()),
                "likes" => self.num_likes.to_string(),
                "downloads" => self.num_downloads.to_string(),
                "public" => self.is_public.to_string(),
                "description" => self
                    .description
                    .clone()
                    .unwrap_or_default()
                    .replace(['\t', '\n'], " "),
                "created_at" => self
                    .created_at
                    .clone()
                    .unwrap_or_default()
                    .replace(['\t', '\n'], " "),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join("\t")
    }
}

/// Truncate description to fit table display
fn truncate_description(desc: Option<&String>, max_len: usize) -> String {
    desc.map(|d| {
        if d.len() > max_len {
            format!("{}...", &d[..max_len.saturating_sub(3)])
        } else {
            d.clone()
        }
    })
    .unwrap_or_default()
}

impl From<&Prompt> for CompactPromptRow {
    fn from(prompt: &Prompt) -> Self {
        let handle = prompt
            .full_name
            .clone()
            .or_else(|| {
                prompt
                    .owner
                    .as_ref()
                    .map(|owner| format!("{}/{}", owner, prompt.repo_handle))
            })
            .unwrap_or_else(|| prompt.repo_handle.clone());

        Self {
            repo_handle: handle,
            num_downloads: prompt.num_downloads,
            description: truncate_description(prompt.description.as_ref(), 40),
        }
    }
}

impl From<&Prompt> for FullPromptRow {
    fn from(prompt: &Prompt) -> Self {
        let handle = prompt
            .full_name
            .clone()
            .or_else(|| {
                prompt
                    .owner
                    .as_ref()
                    .map(|owner| format!("{}/{}", owner, prompt.repo_handle))
            })
            .unwrap_or_else(|| prompt.repo_handle.clone());

        Self {
            repo_handle: handle,
            num_likes: prompt.num_likes,
            num_downloads: prompt.num_downloads,
            is_public: if prompt.is_public { "yes" } else { "no" }.to_string(),
            description: truncate_description(prompt.description.as_ref(), 50),
        }
    }
}

impl PromptCommands {
    /// Apply organization and workspace ID overrides to the client
    ///
    /// Precedence order: CLI flags → config (which includes env vars)
    fn apply_scoping(
        client: LangchainClient,
        flag_org_id: &Option<String>,
        flag_workspace_id: &Option<String>,
        config: &Config,
    ) -> LangchainClient {
        let mut client = client;

        // Warn if both org and workspace IDs are specified
        if !config.hide_workspace_and_org_id_message
            && flag_org_id.is_some()
            && flag_workspace_id.is_some()
        {
            use crate::commands::config::messages;
            eprintln!("⚠ Warning: Both organization and workspace IDs specified");
            eprintln!("  → Using workspace scope (narrower scope takes precedence)");
            eprintln!("  {}", messages::SUPPRESS_WORKSPACE_ORG_WARNING);
        }

        // Apply organization ID if provided via flag (overrides config/env)
        if let Some(org_id) = flag_org_id {
            client = client.with_organization_id(org_id.clone());
        }

        // Apply workspace ID if provided via flag (overrides config/env)
        if let Some(workspace_id) = flag_workspace_id {
            client = client.with_workspace_id(workspace_id.clone());
        }

        // Also warn if client now has both (from config/env combination)
        if !config.hide_workspace_and_org_id_message
            && client.organization_id().is_some()
            && client.workspace_id().is_some()
            && flag_org_id.is_none()
            && flag_workspace_id.is_none()
        {
            use crate::commands::config::messages;
            eprintln!("ℹ Info: Both organization and workspace IDs configured");
            eprintln!("  → Using workspace scope (narrower scope takes precedence)");
            eprintln!("  {}", messages::SUPPRESS_WORKSPACE_ORG_WARNING);
        }

        client
    }

    /// Print scope information for verbose output
    fn print_scope_info(client: &LangchainClient, visibility: Visibility) {
        let scope = if let Some(workspace_id) = client.workspace_id() {
            let truncated = if workspace_id.len() >= 8 {
                &workspace_id[..8]
            } else {
                workspace_id
            };
            format!("Workspace ({})", truncated)
        } else if let Some(org_id) = client.organization_id() {
            let truncated = if org_id.len() >= 8 {
                &org_id[..8]
            } else {
                org_id
            };
            format!("Organization ({})", truncated)
        } else {
            "Global".to_string()
        };

        let visibility_str = match visibility {
            Visibility::Private => "private only",
            Visibility::Public => "public only",
            Visibility::Any => "all",
        };

        eprintln!("ℹ Scope: {} | Visibility: {}", scope, visibility_str);
    }

    /// Determine visibility based on scoping and --public flag
    ///
    /// Logic:
    /// - If scoped (org/workspace ID set) and no --public flag: Private
    /// - If scoped and --public flag: Public
    /// - If not scoped: Any (current behavior)
    fn determine_visibility(client: &LangchainClient, public_flag: bool) -> Visibility {
        let is_scoped = client.organization_id().is_some() || client.workspace_id().is_some();

        if is_scoped {
            if public_flag {
                Visibility::Public
            } else {
                Visibility::Private
            }
        } else {
            Visibility::Any
        }
    }

    /// Execute the prompt command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;
        let formatter = OutputFormatter::new(format);

        match self {
            PromptCommands::List {
                limit,
                offset,
                organization_id,
                public,
                full,
                columns,
                show_columns,
            } => {
                // Handle --show-columns flag: display available columns and exit
                if *show_columns {
                    println!("Available columns for prompt list:");
                    for col in PROMPT_COLUMNS {
                        println!("  {}", col);
                    }
                    println!("\nUsage: langstar prompt list -f text --columns handle,downloads");
                    return Ok(());
                }

                // Validate --columns if provided
                if let Some(cols) = columns {
                    for col in cols {
                        if !PROMPT_COLUMNS.contains(&col.as_str()) {
                            return Err(crate::error::CliError::Config(format!(
                                "Invalid column '{}'. Available columns: {}",
                                col,
                                PROMPT_COLUMNS.join(", ")
                            )));
                        }
                    }
                }

                let client = Self::apply_scoping(
                    client,
                    organization_id,
                    &config.workspace_id_flag(),
                    config,
                );
                let visibility = Self::determine_visibility(&client, *public);

                // Show scope information
                Self::print_scope_info(&client, visibility);

                formatter.info(&format!(
                    "Fetching prompts (limit: {}, offset: {})...",
                    limit, offset
                ));

                let prompts = client
                    .prompts()
                    .list(Some(*limit), Some(*offset), Some(visibility))
                    .await?;

                match format {
                    OutputFormat::Json => {
                        formatter.print(&prompts)?;
                    }
                    OutputFormat::Text => {
                        // Use provided columns or default to all columns
                        formatter.print_text(&prompts, columns.as_deref())?;
                    }
                    OutputFormat::Table => {
                        if *full {
                            let rows: Vec<FullPromptRow> =
                                prompts.iter().map(FullPromptRow::from).collect();
                            formatter.print_table(&rows)?;
                            println!("\nFound {} prompts", prompts.len());
                        } else {
                            let rows: Vec<CompactPromptRow> =
                                prompts.iter().map(CompactPromptRow::from).collect();
                            formatter.print_table(&rows)?;
                            println!(
                                "\nFound {} prompts (use --full for all columns)",
                                prompts.len()
                            );

                            // Show hint if scoped and no results
                            if prompts.is_empty()
                                && (client.organization_id().is_some()
                                    || client.workspace_id().is_some())
                                && !*public
                            {
                                eprintln!("\n💡 Hint: No private prompts found in this scope.");
                                eprintln!("  Try using --public flag to see public prompts:");
                                eprintln!("    langstar prompt list --public");
                            }
                        }
                    }
                }
            }

            PromptCommands::Get {
                handle,
                organization_id,
            } => {
                let client = Self::apply_scoping(
                    client,
                    organization_id,
                    &config.workspace_id_flag(),
                    config,
                );

                // Detect if input is a valid UUID using the uuid crate for robust parsing
                let is_uuid = Uuid::parse_str(handle).is_ok();

                let prompt = if is_uuid {
                    formatter.info(&format!("Fetching prompt by ID '{}'...", handle));
                    client.prompts().get_by_id(handle).await?
                } else {
                    formatter.info(&format!("Fetching prompt '{}'...", handle));
                    client.prompts().get(handle).await?
                };

                if format == OutputFormat::Json {
                    formatter.print(&prompt)?;
                } else {
                    println!("\n{}", "Prompt Details".to_uppercase());
                    println!("─────────────────────────────────────────");

                    // Display full_name if available, otherwise fall back to repo_handle
                    let display_handle = prompt.full_name.as_ref().unwrap_or(&prompt.repo_handle);
                    println!("Handle:      {}", display_handle);
                    println!("ID:          {}", prompt.id);
                    println!("Likes:       {}", prompt.num_likes);
                    println!("Downloads:   {}", prompt.num_downloads);
                    println!(
                        "Public:      {}",
                        if prompt.is_public { "yes" } else { "no" }
                    );
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

            PromptCommands::Search {
                query,
                limit,
                organization_id,
                public,
                full,
            } => {
                let client = Self::apply_scoping(
                    client,
                    organization_id,
                    &config.workspace_id_flag(),
                    config,
                );
                let visibility = Self::determine_visibility(&client, *public);

                // Show scope information
                Self::print_scope_info(&client, visibility);

                formatter.info(&format!("Searching for '{}'...", query));

                let prompts = client
                    .prompts()
                    .search(query, Some(*limit), Some(visibility))
                    .await?;

                if format == OutputFormat::Json {
                    formatter.print(&prompts)?;
                } else if *full {
                    let rows: Vec<FullPromptRow> =
                        prompts.iter().map(FullPromptRow::from).collect();
                    formatter.print_table(&rows)?;
                    println!("\nFound {} prompts", prompts.len());
                } else {
                    let rows: Vec<CompactPromptRow> =
                        prompts.iter().map(CompactPromptRow::from).collect();
                    formatter.print_table(&rows)?;
                    println!(
                        "\nFound {} prompts (use --full for all columns)",
                        prompts.len()
                    );

                    // Show hint if scoped and no results
                    if prompts.is_empty()
                        && (client.organization_id().is_some() || client.workspace_id().is_some())
                        && !*public
                    {
                        eprintln!(
                            "\n💡 Hint: No private prompts found matching '{}' in this scope.",
                            query
                        );
                        eprintln!("  Try using --public flag to search public prompts:");
                        eprintln!("    langstar prompt search \"{}\" --public", query);
                    }
                }
            }

            PromptCommands::Push {
                owner,
                repo,
                template,
                input_variables,
                template_format,
                schema,
                schema_method,
                organization_id,
            } => {
                // Apply scoping from flags/config
                let mut client = Self::apply_scoping(
                    client,
                    organization_id,
                    &config.workspace_id_flag(),
                    config,
                );

                // If no organization ID was explicitly provided, try to fetch it
                if organization_id.is_none() && client.organization_id().is_none() {
                    formatter.info("Fetching organization information...");
                    match client.get_current_organization().await {
                        Ok(org) => {
                            if let Some(org_id) = &org.id {
                                println!(
                                    "✓ Organization: {}",
                                    org.display_name.as_deref().unwrap_or("Unknown")
                                );
                                println!("  ID: {}", org_id);
                                client = client.with_organization_id(org_id.clone());
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠ Warning: Could not fetch organization: {}", e);
                            eprintln!("  Proceeding without X-Organization-Id header");
                        }
                    }
                }

                // Try to create repository if it doesn't exist
                let repo_handle = format!("{}/{}", owner, repo);
                formatter.info(&format!("Checking if repository {} exists...", repo_handle));

                let repo_exists = match client.prompts().get(&repo_handle).await {
                    Ok(_) => {
                        println!("✓ Repository exists");
                        true
                    }
                    Err(langstar_sdk::error::LangstarError::ApiError { status: 404, .. }) => {
                        formatter.info(&format!(
                            "Repository not found, creating {}...",
                            repo_handle
                        ));
                        match client
                            .prompts()
                            .create_repo(
                                &repo_handle,
                                Some("Created via langstar CLI".to_string()),
                                None,
                                false, // Private by default
                                Some(vec!["cli".to_string(), "langstar".to_string()]),
                            )
                            .await
                        {
                            Ok(_) => {
                                println!("✓ Repository created successfully");
                                false // New repo, no commits yet
                            }
                            Err(e) => {
                                eprintln!("⚠ Warning: Could not create repository: {}", e);
                                eprintln!(
                                    "  Will attempt to push anyway, without a parent commit..."
                                );
                                false // Repository creation failed, so no parent commit is fetched
                            }
                        }
                    }
                    Err(e) => return Err(crate::error::CliError::Sdk(e)),
                };

                // Determine parent commit for the push
                // Automatically fetch latest commit as parent if repo exists
                let final_parent_commit = if repo_exists {
                    formatter.info("Fetching latest commit as parent...");
                    match client.prompts().get_commit(owner, repo, "latest").await {
                        Ok(commit_info) => {
                            println!("✓ Latest commit: {}", commit_info.commit_hash);
                            Some(commit_info.commit_hash)
                        }
                        // A 404 means the repo exists but has no commits yet
                        Err(langstar_sdk::error::LangstarError::ApiError {
                            status: 404, ..
                        }) => {
                            formatter
                                .info("Repository has no commits yet, no parent commit needed");
                            None
                        }
                        Err(e) => return Err(crate::error::CliError::Sdk(e)),
                    }
                } else {
                    formatter.info("New repository, no parent commit needed");
                    None
                };

                // Parse input variables
                let vars: Vec<String> = if let Some(vars_str) = input_variables {
                    vars_str.split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    vec![]
                };

                // Check if schema is provided - use structured prompt if so
                let response = if let Some(schema_path) = schema {
                    formatter.info("Loading schema file...");

                    // Read and parse schema file
                    let schema_content = fs::read_to_string(schema_path).map_err(|e| {
                        crate::error::CliError::Sdk(langstar_sdk::error::LangstarError::Other(
                            format!("Failed to read schema file: {}", e),
                        ))
                    })?;

                    let schema_value: serde_json::Value = serde_json::from_str(&schema_content)?;

                    // Validate schema before proceeding
                    formatter.info("Validating schema...");
                    validate_json_schema(&schema_value).map_err(crate::error::CliError::Sdk)?;

                    // Validate method
                    validate_method(schema_method).map_err(crate::error::CliError::Sdk)?;

                    formatter.info("✓ Schema valid");

                    formatter.info(&format!(
                        "Pushing structured prompt to {}/{} (method: {})...",
                        owner, repo, schema_method
                    ));

                    // Build StructuredPrompt
                    let prompt_template_kwargs = PromptTemplateKwargs {
                        input_variables: vars.clone(),
                        template: template.clone(),
                        template_format: template_format.clone(),
                    };

                    let prompt_template_lc = LcJson::new(
                        vec![
                            "langchain_core".to_string(),
                            "prompts".to_string(),
                            "prompt".to_string(),
                            "PromptTemplate".to_string(),
                        ],
                        prompt_template_kwargs,
                    );

                    let message_kwargs = MessagePromptTemplateKwargs {
                        prompt: prompt_template_lc,
                    };

                    let message_lc = LcJson::new(
                        vec![
                            "langchain_core".to_string(),
                            "prompts".to_string(),
                            "chat".to_string(),
                            "HumanMessagePromptTemplate".to_string(),
                        ],
                        message_kwargs,
                    );

                    let structured_prompt = StructuredPrompt {
                        input_variables: if vars.is_empty() { None } else { Some(vars) },
                        messages: vec![message_lc],
                        schema_: schema_value,
                        structured_output_kwargs: StructuredOutputKwargs {
                            method: schema_method.clone(),
                        },
                    };

                    // Push structured prompt
                    client
                        .prompts()
                        .push_structured_prompt(
                            owner,
                            repo,
                            structured_prompt,
                            final_parent_commit.clone(),
                        )
                        .await
                        .map_err(crate::error::CliError::Sdk)?
                } else {
                    // Regular prompt push (no schema)
                    formatter.info(&format!("Pushing prompt to {}/{}...", owner, repo));

                    let commit_request = CommitRequest {
                        manifest: json!({
                            "type": "prompt",
                            "template": template,
                            "input_variables": vars,
                            "template_format": template_format
                        }),
                        description: None,
                        parent_commit: final_parent_commit,
                        example_run_ids: None,
                    };

                    client
                        .prompts()
                        .push(owner, repo, &commit_request)
                        .await
                        .map_err(crate::error::CliError::Sdk)?
                };

                // Display success message
                if format == OutputFormat::Json {
                    formatter.print(&response)?;
                } else {
                    println!("\n✓ Prompt commit pushed successfully!");
                    println!("  Repository: {}/{}", owner, repo);
                    println!("  Commit hash: {}", response.commit.commit_hash);
                    if let Some(url) = &response.commit.url {
                        println!("  URL: {}", url);
                    }
                    if schema.is_some() {
                        println!("  Type: Structured prompt with JSON schema");
                    }
                }
            }

            PromptCommands::Pull {
                handle,
                commit,
                organization_id,
            } => {
                let client = Self::apply_scoping(
                    client,
                    organization_id,
                    &config.workspace_id_flag(),
                    config,
                );

                // Parse handle into owner/repo
                let parts: Vec<&str> = handle.split('/').collect();
                if parts.len() != 2 {
                    return Err(crate::error::CliError::Other(anyhow::anyhow!(
                        "Invalid handle format. Expected: owner/repo-name"
                    )));
                }
                let (owner, repo) = (parts[0], parts[1]);

                formatter.info(&format!(
                    "Pulling prompt '{}' (commit: {})...",
                    handle, commit
                ));

                let manifest = client.prompts().pull(owner, repo, commit).await?;

                if format == OutputFormat::Json {
                    formatter.print(&manifest)?;
                } else {
                    // Try to detect if it's a structured prompt
                    let is_structured = manifest
                        .get("id")
                        .and_then(|id| id.as_array())
                        .map(|arr| {
                            arr.last()
                                .and_then(|v| v.as_str())
                                .map(|s| s == "StructuredPrompt")
                                .unwrap_or(false)
                        })
                        .unwrap_or(false);

                    println!("\n{}", "Prompt Manifest".to_uppercase());
                    println!("─────────────────────────────────────────");
                    println!("Handle:  {}", handle);
                    println!("Commit:  {}", commit);

                    if is_structured {
                        println!("Type:    Structured prompt with JSON schema\n");

                        // Try to parse as structured prompt and display schema
                        match serde_json::from_value::<LcJson<StructuredPrompt>>(manifest.clone()) {
                            Ok(lc_json) => {
                                let structured = lc_json.kwargs;

                                // Show input variables
                                if let Some(vars) = &structured.input_variables
                                    && !vars.is_empty()
                                {
                                    println!("Input Variables:");
                                    for var in vars {
                                        println!("  - {}", var);
                                    }
                                    println!();
                                }

                                // Show schema
                                println!("JSON Schema:");
                                let schema_pretty =
                                    serde_json::to_string_pretty(&structured.schema_)?;
                                for line in schema_pretty.lines() {
                                    println!("  {}", line);
                                }
                                println!();

                                // Show method
                                println!("Method: {}", structured.structured_output_kwargs.method);

                                // Show template from first message
                                if let Some(msg) = structured.messages.first() {
                                    println!("\nTemplate:");
                                    println!("  {}", msg.kwargs.prompt.kwargs.template);
                                }
                            }
                            Err(e) => {
                                // Fallback to raw JSON if parsing fails
                                eprintln!("⚠ Warning: Could not parse as StructuredPrompt: {}", e);
                                println!("Raw Manifest:");
                                let manifest_pretty = serde_json::to_string_pretty(&manifest)?;
                                for line in manifest_pretty.lines() {
                                    println!("  {}", line);
                                }
                            }
                        }
                    } else {
                        println!("Type:    Regular prompt\n");
                        println!("Raw Manifest:");
                        let manifest_pretty = serde_json::to_string_pretty(&manifest)?;
                        for line in manifest_pretty.lines() {
                            println!("  {}", line);
                        }
                    }
                }
            }

            PromptCommands::Export {
                handle,
                commit,
                output,
                not_a_secret,
                organization_id,
            } => {
                let client = Self::apply_scoping(
                    client,
                    organization_id,
                    &config.workspace_id_flag(),
                    config,
                );
                let request = ExportRequest {
                    handle,
                    commit: commit.as_deref(),
                    output: output.as_deref(),
                    not_a_secret,
                };
                run_export(
                    &client,
                    &request,
                    &mut std::io::stdout(),
                    &mut std::io::stderr(),
                )
                .await?;
            }
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// prompt export
// ═══════════════════════════════════════════════════════════════════════

/// What `langstar prompt export` was asked to do.
struct ExportRequest<'a> {
    /// The handle as the user typed it: `my-prompt`, `-/my-prompt` or `owner/my-prompt`.
    handle: &'a str,
    /// A commit hash or tag, or `None` for the latest commit.
    commit: Option<&'a str>,
    /// The file to write, or `None` for stdout.
    output: Option<&'a std::path::Path>,
    /// The JSON Pointers the user named with `--not-a-secret`.
    not_a_secret: &'a [String],
}

/// Export one prompt commit: read it, check it for credentials, and write the
/// file to `output` or to `stdout`.
///
/// Every message goes to `stderr`, so stdout holds the file and nothing else.
/// When the check flags a string, this returns the refusal before it writes
/// anything, and `main` prints it and exits with status 1.
async fn run_export(
    client: &LangchainClient,
    request: &ExportRequest<'_>,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> Result<()> {
    let (owner, repo) = split_handle(request.handle)?;
    let commit = request.commit.unwrap_or("latest");

    writeln!(stderr, "Reading {}/{} at {}...", owner, repo, commit)?;
    let export = read_prompt_export(client, request.handle, &owner, &repo, commit).await?;
    let allowed = check_export_for_credentials(&export, request.not_a_secret)?;
    for item in &allowed {
        writeln!(
            stderr,
            "⚠ Warning: writing the string at {} ({}), because --not-a-secret names it",
            item.path, item.rule
        )?;
    }

    let text = crate::export_file::to_sorted_json(&export)?;
    match request.output {
        Some(path) => {
            fs::write(path, &text).map_err(|source| {
                crate::error::CliError::Other(anyhow::anyhow!(
                    "Could not write {}: {}",
                    path.display(),
                    source
                ))
            })?;
            writeln!(
                stderr,
                "✓ Wrote commit {} of {} to {}",
                export.source.commit_hash,
                export.source.handle,
                path.display()
            )?;
        }
        None => {
            stdout.write_all(text.as_bytes())?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// Split a handle into the owner and repo the commit read takes. A handle with
/// no owner, `my-prompt`, names a prompt in the current workspace, which
/// LangSmith reads as `-/my-prompt`.
fn split_handle(handle: &str) -> Result<(String, String)> {
    let invalid = || {
        crate::error::CliError::Other(anyhow::anyhow!(
            "Invalid handle `{}`. Expected my-prompt, -/my-prompt or owner/my-prompt",
            handle
        ))
    };
    match handle.split_once('/') {
        None if !handle.is_empty() => Ok(("-".to_string(), handle.to_string())),
        Some((owner, repo)) if !owner.is_empty() && !repo.is_empty() && !repo.contains('/') => {
            Ok((owner.to_string(), repo.to_string()))
        }
        _ => Err(invalid()),
    }
}

/// Read the repo, the commit with its model, and the commit's tags, and build
/// the export file from them.
async fn read_prompt_export(
    client: &LangchainClient,
    handle: &str,
    owner: &str,
    repo: &str,
    commit: &str,
) -> Result<crate::export_file::PromptExport> {
    use crate::export_file::{
        FORMAT_VERSION, Kind, PromptCommit, PromptExport, PromptRepo, PromptRequires, PromptSource,
    };
    use langstar_sdk::error::LangstarError;

    let prompts = client.prompts();
    let details = match prompts.get(&format!("{}/{}", owner, repo)).await {
        Err(LangstarError::ApiError { status: 404, .. }) => {
            return Err(crate::error::CliError::Other(anyhow::anyhow!(
                "No prompt in this workspace has the handle `{}`",
                handle
            )));
        }
        other => other?,
    };
    let read = match prompts.get_commit_with_model(owner, repo, commit).await {
        Err(LangstarError::ApiError { status: 404, .. }) => {
            return Err(crate::error::CliError::Other(anyhow::anyhow!(
                "The prompt `{}` has no commit or tag `{}`",
                handle,
                commit
            )));
        }
        other => other?,
    };
    let commit_tags = prompts
        .list_tags(owner, repo)
        .await?
        .into_iter()
        .filter(|tag| tag.commit_hash == read.commit_hash)
        .map(|tag| tag.tag_name)
        .collect();

    let workspace_id = details
        .tenant_id
        .clone()
        .or_else(|| client.workspace_id().map(str::to_string))
        .ok_or_else(|| {
            crate::error::CliError::Other(anyhow::anyhow!(
                "LangSmith returned no workspace for the prompt `{}`. Set --workspace-id",
                handle
            ))
        })?;

    Ok(PromptExport {
        kind: Kind::Prompt,
        format_version: FORMAT_VERSION,
        source: PromptSource {
            workspace_id,
            handle: details.repo_handle.clone(),
            commit_hash: read.commit_hash.clone(),
        },
        repo: PromptRepo {
            handle: details.repo_handle,
            description: details.description,
            readme: details.readme,
            tags: details.tags.into_iter().collect(),
            is_public: details.is_public,
        },
        requires: PromptRequires {
            secrets: crate::credential_check::secret_names(&read.manifest),
            model_class: model_class(&read.manifest),
        },
        commit: PromptCommit {
            description: read.description,
            manifest: read.manifest,
            model_config: read.model_config,
            model_provider: read.model_provider,
            tags: commit_tags,
        },
    })
}

/// The class name of the manifest's chat model, such as `ChatOpenAI`, or `None`
/// for a prompt saved without a model.
///
/// The commit read with `include=model` puts the chat model at `kwargs.last`.
/// On a prompt saved in the playground a `RunnableBinding` sits there and the
/// constructor is at its `kwargs.bound`, so this steps through the binding
/// before it reads the last element of the constructor's `id`.
fn model_class(manifest: &serde_json::Value) -> Option<String> {
    fn class_name(value: &serde_json::Value) -> Option<&str> {
        value.get("id")?.as_array()?.last()?.as_str()
    }
    let last = manifest.get("kwargs")?.get("last")?;
    let constructor = if class_name(last) == Some("RunnableBinding") {
        last.get("kwargs")?.get("bound")?
    } else {
        last
    };
    class_name(constructor).map(str::to_string)
}

/// Run the credential check over the `repo` and `commit` objects the file would
/// hold, and return the flagged strings the user named with `--not-a-secret`.
///
/// A `--not-a-secret` pointer at a string the check lets through is an error,
/// and so is any flagged string the user did not name. Neither error holds any
/// part of a flagged string.
fn check_export_for_credentials(
    export: &crate::export_file::PromptExport,
    not_a_secret: &[String],
) -> Result<Vec<crate::credential_check::Flagged>> {
    use crate::credential_check::{find_credentials, refusal_message};

    let checked = json!({
        "repo": serde_json::to_value(&export.repo)?,
        "commit": serde_json::to_value(&export.commit)?,
    });
    let flagged = find_credentials(&checked);

    let unflagged: Vec<&String> = not_a_secret
        .iter()
        .filter(|pointer| !flagged.iter().any(|item| &item.path == *pointer))
        .collect();
    if !unflagged.is_empty() {
        let mut message =
            String::from("--not-a-secret names a path where the check flags no string:\n");
        for pointer in unflagged {
            message.push_str(&format!("  {}\n", pointer));
        }
        message.push_str(
            "Remove each one, or name the path the refusal printed. No file was written.",
        );
        return Err(crate::error::CliError::Other(anyhow::anyhow!(message)));
    }

    let (allowed, refused): (Vec<_>, Vec<_>) = flagged
        .into_iter()
        .partition(|item| not_a_secret.contains(&item.path));
    if !refused.is_empty() {
        return Err(crate::error::CliError::Other(anyhow::anyhow!(
            "{}\nNo file was written.",
            refusal_message(&refused)
        )));
    }
    Ok(allowed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use langstar_sdk::AuthConfig;

    #[test]
    fn test_apply_scoping_with_no_flags() {
        // Client with no scoping
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let config = Config::default();

        assert_eq!(client.organization_id(), None);
        assert_eq!(client.workspace_id(), None);

        // Apply scoping with no flags - should remain unchanged
        let scoped_client = PromptCommands::apply_scoping(client, &None, &None, &config);

        assert_eq!(scoped_client.organization_id(), None);
        assert_eq!(scoped_client.workspace_id(), None);
    }

    #[test]
    fn test_apply_scoping_with_org_flag() {
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let config = Config::default();

        let org_id = Some("test-org-id".to_string());
        let scoped_client = PromptCommands::apply_scoping(client, &org_id, &None, &config);

        assert_eq!(scoped_client.organization_id(), Some("test-org-id"));
        assert_eq!(scoped_client.workspace_id(), None);
    }

    #[test]
    fn test_apply_scoping_with_workspace_flag() {
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let config = Config::default();

        let workspace_id = Some("test-workspace-id".to_string());
        let scoped_client = PromptCommands::apply_scoping(client, &None, &workspace_id, &config);

        assert_eq!(scoped_client.organization_id(), None);
        assert_eq!(scoped_client.workspace_id(), Some("test-workspace-id"));
    }

    #[test]
    fn test_apply_scoping_with_both_flags() {
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let config = Config::default();

        let org_id = Some("test-org-id".to_string());
        let workspace_id = Some("test-workspace-id".to_string());
        let scoped_client = PromptCommands::apply_scoping(client, &org_id, &workspace_id, &config);

        assert_eq!(scoped_client.organization_id(), Some("test-org-id"));
        assert_eq!(scoped_client.workspace_id(), Some("test-workspace-id"));
    }

    #[test]
    fn test_apply_scoping_flag_overrides_config() {
        // Client with org ID from config
        let auth = AuthConfig::new(
            Some("test_key".to_string()),
            Some("config-org-id".to_string()),
            None,
        );
        let client = LangchainClient::new(auth).unwrap();
        let config = Config::default();

        assert_eq!(client.organization_id(), Some("config-org-id"));

        // Flag should override config
        let org_id = Some("flag-org-id".to_string());
        let scoped_client = PromptCommands::apply_scoping(client, &org_id, &None, &config);

        assert_eq!(scoped_client.organization_id(), Some("flag-org-id"));
    }

    #[test]
    fn test_determine_visibility_unscoped() {
        // Client with no scoping should default to Any
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();

        // Without --public flag
        let visibility = PromptCommands::determine_visibility(&client, false);
        assert_eq!(visibility, Visibility::Any);

        // With --public flag (should still be Any when unscoped)
        let visibility = PromptCommands::determine_visibility(&client, true);
        assert_eq!(visibility, Visibility::Any);
    }

    #[test]
    fn test_determine_visibility_scoped_with_org_id() {
        // Client with organization ID
        let auth = AuthConfig::new(
            Some("test_key".to_string()),
            Some("test-org-id".to_string()),
            None,
        );
        let client = LangchainClient::new(auth).unwrap();

        // Without --public flag should default to Private
        let visibility = PromptCommands::determine_visibility(&client, false);
        assert_eq!(visibility, Visibility::Private);

        // With --public flag should be Public
        let visibility = PromptCommands::determine_visibility(&client, true);
        assert_eq!(visibility, Visibility::Public);
    }

    #[test]
    fn test_determine_visibility_scoped_with_workspace_id() {
        // Client with workspace ID
        let auth = AuthConfig::new(
            Some("test_key".to_string()),
            None,
            Some("test-workspace-id".to_string()),
        );
        let client = LangchainClient::new(auth).unwrap();

        // Without --public flag should default to Private
        let visibility = PromptCommands::determine_visibility(&client, false);
        assert_eq!(visibility, Visibility::Private);

        // With --public flag should be Public
        let visibility = PromptCommands::determine_visibility(&client, true);
        assert_eq!(visibility, Visibility::Public);
    }

    #[test]
    fn test_determine_visibility_scoped_with_both_ids() {
        // Client with both organization and workspace IDs
        let auth = AuthConfig::new(
            Some("test_key".to_string()),
            Some("test-org-id".to_string()),
            Some("test-workspace-id".to_string()),
        );
        let client = LangchainClient::new(auth).unwrap();

        // Without --public flag should default to Private
        let visibility = PromptCommands::determine_visibility(&client, false);
        assert_eq!(visibility, Visibility::Private);

        // With --public flag should be Public
        let visibility = PromptCommands::determine_visibility(&client, true);
        assert_eq!(visibility, Visibility::Public);
    }

    #[test]
    fn test_truncate_description_long() {
        // Test truncation when description exceeds max_len
        let long_desc = "a".repeat(100);
        let result = truncate_description(Some(&long_desc), 40);
        assert_eq!(result.len(), 40);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_description_short() {
        // Test no truncation when description is shorter than max_len
        let short_desc = "Short description".to_string();
        let result = truncate_description(Some(&short_desc), 40);
        assert_eq!(result, "Short description");
    }

    #[test]
    fn test_truncate_description_none() {
        // Test handling of None
        let result = truncate_description(None, 40);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_description_exact_length() {
        // Test when description is exactly max_len
        let exact_desc = "a".repeat(40);
        let result = truncate_description(Some(&exact_desc), 40);
        assert_eq!(result, exact_desc);
        assert_eq!(result.len(), 40);
    }

    #[test]
    fn test_truncate_description_edge_case_small_max() {
        // Test edge case with max_len < 3
        let desc = "Hello world".to_string();
        let result = truncate_description(Some(&desc), 3);
        // With max_len=3 and saturating_sub(3)=0, we get "" + "..."
        assert_eq!(result, "...");
    }
}

/// Tests of `prompt export` against a `mockito` server that serves one prompt.
///
/// Each test serves the three reads export sends — the repo, the commit with
/// `include=model`, and the repo's tags — and calls the export code the
/// command runs, with stdout and stderr captured.
#[cfg(test)]
mod export_tests {
    use super::*;
    use langstar_sdk::AuthConfig;
    use mockito::{Matcher, Mock, ServerGuard};
    use serde_json::Value;
    use std::path::Path;

    /// The workspace the served repo says it is in.
    const TENANT_ID: &str = "00000000-0000-4000-8000-000000000001";
    /// The hash of the commit the server serves.
    const COMMIT_HASH: &str = "0a1b2c3d4e5f";
    /// A provider token under a neutral key, made at run time so no scanner
    /// reads the source as holding one.
    fn provider_token() -> String {
        format!("sk-{}", "0123456789abcdefghij".repeat(2))
    }

    /// One served prompt: the repo, the commit and the tags.
    struct Served {
        server: ServerGuard,
        _mocks: Vec<Mock>,
    }

    fn repo(readme: Option<&str>) -> Value {
        json!({
            "id": "00000000-0000-4000-8000-000000000002",
            "repo_handle": "my-prompt",
            "description": "A prompt for tests",
            "readme": readme,
            "tags": ["support", "ChatPromptTemplate"],
            "tenant_id": TENANT_ID,
            "is_public": false,
        })
    }

    fn chat_prompt(template: &str) -> Value {
        json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "prompts", "chat", "ChatPromptTemplate"],
            "kwargs": {
                "input_variables": ["question"],
                "messages": [{
                    "lc": 1,
                    "type": "constructor",
                    "id": ["langchain", "prompts", "chat", "HumanMessagePromptTemplate"],
                    "kwargs": {
                        "prompt": {
                            "lc": 1,
                            "type": "constructor",
                            "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
                            "kwargs": {
                                "input_variables": ["question"],
                                "template": template,
                                "template_format": "f-string",
                            },
                        },
                    },
                }],
            },
        })
    }

    fn chat_open_ai(kwargs: Value) -> Value {
        json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
            "kwargs": kwargs,
        })
    }

    fn secret_reference(name: &str) -> Value {
        json!({"lc": 1, "type": "secret", "id": [name]})
    }

    fn binding(bound: Value) -> Value {
        json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "schema", "runnable", "RunnableBinding"],
            "kwargs": {"bound": bound, "kwargs": {}},
        })
    }

    fn sequence(first: Value, last: Value) -> Value {
        json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "schema", "runnable", "RunnableSequence"],
            "kwargs": {"first": first, "last": last},
        })
    }

    /// A prompt whose model reads its key from the `OPENAI_API_KEY` secret.
    fn safe_manifest() -> Value {
        sequence(
            chat_prompt("Answer {question}"),
            chat_open_ai(json!({
                "model": "gpt-4o-mini",
                "max_tokens": 1024,
                "openai_api_key": secret_reference("OPENAI_API_KEY"),
            })),
        )
    }

    async fn serve(repo: Value, manifest: Value) -> Served {
        let mut server = mockito::Server::new_async().await;
        let repo_mock = server
            .mock("GET", "/api/v1/repos/-/my-prompt")
            .with_status(200)
            .with_body(json!({"repo": repo}).to_string())
            .create_async()
            .await;
        let commit_mock = server
            .mock("GET", "/api/v1/commits/-/my-prompt/latest")
            .match_query(Matcher::UrlEncoded("include".into(), "model".into()))
            .with_status(200)
            .with_body(
                json!({
                    "commit_hash": COMMIT_HASH,
                    "manifest": manifest,
                    "description": "First commit",
                    "examples": [],
                })
                .to_string(),
            )
            .create_async()
            .await;
        let tags_mock = server
            .mock("GET", "/api/v1/repos/-/my-prompt/tags")
            .with_status(200)
            .with_body(
                json!([
                    {"id": "t1", "tag_name": "prod", "commit_id": "c1", "commit_hash": COMMIT_HASH},
                    {"id": "t2", "tag_name": "dev", "commit_id": "c2", "commit_hash": "ffffffffffff"},
                ])
                .to_string(),
            )
            .create_async()
            .await;
        Served {
            server,
            _mocks: vec![repo_mock, commit_mock, tags_mock],
        }
    }

    fn client_for(served: &Served, workspace_id: Option<&str>) -> LangchainClient {
        LangchainClient::with_base_urls(
            AuthConfig::new(
                Some("test_key".to_string()),
                None,
                workspace_id.map(str::to_string),
            ),
            served.server.url(),
            served.server.url(),
            served.server.url(),
        )
        .unwrap()
    }

    /// What one export run left behind.
    struct Run {
        result: Result<()>,
        stdout: String,
        stderr: String,
    }

    async fn export(
        served: &Served,
        handle: &str,
        output: Option<&Path>,
        not_a_secret: &[&str],
    ) -> Run {
        let client = client_for(served, None);
        let not_a_secret: Vec<String> = not_a_secret.iter().map(|p| p.to_string()).collect();
        let request = ExportRequest {
            handle,
            commit: None,
            output,
            not_a_secret: &not_a_secret,
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = run_export(&client, &request, &mut stdout, &mut stderr).await;
        Run {
            result,
            stdout: String::from_utf8(stdout).unwrap(),
            stderr: String::from_utf8(stderr).unwrap(),
        }
    }

    /// Export to stdout and parse what it printed.
    async fn export_document(served: &Served, not_a_secret: &[&str]) -> (Value, Run) {
        let run = export(served, "my-prompt", None, not_a_secret).await;
        assert!(run.result.is_ok(), "export failed: {:?}", run.result);
        let document = serde_json::from_str(&run.stdout).expect("stdout is one JSON document");
        (document, run)
    }

    /// Export once with `--output` and once to stdout, and assert that both
    /// refuse with the path and the rule, without the string, and write nothing.
    async fn assert_refused(repo: Value, manifest: Value, path: &str, rule: &str, string: &str) {
        let served = serve(repo, manifest).await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my-prompt.json");

        for output in [Some(file.as_path()), None] {
            let run = export(&served, "my-prompt", output, &[]).await;
            let message = match &run.result {
                Err(error) => error.to_string(),
                Ok(()) => panic!("export wrote a file holding {path}"),
            };
            assert!(message.contains(path), "{message}");
            assert!(message.contains(&format!("({rule})")), "{message}");
            assert!(!message.contains(string), "the refusal holds the string");
            assert!(!run.stderr.contains(string), "stderr holds the string");
            assert!(run.stdout.is_empty(), "stdout holds {:?}", run.stdout);
            assert!(!file.exists(), "export wrote {}", file.display());
        }
    }

    #[test]
    fn test_model_class_steps_through_a_runnable_binding() {
        let manifest = sequence(chat_prompt("Hi"), binding(chat_open_ai(json!({}))));
        assert_eq!(model_class(&manifest).as_deref(), Some("ChatOpenAI"));
        assert_eq!(model_class(&chat_prompt("Hi")), None);
    }

    #[tokio::test]
    async fn test_export_finds_the_model_class_inside_a_runnable_binding() {
        let manifest = sequence(
            chat_prompt("Answer {question}"),
            binding(chat_open_ai(json!({"model": "gpt-4o-mini"}))),
        );
        let served = serve(repo(None), manifest).await;
        let (document, _) = export_document(&served, &[]).await;
        assert_eq!(document["requires"]["model_class"], "ChatOpenAI");
        assert!(!document["requires"].to_string().contains("RunnableBinding"));
    }

    #[tokio::test]
    async fn test_export_finds_the_model_class_with_no_binding() {
        let manifest = sequence(
            chat_prompt("Answer {question}"),
            chat_open_ai(json!({"model": "gpt-4o-mini"})),
        );
        let served = serve(repo(None), manifest).await;
        let (document, _) = export_document(&served, &[]).await;
        assert_eq!(document["requires"]["model_class"], "ChatOpenAI");
        assert!(!document["requires"].to_string().contains("RunnableBinding"));
    }

    #[tokio::test]
    async fn test_export_writes_null_model_class_for_a_prompt_without_a_model() {
        let served = serve(repo(None), chat_prompt("Answer {question}")).await;
        let (document, _) = export_document(&served, &[]).await;
        assert_eq!(document["requires"]["model_class"], Value::Null);
        assert_eq!(document["requires"]["secrets"], json!([]));
    }

    #[tokio::test]
    async fn test_export_refuses_a_literal_key_under_openai_api_key() {
        let value = "literal-value-0123456789";
        let manifest = sequence(
            chat_prompt("Hi"),
            chat_open_ai(json!({"model": "gpt-4o-mini", "openai_api_key": value})),
        );
        assert_refused(
            repo(None),
            manifest,
            "/commit/manifest/kwargs/last/kwargs/openai_api_key",
            "key name",
            value,
        )
        .await;
    }

    #[tokio::test]
    async fn test_export_refuses_a_literal_key_under_upper_case_openai_api_key() {
        let value = "literal-value-0123456789";
        let manifest = sequence(
            chat_prompt("Hi"),
            chat_open_ai(json!({"model": "gpt-4o-mini", "OPENAI_API_KEY": value})),
        );
        assert_refused(
            repo(None),
            manifest,
            "/commit/manifest/kwargs/last/kwargs/OPENAI_API_KEY",
            "key name",
            value,
        )
        .await;
    }

    #[tokio::test]
    async fn test_export_refuses_an_authorization_header() {
        let token = "abcdefghijklmnopqrstuvwxyz012345";
        let manifest = sequence(
            chat_prompt("Hi"),
            chat_open_ai(json!({
                "model": "gpt-4o-mini",
                "default_headers": {"Authorization": format!("Bearer {token}")},
            })),
        );
        assert_refused(
            repo(None),
            manifest,
            "/commit/manifest/kwargs/last/kwargs/default_headers/Authorization",
            "key name",
            token,
        )
        .await;
    }

    #[tokio::test]
    async fn test_export_refuses_a_provider_token_under_a_neutral_key() {
        let token = provider_token();
        let manifest = sequence(
            chat_prompt("Hi"),
            chat_open_ai(json!({
                "model": "gpt-4o-mini",
                "model_kwargs": {"extra": token},
            })),
        );
        assert_refused(
            repo(None),
            manifest,
            "/commit/manifest/kwargs/last/kwargs/model_kwargs/extra",
            "provider token",
            &token[3..],
        )
        .await;
    }

    #[tokio::test]
    async fn test_export_refuses_a_connection_string_with_a_password_in_metadata() {
        let password = "hunter2-db-password";
        let mut prompt = chat_prompt("Hi");
        prompt["kwargs"]["metadata"] = json!({
            "source_db": format!("postgresql://app:{password}@db.example.com/prod"),
        });
        assert_refused(
            repo(None),
            sequence(prompt, chat_open_ai(json!({"model": "gpt-4o-mini"}))),
            "/commit/manifest/kwargs/first/kwargs/metadata/source_db",
            "URL with a password",
            password,
        )
        .await;
    }

    #[tokio::test]
    async fn test_export_refuses_a_private_key_in_the_readme() {
        let body = "MIIEvQIBADANBgkqhkiG9w0BAQEFAASC";
        let readme =
            format!("Setup\n-----BEGIN PRIVATE KEY-----\n{body}\n-----END PRIVATE KEY-----\n");
        assert_refused(
            repo(Some(&readme)),
            safe_manifest(),
            "/repo/readme",
            "PEM private key",
            body,
        )
        .await;
    }

    #[tokio::test]
    async fn test_export_writes_a_secret_reference() {
        let served = serve(repo(None), safe_manifest()).await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my-prompt.json");
        let run = export(&served, "my-prompt", Some(&file), &[]).await;
        assert!(run.result.is_ok(), "{:?}", run.result);
        assert!(run.stdout.is_empty());

        let document: Value =
            serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        let model = &document["commit"]["manifest"]["kwargs"]["last"]["kwargs"];
        assert_eq!(model["openai_api_key"], secret_reference("OPENAI_API_KEY"));
        assert_eq!(model["max_tokens"], 1024);
        assert_eq!(document["requires"]["secrets"], json!(["OPENAI_API_KEY"]));
        assert_eq!(document["requires"]["model_class"], "ChatOpenAI");
    }

    #[tokio::test]
    async fn test_export_writes_every_field_and_only_the_commits_tags() {
        let served = serve(repo(Some("Read me")), safe_manifest()).await;
        let (document, run) = export_document(&served, &[]).await;
        assert!(run.stderr.contains("Reading"), "{}", run.stderr);
        assert_eq!(
            document,
            json!({
                "kind": "prompt",
                "format_version": 1,
                "source": {
                    "workspace_id": TENANT_ID,
                    "handle": "my-prompt",
                    "commit_hash": COMMIT_HASH,
                },
                "repo": {
                    "handle": "my-prompt",
                    "description": "A prompt for tests",
                    "readme": "Read me",
                    "tags": ["ChatPromptTemplate", "support"],
                    "is_public": false,
                },
                "commit": {
                    "description": "First commit",
                    "manifest": safe_manifest(),
                    "tags": ["prod"],
                },
                "requires": {"secrets": ["OPENAI_API_KEY"], "model_class": "ChatOpenAI"},
            })
        );
    }

    #[tokio::test]
    async fn test_export_reads_a_private_handle_with_or_without_the_dash() {
        let served = serve(repo(None), safe_manifest()).await;
        let short = export(&served, "my-prompt", None, &[]).await;
        let dashed = export(&served, "-/my-prompt", None, &[]).await;
        assert!(short.result.is_ok() && dashed.result.is_ok());
        assert_eq!(short.stdout, dashed.stdout);
    }

    #[tokio::test]
    async fn test_export_names_a_handle_the_workspace_has_no_prompt_for() {
        let mut server = mockito::Server::new_async().await;
        let _missing = server
            .mock("GET", "/api/v1/repos/-/no-such-prompt")
            .with_status(404)
            .with_body(r#"{"detail":"Repo not found"}"#)
            .create_async()
            .await;
        let served = Served {
            server,
            _mocks: Vec::new(),
        };
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my-prompt.json");
        let run = export(&served, "no-such-prompt", Some(&file), &[]).await;
        let message = run.result.unwrap_err().to_string();
        assert!(message.contains("`no-such-prompt`"), "{message}");
        assert!(run.stdout.is_empty());
        assert!(!file.exists());
    }

    /// A prompt whose message template shows an example bearer token.
    fn template_with_example_token() -> Value {
        sequence(
            chat_prompt("Send the header Authorization: Bearer abcdefghijklmnopqrstuvwx"),
            chat_open_ai(json!({
                "model": "gpt-4o-mini",
                "openai_api_key": secret_reference("OPENAI_API_KEY"),
            })),
        )
    }

    const TEMPLATE_PATH: &str =
        "/commit/manifest/kwargs/first/kwargs/messages/0/kwargs/prompt/kwargs/template";

    #[tokio::test]
    async fn test_not_a_secret_writes_the_named_template_and_warns() {
        let served = serve(repo(None), template_with_example_token()).await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my-prompt.json");
        let run = export(&served, "my-prompt", Some(&file), &[TEMPLATE_PATH]).await;
        assert!(run.result.is_ok(), "{:?}", run.result);
        assert!(run.stderr.contains("Warning"), "{}", run.stderr);
        assert!(run.stderr.contains(TEMPLATE_PATH), "{}", run.stderr);

        let document: Value =
            serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(
            document["commit"]["manifest"],
            template_with_example_token(),
            "the template changed"
        );
    }

    #[tokio::test]
    async fn test_not_a_secret_still_refuses_a_second_flagged_path() {
        let mut manifest = template_with_example_token();
        manifest["kwargs"]["last"]["kwargs"]["model_kwargs"] = json!({"extra": provider_token()});
        let served = serve(repo(None), manifest).await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my-prompt.json");
        let run = export(&served, "my-prompt", Some(&file), &[TEMPLATE_PATH]).await;
        let message = run.result.unwrap_err().to_string();
        assert!(
            message.contains("/commit/manifest/kwargs/last/kwargs/model_kwargs/extra"),
            "{message}"
        );
        assert!(!message.contains(TEMPLATE_PATH), "{message}");
        assert!(run.stdout.is_empty());
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn test_not_a_secret_refuses_a_path_the_check_lets_through() {
        let served = serve(repo(None), safe_manifest()).await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("my-prompt.json");
        let run = export(&served, "my-prompt", Some(&file), &["/repo/description"]).await;
        let message = run.result.unwrap_err().to_string();
        assert!(message.contains("/repo/description"), "{message}");
        assert!(run.stdout.is_empty());
        assert!(!file.exists());
    }

    #[tokio::test]
    async fn test_export_takes_the_workspace_from_the_repos_tenant_id() {
        let served = serve(repo(None), safe_manifest()).await;
        // `export` builds its client with no workspace ID.
        let (document, _) = export_document(&served, &[]).await;
        assert_eq!(document["source"]["workspace_id"], TENANT_ID);
    }

    #[test]
    fn test_split_handle() {
        let split = |handle| split_handle(handle).ok();
        let pair = |owner: &str, repo: &str| Some((owner.to_string(), repo.to_string()));
        assert_eq!(split("my-prompt"), pair("-", "my-prompt"));
        assert_eq!(split("-/my-prompt"), pair("-", "my-prompt"));
        assert_eq!(split("owner/my-prompt"), pair("owner", "my-prompt"));
        assert_eq!(split(""), None);
        assert_eq!(split("a/b/c"), None);
        assert_eq!(split("/my-prompt"), None);
        assert_eq!(split("owner/"), None);
    }

    #[test]
    fn test_export_takes_a_dashed_handle_without_a_double_dash() {
        use clap::Parser;
        #[derive(Parser)]
        struct Wrapper {
            #[command(subcommand)]
            command: PromptCommands,
        }
        let parsed = Wrapper::try_parse_from([
            "prompt",
            "export",
            "-/my-prompt",
            "--output",
            "my-prompt.json",
        ])
        .expect("clap reads -/my-prompt as the handle");
        let PromptCommands::Export { handle, output, .. } = parsed.command else {
            panic!("not an export");
        };
        assert_eq!(handle, "-/my-prompt");
        assert_eq!(output.as_deref(), Some(Path::new("my-prompt.json")));
    }
}
