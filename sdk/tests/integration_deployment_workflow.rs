use langstar_sdk::test_utils::{
    DeploymentGuard, TestDeploymentConfig, get_or_create_deployment, wait_for_deployment,
};
use langstar_sdk::{AuthConfig, LangchainClient, PatchDeploymentRequest, RevisionStatus};

/// Integration test for deployment workflow using reusable test deployment
///
/// This test uses a **persistent test deployment** for faster iteration during development.
/// Deployments with the `pr-integration-test` prefix are reused across test runs via
/// the get-or-create pattern.
///
/// **What this test validates:**
/// 1. Get or create test deployment using shared TestDeploymentConfig
/// 2. Verify deployment is ready (RevisionStatus::Deployed)
/// 3. Patch deployment (triggers new revision)
/// 4. Poll new revision status until DEPLOYED
/// 5. Leave deployment running for future test runs (cleaned by cron)
///
/// **Note:** This test does NOT delete the deployment after running. The deployment
/// persists between test runs for faster iteration and is cleaned up by the
/// periodic cleanup workflow after 4 hours.
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY with write permissions
/// 2. Valid LANGSMITH_WORKSPACE_ID
/// 3. GitHub integration configured with access to the target repository
/// 4. Repository must contain tests/fixtures/test-graph-deployment/langgraph.json
///
/// **Environment Variables:**
/// - LANGSMITH_API_KEY: Required
/// - LANGSMITH_WORKSPACE_ID: Required
/// - REPOSITORY_OWNER: Default "codekiln"
/// - REPOSITORY_NAME: Default "langstar"
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow test_deployment_workflow -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_deployment_workflow() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");

    // Verify we have required credentials
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required for this test");

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    // Use shared test deployment config (pr-integration-test-{ts} with prefix-based reuse)
    let config = TestDeploymentConfig::default();
    println!("🚀 Starting deployment workflow test");
    println!("   Config name: {}", config.name);
    println!("   Config prefix: {:?}", config.name_prefix);
    println!();

    // Step 1: Get or create test deployment using shared utility
    println!("📦 Getting or creating deployment...");
    let (deployment_id, revision_id, deployment_name) = get_or_create_deployment(&client, &config)
        .await
        .expect("Failed to get or create deployment");
    println!(
        "✓ Deployment ready: {} ({}, revision: {})",
        deployment_name, deployment_id, revision_id
    );
    println!();

    // Step 2: Get deployment details and validate
    let deployment = client
        .deployments()
        .get(&deployment_id)
        .await
        .expect("Failed to get deployment");

    assert_eq!(
        deployment.source,
        langstar_sdk::DeploymentSource::Github,
        "Deployment source should be Github"
    );
    println!("✓ Validated deployment source: Github");

    if let Some(url) = deployment.custom_url() {
        println!("✓ Deployment URL: {}", url);
    }
    println!();

    // Step 3: Patch deployment (triggers new revision)
    println!("🔧 Patching deployment (triggering new revision)...");
    let patch_request = PatchDeploymentRequest {
        source_config: Some(serde_json::json!({
            "build_on_push": true,
        })),
        source_revision_config: Some(serde_json::json!({
            "repo_ref": "main",
            "langgraph_config_path": "tests/fixtures/test-graph-deployment/langgraph.json",
        })),
    };

    client
        .deployments()
        .patch(&deployment_id, &patch_request)
        .await
        .expect("Failed to patch deployment");
    println!("✓ Deployment patched");
    println!();

    // Step 4: Get new latest revision
    println!("📋 Fetching new revisions...");
    let revisions = client
        .deployments()
        .list_revisions(&deployment_id)
        .await
        .expect("Failed to list revisions after patch");

    let new_latest_revision = &revisions.resources[0];
    let new_revision_id = new_latest_revision.id.clone();
    println!("✓ New latest revision: {}", new_revision_id);
    println!("  Status: {:?}", new_latest_revision.status);
    println!();

    // Step 5: Poll new revision status until DEPLOYED
    println!("⏳ Waiting for new revision to deploy...");
    wait_for_deployment(&client, &deployment_id, &new_revision_id)
        .await
        .expect("New revision failed to deploy");
    println!("✓ New revision deployed successfully!");
    println!();

    // Validate final revision status
    let final_revision = client
        .deployments()
        .get_revision(&deployment_id, &new_revision_id)
        .await
        .expect("Failed to get final revision");
    assert_eq!(
        final_revision.status,
        RevisionStatus::Deployed,
        "Final revision should have Deployed status"
    );
    println!("✓ Validated final revision status: Deployed");
    println!();

    // Note: We do NOT delete the deployment - it's reused across test runs
    println!("💾 Deployment remains active for future test runs");
    println!("   Cleaned up by periodic cleanup workflow (4hr threshold)");
    println!();

    println!("✅ Deployment workflow test completed successfully!");
}

