//! CLI Integration tests for deployment commands
//!
//! These tests verify that the langstar CLI correctly:
//! 1. Lists deployments
//! 2. Gets deployment details by ID
//! 3. Handles deployment filtering (by type, status, name)
//! 4. Produces valid JSON output
//!
//! **Prerequisites:**
//! 1. Valid LANGSMITH_API_KEY environment variable
//! 2. Valid LANGSMITH_WORKSPACE_ID environment variable
//!
//! Run with: cargo test --test deployment_command_test
//!
//! **Note:** These tests use the Control Plane API, not the Agent Server API.
//! Deployments are organization-level resources managed via the Control Plane.

use assert_cmd::Command;
use escargot::CargoBuild;
use std::sync::OnceLock;

mod common;
use common::fixtures::TestDeployment;

/// Shared test deployment
static TEST_DEPLOYMENT: OnceLock<TestDeployment> = OnceLock::new();

/// Helper function to get a CLI command builder
fn langstar_cmd() -> Command {
    let bin = CargoBuild::new()
        .bin("langstar")
        .run()
        .expect("Failed to build langstar binary")
        .path()
        .to_owned();
    Command::new(bin)
}

/// Helper to check if environment variables are set
fn check_env() -> bool {
    let langsmith_key = std::env::var("LANGSMITH_API_KEY").ok();
    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID").ok();

    langsmith_key.is_some()
        && workspace_id.is_some()
        && !langsmith_key.as_ref().unwrap().is_empty()
        && !workspace_id.as_ref().unwrap().is_empty()
}

/// Helper to get or create test deployment
/// Returns None if environment variables are not set (tests will be skipped)
fn get_test_deployment() -> Option<&'static TestDeployment> {
    if !check_env() {
        return None;
    }

    Some(TEST_DEPLOYMENT.get_or_init(|| {
        println!("\n📦 Initializing test deployment for deployment tests...");
        TestDeployment::create()
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// Deployment List Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_basic() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing basic deployment list command");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Command output:\n{}", stdout);

    // Output should contain deployment table or "No deployments found"
    assert!(
        stdout.contains("Name") || stdout.contains("No deployments found"),
        "Output should contain table headers or no deployments message"
    );

    println!("✓ CLI successfully listed deployments");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_with_limit() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment list with --limit flag");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--limit", "5"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✓ CLI successfully handled --limit parameter");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_json_output() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment list with JSON output");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--format", "json"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("JSON output:\n{}", stdout);

    // Should be parseable as JSON
    let result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(result.is_ok(), "Output should be valid JSON");

    // JSON should have expected structure
    let json = result.unwrap();
    assert!(
        json.get("resources").is_some(),
        "JSON should contain 'resources' field"
    );
    assert!(
        json.get("offset").is_some(),
        "JSON should contain 'offset' field"
    );

    println!("✓ CLI successfully output JSON format");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_filter_by_type() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment list with --deployment-type filter");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--deployment-type", "dev_free"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✓ CLI successfully filtered by deployment type");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_filter_by_status() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment list with --status filter");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--status", "READY"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✓ CLI successfully filtered by status");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_filter_by_name() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment list with --name-contains filter");

    let mut cmd = langstar_cmd();
    // Use a common substring that might appear in deployment names
    cmd.args(["deployment", "list", "--name-contains", "test"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    println!("✓ CLI successfully filtered by name");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_invalid_type() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment list with invalid --deployment-type");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--deployment-type", "invalid_type"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Command should fail for invalid type"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid") || stderr.contains("invalid"),
        "Error should mention invalid type"
    );

    println!("✓ CLI correctly rejected invalid deployment type");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_list_invalid_status() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment list with invalid --status");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--status", "INVALID_STATUS"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Command should fail for invalid status"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid") || stderr.contains("invalid"),
        "Error should mention invalid status"
    );

    println!("✓ CLI correctly rejected invalid status");
}

// ═══════════════════════════════════════════════════════════════════════════
// Deployment Get Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_get_basic() {
    let Some(deployment) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    println!(
        "Testing deployment get command for deployment: {}",
        deployment.id
    );

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "get", &deployment.id]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Command output:\n{}", stdout);

    // Output should contain deployment info
    assert!(
        stdout.contains(&deployment.name) || stdout.contains(&deployment.id),
        "Output should contain deployment name or ID"
    );

    println!("✓ CLI successfully retrieved deployment details");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_get_json_output() {
    let Some(deployment) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    println!("Testing deployment get with JSON output");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "get", &deployment.id, "--format", "json"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("JSON output:\n{}", stdout);

    // Should be parseable as JSON
    let result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(result.is_ok(), "Output should be valid JSON");

    // JSON should have expected fields
    let json = result.unwrap();
    assert!(json.get("id").is_some(), "JSON should contain 'id' field");
    assert!(
        json.get("name").is_some(),
        "JSON should contain 'name' field"
    );
    assert!(
        json.get("status").is_some(),
        "JSON should contain 'status' field"
    );

    println!("✓ CLI successfully output JSON format for deployment get");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_get_invalid_id() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing deployment get with invalid ID");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "get", "00000000-0000-0000-0000-000000000000"]);

    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Command should fail for invalid ID"
    );

    println!("✓ CLI correctly handled invalid deployment ID");
}

