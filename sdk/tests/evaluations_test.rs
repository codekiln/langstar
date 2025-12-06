//! HTTP-mocked integration tests for evaluation operations.
//!
//! These tests verify the evaluation and feedback CRUD methods
//! using mockito to mock the LangSmith API responses.

use langstar_sdk::{
    AuthConfig, FeedbackConfig, FeedbackCreate, FeedbackType, FeedbackUpdate, LangchainClient,
};
use mockito::{Matcher, Server};
use serde_json::json;
use uuid::Uuid;

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

/// Helper function to make a minimal valid Feedback JSON response
fn make_feedback_json(id: &str, key: &str, score: f64) -> serde_json::Value {
    json!({
        "id": id,
        "key": key,
        "score": score,
        "created_at": "2024-01-01T12:00:00Z",
        "modified_at": "2024-01-01T12:00:00Z"
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Feedback CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_feedback() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let response_body = make_feedback_json(&feedback_id.to_string(), "accuracy", 0.95);

    let mock = server
        .mock("POST", "/api/v1/feedback")
        .match_body(Matcher::PartialJson(json!({
            "key": "accuracy",
            "score": 0.95,
            "run_id": run_id.to_string()
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = FeedbackCreate {
        key: "accuracy".to_string(),
        score: Some(0.95),
        run_id: Some(run_id),
        feedback_config: Some(FeedbackConfig {
            feedback_type: FeedbackType::Continuous,
            min: Some(0.0),
            max: Some(1.0),
            categories: None,
        }),
        ..Default::default()
    };

    let feedback = client
        .create_feedback(request)
        .await
        .expect("Failed to create feedback");

    assert_eq!(feedback.key, "accuracy");
    assert_eq!(feedback.score, Some(0.95));
    assert_eq!(feedback.id, feedback_id);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_create_feedback_categorical() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();

    let response_body = json!({
        "id": feedback_id.to_string(),
        "key": "correctness",
        "value": "Y",
        "run_id": run_id.to_string(),
        "created_at": "2024-01-01T12:00:00Z",
        "modified_at": "2024-01-01T12:00:00Z"
    });

    let mock = server
        .mock("POST", "/api/v1/feedback")
        .match_body(Matcher::PartialJson(json!({
            "key": "correctness",
            "value": "Y",
            "run_id": run_id.to_string()
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = FeedbackCreate {
        key: "correctness".to_string(),
        value: Some(json!("Y")),
        run_id: Some(run_id),
        ..Default::default()
    };

    let feedback = client
        .create_feedback(request)
        .await
        .expect("Failed to create categorical feedback");

    assert_eq!(feedback.key, "correctness");
    assert_eq!(feedback.value, Some(json!("Y")));
    assert_eq!(feedback.id, feedback_id);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_feedback() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();
    let response_body = make_feedback_json(&feedback_id.to_string(), "helpfulness", 0.8);

    let mock = server
        .mock("GET", format!("/api/v1/feedback/{}", feedback_id).as_str())
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let feedback = client
        .get_feedback(feedback_id)
        .await
        .expect("Failed to get feedback");

    assert_eq!(feedback.key, "helpfulness");
    assert_eq!(feedback.score, Some(0.8));
    assert_eq!(feedback.id, feedback_id);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_feedback() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();
    let response_body = make_feedback_json(&feedback_id.to_string(), "accuracy", 0.99);

    let mock = server
        .mock(
            "PATCH",
            format!("/api/v1/feedback/{}", feedback_id).as_str(),
        )
        .match_body(Matcher::PartialJson(json!({
            "score": 0.99,
            "comment": "Updated score"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let update = FeedbackUpdate {
        score: Some(0.99),
        comment: Some("Updated score".to_string()),
        ..Default::default()
    };

    let feedback = client
        .update_feedback(feedback_id, update)
        .await
        .expect("Failed to update feedback");

    assert_eq!(feedback.score, Some(0.99));
    assert_eq!(feedback.id, feedback_id);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_feedback() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();

    let mock = server
        .mock(
            "DELETE",
            format!("/api/v1/feedback/{}", feedback_id).as_str(),
        )
        .with_status(204)
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    client
        .delete_feedback(feedback_id)
        .await
        .expect("Failed to delete feedback");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_feedback_by_run() {
    let mut server = Server::new_async().await;

    let run_id = Uuid::new_v4();
    let feedback_id_1 = Uuid::new_v4();
    let feedback_id_2 = Uuid::new_v4();

    let response_body = json!([
        make_feedback_json(&feedback_id_1.to_string(), "accuracy", 0.95),
        make_feedback_json(&feedback_id_2.to_string(), "helpfulness", 0.85)
    ]);

    let mock = server
        .mock("GET", "/api/v1/feedback")
        .match_query(Matcher::UrlEncoded("run".to_string(), run_id.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let feedback_list = client
        .list_feedback(Some(run_id))
        .await
        .expect("Failed to list feedback");

    assert_eq!(feedback_list.len(), 2);
    assert_eq!(feedback_list[0].key, "accuracy");
    assert_eq!(feedback_list[1].key, "helpfulness");

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Evaluation Result Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_evaluation_result_as_feedback() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let response_body = json!({
        "id": feedback_id.to_string(),
        "key": "exact_match",
        "score": 1.0,
        "comment": "Output matches expected",
        "run_id": run_id.to_string(),
        "created_at": "2024-01-01T12:00:00Z",
        "modified_at": "2024-01-01T12:00:00Z"
    });

    let mock = server
        .mock("POST", "/api/v1/feedback")
        .match_body(Matcher::PartialJson(json!({
            "key": "exact_match",
            "score": 1.0,
            "comment": "Output matches expected",
            "run_id": run_id.to_string()
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    // Create evaluation result as feedback
    let request = FeedbackCreate {
        key: "exact_match".to_string(),
        score: Some(1.0),
        comment: Some("Output matches expected".to_string()),
        run_id: Some(run_id),
        ..Default::default()
    };

    let feedback = client
        .create_feedback(request)
        .await
        .expect("Failed to create evaluation result");

    assert_eq!(feedback.key, "exact_match");
    assert_eq!(feedback.score, Some(1.0));
    assert_eq!(
        feedback.comment,
        Some("Output matches expected".to_string())
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_all_feedback() {
    let mut server = Server::new_async().await;

    let feedback_id_1 = Uuid::new_v4();
    let feedback_id_2 = Uuid::new_v4();

    let response_body = json!([
        make_feedback_json(&feedback_id_1.to_string(), "accuracy", 0.95),
        make_feedback_json(&feedback_id_2.to_string(), "helpfulness", 0.85)
    ]);

    let mock = server
        .mock("GET", "/api/v1/feedback")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let feedback_list = client
        .list_feedback(None)
        .await
        .expect("Failed to list all feedback");

    assert_eq!(feedback_list.len(), 2);
    assert_eq!(feedback_list[0].key, "accuracy");
    assert_eq!(feedback_list[1].key, "helpfulness");

    mock.assert_async().await;
}

// ═══════════════════════════════════════════════════════════════════════════
// Error Handling Tests
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_feedback_not_found() {
    let mut server = Server::new_async().await;

    let run_id = Uuid::new_v4();

    let mock = server
        .mock("POST", "/api/v1/feedback")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"detail": "Run not found"}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let request = FeedbackCreate {
        key: "accuracy".to_string(),
        score: Some(0.95),
        run_id: Some(run_id),
        ..Default::default()
    };

    let result = client.create_feedback(request).await;
    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_feedback_not_found() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();

    let mock = server
        .mock("GET", format!("/api/v1/feedback/{}", feedback_id).as_str())
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(json!({"detail": "Feedback not found"}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let result = client.get_feedback(feedback_id).await;
    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_feedback_validation_error() {
    let mut server = Server::new_async().await;

    let feedback_id = Uuid::new_v4();

    let mock = server
        .mock(
            "PATCH",
            format!("/api/v1/feedback/{}", feedback_id).as_str(),
        )
        .with_status(422)
        .with_header("content-type", "application/json")
        .with_body(json!({"detail": "Invalid score value"}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());

    let update = FeedbackUpdate {
        score: Some(999.0), // Invalid score
        ..Default::default()
    };

    let result = client.update_feedback(feedback_id, update).await;
    assert!(result.is_err());

    mock.assert_async().await;
}
