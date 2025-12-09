use langstar_sdk::test_utils::TestDeploymentConfig;
use langstar_sdk::{AuthConfig, DeploymentFilters, DeploymentType, LangchainClient};

/// Integration test for listing LangGraph deployments via Control Plane API
///
/// This test queries the Control Plane API to list deployments in your workspace.
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY
/// 2. Valid LANGSMITH_WORKSPACE_ID (tenant ID)
/// 3. At least one LangGraph deployment in your workspace
///
/// Run with: cargo test --test graph_integration_test -- --ignored --nocapture
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_list_deployments() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and LANGSMITH_WORKSPACE_ID must be set for integration tests");

    // Verify we have required credentials
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required for this test");

    if auth.workspace_id.is_none() {
        panic!("LANGSMITH_WORKSPACE_ID is required for Control Plane API access");
    }

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    println!("Testing Control Plane API: List Deployments");
    println!("================================================\n");

    // Test 1: List all deployments (default limit)
    println!("Test 1: List all deployments (default limit: 20)");
    let deployments_list = client
        .deployments()
        .list(None, None, None)
        .await
        .expect("Failed to list deployments");

    println!("✓ Successfully listed deployments");
    println!("  Total: {}", deployments_list.resources.len());
    println!("  Offset: {}", deployments_list.offset);

    if !deployments_list.resources.is_empty() {
        println!("\n  Deployments:");
        for deployment in &deployments_list.resources {
            println!("    - {} ({})", deployment.name, deployment.id);
            println!("      Status: {:?}", deployment.status);
            println!("      Source: {:?}", deployment.source);
            println!("      Created: {}", deployment.created_at);
        }
    } else {
        println!("  No deployments found in workspace.");
        println!("  Create a deployment at: https://smith.langchain.com/");
    }

    // Test 2: List with limit
    println!("\n\nTest 2: List with limit=5");
    let deployments_list_limited = client
        .deployments()
        .list(Some(5), None, None)
        .await
        .expect("Failed to list deployments with limit");

    println!("✓ Successfully listed deployments with limit");
    println!("  Total: {}", deployments_list_limited.resources.len());
    assert!(
        deployments_list_limited.resources.len() <= 5,
        "Should respect limit parameter"
    );

    // Test 3: Filter by deployment type (if we have deployments)
    if !deployments_list.resources.is_empty() {
        println!("\n\nTest 3: Filter by deployment type (dev)");
        let filters = DeploymentFilters {
            deployment_type: Some(DeploymentType::Dev),
            ..Default::default()
        };

        let filtered_deployments = client
            .deployments()
            .list(None, None, Some(filters))
            .await
            .expect("Failed to list deployments with type filter");

        println!("✓ Successfully filtered by deployment type");
        println!(
            "  Dev deployments found: {}",
            filtered_deployments.resources.len()
        );
    }

    // Test 4: Get single deployment (if we have any)
    if let Some(first_deployment) = deployments_list.resources.first() {
        println!("\n\nTest 4: Get single deployment details");
        let deployment = client
            .deployments()
            .get(&first_deployment.id)
            .await
            .expect("Failed to get deployment details");

        println!("✓ Successfully fetched deployment details");
        println!("  Name: {}", deployment.name);
        println!("  ID: {}", deployment.id);
        println!("  Status: {:?}", deployment.status);
        println!("  Source: {:?}", deployment.source);
        println!("  Created: {}", deployment.created_at);
        println!("  Updated: {}", deployment.updated_at);

        assert_eq!(deployment.id, first_deployment.id);
        assert_eq!(deployment.name, first_deployment.name);
    }

    println!("\n\n================================================");
    println!("All integration tests passed! ✓");
}

/// Test deployment filtering by name
///
/// Run with: cargo test --test graph_integration_test test_filter_deployments_by_name -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_filter_deployments_by_name() {
    let auth = AuthConfig::from_env().expect("Auth required");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    println!("Test: Filter deployments by name");
    println!("==================================\n");

    // First, list all deployments to see what we have
    let all_deployments = client
        .deployments()
        .list(None, None, None)
        .await
        .expect("Failed to list all deployments");

    if all_deployments.resources.is_empty() {
        println!("No deployments found. Skipping filter test.");
        return;
    }

    // Use part of the first deployment's name as a filter
    let first_name = &all_deployments.resources[0].name;
    let search_term = if first_name.len() > 3 {
        &first_name[..3]
    } else {
        first_name
    };

    println!(
        "Searching for deployments with name containing: '{}'",
        search_term
    );

    let filters = DeploymentFilters {
        name_contains: Some(search_term.to_string()),
        ..Default::default()
    };

    let filtered = client
        .deployments()
        .list(None, None, Some(filters))
        .await
        .expect("Failed to filter by name");

    println!("✓ Found {} matching deployments", filtered.resources.len());
    for deployment in &filtered.resources {
        println!("  - {}", deployment.name);
        assert!(
            deployment.name.contains(search_term),
            "Deployment name should contain search term"
        );
    }

    println!("\n✓ Name filtering test passed!");
}

