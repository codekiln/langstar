//! Test fixtures for integration tests
//!
//! This module provides shared test infrastructure for managing LangGraph deployments
//! during integration tests. Tests use these fixtures to create temporary deployments,
//! run tests against them, and clean up afterwards.

use assert_cmd::Command;
use escargot::CargoBuild;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Test deployment that automatically manages lifecycle
pub struct TestDeployment {
    pub id: String,
    pub name: String,
}

impl TestDeployment {
    /// Create or reuse a test deployment
    ///
    /// This function:
    /// 1. Checks for existing READY test deployments (name starts with "test-deployment-")
    /// 2. Reuses the most recent one if found
    /// 3. Creates a new deployment if none exist
    /// 4. Waits for deployment to reach READY status
    /// 5. Returns deployment info for use in tests
    ///
    /// # Prerequisites
    ///
    /// Requires GitHub integration ID to be available via one of:
    /// - Environment variable: LANGGRAPH_GITHUB_INTEGRATION_ID
    /// - Config file: github_integration_id field
    /// - Auto-discovery: At least one existing GitHub deployment
    ///
    /// For first-time setup, create initial deployment via LangSmith UI or set integration_id.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Required environment variables not set (LANGSMITH_API_KEY, LANGCHAIN_WORKSPACE_ID)
    /// - GitHub integration ID cannot be determined (not in config/env and no existing deployments)
    /// - Deployment creation fails
    /// - Deployment doesn't reach READY status within timeout
    pub fn create() -> Self {
        Self::check_env_vars();

        // Try to find and reuse existing test deployment
        if let Some(existing) = Self::find_active_test_deployment() {
            println!("\n=================================================");
            println!("♻️  Reusing existing test deployment");
            println!("   Name: {}", existing.name);
            println!("   ID: {}", existing.id);
            println!("=================================================\n");
            return existing;
        }

        // No existing deployment found, create new one
        println!("\n=================================================");
        println!("🔍 No existing test deployment found");
        println!("   Creating new deployment...");
        println!("=================================================\n");

        Self::create_new_deployment()
    }

    /// Find an existing active test deployment
    ///
    /// Queries for deployments matching:
    /// - Name starts with "test-deployment-"
    /// - Status is READY
    /// - Source is github
    ///
    /// Returns the most recent matching deployment, or None if no matches found.
    fn find_active_test_deployment() -> Option<Self> {
        // Build langstar binary
        let bin = CargoBuild::new()
            .bin("langstar")
            .run()
            .ok()?
            .path()
            .to_owned();

        // Query deployments with filter for test deployments
        let mut cmd = Command::new(&bin);
        cmd.args([
            "graph",
            "list",
            "--name-contains",
            "test-deployment-",
            "--status",
            "READY",
            "--format",
            "json",
        ]);

        let output = cmd.output().ok()?;

        if !output.status.success() {
            eprintln!(
                "⚠️  Warning: Failed to query existing deployments.\nStderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON output
        let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;

        // Get deployments array
        let deployments = json.as_array()?;

        if deployments.is_empty() {
            return None;
        }

        // Find most recent deployment (first in list, as API returns most recent first)
        let deployment = &deployments[0];

        let id = deployment["id"].as_str()?.to_string();
        let name = deployment["name"].as_str()?.to_string();

        Some(Self { id, name })
    }

    /// Create a new test deployment
    ///
    /// This is the original creation logic, now separated from the reuse logic.
    fn create_new_deployment() -> Self {
        // Generate unique deployment name with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let deployment_name = format!("test-deployment-{}", timestamp);

        println!("\n=================================================");
        println!("🚀 Creating test deployment: {}", deployment_name);
        println!("   (Integration ID: CLI flag > env/config > auto-discovery)");
        println!("=================================================\n");

        // Build langstar binary
        let bin = CargoBuild::new()
            .bin("langstar")
            .run()
            .expect("Failed to build langstar binary")
            .path()
            .to_owned();

        // Create deployment with --wait flag
        // Note: integration_id will be auto-discovered from existing GitHub deployments
        let mut cmd = Command::new(&bin);
        cmd.args([
            "graph",
            "create",
            "--name",
            &deployment_name,
            "--source",
            "github",
            "--repo-url",
            "https://github.com/langchain-ai/langgraph-example",
            "--branch",
            "main",
            "--deployment-type",
            "dev_free",
            "--wait",
            "--format",
            "json",
        ]);

        let start = Instant::now();
        let output = cmd.output().expect("Failed to execute deployment creation");

        if !output.status.success() {
            panic!(
                "Failed to create test deployment.\nStdout: {}\nStderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract deployment ID from JSON output
        let json_start = stdout
            .find('{')
            .expect("Should contain JSON object in output");
        let json_str = &stdout[json_start..];
        let json: serde_json::Value =
            serde_json::from_str(json_str).expect("Should return valid JSON");
        let deployment_id = json["id"]
            .as_str()
            .expect("Should have 'id' field")
            .to_string();

        let elapsed = start.elapsed();
        println!("\n✅ Test deployment created successfully!");
        println!("   Name: {}", deployment_name);
        println!("   ID: {}", deployment_id);
        println!("   Creation time: {:.1}s", elapsed.as_secs_f32());
        println!("=================================================\n");

        Self {
            id: deployment_id,
            name: deployment_name,
        }
    }

    /// Delete the test deployment
    ///
    /// This function runs `langstar graph delete --yes` to remove the deployment
    /// without requiring confirmation.
    ///
    /// # Panics
    ///
    /// Panics if deletion fails
    pub fn cleanup(&self) {
        println!("\n=================================================");
        println!("🧹 Cleaning up test deployment: {}", self.name);
        println!("=================================================\n");

        // Build langstar binary
        let bin = CargoBuild::new()
            .bin("langstar")
            .run()
            .expect("Failed to build langstar binary")
            .path()
            .to_owned();

        // Delete deployment
        let mut cmd = Command::new(&bin);
        cmd.args(["graph", "delete", &self.id, "--yes"]);

        let output = cmd.output().expect("Failed to execute deployment deletion");

        if !output.status.success() {
            eprintln!(
                "⚠️  Warning: Failed to delete test deployment.\nStdout: {}\nStderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            println!("✅ Test deployment deleted successfully");
            println!("=================================================\n");
        }
    }

    /// Check that required environment variables are set
    fn check_env_vars() {
        let api_key = std::env::var("LANGSMITH_API_KEY")
            .expect("LANGSMITH_API_KEY environment variable must be set for integration tests");
        let workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID").expect(
            "LANGCHAIN_WORKSPACE_ID environment variable must be set for integration tests",
        );

        if api_key.is_empty() || workspace_id.is_empty() {
            panic!("LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID must not be empty");
        }

        println!("✓ Environment variables validated");
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
        assert!(deployment.name.starts_with("test-deployment-"));

        println!(
            "Test deployment created: {} ({})",
            deployment.name, deployment.id
        );

        // Deployment will be automatically cleaned up when it goes out of scope
    }
}
