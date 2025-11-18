mod common;

use assert_cmd::Command;
use common::fixtures::TestDeployment;
use escargot::CargoBuild;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared test deployment for all assistant tests
static TEST_DEPLOYMENT: OnceLock<TestDeployment> = OnceLock::new();

/// CLI Integration tests for assistant commands
///
/// These tests verify that the langstar CLI correctly:
/// 1. Discovers LangGraph deployments
/// 2. Creates, gets, updates, and deletes assistants
/// 3. Handles deployment targeting via --deployment flag
/// 4. Outputs JSON and table formats
///
/// **Test Infrastructure:**
/// These tests use a self-managed test deployment:
/// - Created automatically before first test runs
/// - Shared across all tests in this suite
/// - Cleaned up automatically when tests complete
/// - Uses `langstar graph create/delete` commands for lifecycle management
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY environment variable (used for both LangSmith and LangGraph)
/// 2. Valid LANGCHAIN_WORKSPACE_ID environment variable
///
/// **Known Issues:**
/// - List command blocked by #127 (405 Method Not Allowed)
/// - Search command blocked by #128 (JSON decode error)
///
/// Run with: cargo test --test assistant_command_test -- --nocapture --test-threads=1
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

/// Helper to generate unique test names
fn generate_test_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    format!("{}-{}", prefix, timestamp)
}

/// Get or create the shared test deployment
///
/// This function ensures a test deployment exists for all tests to use.
/// The deployment is created once and reused across all tests in this suite.
///
/// Returns: (deployment_name, graph_id)
fn get_test_deployment() -> Option<(String, String)> {
    // Check if environment variables are set
    let langsmith_key = std::env::var("LANGSMITH_API_KEY").ok()?;
    let workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID").ok()?;

    if langsmith_key.is_empty() || workspace_id.is_empty() {
        return None;
    }

    // Get or create test deployment
    let deployment = TEST_DEPLOYMENT.get_or_init(|| {
        println!("\n📦 Initializing test deployment for assistant tests...");
        TestDeployment::create()
    });

    println!(
        "Using test deployment: {} ({})",
        deployment.name, deployment.id
    );
    Some((deployment.name.clone(), "test_graph".to_string()))
}

#[test]
fn test_assistant_create_basic() {
    println!("==================================================");
    println!("Test: Assistant Create (Basic)");
    println!("==================================================\n");

    let Some((deployment_name, _graph_name)) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        println!("Set LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID to run this test");
        return;
    };
    let assistant_name = generate_test_name("cli-test-assistant");

    println!("Creating assistant: {}", assistant_name);

    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "create",
        "--deployment",
        &deployment_name,
        "--graph-id",
        "test_graph",
        "--name",
        &assistant_name,
    ]);

    let output = cmd.output().expect("Failed to execute command");

    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    // Should succeed
    assert!(
        output.status.success(),
        "Assistant create command should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Output should contain assistant info
    assert!(
        stdout.contains(&assistant_name) || stdout.contains("assistant_id"),
        "Output should contain assistant information"
    );

    println!("✓ CLI successfully created assistant");
    println!(
        "\nNote: Assistant '{}' created (cleanup needed)",
        assistant_name
    );
}

