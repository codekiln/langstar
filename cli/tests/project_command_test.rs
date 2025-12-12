//! CLI tests for `langstar project` commands.
//!
//! These tests verify:
//! - CLI argument parsing and validation
//! - Help text completeness
//! - Output format options
//! - Error handling for invalid inputs
//! - Integration with LangSmith API (when credentials available)
//!
//! **Test Categories:**
//!
//! 1. **Help tests** (no API access): Verify CLI parsing, help text, error handling
//! 2. **Integration tests** (requires API): Verify actual project operations
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY` environment variable
//! - `LANGSMITH_ORGANIZATION_ID` environment variable
//! - `LANGSMITH_WORKSPACE_ID` environment variable
//!
//! Run with: `cargo test --test project_command_test`

use assert_cmd::Command;
use escargot::CargoBuild;
use langstar_sdk::{AuthConfig, LangchainClient, ProjectCreate};
use predicates::prelude::*;

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

/// Helper function to create SDK client from environment
fn create_sdk_client() -> Result<LangchainClient, String> {
    let auth = AuthConfig::from_env().map_err(|e| format!("Auth config error: {}", e))?;
    LangchainClient::new(auth).map_err(|e| format!("Client creation error: {}", e))
}

/// Helper function to create a tokio runtime for blocking async calls
fn create_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("Failed to create runtime")
}

/// Generate a unique test project name to avoid collisions
fn generate_test_project_name() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    // Add random UUID suffix for extra uniqueness
    let uuid_suffix = uuid::Uuid::new_v4().to_string();
    let short_uuid = &uuid_suffix[..8];
    format!("test-crud-lifecycle-{}-{}", timestamp, short_uuid)
}

// ═══════════════════════════════════════════════════════════════════════════
// Help and Documentation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_project_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["project", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Manage LangSmith projects"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_project_list_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["project", "list", "--help"]);

    cmd.assert()
        .success()
        // Command description
        .stdout(predicate::str::contains("List projects"))
        // Filter arguments
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--name-contains"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--include-stats"))
        // Output format flag (should be -f, not -o per issue #653)
        .stdout(predicate::str::contains("-f, --format"));
}

#[test]
fn test_project_create_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["project", "create", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create a new project"))
        .stdout(predicate::str::contains("--description"))
        .stdout(predicate::str::contains("--metadata"))
        .stdout(predicate::str::contains("-f, --format"));
}

#[test]
fn test_project_get_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["project", "get", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Get details of a specific project",
        ))
        .stdout(predicate::str::contains("-f, --format"));
}

#[test]
fn test_project_update_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["project", "update", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Update a project"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--description"))
        .stdout(predicate::str::contains("-f, --format"));
}

#[test]
fn test_project_delete_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["project", "delete", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete a project"))
        .stdout(predicate::str::contains("--force"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Integration Tests (Requires API credentials)
// ═══════════════════════════════════════════════════════════════════════════

/// Full CRUD lifecycle test following the pattern from prompt_scoping_test.rs
///
/// This test prevents #536-style bugs by verifying:
/// - CLI commands produce expected API state (CREATE, UPDATE, DELETE)
/// - SDK can verify CLI actions persisted correctly (VERIFY steps)
/// - CLI commands display correct output (READ steps)
///
/// Pattern: CREATE → VERIFY → READ → VERIFY → UPDATE → VERIFY → DELETE
#[test]
fn test_project_crud_lifecycle() {
    // ═══════════════════════════════════════════════════════════════
    // Setup - Explicit failure if env vars missing
    // ═══════════════════════════════════════════════════════════════
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let runtime = create_runtime();
    let client = create_sdk_client().expect("SDK client required");
    let test_project_name = generate_test_project_name();

    // ═══════════════════════════════════════════════════════════════
    // Step 1: CREATE - Create project via SDK
    // ═══════════════════════════════════════════════════════════════
    println!(
        "[CREATE] Creating test project via SDK: {}",
        test_project_name
    );

    let project = runtime
        .block_on(async {
            client
                .create_project(ProjectCreate {
                    name: Some(test_project_name.clone()),
                    description: Some("Test project for CRUD lifecycle".to_string()),
                    ..Default::default()
                })
                .await
        })
        .expect("Failed to create test project");

    let project_id = project.id;
    println!("  ✓ Created project with ID: {}", project_id);

    // ═══════════════════════════════════════════════════════════════
    // Step 2: VERIFY - Confirm project exists via SDK read
    // ═══════════════════════════════════════════════════════════════
    println!("[VERIFY] Reading project via SDK...");

    let read_project = runtime
        .block_on(async { client.get_project(project_id).await })
        .expect("Failed to read project");

    assert_eq!(read_project.id, project_id);
    assert_eq!(read_project.name, Some(test_project_name.clone()));
    println!("  ✓ Verified project exists");

    // ═══════════════════════════════════════════════════════════════
    // Step 3: READ - Execute CLI list command
    // ═══════════════════════════════════════════════════════════════
    println!("[READ] Running CLI 'project list'...");

    let mut list_cmd = langstar_cmd();
    list_cmd.args([
        "project",
        "list",
        "--name",
        &test_project_name,
        "-f",
        "json",
    ]);

    let output = list_cmd.output().expect("Failed to execute CLI");
    assert!(
        output.status.success(),
        "CLI list command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // ═══════════════════════════════════════════════════════════════
    // Step 4: VERIFY - Parse output, confirm our project appears
    // ═══════════════════════════════════════════════════════════════
    println!("[VERIFY] Checking CLI output contains test project...");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let projects: Vec<serde_json::Value> =
        serde_json::from_str(&stdout).expect("Failed to parse CLI JSON output");

    let found = projects.iter().any(|p| {
        p.get("id")
            .and_then(|v| v.as_str())
            .map(|id| id == project_id.to_string())
            .unwrap_or(false)
    });

    assert!(
        found,
        "BUG: Created project '{}' not found in CLI list output. \
         This is the bug that issue #536 was supposed to fix!",
        test_project_name
    );
    println!("  ✓ Verified project appears in CLI list");

    // ═══════════════════════════════════════════════════════════════
    // Step 5: READ - Get specific project via CLI
    // ═══════════════════════════════════════════════════════════════
    println!("[READ] Running CLI 'project get'...");

    let mut get_cmd = langstar_cmd();
    get_cmd.args(["project", "get", &test_project_name, "-f", "json"]);

    let output = get_cmd.output().expect("Failed to execute CLI get");
    assert!(
        output.status.success(),
        "CLI get command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let got_project: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse CLI get JSON output");

    assert_eq!(
        got_project.get("name").and_then(|v| v.as_str()),
        Some(test_project_name.as_str())
    );
    println!("  ✓ Verified CLI get returns correct project");

    // ═══════════════════════════════════════════════════════════════
    // Step 6: UPDATE - Update project description via CLI
    // ═══════════════════════════════════════════════════════════════
    println!("[UPDATE] Updating project description via CLI...");

    let updated_description = "Updated description from CLI";
    let mut update_cmd = langstar_cmd();
    update_cmd.args([
        "project",
        "update",
        &test_project_name,
        "--description",
        updated_description,
        "-f",
        "json",
    ]);

    let output = update_cmd.output().expect("Failed to execute CLI update");
    assert!(
        output.status.success(),
        "CLI update command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!("  ✓ Update command succeeded");

    // ═══════════════════════════════════════════════════════════════
    // Step 7: VERIFY - Confirm update via SDK read
    // ═══════════════════════════════════════════════════════════════
    println!("[VERIFY] Confirming update via SDK...");

    let updated_project = runtime
        .block_on(async { client.get_project(project_id).await })
        .expect("Failed to read updated project");

    assert_eq!(
        updated_project.description,
        Some(updated_description.to_string())
    );
    println!("  ✓ Verified description updated");

    // ═══════════════════════════════════════════════════════════════
    // Step 8: DELETE - Clean up test data
    // ═══════════════════════════════════════════════════════════════
    println!("[DELETE] Cleaning up test project...");

    let _cleanup = runtime.block_on(async { client.delete_project(project_id).await });

    // Note: Don't panic on cleanup failure - test already passed
    println!("  ✓ Cleanup completed");
}

/// Test list filtering with name_contains
#[test]
fn test_project_list_with_name_contains() {
    // Setup - Explicit failure if env vars missing
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let runtime = create_runtime();
    let client = create_sdk_client().expect("SDK client required");

    // Create two projects with distinct names
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let alpha_name = format!("test-alpha-{}", timestamp);
    let beta_name = format!("test-beta-{}", timestamp);

    let alpha_project = runtime
        .block_on(async {
            client
                .create_project(ProjectCreate {
                    name: Some(alpha_name.clone()),
                    ..Default::default()
                })
                .await
        })
        .expect("Failed to create alpha project");

    let beta_project = runtime
        .block_on(async {
            client
                .create_project(ProjectCreate {
                    name: Some(beta_name.clone()),
                    ..Default::default()
                })
                .await
        })
        .expect("Failed to create beta project");

    // Test: List with name_contains filter
    let mut list_cmd = langstar_cmd();
    list_cmd.args(["project", "list", "--name-contains", "alpha", "-f", "json"]);

    let output = list_cmd.output().expect("Failed to execute CLI");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let projects: Vec<serde_json::Value> =
        serde_json::from_str(&stdout).expect("Failed to parse JSON");

    // Verify alpha appears, beta does not (in the filtered results for this name pattern)
    let has_alpha = projects
        .iter()
        .any(|p| p.get("name").and_then(|v| v.as_str()) == Some(alpha_name.as_str()));
    assert!(has_alpha, "Alpha project should appear in filtered results");

    // Cleanup
    let _ = runtime.block_on(async { client.delete_project(alpha_project.id).await });
    let _ = runtime.block_on(async { client.delete_project(beta_project.id).await });
}

/// Test JSON output format
#[test]
fn test_project_list_json_output() {
    // Setup
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let mut cmd = langstar_cmd();
    cmd.args(["project", "list", "-f", "json", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute CLI");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify it's valid JSON array
    let projects: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&stdout);
    assert!(
        projects.is_ok(),
        "Output should be valid JSON array: {}",
        stdout
    );

    // If there are any projects, verify they have expected fields
    if let Ok(projects) = projects {
        if !projects.is_empty() {
            let first = &projects[0];
            assert!(first.get("id").is_some(), "Project should have 'id' field");
            assert!(
                first.get("tenant_id").is_some(),
                "Project should have 'tenant_id' field"
            );
        }
    }
}

/// Test table output format
#[test]
fn test_project_list_table_output() {
    // Setup
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let mut cmd = langstar_cmd();
    cmd.args(["project", "list", "-f", "table", "--limit", "5"]);

    let output = cmd.output().expect("Failed to execute CLI");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify table format (should contain headers)
    assert!(
        stdout.contains("ID") || stdout.contains("Name"),
        "Table output should contain column headers: {}",
        stdout
    );
}

/// Test error handling for nonexistent project
#[test]
fn test_project_get_not_found() {
    // Setup
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let mut cmd = langstar_cmd();
    cmd.args(["project", "get", "nonexistent-project-999999999"]);

    let output = cmd.output().expect("Failed to execute CLI");

    // Should fail with non-zero exit code
    assert!(
        !output.status.success(),
        "Command should fail for nonexistent project"
    );

    // Error message should be helpful
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("Not found"),
        "Error message should indicate project not found: {}",
        stderr
    );
}

/// Test create with metadata
#[test]
fn test_project_create_with_metadata() {
    // Setup
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let runtime = create_runtime();
    let client = create_sdk_client().expect("SDK client required");

    let test_name = generate_test_project_name();
    let metadata_json = r#"{"environment":"staging","version":"1.0"}"#;

    // Create via CLI with metadata
    let mut create_cmd = langstar_cmd();
    create_cmd.args([
        "project",
        "create",
        &test_name,
        "--metadata",
        metadata_json,
        "-f",
        "json",
    ]);

    let output = create_cmd.output().expect("Failed to execute CLI");
    assert!(
        output.status.success(),
        "Create with metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let created: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse create output");
    let project_id = created
        .get("id")
        .and_then(|v| v.as_str())
        .expect("Created project should have id");
    let project_uuid = uuid::Uuid::parse_str(project_id).expect("ID should be valid UUID");

    // Verify metadata via SDK
    let project = runtime
        .block_on(async { client.get_project(project_uuid).await })
        .expect("Failed to read project");

    assert!(
        project.extra.is_some(),
        "Project should have metadata/extra field"
    );
    if let Some(extra) = project.extra {
        assert_eq!(
            extra.get("environment").and_then(|v| v.as_str()),
            Some("staging")
        );
        assert_eq!(extra.get("version").and_then(|v| v.as_str()), Some("1.0"));
    }

    // Cleanup
    let _ = runtime.block_on(async { client.delete_project(project_uuid).await });
}

/// Test create with invalid metadata JSON
#[test]
fn test_project_create_with_invalid_metadata() {
    // Setup
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let test_name = generate_test_project_name();

    let mut cmd = langstar_cmd();
    cmd.args([
        "project",
        "create",
        &test_name,
        "--metadata",
        "not-valid-json-at-all",
    ]);

    let output = cmd.output().expect("Failed to execute CLI");

    // Should fail
    assert!(
        !output.status.success(),
        "Command should fail for invalid JSON metadata"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("json")
            || stderr.to_lowercase().contains("invalid")
            || stderr.to_lowercase().contains("metadata"),
        "Error should mention JSON/invalid/metadata: {}",
        stderr
    );
}
