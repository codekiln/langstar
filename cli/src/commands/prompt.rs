use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::prompts::{
    LcJson, MessagePromptTemplateKwargs, PromptTemplateKwargs, StructuredOutputKwargs,
    StructuredPrompt, validate_json_schema, validate_method,
};
use langstar_sdk::{CommitRequest, LangchainClient, Prompt, Visibility};
use serde_json::json;
use std::fs;
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

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,

        /// Workspace ID for narrower scoping (overrides config/env)
        #[arg(long)]
        workspace_id: Option<String>,

        /// Show only public prompts (default: private when scoped, any when not scoped)
        #[arg(long)]
        public: bool,
    },

    /// Get details of a specific prompt
    Get {
        /// Prompt handle (e.g., "owner/prompt-name")
        handle: String,

        /// Organization ID for scoping (overrides config/env)
        #[arg(long)]
        organization_id: Option<String>,

        /// Workspace ID for narrower scoping (overrides config/env)
        #[arg(long)]
        workspace_id: Option<String>,
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

        /// Workspace ID for narrower scoping (overrides config/env)
        #[arg(long)]
        workspace_id: Option<String>,

        /// Show only public prompts (default: private when scoped, any when not scoped)
        #[arg(long)]
        public: bool,
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

        /// Workspace ID for narrower scoping (overrides config/env)
        #[arg(long)]
        workspace_id: Option<String>,
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

        /// Workspace ID for narrower scoping (overrides config/env)
        #[arg(long)]
        workspace_id: Option<String>,
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
                workspace_id,
                public,
            } => {
                let client = Self::apply_scoping(client, organization_id, workspace_id, config);
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

                if format == OutputFormat::Json {
                    formatter.print(&prompts)?;
                } else {
                    let rows: Vec<PromptRow> = prompts.iter().map(PromptRow::from).collect();
                    formatter.print_table(&rows)?;
                    println!("\nFound {} prompts", prompts.len());

                    // Show hint if scoped and no results
                    if prompts.is_empty()
                        && (client.organization_id().is_some() || client.workspace_id().is_some())
                        && !*public
                    {
                        eprintln!("\n💡 Hint: No private prompts found in this scope.");
                        eprintln!("  Try using --public flag to see public prompts:");
                        eprintln!("    langstar prompt list --public");
                    }
                }
            }

            PromptCommands::Get {
                handle,
                organization_id,
                workspace_id,
            } => {
                let client = Self::apply_scoping(client, organization_id, workspace_id, config);
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
                workspace_id,
                public,
            } => {
                let client = Self::apply_scoping(client, organization_id, workspace_id, config);
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
                } else {
                    let rows: Vec<PromptRow> = prompts.iter().map(PromptRow::from).collect();
                    formatter.print_table(&rows)?;
                    println!("\nFound {} prompts", prompts.len());

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
                workspace_id,
            } => {
                // Apply scoping from flags/config
                let mut client = Self::apply_scoping(client, organization_id, workspace_id, config);

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

                match client.prompts().get(&repo_handle).await {
                    Ok(_) => {
                        println!("✓ Repository exists");
                    }
                    Err(_) => {
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
                            }
                            Err(e) => {
                                eprintln!("⚠ Warning: Could not create repository: {}", e);
                                eprintln!("  Will attempt to push anyway...");
                            }
                        }
                    }
                }

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
                        .push_structured_prompt(owner, repo, structured_prompt, None)
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
                        parent_commit: None,
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
                workspace_id,
            } => {
                let client = Self::apply_scoping(client, organization_id, workspace_id, config);

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
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use langstar_sdk::AuthConfig;

    #[test]
    fn test_apply_scoping_with_no_flags() {
        // Client with no scoping
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None, None);
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
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None, None);
        let client = LangchainClient::new(auth).unwrap();
        let config = Config::default();

        let org_id = Some("test-org-id".to_string());
        let scoped_client = PromptCommands::apply_scoping(client, &org_id, &None, &config);

        assert_eq!(scoped_client.organization_id(), Some("test-org-id"));
        assert_eq!(scoped_client.workspace_id(), None);
    }

    #[test]
    fn test_apply_scoping_with_workspace_flag() {
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None, None);
        let client = LangchainClient::new(auth).unwrap();
        let config = Config::default();

        let workspace_id = Some("test-workspace-id".to_string());
        let scoped_client = PromptCommands::apply_scoping(client, &None, &workspace_id, &config);

        assert_eq!(scoped_client.organization_id(), None);
        assert_eq!(scoped_client.workspace_id(), Some("test-workspace-id"));
    }

    #[test]
    fn test_apply_scoping_with_both_flags() {
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None, None);
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
            None,
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
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None, None);
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
            None,
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
            None,
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
}
