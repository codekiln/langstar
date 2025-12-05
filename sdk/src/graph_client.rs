//! LangGraph Graph Client
//!
//! This module provides access to graph operations in LangGraph Cloud via the Agent Server API.
//!
//! ## Overview
//!
//! Graphs in LangGraph Cloud are not first-class API resources. Instead, they are discovered
//! through assistants, where each assistant has a `graph_id` field. The graph client provides
//! convenient methods to:
//! - List all unique graphs in a deployment by scanning assistants
//! - Get the structure of a specific graph
//! - List subgraphs for a graph
//!
//! ## Deployment-Level Resources
//!
//! Like assistants, graphs are deployment-level resources. They are scoped by API key
//! and the deployment URL you're targeting.
//!
//! ## Usage Example
//!
//! ```no_run
//! use langstar_sdk::{LangchainClient, AuthConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create auth config and client
//!     let auth = AuthConfig::from_env()?;
//!     let client = LangchainClient::new(auth)?
//!         .with_langgraph_url("https://my-deployment.us.langgraph.app".to_string());
//!
//!     // List all unique graphs in the deployment
//!     let graphs = client.graphs().list(true).await?;
//!
//!     for graph_summary in graphs {
//!         println!("Graph: {}", graph_summary.graph_id);
//!         println!("  Assistants: {}", graph_summary.assistant_names.join(", "));
//!         println!("  Nodes: {}", graph_summary.node_names.join(", "));
//!     }
//!
//!     // Get detailed structure of a specific graph
//!     let graph = client.graphs().get("agent", true).await?;
//!     println!("Graph has {} nodes and {} edges", graph.nodes.len(), graph.edges.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## API Reference
//!
//! For detailed API documentation, see:
//! - [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)
//! - [Agent Server API Reference](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref/)

use crate::assistants::{Assistant, AssistantSearchRequest};
use crate::client::LangchainClient;
use crate::error::Result;
use crate::graph::{Graph, GraphSummary};
use std::collections::HashMap;

/// Client for interacting with LangGraph graphs via the Agent Server API
pub struct GraphClient<'a> {
    client: &'a LangchainClient,
}

impl<'a> GraphClient<'a> {
    /// Create a new GraphClient
    pub fn new(client: &'a LangchainClient) -> Self {
        Self { client }
    }

    /// List all unique graphs in a deployment
    ///
    /// This method works by:
    /// 1. Calling POST /assistants/search to get all assistants
    /// 2. Extracting unique graph_id values
    /// 3. Optionally fetching the graph structure for each unique graph
    ///
    /// # Arguments
    /// * `include_structure` - If true, fetches the graph structure for each unique graph
    ///   to populate node names. If false, only returns graph IDs
    ///   and assistant associations.
    ///
    /// # Returns
    /// A vector of `GraphSummary` objects, each representing a unique graph with:
    /// - The graph_id
    /// - Names of all assistants using this graph
    /// - Count of assistants
    /// - Node names (if include_structure is true)
    ///
    /// # Performance Note
    /// This method makes:
    /// - N API calls to paginate through all assistants (where N = ceil(total_assistants / 100))
    /// - M additional API calls to fetch structure for M unique graphs (when `include_structure` is true)
    ///
    /// The graph structure calls are made sequentially. For deployments with many unique graphs,
    /// consider using parallel fetching or calling `get()` individually as needed.
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{LangchainClient, AuthConfig};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = AuthConfig::from_env()?;
    /// # let client = LangchainClient::new(auth)?;
    /// let graphs = client.graphs().list(true).await?;
    /// for graph in graphs {
    ///     println!("{}: {} assistants", graph.graph_id, graph.assistant_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, include_structure: bool) -> Result<Vec<GraphSummary>> {
        // Get all assistants using pagination to ensure complete results
        let mut all_assistants: Vec<Assistant> = Vec::new();
        let limit: u32 = 100; // Reasonable page size
        let mut offset: u32 = 0;

        loop {
            let request_body = AssistantSearchRequest {
                query: None, // Empty query lists all assistants
                limit: Some(limit),
                offset: Some(offset),
            };

            let path = "/assistants/search";
            let request = self.client.langgraph_post(path)?.json(&request_body);
            let mut assistants: Vec<Assistant> = self.client.execute(request).await?;

            let count = assistants.len();
            all_assistants.append(&mut assistants);

            // If we got fewer results than the limit, we've reached the end
            if (count as u32) < limit {
                break;
            }
            offset += limit;
        }

        // Group assistants by graph_id
        let mut graph_map: HashMap<String, Vec<String>> = HashMap::new();
        for assistant in all_assistants {
            graph_map
                .entry(assistant.graph_id)
                .or_default()
                .push(assistant.name);
        }

        // Create graph summaries
        let mut summaries = Vec::new();
        for (graph_id, assistant_names) in graph_map {
            let assistant_count = assistant_names.len();

            // Optionally fetch graph structure to get node names
            let node_names = if include_structure {
                match self.get(&graph_id, true).await {
                    Ok(graph) => graph
                        .nodes
                        .iter()
                        .filter_map(|node| {
                            // Filter out special control nodes
                            if node.id == "__start__" || node.id == "__end__" {
                                None
                            } else {
                                Some(node.id.clone())
                            }
                        })
                        .collect(),
                    Err(_) => Vec::new()
                }
            } else {
                Vec::new()
            };

            summaries.push(GraphSummary {
                graph_id,
                assistant_names,
                assistant_count,
                node_names,
            });
        }

