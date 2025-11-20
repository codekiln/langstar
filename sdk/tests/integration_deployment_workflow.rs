use langstar_sdk::{
    AuthConfig, CreateDeploymentRequest, LangchainClient, PatchDeploymentRequest, RevisionStatus,
};
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Integration test for the complete deployment workflow
///
/// This test validates the full lifecycle of a LangGraph deployment:
/// 1. Find GitHub integration ID dynamically
/// 2. Create deployment with unique timestamp-based name
/// 3. List revisions and get the latest one
/// 4. Poll revision status until DEPLOYED (60s interval, 30min timeout)
/// 5. Patch deployment (triggers new revision)
/// 6. Poll new revision status until DEPLOYED
/// 7. Delete deployment (cleanup)
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY with write permissions
/// 2. Valid LANGSMITH_WORKSPACE_ID (or LANGCHAIN_WORKSPACE_ID)
/// 3. GitHub integration configured with access to the target repository
/// 4. Repository must contain tests/fixtures/test-graph-deployment/langgraph.json
///
/// **Environment Variables:**
/// - LANGSMITH_API_KEY: Required
/// - LANGSMITH_WORKSPACE_ID or LANGCHAIN_WORKSPACE_ID: Required
/// - REPOSITORY_OWNER: Default "codekiln"
/// - REPOSITORY_NAME: Default "langstar"
///
/// Run with:
/// ```bash
/// cargo test --test integration_deployment_workflow -- --ignored --nocapture
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

    // Get repository configuration from environment
    let repository_owner =
        std::env::var("REPOSITORY_OWNER").unwrap_or_else(|_| "codekiln".to_string());
    let repository_name =
        std::env::var("REPOSITORY_NAME").unwrap_or_else(|_| "langstar".to_string());

    println!("🚀 Starting deployment workflow test");
    println!("   Repository: {}/{}", repository_owner, repository_name);
    println!();

    // Create client
    let client = LangchainClient::new(auth).expect("Failed to create LangchainClient");

    // Step 1: Find GitHub integration ID
    println!(
        "🔍 Finding GitHub integration for {}/{}",
        repository_owner, repository_name
    );
    let integration_id = client
        .integrations()
        .find_integration_for_repo(&repository_owner, &repository_name)
        .await
        .expect("Failed to find GitHub integration for repository");
    println!("✓ Found integration ID: {}", integration_id);
    println!();

    // Step 2: Create deployment with timestamp-based unique name
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let deployment_name = format!("{}-test-{}", repository_name, timestamp);

    println!("📦 Creating deployment: {}", deployment_name);
    let create_request = CreateDeploymentRequest {
        name: deployment_name.clone(),
        source: "github".to_string(),
        source_config: json!({
            "integration_id": integration_id,
            "repo_url": format!("https://github.com/{}/{}", repository_owner, repository_name),
            "deployment_type": "dev",
            "build_on_push": false,
            "custom_url": null,
            "resource_spec": null,
        }),
        source_revision_config: json!({
            "repo_ref": "main",
            "langgraph_config_path": "tests/fixtures/test-graph-deployment/langgraph.json",
            "image_uri": null,
        }),
        secrets: vec![],
    };

    let deployment = client
        .deployments()
        .create(&create_request)
        .await
        .expect("Failed to create deployment");
    let deployment_id = deployment.id.clone();
    println!(
        "✓ Created deployment: {} ({})",
        deployment_name, deployment_id
    );
    println!();

    // Step 3: Get latest revision
    println!("📋 Fetching revisions...");
    let revisions = client
        .deployments()
        .list_revisions(&deployment_id)
        .await
        .expect("Failed to list revisions");

    assert!(
        !revisions.resources.is_empty(),
        "Expected at least one revision after creating deployment"
    );

    let latest_revision = &revisions.resources[0];
    let mut latest_revision_id = latest_revision.id.clone();
    println!("✓ Latest revision: {}", latest_revision_id);
    println!("  Status: {:?}", latest_revision.status);
    println!();

    // Step 4: Poll revision status until DEPLOYED (first revision)
    println!("⏳ Waiting for first revision to deploy...");
    wait_for_deployment(&client, &deployment_id, &latest_revision_id)
        .await
        .expect("First revision failed to deploy");
    println!("✓ First revision deployed successfully!");
    println!();

    // Step 5: Patch deployment (triggers new revision)
    println!("🔧 Patching deployment (triggering new revision)...");
    let patch_request = PatchDeploymentRequest {
        source_config: Some(json!({
            "build_on_push": true,
        })),
        source_revision_config: Some(json!({
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

    // Step 6: Get new latest revision
    println!("📋 Fetching new revisions...");
    let revisions = client
        .deployments()
        .list_revisions(&deployment_id)
        .await
        .expect("Failed to list revisions after patch");

    let new_latest_revision = &revisions.resources[0];
    latest_revision_id = new_latest_revision.id.clone();
    println!("✓ New latest revision: {}", latest_revision_id);
    println!("  Status: {:?}", new_latest_revision.status);
    println!();

    // Step 7: Poll new revision status until DEPLOYED
    println!("⏳ Waiting for second revision to deploy...");
    wait_for_deployment(&client, &deployment_id, &latest_revision_id)
        .await
        .expect("Second revision failed to deploy");
    println!("✓ Second revision deployed successfully!");
    println!();

    // Step 8: Delete deployment (cleanup)
    println!("🗑️  Deleting deployment...");
    client
        .deployments()
        .delete(&deployment_id)
        .await
        .expect("Failed to delete deployment");
    println!("✓ Deployment deleted: {}", deployment_id);
    println!();

    println!("✅ Deployment workflow test completed successfully!");
}

/// Wait for a revision to reach DEPLOYED status
///
/// Polls the revision status every 60 seconds until:
/// - Status is DEPLOYED (success)
/// - Status contains "FAILED" (error)
/// - Timeout of 30 minutes is reached (error)
///
/// # Arguments
/// * `client` - The LangchainClient
/// * `deployment_id` - UUID of the deployment
/// * `revision_id` - UUID of the revision to poll
///
/// # Returns
/// * `Ok(())` - Revision reached DEPLOYED status
/// * `Err(...)` - Revision failed or timeout occurred
async fn wait_for_deployment(
    client: &LangchainClient,
    deployment_id: &str,
    revision_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    const POLL_INTERVAL: Duration = Duration::from_secs(60);
    const MAX_WAIT_TIME: Duration = Duration::from_secs(1800); // 30 minutes

    let start_time = tokio::time::Instant::now();

    loop {
        // Check timeout
        if start_time.elapsed() >= MAX_WAIT_TIME {
            return Err(format!(
                "Timeout waiting for revision {} to be DEPLOYED after 30 minutes",
                revision_id
            )
            .into());
        }

        // Get revision status
        let revision = client
            .deployments()
            .get_revision(deployment_id, revision_id)
            .await?;

        println!("  Revision status: {:?}", revision.status);

        // Check status
        match revision.status {
            RevisionStatus::Deployed => {
                return Ok(());
            }
            RevisionStatus::BuildFailed
            | RevisionStatus::DeployFailed
            | RevisionStatus::Cancelled => {
                return Err(format!(
                    "Revision {} failed with status: {:?}",
                    revision_id, revision.status
                )
                .into());
            }
            _ => {
                // Still in progress, wait and poll again
                println!(
                    "  Waiting {} seconds before next check...",
                    POLL_INTERVAL.as_secs()
                );
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        }
    }
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

    // List deployments (limit to 5 for faster test)
    let result = client.deployments().list(Some(5), None, None).await;

    match result {
        Ok(deployments_list) => {
            println!("✓ Successfully fetched deployments");
            println!("  Total returned: {}", deployments_list.resources.len());
            println!("  Offset: {}", deployments_list.offset);

            // Display first few deployments
            for (i, deployment) in deployments_list.resources.iter().take(3).enumerate() {
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
