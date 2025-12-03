//! Integration tests for workspace secrets SDK.
//!
//! These tests verify the SDK client methods for managing workspace secrets
//! against the actual LangSmith API.
//!
//! **Prerequisites:**
//!
//! - `LANGSMITH_API_KEY` environment variable with `workspaces:manage` permission
//!
//! **Run with:**
//!
//! ```bash
//! cargo test --test secrets_integration_test -- --ignored --nocapture
//! ```
//!
//! **Security Note:**
//!
//! These tests use a dedicated test secret key prefix `LANGSTAR_TEST_` to avoid
//! conflicts with real secrets. All test secrets are cleaned up after each test.

use langstar_sdk::secrets::{SecretKey, SecretUpsert};
use langstar_sdk::{AuthConfig, LangchainClient};

/// Generate a unique test secret key with timestamp to avoid collisions
fn test_secret_key(suffix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("LANGSTAR_TEST_{}_{}", suffix, timestamp)
}

/// Create a configured client for testing
async fn create_test_client() -> LangchainClient {
    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set for integration tests");
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required for this test");
    LangchainClient::new(auth).expect("Failed to create LangchainClient")
}

/// Cleanup helper to delete test secrets
async fn cleanup_test_secret(client: &LangchainClient, key: &str) {
    // Ignore errors during cleanup - secret may not exist
    let _ = client.delete_workspace_secret(key).await;
}

// ============================================================================
// List Secrets Tests
// ============================================================================

/// Test listing workspace secrets returns valid response.
///
/// Verifies:
/// - API call succeeds
/// - Response is a valid Vec<SecretKey>
/// - Each key has a non-empty name
#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_list_workspace_secrets() {
    let client = create_test_client().await;

    println!("Listing workspace secrets...");
    let result = client.list_workspace_secrets().await;

    match result {
        Ok(keys) => {
            println!("Found {} secrets", keys.len());
            for key in &keys {
                // Security: Only print key names, never values (which don't exist anyway)
                println!("  - {}", key.key);
                assert!(!key.key.is_empty(), "Secret key should not be empty");
            }
        }
        Err(e) => {
            // 403 Forbidden is expected if API key lacks workspaces:manage permission
            panic!("Failed to list secrets: {:?}", e);
        }
    }
}

// ============================================================================
// Upsert Secrets Tests
// ============================================================================

/// Test creating a new secret via upsert.
///
/// Verifies:
/// - Secret can be created
/// - Secret appears in list after creation
/// - Cleanup works correctly
#[tokio::test]
#[ignore]
async fn test_upsert_create_secret() {
    let client = create_test_client().await;
    let test_key = test_secret_key("CREATE");

    // Cleanup in case of previous test failure
    cleanup_test_secret(&client, &test_key).await;

    println!("Creating secret '{}'...", test_key);

    // Create the secret
    let secrets = vec![SecretUpsert::set(&test_key, "test_value_12345")];
    let result = client.upsert_workspace_secrets(secrets).await;

    match result {
        Ok(()) => {
            println!("Secret created successfully");

            // Verify it appears in the list
            let keys = client
                .list_workspace_secrets()
                .await
                .expect("Failed to list secrets");

            let found = keys.iter().any(|k| k.key == test_key);
            assert!(found, "Created secret should appear in list");
            println!("Verified secret appears in list");
        }
        Err(e) => {
            panic!("Failed to create secret: {:?}", e);
        }
    }

    // Cleanup
    cleanup_test_secret(&client, &test_key).await;
    println!("Cleanup complete");
}

/// Test updating an existing secret via upsert.
///
/// Verifies:
/// - Secret can be created
/// - Same key can be updated with new value
/// - No error on update (API doesn't return values, so we can't verify content)
#[tokio::test]
#[ignore]
async fn test_upsert_update_secret() {
    let client = create_test_client().await;
    let test_key = test_secret_key("UPDATE");

    // Cleanup in case of previous test failure
    cleanup_test_secret(&client, &test_key).await;

    println!("Creating initial secret '{}'...", test_key);

    // Create initial secret
    let secrets = vec![SecretUpsert::set(&test_key, "initial_value")];
    client
        .upsert_workspace_secrets(secrets)
        .await
        .expect("Failed to create initial secret");

    println!("Updating secret with new value...");

    // Update with new value
    let secrets = vec![SecretUpsert::set(&test_key, "updated_value")];
    let result = client.upsert_workspace_secrets(secrets).await;

    match result {
        Ok(()) => {
            println!("Secret updated successfully");

            // Verify it still exists (only key, not value)
            let keys = client
                .list_workspace_secrets()
                .await
                .expect("Failed to list secrets");

            let found = keys.iter().any(|k| k.key == test_key);
            assert!(found, "Updated secret should still appear in list");
        }
        Err(e) => {
            cleanup_test_secret(&client, &test_key).await;
            panic!("Failed to update secret: {:?}", e);
        }
    }

    // Cleanup
    cleanup_test_secret(&client, &test_key).await;
    println!("Cleanup complete");
}