#[test]
#[ignore] // Blocked by #131 - Delete command has clap flag conflict
fn test_assistant_lifecycle() {
    println!("==================================================");
    println!("Test: Assistant Lifecycle (Create → Get → Update → Delete)");
    println!("==================================================\n");
    println!("⚠️  This test is blocked by issue #131:");
    println!("    Delete command has clap short flag conflict");
    println!("    https://github.com/codekiln/langstar/issues/131");
    println!("\n==================================================\n");

    let Some((deployment_name, _graph_name)) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        println!("Set LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID to run this test");
        return;
    };
    let assistant_name = generate_test_name("cli-lifecycle-test");

    // Step 1: Create
    println!("1. CREATE assistant: {}", assistant_name);
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "create",
        "--deployment",
        &deployment_name,
        "--graph-id",
        "test_graph",
        "--name",
        &assistant_name,
        "--format",
        "json",
    ]);

    let output = cmd.output().expect("Failed to create assistant");
    assert!(output.status.success(), "Create should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Create output: {}", stdout);

    // Extract JSON from output (skip info messages)
    let json_start = stdout.find('{').expect("Should contain JSON object");
    let json_str = &stdout[json_start..];

    let json: serde_json::Value = serde_json::from_str(json_str).expect("Should return valid JSON");
    let assistant_id = json["assistant_id"]
        .as_str()
        .expect("Should have assistant_id field");

    println!("✓ Created assistant ID: {}", assistant_id);

    // Step 2: Get
    println!("\n2. GET assistant: {}", assistant_id);
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "get",
        assistant_id,
        "--deployment",
        &deployment_name,
        "--format",
        "json",
    ]);

    let output = cmd.output().expect("Failed to get assistant");
    assert!(output.status.success(), "Get should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Get output: {}", stdout);

    let json_start = stdout.find('{').expect("Should contain JSON");
    let json: serde_json::Value =
        serde_json::from_str(&stdout[json_start..]).expect("Should return valid JSON");
    assert_eq!(
        json["assistant_id"].as_str().unwrap(),
        assistant_id,
        "Should return same assistant"
    );
    assert_eq!(
        json["name"].as_str().unwrap(),
        assistant_name,
        "Name should match"
    );

    println!("✓ Successfully fetched assistant");

    // Step 3: Update
    println!("\n3. UPDATE assistant: {}", assistant_id);
    let updated_name = format!("{}-updated", assistant_name);

    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "update",
        assistant_id,
        "--deployment",
        &deployment_name,
        "--name",
        &updated_name,
        "--format",
        "json",
    ]);

    let output = cmd.output().expect("Failed to update assistant");
    assert!(output.status.success(), "Update should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Update output: {}", stdout);

    let json_start = stdout.find('{').expect("Should contain JSON");
    let json: serde_json::Value =
        serde_json::from_str(&stdout[json_start..]).expect("Should return valid JSON");
    assert_eq!(
        json["name"].as_str().unwrap(),
        updated_name,
        "Name should be updated"
    );

    println!("✓ Successfully updated assistant");

    // Step 4: Delete
    println!("\n4. DELETE assistant: {}", assistant_id);
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "delete",
        assistant_id,
        "--deployment",
        &deployment_name,
        "--force", // Skip confirmation (use long form due to #131)
    ]);

    let output = cmd.output().expect("Failed to delete assistant");

    if !output.status.success() {
        println!("Delete failed!");
        println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success(), "Delete should succeed");

    println!("✓ Successfully deleted assistant");

    // Step 5: Verify deletion (get should fail)
    println!("\n5. VERIFY deletion (get should fail)");
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "get",
        assistant_id,
        "--deployment",
        &deployment_name,
    ]);

    let output = cmd.output().expect("Failed to execute get command");
    assert!(!output.status.success(), "Get should fail after deletion");

    println!("✓ Confirmed assistant no longer exists");

    println!("\n==================================================");
    println!("✓ All lifecycle tests passed!");
    println!("==================================================\n");
}

#[test]
#[ignore] // Blocked by #131 - Delete command needed for cleanup
fn test_assistant_output_formats() {
    println!("==================================================");
    println!("Test: Output Formats (JSON vs Table)");
    println!("==================================================\n");
    println!("⚠️  This test is blocked by issue #131:");
    println!("    Delete command needed for cleanup");
    println!("    https://github.com/codekiln/langstar/issues/131");
    println!("\n==================================================\n");

    let Some((deployment_name, _graph_name)) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        println!("Set LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID to run this test");
        return;
    };
    let assistant_name = generate_test_name("cli-format-test");

    // Create assistant
    println!("Creating test assistant...");
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "create",
        "--deployment",
        &deployment_name,
        "--graph-id",
        "test_graph",
        "--name",
        &assistant_name,
        "--format",
        "json",
    ]);

    let output = cmd.output().expect("Failed to create assistant");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = stdout.find('{').expect("Should contain JSON");
    let json: serde_json::Value =
        serde_json::from_str(&stdout[json_start..]).expect("Should be valid JSON");
    let assistant_id = json["assistant_id"].as_str().unwrap();

    // Test JSON format
    println!("\n1. Testing JSON format");
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "get",
        assistant_id,
        "--deployment",
        &deployment_name,
        "--format",
        "json",
    ]);

    let output = cmd.output().expect("Failed to get assistant");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = stdout.find('{').expect("Should contain JSON");
    let json: serde_json::Value =
        serde_json::from_str(&stdout[json_start..]).expect("JSON format should be valid");
    assert!(json["assistant_id"].is_string());
    assert!(json["name"].is_string());
    assert!(json["graph_id"].is_string());

    println!("✓ JSON format valid");

    // Test table format (default)
    println!("\n2. Testing table format (default)");
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "get",
        assistant_id,
        "--deployment",
        &deployment_name,
    ]);

    let output = cmd.output().expect("Failed to get assistant");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Table output should contain the assistant name and some formatting
    assert!(stdout.contains(&assistant_name) || stdout.contains(assistant_id));

    println!("✓ Table format displayed");

    // Cleanup
    println!("\n3. Cleanup");
    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "delete",
        assistant_id,
        "--deployment",
        &deployment_name,
        "--force",
    ]);
    let output = cmd.output().expect("Failed to delete");
    assert!(output.status.success());

    println!("✓ Cleanup complete");

    println!("\n==================================================");
    println!("✓ All format tests passed!");
    println!("==================================================\n");
}

