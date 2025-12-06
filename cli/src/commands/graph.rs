use crate::config::Config;
use crate::error::{CliError, Result};
use crate::output::{OutputFormat, OutputFormatter};
use clap::Subcommand;
use langstar_sdk::{AuthConfig, GraphSummary, LangchainClient};
use serde_json::json;
use tabled::Tabled;

/// Commands for interacting with LangGraph Graphs
#[derive(Debug, Subcommand)]
pub enum GraphCommands {
    /// List graphs within a deployment
    List {
        /// Deployment name or ID (required)
        deployment: String,

        /// Show graph node details
        #[arg(long)]
        show_nodes: bool,
    },

    /// Get graph structure
    Get {
        /// Graph ID
        graph_id: String,

        /// Deployment name or ID
        #[arg(long, required = true)]
        deployment: String,

        /// Include subgraph details
        #[arg(long)]
        xray: bool,
    },
}

/// Table row for displaying graph summaries
#[derive(Tabled)]
struct GraphRow {
    #[tabled(rename = "Graph ID")]
    graph_id: String,
    #[tabled(rename = "Assistants")]
    assistants: String,
    #[tabled(rename = "# Assists")]
    assistant_count: String,
    #[tabled(rename = "Nodes")]
    nodes: String,
}

impl From<GraphSummary> for GraphRow {
    fn from(summary: GraphSummary) -> Self {
        Self {
            graph_id: summary.graph_id,
            assistants: summary.assistant_names.join(", "),
            assistant_count: summary.assistant_count.to_string(),
            nodes: summary.node_names.join(", "),
        }
    }
}

/// Resolve deployment name or ID to deployment URL
///
/// This function queries the Control Plane API to find a deployment matching
/// the provided name or UUID, then extracts its custom_url for Agent Server API calls.
async fn resolve_deployment_url(config: &Config, deployment_name_or_id: &str) -> Result<String> {
    // Create Control Plane client for deployment lookup
    let auth = AuthConfig::new(
        config.langsmith_api_key.clone(),
        None,
        None,
        config.workspace_id.clone(),
    );
    let client = LangchainClient::new(auth)?;

    // List deployments (limit 100 to catch most cases)
    let deployments_list = client.deployments().list(Some(100), Some(0), None).await?;

    // Find deployment by name or ID
    let deployment = deployments_list
        .resources
        .iter()
        .find(|d| d.name == deployment_name_or_id || d.id == deployment_name_or_id)
        .ok_or_else(|| {
            CliError::Config(format!(
                "Deployment '{}' not found. Run 'langstar deployment list' to see available deployments.",
                deployment_name_or_id
            ))
        })?;

    // Extract custom_url
    deployment.custom_url().ok_or_else(|| {
        CliError::Config(format!(
            "Deployment '{}' has no custom_url in source_config",
            deployment.name
        ))
    })
}

impl GraphCommands {
    /// Execute the graph command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        // Extract deployment name from command
        let deployment_name = match self {
            GraphCommands::List { deployment, .. } => deployment,
            GraphCommands::Get { deployment, .. } => deployment,
        };

        // Resolve deployment to URL
        let deployment_url = resolve_deployment_url(config, deployment_name).await?;

        // Create client with custom deployment URL
        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?.with_langgraph_url(deployment_url);
        let formatter = OutputFormatter::new(format);

        match self {
            GraphCommands::List {
                deployment: _,
                show_nodes,
            } => {
                let summaries = client.graphs().list(*show_nodes).await?;

                match format {
                    OutputFormat::Json => {
                        formatter.print(&summaries)?;
                    }
                    OutputFormat::Table | OutputFormat::Text => {
                        let rows: Vec<GraphRow> = summaries.into_iter().map(Into::into).collect();
                        formatter.print_table(&rows)?;
                    }
                }
            }
            GraphCommands::Get {
                graph_id,
                deployment: _,
                xray,
            } => {
                let graph = client.graphs().get(graph_id, *xray).await?;

                match format {
                    OutputFormat::Json => {
                        formatter.print(&graph)?;
                    }
                    OutputFormat::Table | OutputFormat::Text => {
                        // For table/text format, display a formatted representation
                        let nodes_summary = format!("{} nodes", graph.nodes.len());
                        let edges_summary = format!("{} edges", graph.edges.len());

                        // Extract non-control nodes
                        let node_list: Vec<String> = graph
                            .nodes
                            .iter()
                            .filter(|n| n.id != "__start__" && n.id != "__end__")
                            .map(|n| n.id.clone())
                            .collect();

                        // Display summary using JSON for structured output
                        let summary = json!({
                            "graph_id": graph_id,
                            "nodes_count": graph.nodes.len(),
                            "edges_count": graph.edges.len(),
                            "nodes": node_list,
                            "nodes_summary": nodes_summary,
                            "edges_summary": edges_summary
                        });
                        formatter.print(&summary)?;
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
    use langstar_sdk::GraphSummary;

    #[test]
    fn test_graph_row_conversion() {
        let summary = GraphSummary {
            graph_id: "agent".to_string(),
            assistant_names: vec!["default".to_string(), "custom-v1".to_string()],
            assistant_count: 2,
            node_names: vec!["Responder".to_string(), "Feedback".to_string()],
        };

        let row: GraphRow = summary.into();

        assert_eq!(row.graph_id, "agent");
        assert_eq!(row.assistants, "default, custom-v1");
        assert_eq!(row.assistant_count, "2");
        assert_eq!(row.nodes, "Responder, Feedback");
    }

    #[test]
    fn test_graph_row_empty_lists() {
        let summary = GraphSummary {
            graph_id: "empty".to_string(),
            assistant_names: vec![],
            assistant_count: 0,
            node_names: vec![],
        };

        let row: GraphRow = summary.into();

        assert_eq!(row.graph_id, "empty");
        assert_eq!(row.assistants, "");
        assert_eq!(row.assistant_count, "0");
        assert_eq!(row.nodes, "");
    }
}