        Ok(summaries)
    }

    /// Get graph structure by graph_id
    ///
    /// Fetches the complete graph topology including nodes and edges.
    /// The graph_id can be obtained from an assistant's `graph_id` field.
    ///
    /// # Arguments
    /// * `graph_id` - The graph ID to fetch
    /// * `xray` - If true, includes subgraph representation (default should be true)
    ///
    /// # Returns
    /// A `Graph` object containing nodes and edges
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{LangchainClient, AuthConfig};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = AuthConfig::from_env()?;
    /// # let client = LangchainClient::new(auth)?;
    /// let graph = client.graphs().get("agent", true).await?;
    /// println!("Graph has {} nodes", graph.nodes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, graph_id: &str, xray: bool) -> Result<Graph> {
        let xray_param = if xray { "?xray=true" } else { "" };
        let path = format!("/assistants/{}/graph{}", graph_id, xray_param);
        let request = self.client.langgraph_get(&path)?;

        let graph: Graph = self.client.execute(request).await?;
        Ok(graph)
    }

    /// Get subgraphs for a graph
    ///
    /// Returns a list of subgraphs that are part of the specified graph.
    ///
    /// # Arguments
    /// * `graph_id` - The graph ID to get subgraphs for
    ///
    /// # Returns
    /// A vector of `Graph` objects representing subgraphs
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{LangchainClient, AuthConfig};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = AuthConfig::from_env()?;
    /// # let client = LangchainClient::new(auth)?;
    /// let subgraphs = client.graphs().subgraphs("agent").await?;
    /// println!("Found {} subgraphs", subgraphs.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subgraphs(&self, graph_id: &str) -> Result<Vec<Graph>> {
        let path = format!("/assistants/{}/subgraphs", graph_id);
        let request = self.client.langgraph_get(&path)?;

        let subgraphs: Vec<Graph> = self.client.execute(request).await?;
        Ok(subgraphs)
    }
}

impl LangchainClient {
    /// Get a GraphClient for interacting with graphs
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{LangchainClient, AuthConfig};
    /// # let auth = AuthConfig::from_env().unwrap();
    /// let client = LangchainClient::new(auth).unwrap();
    /// let graph_client = client.graphs();
    /// ```
    pub fn graphs(&self) -> GraphClient<'_> {
        GraphClient::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthConfig;
    use crate::graph::{Graph, GraphEdge, GraphNode, GraphNodeData};

    #[test]
    fn test_graph_client_creation() {
        let auth = AuthConfig::new(None, Some("test".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let _graph_client = client.graphs();
    }

    #[test]
    fn test_graph_summary_aggregation() {
        // Test that GraphSummary can be created with expected data
        let summary = GraphSummary {
            graph_id: "agent".to_string(),
            assistant_names: vec!["default".to_string(), "custom-v1".to_string()],
            assistant_count: 2,
            node_names: vec!["Responder".to_string(), "Feedback".to_string()],
        };

        assert_eq!(summary.graph_id, "agent");
        assert_eq!(summary.assistant_count, 2);
        assert_eq!(summary.assistant_names.len(), 2);
        assert_eq!(summary.node_names.len(), 2);
    }

    #[test]
    fn test_graph_summary_serialization() {
        let summary = GraphSummary {
            graph_id: "test_graph".to_string(),
            assistant_names: vec!["assistant1".to_string()],
            assistant_count: 1,
            node_names: vec!["node1".to_string(), "node2".to_string()],
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("test_graph"));
        assert!(json.contains("assistant1"));

        // Test deserialization
        let deserialized: GraphSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.graph_id, summary.graph_id);
        assert_eq!(deserialized.assistant_count, summary.assistant_count);
    }

    #[test]
    fn test_filter_control_nodes() {
        // Test that __start__ and __end__ nodes would be filtered correctly
        let graph = Graph {
            nodes: vec![
                GraphNode {
                    id: "__start__".to_string(),
                    node_type: Some("runnable".to_string()),
                    data: Some(GraphNodeData {
                        name: Some("__start__".to_string()),
                    }),
                },
                GraphNode {
                    id: "Responder".to_string(),
                    node_type: Some("runnable".to_string()),
                    data: Some(GraphNodeData {
                        name: Some("Responder".to_string()),
                    }),
                },
                GraphNode {
                    id: "__end__".to_string(),
                    node_type: None,
                    data: None,
                },
            ],
            edges: vec![GraphEdge {
                source: "__start__".to_string(),
                target: "Responder".to_string(),
                conditional: false,
            }],
        };

        // Simulate the filtering logic used in list()
        let filtered_nodes: Vec<String> = graph
            .nodes
            .iter()
            .filter_map(|node| {
                if node.id == "__start__" || node.id == "__end__" {
                    None
                } else {
                    Some(node.id.clone())
                }
            })
            .collect();

        assert_eq!(filtered_nodes.len(), 1);
        assert_eq!(filtered_nodes[0], "Responder");
    }

    #[test]
    fn test_client_with_custom_url() {
        let auth = AuthConfig::new(None, Some("test".to_string()), None, None);
        let client = LangchainClient::new(auth)
            .unwrap()
            .with_langgraph_url("https://custom-deployment.us.langgraph.app".to_string());

        let _graph_client = client.graphs();
        // This test verifies that the client can be configured with a custom URL
        // and the graph client can be created from it
    }
}