// ═══════════════════════════════════════════════════════════════════════════
// Help and Usage Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_deployment_commands_help() {
    println!("Testing deployment command help output");

    // Test main deployment help
    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "--help"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("list"),
        "Help should mention 'list' subcommand"
    );
    assert!(
        stdout.contains("get"),
        "Help should mention 'get' subcommand"
    );
    assert!(
        stdout.contains("create"),
        "Help should mention 'create' subcommand"
    );
    assert!(
        stdout.contains("delete"),
        "Help should mention 'delete' subcommand"
    );

    // Test deployment list help
    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--help"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("limit"),
        "Help should mention --limit option"
    );
    assert!(
        stdout.contains("offset"),
        "Help should mention --offset option"
    );
    assert!(
        stdout.contains("deployment-type"),
        "Help should mention --deployment-type option"
    );
    assert!(
        stdout.contains("status"),
        "Help should mention --status option"
    );

    // Test deployment get help
    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "get", "--help"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("deployment_id") || stdout.contains("DEPLOYMENT_ID"),
        "Help should mention deployment_id parameter"
    );

    // Test deployment create help
    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "create", "--help"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name"), "Help should mention --name option");
    assert!(
        stdout.contains("source"),
        "Help should mention --source option"
    );

    // Test deployment delete help
    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "delete", "--help"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("yes") || stdout.contains("-y"),
        "Help should mention --yes or -y option"
    );

    println!("✓ All deployment help commands work correctly");
}

// ═══════════════════════════════════════════════════════════════════════════
// Workflow Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_workflow_list_then_get() {
    if !check_env() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("\n==================================================");
    println!("Test: Deployment Workflow (List then Get)");
    println!("==================================================\n");

    // Step 1: List deployments to find available IDs
    println!("Step 1: Listing deployments");
    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "list", "--format", "json", "--limit", "5"]);

    let output = cmd.output().expect("Failed to list deployments");
    assert!(
        output.status.success(),
        "List command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("List output:\n{}", stdout);

    // Parse JSON to get first deployment ID
    let response: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");
    let resources = response
        .get("resources")
        .and_then(|r| r.as_array())
        .expect("Should have resources array");

    if resources.is_empty() {
        println!("⚠ No deployments found - skipping get test");
        return;
    }

    let first_deployment_id = resources[0]["id"]
        .as_str()
        .expect("Should have id field")
        .to_string();
    let first_deployment_name = resources[0]["name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    println!(
        "✓ Found deployment: {} ({})",
        first_deployment_name, first_deployment_id
    );

    // Step 2: Get the deployment details
    println!(
        "\nStep 2: Getting deployment details for '{}'",
        first_deployment_id
    );
    let mut cmd = langstar_cmd();
    cmd.args([
        "deployment",
        "get",
        &first_deployment_id,
        "--format",
        "json",
    ]);

    let output = cmd.output().expect("Failed to get deployment");
    assert!(
        output.status.success(),
        "Get command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Get output:\n{}", stdout);

    // Verify we got the same deployment
    let deployment: serde_json::Value =
        serde_json::from_str(&stdout).expect("Should be valid JSON");
    assert_eq!(
        deployment["id"].as_str(),
        Some(first_deployment_id.as_str()),
        "Should get the same deployment"
    );

    println!("✓ Successfully retrieved deployment details");

    println!("\n==================================================");
    println!("✓ Workflow test completed successfully");
    println!("==================================================\n");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_secrets_redacted() {
    let Some(deployment) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    println!("Testing that deployment secrets are redacted in output");

    let mut cmd = langstar_cmd();
    cmd.args(["deployment", "get", &deployment.id, "--format", "json"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON to check for secret values
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Should be valid JSON");

    // If there are secrets, they should be redacted
    if let Some(secrets) = json.get("secrets").and_then(|s| s.as_array()) {
        for secret in secrets {
            if let Some(value) = secret.get("value").and_then(|v| v.as_str()) {
                assert!(
                    value.contains("<redacted>") || value.is_empty(),
                    "Secret values should be redacted, got: {}",
                    value
                );
            }
        }
    }

    // Also check source_config for integration_id which might be sensitive
    // It's OK for integration_id to be present as it's not a secret value

    println!("✓ Deployment output has appropriate secret handling");
}
