//! LangGraph Graph Structures
//!
//! This module provides types for representing LangGraph graph topology.
//! These structures represent the nodes and edges returned by the
//! `/assistants/{id}/graph` endpoint.
//!
//! ## Overview
//!
//! A LangGraph graph consists of:
//! - **Nodes**: Individual processing steps or states in the graph
//! - **Edges**: Connections between nodes that define execution flow
//!
//! ## Example
//!
//! ```no_run
//! use langstar_sdk::{LangchainClient, AuthConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let auth = AuthConfig::from_env()?;
//!     let client = LangchainClient::new(auth)?;
//!
//!     // Get graph topology for an assistant
//!     // Note: get_graph() method will be implemented in a future PR
//!     let graph = client.assistants().get_graph("assistant-id").await?;
//!
//!     println!("Graph has {} nodes and {} edges",
//!              graph.nodes.len(), graph.edges.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## API Reference
//!
//! See the [Agent Server API documentation](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref/)
//! for endpoint details.

use serde::{Deserialize, Serialize};

/// Graph structure returned by `/assistants/{id}/graph`
///
/// Represents the complete topology of a LangGraph graph, including
/// all nodes and the edges connecting them.
///
/// ## Example JSON
///
/// ```json
/// {
///   "nodes": [
///     { "id": "__start__", "type": "runnable", "data": { "name": "__start__" } },
///     { "id": "echo", "type": "runnable", "data": { "name": "echo" } },
///     { "id": "__end__" }
///   ],
///   "edges": [
///     { "source": "__start__", "target": "echo" },
///     { "source": "echo", "target": "__end__" }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Graph {
    /// List of nodes in the graph
    pub nodes: Vec<GraphNode>,
    /// List of edges connecting nodes
    pub edges: Vec<GraphEdge>,
}

/// A node in the graph
///
/// Each node represents a processing step or state in the graph workflow.
/// Special nodes include:
/// - `__start__`: Entry point of the graph
/// - `__end__`: Exit point of the graph
///
/// ## Fields
///
/// - `id`: Unique identifier for the node
/// - `node_type`: Optional type classification (e.g., "runnable")
/// - `data`: Optional metadata about the node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    /// Unique identifier for this node
    pub id: String,

    /// Type of the node (e.g., "runnable")
    ///
    /// This field uses serde rename to match the JSON field name "type",
    /// which is a reserved keyword in Rust.
    #[serde(rename = "type")]
    pub node_type: Option<String>,

    /// Optional metadata about the node
    pub data: Option<GraphNodeData>,
}

/// Metadata associated with a graph node
///
/// Currently contains the display name of the node.
/// May be extended with additional fields in future API versions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNodeData {
    /// Display name for this node
    pub name: Option<String>,
}

/// An edge connecting two nodes
///
/// Edges define the flow of execution between nodes in the graph.
/// Conditional edges allow branching based on runtime conditions.
///
/// ## Example
///
/// ```json
/// { "source": "node_a", "target": "node_b", "conditional": false }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEdge {
    /// ID of the source node
    pub source: String,

    /// ID of the target node
    pub target: String,

    /// Whether this edge is conditional (branching)
    ///
    /// Defaults to `false` if not present in JSON.
    #[serde(default)]
    pub conditional: bool,
}

