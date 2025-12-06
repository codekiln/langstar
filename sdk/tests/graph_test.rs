//! HTTP-mocked integration tests for graph operations.
//!
//! These tests verify the graph client methods using mockito
//! to mock the LangGraph API responses.
//!
//! ## What These Tests Cover
//!
//! - `list()` returns unique graph_ids with assistant associations
//! - `get()` returns correct graph structure with nodes and edges
//! - Error handling for missing deployments/graphs
//! - Pagination handling in list operations

use langstar_sdk::{AuthConfig, LangchainClient};
use mockito::{Matcher, Server};
use serde_json::json;

/// Helper function to create a test client with mock server for LangGraph API
fn create_test_client(langgraph_url: &str) -> LangchainClient {
    let auth = AuthConfig::new(Some("test_langsmith_key".to_string()), None, None);

    LangchainClient::with_base_urls(
        auth,
        "https://api.smith.langchain.com".to_string(),
        langgraph_url.to_string(),
        "https://api.host.langchain.com".to_string(),
    )
    .expect("Failed to create test client")
}

/// Helper to create an assistant JSON response
fn make_assistant_json(assistant_id: &str, name: &str, graph_id: &str) -> serde_json::Value {
    json!({
        "assistant_id": assistant_id,
        "name": name,
        "graph_id": graph_id,
        "config": {},
        "metadata": {},
        "created_at": "2024-01-01T12:00:00Z",
        "updated_at": "2024-01-01T12:00:00Z"
    })
}

