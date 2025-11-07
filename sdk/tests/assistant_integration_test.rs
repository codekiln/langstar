use langstar_sdk::{AuthConfig, CreateAssistantRequest, LangchainClient, UpdateAssistantRequest};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper function to generate unique test names using timestamp
fn generate_test_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    format!("{}-{}", prefix, timestamp)
}

/// Helper function to discover the test deployment and get its custom URL
/// Returns (graph_name, custom_url)
async fn discover_test_deployment(client: &LangchainClient) -> (String, String) {
    let test_deployment_name = std::env::var("TEST_GRAPH_ID")
        .expect("TEST_GRAPH_ID must be set. Deploy the test graph fixture first.");

    println!("Discovering test deployment...");
    println!("  Looking for deployment name: {}", test_deployment_name);

    // List all deployments
    let deployments = client
        .deployments()
        .list(None, None, None)
        .await
        .expect("Failed to list deployments");

    // Find deployment with matching name
    let test_deployment = deployments
        .resources
        .iter()
        .find(|d| d.name == test_deployment_name || d.id == test_deployment_name)
        .expect(&format!(
            "Test deployment not found. Expected deployment name: {}",
            test_deployment_name
        ));

    println!("✓ Found test deployment:");
    println!("  Name: {}", test_deployment.name);
    println!("  ID: {}", test_deployment.id);
    println!("  Status: {:?}", test_deployment.status);

    let custom_url = test_deployment
        .custom_url()
        .expect("Test deployment has no custom_url");

    println!("  Custom URL: {}", custom_url);

    // The graph name is "test_graph" as defined in tests/fixtures/test-graph-deployment/langgraph.json
    let graph_name = "test_graph".to_string();

    (graph_name, custom_url)
}

/// Integration test for assistant lifecycle: create → get → update → delete
///
/// This test verifies the complete CRUD workflow for assistants.
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY
/// 2. Valid LANGCHAIN_WORKSPACE_ID
/// 3. TEST_GRAPH_ID environment variable set (from deployed test graph)
///
/// Run with: cargo test --test assistant_integration_test -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_assistant_lifecycle() {
    println!("==================================================");
    println!("Test: Assistant Lifecycle (Create → Get → Update → Delete)");
    println!("==================================================\n");

    // Load authentication from environment
    let auth =
        AuthConfig::from_env().expect("LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID must be set");

    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required");

    if auth.workspace_id.is_none() {
        panic!("LANGCHAIN_WORKSPACE_ID is required for assistant tests");
    }

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // Discover test deployment and get custom URL
    let (graph_name, custom_url) = discover_test_deployment(&client).await;

    // Override client with deployment's custom URL
    let client = client.with_langgraph_url(custom_url);

    println!("\n1. CREATE assistant");
    println!("--------------------------------------------------");

    let test_name = generate_test_name("test-assistant");
    println!("  Creating assistant: {}", test_name);

    let create_request = CreateAssistantRequest {
        graph_id: graph_name.clone(),
        name: test_name.clone(),
        config: Some(json!({"configurable": {"test": true}})),
        metadata: Some(json!({"purpose": "integration_test"})),
    };

    let created_assistant = client
        .assistants()
        .create(&create_request)
        .await
        .expect("Failed to create assistant");

    println!("✓ Assistant created successfully");
    println!("  Assistant ID: {}", created_assistant.assistant_id);
    println!("  Name: {}", created_assistant.name);
    println!("  Graph ID: {}", created_assistant.graph_id);

    assert_eq!(created_assistant.name, test_name);
    assert_eq!(created_assistant.graph_id, graph_name);
    assert!(created_assistant.config.is_some());
    assert!(created_assistant.metadata.is_some());

    let assistant_id = created_assistant.assistant_id.clone();

    println!("\n2. GET assistant by ID");
    println!("--------------------------------------------------");
    println!("  Fetching assistant: {}", assistant_id);

    let fetched_assistant = client
        .assistants()
        .get(&assistant_id)
        .await
        .expect("Failed to get assistant");

    println!("✓ Assistant fetched successfully");
    println!("  Name: {}", fetched_assistant.name);
    println!("  Graph ID: {}", fetched_assistant.graph_id);

    assert_eq!(fetched_assistant.assistant_id, assistant_id);
    assert_eq!(fetched_assistant.name, test_name);

    println!("\n3. UPDATE assistant");
    println!("--------------------------------------------------");

    let updated_name = format!("{}-updated", test_name);
    println!("  Updating assistant name to: {}", updated_name);

    let update_request = UpdateAssistantRequest {
        name: Some(updated_name.clone()),
        config: Some(json!({"configurable": {"test": true, "updated": true}})),
        metadata: Some(json!({"purpose": "integration_test", "updated": true})),
    };

    let updated_assistant = client
        .assistants()
        .update(&assistant_id, &update_request)
        .await
        .expect("Failed to update assistant");

    println!("✓ Assistant updated successfully");
    println!("  New name: {}", updated_assistant.name);

    assert_eq!(updated_assistant.assistant_id, assistant_id);
    assert_eq!(updated_assistant.name, updated_name);

    // Verify update persisted
    let refetched_assistant = client
        .assistants()
        .get(&assistant_id)
        .await
        .expect("Failed to refetch assistant");

    assert_eq!(refetched_assistant.name, updated_name);

    println!("\n4. DELETE assistant");
    println!("--------------------------------------------------");
    println!("  Deleting assistant: {}", assistant_id);

    client
        .assistants()
        .delete(&assistant_id)
        .await
        .expect("Failed to delete assistant");

    println!("✓ Assistant deleted successfully");

    // Verify deletion - get should fail
    println!("  Verifying deletion...");
    let get_result = client.assistants().get(&assistant_id).await;

    assert!(
        get_result.is_err(),
        "Get should fail after deletion, but succeeded"
    );
    println!("✓ Confirmed assistant no longer exists");

    println!("\n==================================================");
    println!("✓ All lifecycle tests passed!");
    println!("==================================================\n");
}

