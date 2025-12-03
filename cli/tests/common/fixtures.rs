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
    /// # GitHub Integration ID Discovery
    ///
    /// When creating a new deployment, the GitHub integration ID is determined via:
    /// 1. **Environment variable**: LANGGRAPH_GITHUB_INTEGRATION_ID (highest priority)
    /// 2. **Auto-discovery**: Query existing deployments to extract integration_id (fallback)
    /// 3. **Fail with helpful message**: If neither succeeds
    ///
    /// This ensures tests work in CI environments where existing deployments provide
    /// the integration ID, without requiring the env var to be set.
    ///
    /// # Prerequisites
    ///
    /// Required environment variables:
    /// - LANGSMITH_API_KEY: LangSmith API key for authentication
    /// - LANGSMITH_WORKSPACE_ID: Workspace ID for deployment creation
    ///
    /// For first-time setup in a clean environment:
    /// - Set LANGGRAPH_GITHUB_INTEGRATION_ID, OR
    /// - Create at least one GitHub deployment via LangSmith UI
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Required environment variables not set (LANGSMITH_API_KEY, LANGSMITH_WORKSPACE_ID)
    /// - GitHub integration ID cannot be determined (not in env var and no existing deployments)
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
        let bin = match CargoBuild::new().bin("langstar").run() {
            Ok(bin) => bin.path().to_owned(),
            Err(e) => {
                eprintln!(
                    "⚠️  Warning: Failed to build langstar binary for deployment query.\nError: {}",
                    e
                );
                return None;
            }
        };

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

        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!(
                    "⚠️  Warning: Failed to execute deployment query command.\nError: {}",
                    e
                );
                return None;
            }
        };

        if !output.status.success() {
            eprintln!(
                "⚠️  Warning: Deployment query command failed.\nStderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON output
        let json: serde_json::Value = match serde_json::from_str(&stdout) {
            Ok(json) => json,
            Err(e) => {
                eprintln!(
                    "⚠️  Warning: Failed to parse deployment query JSON.\nError: {}\nOutput: {}",
                    e, stdout
                );
                return None;
            }
        };

        // Get deployments array from the "resources" field
        let deployments = match json["resources"].as_array() {
            Some(arr) => arr,
            None => {
                eprintln!(
                    "⚠️  Warning: JSON response missing 'resources' array.\nJSON keys: {:?}",
                    json.as_object().map(|obj| obj.keys().collect::<Vec<_>>())
                );
                return None;
            }
        };

        if deployments.is_empty() {
            return None;
        }

        // Find most recent deployment (first in list, as API returns most recent first)
        let deployment = &deployments[0];

        let id = match deployment["id"].as_str() {
            Some(id) => id.to_string(),
            None => {
                eprintln!(
                    "⚠️  Warning: Deployment missing 'id' field.\nAvailable fields: {:?}",
                    deployment
                        .as_object()
                        .map(|obj| obj.keys().collect::<Vec<_>>())
                );
                return None;
            }
        };

        let name = match deployment["name"].as_str() {
            Some(name) => name.to_string(),
            None => {
                eprintln!(
                    "⚠️  Warning: Deployment missing 'name' field.\nAvailable fields: {:?}",
                    deployment
                        .as_object()
                        .map(|obj| obj.keys().collect::<Vec<_>>())
                );
                return None;
            }
        };

        Some(Self { id, name })
    }

    /// Query GitHub integrations API to find the first available integration ID
    ///
    /// This is used as a fallback when LANGGRAPH_GITHUB_INTEGRATION_ID is not set
    /// and no existing deployments are found for auto-discovery.
    ///
    /// Returns the first GitHub integration ID found, or None if none exist or query fails.
    fn query_github_integration_id() -> Option<String> {
        println!("\n🔍 Querying GitHub integrations API...");

        // Build langstar binary
        let bin = match CargoBuild::new().bin("langstar").run() {
            Ok(bin) => bin.path().to_owned(),
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to build langstar binary.\nError: {}", e);
                return None;
            }
        };

        // Query GitHub integrations using the integrations command
        // This requires the integrations command to be implemented
        // For now, we'll try to list deployments and extract integration_id
        let mut cmd = Command::new(&bin);
        cmd.args(["graph", "list", "--limit", "100", "--format", "json"]);

        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!(
                    "⚠️  Warning: Failed to query deployments for integration ID.\nError: {}",
                    e
                );
                return None;
            }
        };

        if !output.status.success() {
            eprintln!(
                "⚠️  Warning: Deployments query failed.\nStderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON to extract integration_id from any GitHub deployment
        let json: serde_json::Value = match serde_json::from_str(&stdout) {
            Ok(json) => json,
            Err(e) => {
                eprintln!(
                    "⚠️  Warning: Failed to parse JSON.\nError: {}\nOutput: {}",
                    e, stdout
                );
                return None;
            }
        };

        // Look through all deployments for a GitHub source with integration_id
        if let Some(deployments) = json["resources"].as_array() {
            for deployment in deployments {
                // Check if this is a GitHub deployment
                if let Some(source) = deployment["source"].as_str() {
                    if source == "github" {
                        // Try to extract integration_id from source_config
                        if let Some(source_config) = deployment["source_config"].as_object() {
                            if let Some(integration_id) = source_config
                                .get("integration_id")
                                .and_then(|v| v.as_str())
                            {
                                println!("✓ Found GitHub integration ID: {}", integration_id);
                                return Some(integration_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        eprintln!("⚠️  Warning: No GitHub deployments found with integration_id");
        None
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
        println!("   (Integration ID: env var > API query > fail)");
        println!("=================================================\n");

        // Determine integration ID
        let integration_id = std::env::var("LANGGRAPH_GITHUB_INTEGRATION_ID")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                println!("LANGGRAPH_GITHUB_INTEGRATION_ID not set, querying API...");
                Self::query_github_integration_id()
            })
            .expect(
                "GitHub integration ID required but not found. Please either:\n\
                1. Set LANGGRAPH_GITHUB_INTEGRATION_ID environment variable\n\
                2. Create at least one GitHub deployment to enable auto-discovery\n\
                3. Set up GitHub integration via LangSmith UI",
            );

        println!("Using GitHub integration ID: {}", integration_id);

        // Build langstar binary
        let bin = CargoBuild::new()
            .bin("langstar")
            .run()
            .expect("Failed to build langstar binary")
            .path()
            .to_owned();

        // Create deployment with --wait flag and explicit --integration-id
        let mut cmd = Command::new(&bin);
        cmd.args([
            "graph",
            "create",
            "--name",
            &deployment_name,
            "--source",
            "github",
            "--repo-url",
            "https://github.com/codekiln/langstar",
            "--branch",
            "main",
            "--integration-id",
            &integration_id,
            "--config-path",
            "tests/fixtures/test-graph-deployment/langgraph.json",
            "--deployment-type",
            "dev",
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
    #[allow(dead_code)]
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
        let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID").expect(
            "LANGSMITH_WORKSPACE_ID environment variable must be set for integration tests",
        );

        if api_key.is_empty() || workspace_id.is_empty() {
            panic!("LANGSMITH_API_KEY and LANGSMITH_WORKSPACE_ID must not be empty");
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