/// Helper to create a graph JSON response
fn make_graph_json(
    nodes: Vec<(&str, Option<&str>)>,
    edges: Vec<(&str, &str, bool)>,
) -> serde_json::Value {
    let nodes_json: Vec<serde_json::Value> = nodes
        .into_iter()
        .map(|(id, node_type)| {
            let mut node = json!({ "id": id });
            if let Some(t) = node_type {
                node["type"] = json!(t);
                node["data"] = json!({ "name": id });
            }
            node
        })
        .collect();

    let edges_json: Vec<serde_json::Value> = edges
        .into_iter()
        .map(|(source, target, conditional)| {
            json!({
                "source": source,
                "target": target,
                "conditional": conditional
            })
        })
        .collect();

    json!({
        "nodes": nodes_json,
        "edges": edges_json
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Graph List Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_list_graphs_returns_unique_graph_ids() {
    let mut server = Server::new_async().await;

    // Mock assistants search endpoint - returns assistants with same graph_id
    let assistants = json!([
        make_assistant_json("assistant-1", "default", "agent"),
        make_assistant_json("assistant-2", "custom-v1", "agent"),
        make_assistant_json("assistant-3", "chatbot", "chatbot_graph")
    ]);

    let mock_search = server
        .mock("POST", "/assistants/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(assistants.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    // Call list without fetching structure (include_structure = false)
    let graphs = client
        .graphs()
        .list(false)
        .await
        .expect("list graphs failed");

    // Should return 2 unique graphs
    assert_eq!(graphs.len(), 2);

    // Find the "agent" graph - it should have 2 assistants
    let agent_graph = graphs.iter().find(|g| g.graph_id == "agent");
    assert!(agent_graph.is_some(), "Should find 'agent' graph");
    let agent = agent_graph.unwrap();
    assert_eq!(agent.assistant_count, 2);
    assert!(agent.assistant_names.contains(&"default".to_string()));
    assert!(agent.assistant_names.contains(&"custom-v1".to_string()));

    // Find the "chatbot_graph" - it should have 1 assistant
    let chatbot_graph = graphs.iter().find(|g| g.graph_id == "chatbot_graph");
    assert!(chatbot_graph.is_some(), "Should find 'chatbot_graph' graph");
    let chatbot = chatbot_graph.unwrap();
    assert_eq!(chatbot.assistant_count, 1);
    assert!(chatbot.assistant_names.contains(&"chatbot".to_string()));

    mock_search.assert_async().await;
}

#[tokio::test]
async fn test_list_graphs_with_structure_fetches_nodes() {
    let mut server = Server::new_async().await;

    // Mock assistants search endpoint
    let assistants = json!([make_assistant_json("assistant-1", "default", "agent")]);

    let mock_search = server
        .mock("POST", "/assistants/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(assistants.to_string())
        .create_async()
        .await;

    // Mock graph get endpoint for fetching structure
    let graph = make_graph_json(
        vec![
            ("__start__", Some("runnable")),
            ("Responder", Some("runnable")),
            ("Feedback", Some("runnable")),
            ("__end__", None),
        ],
        vec![
            ("__start__", "Responder", false),
            ("Responder", "Feedback", true),
            ("Feedback", "__end__", false),
        ],
    );

    let mock_graph = server
        .mock("GET", "/assistants/agent/graph")
        .match_query(Matcher::UrlEncoded("xray".into(), "true".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(graph.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    // Call list with structure fetching (include_structure = true)
    let graphs = client
        .graphs()
        .list(true)
        .await
        .expect("list graphs failed");

    assert_eq!(graphs.len(), 1);
    let summary = &graphs[0];
    assert_eq!(summary.graph_id, "agent");
    // Should have node names excluding __start__ and __end__
    assert_eq!(summary.node_names.len(), 2);
    assert!(summary.node_names.contains(&"Responder".to_string()));
    assert!(summary.node_names.contains(&"Feedback".to_string()));

    mock_search.assert_async().await;
    mock_graph.assert_async().await;
}

#[tokio::test]
async fn test_list_graphs_empty_deployment() {
    let mut server = Server::new_async().await;

    // Mock empty assistants list
    let mock_search = server
        .mock("POST", "/assistants/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let graphs = client
        .graphs()
        .list(false)
        .await
        .expect("list graphs failed");

    assert!(
        graphs.is_empty(),
        "Should return empty list for empty deployment"
    );

    mock_search.assert_async().await;
}

#[tokio::test]
async fn test_list_graphs_handles_pagination() {
    let mut server = Server::new_async().await;

    // First page - returns 100 assistants (full page)
    let page1: Vec<serde_json::Value> = (0..100)
        .map(|i| make_assistant_json(&format!("assistant-{}", i), &format!("name-{}", i), "agent"))
        .collect();

    // Second page - returns fewer than 100 (end of results)
    let page2: Vec<serde_json::Value> = (100..105)
        .map(|i| make_assistant_json(&format!("assistant-{}", i), &format!("name-{}", i), "agent"))
        .collect();

    let mock_page1 = server
        .mock("POST", "/assistants/search")
        .match_body(Matcher::PartialJson(json!({ "offset": 0, "limit": 100 })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&page1).unwrap())
        .create_async()
        .await;

    let mock_page2 = server
        .mock("POST", "/assistants/search")
        .match_body(Matcher::PartialJson(json!({ "offset": 100, "limit": 100 })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&page2).unwrap())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let graphs = client
        .graphs()
        .list(false)
        .await
        .expect("list graphs failed");

    // Should have aggregated all 105 assistants into 1 graph
    assert_eq!(graphs.len(), 1);
    assert_eq!(graphs[0].assistant_count, 105);

    mock_page1.assert_async().await;
    mock_page2.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Graph Get Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_graph_returns_structure() {
    let mut server = Server::new_async().await;

    let graph = make_graph_json(
        vec![
            ("__start__", Some("runnable")),
            ("echo", Some("runnable")),
            ("__end__", None),
        ],
        vec![("__start__", "echo", false), ("echo", "__end__", false)],
    );

    let mock = server
        .mock("GET", "/assistants/test_graph/graph")
        .match_query(Matcher::UrlEncoded("xray".into(), "true".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(graph.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client
        .graphs()
        .get("test_graph", true)
        .await
        .expect("get graph failed");

    assert_eq!(result.nodes.len(), 3);
    assert_eq!(result.edges.len(), 2);

    // Verify node details
    let start_node = result.nodes.iter().find(|n| n.id == "__start__");
    assert!(start_node.is_some());
    assert_eq!(start_node.unwrap().node_type, Some("runnable".to_string()));

    let echo_node = result.nodes.iter().find(|n| n.id == "echo");
    assert!(echo_node.is_some());

    let end_node = result.nodes.iter().find(|n| n.id == "__end__");
    assert!(end_node.is_some());
    assert_eq!(end_node.unwrap().node_type, None);

    // Verify edge details
    let edge1 = &result.edges[0];
    assert_eq!(edge1.source, "__start__");
    assert_eq!(edge1.target, "echo");
    assert!(!edge1.conditional);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_graph_with_conditional_edges() {
    let mut server = Server::new_async().await;

    let graph = make_graph_json(
        vec![
            ("__start__", Some("runnable")),
            ("router", Some("runnable")),
            ("path_a", Some("runnable")),
            ("path_b", Some("runnable")),
            ("__end__", None),
        ],
        vec![
            ("__start__", "router", false),
            ("router", "path_a", true),
            ("router", "path_b", true),
            ("path_a", "__end__", false),
            ("path_b", "__end__", false),
        ],
    );

    // Note: When xray=false, the SDK doesn't send a query param at all
    let mock = server
        .mock("GET", "/assistants/branching_graph/graph")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(graph.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client
        .graphs()
        .get("branching_graph", false)
        .await
        .expect("get graph failed");

    // Check conditional edges
    let conditional_edges: Vec<_> = result.edges.iter().filter(|e| e.conditional).collect();
    assert_eq!(conditional_edges.len(), 2);

    // Both should be from router
    for edge in conditional_edges {
        assert_eq!(edge.source, "router");
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_graph_without_xray() {
    let mut server = Server::new_async().await;

    let graph = make_graph_json(
        vec![("__start__", Some("runnable")), ("__end__", None)],
        vec![("__start__", "__end__", false)],
    );

    // Important: when xray=false, the query param should NOT have xray=true
    let mock = server
        .mock("GET", "/assistants/simple/graph")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(graph.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client
        .graphs()
        .get("simple", false)
        .await
        .expect("get graph failed");

    assert_eq!(result.nodes.len(), 2);

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Error Handling Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_graph_not_found() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/assistants/nonexistent/graph")
        .match_query(Matcher::UrlEncoded("xray".into(), "true".into()))
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"detail": "Graph not found"}"#)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client.graphs().get("nonexistent", true).await;

    assert!(result.is_err(), "Should return error for 404");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("404") || err.to_string().contains("not found"),
        "Error should indicate not found: {}",
        err
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_graphs_api_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/assistants/search")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"detail": "Internal server error"}"#)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client.graphs().list(false).await;

    assert!(result.is_err(), "Should return error for 500");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("500") || err.to_string().contains("Internal"),
        "Error should indicate server error: {}",
        err
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_graphs_structure_fetch_failure_graceful() {
    let mut server = Server::new_async().await;

    // Mock assistants search endpoint
    let assistants = json!([make_assistant_json("assistant-1", "default", "agent")]);

    let mock_search = server
        .mock("POST", "/assistants/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(assistants.to_string())
        .create_async()
        .await;

    // Mock graph get to fail - should be handled gracefully
    let mock_graph = server
        .mock("GET", "/assistants/agent/graph")
        .match_query(Matcher::UrlEncoded("xray".into(), "true".into()))
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"detail": "Failed to fetch graph"}"#)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    // Should succeed even if graph structure fetch fails
    let graphs = client
        .graphs()
        .list(true)
        .await
        .expect("list should succeed even with structure fetch failure");

    // Should have the graph but with empty node_names
    assert_eq!(graphs.len(), 1);
    assert_eq!(graphs[0].graph_id, "agent");
    assert!(
        graphs[0].node_names.is_empty(),
        "node_names should be empty on fetch failure"
    );

    mock_search.assert_async().await;
    mock_graph.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Subgraphs Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_subgraphs() {
    let mut server = Server::new_async().await;

    let subgraphs = json!([
        make_graph_json(
            vec![("sub_start", Some("runnable")), ("sub_end", None)],
            vec![("sub_start", "sub_end", false)]
        ),
        make_graph_json(
            vec![("other_start", Some("runnable")), ("other_end", None)],
            vec![("other_start", "other_end", false)]
        )
    ]);

    let mock = server
        .mock("GET", "/assistants/parent_graph/subgraphs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(subgraphs.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client
        .graphs()
        .subgraphs("parent_graph")
        .await
        .expect("get subgraphs failed");

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].nodes.len(), 2);
    assert_eq!(result[1].nodes.len(), 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_subgraphs_empty() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/assistants/flat_graph/subgraphs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client
        .graphs()
        .subgraphs("flat_graph")
        .await
        .expect("get subgraphs failed");

    assert!(
        result.is_empty(),
        "Should return empty list for graph without subgraphs"
    );

    mock.assert_async().await;
}