// ═══════════════════════════════════════════════════════════════════════════
// Graph API Tests (via Agent Server API)
// ═══════════════════════════════════════════════════════════════════════════

/// Helper to get or create a test deployment and return the client with deployment URL
async fn get_test_deployment_client() -> Option<(LangchainClient, String)> {
    let auth = AuthConfig::from_env().ok()?;
    let client = LangchainClient::new(auth.clone()).ok()?;

    // Get existing pr-integration-test deployment or create one
    let config = TestDeploymentConfig::default();
    let (deployment_id, _revision_id, _deployment_name) =
        langstar_sdk::test_utils::get_or_create_deployment(&client, &config)
            .await
            .ok()?;

    // Fetch full deployment details to get custom_url
    let deployment = client.deployments().get(&deployment_id).await.ok()?;

    // Extract deployment URL from source_config
    let deployment_url = deployment.custom_url()?;

    // Create client with deployment URL
    let deployment_client = LangchainClient::new(auth)
        .ok()?
        .with_langgraph_url(deployment_url.clone());

    Some((deployment_client, deployment_url))
}

/// Test listing graphs from a real deployment via Agent Server API
///
/// This test:
/// 1. Gets or creates a test deployment
/// 2. Creates a client pointing to that deployment's Agent Server
/// 3. Calls graphs().list() to discover graphs
///
/// Run with: cargo test --test graph_integration_test test_graph_list_real_deployment -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_graph_list_real_deployment() {
    let Some((client, deployment_url)) = get_test_deployment_client().await else {
        println!("⚠ Skipping test: Could not create test deployment");
        println!("  Ensure LANGSMITH_API_KEY and LANGSMITH_WORKSPACE_ID are set");
        return;
    };

    println!("Test: List Graphs via Agent Server API");
    println!("========================================\n");
    println!("Deployment URL: {}", deployment_url);

    // List graphs without structure (faster)
    println!("\nTest 1: List graphs (without structure)");
    let graphs = client
        .graphs()
        .list(false)
        .await
        .expect("Failed to list graphs");

    println!("✓ Successfully listed graphs");
    println!("  Found {} unique graph(s)", graphs.len());

    for graph in &graphs {
        println!(
            "  - {} ({} assistants): {}",
            graph.graph_id,
            graph.assistant_count,
            graph.assistant_names.join(", ")
        );
    }

    // Verify expected behavior
    // Test deployment should have at least one graph
    if graphs.is_empty() {
        println!("⚠ No graphs found in deployment");
        println!("  This is unexpected for the test deployment");
    }

    // List graphs with structure (includes node names)
    println!("\nTest 2: List graphs (with structure)");
    let graphs_with_structure = client
        .graphs()
        .list(true)
        .await
        .expect("Failed to list graphs with structure");

    println!("✓ Successfully listed graphs with structure");
    for graph in &graphs_with_structure {
        println!(
            "  - {}: nodes = [{}]",
            graph.graph_id,
            graph.node_names.join(", ")
        );
    }

    println!("\n✓ Graph list test passed!");
}

/// Test getting a specific graph structure from a real deployment
///
/// Run with: cargo test --test graph_integration_test test_graph_get_real_deployment -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_graph_get_real_deployment() {
    let Some((client, deployment_url)) = get_test_deployment_client().await else {
        println!("⚠ Skipping test: Could not create test deployment");
        return;
    };

    println!("Test: Get Graph Structure via Agent Server API");
    println!("================================================\n");
    println!("Deployment URL: {}", deployment_url);

    // First, list graphs to get a valid graph_id
    let graphs = client
        .graphs()
        .list(false)
        .await
        .expect("Failed to list graphs");

    if graphs.is_empty() {
        println!("⚠ No graphs found in deployment - skipping get test");
        return;
    }

    let graph_id = &graphs[0].graph_id;
    println!("Using graph_id: {}", graph_id);

    // Get graph without xray
    println!("\nTest 1: Get graph (xray=false)");
    let graph = client
        .graphs()
        .get(graph_id, false)
        .await
        .expect("Failed to get graph");

    println!("✓ Successfully retrieved graph structure");
    println!("  Nodes: {}", graph.nodes.len());
    println!("  Edges: {}", graph.edges.len());

    // List non-control nodes
    let user_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.id != "__start__" && n.id != "__end__")
        .map(|n| n.id.as_str())
        .collect();
    println!("  User nodes: [{}]", user_nodes.join(", "));

    // Verify structure
    assert!(
        !graph.nodes.is_empty(),
        "Graph should have at least one node"
    );
    assert!(
        !graph.edges.is_empty(),
        "Graph should have at least one edge"
    );

    // Verify __start__ node exists (required for all LangGraph graphs)
    let has_start = graph.nodes.iter().any(|n| n.id == "__start__");
    assert!(has_start, "Graph should have __start__ node");

    // Get graph with xray (includes subgraph details)
    println!("\nTest 2: Get graph (xray=true)");
    let graph_xray = client
        .graphs()
        .get(graph_id, true)
        .await
        .expect("Failed to get graph with xray");

    println!("✓ Successfully retrieved graph with xray");
    println!("  Nodes: {}", graph_xray.nodes.len());
    println!("  Edges: {}", graph_xray.edges.len());

    println!("\n✓ Graph get test passed!");
}

