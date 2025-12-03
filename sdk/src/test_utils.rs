//! Test utilities for integration tests
//!
//! This module provides shared test infrastructure for managing LangGraph deployments
//! during integration tests. Both SDK and CLI tests use these utilities for deployment
//! lifecycle management.
//!
//! # Usage
//!
//! Enable the `test-utils` feature in your Cargo.toml:
//!
//! ```toml
//! [dev-dependencies]
//! langstar-sdk = { path = "../sdk", features = ["test-utils"] }
//! ```
//!
//! Then use the utilities in your tests:
//!
//! ```ignore
//! use langstar_sdk::test_utils::{wait_for_deployment, DeploymentGuard, TestDeploymentConfig};
//! ```

use crate::{
    CreateDeploymentRequest, DeploymentFilters, LangchainClient, Revision, RevisionStatus,
};
use serde_json::json;
use std::time::Duration;

/// Configuration for creating test deployments
#[derive(Debug, Clone)]
pub struct TestDeploymentConfig {
    /// Name of the deployment
    pub name: String,
    /// Repository owner (e.g., "codekiln")
    pub repository_owner: String,
    /// Repository name (e.g., "langstar")
    pub repository_name: String,
    /// Branch to deploy from (default: "main")
    pub branch: String,
    /// Path to langgraph.json config file
    pub config_path: String,
}

impl Default for TestDeploymentConfig {
    fn default() -> Self {
        Self {
            name: "langstar-integration-test".to_string(),
            repository_owner: std::env::var("REPOSITORY_OWNER")
                .unwrap_or_else(|_| "codekiln".to_string()),
            repository_name: std::env::var("REPOSITORY_NAME")
                .unwrap_or_else(|_| "langstar".to_string()),
            branch: "main".to_string(),
            config_path: "tests/fixtures/test-graph-deployment/langgraph.json".to_string(),
        }
    }
}

/// RAII guard to remind about deployment cleanup
///
/// This guard provides a warning if a test fails before manually cleaning up
/// a deployment. Due to async context limitations, it cannot perform automatic
/// cleanup from Drop, but serves as a reminder to clean up orphaned deployments.
///
/// Use `disarm()` after manual deletion to prevent the warning.
///
/// # Example
///
/// ```ignore
/// let guard = DeploymentGuard::new(deployment_id.clone());
///
/// // ... test code that might fail ...
///
/// // After manual cleanup
/// client.deployments().delete(&deployment_id).await?;
/// guard.disarm();
/// ```
pub struct DeploymentGuard {
    deployment_id: String,
    armed: bool,
}

impl DeploymentGuard {
    /// Create a new deployment guard
    pub fn new(deployment_id: String) -> Self {
        Self {
            deployment_id,
            armed: true,
        }
    }

    /// Disarm the guard to prevent automatic cleanup warning
    ///
    /// Call this when you want to manually control deployment deletion
    /// (e.g., after explicitly deleting it in the test)
    pub fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for DeploymentGuard {
    fn drop(&mut self) {
        if self.armed {
            eprintln!(
                "DeploymentGuard: Test failed before manual cleanup of deployment {}",
                self.deployment_id
            );
            eprintln!("   Please manually delete this deployment if it still exists.");
            eprintln!("   Note: Automatic cleanup from Drop is not supported in async contexts.");
        }
    }
}

/// Default poll interval for waiting on deployment status
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(60);

/// Default maximum wait time for deployment to reach DEPLOYED status (30 minutes)
pub const DEFAULT_MAX_WAIT_TIME: Duration = Duration::from_secs(1800);

/// Wait for a deployment revision to reach DEPLOYED status
///
/// Polls the revision status at the specified interval until:
/// - Status is DEPLOYED (success)
/// - Status is a failure state (BuildFailed, DeployFailed, Cancelled) (error)
/// - Timeout is reached (error)
///
/// # Arguments
///
/// * `client` - The LangchainClient
/// * `deployment_id` - UUID of the deployment
/// * `revision_id` - UUID of the revision to poll
///
/// # Returns
///
/// * `Ok(Revision)` - The revision after reaching DEPLOYED status
/// * `Err(...)` - If revision failed or timeout occurred
///
/// # Example
///
/// ```ignore
/// let revision = wait_for_deployment(&client, &deployment_id, &revision_id).await?;
/// assert_eq!(revision.status, RevisionStatus::Deployed);
/// ```
pub async fn wait_for_deployment(
    client: &LangchainClient,
    deployment_id: &str,
    revision_id: &str,
) -> Result<Revision, Box<dyn std::error::Error + Send + Sync>> {
    wait_for_deployment_with_options(
        client,
        deployment_id,
        revision_id,
        DEFAULT_POLL_INTERVAL,
        DEFAULT_MAX_WAIT_TIME,
    )
    .await
}

/// Wait for a deployment revision to reach DEPLOYED status with custom options
///
/// Like `wait_for_deployment` but allows customizing poll interval and timeout.
///
/// # Arguments
///
/// * `client` - The LangchainClient
/// * `deployment_id` - UUID of the deployment
/// * `revision_id` - UUID of the revision to poll
/// * `poll_interval` - How often to poll the status
/// * `max_wait_time` - Maximum time to wait before timeout
///
/// # Returns
///
/// * `Ok(Revision)` - The revision after reaching DEPLOYED status
/// * `Err(...)` - If revision failed or timeout occurred
pub async fn wait_for_deployment_with_options(
    client: &LangchainClient,
    deployment_id: &str,
    revision_id: &str,
    poll_interval: Duration,
    max_wait_time: Duration,
) -> Result<Revision, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = tokio::time::Instant::now();

