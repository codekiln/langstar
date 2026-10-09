//! Test utilities for integration tests
//!
//! This module provides shared test infrastructure for managing LangGraph deployments
//! during integration tests. Both SDK and CLI tests use these utilities for deployment
//! lifecycle management.
//!
//! # Deployment vs Revision Status
//!
//! LangGraph Cloud has two distinct status types:
//! - [`DeploymentStatus`](crate::DeploymentStatus) - Overall deployment state (terminal: `Ready`)
//! - [`RevisionStatus`](crate::RevisionStatus) - Build/deploy state of a revision (terminal: `Deployed`)
//!
//! See `docs/langgraph-deployments-and-revisions.md` for detailed documentation.
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
    CreateDeploymentRequest, DeploymentFilters, DeploymentStatus, LangchainClient, LangstarError,
    Revision, RevisionStatus,
};
use serde_json::json;
use std::time::Duration;

/// Prefix for PR/dev test deployments (reusable via get-or-create)
/// Note: No trailing hyphen - this matches both old "pr-integration-test" and new "pr-integration-test-{ts}"
pub const PR_TEST_DEPLOYMENT_PREFIX: &str = "pr-integration-test";

/// Prefix for release lifecycle test deployments (create fresh, delete after)
pub const RELEASE_TEST_DEPLOYMENT_PREFIX: &str = "release-integration-test";

/// Configuration for creating test deployments
#[derive(Debug, Clone)]
pub struct TestDeploymentConfig {
    /// Name of the deployment
    pub name: String,
    /// Optional prefix for get-or-create lookup (None = always create fresh)
    pub name_prefix: Option<String>,
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
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            // Use timestamped name with prefix for get-or-create pattern.
            // Both SDK and CLI tests share deployments via prefix-based lookup.
            // The "pr-" prefix indicates these are reused for PR/development testing.
            // See also: for_release_tests() for one-time lifecycle testing.
            name: format!("{}-{}", PR_TEST_DEPLOYMENT_PREFIX, timestamp),
            name_prefix: Some(PR_TEST_DEPLOYMENT_PREFIX.to_string()),
            repository_owner: std::env::var("REPOSITORY_OWNER")
                .unwrap_or_else(|_| "codekiln".to_string()),
            repository_name: std::env::var("REPOSITORY_NAME")
                .unwrap_or_else(|_| "langstar".to_string()),
            branch: "main".to_string(),
            config_path: "tests/fixtures/test-graph-deployment/langgraph.json".to_string(),
        }
    }
}

impl TestDeploymentConfig {
    /// Create configuration for release/lifecycle tests
    ///
    /// Uses a fresh `release-integration-test-*` name, so the full
    /// create → test → delete lifecycle runs against a deployment of its own.
    /// These deployments should be cleaned up after the test completes.
    ///
    /// Sets `name_prefix: None` so get-or-create always creates fresh.
    pub fn for_release_tests() -> Self {
        Self {
            name: release_test_deployment_name(),
            name_prefix: None, // No prefix search - always create fresh
            ..Default::default()
        }
    }
}

