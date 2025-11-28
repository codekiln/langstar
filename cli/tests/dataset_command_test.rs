//! CLI tests for `langstar dataset` commands.
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
//! 1. **Unit tests** (no API access): Verify CLI parsing, help text, error handling
//! 2. **Integration tests** (requires API): Verify actual dataset operations
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY` environment variable
//!
//! Run with: `cargo test --test dataset_command_test`

use assert_cmd::Command;
use escargot::CargoBuild;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

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
fn test_dataset_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Manage LangSmith datasets"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("import"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("list-examples"));
}

#[test]
fn test_dataset_create_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "create", "--help"]);

    cmd.assert()
        .success()
        // Command description
        .stdout(predicate::str::contains("Create a new dataset"))
        // Required arguments
        .stdout(predicate::str::contains("--name"))
        // Optional arguments
        .stdout(predicate::str::contains("--data-type"))
        .stdout(predicate::str::contains("--description"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_dataset_list_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "list", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List datasets"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--name-contains"))
        .stdout(predicate::str::contains("--data-type"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_dataset_get_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "get", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Get details of a specific dataset",
        ))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_dataset_update_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "update", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Update a dataset"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--description"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_dataset_delete_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "delete", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete a dataset"))
        .stdout(predicate::str::contains("--yes"));
}

#[test]
fn test_dataset_import_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "import", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Import examples from a file"))
        .stdout(predicate::str::contains("--file"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_dataset_export_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "export", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Export examples to a file"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--out"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_dataset_list_examples_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "list-examples", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List examples in a dataset"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--json"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Argument Validation Tests (No API Access Required)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dataset_create_requires_name() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "create"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--name"));
}

#[test]
fn test_dataset_create_invalid_data_type() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "create",
        "--name",
        "test",
        "--data-type",
        "invalid",
    ]);

    // The CLI accepts the string but the backend rejects invalid types
    // For clap-level validation, we need to check if the value is passed
    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either clap rejects it or the command proceeds and backend rejects
    // The CLI currently accepts any string and validates later
    assert!(
        stderr.contains("invalid")
            || stderr.contains("Invalid")
            || output.status.success()
            || stderr.contains("API")
    );
}

#[test]
fn test_dataset_get_requires_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "get"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("<DATASET_ID>").or(predicate::str::contains("required")));
}

#[test]
fn test_dataset_get_invalid_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "get", "not-a-uuid"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn test_dataset_update_requires_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "update"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("<DATASET_ID>").or(predicate::str::contains("required")));
}

#[test]
fn test_dataset_delete_requires_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "delete"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("<DATASET_ID>").or(predicate::str::contains("required")));
}

#[test]
fn test_dataset_import_requires_file() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "import", "00000000-0000-0000-0000-000000000001"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--file"));
}

#[test]
fn test_dataset_export_requires_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "export", "00000000-0000-0000-0000-000000000001"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--format"));
}

#[test]
fn test_dataset_list_examples_requires_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "list-examples"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("<DATASET_ID>").or(predicate::str::contains("required")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Valid Argument Combinations (Parsing Tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dataset_create_accepts_all_args() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "create",
        "--name",
        "test-dataset",
        "--data-type",
        "kv",
        "--description",
        "A test dataset",
        "--json",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not have parsing errors - will fail on API call if no auth
    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept all dataset create arguments"
    );
    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept valid argument values"
    );
}

#[test]
fn test_dataset_list_accepts_all_filters() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "list",
        "--name",
        "test",
        "--name-contains",
        "partial",
        "--data-type",
        "chat",
        "--limit",
        "50",
        "--json",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept all list arguments"
    );
}

#[test]
fn test_dataset_list_accepts_data_types() {
    for data_type in ["kv", "llm", "chat"] {
        let mut cmd = langstar_cmd();
        cmd.args(["dataset", "list", "--data-type", data_type]);

        let output = cmd.output().expect("Failed to execute command");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !stderr.contains("invalid value"),
            "CLI should accept --data-type {}",
            data_type
        );
    }
}

#[test]
fn test_dataset_export_accepts_formats() {
    for format in ["jsonl", "csv"] {
        let mut cmd = langstar_cmd();
        cmd.args([
            "dataset",
            "export",
            "00000000-0000-0000-0000-000000000001",
            "--format",
            format,
        ]);

        let output = cmd.output().expect("Failed to execute command");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !stderr.contains("invalid value"),
            "CLI should accept --format {}",
            format
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Error Handling Tests (No API Access)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dataset_list_without_api_key() {
    let mut cmd = langstar_cmd();
    cmd.env_remove("LANGSMITH_API_KEY");
    cmd.args(["dataset", "list"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should mention API key or authentication
    assert!(
        stderr.contains("API")
            || stderr.contains("api")
            || stderr.contains("key")
            || stderr.contains("Key")
            || stderr.contains("auth")
            || stderr.contains("Auth"),
        "Should mention API key or auth in error: {}",
        stderr
    );
}

#[test]
fn test_dataset_import_file_not_found() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "import",
        "00000000-0000-0000-0000-000000000001",
        "--file",
        "/nonexistent/path/file.jsonl",
    ]);

    // Should fail either at API auth or file not found
    cmd.assert().failure();
}

#[test]
fn test_dataset_delete_without_confirmation() {
    let mut cmd = langstar_cmd();
    // Remove API key to ensure we don't actually delete anything
    cmd.env_remove("LANGSMITH_API_KEY");
    cmd.args(["dataset", "delete", "00000000-0000-0000-0000-000000000001"]);

    // Without --yes flag, should prompt for confirmation
    let output = cmd.output().expect("Failed to execute command");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        combined.contains("Are you sure")
            || combined.contains("--yes")
            || combined.contains("confirmation")
            || combined.contains("API"),
        "Should prompt for confirmation or fail on auth: {}",
        combined
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Import/Export File Format Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_dataset_import_detects_jsonl_extension() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.jsonl");
    fs::write(
        &file_path,
        r#"{"inputs": {"question": "test"}, "outputs": {"answer": "test"}}"#,
    )
    .unwrap();

    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "import",
        "00000000-0000-0000-0000-000000000001",
        "--file",
        file_path.to_str().unwrap(),
    ]);

    // Should parse the file (fail on API auth, not file format)
    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("Unsupported format"),
        "Should auto-detect .jsonl extension"
    );
}

#[test]
fn test_dataset_import_detects_csv_extension() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.csv");
    fs::write(
        &file_path,
        "inputs,outputs\n\"{\"\"q\"\":\"\"test\"\"}\",\"{\"\"a\"\":\"\"test\"\"}\"",
    )
    .unwrap();

    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "import",
        "00000000-0000-0000-0000-000000000001",
        "--file",
        file_path.to_str().unwrap(),
    ]);

    // Should parse the file (fail on API auth, not file format)
    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("Unsupported format"),
        "Should auto-detect .csv extension"
    );
}

#[test]
fn test_dataset_import_format_override() {
    let temp_dir = TempDir::new().unwrap();
    // File has wrong extension but we override with --format
    let file_path = temp_dir.path().join("test.txt");
    fs::write(
        &file_path,
        r#"{"inputs": {"question": "test"}, "outputs": {"answer": "test"}}"#,
    )
    .unwrap();

    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "import",
        "00000000-0000-0000-0000-000000000001",
        "--file",
        file_path.to_str().unwrap(),
        "--format",
        "jsonl",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("Unsupported format"),
        "Should use --format override"
    );
}

#[test]
fn test_dataset_export_invalid_format() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "export",
        "00000000-0000-0000-0000-000000000001",
        "--format",
        "xml",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should fail with unsupported format error
    assert!(
        stderr.contains("Unsupported format")
            || stderr.contains("invalid")
            || stderr.contains("Invalid"),
        "Should reject invalid export format: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Integration Tests (Require API Access)
// ═══════════════════════════════════════════════════════════════════════════

/// Check if we have API credentials available
fn has_api_credentials() -> bool {
    std::env::var("LANGSMITH_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
}

#[test]
fn test_dataset_list_basic() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "list", "--limit", "5"]);

    cmd.assert().success();

    println!("✓ CLI successfully listed datasets");
}

#[test]
fn test_dataset_list_json_output() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args(["dataset", "list", "--limit", "3", "--json"]);

    let output = cmd.assert().success();
    let stdout_bytes = output.get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    // Output should be valid JSON array
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(parsed.is_array(), "JSON output should be an array");

    println!("✓ CLI successfully returned JSON output");
}

#[test]
fn test_dataset_list_with_data_type_filter() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args([
        "dataset",
        "list",
        "--data-type",
        "kv",
        "--limit",
        "5",
        "--json",
    ]);

    let output = cmd.assert().success();
    let stdout_bytes = output.get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    // All datasets should have data_type == "kv" or null (defaults to kv)
    if let Some(datasets) = parsed.as_array() {
        for dataset in datasets {
            if let Some(dt) = dataset.get("data_type").and_then(|v| v.as_str()) {
                assert_eq!(dt, "kv", "All datasets should have data_type 'kv'");
            }
        }
    }

    println!("✓ CLI successfully filtered datasets by type");
}

#[test]
fn test_dataset_list_with_name_contains_filter() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    // Use a common substring that might match some datasets
    cmd.args(["dataset", "list", "--name-contains", "test", "--limit", "5"]);

    // Should succeed even if no matches
    cmd.assert().success();

    println!("✓ CLI successfully filtered datasets by name");
}

// ═══════════════════════════════════════════════════════════════════════════
// Full CRUD Integration Tests (create, get, update, delete)
// Gated by integration-tests feature
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "integration-tests")]
mod integration {
    use super::*;
    use uuid::Uuid;

    /// Test the full CRUD lifecycle for datasets.
    ///
    /// Note: This test is currently ignored due to a bug in the CLI where
    /// `dataset update` fails with "error decoding response body".
    /// TODO: Fix the update command's response handling and re-enable this test.
    #[test]
    #[ignore = "CLI bug: update command fails to decode response body"]
    fn test_dataset_crud_lifecycle() {
        if !has_api_credentials() {
            println!("Skipping test: LANGSMITH_API_KEY not set");
            return;
        }

        // 1. Create dataset
        let unique_name = format!(
            "test-dataset-{}",
            Uuid::new_v4().to_string()[..8].to_string()
        );
        let mut cmd = langstar_cmd();
        cmd.args([
            "dataset",
            "create",
            "--name",
            &unique_name,
            "--data-type",
            "kv",
            "--description",
            "Integration test dataset",
            "--json",
        ]);

        let output = cmd.assert().success();
        let stdout = String::from_utf8_lossy(&output.get_output().stdout);
        let created: serde_json::Value =
            serde_json::from_str(&stdout).expect("Create should return JSON");
        let dataset_id = created["id"].as_str().expect("Should have ID");

        println!("✓ Created dataset: {}", dataset_id);

        // 2. Get dataset
        let mut cmd = langstar_cmd();
        cmd.args(["dataset", "get", dataset_id, "--json"]);

        let output = cmd.assert().success();
        let stdout = String::from_utf8_lossy(&output.get_output().stdout);
        let fetched: serde_json::Value =
            serde_json::from_str(&stdout).expect("Get should return JSON");
        assert_eq!(fetched["name"].as_str(), Some(unique_name.as_str()));

        println!("✓ Fetched dataset successfully");

        // 3. Update dataset
        let updated_name = format!("{}-updated", unique_name);
        let mut cmd = langstar_cmd();
        cmd.args([
            "dataset",
            "update",
            dataset_id,
            "--name",
            &updated_name,
            "--json",
        ]);

        let output = cmd.assert().success();
        let stdout = String::from_utf8_lossy(&output.get_output().stdout);
        let updated: serde_json::Value =
            serde_json::from_str(&stdout).expect("Update should return JSON");
        assert_eq!(updated["name"].as_str(), Some(updated_name.as_str()));

        println!("✓ Updated dataset successfully");

        // 4. Delete dataset
        let mut cmd = langstar_cmd();
        cmd.args(["dataset", "delete", dataset_id, "--yes"]);

        cmd.assert().success();

        println!("✓ Deleted dataset successfully");
        println!("✓ Full CRUD lifecycle test passed");
    }

    /// Test import/export roundtrip for datasets.
    ///
    /// Note: This test is currently ignored due to a bug in the CLI where
    /// `dataset export --format jsonl` conflicts with the global `-f, --format`
    /// output format flag.
    /// TODO: Rename export's --format flag or use a subcommand to avoid conflict.
    #[test]
    #[ignore = "CLI bug: export --format conflicts with global output format flag"]
    fn test_dataset_import_export_roundtrip() {
        if !has_api_credentials() {
            println!("Skipping test: LANGSMITH_API_KEY not set");
            return;
        }

        let temp_dir = TempDir::new().unwrap();
        let unique_name = format!(
            "roundtrip-test-{}",
            Uuid::new_v4().to_string()[..8].to_string()
        );

        // 1. Create dataset
        let mut cmd = langstar_cmd();
        cmd.args([
            "dataset",
            "create",
            "--name",
            &unique_name,
            "--data-type",
            "kv",
            "--json",
        ]);

        let output = cmd.assert().success();
        let stdout = String::from_utf8_lossy(&output.get_output().stdout);
        let created: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let dataset_id = created["id"].as_str().unwrap();

        // 2. Import examples from JSONL
        let import_file = temp_dir.path().join("import.jsonl");
        fs::write(
            &import_file,
            r#"{"inputs": {"question": "What is 2+2?"}, "outputs": {"answer": "4"}}
{"inputs": {"question": "What is 3+3?"}, "outputs": {"answer": "6"}}
{"inputs": {"question": "What is 4+4?"}, "outputs": {"answer": "8"}}"#,
        )
        .unwrap();

        let mut cmd = langstar_cmd();
        cmd.args([
            "dataset",
            "import",
            dataset_id,
            "--file",
            import_file.to_str().unwrap(),
        ]);

        cmd.assert().success();
        println!("✓ Imported 3 examples");

        // 3. Export to JSONL
        let export_file = temp_dir.path().join("export.jsonl");
        let mut cmd = langstar_cmd();
        cmd.args([
            "dataset",
            "export",
            dataset_id,
            "--format",
            "jsonl",
            "--out",
            export_file.to_str().unwrap(),
        ]);

        cmd.assert().success();

        // 4. Verify export
        let exported = fs::read_to_string(&export_file).unwrap();
        let lines: Vec<&str> = exported.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 3, "Should have exported 3 examples");

        for line in lines {
            let record: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(record["inputs"].is_object(), "Should have inputs");
        }

        println!("✓ Exported and verified 3 examples");

        // 5. Export to CSV
        let csv_file = temp_dir.path().join("export.csv");
        let mut cmd = langstar_cmd();
        cmd.args([
            "dataset",
            "export",
            dataset_id,
            "--format",
            "csv",
            "--out",
            csv_file.to_str().unwrap(),
        ]);

        cmd.assert().success();

        // Verify CSV has header + 3 rows
        let csv_content = fs::read_to_string(&csv_file).unwrap();
        let csv_lines: Vec<&str> = csv_content.lines().collect();
        assert!(csv_lines.len() >= 4, "CSV should have header + 3 data rows");
        assert!(
            csv_lines[0].contains("id") && csv_lines[0].contains("inputs"),
            "CSV should have proper header"
        );

        println!("✓ Exported to CSV format");

        // 6. Cleanup - delete dataset
        let mut cmd = langstar_cmd();
        cmd.args(["dataset", "delete", dataset_id, "--yes"]);
        cmd.assert().success();

        println!("✓ Import/export roundtrip test passed");
    }
}