/// Integration test for assistant search functionality
///
/// This test verifies search can find assistants by name with:
/// - Exact match
/// - Partial match
/// - No results case
///
/// **Prerequisites:** Same as test_assistant_lifecycle
///
/// Run with: cargo test --test assistant_integration_test test_assistant_search -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_assistant_search() {
    println!("==================================================");
    println!("Test: Assistant Search");
    println!("==================================================\n");

    // Setup
    let auth =
        AuthConfig::from_env().expect("LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID must be set");
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required");
    if auth.workspace_id.is_none() {
        panic!("LANGCHAIN_WORKSPACE_ID is required");
    }

    let client = LangchainClient::new(auth).expect("Failed to create client");
    let (graph_name, custom_url) = discover_test_deployment(&client).await;
    let client = client.with_langgraph_url(custom_url);

    // Create test assistants with unique names
    let base_name = generate_test_name("search-test");
    let assistant1_name = format!("{}-alpha", base_name);
    let assistant2_name = format!("{}-beta", base_name);

    println!("Creating test assistants for search...");
    println!("  Assistant 1: {}", assistant1_name);
    println!("  Assistant 2: {}", assistant2_name);

    let create_request1 = CreateAssistantRequest {
        graph_id: graph_name.clone(),
        name: assistant1_name.clone(),
        config: None,
        metadata: Some(json!({"test": "search"})),
    };

    let created1 = client
        .assistants()
        .create(&create_request1)
        .await
        .expect("Failed to create assistant 1");

    let create_request2 = CreateAssistantRequest {
        graph_id: graph_name.clone(),
        name: assistant2_name.clone(),
        config: None,
        metadata: Some(json!({"test": "search"})),
    };

    let created2 = client
        .assistants()
        .create(&create_request2)
        .await
        .expect("Failed to create assistant 2");

    println!("✓ Test assistants created");

    // Test 1: Search with partial match (should find both)
    println!("\n1. Search with partial match: '{}'", base_name);
    println!("--------------------------------------------------");

    let search_results = client
        .assistants()
        .search(&base_name, None)
        .await
        .expect("Search failed");

    println!("✓ Search completed");
    println!("  Results found: {}", search_results.len());

    assert!(
        search_results.len() >= 2,
        "Expected at least 2 results, got {}",
        search_results.len()
    );

    let found_names: Vec<String> = search_results.iter().map(|a| a.name.clone()).collect();
    assert!(
        found_names.contains(&assistant1_name),
        "Should find assistant 1"
    );
    assert!(
        found_names.contains(&assistant2_name),
        "Should find assistant 2"
    );

    // Test 2: Search with exact match
    println!("\n2. Search with exact match: '{}'", assistant1_name);
    println!("--------------------------------------------------");

    let exact_results = client
        .assistants()
        .search(&assistant1_name, None)
        .await
        .expect("Exact search failed");

    println!("✓ Exact search completed");
    println!("  Results found: {}", exact_results.len());

    assert!(
        exact_results.len() >= 1,
        "Expected at least 1 result for exact match"
    );
    let found_exact = exact_results.iter().any(|a| a.name == assistant1_name);
    assert!(found_exact, "Should find exact match");

    // Test 3: Search with no results
    println!("\n3. Search with no results: 'nonexistent-assistant-xyz-123'");
    println!("--------------------------------------------------");

    let no_results = client
        .assistants()
        .search("nonexistent-assistant-xyz-123", None)
        .await
        .expect("No results search failed");

    println!("✓ No results search completed");
    println!("  Results found: {}", no_results.len());

    assert_eq!(no_results.len(), 0, "Should return empty results");

    // Cleanup
    println!("\n4. Cleanup test assistants");
    println!("--------------------------------------------------");

    client
        .assistants()
        .delete(&created1.assistant_id)
        .await
        .expect("Failed to delete assistant 1");
    println!("✓ Deleted assistant 1");

    client
        .assistants()
        .delete(&created2.assistant_id)
        .await
        .expect("Failed to delete assistant 2");
    println!("✓ Deleted assistant 2");

    println!("\n==================================================");
    println!("✓ All search tests passed!");
    println!("==================================================\n");
}

