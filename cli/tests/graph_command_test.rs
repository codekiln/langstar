use assert_cmd::Command;
use escargot::CargoBuild;

/// CLI Integration tests for graph commands
///
/// These tests verify that the langstar CLI correctly:
/// 1. Lists LangGraph deployments via Control Plane API
/// 2. Accepts various filter options
/// 3. Outputs JSON format when requested
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY environment variable
/// 2. Valid LANGCHAIN_WORKSPACE_ID environment variable (required for Control Plane API)
///
/// Run with: cargo test --test graph_command_test

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

#[test]
fn test_graph_list_basic() {
    // Requires LANGSMITH_API_KEY and LANGCHAIN_WORKSPACE_ID
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing basic graph list command");
    println!("Workspace ID: {}", workspace_id);

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list"]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(output.status.success(), "Command should succeed");

    // Output should contain deployment information
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("Command output:");
    println!("{}", stdout);

    // Should contain either deployment info or "No deployments found"
    let has_deployments = stdout.contains("deployment") || stdout.contains("No deployments");
    assert!(
        has_deployments,
        "Output should contain deployment info or 'No deployments found'"
    );

    println!("✓ CLI successfully listed deployments");
}

#[test]
fn test_graph_list_with_limit() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with --limit flag");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--limit", "5"]);

    // Run the command
    let assert = cmd.assert();

    // Should succeed
    assert.success();

    println!("✓ CLI successfully handled --limit parameter");
}

#[test]
fn test_graph_list_json_output() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with JSON output");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--format", "json"]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(output.status.success(), "Command should succeed");

    // Output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("JSON output:");
    println!("{}", stdout);

    // Should contain JSON structure
    assert!(
        stdout.contains("resources") || stdout.contains("offset"),
        "JSON output should contain 'resources' or 'offset' field"
    );

    println!("✓ CLI successfully output JSON format");
}

#[test]
fn test_graph_list_with_deployment_type_filter() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with --deployment-type filter");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--deployment-type", "prod"]);

    // Run the command
    let assert = cmd.assert();

    // Should succeed (even if no prod deployments exist)
    assert.success();

    println!("✓ CLI successfully handled --deployment-type filter");
}

#[test]
fn test_graph_list_with_status_filter() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with --status filter");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--status", "READY"]);

    // Run the command
    let assert = cmd.assert();

    // Should succeed
    assert.success();

    println!("✓ CLI successfully handled --status filter");
}

#[test]
fn test_graph_list_with_name_filter() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with --name-contains filter");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--name-contains", "test"]);

    // Run the command
    let assert = cmd.assert();

    // Should succeed
    assert.success();

    println!("✓ CLI successfully handled --name-contains filter");
}

#[test]
fn test_graph_list_invalid_deployment_type() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with invalid --deployment-type");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--deployment-type", "invalid_type"]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail with error message
    assert!(!output.status.success(), "Command should fail");

    // Should contain helpful error message
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("Invalid deployment type") || stderr.contains("valid values"),
        "Error message should mention invalid deployment type"
    );

    println!("✓ CLI correctly rejected invalid deployment type");
}

#[test]
fn test_graph_list_invalid_status() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with invalid --status");

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--status", "INVALID_STATUS"]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail with error message
    assert!(!output.status.success(), "Command should fail");

    // Should contain helpful error message
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("Invalid status") || stderr.contains("valid values"),
        "Error message should mention invalid status"
    );

    println!("✓ CLI correctly rejected invalid status");
}

#[test]
fn test_graph_list_multiple_filters() {
    let _api_key =
        std::env::var("LANGSMITH_API_KEY").expect("LANGSMITH_API_KEY must be set for this test");
    let _workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID")
        .expect("LANGCHAIN_WORKSPACE_ID must be set for this test");

    println!("Testing graph list with multiple filters");

    let mut cmd = langstar_cmd();
    cmd.args([
        "graph",
        "list",
        "--limit",
        "10",
        "--deployment-type",
        "dev",
        "--status",
        "READY",
    ]);

    // Run the command
    let assert = cmd.assert();

    // Should succeed
    assert.success();

    println!("✓ CLI successfully handled multiple filters");
}