/// Test batch upsert of multiple secrets.
///
/// Verifies:
/// - Multiple secrets can be created in a single request
/// - All secrets appear in list after creation
#[tokio::test]
#[ignore]
async fn test_upsert_batch_secrets() {
    let client = create_test_client().await;
    let test_key1 = test_secret_key("BATCH1");
    let test_key2 = test_secret_key("BATCH2");
    let test_key3 = test_secret_key("BATCH3");

    // Cleanup in case of previous test failure
    cleanup_test_secret(&client, &test_key1).await;
    cleanup_test_secret(&client, &test_key2).await;
    cleanup_test_secret(&client, &test_key3).await;

    println!("Creating batch of secrets...");

    // Create multiple secrets in one request
    let secrets = vec![
        SecretUpsert::set(&test_key1, "value1"),
        SecretUpsert::set(&test_key2, "value2"),
        SecretUpsert::set(&test_key3, "value3"),
    ];
    let result = client.upsert_workspace_secrets(secrets).await;

    match result {
        Ok(()) => {
            println!("Batch secrets created successfully");

            // Verify all appear in list
            let keys = client
                .list_workspace_secrets()
                .await
                .expect("Failed to list secrets");

            let found1 = keys.iter().any(|k| k.key == test_key1);
            let found2 = keys.iter().any(|k| k.key == test_key2);
            let found3 = keys.iter().any(|k| k.key == test_key3);

            assert!(found1, "First batch secret should appear in list");
            assert!(found2, "Second batch secret should appear in list");
            assert!(found3, "Third batch secret should appear in list");
            println!("All batch secrets verified");
        }
        Err(e) => {
            panic!("Failed to create batch secrets: {:?}", e);
        }
    }

    // Cleanup
    cleanup_test_secret(&client, &test_key1).await;
    cleanup_test_secret(&client, &test_key2).await;
    cleanup_test_secret(&client, &test_key3).await;
    println!("Cleanup complete");
}

// ============================================================================
// Delete Secrets Tests
// ============================================================================

/// Test deleting a secret via the convenience method.
///
/// Verifies:
/// - Secret can be created
/// - Secret can be deleted via delete_workspace_secret
/// - Secret no longer appears in list after deletion
#[tokio::test]
#[ignore]
async fn test_delete_secret_convenience_method() {
    let client = create_test_client().await;
    let test_key = test_secret_key("DELETE");

    // Create a secret to delete
    println!("Creating secret '{}' to delete...", test_key);
    let secrets = vec![SecretUpsert::set(&test_key, "to_be_deleted")];
    client
        .upsert_workspace_secrets(secrets)
        .await
        .expect("Failed to create secret");

    // Verify it exists
    let keys = client
        .list_workspace_secrets()
        .await
        .expect("Failed to list secrets");
    assert!(
        keys.iter().any(|k| k.key == test_key),
        "Secret should exist before deletion"
    );

    println!("Deleting secret...");

    // Delete using convenience method
    let result = client.delete_workspace_secret(&test_key).await;

    match result {
        Ok(()) => {
            println!("Secret deleted successfully");

            // Verify it no longer exists
            let keys = client
                .list_workspace_secrets()
                .await
                .expect("Failed to list secrets");

            let found = keys.iter().any(|k| k.key == test_key);
            assert!(!found, "Deleted secret should not appear in list");
            println!("Verified secret no longer in list");
        }
        Err(e) => {
            cleanup_test_secret(&client, &test_key).await;
            panic!("Failed to delete secret: {:?}", e);
        }
    }
}

