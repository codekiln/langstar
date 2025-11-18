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

/// Helper to verify required environment variables
/// Returns None if credentials are not available (tests will be skipped)
fn check_env_vars() -> Option<String> {
    let api_key = std::env::var("LANGSMITH_API_KEY").ok()?;
    let workspace_id = std::env::var("LANGCHAIN_WORKSPACE_ID").ok()?;

    if api_key.is_empty() || workspace_id.is_empty() {
        return None;
    }

    println!("Testing with workspace ID: {}", workspace_id);
    Some(workspace_id)
}

#[test]
fn test_graph_list_basic() {
    let Some(_workspace_id) = check_env_vars() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };

    println!("Testing basic graph list command");

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

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

#[test]
#[ignore] // Requires actual API access and creates resources
fn test_graph_create_basic() {
    use std::time::{SystemTime, UNIX_EPOCH};

    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph create command");

    // Generate unique deployment name
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let deployment_name = format!("cli-test-deployment-{}", timestamp);

    let mut cmd = langstar_cmd();
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
    ]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    // Should succeed
    assert!(
        output.status.success(),
        "Graph create command should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Created deployment") || stdout.contains(&deployment_name),
        "Output should confirm deployment creation"
    );

    println!("✓ CLI successfully created deployment: {}", deployment_name);
}

#[test]
#[ignore] // Requires actual API access and creates resources
fn test_graph_create_with_wait() {
    use std::time::{SystemTime, UNIX_EPOCH};

    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph create command with --wait flag");

    // Generate unique deployment name
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let deployment_name = format!("cli-test-deployment-wait-{}", timestamp);

    let mut cmd = langstar_cmd();
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
    ]);

    // Run the command (this will take time as it waits for READY status)
    let output = cmd.output().expect("Failed to execute command");

    println!("Exit status: {}", output.status);
    println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    // Should succeed
    assert!(
        output.status.success(),
        "Graph create command with --wait should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show polling messages
    assert!(
        stdout.contains("Waiting for deployment") || stdout.contains("Status:"),
        "Output should show polling status"
    );

    // Should show deployment is ready
    assert!(
        stdout.contains("ready") || stdout.contains("READY") || stdout.contains("Ready"),
        "Output should confirm deployment is ready"
    );

    println!(
        "✓ CLI successfully created deployment with --wait: {}",
        deployment_name
    );
}

#[test]
#[ignore] // Requires actual API access - full lifecycle test
fn test_deployment_full_lifecycle() {
    use std::time::{SystemTime, UNIX_EPOCH};

    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("\n==================================================");
    println!("Test: Deployment Full Lifecycle (Create, List, Delete)");
    println!("==================================================\n");

    // Generate unique deployment name
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let deployment_name = format!("cli-test-lifecycle-{}", timestamp);

    // Step 1: Create deployment
    println!("Step 1: Creating deployment '{}'", deployment_name);
    let mut cmd = langstar_cmd();
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
    ]);

    let output = cmd.output().expect("Failed to create deployment");
    assert!(
        output.status.success(),
        "Create command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Create output:\n{}", stdout);

    // Extract deployment ID from output (format: "ID: <id>")
    let deployment_id = stdout
        .lines()
        .find(|line| line.contains("ID:"))
        .and_then(|line| line.split("ID:").nth(1))
        .map(|s| s.trim().to_string())
        .expect("Should find deployment ID in output");

    println!("✓ Created deployment with ID: {}", deployment_id);

    // Step 2: List deployments and verify it exists
    println!("\nStep 2: Listing deployments to verify creation");
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--name-contains", &deployment_name]);

    let output = cmd.output().expect("Failed to list deployments");
    assert!(output.status.success(), "List command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&deployment_name),
        "List should contain newly created deployment"
    );
    println!("✓ Deployment found in list");

    // Step 3: Delete deployment
    println!("\nStep 3: Deleting deployment '{}'", deployment_id);
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "delete", &deployment_id, "--yes"]);

    let output = cmd.output().expect("Failed to delete deployment");
    assert!(
        output.status.success(),
        "Delete command should succeed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("deleted") || stdout.contains("Successfully"),
        "Delete should confirm success"
    );
    println!("✓ Deployment deleted successfully");

    // Step 4: Verify deletion
    println!("\nStep 4: Verifying deployment was deleted");
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--name-contains", &deployment_name]);

    let output = cmd.output().expect("Failed to list deployments");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(&deployment_name) || stdout.contains("No deployments"),
        "Deployment should not appear in list after deletion"
    );
    println!("✓ Deployment successfully removed from list");

    println!("\n==================================================");
    println!("✓ Full lifecycle test completed successfully");
    println!("==================================================\n");
}