/// Summary of a graph derived from assistants
///
/// This is a derived structure (not from API) that aggregates
/// information about assistants using a particular graph.
///
/// ## Use Case
///
/// When listing multiple assistants and grouping them by graph,
/// this summary provides aggregate statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphSummary {
    /// The graph ID these assistants use
    pub graph_id: String,

    /// Names of assistants using this graph
    pub assistant_names: Vec<String>,

    /// Total number of assistants using this graph
    pub assistant_count: usize,

    /// Names of nodes in the graph (if available)
    pub node_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_deserialize_simple() {
        let json = r#"{
            "nodes": [
                { "id": "__start__", "type": "runnable", "data": { "name": "__start__" } },
                { "id": "echo", "type": "runnable", "data": { "name": "echo" } },
                { "id": "__end__" }
            ],
            "edges": [
                { "source": "__start__", "target": "echo" },
                { "source": "echo", "target": "__end__" }
            ]
        }"#;

        let graph: Graph = serde_json::from_str(json).expect("Failed to deserialize graph");

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);

        // Check __start__ node
        assert_eq!(graph.nodes[0].id, "__start__");
        assert_eq!(graph.nodes[0].node_type, Some("runnable".to_string()));
        assert_eq!(
            graph.nodes[0].data.as_ref().unwrap().name,
            Some("__start__".to_string())
        );

        // Check echo node
        assert_eq!(graph.nodes[1].id, "echo");
        assert_eq!(graph.nodes[1].node_type, Some("runnable".to_string()));
        assert_eq!(
            graph.nodes[1].data.as_ref().unwrap().name,
            Some("echo".to_string())
        );

        // Check __end__ node (minimal fields)
        assert_eq!(graph.nodes[2].id, "__end__");
        assert_eq!(graph.nodes[2].node_type, None);
        assert_eq!(graph.nodes[2].data, None);

        // Check edges
        assert_eq!(graph.edges[0].source, "__start__");
        assert_eq!(graph.edges[0].target, "echo");
        assert!(!graph.edges[0].conditional);

        assert_eq!(graph.edges[1].source, "echo");
        assert_eq!(graph.edges[1].target, "__end__");
        assert!(!graph.edges[1].conditional);
    }

    #[test]
    fn test_graph_deserialize_conditional_edges() {
        let json = r#"{
            "nodes": [
                { "id": "__start__", "type": "runnable", "data": { "name": "__start__" } },
                { "id": "Responder", "type": "runnable", "data": { "name": "Responder" } },
                { "id": "Feedback", "type": "runnable", "data": { "name": "Feedback" } },
                { "id": "__end__" }
            ],
            "edges": [
                { "source": "Responder", "target": "Feedback", "conditional": true },
                { "source": "Responder", "target": "__end__", "conditional": true },
                { "source": "__start__", "target": "Feedback", "conditional": true },
                { "source": "__start__", "target": "Responder", "conditional": true }
            ]
        }"#;

        let graph: Graph = serde_json::from_str(json).expect("Failed to deserialize graph");

        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 4);

        // All edges should be conditional
        for edge in &graph.edges {
            assert!(
                edge.conditional,
                "Edge from {} to {} should be conditional",
                edge.source, edge.target
            );
        }
    }

    #[test]
    fn test_graph_serialize() {
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
                    id: "__end__".to_string(),
                    node_type: None,
                    data: None,
                },
            ],
            edges: vec![GraphEdge {
                source: "__start__".to_string(),
                target: "__end__".to_string(),
                conditional: false,
            }],
        };

        let json = serde_json::to_string(&graph).expect("Failed to serialize graph");
        let deserialized: Graph =
            serde_json::from_str(&json).expect("Failed to deserialize serialized graph");

        assert_eq!(graph, deserialized);
    }

    #[test]
    fn test_graph_edge_default_conditional() {
        // Test that conditional defaults to false when not present
        let json = r#"{ "source": "a", "target": "b" }"#;
        let edge: GraphEdge = serde_json::from_str(json).expect("Failed to deserialize edge");
        assert!(!edge.conditional);

        // Test explicit false
        let json = r#"{ "source": "a", "target": "b", "conditional": false }"#;
        let edge: GraphEdge = serde_json::from_str(json).expect("Failed to deserialize edge");
        assert!(!edge.conditional);

        // Test explicit true
        let json = r#"{ "source": "a", "target": "b", "conditional": true }"#;
        let edge: GraphEdge = serde_json::from_str(json).expect("Failed to deserialize edge");
        assert!(edge.conditional);
    }

    #[test]
    fn test_graph_summary_creation() {
        let summary = GraphSummary {
            graph_id: "test_graph".to_string(),
            assistant_names: vec!["assistant1".to_string(), "assistant2".to_string()],
            assistant_count: 2,
            node_names: vec![
                "__start__".to_string(),
                "echo".to_string(),
                "__end__".to_string(),
            ],
        };

        assert_eq!(summary.graph_id, "test_graph");
        assert_eq!(summary.assistant_count, 2);
        assert_eq!(summary.node_names.len(), 3);
    }
}
