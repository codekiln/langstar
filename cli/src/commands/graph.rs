use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{
    CreateDeploymentRequest, Deployment, DeploymentFilters, DeploymentStatus, DeploymentType,
    LangchainClient,
};
use serde_json::json;
use tabled::Tabled;

/// Commands for interacting with LangGraph deployments via Control Plane API
#[derive(Debug, Subcommand)]
pub enum GraphCommands {
    /// List all LangGraph deployments
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
    },

    /// Get a specific deployment by ID
    Get {
        /// Deployment ID
        deployment_id: String,
    },

    /// Create a new LangGraph deployment
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

        /// GitHub integration ID (for github source, optional - will auto-discover from existing deployments if not provided)
        #[arg(long)]
        integration_id: Option<String>,

        /// Path to langgraph.json config file in repository (for github source)
        #[arg(long, default_value = "langgraph.json")]
        config_path: String,

        /// Docker image URI (for external_docker source)
        /// Example: ghcr.io/owner/repo:tag
        #[arg(long)]
        image_uri: Option<String>,

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

impl GraphCommands {
    /// Execute the graph command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;
        let formatter = OutputFormatter::new(format);

        match self {
            GraphCommands::List {
                limit,
                offset,
                deployment_type,
                status,
                name_contains,
            } => {
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
                        "UNKNOWN" => DeploymentStatus::Unknown,
                        _ => {
                            return Err(crate::error::CliError::Config(format!(
                                "Invalid status: {}. Valid values: READY, AWAITING_DATABASE, UNUSED, AWAITING_DELETE, UNKNOWN",
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
                if format == OutputFormat::Json {
                    formatter.print(&json!({
                        "resources": deployments_list.resources,
                        "offset": deployments_list.offset
                    }))?;
                } else if deployments_list.resources.is_empty() {
                    formatter.info("No deployments found.");
                } else {
                    let rows: Vec<DeploymentRow> = deployments_list
                        .resources
                        .iter()
                        .map(|d| d.into())
                        .collect();
                    formatter.print_table(&rows)?;
                    formatter.info(&format!(
                        "\nTotal: {} deployment(s) (offset: {})",
                        deployments_list.resources.len(),
                        deployments_list.offset
                    ));
                }

                Ok(())
            }

            GraphCommands::Get { deployment_id } => {
                formatter.info(&format!("Fetching deployment '{}'...", deployment_id));

                let deployment = client.deployments().get(deployment_id).await?;

                // Output in JSON format
                formatter.print(&serde_json::to_value(&deployment)?)?;

                Ok(())
            }

            GraphCommands::Create {
                name,
                source,
                repo_url,
                branch,
                integration_id,
                config_path,
                image_uri,
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

                // Determine integration_id with precedence: CLI flag > config/env > auto-discovery
                let integration_id = if source == "github" {
                    // 1. CLI flag (highest priority)
                    if let Some(id) = integration_id {
                        formatter.info("Using GitHub integration ID from command line");
                        Some(id.clone())
                    }
                    // 2. Config/env var
                    else if let Some(id) = &config.github_integration_id {
                        formatter.info("Using GitHub integration ID from config/environment");
                        Some(id.clone())
                    }
                    // 3. Auto-discovery (fallback for backward compatibility)
                    else {
                        formatter
                            .info("Looking up GitHub integration ID from existing deployments...");

                        // Query existing deployments to find integration_id
                        let existing = client.deployments().list(Some(100), Some(0), None).await?;

                        // Find first GitHub deployment and extract integration_id
                        let github_deployment = existing.resources.iter().find(|d| {
                            d.source == langstar_sdk::DeploymentSource::Github
                                && d.source_config.is_some()
                        });

                        if let Some(deployment) = github_deployment {
                            if let Some(source_config) = &deployment.source_config {
                                if let Some(id) =
                                    source_config.get("integration_id").and_then(|v| v.as_str())
                                {
                                    formatter.info(&format!("Found GitHub integration ID: {}", id));
                                    Some(id.to_string())
                                } else {
                                    return Err(crate::error::CliError::Config(
                                        "Found GitHub deployment but integration_id is missing from source_config".to_string()
                                    ));
                                }
                            } else {
                                None
                            }
                        } else {
                            // No existing deployments found - provide helpful error
                            return Err(crate::error::CliError::Config(
                                "GitHub integration ID not found. Please provide it via:\n\
                                1. CLI flag: --integration-id <your-integration-id>\n\
                                2. Environment variable: LANGGRAPH_GITHUB_INTEGRATION_ID=<your-integration-id>\n\
                                3. Config file: github_integration_id = \"<your-integration-id>\"\n\n\
                                To get your integration ID:\n\
                                1. Log in to LangSmith UI (https://smith.langchain.com/)\n\
                                2. Navigate to Deployments → + New Deployment\n\
                                3. Click 'Import from GitHub' and authorize the 'hosted-langserve' GitHub app\n\
                                4. After setup, you can find your integration ID in existing deployment configs".to_string()
                            ));
                        }
                    }
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
                        let image = image_uri.as_ref().ok_or_else(|| {
                            crate::error::CliError::Config(
                                "image_uri is required for external_docker source".to_string(),
                            )
                        })?;
                        // For external_docker, integration_id must be null, image_path is required
                        json!({
                            "integration_id": null,
                            "image_path": image
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
                let mut request = CreateDeploymentRequest::new(
                    name.clone(),
                    source.clone(),
                    source_config,
                    deployment_type.clone(),
                )
                .with_source_revision_config(source_revision_config);

                if !env_vars.is_empty() {
                    request = request.with_env_vars(env_vars);
                }

                // Execute the creation
                let mut deployment = client.deployments().create(request).await?;

                if format == OutputFormat::Json && !*wait {
                    formatter.print(&deployment)?;
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
                        formatter.print(&deployment)?;
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

            GraphCommands::Delete { deployment_id, yes } => {
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
                    io::stdout().flush().unwrap();
                    let mut confirmation = String::new();
                    io::stdin().read_line(&mut confirmation).unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;

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
