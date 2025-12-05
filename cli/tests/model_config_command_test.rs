//! CLI tests for `langstar model-config` commands.
//!
//! These tests verify:
//! - CLI argument parsing and validation
//! - Help text completeness
//! - Output format options
//! - Error handling for invalid inputs
//!
//! **Test Categories:**
//!
//! 1. **Unit tests** (no API access): Verify CLI parsing, help text, error handling
//! 2. **Integration tests** (requires API): Verify actual model-config operations
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY` environment variable
//! - `LANGSMITH_ORGANIZATION_ID` environment variable (or auto-discovery)
//!
//! Run with: `cargo test --test model_config_command_test`

use assert_cmd::Command;
use escargot::CargoBuild;
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

// ═══════════════════════════════════════════════════════════════════════════
// Help and Documentation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_model_config_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage LangSmith model configurations",
        ))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_model_config_list_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "list", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all model configurations"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--offset"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_model_config_get_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "get", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Get details of a specific model configuration",
        ))
        .stdout(predicate::str::contains("<ID>"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_model_config_create_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "create", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create a new model configuration"))
        .stdout(predicate::str::contains("--file"))
        .stdout(predicate::str::contains(
            "Path to JSON file containing configuration",
        ));
}

#[test]
fn test_model_config_update_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "update", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Update an existing model configuration",
        ))
        .stdout(predicate::str::contains("<ID>"))
        .stdout(predicate::str::contains("--file"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--description"));
}

#[test]
fn test_model_config_delete_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "delete", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete a model configuration"))
        .stdout(predicate::str::contains("<ID>"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Validation Tests (No API Required)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_model_config_get_missing_id() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "get"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"));
}

#[test]
fn test_model_config_get_invalid_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "get", "not-a-uuid"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'not-a-uuid'"));
}

#[test]
fn test_model_config_create_missing_file() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "create"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"));
}

#[test]
fn test_model_config_create_nonexistent_file() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "create", "--file", "/nonexistent/file.json"]);

    // This should fail because the file doesn't exist
    cmd.assert().failure();
}

#[test]
fn test_model_config_update_missing_id() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "update"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"));
}

#[test]
fn test_model_config_delete_missing_id() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "delete"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Integration Tests (Require LANGSMITH_API_KEY)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_model_config_list_integration() {
    // This test requires LANGSMITH_API_KEY
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "list", "--format", "json"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"));
}

#[test]
fn test_model_config_list_with_pagination() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "model-config",
        "list",
        "--limit",
        "5",
        "--offset",
        "0",
        "--format",
        "json",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"));
}

#[test]
fn test_model_config_create_update_delete_cycle() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a test configuration file
    let create_config = serde_json::json!({
        "name": "CLI Test Config",
        "description": "Test configuration created by CLI tests",
        "settings": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
            "kwargs": {
                "model": "claude-3-5-sonnet-20241022",
                "temperature": 0.0
            }
        },
        "options": {
            "requests_per_second": 5
        }
    });

    let mut config_file = NamedTempFile::new().expect("Failed to create temp file");
    config_file
        .write_all(create_config.to_string().as_bytes())
        .expect("Failed to write config file");

    let mut create_cmd = langstar_cmd();
    create_cmd.args([
        "model-config",
        "create",
        "--file",
        config_file.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    let create_output = create_cmd.assert().success().get_output().stdout.clone();

    let create_json: serde_json::Value =
        serde_json::from_slice(&create_output).expect("Failed to parse create output as JSON");

    let config_id = create_json["id"]
        .as_str()
        .expect("Missing 'id' in create response");

    // Update the configuration
    let mut update_cmd = langstar_cmd();
    update_cmd.args([
        "model-config",
        "update",
        config_id,
        "--name",
        "CLI Test Config - Updated",
        "--format",
        "json",
    ]);

    // Verify the update was successful and name changed
    let update_output = update_cmd.assert().success().get_output().stdout.clone();
    let update_json: serde_json::Value =
        serde_json::from_slice(&update_output).expect("Failed to parse update output as JSON");
    assert_eq!(
        update_json["name"].as_str(),
        Some("CLI Test Config - Updated"),
        "Update should change the name"
    );

    // Delete the configuration
    let mut delete_cmd = langstar_cmd();
    delete_cmd.args(["model-config", "delete", config_id, "--yes"]);

    delete_cmd.assert().success();
}

#[test]
fn test_model_config_get_nonexistent() {
    // Try to get a non-existent config (random UUID)
    let fake_id = "00000000-0000-0000-0000-000000000000";

    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "get", fake_id]);

    cmd.assert().failure().stderr(
        predicate::str::contains("not found")
            .or(predicate::str::contains("404"))
            .or(predicate::str::contains("error")),
    );
}

#[test]
fn test_model_config_delete_nonexistent() {
    // Try to delete a non-existent config (random UUID)
    // Note: The API supports idempotent deletes, so this returns success even for nonexistent UUIDs
    let fake_id = "00000000-0000-0000-0000-000000000000";

    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "delete", fake_id, "--yes"]);

    // The delete operation is idempotent and succeeds even for nonexistent resources
    cmd.assert().success();
}

// ═══════════════════════════════════════════════════════════════════════════
// Output Format Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_model_config_list_json_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "list", "--format", "json"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"));
}

#[test]
fn test_model_config_list_table_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "list"]);

    cmd.assert()
        .success()
        // Should have some table-like output (columns, headers, etc.)
        .stdout(predicate::str::contains("ID").or(predicate::str::contains("Name")));
}

#[test]
fn test_model_config_list_text_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["model-config", "list", "--format", "text"]);

    // Phase 1: Text format falls back to JSON
    // In future phases, this will output tab-separated values
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"));
}
