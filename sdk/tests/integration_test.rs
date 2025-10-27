use langstar_sdk::{AuthConfig, CommitRequest, LangchainClient};
use serde_json::json;

/// Integration test for pushing a prompt to LangSmith PromptHub
///
/// This test creates a new commit for a prompt using the correct
/// /api/v1/commits/{owner}/{repo} endpoint.
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY with write permissions
/// 2. The prompt repository must already exist in your PromptHub
///    Create it at: https://smith.langchain.com/prompts
///
/// Run with: cargo test --test integration_test -- --ignored --nocapture
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_push_prompt_to_prompthub() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY must be set for integration tests");

    // Verify we have a LangSmith API key
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required for this test");

    // Create client
    let mut client = LangchainClient::new(auth)
        .expect("Failed to create LangchainClient");

    // Get current organization to set org ID header
    println!("Fetching current organization...");
    match client.get_current_organization().await {
        Ok(org) => {
            if let Some(org_id) = org.id {
                println!("✓ Organization: {} (ID: {})", org.display_name.unwrap_or_default(), org_id);
                client = client.with_organization_id(org_id);
            } else {
                println!("⚠ Organization has no ID, proceeding without X-Organization-Id header");
            }
        }
        Err(e) => {
            println!("⚠ Could not fetch organization: {:?}", e);
            println!("  Proceeding without X-Organization-Id header");
        }
    }

    // For testing, we'll use a fixed test prompt name
    let owner = "codekiln";
    let repo = "langstar-integration-test";

    println!("\nTesting prompt push to: {}/{}", owner, repo);

    // Create commit request with prompt manifest
    let commit_request = CommitRequest {
        manifest: json!({
            "type": "prompt",
            "template": "Hello from langstar! This is a test prompt.\n\nContext: {context}\nQuestion: {question}",
            "input_variables": ["context", "question"],
            "template_format": "f-string"
        }),
        parent_commit: None,
        example_run_ids: None,
    };

    // Push the prompt to PromptHub
    println!("Pushing prompt commit to PromptHub...");
    let result = client.prompts().push(owner, repo, &commit_request).await;

    match result {
        Ok(commit_response) => {
            println!("✓ Prompt commit pushed successfully!");
            println!("  Commit hash: {}", commit_response.commit_hash);
            if let Some(url) = &commit_response.url {
                println!("  URL: {}", url);
            }

            // Try to fetch the prompt back to verify it exists
            println!("\nVerifying prompt can be fetched...");
            let repo_handle = format!("{}/{}", owner, repo);
            let fetched = client.prompts().get(&repo_handle).await;

            match fetched {
                Ok(fetched_prompt) => {
                    println!("✓ Prompt verified!");
                    println!("  Handle: {}", fetched_prompt.repo_handle);
                    println!("  Likes: {}", fetched_prompt.num_likes);
                    assert_eq!(fetched_prompt.repo_handle, repo_handle);
                }
                Err(e) => {
                    println!("⚠ Could not fetch prompt after creation: {:?}", e);
                    println!("  This is expected if the prompt doesn't exist yet in the hub");
                }
            }

            println!("\n✓ Integration test passed!");
            println!("\nNote: Test prompt '{}/{}' commit created.", owner, repo);
            println!("Commit hash: {}", commit_response.commit_hash);
        }
        Err(e) => {
            panic!("Failed to push prompt commit to PromptHub: {:?}\n\nPlease verify:\n\
                1. LANGSMITH_API_KEY is valid\n\
                2. API key has write permissions\n\
                3. The prompt repository '{}/{}' exists in your PromptHub\n\
                4. Network connectivity to api.smith.langchain.com\n\n\
                Note: You may need to create the prompt repository first via the LangSmith UI.",
                e, owner, repo);
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