/// Full lifecycle integration test for pre-release validation
///
/// This test performs a **complete create/delete cycle** with a uniquely-named deployment.
/// Uses `TestDeploymentConfig::for_release_tests()` which creates a `release-integration-test-{ts}`
/// deployment that is always fresh (no prefix-based reuse).
///
/// **What this test validates:**
/// 1. Create fresh deployment using shared TestDeploymentConfig
/// 2. Verify deployment is ready (RevisionStatus::Deployed)
/// 3. Patch deployment (triggers new revision)
/// 4. Poll new revision status until DEPLOYED
/// 5. Delete deployment (cleanup)
///
/// **Note:** This test creates a NEW deployment every run and cleans up after itself.
/// It's slower but provides full isolation. Use `test_deployment_workflow` for faster
/// iteration during development.
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY with write permissions
/// 2. Valid LANGSMITH_WORKSPACE_ID
/// 3. GitHub integration configured with access to the target repository
/// 4. Repository must contain tests/fixtures/test-graph-deployment/langgraph.json
///
/// **Environment Variables:**
/// - LANGSMITH_API_KEY: Required
/// - LANGSMITH_WORKSPACE_ID: Required
/// - REPOSITORY_OWNER: Default "codekiln"
/// - REPOSITORY_NAME: Default "langstar"
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow test_deployment_workflow_full_lifecycle -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_deployment_workflow_full_lifecycle() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");

    // Verify we have required credentials
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required for this test");

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    // Use release test config (release-integration-test-{ts}, always creates fresh)
    let config = TestDeploymentConfig::for_release_tests();
    println!("🚀 Starting FULL LIFECYCLE deployment workflow test");
    println!("   Config name: {}", config.name);
    println!(
        "   Config prefix: {:?} (None = always create fresh)",
        config.name_prefix
    );
    println!();

    // Step 1: Create fresh deployment using shared utility
    // (for_release_tests has name_prefix: None, so get_or_create always creates)
    println!("📦 Creating fresh deployment...");
    let (deployment_id, revision_id, deployment_name) = get_or_create_deployment(&client, &config)
        .await
        .expect("Failed to create deployment");
    println!(
        "✓ Deployment created: {} ({}, revision: {})",
        deployment_name, deployment_id, revision_id
    );
    println!();

    // Create RAII guard for automatic cleanup on failure
    let mut guard = DeploymentGuard::new(deployment_id.clone());

    // Step 2: Get deployment details and validate
    let deployment = client
        .deployments()
        .get(&deployment_id)
        .await
        .expect("Failed to get deployment");

    assert_eq!(
        deployment.source,
        langstar_sdk::DeploymentSource::Github,
        "Deployment source should be Github"
    );
    println!("✓ Validated deployment source: Github");

    if let Some(url) = deployment.custom_url() {
        println!("✓ Deployment URL: {}", url);
    }
    println!();

    // Step 3: Patch deployment (triggers new revision)
    println!("🔧 Patching deployment (triggering new revision)...");
    let patch_request = PatchDeploymentRequest {
        source_config: Some(serde_json::json!({
            "build_on_push": true,
        })),
        source_revision_config: Some(serde_json::json!({
            "repo_ref": "main",
            "langgraph_config_path": "tests/fixtures/test-graph-deployment/langgraph.json",
        })),
    };

    client
        .deployments()
        .patch(&deployment_id, &patch_request)
        .await
        .expect("Failed to patch deployment");
    println!("✓ Deployment patched");
    println!();

    // Step 4: Get new latest revision
    println!("📋 Fetching new revisions...");
    let revisions = client
        .deployments()
        .list_revisions(&deployment_id)
        .await
        .expect("Failed to list revisions after patch");

    let new_latest_revision = &revisions.resources[0];
    let new_revision_id = new_latest_revision.id.clone();
    println!("✓ New latest revision: {}", new_revision_id);
    println!("  Status: {:?}", new_latest_revision.status);
    println!();

    // Step 5: Poll new revision status until DEPLOYED
    println!("⏳ Waiting for new revision to deploy...");
    wait_for_deployment(&client, &deployment_id, &new_revision_id)
        .await
        .expect("New revision failed to deploy");
    println!("✓ New revision deployed successfully!");
    println!();

    // Validate final revision status
    let final_revision = client
        .deployments()
        .get_revision(&deployment_id, &new_revision_id)
        .await
        .expect("Failed to get final revision");
    assert_eq!(
        final_revision.status,
        RevisionStatus::Deployed,
        "Final revision should have Deployed status"
    );
    println!("✓ Validated final revision status: Deployed");
    println!();

    // Step 6: Delete deployment (cleanup)
    println!("🗑️  Deleting deployment...");
    client
        .deployments()
        .delete(&deployment_id)
        .await
        .expect("Failed to delete deployment");
    println!("✓ Deployment deleted: {}", deployment_id);
    println!();

    // Disarm guard since we manually deleted
    guard.disarm();

    println!("✅ Full lifecycle deployment workflow test completed successfully!");
}

