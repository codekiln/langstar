/// Integration tests for structured prompt push/pull with real LangSmith API
///
/// These tests require:
/// - LANGSMITH_API_KEY environment variable
/// - LANGSMITH_ORGANIZATION_ID environment variable (or auto-discovery)
/// - Test repository: codekiln/langstar-structured-test (auto-created if it does not exist)
///
/// Run with: cargo test --test structured_prompts_integration_test -- --ignored --nocapture
use langstar_sdk::prompts::{
    LcJson, MessagePromptTemplateKwargs, PromptTemplateKwargs, StructuredOutputKwargs,
    StructuredPrompt,
};
use langstar_sdk::{AuthConfig, LangchainClient};
use serde_json::json;

const TEST_OWNER: &str = "codekiln";
const TEST_REPO: &str = "langstar-structured-test";

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

/// Helper to create a test structured prompt with movie review schema
fn create_test_movie_review_prompt() -> StructuredPrompt {
    let schema = json!({
        "type": "object",
        "title": "MovieReview",
        "description": "A structured movie review",
        "properties": {
            "title": {
                "type": "string",
                "description": "The movie title"
            },
            "rating": {
                "type": "integer",
                "minimum": 1,
                "maximum": 10,
                "description": "Rating from 1-10"
            },
            "summary": {
                "type": "string",
                "description": "Brief review summary"
            }
        },
        "required": ["title", "rating", "summary"]
    });

    let system_prompt_kwargs = PromptTemplateKwargs {
        input_variables: vec![],
        template: "You are a movie critic. Provide a structured review.".to_string(),
        template_format: "f-string".to_string(),
    };

    let system_prompt = LcJson::new(
        vec![
            "langchain_core".to_string(),
            "prompts".to_string(),
            "prompt".to_string(),
            "PromptTemplate".to_string(),
        ],
        system_prompt_kwargs,
    );

    let system_message = LcJson::new(
        vec![
            "langchain_core".to_string(),
            "prompts".to_string(),
            "chat".to_string(),
            "SystemMessagePromptTemplate".to_string(),
        ],
        MessagePromptTemplateKwargs {
            prompt: system_prompt,
        },
    );

    let human_prompt_kwargs = PromptTemplateKwargs {
        input_variables: vec!["movie_name".to_string()],
        template: "Review the movie: {movie_name}".to_string(),
        template_format: "f-string".to_string(),
    };

    let human_prompt = LcJson::new(
        vec![
            "langchain_core".to_string(),
            "prompts".to_string(),
            "prompt".to_string(),
            "PromptTemplate".to_string(),
        ],
        human_prompt_kwargs,
    );

    let human_message = LcJson::new(
        vec![
            "langchain_core".to_string(),
            "prompts".to_string(),
            "chat".to_string(),
            "HumanMessagePromptTemplate".to_string(),
        ],
        MessagePromptTemplateKwargs {
            prompt: human_prompt,
        },
    );

    StructuredPrompt {
        input_variables: Some(vec!["movie_name".to_string()]),
        messages: vec![system_message, human_message],
        schema_: schema,
        structured_output_kwargs: StructuredOutputKwargs {
            method: "json_schema".to_string(),
        },
    }
}

