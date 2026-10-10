//! HTTP-mocked integration tests for runs query functionality.
//!
//! These tests verify the query_runs and query_runs_paginated methods
//! using mockito to mock the LangSmith API responses.

use langstar_sdk::{AuthConfig, LangchainClient, QueryRunsRequest, RunSelectField, RunType};
use mockito::{Matcher, Server};
use serde_json::json;
use tokio_stream::StreamExt;

/// Helper function to create a run as `POST /api/v2/runs/query` returns it
fn make_run_json(id: &str, name: &str, run_type: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "run_type": run_type.to_uppercase(),
        "trace_id": "223e4567-e89b-12d3-a456-426614174001",
        "status": "SUCCESS",
        "project_id": "323e4567-e89b-12d3-a456-426614174002"
    })
}

/// Helper function to create a test client with mock server
fn create_test_client(server_url: &str) -> LangchainClient {
    let auth = AuthConfig::new(Some("test_langsmith_key".to_string()), None, None);

    LangchainClient::with_base_urls(
        auth,
        server_url.to_string(),
        "https://api.langgraph.cloud".to_string(),
        "https://api.host.langchain.com".to_string(),
    )
    .expect("Failed to create test client")
}

#[tokio::test]
async fn test_query_runs_single_page() {
    let mut server = Server::new_async().await;

    let response_body = json!({
        "items": [
            make_run_json("123e4567-e89b-12d3-a456-426614174000", "ChatOpenAI", "llm"),
            make_run_json("223e4567-e89b-12d3-a456-426614174001", "ToolExecutor", "tool"),
        ],
        "next_cursor": null
    });

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest {
        is_root: Some(true),
        page_size: Some(10),
        ..Default::default()
    };

    let response = client.query_runs(request).await.expect("query_runs failed");

    assert_eq!(response.items.len(), 2);
    assert_eq!(response.items[0].name.as_deref(), Some("ChatOpenAI"));
    assert_eq!(response.items[0].run_type, Some(RunType::Llm));
    assert_eq!(response.items[1].name.as_deref(), Some("ToolExecutor"));
    assert_eq!(response.items[1].run_type, Some(RunType::Tool));
    assert!(response.next_cursor.is_none());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_with_filter() {
    let mut server = Server::new_async().await;

    let response_body = json!({
        "items": [
            make_run_json("123e4567-e89b-12d3-a456-426614174000", "FailedChain", "chain"),
        ],
        "next_cursor": null
    });

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .match_body(Matcher::PartialJson(json!({
            "filter": "eq(status, \"error\")"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest {
        filter: Some("eq(status, \"error\")".to_string()),
        ..Default::default()
    };

    let response = client.query_runs(request).await.expect("query_runs failed");

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].name.as_deref(), Some("FailedChain"));
    assert_eq!(response.items[0].run_type, Some(RunType::Chain));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_empty_response() {
    let mut server = Server::new_async().await;

    let response_body = json!({
        "items": [],
        "next_cursor": null});

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest::default();

    let response = client.query_runs(request).await.expect("query_runs failed");

    assert!(response.items.is_empty());
    assert!(response.next_cursor.is_none());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_paginated_single_page() {
    let mut server = Server::new_async().await;

    let response_body = json!({
        "items": [
            make_run_json("123e4567-e89b-12d3-a456-426614174000", "Run1", "llm"),
            make_run_json("223e4567-e89b-12d3-a456-426614174001", "Run2", "llm"),
        ],
        "next_cursor": null});

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest::default();
    let mut stream = client.query_runs_paginated(request, None);

    let mut runs = Vec::new();
    while let Some(result) = stream.next().await {
        runs.push(result.expect("Expected successful run"));
    }

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].name.as_deref(), Some("Run1"));
    assert_eq!(runs[1].name.as_deref(), Some("Run2"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_paginated_multiple_pages() {
    let mut server = Server::new_async().await;

    // First page response
    let page1_response = json!({
        "items": [
            make_run_json("123e4567-e89b-12d3-a456-426614174001", "Run1", "llm"),
            make_run_json("223e4567-e89b-12d3-a456-426614174002", "Run2", "llm"),
        ],
        "next_cursor": "cursor_page2"
    });

    // Second page response
    let page2_response = json!({
        "items": [
            make_run_json("323e4567-e89b-12d3-a456-426614174003", "Run3", "llm"),
            make_run_json("423e4567-e89b-12d3-a456-426614174004", "Run4", "llm"),
        ],
        "next_cursor": null});

    // In mockito, mocks are matched in LIFO order (last created = first matched)
    // Create the cursor mock first so it matches second (when cursor is present)
    let mock2 = server
        .mock("POST", "/api/v2/runs/query")
        .match_body(Matcher::PartialJson(json!({
            "cursor": "cursor_page2"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page2_response.to_string())
        .expect(1)
        .create_async()
        .await;

    // Create the "no cursor" mock last so it's checked first (LIFO).
    // Although PartialJson({}) is less specific than the cursor mock above,
    // mockito's LIFO ordering means this mock is checked first on the initial request
    // (which has no cursor), and the more specific cursor mock is checked on subsequent requests.
    let mock1 = server
        .mock("POST", "/api/v2/runs/query")
        .match_body(Matcher::PartialJson(json!({})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page1_response.to_string())
        .expect(1)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest::default();
    let mut stream = client.query_runs_paginated(request, None);

    let mut runs = Vec::new();
    while let Some(result) = stream.next().await {
        runs.push(result.expect("Expected successful run"));
    }

    assert_eq!(runs.len(), 4);
    assert_eq!(runs[0].name.as_deref(), Some("Run1"));
    assert_eq!(runs[1].name.as_deref(), Some("Run2"));
    assert_eq!(runs[2].name.as_deref(), Some("Run3"));
    assert_eq!(runs[3].name.as_deref(), Some("Run4"));

    mock1.assert_async().await;
    mock2.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_paginated_with_limit() {
    let mut server = Server::new_async().await;

    // First page response with next cursor
    let page1_response = json!({
        "items": [
            make_run_json("123e4567-e89b-12d3-a456-426614174001", "Run1", "llm"),
            make_run_json("223e4567-e89b-12d3-a456-426614174002", "Run2", "llm"),
            make_run_json("323e4567-e89b-12d3-a456-426614174003", "Run3", "llm"),
        ],
        "next_cursor": "cursor_page2"
    });

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page1_response.to_string())
        .expect(1)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest::default();
    // Limit to 2 runs - should stop before fetching page 2
    let mut stream = client.query_runs_paginated(request, Some(2));

    let mut runs = Vec::new();
    while let Some(result) = stream.next().await {
        runs.push(result.expect("Expected successful run"));
    }

    // Should only return 2 runs due to limit
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].name.as_deref(), Some("Run1"));
    assert_eq!(runs[1].name.as_deref(), Some("Run2"));

    // Only first page should be requested
    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_api_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .with_status(401)
        .with_body(r#"{"detail": "Invalid API key"}"#)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest::default();
    let result = client.query_runs(request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("401") || err.to_string().contains("Invalid API key"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_with_run_type_filter() {
    let mut server = Server::new_async().await;

    let response_body = json!({
        "items": [
            make_run_json("123e4567-e89b-12d3-a456-426614174000", "MyRetriever", "retriever"),
        ],
        "next_cursor": null});

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .match_body(Matcher::PartialJson(json!({
            "run_type": "RETRIEVER"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest {
        run_type: Some(RunType::Retriever),
        ..Default::default()
    };

    let response = client.query_runs(request).await.expect("query_runs failed");

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].run_type, Some(RunType::Retriever));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_with_is_root_filter() {
    let mut server = Server::new_async().await;

    let response_body = json!({
        "items": [
            make_run_json("123e4567-e89b-12d3-a456-426614174000", "RootRun", "chain"),
        ],
        "next_cursor": null});

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .match_body(Matcher::PartialJson(json!({
            "is_root": true
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest {
        is_root: Some(true),
        ..Default::default()
    };

    let response = client.query_runs(request).await.expect("query_runs failed");

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].name.as_deref(), Some("RootRun"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_query_runs_sends_v2_body() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v2/runs/query")
        .match_body(Matcher::Json(json!({
            "project_ids": ["323e4567-e89b-12d3-a456-426614174002"],
            "min_start_time": "2024-01-01T00:00:00Z",
            "page_size": 25,
            "selects": ["ID", "NAME", "PROJECT_ID"]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"items": [], "next_cursor": null}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = QueryRunsRequest {
        project_ids: Some(vec![
            uuid::Uuid::parse_str("323e4567-e89b-12d3-a456-426614174002").unwrap(),
        ]),
        min_start_time: Some("2024-01-01T00:00:00Z".parse().unwrap()),
        page_size: Some(25),
        selects: Some(vec![
            RunSelectField::Id,
            RunSelectField::Name,
            RunSelectField::ProjectId,
        ]),
        ..Default::default()
    };

    let response = client.query_runs(request).await.expect("query_runs failed");
    assert!(response.items.is_empty());

    mock.assert_async().await;
}
