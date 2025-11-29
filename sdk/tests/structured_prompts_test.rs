/// Tests for structured prompt SDK client methods with mocked HTTP responses
///
/// These tests verify the behavior of `push_structured_prompt` and `pull_structured_prompt`
/// without making real API calls by using mockito to simulate the LangSmith API.
use langstar_sdk::prompts::{
    LcJson, MessagePromptTemplateKwargs, PromptTemplateKwargs, StructuredOutputKwargs,
    StructuredPrompt,
};
use langstar_sdk::{AuthConfig, LangchainClient};
use mockito::{Server, ServerGuard};
use serde_json::json;

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

/// Helper to create a test structured prompt
fn create_test_structured_prompt() -> StructuredPrompt {
    let schema = json!({
        "type": "object",
        "properties": {
            "answer": {"type": "string"},
            "confidence": {"type": "number"}
        },
        "required": ["answer"]
    });

    let prompt_template_kwargs = PromptTemplateKwargs {
        input_variables: vec!["question".to_string()],
        template: "Answer this question: {question}".to_string(),
        template_format: "f-string".to_string(),
    };

    let prompt_template = LcJson::new(
        vec![
            "langchain_core".to_string(),
            "prompts".to_string(),
            "prompt".to_string(),
            "PromptTemplate".to_string(),
        ],
        prompt_template_kwargs,
    );

    let message_kwargs = MessagePromptTemplateKwargs {
        prompt: prompt_template,
    };

    let message = LcJson::new(
        vec![
            "langchain_core".to_string(),
            "prompts".to_string(),
            "chat".to_string(),
            "HumanMessagePromptTemplate".to_string(),
        ],
        message_kwargs,
    );

    StructuredPrompt {
        input_variables: Some(vec!["question".to_string()]),
        messages: vec![message],
        schema_: schema,
        structured_output_kwargs: StructuredOutputKwargs {
            method: "json_schema".to_string(),
        },
    }
}

