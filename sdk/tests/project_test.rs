//! HTTP-mocked integration tests for project operations.
//!
//! These tests verify the project CRUD methods using mockito to mock
//! the LangSmith API responses.

use langstar_sdk::{
    AuthConfig, LangchainClient, ListProjectsParams, ProjectCreate, ProjectUpdate, TraceTier,
};
use mockito::{Matcher, Server};
use serde_json::json;
use uuid::Uuid;

/// Helper function to make a minimal valid Project JSON response
fn make_project_json(id: &str, name: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "tenant_id": "87654321-4321-4321-4321-210987654321",
        "start_time": "2024-01-01T12:00:00Z"
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

// ═══════════════════════════════════════════════════════════════════════════
// Create Project Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_project_minimal() {
    let mut server = Server::new_async().await;

    let response_body = make_project_json("12345678-1234-1234-1234-123456789012", "Test Project");

    let mock = server
        .mock("POST", "/api/v1/sessions")
        .match_body(Matcher::PartialJson(json!({
            "name": "Test Project"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = ProjectCreate {
        name: Some("Test Project".to_string()),
        ..Default::default()
    };

    let project = client
        .create_project(request)
        .await
        .expect("create_project failed");

    assert_eq!(project.name, Some("Test Project".to_string()));
    assert_eq!(
        project.id,
        Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap()
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_project_with_description() {
    let mut server = Server::new_async().await;

    let mut response = make_project_json("12345678-1234-1234-1234-123456789012", "Eval Project");
    response["description"] = json!("A project for evaluation");

    let mock = server
        .mock("POST", "/api/v1/sessions")
        .match_body(Matcher::PartialJson(json!({
            "name": "Eval Project",
            "description": "A project for evaluation"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = ProjectCreate {
        name: Some("Eval Project".to_string()),
        description: Some("A project for evaluation".to_string()),
        ..Default::default()
    };

    let project = client
        .create_project(request)
        .await
        .expect("create_project failed");

    assert_eq!(project.name, Some("Eval Project".to_string()));
    assert_eq!(
        project.description,
        Some("A project for evaluation".to_string())
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_project_with_metadata() {
    let mut server = Server::new_async().await;

    let metadata = json!({"environment": "staging", "version": "1.0"});
    let mut response = make_project_json("12345678-1234-1234-1234-123456789012", "Meta Project");
    response["extra"] = metadata.clone();

    let mock = server
        .mock("POST", "/api/v1/sessions")
        .match_body(Matcher::PartialJson(json!({
            "name": "Meta Project",
            "extra": metadata
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = ProjectCreate {
        name: Some("Meta Project".to_string()),
        extra: Some(metadata.clone()),
        ..Default::default()
    };

    let project = client
        .create_project(request)
        .await
        .expect("create_project failed");

    assert_eq!(project.name, Some("Meta Project".to_string()));
    assert_eq!(project.extra, Some(metadata));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_project_with_trace_tier() {
    let mut server = Server::new_async().await;

    let mut response = make_project_json("12345678-1234-1234-1234-123456789012", "Tier Project");
    response["trace_tier"] = json!("shortlived");

    let mock = server
        .mock("POST", "/api/v1/sessions")
        .match_body(Matcher::PartialJson(json!({
            "name": "Tier Project",
            "trace_tier": "shortlived"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = ProjectCreate {
        name: Some("Tier Project".to_string()),
        trace_tier: Some(TraceTier::Shortlived),
        ..Default::default()
    };

    let project = client
        .create_project(request)
        .await
        .expect("create_project failed");

    assert_eq!(project.name, Some("Tier Project".to_string()));
    assert_eq!(project.trace_tier, Some(TraceTier::Shortlived));

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// List Projects Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_list_projects_all() {
    let mut server = Server::new_async().await;

    let response_body = json!([
        make_project_json("11111111-1111-1111-1111-111111111111", "Project Alpha"),
        make_project_json("22222222-2222-2222-2222-222222222222", "Project Beta"),
    ]);

    let mock = server
        .mock("GET", "/api/v1/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let projects = client
        .list_projects(ListProjectsParams::default())
        .await
        .expect("list_projects failed");

    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, Some("Project Alpha".to_string()));
    assert_eq!(projects[1].name, Some("Project Beta".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_projects_with_name_filter() {
    let mut server = Server::new_async().await;

    let response_body = json!([make_project_json(
        "11111111-1111-1111-1111-111111111111",
        "my-project"
    ),]);

    let mock = server
        .mock("GET", "/api/v1/sessions")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "name".to_string(),
            "my-project".to_string(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListProjectsParams {
        name: Some("my-project".to_string()),
        ..Default::default()
    };

    let projects = client
        .list_projects(params)
        .await
        .expect("list_projects failed");

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, Some("my-project".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_projects_with_name_contains_filter() {
    let mut server = Server::new_async().await;

    let response_body = json!([
        make_project_json("11111111-1111-1111-1111-111111111111", "prod-api"),
        make_project_json("22222222-2222-2222-2222-222222222222", "prod-web"),
    ]);

    let mock = server
        .mock("GET", "/api/v1/sessions")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "name_contains".to_string(),
            "prod".to_string(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListProjectsParams {
        name_contains: Some("prod".to_string()),
        ..Default::default()
    };

    let projects = client
        .list_projects(params)
        .await
        .expect("list_projects failed");

    assert_eq!(projects.len(), 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_projects_with_limit() {
    let mut server = Server::new_async().await;

    let response_body = json!([
        make_project_json("11111111-1111-1111-1111-111111111111", "Project 1"),
        make_project_json("22222222-2222-2222-2222-222222222222", "Project 2"),
    ]);

    let mock = server
        .mock("GET", "/api/v1/sessions")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "limit".to_string(),
            "5".to_string(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListProjectsParams {
        limit: Some(5),
        ..Default::default()
    };

    let projects = client
        .list_projects(params)
        .await
        .expect("list_projects failed");

    assert_eq!(projects.len(), 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_projects_with_include_stats() {
    let mut server = Server::new_async().await;

    let mut project_with_stats =
        make_project_json("11111111-1111-1111-1111-111111111111", "Stats Project");
    project_with_stats["run_count"] = json!(42);
    project_with_stats["latency_p50"] = json!(0.5);
    project_with_stats["latency_p99"] = json!(1.2);

    let response_body = json!([project_with_stats]);

    let mock = server
        .mock("GET", "/api/v1/sessions")
        .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
            "include_stats".to_string(),
            "true".to_string(),
        )]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let params = ListProjectsParams {
        include_stats: Some(true),
        ..Default::default()
    };

    let projects = client
        .list_projects(params)
        .await
        .expect("list_projects failed");

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].run_count, Some(42));
    assert_eq!(projects[0].latency_p50, Some(0.5));
    assert_eq!(projects[0].latency_p99, Some(1.2));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_projects_empty() {
    let mut server = Server::new_async().await;

    let response_body = json!([]);

    let mock = server
        .mock("GET", "/api/v1/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let projects = client
        .list_projects(ListProjectsParams::default())
        .await
        .expect("list_projects failed");

    assert_eq!(projects.len(), 0);

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Get Project Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_project_by_id() {
    let mut server = Server::new_async().await;

    let project_id = "12345678-1234-1234-1234-123456789012";
    let response_body = make_project_json(project_id, "Test Project");

    let mock = server
        .mock("GET", format!("/api/v1/sessions/{}", project_id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let project = client
        .get_project(Uuid::parse_str(project_id).unwrap())
        .await
        .expect("get_project failed");

    assert_eq!(project.name, Some("Test Project".to_string()));
    assert_eq!(project.id, Uuid::parse_str(project_id).unwrap());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_project_not_found() {
    let mut server = Server::new_async().await;

    let project_id = "12345678-1234-1234-1234-123456789012";

    let mock = server
        .mock("GET", format!("/api/v1/sessions/{}", project_id).as_str())
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"detail": "Not found"}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client
        .get_project(Uuid::parse_str(project_id).unwrap())
        .await;

    assert!(result.is_err(), "Expected error for 404 response");

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Update Project Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_update_project_description() {
    let mut server = Server::new_async().await;

    let project_id = "12345678-1234-1234-1234-123456789012";
    let mut response = make_project_json(project_id, "Test Project");
    response["description"] = json!("Updated description");

    let mock = server
        .mock("PATCH", format!("/api/v1/sessions/{}", project_id).as_str())
        .match_body(Matcher::PartialJson(json!({
            "description": "Updated description"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let update = ProjectUpdate {
        description: Some("Updated description".to_string()),
        ..Default::default()
    };

    let project = client
        .update_project(Uuid::parse_str(project_id).unwrap(), update)
        .await
        .expect("update_project failed");

    assert_eq!(project.description, Some("Updated description".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_project_name() {
    let mut server = Server::new_async().await;

    let project_id = "12345678-1234-1234-1234-123456789012";
    let response = make_project_json(project_id, "New Project Name");

    let mock = server
        .mock("PATCH", format!("/api/v1/sessions/{}", project_id).as_str())
        .match_body(Matcher::PartialJson(json!({
            "name": "New Project Name"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let update = ProjectUpdate {
        name: Some("New Project Name".to_string()),
        ..Default::default()
    };

    let project = client
        .update_project(Uuid::parse_str(project_id).unwrap(), update)
        .await
        .expect("update_project failed");

    assert_eq!(project.name, Some("New Project Name".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_project_metadata() {
    let mut server = Server::new_async().await;

    let project_id = "12345678-1234-1234-1234-123456789012";
    let metadata = json!({"updated": true, "version": "2.0"});
    let mut response = make_project_json(project_id, "Meta Project");
    response["extra"] = metadata.clone();

    let mock = server
        .mock("PATCH", format!("/api/v1/sessions/{}", project_id).as_str())
        .match_body(Matcher::PartialJson(json!({
            "extra": metadata
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let update = ProjectUpdate {
        extra: Some(metadata.clone()),
        ..Default::default()
    };

    let project = client
        .update_project(Uuid::parse_str(project_id).unwrap(), update)
        .await
        .expect("update_project failed");

    assert_eq!(project.extra, Some(metadata));

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Delete Project Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_delete_project() {
    let mut server = Server::new_async().await;

    let project_id = "12345678-1234-1234-1234-123456789012";

    let mock = server
        .mock(
            "DELETE",
            format!("/api/v1/sessions/{}", project_id).as_str(),
        )
        .with_status(204)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    client
        .delete_project(Uuid::parse_str(project_id).unwrap())
        .await
        .expect("delete_project failed");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_project_not_found() {
    let mut server = Server::new_async().await;

    let project_id = "12345678-1234-1234-1234-123456789012";

    let mock = server
        .mock(
            "DELETE",
            format!("/api/v1/sessions/{}", project_id).as_str(),
        )
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"detail": "Not found"}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client
        .delete_project(Uuid::parse_str(project_id).unwrap())
        .await;

    assert!(result.is_err(), "Expected error for 404 response");

    mock.assert_async().await;
}
