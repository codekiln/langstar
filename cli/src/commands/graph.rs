use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{
    Deployment, DeploymentFilters, DeploymentStatus, DeploymentType, LangchainClient,
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
