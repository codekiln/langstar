use crate::config::Config;
use crate::error::Result;
use crate::output::{ColumnMetadata, OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{
    CreateDeploymentRequest, Deployment, DeploymentFilters, DeploymentStatus, DeploymentType,
    LangchainClient,
};
use serde_json::json;
use tabled::Tabled;

/// Available columns for deployment list text output
/// Note: deployment_type is not included because the API doesn't provide this field
const DEPLOYMENT_COLUMNS: &[&str] = &["name", "id", "status", "source", "created_at"];

/// Commands for interacting with LangGraph deployments via Control Plane API
///
/// # Security Note
/// All commands sanitize secret values in their output. Secret values are replaced
/// with "<redacted>" to prevent accidental exposure in logs, terminal output,
/// or shared command output. This applies to all output formats (JSON and table).
#[derive(Debug, Subcommand)]
pub enum DeploymentCommands {
    /// List all LangGraph deployments
    ///
    /// Note: Secret values in deployment output are automatically redacted for security.
    List {
        /// Maximum number of deployments to return
        #[arg(short, long, default_value = "20")]
        limit: u32,

        /// Number of deployments to skip (pagination)
        #[arg(long, default_value = "0")]
        offset: u32,

        /// Filter by deployment type (dev_free, dev, prod)
        #[arg(long)]
        deployment_type: Option<String>,

        /// Filter by deployment status (READY, AWAITING_DATABASE, etc.)
        #[arg(long)]
        status: Option<String>,

        /// Filter by name (substring match)
        #[arg(long)]
        name_contains: Option<String>,

        /// Select specific columns for text output (comma-separated)
        /// Available: name, id, status, source, created_at
        #[arg(long, value_delimiter = ',')]
        columns: Option<Vec<String>>,

        /// Show available columns for text output
        #[arg(long)]
        show_columns: bool,
    },

    /// Get a specific deployment by ID
    ///
    /// Note: Secret values in deployment output are automatically redacted for security.
    Get {
        /// Deployment ID
        deployment_id: String,
    },

    /// Create a new LangGraph deployment
    ///
    /// Note: Secret values in deployment output are automatically redacted for security.
    Create {
        /// Name of the deployment
        #[arg(short, long)]
        name: String,

        /// Source type (github or external_docker)
        #[arg(short, long, default_value = "github")]
        source: String,

        /// Repository URL (for github source)
        #[arg(long)]
        repo_url: Option<String>,

        /// Git branch (for github source)
        #[arg(long)]
        branch: Option<String>,

        /// GitHub integration ID (for github source). Optional: without it, the CLI asks the
        /// Control Plane API which GitHub integration can access --repo-url
        #[arg(long)]
        integration_id: Option<String>,

        /// Path to langgraph.json config file in repository (for github source)
        #[arg(long, default_value = "langgraph.json")]
        config_path: String,

        /// Deployment type (dev_free, dev, or prod)
        #[arg(short = 't', long, default_value = "dev_free")]
        deployment_type: String,

        /// Environment variables (KEY=VALUE format, can be specified multiple times)
        #[arg(short, long)]
        env: Vec<String>,

        /// Wait for deployment to reach READY status
        #[arg(short, long)]
        wait: bool,
    },

    /// Delete a LangGraph deployment by ID
    Delete {
        /// Deployment ID to delete
        deployment_id: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Simplified deployment info for table display
#[derive(Debug, Tabled)]
struct DeploymentRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Type")]
    deployment_type: String,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

impl From<&Deployment> for DeploymentRow {
    fn from(deployment: &Deployment) -> Self {
        // Truncate long IDs for readability
        let id = if deployment.id.len() > 20 {
            format!("{}...", &deployment.id[..17])
        } else {
            deployment.id.clone()
        };

        // Truncate long names
        let name = if deployment.name.len() > 30 {
            format!("{}...", &deployment.name[..27])
        } else {
            deployment.name.clone()
        };

        // Format status nicely
        let status = format!("{:?}", deployment.status);

        // Try to infer deployment type from other fields (not directly in response)
        // For now, show "N/A" - this could be enhanced later
        let deployment_type = "N/A".to_string();

        // Format source
        let source = format!("{:?}", deployment.source);

        // Extract date from created_at (YYYY-MM-DD)
        let created_at = deployment
            .created_at
            .split('T')
            .next()
            .unwrap_or("N/A")
            .to_string();

        Self {
            name,
            id,
            status,
            deployment_type,
            source,
            created_at,
        }
    }
}

/// Implement ColumnMetadata for Deployment to support text output
impl ColumnMetadata for Deployment {
    fn available_columns() -> Vec<&'static str> {
        DEPLOYMENT_COLUMNS.to_vec()
    }

    fn render_tsv(&self, columns: &[String]) -> String {
        columns
            .iter()
            .map(|col| match col.as_str() {
                "name" => self.name.replace(['\t', '\n'], " "),
                "id" => self.id.clone(),
                "status" => format!("{:?}", self.status),
                "source" => format!("{:?}", self.source),
                "created_at" => self
                    .created_at
                    .split('T')
                    .next()
                    .unwrap_or("N/A")
                    .to_string(),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join("\t")
    }
}

impl DeploymentCommands {
    /// Execute the deployment command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;
        let formatter = OutputFormatter::new(format);

        match self {
            DeploymentCommands::List {
                limit,
                offset,
                deployment_type,
                status,
                name_contains,
                columns,
                show_columns,
            } => {
                // Handle --show-columns flag: display available columns and exit
                if *show_columns {
                    println!("Available columns for deployment list:");
                    for col in DEPLOYMENT_COLUMNS {
                        println!("  {}", col);
                    }
                    println!("\nUsage: langstar deployment list -f text --columns name,id,status");
                    return Ok(());
                }

                // Validate --columns if provided
                if let Some(cols) = columns {
                    for col in cols {
                        if !DEPLOYMENT_COLUMNS.contains(&col.as_str()) {
                            return Err(crate::error::CliError::Config(format!(
                                "Invalid column '{}'. Available columns: {}",
                                col,
                                DEPLOYMENT_COLUMNS.join(", ")
                            )));
                        }
                    }
                }

                formatter.info(&format!(
                    "Fetching deployments (limit: {}, offset: {})...",
                    limit, offset
                ));

                // Build filters
                let mut filters = DeploymentFilters::default();

                if let Some(name) = name_contains {
                    filters.name_contains = Some(name.clone());
                }

                if let Some(status_str) = status {
                    let parsed_status = match status_str.to_uppercase().as_str() {
                        "READY" => DeploymentStatus::Ready,
                        "AWAITING_DATABASE" => DeploymentStatus::AwaitingDatabase,
                        "UNUSED" => DeploymentStatus::Unused,
                        "AWAITING_DELETE" => DeploymentStatus::AwaitingDelete,
                        "AWAITING_FINAL_DELETE" => DeploymentStatus::AwaitingFinalDelete,
                        "UNKNOWN" => DeploymentStatus::Unknown,
                        _ => {
                            return Err(crate::error::CliError::Config(format!(
                                "Invalid status: {}. Valid values: READY, AWAITING_DATABASE, UNUSED, AWAITING_DELETE, AWAITING_FINAL_DELETE, UNKNOWN",
                                status_str
                            )));
                        }
                    };
                    filters.status = Some(parsed_status);
                }

                if let Some(type_str) = deployment_type {
                    let parsed_type = match type_str.to_lowercase().as_str() {
                        "dev_free" => DeploymentType::DevFree,
                        "dev" => DeploymentType::Dev,
                        "prod" => DeploymentType::Prod,
                        _ => {
                            return Err(crate::error::CliError::Config(format!(
                                "Invalid deployment type: {}. Valid values: dev_free, dev, prod",
                                type_str
                            )));
                        }
                    };
                    filters.deployment_type = Some(parsed_type);
                }

                let filters_option = if filters.name_contains.is_some()
                    || filters.status.is_some()
                    || filters.deployment_type.is_some()
                {
                    Some(filters)
                } else {
                    None
                };

                // Fetch deployments
                let deployments_list = client
                    .deployments()
                    .list(Some(*limit), Some(*offset), filters_option)
                    .await?;

                // Output results
                match format {
                    OutputFormat::Json => {
                        // Sanitize secrets before outputting
                        let sanitized_resources: Vec<Deployment> = deployments_list
                            .resources
                            .iter()
                            .map(|d| d.sanitize_secrets())
                            .collect();
                        formatter.print(&json!({
                            "resources": sanitized_resources,
                            "offset": deployments_list.offset
                        }))?;
                    }
                    OutputFormat::Text => {
                        if deployments_list.resources.is_empty() {
                            formatter.info("No deployments found.");
                        } else {
                            // Sanitize secrets before outputting
                            let sanitized_resources: Vec<Deployment> = deployments_list
                                .resources
                                .iter()
                                .map(|d| d.sanitize_secrets())
                                .collect();
                            formatter.print_text(&sanitized_resources, columns.as_deref())?;
                        }
                    }
                    OutputFormat::Table => {
                        if deployments_list.resources.is_empty() {
                            formatter.info("No deployments found.");
                        } else {
                            // Sanitize secrets before creating rows for consistency
                            let rows: Vec<DeploymentRow> = deployments_list
                                .resources
                                .iter()
                                .map(|d| d.sanitize_secrets())
                                .map(|d| (&d).into())
                                .collect();
                            formatter.print_table(&rows)?;
                            formatter.info(&format!(
                                "\nTotal: {} deployment(s) (offset: {})",
                                deployments_list.resources.len(),
                                deployments_list.offset
                            ));
                        }
                    }
                }

                Ok(())
            }

            DeploymentCommands::Get { deployment_id } => {
                formatter.info(&format!("Fetching deployment '{}'...", deployment_id));

                let deployment = client.deployments().get(deployment_id).await?;

                // Sanitize secrets before outputting
                let sanitized = deployment.sanitize_secrets();
                formatter.print(&serde_json::to_value(&sanitized)?)?;

                Ok(())
            }

            DeploymentCommands::Create {
                name,
                source,
                repo_url,
                branch,
                integration_id,
                config_path,
                deployment_type,
                env,
                wait,
            } => {
                formatter.info(&format!("Creating deployment '{}'...", name));

                // Parse environment variables
                let mut env_vars = std::collections::HashMap::new();
                for env_str in env {
                    if let Some((key, value)) = env_str.split_once('=') {
                        env_vars.insert(key.to_string(), value.to_string());
                    } else {
                        return Err(crate::error::CliError::Config(format!(
                            "Invalid environment variable format: {}. Expected KEY=VALUE",
                            env_str
                        )));
                    }
                }

                // Find the GitHub integration: --integration-id wins, otherwise the
                // Control Plane API names the integration with access to the repository.
                let integration_id = if source == "github" {
                    let repo = repo_url.as_deref().ok_or_else(|| {
                        crate::error::CliError::Config(
                            "repo_url is required for github source".to_string(),
                        )
                    })?;
                    if integration_id.is_some() {
                        formatter.info("Using the GitHub integration ID from --integration-id");
                    } else {
                        formatter.info(
                            "Looking up the GitHub integration with access to the repository...",
                        );
                    }
                    Some(resolve_integration_id(&client, integration_id.as_deref(), repo).await?)
                } else {
                    None
                };

                // Build source_config based on source type
                let source_config = match source.as_str() {
                    "github" => {
                        let repo = repo_url.as_ref().ok_or_else(|| {
                            crate::error::CliError::Config(
                                "repo_url is required for github source".to_string(),
                            )
                        })?;
                        // Validate branch is present
                        if branch.is_none() {
                            return Err(crate::error::CliError::Config(
                                "branch is required for github source".to_string(),
                            ));
                        }

                        // Include integration_id for GitHub sources
                        json!({
                            "integration_id": integration_id,
                            "repo_url": repo,
                            "deployment_type": deployment_type,
                            "build_on_push": false,
                            "custom_url": null,
                            "resource_spec": null,
                        })
                    }
                    "external_docker" => {
                        // For external_docker, integration_id must be null
                        json!({
                            "integration_id": null
                        })
                    }
                    _ => {
                        return Err(crate::error::CliError::Config(format!(
                            "Invalid source type: {}. Valid values: github, external_docker",
                            source
                        )));
                    }
                };

                // Build source_revision_config based on source type
                let source_revision_config = match source.as_str() {
                    "github" => {
                        let branch = branch.as_ref().unwrap(); // Already validated above
                        json!({
                            "repo_ref": branch,
                            "langgraph_config_path": config_path
                        })
                    }
                    _ => json!(null), // null for non-github sources
                };

                // Create the request
                // Convert env_vars HashMap to Vec<DeploymentSecret>
                use langstar_sdk::DeploymentSecret;
                let secrets: Vec<DeploymentSecret> = env_vars
                    .into_iter()
                    .map(|(name, value)| DeploymentSecret { name, value })
                    .collect();

                let request = CreateDeploymentRequest {
                    name: name.clone(),
                    source: source.clone(),
                    source_config,
                    source_revision_config,
                    secrets,
                };

                // Execute the creation
                let mut deployment = client.deployments().create(&request).await?;

                if format == OutputFormat::Json && !*wait {
                    // Sanitize secrets before outputting
                    formatter.print(&deployment.sanitize_secrets())?;
                } else if !*wait {
                    formatter.success(&format!(
                        "Created deployment: {} (ID: {})",
                        name, deployment.id
                    ));
                    formatter.info(&format!("Status: {:?}", deployment.status));
                }

                // Poll for READY status if --wait flag is set
                if *wait {
                    formatter.info("⏳ Waiting for deployment to be ready...");

                    let start_time = std::time::Instant::now();
                    let mut poll_count = 0;

                    loop {
                        // Check current status
                        if deployment.status == DeploymentStatus::Ready {
                            break;
                        }

                        // Determine polling interval based on elapsed time
                        let elapsed = start_time.elapsed().as_secs();
                        let poll_interval = if elapsed < 30 {
                            // First 30 seconds: poll every 10 seconds
                            std::time::Duration::from_secs(10)
                        } else {
                            // After 30 seconds: poll every 30 seconds
                            std::time::Duration::from_secs(30)
                        };

                        poll_count += 1;
                        formatter.info(&format!(
                            "⏳ Status: {:?} (check #{}, elapsed: {}s)",
                            deployment.status, poll_count, elapsed
                        ));

                        // Wait before next poll
                        tokio::time::sleep(poll_interval).await;

                        // Fetch updated deployment status
                        deployment = client.deployments().get(&deployment.id).await?;
                    }

                    // Deployment is ready
                    if format == OutputFormat::Json {
                        // Sanitize secrets before outputting
                        formatter.print(&deployment.sanitize_secrets())?;
                    } else {
                        formatter.success(&format!(
                            "✓ Deployment ready: {} (ID: {})",
                            name, deployment.id
                        ));
                        formatter.info(&format!("Status: {:?}", deployment.status));
                        formatter.info(&format!(
                            "Total wait time: {}s",
                            start_time.elapsed().as_secs()
                        ));
                    }
                }

                Ok(())
            }

            DeploymentCommands::Delete { deployment_id, yes } => {
                // Confirmation prompt (unless --yes is provided)
                if !yes {
                    formatter.info(&format!(
                        "Are you sure you want to delete deployment '{}'?",
                        deployment_id
                    ));
                    formatter.info("This action cannot be undone. Use --yes to skip this prompt.");

                    // Read from stdin
                    use std::io::{self, Write};
                    print!("Type 'yes' to confirm: ");
                    io::stdout().flush()?;
                    let mut confirmation = String::new();
                    io::stdin().read_line(&mut confirmation)?;

                    if confirmation.trim().to_lowercase() != "yes" {
                        formatter.info("Deletion cancelled.");
                        return Ok(());
                    }
                }

                formatter.info(&format!("Deleting deployment '{}'...", deployment_id));

                // Execute the deletion
                client.deployments().delete(deployment_id).await?;

                if format == OutputFormat::Json {
                    formatter.print(&json!({
                        "status": "deleted",
                        "deployment_id": deployment_id
                    }))?;
                } else {
                    formatter.success(&format!(
                        "Successfully deleted deployment: {}",
                        deployment_id
                    ));
                }

                Ok(())
            }
        }
    }
}

/// Read the owner and repository name from a GitHub URL such as
/// `https://github.com/<owner>/<repo>` or `git@github.com:<owner>/<repo>.git`.
fn parse_github_repo(repo_url: &str) -> Option<(String, String)> {
    let trimmed = repo_url.trim().trim_end_matches('/');
    let trimmed = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    let path = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("git@github.com:"))?;
    let mut parts = path.split('/');
    let owner = parts.next().filter(|s| !s.is_empty())?;
    let repo = parts.next().filter(|s| !s.is_empty())?;
    if parts.next().is_some() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

/// Find the GitHub integration ID for a `github` source deployment.
///
/// `--integration-id` wins. Otherwise the Control Plane API lists the workspace's
/// GitHub integrations (`GET /v1/integrations/github/install`, the endpoint the
/// spec names for `integration_id`) and the one whose repositories include
/// `repo_url` is used. No environment variable or config key holds this ID.
async fn resolve_integration_id(
    client: &LangchainClient,
    flag: Option<&str>,
    repo_url: &str,
) -> Result<String> {
    if let Some(id) = flag {
        return Ok(id.to_string());
    }
    let (owner, repo) = parse_github_repo(repo_url).ok_or_else(|| {
        crate::error::CliError::Config(format!(
            "Cannot read the GitHub owner and repository from --repo-url {}. \
             Use https://github.com/<owner>/<repo>, or pass --integration-id.",
            repo_url
        ))
    })?;
    client
        .integrations()
        .find_integration_for_repo(&owner, &repo)
        .await
        .map_err(|e| match e {
            langstar_sdk::LangstarError::ApiError { status: 404, .. } => {
                crate::error::CliError::Config(format!(
                    "No GitHub integration in this workspace has access to {}/{}. \
                     In LangSmith, open Deployments, choose + New Deployment, then \
                     Import from GitHub, and give the 'hosted-langserve' GitHub app \
                     access to that repository. Or pass --integration-id.",
                    owner, repo
                ))
            }
            other => other.into(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_repo() {
        let expected = Some(("codekiln".to_string(), "langstar".to_string()));
        for url in [
            "https://github.com/codekiln/langstar",
            "https://github.com/codekiln/langstar/",
            "https://github.com/codekiln/langstar.git",
            "git@github.com:codekiln/langstar.git",
        ] {
            assert_eq!(parse_github_repo(url), expected, "url {}", url);
        }
        for url in [
            "https://gitlab.com/codekiln/langstar",
            "https://github.com/codekiln",
            "https://github.com/codekiln/langstar/tree/main",
        ] {
            assert_eq!(parse_github_repo(url), None, "url {}", url);
        }
    }

    fn mock_client(server: &mockito::ServerGuard) -> LangchainClient {
        LangchainClient::with_base_urls(
            langstar_sdk::AuthConfig::new(
                Some("test_key".to_string()),
                None,
                Some("test_workspace".to_string()),
            ),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_resolve_integration_id_flag_skips_api() {
        let mut server = mockito::Server::new_async().await;
        let install = server
            .mock("GET", "/v1/integrations/github/install")
            .expect(0)
            .create_async()
            .await;
        let client = mock_client(&server);

        let id = resolve_integration_id(
            &client,
            Some("flag-integration"),
            "https://github.com/codekiln/langstar",
        )
        .await
        .unwrap();

        assert_eq!(id, "flag-integration");
        install.assert_async().await;
    }

    #[tokio::test]
    async fn test_resolve_integration_id_finds_repo_through_api() {
        let mut server = mockito::Server::new_async().await;
        let install = server
            .mock("GET", "/v1/integrations/github/install")
            .match_header("x-tenant-id", "test_workspace")
            .with_body(
                r#"[{"id":"other-integration","installation_id":1,"name":"other"},
                    {"id":"langstar-integration","installation_id":2,"name":"codekiln"}]"#,
            )
            .create_async()
            .await;
        let other_repos = server
            .mock("GET", "/v1/integrations/github/other-integration/repos")
            .with_body(r#"[{"owner":"someone","name":"elsewhere"}]"#)
            .create_async()
            .await;
        let langstar_repos = server
            .mock("GET", "/v1/integrations/github/langstar-integration/repos")
            .with_body(r#"[{"owner":"codekiln","name":"langstar"}]"#)
            .create_async()
            .await;
        let client = mock_client(&server);

        let id = resolve_integration_id(&client, None, "https://github.com/codekiln/langstar.git")
            .await
            .unwrap();

        assert_eq!(id, "langstar-integration");
        install.assert_async().await;
        other_repos.assert_async().await;
        langstar_repos.assert_async().await;
    }

    #[tokio::test]
    async fn test_resolve_integration_id_reports_repo_without_integration() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/v1/integrations/github/install")
            .with_body(r#"[{"id":"other-integration","installation_id":1,"name":"other"}]"#)
            .create_async()
            .await;
        server
            .mock("GET", "/v1/integrations/github/other-integration/repos")
            .with_body(r#"[{"owner":"someone","name":"elsewhere"}]"#)
            .create_async()
            .await;
        let client = mock_client(&server);

        let err = resolve_integration_id(&client, None, "https://github.com/codekiln/langstar")
            .await
            .unwrap_err();

        let message = err.to_string();
        assert!(
            message.contains(
                "No GitHub integration in this workspace has access to codekiln/langstar"
            ),
            "unexpected error: {}",
            message
        );
        assert!(
            message.contains("--integration-id"),
            "unexpected error: {}",
            message
        );
    }

    #[tokio::test]
    async fn test_resolve_integration_id_rejects_unparseable_repo_url() {
        let mut server = mockito::Server::new_async().await;
        let install = server
            .mock("GET", "/v1/integrations/github/install")
            .expect(0)
            .create_async()
            .await;
        let client = mock_client(&server);

        let err = resolve_integration_id(&client, None, "https://gitlab.com/codekiln/langstar")
            .await
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("Cannot read the GitHub owner and repository")
        );
        install.assert_async().await;
    }

    #[test]
    fn test_deployment_row_truncation() {
        let deployment = Deployment {
            id: "abc-123e4567-e89b-12d3-a456-426614174000".to_string(),
            name: "this-is-a-very-long-deployment-name-that-should-be-truncated".to_string(),
            source: langstar_sdk::DeploymentSource::Github,
            source_config: None,
            source_revision_config: None,
            secrets: None,
            created_at: "2024-01-15T10:30:00Z".to_string(),
            updated_at: "2024-01-16T12:00:00Z".to_string(),
            status: langstar_sdk::DeploymentStatus::Ready,
            latest_revision_id: None,
            active_revision_id: None,
            image_version: None,
        };

        let row = DeploymentRow::from(&deployment);

        // ID should be truncated to 20 chars (17 + "...")
        assert_eq!(row.id.len(), 20);
        assert!(row.id.ends_with("..."));

        // Name should be truncated to 30 chars (27 + "...")
        assert_eq!(row.name.len(), 30);
        assert!(row.name.ends_with("..."));

        // Created date should be extracted
        assert_eq!(row.created_at, "2024-01-15");
    }
}
