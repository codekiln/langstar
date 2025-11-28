//! HTTP-mocked integration tests for dataset operations.
//!
//! These tests verify the dataset and example CRUD methods
//! using mockito to mock the LangSmith API responses.

use langstar_sdk::{
    AuthConfig, DataType, DatasetCreate, DatasetUpdate, ExampleCreate, ExampleUpdate,
    LangchainClient, ListDatasetsParams, ListExamplesParams,
};
use mockito::{Matcher, Server};
use serde_json::json;
use uuid::Uuid;

/// Helper function to make a minimal valid Dataset JSON response
fn make_dataset_json(id: &str, name: &str, example_count: i64) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "tenant_id": "87654321-4321-4321-4321-210987654321",
        "example_count": example_count,
        "session_count": 0,
        "modified_at": "2024-01-01T12:00:00Z",
        "data_type": "kv"
    })
}

/// Helper function to make a minimal Dataset update response (DatasetSchemaForUpdate)
/// This matches what the PATCH endpoint actually returns - no example_count, session_count, or modified_at
fn make_dataset_update_json(id: &str, name: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "tenant_id": "87654321-4321-4321-4321-210987654321",
        "data_type": "kv"
    })
}

/// Helper function to make a minimal valid Example JSON response
fn make_example_json(id: &str, dataset_id: &str) -> serde_json::Value {
    json!({
        "id": id,
        "dataset_id": dataset_id,
        "inputs": {"question": "test"},
        "name": format!("example-{}", &id[..8]),
        "outputs": {"answer": "test"},
        "created_at": "2024-01-01T12:00:00Z"
    })
}

/// Helper function to create a test client with mock server
fn create_test_client(server_url: &str) -> LangchainClient {
    let auth = AuthConfig::new(
        Some("test_langsmith_key".to_string()),
        Some("test_langgraph_key".to_string()),
        None,
        None,
    );

    LangchainClient::with_base_urls(
        auth,
        server_url.to_string(),
        "https://api.langgraph.cloud".to_string(),
        "https://api.host.langchain.com".to_string(),
    )
    .expect("Failed to create test client")
}