/// Test listing deployments with name filter
///
/// This is a simpler read-only test to verify basic deployment listing works.
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow test_list_deployments -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_list_deployments() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    println!("📋 Listing deployments...");

    // List deployments (limit to 100 to see all test deployments)
    let result = client.deployments().list(Some(100), None, None).await;

    match result {
        Ok(deployments_list) => {
            println!("✓ Successfully fetched deployments");
            println!("  Total returned: {}", deployments_list.resources.len());
            println!("  Offset: {}", deployments_list.offset);

            // Display all deployments
            for (i, deployment) in deployments_list.resources.iter().enumerate() {
                println!("\nDeployment {}:", i + 1);
                println!("  Name: {}", deployment.name);
                println!("  ID: {}", deployment.id);
                println!("  Status: {:?}", deployment.status);
                println!("  Source: {:?}", deployment.source);
                if let Some(url) = deployment.custom_url() {
                    println!("  URL: {}", url);
                }
            }

            println!("\n✅ List deployments test passed!");
        }
        Err(e) => {
            panic!(
                "Failed to list deployments: {:?}\n\nPlease verify:\n\
                1. LANGSMITH_API_KEY is valid\n\
                2. Workspace ID is set correctly\n\
                3. Network connectivity to api.host.langchain.com",
                e
            );
        }
    }
}

/// Test listing GitHub integrations
///
/// This test validates the ability to list all configured GitHub integrations
/// for the workspace. Useful for debugging integration setup issues.
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow test_list_github_integrations -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_list_github_integrations() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    println!("🔍 Listing GitHub integrations...");

    // List all integrations
    let result = client.integrations().list_github_integrations().await;

    match result {
        Ok(integrations) => {
            println!("✓ Successfully fetched GitHub integrations");
            println!("  Total integrations: {}", integrations.len());

            // Display all integrations
            for (i, integration) in integrations.iter().enumerate() {
                println!("\nIntegration {}:", i + 1);
                println!("  ID: {}", integration.id);
                if let Some(name) = &integration.name {
                    println!("  Name: {}", name);
                }
            }

            println!("\n✅ List GitHub integrations test passed!");
        }
        Err(e) => {
            panic!(
                "Failed to list GitHub integrations: {:?}\n\nPlease verify:\n\
                1. LANGSMITH_API_KEY is valid\n\
                2. Workspace ID is set correctly\n\
                3. At least one GitHub integration is configured",
                e
            );
        }
    }
}

/// Test listing GitHub repositories for an integration
///
/// This test validates the ability to list all repositories accessible
/// through a specific GitHub integration. Useful for verifying repository
/// access and debugging permission issues.
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow test_list_github_repositories -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_list_github_repositories() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    println!("🔍 Listing GitHub integrations...");

    // First, get all integrations
    let integrations = client
        .integrations()
        .list_github_integrations()
        .await
        .expect("Failed to list GitHub integrations");

    assert!(
        !integrations.is_empty(),
        "At least one GitHub integration must be configured for this test"
    );

    let integration = &integrations[0];
    println!("✓ Using integration: {}", integration.id);
    if let Some(name) = &integration.name {
        println!("  Name: {}", name);
    }
    println!();

    println!(
        "📚 Listing repositories for integration {}...",
        integration.id
    );

    // List repositories for the first integration
    let result = client
        .integrations()
        .list_github_repositories(&integration.id)
        .await;

    match result {
        Ok(repos) => {
            println!("✓ Successfully fetched repositories");
            println!("  Total repositories: {}", repos.len());

            // Display first 5 repositories
            for (i, repo) in repos.iter().take(5).enumerate() {
                println!("\nRepository {}:", i + 1);
                println!("  Owner: {}", repo.owner);
                println!("  Name: {}", repo.name);
                println!("  Full name: {}/{}", repo.owner, repo.name);
            }

            if repos.len() > 5 {
                println!("\n  ... and {} more repositories", repos.len() - 5);
            }

            println!("\n✅ List GitHub repositories test passed!");
        }
        Err(e) => {
            panic!(
                "Failed to list GitHub repositories: {:?}\n\nPlease verify:\n\
                1. Integration ID {} is valid\n\
                2. Integration has repository access configured\n\
                3. API permissions are sufficient",
                e, integration.id
            );
        }
    }
}

