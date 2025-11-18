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
    /// Create a new test deployment and wait for it to be READY
    ///
    /// This function:
    /// 1. Generates a unique deployment name with timestamp
    /// 2. Runs `langstar graph create --wait` to create deployment
    /// 3. Waits for deployment to reach READY status (auto-discovers integration_id)
    /// 4. Returns deployment info for use in tests
    ///
    /// # Prerequisites
    ///
    /// Requires at least one GitHub deployment to exist (for integration_id discovery).
    /// If this is your first deployment, create it via LangSmith UI first.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Required environment variables not set (LANGSMITH_API_KEY, LANGCHAIN_WORKSPACE_ID)
    /// - No existing GitHub deployments found (for integration_id)
    /// - Deployment creation fails
    /// - Deployment doesn't reach READY status within timeout
    pub fn create() -> Self {
        Self::check_env_vars();

        // Generate unique deployment name with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let deployment_name = format!("test-deployment-{}", timestamp);

        println!("\n=================================================");
        println!("🚀 Creating test deployment: {}", deployment_name);
        println!("   (Will auto-discover GitHub integration_id from existing deployments)");
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

impl Drop for TestDeployment {
    /// Automatically clean up deployment when TestDeployment goes out of scope
    fn drop(&mut self) {
        self.cleanup();
    }
}

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