/// Test CRUD lifecycle: Create deployment → List graphs → Get graph → Verify
///
/// This test follows the CRUD lifecycle pattern to verify:
/// 1. SDK can list graphs from a deployment
/// 2. SDK can get individual graph structures
/// 3. Data returned matches expected format
///
/// Run with: cargo test --test graph_integration_test test_graph_crud_lifecycle -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_graph_crud_lifecycle() {
    let Some((client, deployment_url)) = get_test_deployment_client().await else {
        println!("⚠ Skipping test: Could not create test deployment");
        return;
    };

    println!("Test: Graph CRUD Lifecycle");
    println!("===========================\n");
    println!("Deployment URL: {}", deployment_url);

    // ═══════════════════════════════════════════════════════════════════
    // Step 1: LIST - Get all graphs in deployment
    // ═══════════════════════════════════════════════════════════════════
    println!("\n[LIST] Listing graphs in deployment...");

    let graphs = client
        .graphs()
        .list(true) // Include structure
        .await
        .expect("Failed to list graphs");

    println!("✓ Found {} graph(s)", graphs.len());

    if graphs.is_empty() {
        println!("⚠ No graphs found - test deployment may not be properly configured");
        return;
    }

    // ═══════════════════════════════════════════════════════════════════
    // Step 2: VERIFY - Check list response structure
    // ═══════════════════════════════════════════════════════════════════
    println!("\n[VERIFY] Checking list response structure...");

    let first_graph = &graphs[0];
    assert!(
        !first_graph.graph_id.is_empty(),
        "graph_id should not be empty"
    );
    assert!(
        first_graph.assistant_count > 0,
        "Should have at least one assistant"
    );
    assert!(
        !first_graph.assistant_names.is_empty(),
        "assistant_names should not be empty"
    );

    println!("✓ List response has correct structure");
    println!("  Graph ID: {}", first_graph.graph_id);
    println!("  Assistants: {}", first_graph.assistant_names.join(", "));
    println!("  Nodes: {}", first_graph.node_names.join(", "));

    // ═══════════════════════════════════════════════════════════════════
    // Step 3: GET - Fetch individual graph structure
    // ═══════════════════════════════════════════════════════════════════
    println!(
        "\n[GET] Fetching graph structure for '{}'...",
        first_graph.graph_id
    );

    let graph = client
        .graphs()
        .get(&first_graph.graph_id, true)
        .await
        .expect("Failed to get graph");

    println!("✓ Retrieved graph structure");

    // ═══════════════════════════════════════════════════════════════════
    // Step 4: VERIFY - Check graph structure
    // ═══════════════════════════════════════════════════════════════════
    println!("\n[VERIFY] Checking graph structure...");

    assert!(!graph.nodes.is_empty(), "Graph should have nodes");
    assert!(!graph.edges.is_empty(), "Graph should have edges");

    // Verify __start__ and __end__ exist
    let node_ids: Vec<_> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(
        node_ids.contains(&"__start__"),
        "Graph should have __start__"
    );
    assert!(node_ids.contains(&"__end__"), "Graph should have __end__");

    println!("✓ Graph structure is valid");
    println!("  Total nodes: {}", graph.nodes.len());
    println!("  Total edges: {}", graph.edges.len());

    // Verify edge references valid nodes
    for edge in &graph.edges {
        assert!(
            node_ids.contains(&edge.source.as_str()),
            "Edge source '{}' should reference a valid node",
            edge.source
        );
        assert!(
            node_ids.contains(&edge.target.as_str()),
            "Edge target '{}' should reference a valid node",
            edge.target
        );
    }

    println!("✓ All edges reference valid nodes");

    // ═══════════════════════════════════════════════════════════════════
    // Step 5: Verify node names from list match get
    // ═══════════════════════════════════════════════════════════════════
    println!("\n[VERIFY] Cross-checking list nodes vs get nodes...");

    let get_node_names: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.id != "__start__" && n.id != "__end__")
        .map(|n| n.id.clone())
        .collect();

    // Node names from list should be subset of get results
    for name in &first_graph.node_names {
        assert!(
            get_node_names.contains(name),
            "Node '{}' from list should appear in get results",
            name
        );
    }

    println!("✓ Node names consistent between list and get");

    println!("\n===========================");
    println!("✓ CRUD lifecycle test passed!");
    println!("===========================\n");
}
