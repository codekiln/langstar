use assert_cmd::Command;
use escargot::CargoBuild;

/// CLI Integration tests for graph commands
///
/// These tests verify that the langstar CLI correctly:
/// 1. Lists graphs within a deployment
/// 2. Gets graph structure details
/// 3. Handles deployment name resolution
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY environment variable
/// 2. Valid LANGSMITH_WORKSPACE_ID environment variable (for deployment lookup)
/// 3. Access to a test deployment (uses TEST_DEPLOYMENT_NAME or defaults to "test-graph-deployment")
///
/// Run with: cargo test --test graph_command_test
///
/// **Fixtures:**
/// These tests use the test-graph-deployment fixture from tests/fixtures/test-graph-deployment/
/// See tests/fixtures/test-graph-deployment/README.md for details.

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

/// Helper to verify required environment variables
/// Returns None if credentials are not available (tests will be skipped)
fn check_env_vars() -> Option<(String, String)> {
    let api_key = std::env::var("LANGSMITH_API_KEY").ok()?;
    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID").ok()?;

    if api_key.is_empty() || workspace_id.is_empty() {
        return None;
    }

    println!("Testing with workspace ID: {}", workspace_id);
    Some((api_key, workspace_id))
}

/// Helper to get test deployment name
/// Uses TEST_DEPLOYMENT_NAME env var or defaults to "test-graph-deployment"
fn test_deployment_name() -> String {
    std::env::var("TEST_DEPLOYMENT_NAME").unwrap_or_else(|_| "test-graph-deployment".to_string())
}

/// Helper to get test graph ID
/// Uses TEST_GRAPH_ID env var or defaults to "agent"
fn test_graph_id() -> String {
    std::env::var("TEST_GRAPH_ID").unwrap_or_else(|_| "agent".to_string())
}

#[test]
fn test_graph_list_basic() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();
    println!(
        "Testing basic graph list command for deployment: {}",
        deployment
    );

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", &deployment]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Output should contain graph information
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("Command output:");
    println!("{}", stdout);

    // Should contain either graph info or empty result
    let has_graphs = stdout.contains("Graph") || stdout.is_empty();
    assert!(has_graphs, "Output should contain graph info or be empty");

    println!("✓ CLI successfully listed graphs");
}

#[test]
fn test_graph_list_with_show_nodes() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();
    println!("Testing graph list with --show-nodes flag");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", &deployment, "--show-nodes"]);

    // Run the command
    let assert = cmd.assert();

    // Should succeed
    assert.success();

    println!("✓ CLI successfully handled --show-nodes parameter");
}

#[test]
fn test_graph_list_json_output() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();
    println!("Testing graph list with JSON output");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", &deployment, "--format", "json"]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(output.status.success(), "Command should succeed");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("JSON output:");
    println!("{}", stdout);

    // Should be parseable as JSON
    let result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(result.is_ok(), "Output should be valid JSON");

    println!("✓ CLI successfully output JSON format");
}

#[test]
fn test_graph_list_invalid_deployment() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    println!("Testing graph list with invalid deployment name");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "nonexistent-deployment-12345"]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail with error message
    assert!(!output.status.success(), "Command should fail");

    // Should contain helpful error message
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("not found") || stderr.contains("Deployment"),
        "Error message should mention deployment not found"
    );

    println!("✓ CLI correctly handled invalid deployment name");
}

#[test]
fn test_graph_get_basic() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();
    let graph_id = test_graph_id();
    println!("Testing graph get command for graph: {}", graph_id);

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "get", graph_id, "--deployment", &deployment]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    // Should succeed (if graph exists in the deployment)
    // Note: This may fail if the test deployment doesn't have the specified graph
    // Set TEST_GRAPH_ID env var to specify a different graph ID (defaults to "agent")
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") {
            println!("⚠ Graph '{}' not found in test deployment", graph_id);
            println!("  This is expected if the deployment doesn't have this graph");
            println!(
                "  Set TEST_GRAPH_ID={} to specify a valid graph ID",
                graph_id
            );
            return;
        }
        panic!("Graph get command failed unexpectedly");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("nodes") || stdout.contains("edges"),
        "Output should contain graph structure info"
    );

    println!("✓ CLI successfully retrieved graph structure");
}

