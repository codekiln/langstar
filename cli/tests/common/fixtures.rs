//! Test fixtures for integration tests
//!
//! This module provides shared test infrastructure for managing LangGraph deployments
//! during integration tests. It wraps the SDK test utilities and provides a synchronous
//! API for use in CLI integration tests.
//!
//! # Design
//!
//! This module uses the SDK directly via `langstar_sdk::test_utils` instead of shelling
//! out to CLI commands. This approach:
//!
//! 1. **Eliminates code duplication** - Uses the same logic as SDK tests
//! 2. **Fixes the in-progress bug** - Filters by name, not status
//! 3. **Removes env var workarounds** - Uses `find_integration_for_repo()` API
//! 4. **Enables proper wait behavior** - Can wait for in-progress deployments
//!
//! # Usage
//!
//! ```ignore
//! let deployment = TestDeployment::create();
//! // Run tests using deployment.id and deployment.name
//! // Deployment is reused across test runs for faster iteration
//! ```

use langstar_sdk::AuthConfig;
use langstar_sdk::test_utils::{TestDeploymentConfig, get_or_create_deployment};

/// Test deployment that manages lifecycle using SDK utilities
pub struct TestDeployment {
    /// Deployment UUID
    pub id: String,
    /// Deployment name
    pub name: String,
}

impl TestDeployment {
    /// Create or reuse a test deployment
    ///
    /// This function uses the SDK's `get_or_create_deployment()` utility which:
    /// 1. Looks for existing deployment by name (any status, not just READY)
    /// 2. Waits for it if it's in progress
    /// 3. Creates a new deployment if none exists
    /// 4. Returns deployment info for use in tests
    ///
    /// # Prerequisites
    ///
    /// Requires environment variables:
    /// - LANGSMITH_API_KEY: Valid API key with write permissions
    /// - LANGSMITH_WORKSPACE_ID: Workspace ID for deployments
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Required environment variables not set
    /// - GitHub integration cannot be found for the repository
    /// - Deployment creation fails
    /// - Deployment doesn't reach READY status within timeout
    pub fn create() -> Self {
        Self::create_with_config(TestDeploymentConfig {
            name: format!("test-deployment-cli-{}", std::process::id()),
            ..Default::default()
        })
    }

    /// Create or reuse a test deployment with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the test deployment
    pub fn create_with_config(config: TestDeploymentConfig) -> Self {
        Self::check_env_vars();

        println!("\n=================================================");
        println!("Getting or creating test deployment: {}", config.name);
        println!(
            "   Repository: {}/{}",
            config.repository_owner, config.repository_name
        );
        println!("   Branch: {}", config.branch);
        println!("=================================================\n");

        // Create tokio runtime for async SDK calls
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        let result = runtime.block_on(async {
            // Create SDK client
            let auth = AuthConfig::from_env()
                .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");
            let client =
                langstar_sdk::LangchainClient::new(auth).expect("Failed to create LangchainClient");

            // Use shared get_or_create_deployment utility
            get_or_create_deployment(&client, &config).await
        });

        let (deployment_id, _revision_id) = result.expect("Failed to get or create deployment");

        println!("\n=================================================");
        println!("Test deployment ready");
        println!("   Name: {}", config.name);
        println!("   ID: {}", deployment_id);
        println!("=================================================\n");

        Self {
            id: deployment_id,
            name: config.name,
        }
    }

    /// Create a test deployment with default "test-deployment-" prefix
    ///
    /// Uses a timestamp-based name for uniqueness when running the full lifecycle.
    #[allow(dead_code)]
    pub fn create_with_timestamp() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let deployment_name = format!("test-deployment-{}", timestamp);

        Self::create_with_config(TestDeploymentConfig {
            name: deployment_name,
            ..Default::default()
        })
    }

    /// Delete the test deployment
    ///
    /// This function deletes the deployment using the SDK.
    ///
    /// # Panics
    ///
    /// Panics if deletion fails
    #[allow(dead_code)]
    pub fn cleanup(&self) {
        println!("\n=================================================");
        println!("Cleaning up test deployment: {}", self.name);
        println!("=================================================\n");

        // Create tokio runtime for async SDK calls
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        let result = runtime.block_on(async {
            // Create SDK client
            let auth = AuthConfig::from_env()
                .expect("LANGSMITH_API_KEY and workspace ID must be set for integration tests");
            let client =
                langstar_sdk::LangchainClient::new(auth).expect("Failed to create LangchainClient");

            client.deployments().delete(&self.id).await
        });

        match result {
            Ok(()) => {
                println!("Test deployment deleted successfully");
                println!("=================================================\n");
            }
            Err(e) => {
                eprintln!("Warning: Failed to delete test deployment.\nError: {}", e);
            }
        }
    }

    /// Check that required environment variables are set
    fn check_env_vars() {
        let api_key = std::env::var("LANGSMITH_API_KEY")
            .expect("LANGSMITH_API_KEY environment variable must be set for integration tests");
        let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID").expect(
            "LANGSMITH_WORKSPACE_ID environment variable must be set for integration tests",
        );

        if api_key.is_empty() || workspace_id.is_empty() {
            panic!("LANGSMITH_API_KEY and LANGSMITH_WORKSPACE_ID must not be empty");
        }

        println!("Environment variables validated");
        println!("  Workspace: {}", workspace_id);
    }
}

// NOTE: Automatic cleanup is disabled to allow deployment reuse across test runs.
// Test deployments are now reused to save API quota and speed up test startup.
// To manually clean up old test deployments, use: langstar graph delete <id> --yes
//
// impl Drop for TestDeployment {
//     /// Automatically clean up deployment when TestDeployment goes out of scope
//     fn drop(&mut self) {
//         self.cleanup();
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only run manually - creates real deployment
    fn test_fixture_lifecycle() {
        // This test validates the fixture itself works correctly
        let deployment = TestDeployment::create();

        // Verify deployment info
        assert!(!deployment.id.is_empty());
        assert!(!deployment.name.is_empty());

        println!(
            "Test deployment created: {} ({})",
            deployment.name, deployment.id
        );

        // Deployment will be automatically cleaned up when it goes out of scope
    }
}