    loop {
        // Check timeout
        if start_time.elapsed() >= max_wait_time {
            return Err(format!(
                "Timeout waiting for revision {} to be DEPLOYED after {} seconds",
                revision_id,
                max_wait_time.as_secs()
            )
            .into());
        }

        // Get revision status
        let revision = client
            .deployments()
            .get_revision(deployment_id, revision_id)
            .await?;

        eprintln!("  Revision status: {:?}", revision.status);

        // Check status
        match revision.status {
            RevisionStatus::Deployed => {
                return Ok(revision);
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
                eprintln!(
                    "  Waiting {} seconds before next check...",
                    poll_interval.as_secs()
                );
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}

/// Get or create a test deployment by name
///
/// This function implements the "get-or-create" pattern:
/// 1. Look for existing deployment by name (any status)
/// 2. If found and in progress, wait for it to become DEPLOYED
/// 3. If not found, create a new deployment
///
/// This approach is faster for repeated test runs because it reuses existing
/// deployments instead of creating new ones each time.
///
/// # Arguments
///
/// * `client` - The LangchainClient
/// * `config` - Configuration for the test deployment
///
/// # Returns
///
/// * `Ok((deployment_id, revision_id))` - IDs of the deployment and its latest revision
/// * `Err(...)` - If creation or waiting failed
///
/// # Example
///
/// ```ignore
/// let config = TestDeploymentConfig::default();
/// let (deployment_id, revision_id) = get_or_create_deployment(&client, &config).await?;
/// ```
pub async fn get_or_create_deployment(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    // Step 1: Find GitHub integration ID
    eprintln!(
        "Finding GitHub integration for {}/{}",
        config.repository_owner, config.repository_name
    );
    let integration_id = client
        .integrations()
        .find_integration_for_repo(&config.repository_owner, &config.repository_name)
        .await?;
    eprintln!("Found integration ID: {}", integration_id);

    // Step 2: Look for existing deployment by name (no status filter!)
    let filters = DeploymentFilters {
        name_contains: Some(config.name.clone()),
        ..Default::default()
    };
    let deployments = client
        .deployments()
        .list(Some(100), None, Some(filters))
        .await?;

    let deployment = if let Some(existing) =
        deployments.resources.iter().find(|d| d.name == config.name)
    {
        eprintln!(
            "Found existing deployment: {} ({})",
            config.name, existing.id
        );
        existing.clone()
    } else {
        // Create new deployment
        eprintln!("No existing deployment found, creating new one...");
        let create_request = CreateDeploymentRequest {
            name: config.name.clone(),
            source: "github".to_string(),
            source_config: json!({
                "integration_id": integration_id,
                "repo_url": format!("https://github.com/{}/{}", config.repository_owner, config.repository_name),
                "deployment_type": "dev",
                "build_on_push": false,
                "custom_url": null,
                "resource_spec": null,
            }),
            source_revision_config: json!({
                "repo_ref": config.branch,
                "langgraph_config_path": config.config_path,
                "image_uri": null,
            }),
            secrets: vec![],
        };

        let new_deployment = client.deployments().create(&create_request).await?;
        eprintln!(
            "Created deployment: {} ({})",
            config.name, new_deployment.id
        );
        new_deployment
    };

    let deployment_id = deployment.id.clone();

    // Step 3: Get latest revision
    let revisions = client.deployments().list_revisions(&deployment_id).await?;

    if revisions.resources.is_empty() {
        return Err(format!(
            "No revisions found for deployment {} - this should not happen",
            deployment_id
        )
        .into());
    }

    let latest_revision = &revisions.resources[0];
    let revision_id = latest_revision.id.clone();

    eprintln!(
        "Latest revision: {} (status: {:?})",
        revision_id, latest_revision.status
    );

    // Step 4: Wait for deployment if not already deployed
    if latest_revision.status != RevisionStatus::Deployed {
        eprintln!("Waiting for deployment to become DEPLOYED...");
        wait_for_deployment(client, &deployment_id, &revision_id).await?;
        eprintln!("Deployment is now DEPLOYED");
    }

    Ok((deployment_id, revision_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_guard_armed() {
        // Test that guard emits warning when dropped while armed
        // (We can't easily test the eprintln output, but we verify it doesn't panic)
        let _guard = DeploymentGuard::new("test-id".to_string());
        // Guard will be dropped and print warning
    }

    #[test]
    fn test_deployment_guard_disarmed() {
        // Test that guard doesn't emit warning when disarmed
        let mut guard = DeploymentGuard::new("test-id".to_string());
        guard.disarm();
        // Guard will be dropped without warning
    }

    #[test]
    fn test_deployment_config_default() {
        let config = TestDeploymentConfig::default();
        assert_eq!(config.name, "langstar-integration-test");
        assert_eq!(config.branch, "main");
        assert!(config.config_path.contains("langgraph.json"));
    }
}
