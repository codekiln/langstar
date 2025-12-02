/// Tests for playground settings SDK client methods with mocked HTTP responses
///
/// These tests verify the behavior of playground settings CRUD operations
/// without making real API calls by using mockito to simulate the LangSmith API.
use langstar_sdk::playground_settings::{
    ListPlaygroundSettingsParams, PlaygroundSavedOptions, PlaygroundSettingsCreateRequest,
    PlaygroundSettingsUpdateRequest,
};
use langstar_sdk::{AuthConfig, LangchainClient};
use mockito::{Server, ServerGuard};
use serde_json::json;
use uuid::Uuid;

/// Helper to create a test client pointing to the mock server
fn create_test_client(server: &ServerGuard) -> LangchainClient {
    let auth = AuthConfig::new(Some("test-api-key".to_string()), None, None, None);
    LangchainClient::with_base_urls(
        auth,
        server.url(), // Mock LangSmith API
        "http://mock-langgraph".to_string(),
        "http://mock-control-plane".to_string(),
    )
    .unwrap()
    .with_organization_id("test-org-id".to_string())
}

/// Helper to create a sample playground settings response JSON
fn create_sample_response_json(id: &str, name: &str) -> serde_json::Value {
    json!({
        "id": id,
        "settings": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
            "kwargs": {
                "model": "claude-3-5-sonnet-20241022",
                "temperature": 0.0
            }
        },
        "options": {
            "requests_per_second": 10
        },
        "name": name,
        "description": "Test configuration",
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-01-15T10:30:00Z"
    })
}

// ============================================================================
// List Playground Settings Tests
// ============================================================================

#[tokio::test]
async fn test_list_playground_settings_success() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/playground-settings")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "20".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "0".into()),
        ]))
        .match_header("x-api-key", "test-api-key")
        .match_header("x-organization-id", "test-org-id")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            json!([
                create_sample_response_json("550e8400-e29b-41d4-a716-446655440001", "Config 1"),
                create_sample_response_json("550e8400-e29b-41d4-a716-446655440002", "Config 2"),
            ])
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);
    let params = ListPlaygroundSettingsParams {
        limit: Some(20),
        offset: Some(0),
    };

    let result = client.list_playground_settings(params).await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let configs = result.unwrap();
    assert_eq!(configs.len(), 2);
    assert_eq!(configs[0].name, Some("Config 1".to_string()));
    assert_eq!(configs[1].name, Some("Config 2".to_string()));
}

#[tokio::test]
async fn test_list_playground_settings_empty() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/playground-settings")
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body("[]")
        .create_async()
        .await;

    let client = create_test_client(&server);
    let params = ListPlaygroundSettingsParams::default();

    let result = client.list_playground_settings(params).await;

    mock.assert_async().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_list_playground_settings_with_pagination() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/playground-settings")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "50".into()),
            mockito::Matcher::UrlEncoded("offset".into(), "100".into()),
        ]))
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(
            json!([create_sample_response_json(
                "550e8400-e29b-41d4-a716-446655440003",
                "Config 3"
            ),])
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);
    let params = ListPlaygroundSettingsParams {
        limit: Some(50),
        offset: Some(100),
    };

    let result = client.list_playground_settings(params).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

// ============================================================================
// Create Playground Settings Tests
// ============================================================================

#[tokio::test]
async fn test_create_playground_settings_success() {
    let mut server = Server::new_async().await;

    let created_id = "660e8400-e29b-41d4-a716-446655440000";
    let response_json = create_sample_response_json(created_id, "New Config");

    let mock = server
        .mock("POST", "/api/v1/playground-settings")
        .match_header("x-api-key", "test-api-key")
        .match_header("x-organization-id", "test-org-id")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(response_json.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server);
    let request = PlaygroundSettingsCreateRequest {
        name: Some("New Config".to_string()),
        description: Some("Test configuration".to_string()),
        settings: json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
            "kwargs": {
                "model": "claude-3-5-sonnet-20241022",
                "temperature": 0.0
            }
        }),
        options: PlaygroundSavedOptions {
            requests_per_second: Some(10),
        },
    };

    let result = client.create_playground_settings(request).await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let created = result.unwrap();
    assert_eq!(created.id, Uuid::parse_str(created_id).unwrap());
    assert_eq!(created.name, Some("New Config".to_string()));
}