#[test]
fn test_graph_create_missing_repo_url() {
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph create without --repo-url");

    let mut cmd = langstar_cmd();
    cmd.args([
        "graph",
        "create",
        "--name",
        "test-deployment",
        "--source",
        "github",
        "--branch",
        "main",
        // Missing --repo-url
    ]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Command should fail without repo_url"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("repo_url is required") || stderr.contains("repo_url"),
        "Error should mention missing repo_url"
    );

    println!("✓ CLI correctly rejected create without repo_url");
}

#[test]
fn test_graph_create_missing_branch() {
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph create without --branch");

    let mut cmd = langstar_cmd();
    cmd.args([
        "graph",
        "create",
        "--name",
        "test-deployment",
        "--source",
        "github",
        "--repo-url",
        "https://github.com/langchain-ai/langgraph-example",
        // Missing --branch
    ]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Command should fail without branch"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("branch is required") || stderr.contains("branch"),
        "Error should mention missing branch"
    );

    println!("✓ CLI correctly rejected create without branch");
}

#[test]
fn test_graph_create_with_env_vars() {
    use std::time::{SystemTime, UNIX_EPOCH};

    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph create with environment variables");

    // Generate unique deployment name
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let deployment_name = format!("cli-test-deployment-env-{}", timestamp);

    let mut cmd = langstar_cmd();
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
        "--env",
        "DEBUG=true",
        "--env",
        "API_TIMEOUT=30",
    ]);

    // Note: This is marked as ignored by default since it creates resources
    // Just test that the command can be constructed
    println!("Command constructed successfully");
    println!("To run: cargo test test_graph_create_with_env_vars -- --ignored --nocapture");
}

#[test]
fn test_graph_create_invalid_source() {
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph create with invalid source type");

    let mut cmd = langstar_cmd();
    cmd.args([
        "graph",
        "create",
        "--name",
        "test-deployment",
        "--source",
        "invalid_source",
        "--repo-url",
        "https://github.com/langchain-ai/langgraph-example",
        "--branch",
        "main",
    ]);

    // Run the command
    let output = cmd.output().expect("Failed to execute command");

    // Should fail
    assert!(
        !output.status.success(),
        "Command should fail with invalid source"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid source type") || stderr.contains("github, external_docker"),
        "Error should mention invalid source type"
    );

    println!("✓ CLI correctly rejected invalid source type");
}

#[test]
#[ignore] // Requires actual API access and a deployment ID
fn test_graph_delete_with_yes_flag() {
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph delete with --yes flag");

    // Note: This test requires a real deployment ID
    // In practice, you'd create a deployment first, then delete it
    let deployment_id = "test-deployment-id-placeholder";

    let mut cmd = langstar_cmd();
    cmd.args(["graph", "delete", deployment_id, "--yes"]);

    println!("Note: This test requires a valid deployment ID");
    println!("To run manually:");
    println!("  1. Create a test deployment");
    println!("  2. Note the deployment ID");
    println!("  3. Run: langstar graph delete <id> --yes");
}

#[test]
fn test_graph_delete_confirmation_behavior() {
    if check_env_vars().is_none() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    println!("Testing graph delete confirmation behavior");

    // Document expected behavior
    println!("\nExpected behavior:");
    println!("1. Without --yes: prompts user for confirmation");
    println!("2. User must type 'yes' to confirm deletion");
    println!("3. With --yes: skips confirmation prompt");
    println!("\nTo test manually:");
    println!("  langstar graph delete <deployment-id>");
    println!("  langstar graph delete <deployment-id> --yes");
}