// ═══════════════════════════════════════════════════════════════════════════
// Dataset CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_dataset() {
    let mut server = Server::new_async().await;

    let response_body =
        make_dataset_json("12345678-1234-1234-1234-123456789012", "Test Dataset", 0);

    let mock = server
        .mock("POST", "/api/v1/datasets")
        .match_body(Matcher::PartialJson(json!({
            "name": "Test Dataset",
            "data_type": "kv"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = DatasetCreate {
        name: "Test Dataset".to_string(),
        data_type: Some(DataType::Kv),
        ..Default::default()
    };

    let dataset = client
        .create_dataset(request)
        .await
        .expect("create_dataset failed");

    assert_eq!(dataset.name, "Test Dataset");
    assert_eq!(dataset.example_count, Some(0));
    assert_eq!(dataset.data_type, Some(DataType::Kv));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_dataset_with_description() {
    let mut server = Server::new_async().await;

    let mut response = make_dataset_json("12345678-1234-1234-1234-123456789012", "Eval Dataset", 0);
    response["description"] = json!("A dataset for evaluation");
    response["data_type"] = json!("chat");

    let mock = server
        .mock("POST", "/api/v1/datasets")
        .match_body(Matcher::PartialJson(json!({
            "name": "Eval Dataset",
            "description": "A dataset for evaluation",
            "data_type": "chat"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = DatasetCreate {
        name: "Eval Dataset".to_string(),
        description: Some("A dataset for evaluation".to_string()),
        data_type: Some(DataType::Chat),
        ..Default::default()
    };

    let dataset = client
        .create_dataset(request)
        .await
        .expect("create_dataset failed");

    assert_eq!(dataset.name, "Eval Dataset");
    assert_eq!(
        dataset.description,
        Some("A dataset for evaluation".to_string())
    );
    assert_eq!(dataset.data_type, Some(DataType::Chat));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_datasets() {
    let mut server = Server::new_async().await;

    let response_body = json!([
        make_dataset_json("12345678-1234-1234-1234-123456789001", "Dataset 1", 10),
        make_dataset_json("12345678-1234-1234-1234-123456789002", "Dataset 2", 20),
        make_dataset_json("12345678-1234-1234-1234-123456789003", "Dataset 3", 30),
    ]);

    let mock = server
        .mock("GET", "/api/v1/datasets")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListDatasetsParams::default();
    let datasets = client
        .list_datasets(params)
        .await
        .expect("list_datasets failed");

    assert_eq!(datasets.len(), 3);
    assert_eq!(datasets[0].name, "Dataset 1");
    assert_eq!(datasets[0].example_count, Some(10));
    assert_eq!(datasets[1].name, "Dataset 2");
    assert_eq!(datasets[2].name, "Dataset 3");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_datasets_with_filters() {
    let mut server = Server::new_async().await;

    let response_body = json!([make_dataset_json(
        "12345678-1234-1234-1234-123456789001",
        "Chat Dataset",
        5
    ),]);

    let mock = server
        .mock("GET", "/api/v1/datasets")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("data_type".into(), "chat".into()),
            Matcher::UrlEncoded("name_contains".into(), "Chat".into()),
            Matcher::UrlEncoded("limit".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListDatasetsParams {
        data_type: Some(DataType::Chat),
        name_contains: Some("Chat".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let datasets = client
        .list_datasets(params)
        .await
        .expect("list_datasets failed");

    assert_eq!(datasets.len(), 1);
    assert_eq!(datasets[0].name, "Chat Dataset");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_datasets_empty() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/datasets")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListDatasetsParams::default();
    let datasets = client
        .list_datasets(params)
        .await
        .expect("list_datasets failed");

    assert!(datasets.is_empty());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_dataset() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    let response_body = make_dataset_json(dataset_id, "My Dataset", 42);

    let mock = server
        .mock("GET", format!("/api/v1/datasets/{}", dataset_id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let uuid = Uuid::parse_str(dataset_id).unwrap();
    let dataset = client.get_dataset(uuid).await.expect("get_dataset failed");

    assert_eq!(dataset.name, "My Dataset");
    assert_eq!(dataset.example_count, Some(42));
    assert_eq!(dataset.id.to_string(), dataset_id);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_dataset_not_found() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";

    let mock = server
        .mock("GET", format!("/api/v1/datasets/{}", dataset_id).as_str())
        .with_status(404)
        .with_body(r#"{"detail": "Dataset not found"}"#)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let uuid = Uuid::parse_str(dataset_id).unwrap();
    let result = client.get_dataset(uuid).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("404") || err.to_string().contains("not found"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_dataset() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    // PATCH responses return DatasetSchemaForUpdate, not full Dataset
    let mut response = make_dataset_update_json(dataset_id, "Updated Dataset");
    response["description"] = json!("New description");

    let mock = server
        .mock("PATCH", format!("/api/v1/datasets/{}", dataset_id).as_str())
        .match_body(Matcher::PartialJson(json!({
            "name": "Updated Dataset",
            "description": "New description"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let uuid = Uuid::parse_str(dataset_id).unwrap();
    let request = DatasetUpdate {
        name: Some("Updated Dataset".to_string()),
        description: Some("New description".to_string()),
        ..Default::default()
    };

    let dataset = client
        .update_dataset(uuid, request)
        .await
        .expect("update_dataset failed");

    assert_eq!(dataset.name, "Updated Dataset");
    assert_eq!(dataset.description, Some("New description".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_dataset() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";

    let mock = server
        .mock(
            "DELETE",
            format!("/api/v1/datasets/{}", dataset_id).as_str(),
        )
        .with_status(200)
        .with_body("")
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let uuid = Uuid::parse_str(dataset_id).unwrap();
    client
        .delete_dataset(uuid)
        .await
        .expect("delete_dataset failed");

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Example CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_example() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    let example_id = "22345678-1234-1234-1234-123456789012";
    let response_body = make_example_json(example_id, dataset_id);

    let mock = server
        .mock("POST", "/api/v1/examples")
        .match_body(Matcher::PartialJson(json!({
            "dataset_id": dataset_id,
            "inputs": {"question": "What is 2+2?"},
            "outputs": {"answer": "4"}
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = ExampleCreate {
        dataset_id: Uuid::parse_str(dataset_id).unwrap(),
        inputs: Some(json!({"question": "What is 2+2?"})),
        outputs: Some(json!({"answer": "4"})),
        ..Default::default()
    };

    let example = client
        .create_example(request)
        .await
        .expect("create_example failed");

    assert_eq!(example.id.to_string(), example_id);
    assert_eq!(example.dataset_id.to_string(), dataset_id);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_examples() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    let response_body = json!([
        make_example_json("22345678-1234-1234-1234-123456789001", dataset_id),
        make_example_json("22345678-1234-1234-1234-123456789002", dataset_id),
        make_example_json("22345678-1234-1234-1234-123456789003", dataset_id),
    ]);

    let mock = server
        .mock("GET", "/api/v1/examples")
        .match_query(Matcher::UrlEncoded("dataset".into(), dataset_id.into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListExamplesParams {
        dataset: Some(Uuid::parse_str(dataset_id).unwrap()),
        ..Default::default()
    };

    let examples = client
        .list_examples(params)
        .await
        .expect("list_examples failed");

    assert_eq!(examples.len(), 3);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_examples_with_limit() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    let response_body = json!([make_example_json(
        "22345678-1234-1234-1234-123456789001",
        dataset_id
    ),]);

    let mock = server
        .mock("GET", "/api/v1/examples")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("dataset".into(), dataset_id.into()),
            Matcher::UrlEncoded("limit".into(), "1".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListExamplesParams {
        dataset: Some(Uuid::parse_str(dataset_id).unwrap()),
        limit: Some(1),
        ..Default::default()
    };

    let examples = client
        .list_examples(params)
        .await
        .expect("list_examples failed");

    assert_eq!(examples.len(), 1);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_example() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    let example_id = "22345678-1234-1234-1234-123456789012";
    let response_body = make_example_json(example_id, dataset_id);

    let mock = server
        .mock("GET", format!("/api/v1/examples/{}", example_id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let uuid = Uuid::parse_str(example_id).unwrap();
    let example = client.get_example(uuid).await.expect("get_example failed");

    assert_eq!(example.id.to_string(), example_id);
    assert_eq!(example.dataset_id.to_string(), dataset_id);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_example() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    let example_id = "22345678-1234-1234-1234-123456789012";
    let mut response = make_example_json(example_id, dataset_id);
    response["outputs"] = json!({"answer": "updated"});

    let mock = server
        .mock("PATCH", format!("/api/v1/examples/{}", example_id).as_str())
        .match_body(Matcher::PartialJson(json!({
            "outputs": {"answer": "updated"}
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let uuid = Uuid::parse_str(example_id).unwrap();
    let request = ExampleUpdate {
        outputs: Some(json!({"answer": "updated"})),
        ..Default::default()
    };

    let example = client
        .update_example(uuid, request)
        .await
        .expect("update_example failed");

    assert_eq!(example.outputs.unwrap()["answer"], "updated");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_example() {
    let mut server = Server::new_async().await;

    let example_id = "22345678-1234-1234-1234-123456789012";

    let mock = server
        .mock(
            "DELETE",
            format!("/api/v1/examples/{}", example_id).as_str(),
        )
        .with_status(200)
        .with_body("")
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let uuid = Uuid::parse_str(example_id).unwrap();
    client
        .delete_example(uuid)
        .await
        .expect("delete_example failed");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_bulk_create_examples() {
    let mut server = Server::new_async().await;

    let dataset_id = "12345678-1234-1234-1234-123456789012";
    let response_body = json!([
        make_example_json("22345678-1234-1234-1234-123456789001", dataset_id),
        make_example_json("22345678-1234-1234-1234-123456789002", dataset_id),
    ]);

    let mock = server
        .mock("POST", "/api/v1/examples/bulk")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let dataset_uuid = Uuid::parse_str(dataset_id).unwrap();
    let examples = vec![
        ExampleCreate {
            dataset_id: dataset_uuid,
            inputs: Some(json!({"q": "1+1"})),
            outputs: Some(json!({"a": "2"})),
            ..Default::default()
        },
        ExampleCreate {
            dataset_id: dataset_uuid,
            inputs: Some(json!({"q": "2+2"})),
            outputs: Some(json!({"a": "4"})),
            ..Default::default()
        },
    ];

    let created = client
        .bulk_create_examples(examples)
        .await
        .expect("bulk_create_examples failed");

    assert_eq!(created.len(), 2);

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Error Handling Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_dataset_api_error_401() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/datasets")
        .with_status(401)
        .with_body(r#"{"detail": "Invalid API key"}"#)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListDatasetsParams::default();
    let result = client.list_datasets(params).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("401") || err.to_string().contains("Invalid API key"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_dataset_api_error_500() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/datasets")
        .with_status(500)
        .with_body(r#"{"detail": "Internal server error"}"#)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = DatasetCreate {
        name: "Test".to_string(),
        ..Default::default()
    };

    let result = client.create_dataset(request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("500") || err.to_string().contains("server error"));

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Live API Integration Tests (Ignored by Default)
// ═══════════════════════════════════════════════════════════════════════════

/// Integration test for listing datasets from the live LangSmith API.
///
/// Run with: cargo test --test dataset_test -- --ignored --nocapture
#[tokio::test]
#[ignore]
async fn test_list_datasets_live_api() {
    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    let params = ListDatasetsParams {
        limit: Some(5),
        ..Default::default()
    };

    let result = client.list_datasets(params).await;

    match result {
        Ok(datasets) => {
            println!("Found {} datasets", datasets.len());
            for ds in datasets.iter().take(3) {
                println!(
                    "  - {} ({} examples)",
                    ds.name,
                    ds.example_count.unwrap_or(0)
                );
            }
        }
        Err(e) => {
            panic!("Failed to list datasets: {:?}", e);
        }
    }
}

/// Full CRUD integration test.
///
/// Run with: cargo test --test dataset_test -- --ignored --nocapture test_dataset_crud_live_api
#[tokio::test]
#[ignore]
async fn test_dataset_crud_live_api() {
    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    // 1. Create dataset
    let unique_name = format!("test-dataset-{}", &Uuid::new_v4().to_string()[..8]);
    let request = DatasetCreate {
        name: unique_name.clone(),
        description: Some("Integration test dataset".to_string()),
        data_type: Some(DataType::Kv),
        ..Default::default()
    };

    let dataset = client
        .create_dataset(request)
        .await
        .expect("Failed to create dataset");
    println!("✓ Created dataset: {} ({})", dataset.name, dataset.id);

    // 2. Get dataset
    let fetched = client
        .get_dataset(dataset.id)
        .await
        .expect("Failed to get dataset");
    assert_eq!(fetched.name, unique_name);
    println!("✓ Fetched dataset: {}", fetched.name);

    // 3. Create example
    let example_request = ExampleCreate {
        dataset_id: dataset.id,
        inputs: Some(json!({"question": "What is 2+2?"})),
        outputs: Some(json!({"answer": "4"})),
        ..Default::default()
    };

    let example = client
        .create_example(example_request)
        .await
        .expect("Failed to create example");
    println!("✓ Created example: {}", example.id);

    // 4. List examples
    let params = ListExamplesParams {
        dataset: Some(dataset.id),
        ..Default::default()
    };
    let examples = client
        .list_examples(params)
        .await
        .expect("Failed to list examples");
    assert!(!examples.is_empty());
    println!("✓ Listed {} examples", examples.len());

    // 5. Delete example
    client
        .delete_example(example.id)
        .await
        .expect("Failed to delete example");
    println!("✓ Deleted example");

    // 6. Delete dataset
    client
        .delete_dataset(dataset.id)
        .await
        .expect("Failed to delete dataset");
    println!("✓ Deleted dataset");

    println!("✓ Full CRUD integration test passed");
}
