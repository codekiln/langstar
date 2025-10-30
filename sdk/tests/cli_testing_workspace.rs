use langstar_sdk::{AuthConfig, CommitRequest, LangchainClient};
use serde_json::json;

/// Integration tests for the CLI Testing Workspace in LangSmith
///
/// These tests demonstrate end-to-end prompt management:
/// 1. Create a new prompt repository
/// 2. Push a commit to it
/// 3. Read it back
///
/// Prerequisites:
/// - LANGSMITH_API_KEY must be set with write access to the workspace
/// - Organization ID: 6f52dd84-9870-4f3a-b42d-4eea5fc9dfde
///
/// Run with: cargo test --test cli_testing_workspace -- --ignored --nocapture
const TEST_ORG_ID: &str = "6f52dd84-9870-4f3a-b42d-4eea5fc9dfde";

/// Test: Create a new prompt, push a commit, and read it back
#[tokio::test]
#[ignore]
async fn test_create_push_and_read_prompt() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   LangSmith CLI Testing Workspace Integration Test      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Load authentication from environment
    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set for integration tests");

    // Create client with organization ID
    let client = LangchainClient::new(auth)
        .expect("Failed to create LangchainClient")
        .with_organization_id(TEST_ORG_ID.to_string());

    // Generate unique prompt name using timestamp
    let timestamp = chrono::Utc::now().timestamp();
    let owner = "cli-testing"; // Try using a simple owner name
    let repo_name = format!("langstar-test-{}", timestamp);
    let full_repo_handle = format!("{}-{}", owner, repo_name); // Use hyphen instead of slash for creation

    println!("=== Step 1: Creating new prompt repository ===");
    println!("Repository: {}", full_repo_handle);

    let create_result = client
        .prompts()
        .create_repo(
            &full_repo_handle,
            Some("Integration test prompt created by langstar".to_string()),
            Some(
                "# Test Prompt\n\nThis prompt was created by langstar integration tests."
                    .to_string(),
            ),
            false, // Private prompt
            Some(vec!["test".to_string(), "langstar".to_string()]),
        )
        .await;

    let created_prompt = match create_result {
        Ok(prompt) => {
            println!("✓ Repository created successfully!");
            println!("  Handle: {}", prompt.repo_handle);
            println!("  ID: {}", prompt.id);
            if let Some(desc) = &prompt.description {
                println!("  Description: {}", desc);
            }
            prompt
        }
        Err(e) => {
            panic!(
                "Failed to create repository: {:?}\n\nPlease verify:\n\
                1. LANGSMITH_API_KEY has write permissions\n\
                2. You have permission to create prompts in organization {}\n\
                3. The owner '{}' is correct for your workspace",
                e, TEST_ORG_ID, owner
            );
        }
    };

    println!("\n=== Step 2: Pushing a commit to the prompt ===");

    let commit_request = CommitRequest {
        manifest: json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
            "kwargs": {
                "input_variables": ["user_input"],
                "template": "You are a helpful AI assistant.\n\nUser: {user_input}\n\nPlease provide a helpful response.",
                "template_format": "f-string"
            }
        }),
        parent_commit: None,
        example_run_ids: None,
    };

    // Parse the repo_handle to get owner and repo name
    // If the handle contains a slash, split it; otherwise use "-" as owner (per LangSmith SDK)
    let (push_owner, push_repo) = if created_prompt.repo_handle.contains('/') {
        let parts: Vec<&str> = created_prompt.repo_handle.split('/').collect();
        (parts[0], parts[1])
    } else {
        // For organization-scoped prompts without explicit owner, use "-"
        // This matches the Python SDK behavior for org-scoped repos
        ("-", created_prompt.repo_handle.as_str())
    };

    println!("Pushing to: {}/{}", push_owner, push_repo);

    let push_result = client
        .prompts()
        .push(push_owner, push_repo, &commit_request)
        .await;

    let first_commit_hash = match push_result {
        Ok(commit_response) => {
            println!("✓ Commit pushed successfully!");
            println!("  Commit hash: {}", commit_response.commit.commit_hash);
            if let Some(url) = &commit_response.commit.url {
                println!("  URL: {}", url);
            }
            commit_response.commit.commit_hash.clone()
        }
        Err(e) => {
            panic!("Failed to push commit: {:?}", e);
        }
    };

    println!("\n=== Step 3: Reading the prompt back ===");

    // Use the repo_handle returned from the create operation
    // If it doesn't have a slash, prepend "-/" for organization-scoped prompts
    let read_handle = if created_prompt.repo_handle.contains('/') {
        created_prompt.repo_handle.clone()
    } else {
        format!("-/{}", created_prompt.repo_handle)
    };

    println!("Reading prompt with handle: {}", read_handle);
    let read_result = client.prompts().get(&read_handle).await;

    match read_result {
        Ok(prompt) => {
            println!("✓ Prompt read successfully!");
            println!("  Handle: {}", prompt.repo_handle);
            println!("  Likes: {}", prompt.num_likes);
            println!("  Downloads: {}", prompt.num_downloads);

            // Verify it was fetched successfully
            // Note: The repo_handle returned by the API might include the owner
            println!("  Repo handle from API: {}", prompt.repo_handle);
        }
        Err(e) => {
            panic!("Failed to read prompt back: {:?}", e);
        }
    }

    println!("\n=== Step 4: Pushing a second commit (update) ===");

    let update_commit = CommitRequest {
        manifest: json!({
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
            "kwargs": {
                "input_variables": ["user_input"],
                "template": "You are a helpful AI assistant. [UPDATED]\n\nUser: {user_input}\n\nPlease provide a helpful and detailed response.",
                "template_format": "f-string"
            }
        }),
        parent_commit: Some(first_commit_hash.clone()),
        example_run_ids: None,
    };

    let update_result = client
        .prompts()
        .push(push_owner, push_repo, &update_commit)
        .await;

    match update_result {
        Ok(commit_response) => {
            println!("✓ Update commit pushed successfully!");
            println!("  Commit hash: {}", commit_response.commit.commit_hash);
        }
        Err(e) => {
            panic!("Failed to push update commit: {:?}", e);
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          All Integration Tests Passed! ✓                 ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("\nTest prompt created: {}", created_prompt.repo_handle);
    println!("Organization: {}", TEST_ORG_ID);
    println!("\nNote: You may want to delete this test prompt from:");
    println!(
        "https://smith.langchain.com/prompts/{}?organizationId={}",
        repo_name, TEST_ORG_ID
    );
}

/// Test: Try to read an existing prompt (requires knowing the correct handle)
/// This test is separate because it depends on external state (the prompt must exist)
#[tokio::test]
#[ignore]
async fn test_read_existing_test_prompt() {
    println!("\n=== Test: Reading existing test-prompt ===\n");

    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set for integration tests");

    let client = LangchainClient::new(auth)
        .expect("Failed to create LangchainClient")
        .with_organization_id(TEST_ORG_ID.to_string());

    // NOTE: Update this with the correct handle for your test-prompt
    // You can find this in the LangSmith UI
    let repo_handle = "codekiln/test-prompt"; // UPDATE THIS if needed

    println!("Attempting to fetch: {}", repo_handle);

    let result = client.prompts().get(repo_handle).await;

    match result {
        Ok(prompt) => {
            println!("✓ Successfully fetched prompt!");
            println!("  Handle: {}", prompt.repo_handle);
            println!("  Likes: {}", prompt.num_likes);
            println!("  Downloads: {}", prompt.num_downloads);
            if let Some(desc) = &prompt.description {
                println!("  Description: {}", desc);
            }
        }
        Err(e) => {
            println!("✗ Failed to fetch prompt: {:?}", e);
            println!("\nThis might mean:");
            println!("  1. The prompt handle '{}' is incorrect", repo_handle);
            println!("  2. The prompt doesn't exist yet");
            println!("  3. The API key doesn't have access to this prompt");
            println!("\nPlease check the LangSmith UI to find the correct handle.");
            println!("The handle should be in the format 'owner/prompt-name'");
        }
    }
}
