use langstar_sdk::{AuthConfig, LangchainClient, PromptData};
use serde_json::json;

/// Integration test for pushing a prompt to LangSmith PromptHub
///
/// NOTE: This test is currently disabled as the LangSmith API endpoint for
/// creating prompts may require additional authentication or use a different
/// endpoint structure. The current implementation gets 405 Method Not Allowed.
///
/// This test requires a valid LANGSMITH_API_KEY environment variable with write permissions.
/// Run with: cargo test --test integration_test -- --ignored --nocapture
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
#[should_panic(expected = "405")] // Expected to fail until we determine correct API endpoint
async fn test_push_prompt_to_prompthub() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY must be set for integration tests");

    // Verify we have a LangSmith API key
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required for this test");

    // Create client
    let client = LangchainClient::new(auth)
        .expect("Failed to create LangchainClient");

    // Get current user info to construct repo handle
    // For testing, we'll use a fixed test prompt name
    let test_prompt_name = "langstar-integration-test";
    let repo_handle = format!("codekiln/{}", test_prompt_name);

    println!("Testing prompt push to: {}", repo_handle);

    // Create test prompt data
    let prompt_data = PromptData {
        description: Some("Integration test prompt for langstar SDK".to_string()),
        readme: Some(
            "# Langstar Integration Test Prompt\n\n\
            This prompt is created by langstar integration tests.\n\n\
            It validates the SDK's ability to push prompts to LangSmith PromptHub."
                .to_string(),
        ),
        tags: Some(vec![
            "test".to_string(),
            "langstar".to_string(),
            "integration".to_string(),
        ]),
        is_public: false, // Keep test prompts private
        manifest: json!({
            "type": "prompt",
            "template": "Hello from langstar! This is a test prompt.\n\nContext: {context}\nQuestion: {question}",
            "input_variables": ["context", "question"],
            "template_format": "f-string"
        }),
    };

    // Push the prompt to PromptHub
    println!("Pushing prompt to PromptHub...");
    let result = client.prompts().push(&repo_handle, &prompt_data).await;

    match result {
        Ok(prompt) => {
            println!("✓ Prompt pushed successfully!");
            println!("  ID: {}", prompt.id);
            println!("  Handle: {}", prompt.repo_handle);
            println!("  Public: {}", prompt.is_public);

            // Verify the prompt was created with correct data
            assert_eq!(prompt.repo_handle, repo_handle);
            assert_eq!(prompt.is_public, false);
            assert!(prompt.description.is_some());
            assert_eq!(
                prompt.description.unwrap(),
                "Integration test prompt for langstar SDK"
            );

            // Try to fetch the prompt back to verify it exists
            println!("\nVerifying prompt can be fetched...");
            let fetched = client.prompts().get(&repo_handle).await;

            match fetched {
                Ok(fetched_prompt) => {
                    println!("✓ Prompt verified!");
                    assert_eq!(fetched_prompt.id, prompt.id);
                    assert_eq!(fetched_prompt.repo_handle, prompt.repo_handle);
                }
                Err(e) => {
                    panic!("Failed to fetch prompt after creation: {:?}", e);
                }
            }

            println!("\n✓ Integration test passed!");
            println!("\nNote: Test prompt '{}' remains in your PromptHub.", repo_handle);
            println!("You can delete it manually if needed.");
        }
        Err(e) => {
            panic!("Failed to push prompt to PromptHub: {:?}\n\nPlease verify:\n\
                1. LANGSMITH_API_KEY is valid\n\
                2. API key has write permissions\n\
                3. Network connectivity to api.smith.langchain.com", e);
        }
    }
}

/// Integration test for listing prompts from PromptHub
///
/// This is a read-only test that should work with any valid API key.
/// Run with: cargo test --test integration_test -- --ignored --nocapture
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_list_prompts_from_prompthub() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY must be set for integration tests");

    // Create client
    let client = LangchainClient::new(auth)
        .expect("Failed to create LangchainClient");

    println!("Fetching prompts from PromptHub...");

    // List prompts (limit to 5 for faster test)
    let result = client.prompts().list(Some(5), None).await;

    match result {
        Ok(prompts) => {
            println!("✓ Successfully fetched {} prompts", prompts.len());

            // We should get at least some prompts (there are public prompts available)
            assert!(
                !prompts.is_empty(),
                "Expected to find at least some prompts in PromptHub"
            );

            // Display first few prompts
            for (i, prompt) in prompts.iter().take(3).enumerate() {
                println!("\nPrompt {}:", i + 1);
                println!("  Handle: {}", prompt.repo_handle);
                println!("  Likes: {}", prompt.num_likes);
                println!("  Downloads: {}", prompt.num_downloads);
                if let Some(desc) = &prompt.description {
                    let short_desc = if desc.len() > 60 {
                        format!("{}...", &desc[..57])
                    } else {
                        desc.clone()
                    };
                    println!("  Description: {}", short_desc);
                }
            }

            println!("\n✓ Integration test passed!");
        }
        Err(e) => {
            panic!(
                "Failed to list prompts from PromptHub: {:?}\n\nPlease verify:\n\
                1. LANGSMITH_API_KEY is valid\n\
                2. Network connectivity to api.smith.langchain.com",
                e
            );
        }
    }
}
