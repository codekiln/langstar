use langstar_sdk::{AuthConfig, DeploymentFilters, DeploymentType, LangchainClient};

/// Integration test for listing LangGraph deployments via Control Plane API
///
/// This test queries the Control Plane API to list deployments in your workspace.
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY
/// 2. Valid LANGCHAIN_WORKSPACE_ID (tenant ID)
/// 3. At least one LangGraph deployment in your workspace
///
/// Run with: cargo test --test graph_integration_test -- --ignored --nocapture
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_list_deployments() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID must be set for integration tests");

    // Verify we have required credentials
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required for this test");

    if auth.workspace_id.is_none() {
        panic!("LANGCHAIN_WORKSPACE_ID is required for Control Plane API access");
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