#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_push_structured_prompt_integration() {
    let client = create_integration_test_client().await;

    println!(
        "\n=== Testing push_structured_prompt to {}/{} ===",
        TEST_OWNER, TEST_REPO
    );

    // Create test repository if it doesn't exist
    let repo_handle = format!("{}/{}", TEST_OWNER, TEST_REPO);
    match client.prompts().get(&repo_handle).await {
        Ok(_) => {
            println!("✓ Repository exists");
        }
        Err(_) => {
            println!("Creating repository...");
            match client
                .prompts()
                .create_repo(
                    &repo_handle,
                    Some("Test repository for structured prompts".to_string()),
                    None,
                    false,
                    Some(vec!["test".to_string(), "structured".to_string()]),
                )
                .await
            {
                Ok(_) => println!("✓ Repository created"),
                Err(e) => eprintln!("⚠ Could not create repository: {:?}", e),
            }
        }
    }

    let structured_prompt = create_test_movie_review_prompt();

    println!("Pushing structured prompt with json_schema method...");
    let result = client
        .prompts()
        .push_structured_prompt(TEST_OWNER, TEST_REPO, structured_prompt, None)
        .await;

    assert!(result.is_ok(), "Push should succeed: {:?}", result.err());

    let response = result.unwrap();
    println!("✓ Pushed successfully!");
    println!("  Commit hash: {}", response.commit.commit_hash);
    if let Some(url) = &response.commit.url {
        println!("  URL: {}", url);
    }

    assert!(!response.commit.commit_hash.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_pull_structured_prompt_integration() {
    let client = create_integration_test_client().await;

    println!(
        "\n=== Testing pull_structured_prompt from {}/{} ===",
        TEST_OWNER, TEST_REPO
    );

    println!("Pulling latest commit...");
    let result = client
        .prompts()
        .pull_structured_prompt(TEST_OWNER, TEST_REPO, "latest")
        .await;

    assert!(result.is_ok(), "Pull should succeed: {:?}", result.err());

    let structured_prompt = result.unwrap();
    println!("✓ Pulled successfully!");

    // Verify structure
    assert!(
        structured_prompt.schema_.is_object(),
        "Schema should be an object"
    );
    assert!(
        !structured_prompt.messages.is_empty(),
        "Should have messages"
    );

    println!(
        "  Method: {}",
        structured_prompt.structured_output_kwargs.method
    );
    println!("  Messages count: {}", structured_prompt.messages.len());

    if let Some(vars) = &structured_prompt.input_variables {
        println!("  Input variables: {:?}", vars);
    }

    // Verify schema has expected structure
    let schema = &structured_prompt.schema_;
    assert!(
        schema.get("type").is_some(),
        "Schema should have type field"
    );
    assert!(
        schema.get("properties").is_some(),
        "Schema should have properties field"
    );
}

#[tokio::test]
#[ignore]
async fn test_structured_prompt_round_trip_integration() {
    let client = create_integration_test_client().await;

    println!("\n=== Testing round-trip (push then pull) ===");

    // Step 1: Push a new structured prompt
    let original_prompt = create_test_movie_review_prompt();
    let original_schema = original_prompt.schema_.clone();

    println!("Pushing structured prompt...");
    let push_result = client
        .prompts()
        .push_structured_prompt(TEST_OWNER, TEST_REPO, original_prompt.clone(), None)
        .await;

    assert!(push_result.is_ok(), "Push should succeed");
    let commit_hash = push_result.unwrap().commit.commit_hash;
    println!("✓ Pushed with commit hash: {}", commit_hash);

    // Step 2: Pull it back immediately using the commit hash
    println!("Pulling back using commit hash...");
    let pull_result = client
        .prompts()
        .pull_structured_prompt(TEST_OWNER, TEST_REPO, &commit_hash)
        .await;

    assert!(pull_result.is_ok(), "Pull should succeed");
    let pulled_prompt = pull_result.unwrap();
    println!("✓ Pulled successfully");

    // Step 3: Verify data integrity
    println!("Verifying data integrity...");

    assert_eq!(
        pulled_prompt.schema_, original_schema,
        "Schema should match"
    );
    assert_eq!(
        pulled_prompt.structured_output_kwargs.method,
        original_prompt.structured_output_kwargs.method,
        "Method should match"
    );
    assert_eq!(
        pulled_prompt.input_variables, original_prompt.input_variables,
        "Input variables should match"
    );
    assert_eq!(
        pulled_prompt.messages.len(),
        original_prompt.messages.len(),
        "Message count should match"
    );

    println!("✓ Round-trip successful - all fields match!");
}

#[tokio::test]
#[ignore]
async fn test_push_function_calling_method() {
    let client = create_integration_test_client().await;

    println!("\n=== Testing push with function_calling method ===");

    let mut structured_prompt = create_test_movie_review_prompt();
    structured_prompt.structured_output_kwargs.method = "function_calling".to_string();

    println!("Pushing structured prompt with function_calling method...");
    let result = client
        .prompts()
        .push_structured_prompt(TEST_OWNER, TEST_REPO, structured_prompt, None)
        .await;

    assert!(
        result.is_ok(),
        "Push with function_calling should succeed: {:?}",
        result.err()
    );

    let response = result.unwrap();
    println!("✓ Pushed successfully with function_calling method!");
    println!("  Commit hash: {}", response.commit.commit_hash);

    // Pull it back and verify method
    println!("Verifying method was persisted...");
    let pull_result = client
        .prompts()
        .pull_structured_prompt(TEST_OWNER, TEST_REPO, &response.commit.commit_hash)
        .await;

    assert!(pull_result.is_ok());
    let pulled_prompt = pull_result.unwrap();
    assert_eq!(
        pulled_prompt.structured_output_kwargs.method, "function_calling",
        "Method should be function_calling"
    );
    println!("✓ Method verified as function_calling!");
}