/// A new `release-integration-test-*` name.
///
/// Every run that uses the same workspace shares its deployments, so a name
/// built from a seconds timestamp repeats when two runs start in the same
/// second. The first 12 hex characters of a random UUID make a repeat
/// vanishingly unlikely.
fn release_test_deployment_name() -> String {
    format!(
        "{}-{}",
        RELEASE_TEST_DEPLOYMENT_PREFIX,
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    )
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

    /// Disarm the guard to suppress the cleanup reminder warning
    ///
    /// Call this after manually deleting the deployment in your test
    /// to indicate cleanup was performed and suppress the warning.
    pub fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for DeploymentGuard {
    fn drop(&mut self) {
        if self.armed {
            eprintln!(
                "Warning: Test may have failed before cleanup of deployment {}",
                self.deployment_id
            );
            eprintln!("   Please manually delete this deployment if it still exists.");
            eprintln!("   Note: Automatic cleanup from Drop is not supported in async contexts.");
        }
    }
}

/// Check if an error is a 409 conflict due to orphaned tracing project
///
/// LangSmith automatically creates a tracing project with the same name as a deployment.
/// If the deployment is deleted but the tracing project isn't, subsequent attempts to
/// create a deployment with the same name will fail with a 409 conflict.
fn is_tracing_project_conflict(error: &LangstarError) -> bool {
    match error {
        LangstarError::ApiError { status, message } => {
            // Case-insensitive check in case API returns "Tracing Project" or other variations
            *status == 409 && message.to_lowercase().contains("tracing project")
        }
        _ => false,
    }
}

/// Check if an error is any 409 conflict
fn is_conflict_error(error: &LangstarError) -> bool {
    matches!(error, LangstarError::ApiError { status: 409, .. })
}

/// Default poll interval for waiting on revision status
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(60);

/// Default maximum wait time for revision to reach `RevisionStatus::Deployed` (30 minutes)
pub const DEFAULT_MAX_WAIT_TIME: Duration = Duration::from_secs(1800);

/// Wait for a revision to reach `RevisionStatus::Deployed`
///
/// Polls the revision status at the specified interval until:
/// - `RevisionStatus::Deployed` (success)
/// - A failure state: `BuildFailed`, `DeployFailed`, `Cancelled` (error)
/// - Timeout is reached (error)
///
/// Note: This waits for **revision** status, not deployment status.
/// See `docs/langgraph-deployments-and-revisions.md` for the distinction.
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

/// Helper function to create a new deployment
async fn create_new_deployment(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
    integration_id: &str,
) -> Result<crate::Deployment, Box<dyn std::error::Error + Send + Sync>> {
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

    match client.deployments().create(&create_request).await {
        Ok(new_deployment) => {
            eprintln!(
                "Created deployment: {} ({})",
                config.name, new_deployment.id
            );
            Ok(new_deployment)
        }
        // Keep the original error, so a caller can see it is a 409. The
        // callers print `print_create_conflict_guidance` only once they stop
        // retrying, so a create that a later retry fixes prints no advice.
        Err(err) => Err(err.into()),
    }
}

/// Print what to do about a 409 from a deployment create named `name`.
///
/// Callers print this once they stop retrying. It prints nothing for an error
/// that isn't a 409.
fn print_create_conflict_guidance(
    err: &(dyn std::error::Error + Send + Sync + 'static),
    name: &str,
) {
    let Some(err) = err.downcast_ref::<LangstarError>() else {
        return;
    };
    if is_tracing_project_conflict(err) {
        eprintln!();
        eprintln!("╭────────────────────────────────────────────────────────────────╮");
        eprintln!("│ ⚠️  ORPHANED TRACING PROJECT DETECTED                           │");
        eprintln!("├────────────────────────────────────────────────────────────────┤");
        eprintln!("│ A previous deployment was deleted but its associated tracing  │");
        eprintln!("│ project in LangSmith was not. This blocks creating a new      │");
        eprintln!("│ deployment with the same name.                                │");
        eprintln!("├────────────────────────────────────────────────────────────────┤");
        eprintln!("│ To fix this issue:                                            │");
        eprintln!("│  1. Go to LangSmith UI → Projects tab                         │");
        // Truncate long names to fit in the 24-char column; show full name in error below
        let display_name = if name.len() > 24 {
            format!("{}...", &name[..21])
        } else {
            name.to_string()
        };
        eprintln!("│  2. Find and delete project named: {:24} │", display_name);
        eprintln!("│  3. Re-run the tests                                          │");
        eprintln!("╰────────────────────────────────────────────────────────────────╯");
        eprintln!();
    } else if is_conflict_error(err) {
        eprintln!();
        eprintln!("╭────────────────────────────────────────────────────────────────╮");
        eprintln!("│ ⚠️  409 CONFLICT ERROR                                          │");
        eprintln!("├────────────────────────────────────────────────────────────────┤");
        eprintln!("│ A resource conflict occurred. This may indicate:              │");
        eprintln!("│  - A concurrent test is using the same deployment name        │");
        eprintln!("│  - An orphaned resource needs manual cleanup                  │");
        eprintln!("├────────────────────────────────────────────────────────────────┤");
        eprintln!("│ Suggested actions:                                            │");
        eprintln!("│  1. Check if another CI run is in progress                    │");
        eprintln!("│  2. Check LangSmith UI for orphaned projects/deployments      │");
        eprintln!("│  3. Wait and retry if concurrent access suspected             │");
        eprintln!("╰────────────────────────────────────────────────────────────────╯");
        eprintln!();
    }
}

/// How many times `reuse_or_create_deployment` looks for, or tries to create,
/// the shared deployment before giving up.
const REUSE_ATTEMPTS: u32 = 10;

/// How long to wait before looking again while another CI run's deployment,
/// or one still being deleted, takes up the test graph's agent environment.
const REUSE_RETRY_INTERVAL: Duration = Duration::from_secs(30);

/// True for a deployment that is being deleted and can't be reused.
fn is_being_deleted(deployment: &crate::Deployment) -> bool {
    matches!(
        deployment.status,
        DeploymentStatus::AwaitingDelete | DeploymentStatus::AwaitingFinalDelete
    )
}

/// True when the deployment builds from the repository and graph config in `config`.
fn deploys_test_graph(deployment: &crate::Deployment, config: &TestDeploymentConfig) -> bool {
    let repo_url = format!(
        "https://github.com/{}/{}",
        config.repository_owner, config.repository_name
    );
    let same_repo = deployment
        .source_config
        .as_ref()
        .and_then(|c| c.get("repo_url"))
        .and_then(|v| v.as_str())
        .is_some_and(|url| url.trim_end_matches(".git") == repo_url);
    let same_graph = deployment
        .source_revision_config
        .as_ref()
        .and_then(|c| c.get("langgraph_config_path"))
        .and_then(|v| v.as_str())
        .is_some_and(|path| path == config.config_path);
    same_repo && same_graph
}

/// True when the PR tests can reuse this deployment of the test graph.
///
/// The release lifecycle test deletes its `release-integration-test-*`
/// deployment when it finishes, so this skips those deployments as well as
/// deployments being deleted.
fn is_reusable_by_source(deployment: &crate::Deployment, config: &TestDeploymentConfig) -> bool {
    deploys_test_graph(deployment, config)
        && !is_being_deleted(deployment)
        && !deployment.name.starts_with(RELEASE_TEST_DEPLOYMENT_PREFIX)
}

/// Find a live deployment to reuse: first by name prefix, then by source.
///
/// The control plane allows one deployment per agent environment, so a
/// deployment of the test graph under any name blocks creating another one
/// (409 "A deployment already exists for this agent environment").
async fn find_reusable_deployment(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
) -> Result<Option<crate::Deployment>, Box<dyn std::error::Error + Send + Sync>> {
    let prefix = config.name_prefix.clone().unwrap_or_default();

    let by_name = client
        .deployments()
        .list(
            Some(100),
            None,
            Some(DeploymentFilters {
                name_contains: Some(prefix.clone()),
                ..Default::default()
            }),
        )
        .await?;
    if let Some(existing) = by_name
        .resources
        .into_iter()
        .find(|d| d.name.starts_with(&prefix) && !is_being_deleted(d))
    {
        return Ok(Some(existing));
    }

    // The control plane allows one deployment of the test graph per agent
    // environment across the whole workspace, so read every page.
    const PAGE: u32 = 100;
    let mut offset = 0;
    loop {
        let page = client
            .deployments()
            .list(Some(PAGE), Some(offset), None)
            .await?;
        let count = page.resources.len();
        if let Some(found) = page
            .resources
            .into_iter()
            .find(|d| is_reusable_by_source(d, config))
        {
            return Ok(Some(found));
        }
        if count < PAGE as usize {
            return Ok(None);
        }
        offset += PAGE;
    }
}

/// Reuse whatever live deployment of the test graph exists, or create one.
///
/// Between this function's lookup and its create request, another CI run can
/// create the test deployment, or the scheduled cleanup job can still be
/// deleting an old one. The control plane answers either case with 409, and
/// this function waits and looks again.
async fn reuse_or_create_deployment(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
    integration_id: &str,
) -> Result<crate::Deployment, Box<dyn std::error::Error + Send + Sync>> {
    reuse_or_create_deployment_with_attempts(
        client,
        config,
        integration_id,
        REUSE_ATTEMPTS,
        REUSE_RETRY_INTERVAL,
    )
    .await
}

/// Reuse or create the test deployment, trying up to `attempts` times and
/// sleeping `retry_interval` after each 409. `reuse_or_create_deployment`
/// passes `REUSE_ATTEMPTS` and `REUSE_RETRY_INTERVAL`; the mocked tests pass
/// a zero wait.
async fn reuse_or_create_deployment_with_attempts(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
    integration_id: &str,
    attempts: u32,
    retry_interval: Duration,
) -> Result<crate::Deployment, Box<dyn std::error::Error + Send + Sync>> {
    for attempt in 1..=attempts {
        if let Some(existing) = find_reusable_deployment(client, config).await? {
            eprintln!(
                "Found existing deployment: {} (status: {:?})",
                existing.name, existing.status
            );
            return Ok(existing);
        }

        match create_new_deployment(client, config, integration_id).await {
            Ok(created) => return Ok(created),
            // An orphaned tracing project blocks this exact name, and this
            // loop retries under the same name, so waiting can't help.
            Err(err)
                if attempt < attempts
                    && err.downcast_ref::<LangstarError>().is_some_and(|e| {
                        is_conflict_error(e) && !is_tracing_project_conflict(e)
                    }) =>
            {
                eprintln!(
                    "Create collided with an existing deployment (attempt {}/{}); \
                     waiting {}s and looking again...",
                    attempt,
                    attempts,
                    retry_interval.as_secs()
                );
                tokio::time::sleep(retry_interval).await;
            }
            Err(err) => {
                print_create_conflict_guidance(err.as_ref(), &config.name);
                return Err(err);
            }
        }
    }
    Err(
        format!("no attempt to reuse or create the test deployment ran (attempts = {attempts})")
            .into(),
    )
}

/// How many times `create_fresh_deployment` tries to create the release test
/// deployment before giving up.
const FRESH_CREATE_ATTEMPTS: u32 = 5;

/// How long `create_fresh_deployment` waits after a 409 before trying again.
const FRESH_CREATE_RETRY_INTERVAL: Duration = Duration::from_secs(30);

/// Create a fresh deployment for a config with no `name_prefix`.
///
/// A config whose name starts with `release-integration-test-`, which
/// `TestDeploymentConfig::for_release_tests` builds, gets a retry: when the
/// control plane answers the create with 409, this waits and tries again under
/// a new `release-integration-test-*` name. The retry is a precaution. CI in
/// [🩹 fix(api): adapt the SDK and tests to LangSmith API drift · PR #755](https://github.com/codekiln/langstar/pull/755)
/// got 409 "A deployment already exists for this agent environment" when
/// concurrent runs created the shared `pr-integration-test-*` deployment,
/// which this function never creates. Nobody has seen the release test get
/// it. A new name covers two release runs that pick the same name, and the
/// wait covers a release deployment that is still being deleted. Any other
/// config is created once, under the name the caller chose.
async fn create_fresh_deployment(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
    integration_id: &str,
) -> Result<crate::Deployment, Box<dyn std::error::Error + Send + Sync>> {
    create_fresh_deployment_with_attempts(
        client,
        config,
        integration_id,
        FRESH_CREATE_ATTEMPTS,
        FRESH_CREATE_RETRY_INTERVAL,
    )
    .await
}

/// Create a fresh deployment, trying up to `attempts` times with a new name
/// and a `retry_interval` wait after each 409. `create_fresh_deployment`
/// passes `FRESH_CREATE_ATTEMPTS` and `FRESH_CREATE_RETRY_INTERVAL`; the
/// mocked tests pass a zero wait.
async fn create_fresh_deployment_with_attempts(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
    integration_id: &str,
    attempts: u32,
    retry_interval: Duration,
) -> Result<crate::Deployment, Box<dyn std::error::Error + Send + Sync>> {
    if !config
        .name
        .starts_with(&format!("{RELEASE_TEST_DEPLOYMENT_PREFIX}-"))
    {
        return create_new_deployment(client, config, integration_id)
            .await
            .inspect_err(|err| print_create_conflict_guidance(err.as_ref(), &config.name));
    }
    let mut attempt_config = config.clone();
    for attempt in 1..=attempts {
        match create_new_deployment(client, &attempt_config, integration_id).await {
            Ok(created) => return Ok(created),
            Err(err)
                if attempt < attempts
                    && err
                        .downcast_ref::<LangstarError>()
                        .is_some_and(is_conflict_error) =>
            {
                attempt_config.name = release_test_deployment_name();
                eprintln!(
                    "Create returned 409 (attempt {}/{}); waiting {}s and trying again as {}...",
                    attempt,
                    attempts,
                    retry_interval.as_secs(),
                    attempt_config.name
                );
                tokio::time::sleep(retry_interval).await;
            }
            Err(err) => {
                print_create_conflict_guidance(err.as_ref(), &attempt_config.name);
                return Err(err);
            }
        }
    }
    Err(format!("no attempt to create the fresh deployment ran (attempts = {attempts})").into())
}

/// Get or create a test deployment by name
///
/// This function implements the "get-or-create" pattern:
/// 1. Look for existing deployment by name (any `DeploymentStatus`)
/// 2. If found, wait for latest revision to reach `RevisionStatus::Deployed`
/// 3. If not found, create a new deployment and wait for its revision
///
/// This approach is faster for repeated test runs because it reuses existing
/// deployments instead of creating new ones each time.
///
/// See `docs/langgraph-deployments-and-revisions.md` for status terminology.
///
/// # Arguments
///
/// * `client` - The LangchainClient
/// * `config` - Configuration for the test deployment
///
/// # Returns
///
/// * `Ok((deployment_id, revision_id, deployment_name))` - IDs and actual name of the deployment
/// * `Err(...)` - If creation or waiting failed
///
/// Note: The returned `deployment_name` may differ from `config.name` when an existing
/// deployment is reused via prefix matching, or when a release config's create got a 409
/// and was retried under a new `release-integration-test-*` name. Always use the returned
/// name for assertions.
///
/// # Example
///
/// ```ignore
/// let config = TestDeploymentConfig::default();
/// let (deployment_id, revision_id, name) = get_or_create_deployment(&client, &config).await?;
/// ```
pub async fn get_or_create_deployment(
    client: &LangchainClient,
    config: &TestDeploymentConfig,
) -> Result<(String, String, String), Box<dyn std::error::Error + Send + Sync>> {
    // Step 1: Find GitHub integration ID
    eprintln!(
        "Finding GitHub integration for {}/{}",
        config.repository_owner, config.repository_name
    );
    let integration_id = client
        .integrations()
        .find_integration_for_repo(&config.repository_owner, &config.repository_name)
        .await?;
    eprintln!("Found GitHub integration");

    // Step 2: Reuse a matching deployment, or create one
    let deployment = if config.name_prefix.is_some() {
        reuse_or_create_deployment(client, config, &integration_id).await?
    } else {
        // No prefix: always create fresh
        create_fresh_deployment(client, config, &integration_id).await?
    };

    let deployment_id = deployment.id.clone();
    let deployment_name = deployment.name.clone();

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

    Ok((deployment_id, revision_id, deployment_name))
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
        // Name now has timestamp suffix
        assert!(
            config.name.starts_with("pr-integration-test-"),
            "Default name should start with pr-integration-test-"
        );
        // name_prefix is set for get-or-create lookup
        assert_eq!(
            config.name_prefix,
            Some("pr-integration-test".to_string()),
            "Should have name_prefix for get-or-create"
        );
        assert_eq!(config.branch, "main");
        assert!(config.config_path.contains("langgraph.json"));
    }

    fn deployment_from(status: &str, repo_url: &str, config_path: &str) -> crate::Deployment {
        serde_json::from_value(json!({
            "id": "test-id",
            "name": "some-other-name",
            "source": "github",
            "source_config": {"repo_url": repo_url},
            "source_revision_config": {"langgraph_config_path": config_path},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
            "status": status
        }))
        .expect("test deployment JSON should deserialize")
    }

    #[test]
    fn test_deploys_test_graph_matches_repo_and_config_path() {
        let config = TestDeploymentConfig::default();
        let repo = format!(
            "https://github.com/{}/{}",
            config.repository_owner, config.repository_name
        );

        assert!(deploys_test_graph(
            &deployment_from("READY", &repo, &config.config_path),
            &config
        ));
        assert!(deploys_test_graph(
            &deployment_from("READY", &format!("{repo}.git"), &config.config_path),
            &config
        ));
        assert!(!deploys_test_graph(
            &deployment_from(
                "READY",
                "https://github.com/other/repo",
                &config.config_path
            ),
            &config
        ));
        assert!(!deploys_test_graph(
            &deployment_from("READY", &repo, "other/langgraph.json"),
            &config
        ));
    }

    #[test]
    fn test_is_reusable_by_source_skips_release_lifecycle_deployments() {
        let config = TestDeploymentConfig::default();
        let repo = format!(
            "https://github.com/{}/{}",
            config.repository_owner, config.repository_name
        );
        let mut d = deployment_from("READY", &repo, &config.config_path);
        assert!(is_reusable_by_source(&d, &config));

        d.name = format!("{}-1234", RELEASE_TEST_DEPLOYMENT_PREFIX);
        assert!(!is_reusable_by_source(&d, &config));

        let deleting = deployment_from("AWAITING_DELETE", &repo, &config.config_path);
        assert!(!is_reusable_by_source(&deleting, &config));
    }

    #[test]
    fn test_is_being_deleted() {
        let config = TestDeploymentConfig::default();
        let path = config.config_path.as_str();
        assert!(is_being_deleted(&deployment_from(
            "AWAITING_DELETE",
            "",
            path
        )));
        assert!(is_being_deleted(&deployment_from(
            "AWAITING_FINAL_DELETE",
            "",
            path
        )));
        assert!(!is_being_deleted(&deployment_from("READY", "", path)));
        assert!(!is_being_deleted(&deployment_from(
            "AWAITING_DATABASE",
            "",
            path
        )));
    }

    // ── reuse_or_create_deployment_with_attempts against a mocked control plane ──

    use mockito::{Matcher, Server};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn mock_client(server: &Server) -> LangchainClient {
        LangchainClient::with_base_urls(
            crate::AuthConfig::new(
                Some("test-key".to_string()),
                None,
                Some("test-workspace".to_string()),
            ),
            server.url(),
            server.url(),
            server.url(),
        )
        .expect("client")
    }

    fn deployment_json(name: &str) -> serde_json::Value {
        json!({
            "id": "existing-id",
            "name": name,
            "source": "github",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
            "status": "READY"
        })
    }

    #[tokio::test]
    async fn test_retry_reuses_deployment_that_appears_after_409() {
        let mut server = Server::new_async().await;
        let config = TestDeploymentConfig::default();
        let existing_name = format!("{}-other-run", config.name_prefix.clone().unwrap());

        // The list is empty until a create has been rejected, as when another
        // CI run creates the deployment between our lookup and our create.
        let creates = Arc::new(AtomicUsize::new(0));
        let creates_seen = creates.clone();
        let list = server
            .mock("GET", "/v2/deployments")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body_from_request(move |_| {
                let resources = if creates_seen.load(Ordering::SeqCst) == 0 {
                    json!([])
                } else {
                    json!([deployment_json(&existing_name)])
                };
                json!({"resources": resources, "offset": 0})
                    .to_string()
                    .into_bytes()
            })
            .expect_at_least(3)
            .create_async()
            .await;
        let create = server
            .mock("POST", "/v2/deployments")
            .with_status(409)
            .with_body_from_request(move |_| {
                creates.fetch_add(1, Ordering::SeqCst);
                br#"{"detail":"A deployment already exists for this agent environment."}"#.to_vec()
            })
            .expect(1)
            .create_async()
            .await;

        let found = reuse_or_create_deployment_with_attempts(
            &mock_client(&server),
            &config,
            "integration",
            3,
            Duration::ZERO,
        )
        .await
        .expect("should reuse the deployment found after the 409");

        assert_eq!(found.id, "existing-id");
        create.assert_async().await;
        list.assert_async().await;
    }

    #[tokio::test]
    async fn test_retry_gives_up_after_last_attempt() {
        let mut server = Server::new_async().await;
        let config = TestDeploymentConfig::default();

        server
            .mock("GET", "/v2/deployments")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(r#"{"resources": [], "offset": 0}"#)
            .create_async()
            .await;
        let create = server
            .mock("POST", "/v2/deployments")
            .with_status(409)
            .with_body(r#"{"detail":"A deployment already exists for this agent environment."}"#)
            .expect(2)
            .create_async()
            .await;

        let err = reuse_or_create_deployment_with_attempts(
            &mock_client(&server),
            &config,
            "integration",
            2,
            Duration::ZERO,
        )
        .await
        .expect_err("two 409s with two attempts should fail");

        assert!(
            err.downcast_ref::<LangstarError>()
                .is_some_and(is_conflict_error),
            "the last 409 should be returned, got: {err}"
        );
        create.assert_async().await;
    }

    #[tokio::test]
    async fn test_retry_returns_500_on_first_attempt() {
        let mut server = Server::new_async().await;
        let config = TestDeploymentConfig::default();

        server
            .mock("GET", "/v2/deployments")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(r#"{"resources": [], "offset": 0}"#)
            .create_async()
            .await;
        let create = server
            .mock("POST", "/v2/deployments")
            .with_status(500)
            .with_body(r#"{"detail":"Internal server error"}"#)
            .expect(1)
            .create_async()
            .await;

        let err = reuse_or_create_deployment_with_attempts(
            &mock_client(&server),
            &config,
            "integration",
            5,
            Duration::ZERO,
        )
        .await
        .expect_err("a 500 should not be retried");

        assert!(
            matches!(
                err.downcast_ref::<LangstarError>(),
                Some(LangstarError::ApiError { status: 500, .. })
            ),
            "the 500 should be returned unchanged, got: {err}"
        );
        create.assert_async().await;
    }

    // ── create_fresh_deployment_with_attempts against a mocked control plane ──

    /// The deployment name in a create request body.
    fn requested_name(request: &mockito::Request) -> String {
        let body: serde_json::Value =
            serde_json::from_slice(request.body().expect("request body")).expect("JSON body");
        body["name"].as_str().expect("name").to_string()
    }

    #[tokio::test]
    async fn test_fresh_create_retries_a_409_under_a_new_name() {
        let mut server = Server::new_async().await;
        let config = TestDeploymentConfig::for_release_tests();
        let names = Arc::new(std::sync::Mutex::new(Vec::new()));
        let names_seen = names.clone();

        let create = server
            .mock("POST", "/v2/deployments")
            .with_status(409)
            .with_body_from_request(move |request| {
                names_seen.lock().unwrap().push(requested_name(request));
                br#"{"detail":"A deployment already exists for this agent environment."}"#.to_vec()
            })
            .expect(1)
            .create_async()
            .await;
        let names_seen = names.clone();
        let created = server
            .mock("POST", "/v2/deployments")
            .with_status(201)
            .with_body_from_request(move |request| {
                let name = requested_name(request);
                names_seen.lock().unwrap().push(name.clone());
                deployment_json(&name).to_string().into_bytes()
            })
            .expect(1)
            .create_async()
            .await;

        let deployment = create_fresh_deployment_with_attempts(
            &mock_client(&server),
            &config,
            "integration",
            3,
            Duration::ZERO,
        )
        .await
        .expect("the second create should succeed");

        let names = names.lock().unwrap().clone();
        assert_eq!(names.len(), 2, "two create requests should be sent");
        assert_eq!(
            names[0], config.name,
            "the first create uses the config's name"
        );
        assert_ne!(names[1], names[0], "the retry uses a new name");
        assert!(names[1].starts_with(RELEASE_TEST_DEPLOYMENT_PREFIX));
        assert_eq!(deployment.name, names[1]);
        create.assert_async().await;
        created.assert_async().await;
    }

    /// The control plane's 409 when a deleted deployment left its tracing
    /// project behind under the same name.
    const TRACING_PROJECT_409: &str =
        r#"{"detail":"A tracing project with this name already exists."}"#;

    #[tokio::test]
    async fn test_fresh_create_retries_a_tracing_project_409_under_a_new_name() {
        let mut server = Server::new_async().await;
        let config = TestDeploymentConfig::for_release_tests();
        let names = Arc::new(std::sync::Mutex::new(Vec::new()));
        let names_seen = names.clone();

        let conflict = server
            .mock("POST", "/v2/deployments")
            .with_status(409)
            .with_body_from_request(move |request| {
                names_seen.lock().unwrap().push(requested_name(request));
                TRACING_PROJECT_409.as_bytes().to_vec()
            })
            .expect(1)
            .create_async()
            .await;
        let names_seen = names.clone();
        let created = server
            .mock("POST", "/v2/deployments")
            .with_status(201)
            .with_body_from_request(move |request| {
                let name = requested_name(request);
                names_seen.lock().unwrap().push(name.clone());
                deployment_json(&name).to_string().into_bytes()
            })
            .expect(1)
            .create_async()
            .await;

        let deployment = create_fresh_deployment_with_attempts(
            &mock_client(&server),
            &config,
            "integration",
            3,
            Duration::ZERO,
        )
        .await
        .expect("the create under a new name should succeed");

        let names = names.lock().unwrap().clone();
        assert_eq!(names.len(), 2, "two create requests should be sent");
        assert_ne!(names[1], names[0], "the retry uses a new name");
        assert_eq!(deployment.name, names[1]);
        conflict.assert_async().await;
        created.assert_async().await;
    }

    #[tokio::test]
    async fn test_reuse_does_not_retry_a_tracing_project_409() {
        let mut server = Server::new_async().await;
        let _list = server
            .mock("GET", "/v2/deployments")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(json!({"resources": [], "offset": 0}).to_string())
            .create_async()
            .await;
        let create = server
            .mock("POST", "/v2/deployments")
            .with_status(409)
            .with_body(TRACING_PROJECT_409)
            .expect(1)
            .create_async()
            .await;

        let err = reuse_or_create_deployment_with_attempts(
            &mock_client(&server),
            &TestDeploymentConfig::default(),
            "integration",
            3,
            Duration::ZERO,
        )
        .await
        .expect_err("a tracing-project 409 should not be retried under the same name");

        assert!(
            err.downcast_ref::<LangstarError>()
                .is_some_and(is_tracing_project_conflict),
            "the tracing-project 409 should be returned, got: {err}"
        );
        create.assert_async().await;
    }

    #[tokio::test]
    async fn test_fresh_create_gives_up_after_the_last_409() {
        let mut server = Server::new_async().await;
        let create = server
            .mock("POST", "/v2/deployments")
            .with_status(409)
            .with_body(r#"{"detail":"A deployment already exists for this agent environment."}"#)
            .expect(2)
            .create_async()
            .await;

        let err = create_fresh_deployment_with_attempts(
            &mock_client(&server),
            &TestDeploymentConfig::for_release_tests(),
            "integration",
            2,
            Duration::ZERO,
        )
        .await
        .expect_err("two 409s with two attempts should fail");

        assert!(
            err.downcast_ref::<LangstarError>()
                .is_some_and(is_conflict_error),
            "the last 409 should be returned, got: {err}"
        );
        create.assert_async().await;
    }

    /// Send a fresh create for a config named `name` to a control plane that
    /// answers 409, and check that one create goes out, under `name`, and that
    /// the 409 comes back.
    async fn assert_fresh_create_sends_one_create_under(name: &str) {
        let mut server = Server::new_async().await;
        let config = TestDeploymentConfig {
            name: name.to_string(),
            name_prefix: None,
            ..Default::default()
        };
        let create = server
            .mock("POST", "/v2/deployments")
            .match_body(Matcher::PartialJson(json!({ "name": name })))
            .with_status(409)
            .with_body(r#"{"detail":"A deployment already exists for this agent environment."}"#)
            .expect(1)
            .create_async()
            .await;

        let err = create_fresh_deployment_with_attempts(
            &mock_client(&server),
            &config,
            "integration",
            3,
            Duration::ZERO,
        )
        .await
        .expect_err("a custom-named fresh create should not be retried");

        assert!(
            err.downcast_ref::<LangstarError>()
                .is_some_and(is_conflict_error),
            "the 409 should be returned, got: {err}"
        );
        create.assert_async().await;
    }

    #[tokio::test]
    async fn test_fresh_create_keeps_a_custom_name_and_does_not_retry() {
        assert_fresh_create_sends_one_create_under("custom-fresh-deployment").await;
    }

    #[tokio::test]
    async fn test_fresh_create_does_not_retry_a_name_that_lacks_the_release_hyphen() {
        // Starts with `release-integration-test` but not `release-integration-test-`.
        assert_fresh_create_sends_one_create_under("release-integration-testing").await;
    }

    #[test]
    fn test_deployment_config_for_release() {
        let config = TestDeploymentConfig::for_release_tests();
        let suffix = config
            .name
            .strip_prefix("release-integration-test-")
            .expect("Release name should start with release-integration-test-");
        assert_eq!(suffix.len(), 12, "suffix should be 12 hex characters");
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(
            config.name,
            TestDeploymentConfig::for_release_tests().name,
            "two release configs should get different names"
        );
        // name_prefix is None for release tests - always create fresh
        assert_eq!(
            config.name_prefix, None,
            "Release config should have no name_prefix (always create fresh)"
        );
        assert_eq!(config.branch, "main");
        assert!(config.config_path.contains("langgraph.json"));
    }
}