/// Integration test for listing assistants
///
/// **Prerequisites:** Same as test_assistant_lifecycle
///
/// Run with: cargo test --test assistant_integration_test test_list_assistants -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_list_assistants() {
    println!("==================================================");
    println!("Test: List Assistants");
    println!("==================================================\n");

    // Setup
    let auth =
        AuthConfig::from_env().expect("LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID must be set");
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required");
    if auth.workspace_id.is_none() {
        panic!("LANGCHAIN_WORKSPACE_ID is required");
    }

    let client = LangchainClient::new(auth).expect("Failed to create client");
    let (graph_name, custom_url) = discover_test_deployment(&client).await;
    let client = client.with_langgraph_url(custom_url);

    println!("1. List all assistants (empty or with existing)");
    println!("--------------------------------------------------");

    let initial_list = client
        .assistants()
        .list(None, None)
        .await
        .expect("Failed to list assistants");

    println!("✓ List completed");
    println!("  Total assistants: {}", initial_list.len());

    let initial_count = initial_list.len();

    println!("\n2. Create test assistant");
    println!("--------------------------------------------------");

    let test_name = generate_test_name("list-test");
    let create_request = CreateAssistantRequest {
        graph_id: graph_name.clone(),
        name: test_name.clone(),
        config: None,
        metadata: None,
    };

    let created = client
        .assistants()
        .create(&create_request)
        .await
        .expect("Failed to create assistant");

    println!("✓ Test assistant created: {}", created.assistant_id);

    println!("\n3. List assistants again (should include new one)");
    println!("--------------------------------------------------");

    let updated_list = client
        .assistants()
        .list(None, None)
        .await
        .expect("Failed to list assistants after creation");

    println!("✓ List completed");
    println!("  Total assistants: {}", updated_list.len());

    assert!(
        updated_list.len() > initial_count,
        "List should include newly created assistant"
    );

    let found = updated_list
        .iter()
        .any(|a| a.assistant_id == created.assistant_id);
    assert!(found, "Newly created assistant should be in list");

    println!("\n4. Test list with limit");
    println!("--------------------------------------------------");

    let limited_list = client
        .assistants()
        .list(Some(1), None)
        .await
        .expect("Failed to list with limit");

    println!("✓ Limited list completed");
    println!("  Results: {}", limited_list.len());

    assert!(limited_list.len() <= 1, "Should respect limit parameter");

    println!("\n5. Cleanup");
    println!("--------------------------------------------------");

    client
        .assistants()
        .delete(&created.assistant_id)
        .await
        .expect("Failed to delete test assistant");
    println!("✓ Deleted test assistant");

    println!("\n==================================================");
    println!("✓ All list tests passed!");
    println!("==================================================\n");
}

/// Integration test for error handling
///
/// **Prerequisites:** Same as test_assistant_lifecycle
///
/// Run with: cargo test --test assistant_integration_test test_error_handling -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_error_handling() {
    println!("==================================================");
    println!("Test: Error Handling");
    println!("==================================================\n");

    // Setup
    let auth =
        AuthConfig::from_env().expect("LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID must be set");
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required");
    if auth.workspace_id.is_none() {
        panic!("LANGCHAIN_WORKSPACE_ID is required");
    }

    let client = LangchainClient::new(auth).expect("Failed to create client");
    let (_deployment_id, custom_url) = discover_test_deployment(&client).await;
    let client = client.with_langgraph_url(custom_url);

    println!("1. Test 404 - Get nonexistent assistant");
    println!("--------------------------------------------------");

    let nonexistent_id = "nonexistent-assistant-id-12345";
    let result = client.assistants().get(nonexistent_id).await;

    println!("  Attempted to get: {}", nonexistent_id);
    assert!(result.is_err(), "Should fail with 404");
    println!("✓ Correctly returned error for nonexistent assistant");

    println!("\n2. Test 404 - Delete nonexistent assistant");
    println!("--------------------------------------------------");

    let delete_result = client.assistants().delete(nonexistent_id).await;

    println!("  Attempted to delete: {}", nonexistent_id);
    assert!(delete_result.is_err(), "Should fail with 404");
    println!("✓ Correctly returned error for delete nonexistent");

    println!("\n3. Test 404 - Update nonexistent assistant");
    println!("--------------------------------------------------");

    let update_request = UpdateAssistantRequest {
        name: Some("updated-name".to_string()),
        config: None,
        metadata: None,
    };

    let update_result = client
        .assistants()
        .update(nonexistent_id, &update_request)
        .await;

    println!("  Attempted to update: {}", nonexistent_id);
    assert!(update_result.is_err(), "Should fail with 404");
    println!("✓ Correctly returned error for update nonexistent");

    println!("\n==================================================");
    println!("✓ All error handling tests passed!");
    println!("==================================================\n");
}
