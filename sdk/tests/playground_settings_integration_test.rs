/// Integration tests for playground settings with real LangSmith API
///
/// These tests require:
/// - LANGSMITH_API_KEY environment variable
/// - LANGSMITH_ORGANIZATION_ID environment variable (or auto-discovery)
///
/// Run with: cargo test --test playground_settings_integration_test
use langstar_sdk::playground_settings::{
    ListPlaygroundSettingsParams, PlaygroundSavedOptions, PlaygroundSettingsCreateRequest,
    PlaygroundSettingsUpdateRequest,
};
use langstar_sdk::{AuthConfig, LangchainClient};
use serde_json::json;

/// Helper to create a test client with organization ID
async fn create_integration_test_client() -> LangchainClient {
    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required");

    let mut client = LangchainClient::new(auth).expect("Failed to create client");

    // Try to fetch organization ID
    match client.get_current_organization().await {
        Ok(org) => {
            if let Some(org_id) = org.id {
                println!(
                    "✓ Using organization: {}",
                    org.display_name.unwrap_or_default()
                );
                client = client.with_organization_id(org_id);
            }
        }
        Err(e) => {
            eprintln!("⚠ Warning: Could not fetch organization: {:?}", e);
        }
    }

    client
}

// ============================================================================
// List Integration Tests
// ============================================================================

#[tokio::test]
async fn test_list_playground_settings_integration() {
    let client = create_integration_test_client().await;

    let params = ListPlaygroundSettingsParams {
        limit: Some(10),
        offset: Some(0),
    };

    let result = client.list_playground_settings(params).await;

    assert!(
        result.is_ok(),
        "Failed to list playground settings: {:?}",
        result.err()
    );

    let configs = result.unwrap();
    println!("✓ Retrieved {} playground settings", configs.len());

    // Verify structure of returned configs
    for config in &configs {
        assert!(!config.id.to_string().is_empty(), "Config ID is empty");
        assert!(
            config.settings.is_object(),
            "Config settings should be an object"
        );
        println!(
            "  - {} ({})",
            config.name.as_deref().unwrap_or("-"),
            config.id
        );
    }
}

#[tokio::test]
async fn test_list_playground_settings_pagination() {
    let client = create_integration_test_client().await;

    // Use larger page size to reduce sensitivity to data changes during test
    // Fetch first page
    let first_page = client
        .list_playground_settings(ListPlaygroundSettingsParams {
            limit: Some(10),
            offset: Some(0),
        })
        .await
        .expect("Failed to fetch first page");

    // Fetch second page
    let second_page = client
        .list_playground_settings(ListPlaygroundSettingsParams {
            limit: Some(10),
            offset: Some(10),
        })
        .await
        .expect("Failed to fetch second page");

    println!(
        "✓ First page: {} configs, Second page: {} configs",
        first_page.len(),
        second_page.len()
    );

    // Test pagination consistency: pages should not overlap if both have results
    // Note: API pagination may not be perfectly stable if data changes during test,
    // so we only assert when we have full pages
    if first_page.len() == 10 && second_page.len() == 10 {
        let first_ids: Vec<_> = first_page.iter().map(|c| c.id).collect();
        let second_ids: Vec<_> = second_page.iter().map(|c| c.id).collect();

        let overlap_count = second_ids
            .iter()
            .filter(|id| first_ids.contains(id))
            .count();

        // Allow up to 1 overlapping item to account for API instability during concurrent access
        assert!(
            overlap_count <= 1,
            "Too many configs ({}) appear in both pages (may indicate pagination issue). Overlapping IDs: {:?}",
            overlap_count,
            second_ids
                .iter()
                .filter(|id| first_ids.contains(id))
                .collect::<Vec<_>>()
        );
    } else {
        println!(
            "  ℹ️  Skipping overlap check (pages not full - dataset may be small or changing)"
        );
    }
}

// ============================================================================
// Create/Update/Delete Integration Tests
// ============================================================================

#[tokio::test]
async fn test_create_update_delete_cycle() {
    let client = create_integration_test_client().await;

    // Step 1: Create a new playground setting
    println!("Creating playground setting...");
    let create_request = PlaygroundSettingsCreateRequest {
        name: Some("Test Config - Integration Test".to_string()),
        description: Some("This is a test configuration created by integration tests".to_string()),
        settings: json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
            "kwargs": {
                "model": "claude-3-5-sonnet-20241022",
                "temperature": 0.0,
                "max_tokens": 4096
            }
        }),
        options: PlaygroundSavedOptions {
            requests_per_second: Some(5),
        },
    };

    let created = client
        .create_playground_settings(create_request)
        .await
        .expect("Failed to create playground setting");

    println!(
        "✓ Created playground setting: {} (ID: {})",
        created.name.as_deref().unwrap_or("-"),
        created.id
    );
    assert_eq!(
        created.name,
        Some("Test Config - Integration Test".to_string())
    );
    assert_eq!(
        created.options.as_ref().unwrap().requests_per_second,
        Some(5)
    );

    // Step 2: Update the playground setting
    println!("Updating playground setting...");
    let update_request = PlaygroundSettingsUpdateRequest {
        name: Some("Test Config - Updated".to_string()),
        description: Some("Updated description".to_string()),
        settings: None,
        options: Some(PlaygroundSavedOptions {
            requests_per_second: Some(10),
        }),
    };

    let updated = client
        .update_playground_settings(created.id, update_request)
        .await
        .expect("Failed to update playground setting");

    println!(
        "✓ Updated playground setting: {}",
        updated.name.as_deref().unwrap_or("-")
    );
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, Some("Test Config - Updated".to_string()));
    assert_eq!(
        updated.options.as_ref().unwrap().requests_per_second,
        Some(10)
    );

    // Step 3: Delete the playground setting
    println!("Deleting playground setting...");
    client
        .delete_playground_settings(created.id)
        .await
        .expect("Failed to delete playground setting");

    println!("✓ Deleted playground setting: {}", created.id);

    // Step 4: Verify deletion - attempt to update should fail with 404
    println!("Verifying deletion...");
    let verify_request = PlaygroundSettingsUpdateRequest {
        name: Some("Should Not Exist".to_string()),
        ..Default::default()
    };

    let verify_result = client
        .update_playground_settings(created.id, verify_request)
        .await;

    assert!(verify_result.is_err(), "Update should fail after deletion");
    println!("✓ Verified deletion (update returned error as expected)");
}