/// Test deleting a secret via upsert with null value.
///
/// Verifies:
/// - SecretUpsert::delete() generates correct request (value: null)
/// - Secret is removed via upsert mechanism
#[tokio::test]
#[ignore]
async fn test_delete_secret_via_upsert() {
    let client = create_test_client().await;
    let test_key = test_secret_key("DELETE_UPSERT");

    // Create a secret to delete
    println!("Creating secret '{}' to delete via upsert...", test_key);
    let secrets = vec![SecretUpsert::set(&test_key, "to_be_deleted")];
    client
        .upsert_workspace_secrets(secrets)
        .await
        .expect("Failed to create secret");

    // Verify it exists
    let keys = client
        .list_workspace_secrets()
        .await
        .expect("Failed to list secrets");
    assert!(
        keys.iter().any(|k| k.key == test_key),
        "Secret should exist before deletion"
    );

    println!("Deleting secret via upsert with null value...");

    // Delete using upsert with null value
    let secrets = vec![SecretUpsert::delete(&test_key)];
    let result = client.upsert_workspace_secrets(secrets).await;

    match result {
        Ok(()) => {
            println!("Secret deleted via upsert successfully");

            // Verify it no longer exists
            let keys = client
                .list_workspace_secrets()
                .await
                .expect("Failed to list secrets");

            let found = keys.iter().any(|k| k.key == test_key);
            assert!(!found, "Deleted secret should not appear in list");
            println!("Verified secret no longer in list");
        }
        Err(e) => {
            cleanup_test_secret(&client, &test_key).await;
            panic!("Failed to delete secret via upsert: {:?}", e);
        }
    }
}

// ============================================================================
// End-to-End Workflow Tests
// ============================================================================

/// Test complete CRUD workflow: Create -> Read (list) -> Update -> Delete.
///
/// Verifies full lifecycle of a secret through all operations.
#[tokio::test]
#[ignore]
async fn test_secret_crud_workflow() {
    let client = create_test_client().await;
    let test_key = test_secret_key("CRUD");

    // Cleanup any leftover
    cleanup_test_secret(&client, &test_key).await;

    println!("=== Testing full CRUD workflow for secrets ===\n");

    // 1. CREATE
    println!("1. CREATE: Creating secret '{}'...", test_key);
    let secrets = vec![SecretUpsert::set(&test_key, "initial_crud_value")];
    client
        .upsert_workspace_secrets(secrets)
        .await
        .expect("CREATE failed");
    println!("   CREATE: Success\n");

    // 2. READ (via list - values never returned)
    println!("2. READ: Verifying secret exists in list...");
    let keys = client.list_workspace_secrets().await.expect("READ failed");
    let found = keys.iter().any(|k| k.key == test_key);
    assert!(found, "Secret should exist after CREATE");
    println!("   READ: Found secret in list\n");

    // 3. UPDATE
    println!("3. UPDATE: Updating secret value...");
    let secrets = vec![SecretUpsert::set(&test_key, "updated_crud_value")];
    client
        .upsert_workspace_secrets(secrets)
        .await
        .expect("UPDATE failed");
    // Verify still exists
    let keys = client
        .list_workspace_secrets()
        .await
        .expect("List after UPDATE failed");
    assert!(
        keys.iter().any(|k| k.key == test_key),
        "Secret should still exist after UPDATE"
    );
    println!("   UPDATE: Success, secret still in list\n");

    // 4. DELETE
    println!("4. DELETE: Deleting secret...");
    client
        .delete_workspace_secret(&test_key)
        .await
        .expect("DELETE failed");
    // Verify gone
    let keys = client
        .list_workspace_secrets()
        .await
        .expect("List after DELETE failed");
    let still_found = keys.iter().any(|k| k.key == test_key);
    assert!(!still_found, "Secret should not exist after DELETE");
    println!("   DELETE: Success, secret removed from list\n");

    println!("=== CRUD workflow complete ===");
}

// ============================================================================
// Type Validation Tests
// ============================================================================

/// Test that SecretKey deserialization works correctly with API response format.
///
/// This is a unit test (no API call) that verifies type compatibility.
#[test]
fn test_secret_key_api_format_compatibility() {
    // API returns: [{"key": "SECRET_NAME"}, ...]
    let api_response = r#"[{"key": "ANTHROPIC_API_KEY"}, {"key": "OPENAI_API_KEY"}]"#;

    let keys: Vec<SecretKey> =
        serde_json::from_str(api_response).expect("Failed to deserialize API response format");

    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].key, "ANTHROPIC_API_KEY");
    assert_eq!(keys[1].key, "OPENAI_API_KEY");
}

/// Test that SecretUpsert serializes correctly for API request format.
///
/// This is a unit test (no API call) that verifies type compatibility.
#[test]
fn test_secret_upsert_api_format_compatibility() {
    // Create operation
    let create = SecretUpsert::set("NEW_KEY", "secret_value");
    let json = serde_json::to_string(&create).unwrap();
    assert!(json.contains(r#""key":"NEW_KEY""#));
    assert!(json.contains(r#""value":"secret_value""#));

    // Delete operation (must serialize value as null, not omit it)
    let delete = SecretUpsert::delete("OLD_KEY");
    let json = serde_json::to_string(&delete).unwrap();
    assert!(json.contains(r#""key":"OLD_KEY""#));
    assert!(json.contains(r#""value":null"#));
}