/// Test finding integration for a specific repository
///
/// This test validates the ability to find the correct GitHub integration
/// for a given repository owner and name. This is the key operation used
/// in the deployment workflow to dynamically discover the integration ID.
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow test_find_integration_for_repo -- --ignored --nocapture
/// ```
#[tokio::test]
#[ignore] // Only run when explicitly requested with --ignored flag
async fn test_find_integration_for_repo() {
    // Load authentication from environment
    let auth = AuthConfig::from_env()
        .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");

    // Get repository configuration from environment
    let repository_owner =
        std::env::var("REPOSITORY_OWNER").unwrap_or_else(|_| "codekiln".to_string());
    let repository_name =
        std::env::var("REPOSITORY_NAME").unwrap_or_else(|_| "langstar".to_string());

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    println!(
        "🔍 Finding integration for repository {}/{}...",
        repository_owner, repository_name
    );

    // Find integration for the repository
    let result = client
        .integrations()
        .find_integration_for_repo(&repository_owner, &repository_name)
        .await;

    match result {
        Ok(integration_id) => {
            println!("✓ Successfully found integration for repository");
            println!("  Repository: {}/{}", repository_owner, repository_name);
            println!("  Integration ID: {}", integration_id);

            // Verify the integration ID is a valid UUID
            assert!(
                !integration_id.is_empty(),
                "Integration ID should not be empty"
            );
            assert!(
                integration_id.contains('-'),
                "Integration ID should be a UUID format"
            );

            println!("\n✅ Find integration for repo test passed!");
        }
        Err(e) => {
            panic!(
                "Failed to find integration for repository {}/{}: {:?}\n\nPlease verify:\n\
                1. Repository owner and name are correct\n\
                2. GitHub integration has access to this repository\n\
                3. Integration is properly configured in the workspace",
                repository_owner, repository_name, e
            );
        }
    }
}

/// Unit test for deployment URL extraction from source_config
///
/// This test validates the `custom_url()` helper method that extracts
/// the deployment URL from the source_config JSON. No API calls required.
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow test_deployment_url_extraction
/// ```
#[test]
fn test_deployment_url_extraction() {
    use langstar_sdk::Deployment;
    use serde_json::json;

    println!("🧪 Testing deployment URL extraction...");

    // Test case 1: Deployment with custom_url in source_config
    let deployment_with_url = Deployment {
        id: "test-id-1".to_string(),
        name: "test-deployment".to_string(),
        source: langstar_sdk::DeploymentSource::Github,
        source_config: Some(json!({
            "custom_url": "https://test-deployment.langchain.app",
            "integration_id": "test-integration",
        })),
        source_revision_config: Some(json!({})),
        status: langstar_sdk::DeploymentStatus::Ready,
        secrets: Some(vec![]),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        latest_revision_id: None,
        active_revision_id: None,
        image_version: None,
    };

    let url = deployment_with_url.custom_url();
    assert!(url.is_some(), "Should extract URL from source_config");
    assert_eq!(
        url.unwrap(),
        "https://test-deployment.langchain.app",
        "URL should match the custom_url value"
    );
    println!("✓ Test case 1: URL extraction succeeded");

    // Test case 2: Deployment without custom_url in source_config
    let deployment_without_url = Deployment {
        id: "test-id-2".to_string(),
        name: "test-deployment-2".to_string(),
        source: langstar_sdk::DeploymentSource::Github,
        source_config: Some(json!({
            "integration_id": "test-integration",
        })),
        source_revision_config: Some(json!({})),
        status: langstar_sdk::DeploymentStatus::Ready,
        secrets: Some(vec![]),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        latest_revision_id: None,
        active_revision_id: None,
        image_version: None,
    };

    let url = deployment_without_url.custom_url();
    assert!(
        url.is_none(),
        "Should return None when custom_url is not present"
    );
    println!("✓ Test case 2: Missing URL handled correctly");

    // Test case 3: Deployment with null custom_url
    let deployment_with_null_url = Deployment {
        id: "test-id-3".to_string(),
        name: "test-deployment-3".to_string(),
        source: langstar_sdk::DeploymentSource::Github,
        source_config: Some(json!({
            "custom_url": null,
            "integration_id": "test-integration",
        })),
        source_revision_config: Some(json!({})),
        status: langstar_sdk::DeploymentStatus::Ready,
        secrets: Some(vec![]),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        latest_revision_id: None,
        active_revision_id: None,
        image_version: None,
    };

    let url = deployment_with_null_url.custom_url();
    assert!(url.is_none(), "Should return None when custom_url is null");
    println!("✓ Test case 3: Null URL handled correctly");

    println!("\n✅ Deployment URL extraction test passed!");
}