// ============================================================================
// Error Handling Integration Tests
// ============================================================================

#[tokio::test]
async fn test_update_nonexistent_setting() {
    let client = create_integration_test_client().await;

    // Use a random UUID that doesn't exist
    let nonexistent_id = uuid::Uuid::new_v4();

    let update_request = PlaygroundSettingsUpdateRequest {
        name: Some("This should fail".to_string()),
        ..Default::default()
    };

    let result = client
        .update_playground_settings(nonexistent_id, update_request)
        .await;

    assert!(result.is_err(), "Update of nonexistent setting should fail");
    println!("✓ Update of nonexistent setting failed as expected");
}

#[tokio::test]
async fn test_delete_nonexistent_setting() {
    let client = create_integration_test_client().await;

    // Use a random UUID that doesn't exist
    let nonexistent_id = uuid::Uuid::new_v4();

    let result = client.delete_playground_settings(nonexistent_id).await;

    // API accepts idempotent deletes (returns 200 even if resource doesn't exist)
    assert!(result.is_ok(), "API should accept idempotent deletes");
    println!("✓ Idempotent delete succeeded (API returns 200 for nonexistent resources)");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_with_various_providers() {
    let client = create_integration_test_client().await;

    let test_cases = vec![
        (
            "Anthropic Config",
            json!({
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
                "kwargs": {
                    "model": "claude-3-5-sonnet-20241022",
                    "temperature": 0.7
                }
            }),
        ),
        (
            "OpenAI Config",
            json!({
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                "kwargs": {
                    "model": "gpt-4-turbo",
                    "temperature": 0.5
                }
            }),
        ),
        (
            "Bedrock Config",
            json!({
                "lc": 1,
                "type": "constructor",
                "id": ["langchain_aws", "chat_models", "bedrock_converse", "ChatBedrockConverse"],
                "kwargs": {
                    "model": "anthropic.claude-3-5-sonnet-20241022-v2:0",
                    "region_name": "us-east-1"
                }
            }),
        ),
    ];

    let mut created_ids = Vec::new();

    for (name, settings) in test_cases {
        println!("Testing {}...", name);

        let request = PlaygroundSettingsCreateRequest {
            name: Some(format!("{} - Integration Test", name)),
            description: Some(format!("Integration test for {}", name)),
            settings,
            options: PlaygroundSavedOptions::default(),
        };

        match client.create_playground_settings(request).await {
            Ok(created) => {
                println!("✓ Created {}: {}", name, created.id);
                created_ids.push(created.id);
            }
            Err(e) => {
                eprintln!("✗ Failed to create {}: {:?}", name, e);
            }
        }
    }

    // Ensure at least one configuration was created successfully
    assert!(
        !created_ids.is_empty(),
        "At least one provider configuration should be created successfully"
    );

    // Cleanup: delete all created configs
    for id in created_ids {
        if let Err(e) = client.delete_playground_settings(id).await {
            eprintln!("⚠ Warning: Failed to cleanup {}: {:?}", id, e);
        }
    }
}

// ============================================================================
// Partial Update Tests
// ============================================================================

#[tokio::test]
async fn test_partial_update_name_only() {
    let client = create_integration_test_client().await;

    // Create a config
    let create_request = PlaygroundSettingsCreateRequest {
        name: Some("Original Name".to_string()),
        description: Some("Original Description".to_string()),
        settings: json!({"key": "value"}),
        options: PlaygroundSavedOptions {
            requests_per_second: Some(5),
        },
    };

    let created = client
        .create_playground_settings(create_request)
        .await
        .expect("Failed to create");

    let original_description = created.description.clone();
    let original_rate_limit = created.options.as_ref().unwrap().requests_per_second;

    // Update only the name
    let update_request = PlaygroundSettingsUpdateRequest {
        name: Some("New Name Only".to_string()),
        description: None,
        settings: None,
        options: None,
    };

    let updated = client
        .update_playground_settings(created.id, update_request)
        .await
        .expect("Failed to update");

    // Name should be updated, other fields should remain
    assert_eq!(updated.name, Some("New Name Only".to_string()));
    assert_eq!(updated.description, original_description);
    assert_eq!(
        updated.options.as_ref().unwrap().requests_per_second,
        original_rate_limit
    );

    println!("✓ Partial update preserved unchanged fields");

    // Cleanup
    client
        .delete_playground_settings(created.id)
        .await
        .expect("Failed to cleanup");
}