#[test]
fn test_deployment_discovery_workflow() {
    println!("==================================================");
    println!("Test: Deployment Discovery Workflow");
    println!("==================================================\n");

    let Some((_deployment_name, _graph_name)) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        println!("Set LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID to run this test");
        return;
    };

    // Get the test deployment info
    let deployment = TEST_DEPLOYMENT
        .get()
        .expect("Test deployment should be initialized");

    // Step 1: List deployments
    println!("1. List available deployments");
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list"]);

    let output = cmd.output().expect("Failed to list deployments");
    assert!(output.status.success(), "Graph list should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Deployments available:");
    println!("{}", stdout);

    // Verify our test deployment is in the list
    assert!(
        stdout.contains(&deployment.name),
        "Should find test deployment '{}' in list",
        deployment.name
    );

    println!("✓ Test deployment discovered: {}", deployment.name);

    println!("\n==================================================");
    println!("✓ Deployment discovery workflow passed!");
    println!("==================================================\n");
}

#[test]
fn test_error_handling_missing_deployment() {
    println!("==================================================");
    println!("Test: Error Handling - Missing Deployment Flag");
    println!("==================================================\n");

    let assistant_name = generate_test_name("error-test");

    println!("Attempting to create assistant without --deployment flag...");

    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "create",
        "--graph-id",
        "test_graph",
        "--name",
        &assistant_name,
    ]);

    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Should fail without --deployment flag"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Error output:\n{}", stderr);

    // Should mention deployment requirement
    assert!(
        stderr.contains("deployment") || stderr.contains("required"),
        "Error should mention deployment requirement"
    );

    println!("✓ Correctly rejected command without deployment");

    println!("\n==================================================");
    println!("✓ Error handling test passed!");
    println!("==================================================\n");
}

#[test]
fn test_error_handling_nonexistent_deployment() {
    println!("==================================================");
    println!("Test: Error Handling - Nonexistent Deployment");
    println!("==================================================\n");

    if get_test_deployment().is_none() {
        println!("Skipping test: Required environment variables not set");
        println!("Set LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID to run this test");
        return;
    }
    let assistant_name = generate_test_name("error-test");

    println!("Attempting to create assistant with nonexistent deployment...");

    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "create",
        "--deployment",
        "nonexistent-deployment-xyz-123",
        "--graph-id",
        "test_graph",
        "--name",
        &assistant_name,
    ]);

    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Should fail with nonexistent deployment"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Error output:\n{}", stderr);

    // Should mention deployment not found
    assert!(
        stderr.contains("not found") || stderr.contains("No deployment"),
        "Error should mention deployment not found"
    );

    println!("✓ Correctly rejected nonexistent deployment");

    println!("\n==================================================");
    println!("✓ Error handling test passed!");
    println!("==================================================\n");
}

// ==================================================
// TESTS BLOCKED BY KNOWN ISSUES
// ==================================================

#[test]
#[ignore] // Blocked by #127 - List endpoint returns 405
fn test_assistant_list() {
    println!("==================================================");
    println!("Test: Assistant List (BLOCKED BY #127)");
    println!("==================================================\n");
    println!("⚠️  This test is blocked by issue #127:");
    println!("    Assistant list endpoint returns 405 Method Not Allowed");
    println!("    https://github.com/codekiln/langstar/issues/127");
    println!("\n==================================================\n");

    let Some((deployment_name, _graph_name)) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        println!("Set LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID to run this test");
        return;
    };

    let mut cmd = langstar_cmd();
    cmd.args(["assistant", "list", "--deployment", &deployment_name]);

    let output = cmd.output().expect("Failed to execute command");

    // Currently fails with 405
    // Once #127 is fixed, this should succeed
    if output.status.success() {
        println!("✓ List command succeeded (issue #127 is fixed!)");
    } else {
        println!("✗ List command failed (expected until #127 is fixed)");
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
#[ignore] // Blocked by #128 - Search endpoint JSON decode error
fn test_assistant_search() {
    println!("==================================================");
    println!("Test: Assistant Search (BLOCKED BY #128)");
    println!("==================================================\n");
    println!("⚠️  This test is blocked by issue #128:");
    println!("    Assistant search endpoint returns unexpected JSON structure");
    println!("    https://github.com/codekiln/langstar/issues/128");
    println!("\n==================================================\n");

    let Some((deployment_name, _graph_name)) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        println!("Set LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID to run this test");
        return;
    };

    let mut cmd = langstar_cmd();
    cmd.args([
        "assistant",
        "search",
        "test",
        "--deployment",
        &deployment_name,
    ]);

    let output = cmd.output().expect("Failed to execute command");

    // Currently fails with JSON decode error
    // Once #128 is fixed, this should succeed
    if output.status.success() {
        println!("✓ Search command succeeded (issue #128 is fixed!)");
    } else {
        println!("✗ Search command failed (expected until #128 is fixed)");
        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
}