#[tokio::test]
async fn test_create_playground_settings_minimal() {
    let mut server = Server::new_async().await;

    let created_id = "770e8400-e29b-41d4-a716-446655440000";
    let response_json = json!({
        "id": created_id,
        "settings": {"key": "value"},
        "options": {},
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-01-15T10:30:00Z"
    });

    let mock = server
        .mock("POST", "/api/v1/playground-settings")
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(response_json.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server);
    let request = PlaygroundSettingsCreateRequest {
        name: None,
        description: None,
        settings: json!({"key": "value"}),
        options: PlaygroundSavedOptions::default(),
    };

    let result = client.create_playground_settings(request).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_playground_settings_validation_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/playground-settings")
        .match_header("x-api-key", "test-api-key")
        .with_status(422)
        .with_body(
            json!({
                "detail": "Validation error: settings field is required"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);
    let request = PlaygroundSettingsCreateRequest {
        name: Some("Invalid Config".to_string()),
        description: None,
        settings: json!({}), // Invalid empty settings
        options: PlaygroundSavedOptions::default(),
    };

    let result = client.create_playground_settings(request).await;

    mock.assert_async().await;
    assert!(result.is_err());
}

// ============================================================================
// Update Playground Settings Tests
// ============================================================================

#[tokio::test]
async fn test_update_playground_settings_success() {
    let mut server = Server::new_async().await;

    let settings_id = Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap();
    let response_json = create_sample_response_json(&settings_id.to_string(), "Updated Config");

    let mock = server
        .mock(
            "PATCH",
            format!("/api/v1/playground-settings/{}", settings_id).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .match_header("x-organization-id", "test-org-id")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_body(response_json.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server);
    let request = PlaygroundSettingsUpdateRequest {
        name: Some("Updated Config".to_string()),
        description: Some("Updated description".to_string()),
        settings: None,
        options: None,
    };

    let result = client
        .update_playground_settings(settings_id, request)
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.id, settings_id);
    assert_eq!(updated.name, Some("Updated Config".to_string()));
}

#[tokio::test]
async fn test_update_playground_settings_partial() {
    let mut server = Server::new_async().await;

    let settings_id = Uuid::parse_str("990e8400-e29b-41d4-a716-446655440000").unwrap();
    let response_json = create_sample_response_json(&settings_id.to_string(), "Partially Updated");

    let mock = server
        .mock(
            "PATCH",
            format!("/api/v1/playground-settings/{}", settings_id).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(response_json.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server);
    // Update only name, leave other fields unchanged
    let request = PlaygroundSettingsUpdateRequest {
        name: Some("Partially Updated".to_string()),
        description: None,
        settings: None,
        options: None,
    };

    let result = client
        .update_playground_settings(settings_id, request)
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_playground_settings_not_found() {
    let mut server = Server::new_async().await;

    let settings_id = Uuid::parse_str("000e8400-e29b-41d4-a716-446655440000").unwrap();

    let mock = server
        .mock(
            "PATCH",
            format!("/api/v1/playground-settings/{}", settings_id).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .with_status(404)
        .with_body(
            json!({
                "detail": "Playground settings not found"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);
    let request = PlaygroundSettingsUpdateRequest {
        name: Some("Non-existent".to_string()),
        ..Default::default()
    };

    let result = client
        .update_playground_settings(settings_id, request)
        .await;

    mock.assert_async().await;
    assert!(result.is_err());
}

// ============================================================================
// Delete Playground Settings Tests
// ============================================================================

#[tokio::test]
async fn test_delete_playground_settings_success() {
    let mut server = Server::new_async().await;

    let settings_id = Uuid::parse_str("aa0e8400-e29b-41d4-a716-446655440000").unwrap();

    let mock = server
        .mock(
            "DELETE",
            format!("/api/v1/playground-settings/{}", settings_id).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .match_header("x-organization-id", "test-org-id")
        .with_status(204)
        .create_async()
        .await;

    let client = create_test_client(&server);

    let result = client.delete_playground_settings(settings_id).await;

    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_playground_settings_not_found() {
    let mut server = Server::new_async().await;

    let settings_id = Uuid::parse_str("bb0e8400-e29b-41d4-a716-446655440000").unwrap();

    let mock = server
        .mock(
            "DELETE",
            format!("/api/v1/playground-settings/{}", settings_id).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .with_status(404)
        .with_body(
            json!({
                "detail": "Playground settings not found"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);

    let result = client.delete_playground_settings(settings_id).await;

    mock.assert_async().await;
    assert!(result.is_err());
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[tokio::test]
async fn test_create_update_delete_workflow() {
    let mut server = Server::new_async().await;

    let settings_id = "cc0e8400-e29b-41d4-a716-446655440000";
    let uuid = Uuid::parse_str(settings_id).unwrap();

    // Step 1: Mock create
    let create_response = create_sample_response_json(settings_id, "Workflow Config");
    let create_mock = server
        .mock("POST", "/api/v1/playground-settings")
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(create_response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server);
    let create_request = PlaygroundSettingsCreateRequest {
        name: Some("Workflow Config".to_string()),
        description: Some("Testing workflow".to_string()),
        settings: json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
            "kwargs": {"model": "claude-3-5-sonnet-20241022"}
        }),
        options: PlaygroundSavedOptions::default(),
    };

    let create_result = client.create_playground_settings(create_request).await;
    create_mock.assert_async().await;
    assert!(create_result.is_ok());

    // Step 2: Mock update
    let update_response = create_sample_response_json(settings_id, "Updated Workflow Config");
    let update_mock = server
        .mock(
            "PATCH",
            format!("/api/v1/playground-settings/{}", uuid).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(update_response.to_string())
        .create_async()
        .await;

    let update_request = PlaygroundSettingsUpdateRequest {
        name: Some("Updated Workflow Config".to_string()),
        ..Default::default()
    };

    let update_result = client
        .update_playground_settings(uuid, update_request)
        .await;
    update_mock.assert_async().await;
    assert!(update_result.is_ok());

    // Step 3: Mock delete
    let delete_mock = server
        .mock(
            "DELETE",
            format!("/api/v1/playground-settings/{}", uuid).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .with_status(204)
        .create_async()
        .await;

    let delete_result = client.delete_playground_settings(uuid).await;
    delete_mock.assert_async().await;
    assert!(delete_result.is_ok());
}