#[tokio::test]
async fn test_push_structured_prompt_success() {
    let mut server = Server::new_async().await;

    // Mock successful POST to /api/v1/commits/{owner}/{repo}
    let mock = server
        .mock("POST", "/api/v1/commits/test-owner/test-repo")
        .match_header("x-api-key", "test-api-key")
        .match_header("x-organization-id", "test-org-id")
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            json!({
                "commit": {
                    "commit_hash": "abc123",
                    "url": "https://smith.langchain.com/prompts/test-owner/test-repo?commit=abc123"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);
    let structured_prompt = create_test_structured_prompt();

    let result = client
        .prompts()
        .push_structured_prompt("test-owner", "test-repo", structured_prompt, None)
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.commit.commit_hash, "abc123");
}

#[tokio::test]
async fn test_push_structured_prompt_invalid_schema() {
    let server = Server::new_async().await;
    let client = create_test_client(&server);

    // Create StructuredPrompt with invalid schema
    let mut invalid_prompt = create_test_structured_prompt();
    invalid_prompt.schema_ = json!({"type": "invalid_type"});

    let result = client
        .prompts()
        .push_structured_prompt("test-owner", "test-repo", invalid_prompt, None)
        .await;

    // Should fail with InvalidSchemaError before making any API call
    assert!(result.is_err());
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(err_str.contains("InvalidSchemaError") || err_str.contains("Schema validation failed"));
}

#[tokio::test]
async fn test_push_structured_prompt_invalid_method() {
    let server = Server::new_async().await;
    let client = create_test_client(&server);

    // Create StructuredPrompt with invalid method
    let mut invalid_prompt = create_test_structured_prompt();
    invalid_prompt.structured_output_kwargs.method = "invalid_method".to_string();

    let result = client
        .prompts()
        .push_structured_prompt("test-owner", "test-repo", invalid_prompt, None)
        .await;

    // Should fail with InvalidMethodError before making any API call
    assert!(result.is_err());
    let err_str = format!("{:?}", result.unwrap_err());
    assert!(err_str.contains("InvalidMethodError") || err_str.contains("invalid_method"));
}

#[tokio::test]
async fn test_push_structured_prompt_function_calling_method() {
    let mut server = Server::new_async().await;

    // Mock successful POST for function_calling method
    let mock = server
        .mock("POST", "/api/v1/commits/test-owner/test-repo")
        .match_header("x-api-key", "test-api-key")
        .match_header("x-organization-id", "test-org-id")
        .with_status(200)
        .with_body(
            json!({
                "commit": {
                    "commit_hash": "def456",
                    "url": "https://smith.langchain.com/prompts/test-owner/test-repo?commit=def456"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);
    let mut structured_prompt = create_test_structured_prompt();
    structured_prompt.structured_output_kwargs.method = "function_calling".to_string();

    let result = client
        .prompts()
        .push_structured_prompt("test-owner", "test-repo", structured_prompt, None)
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.commit.commit_hash, "def456");
}

#[tokio::test]
async fn test_pull_structured_prompt_success() {
    let mut server = Server::new_async().await;

    // Create LC-JSON formatted StructuredPrompt for mock response
    let schema = json!({
        "type": "object",
        "properties": {
            "answer": {"type": "string"}
        },
        "required": ["answer"]
    });

    let mock_response = json!({
        "manifest": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain_core", "prompts", "structured", "StructuredPrompt"],
            "name": "StructuredPrompt",
            "kwargs": {
                "input_variables": ["question"],
                "messages": [
                    {
                        "lc": 1,
                        "type": "constructor",
                        "id": ["langchain_core", "prompts", "chat", "HumanMessagePromptTemplate"],
                        "kwargs": {
                            "prompt": {
                                "lc": 1,
                                "type": "constructor",
                                "id": ["langchain_core", "prompts", "prompt", "PromptTemplate"],
                                "kwargs": {
                                    "input_variables": ["question"],
                                    "template": "Answer: {question}",
                                    "template_format": "f-string"
                                }
                            }
                        }
                    }
                ],
                "schema_": schema,
                "structured_output_kwargs": {
                    "method": "json_schema"
                }
            }
        }
    });

    let mock = server
        .mock("GET", "/api/v1/commits/test-owner/test-repo/latest")
        .match_header("x-api-key", "test-api-key")
        .match_header("x-organization-id", "test-org-id")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(mock_response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server);

    let result = client
        .prompts()
        .pull_structured_prompt("test-owner", "test-repo", "latest")
        .await;

    mock.assert_async().await;
    assert!(result.is_ok());

    let structured_prompt = result.unwrap();
    assert_eq!(structured_prompt.schema_, schema);
    assert_eq!(
        structured_prompt.structured_output_kwargs.method,
        "json_schema"
    );
    assert_eq!(
        structured_prompt.input_variables,
        Some(vec!["question".to_string()])
    );
}

#[tokio::test]
async fn test_pull_structured_prompt_deserialization_error() {
    let mut server = Server::new_async().await;

    // Mock API returns invalid JSON structure (missing required fields)
    let invalid_response = json!({
        "manifest": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain_core", "prompts", "structured", "StructuredPrompt"],
            "kwargs": {
                // Missing required fields like messages, schema_, etc.
                "invalid_field": "invalid"
            }
        }
    });

    let mock = server
        .mock("GET", "/api/v1/commits/test-owner/test-repo/latest")
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(invalid_response.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server);

    let result = client
        .prompts()
        .pull_structured_prompt("test-owner", "test-repo", "latest")
        .await;

    mock.assert_async().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_structured_prompt_round_trip_mock() {
    let mut server = Server::new_async().await;

    // Step 1: Mock push
    let push_mock = server
        .mock("POST", "/api/v1/commits/test-owner/test-repo")
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(
            json!({
                "commit": {
                    "commit_hash": "test-commit-123",
                    "url": "https://smith.langchain.com/prompts/test-owner/test-repo?commit=test-commit-123"
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = create_test_client(&server);
    let original_prompt = create_test_structured_prompt();
    let original_schema = original_prompt.schema_.clone();

    // Push
    let push_result = client
        .prompts()
        .push_structured_prompt("test-owner", "test-repo", original_prompt.clone(), None)
        .await;

    push_mock.assert_async().await;
    assert!(push_result.is_ok());
    let commit_hash = push_result.unwrap().commit.commit_hash;

    // Step 2: Mock pull (simulate API returning what we just pushed)
    let wrapped_prompt = original_prompt.to_lc_json();
    let pull_response = json!({
        "manifest": serde_json::to_value(&wrapped_prompt).unwrap()
    });

    let pull_mock = server
        .mock(
            "GET",
            format!("/api/v1/commits/test-owner/test-repo/{}", commit_hash).as_str(),
        )
        .match_header("x-api-key", "test-api-key")
        .with_status(200)
        .with_body(pull_response.to_string())
        .create_async()
        .await;

    // Pull
    let pull_result = client
        .prompts()
        .pull_structured_prompt("test-owner", "test-repo", &commit_hash)
        .await;

    pull_mock.assert_async().await;
    assert!(pull_result.is_ok());

    let pulled_prompt = pull_result.unwrap();

    // Verify round-trip data integrity
    assert_eq!(pulled_prompt.schema_, original_schema);
    assert_eq!(pulled_prompt.structured_output_kwargs.method, "json_schema");
    assert_eq!(
        pulled_prompt.input_variables,
        Some(vec!["question".to_string()])
    );
    assert_eq!(pulled_prompt.messages.len(), 1);
}