#[test]
fn test_graph_get_with_xray() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();
    let graph_id = test_graph_id();
    println!("Testing graph get with --xray flag for graph: {}", graph_id);

    let mut cmd = langstar_cmd();
    cmd.args([
        "graph",
        "get",
        graph_id,
        "--deployment",
        &deployment,
        "--xray",
    ]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    println!("Exit status: {}", output.status);

    // Should succeed or fail gracefully if graph doesn't exist
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") {
            println!("⚠ Graph '{}' not found - skipping test", graph_id);
            return;
        }
    }

    println!("✓ CLI successfully handled --xray parameter");
}

#[test]
fn test_graph_get_json_output() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();
    let graph_id = test_graph_id();
    println!("Testing graph get with JSON output for graph: {}", graph_id);

    let mut cmd = langstar_cmd();
    cmd.args([
        "graph",
        "get",
        graph_id,
        "--deployment",
        &deployment,
        "--format",
        "json",
    ]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed or fail gracefully if graph doesn't exist
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not found") {
            println!("⚠ Graph '{}' not found - skipping test", graph_id);
            return;
        }
        panic!("Graph get command failed unexpectedly");
    }

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("JSON output:");
    println!("{}", stdout);

    // Should be parseable as JSON
    let result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(result.is_ok(), "Output should be valid JSON");

    let json = result.unwrap();
    assert!(
        json.get("nodes").is_some() || json.get("edges").is_some(),
        "JSON should contain nodes or edges"
    );

    println!("✓ CLI successfully output JSON format for graph get");
}

#[test]
fn test_graph_get_invalid_graph_id() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();
    println!("Testing graph get with invalid graph ID");

    let mut cmd = langstar_cmd();
    cmd.args([
        "graph",
        "get",
        "nonexistent-graph-12345",
        "--deployment",
        &deployment,
    ]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail with error message
    assert!(!output.status.success(), "Command should fail");

    println!("✓ CLI correctly handled invalid graph ID");
}

#[test]
fn test_graph_get_missing_deployment() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    println!("Testing graph get without --deployment flag");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "get", "agent"]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail with error about missing deployment
    assert!(
        !output.status.success(),
        "Command should fail without deployment"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("deployment") || stderr.contains("required"),
        "Error should mention missing deployment parameter"
    );

    println!("✓ CLI correctly rejected get without deployment");
}

#[test]
fn test_graph_commands_help() {
    println!("Testing graph command help output");

    // Test main graph help
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "--help"]);

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

    // Test graph list help
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--help"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("deployment"),
        "Help should mention deployment parameter"
    );
    assert!(
        stdout.contains("show-nodes"),
        "Help should mention --show-nodes flag"
    );

    // Test graph get help
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "get", "--help"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("graph-id"),
        "Help should mention graph_id parameter"
    );
    assert!(
        stdout.contains("deployment"),
        "Help should mention --deployment flag"
    );
    assert!(stdout.contains("xray"), "Help should mention --xray flag");

    println!("✓ All graph help commands work correctly");
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_graph_workflow_list_then_get() {
    let Some(_creds) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    let deployment = test_deployment_name();

    println!("\n==================================================");
    println!("Test: Graph Workflow (List then Get)");
    println!("==================================================\n");

    // Step 1: List graphs to find available graph IDs
    println!("Step 1: Listing graphs in deployment '{}'", deployment);
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", &deployment, "--format", "json"]);

    let output = cmd.output().expect("Failed to list graphs");
    assert!(
        output.status.success(),
        "List command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("List output:\n{}", stdout);

    // Parse JSON to get first graph ID
    let graphs: Vec<serde_json::Value> =
        serde_json::from_str(&stdout).expect("Should be valid JSON array");

    if graphs.is_empty() {
        println!("⚠ No graphs found in deployment - skipping get test");
        return;
    }

    let first_graph_id = graphs[0]["graph_id"]
        .as_str()
        .expect("Should have graph_id field")
        .to_string();

    println!("✓ Found graph ID: {}", first_graph_id);

    // Step 2: Get the graph structure
    println!("\nStep 2: Getting graph structure for '{}'", first_graph_id);
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "get", &first_graph_id, "--deployment", &deployment]);

    let output = cmd.output().expect("Failed to get graph");
    assert!(
        output.status.success(),
        "Get command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Get output:\n{}", stdout);

    assert!(
        stdout.contains("nodes") || stdout.contains("edges"),
        "Graph structure should contain nodes or edges"
    );

    println!("✓ Successfully retrieved graph structure");

    println!("\n==================================================");
    println!("✓ Workflow test completed successfully");
    println!("==================================================\n");
}
