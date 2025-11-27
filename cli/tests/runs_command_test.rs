//! CLI tests for `langstar runs` commands.
//!
//! These tests verify:
//! - CLI argument parsing and validation
//! - Help text completeness
//! - Output format options
//! - Error handling for invalid inputs
//! - Filter flag combinations
//! - Integration with LangSmith API (when credentials available)
//!
//! **Test Categories:**
//!
//! 1. **Unit tests** (no API access): Verify CLI parsing, help text, error handling
//! 2. **Integration tests** (requires API): Verify actual runs query behavior
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY` environment variable
//!
//! Run with: `cargo test --test runs_command_test`

mod common;

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
fn test_runs_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Query and manage LangSmith runs"))
        .stdout(predicate::str::contains("query"));
}

#[test]
fn test_runs_query_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--help"]);

    cmd.assert()
        .success()
        // Command description
        .stdout(predicate::str::contains("Query runs with filtering"))
        // Filter options
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--tag"))
        .stdout(predicate::str::contains("--meta"))
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--errors-only"))
        // Project filter
        .stdout(predicate::str::contains("--project"))
        // Time filters
        .stdout(predicate::str::contains("--since"))
        .stdout(predicate::str::contains("--until"))
        // Type filters
        .stdout(predicate::str::contains("--run-type"))
        .stdout(predicate::str::contains("--is-root"))
        // Output options
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--order"))
        // Scoping options
        .stdout(predicate::str::contains("--organization-id"))
        .stdout(predicate::str::contains("--workspace-id"));
}

#[test]
fn test_runs_query_help_shows_run_types() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--help"]);

    cmd.assert()
        .success()
        // Run type values should be shown
        .stdout(predicate::str::contains("llm"))
        .stdout(predicate::str::contains("chain"))
        .stdout(predicate::str::contains("tool"));
}

#[test]
fn test_runs_query_help_shows_output_formats() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--help"]);

    cmd.assert()
        .success()
        // Output format options
        .stdout(predicate::str::contains("table"))
        .stdout(predicate::str::contains("json"))
        .stdout(predicate::str::contains("json-pretty"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Argument Validation Tests (No API Access Required)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_invalid_run_type() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--run-type", "invalid-type"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_runs_query_invalid_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "xml"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_runs_query_invalid_order() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--order", "random"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_runs_query_invalid_limit_not_number() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--limit", "abc"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Filter Flag Combinations (Parsing Tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_accepts_multiple_tags() {
    // This test verifies the CLI accepts multiple --tag flags
    // It will fail at runtime without API key, but the parsing should succeed
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--tag",
        "production",
        "--tag",
        "gpt-4",
        "--limit",
        "1",
    ]);

    // Without API key, this will fail but not due to parsing
    let output = cmd.output().expect("Failed to execute command");

    // Should not be a clap parsing error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept multiple --tag flags"
    );
    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept multiple --tag flags"
    );
}

#[test]
fn test_runs_query_accepts_multiple_meta() {
    // Verify CLI accepts multiple --meta flags
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--meta",
        "environment=prod",
        "--meta",
        "model=gpt-4",
        "--limit",
        "1",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept multiple --meta flags"
    );
}

#[test]
fn test_runs_query_accepts_multiple_projects() {
    // Verify CLI accepts multiple --project flags
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--project",
        "00000000-0000-0000-0000-000000000001",
        "--project",
        "00000000-0000-0000-0000-000000000002",
        "--limit",
        "1",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept multiple --project flags"
    );
}

#[test]
fn test_runs_query_accepts_combined_filters() {
    // Verify CLI accepts a combination of filter flags
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--tag",
        "production",
        "--status",
        "error",
        "--run-type",
        "llm",
        "--is-root",
        "--errors-only",
        "--filter",
        "gt(total_tokens, 100)",
        "--limit",
        "1",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept combined filter flags"
    );
    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept combined filter flags"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Output Format Tests (Parsing Only)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_accepts_table_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "table", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept --output table"
    );
}

#[test]
fn test_runs_query_accepts_json_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "json", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept --output json"
    );
}

#[test]
fn test_runs_query_accepts_json_pretty_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "json-pretty", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept --output json-pretty"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// All Run Types Accepted Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_accepts_all_run_types() {
    let run_types = [
        "tool",
        "chain",
        "llm",
        "retriever",
        "embedding",
        "prompt",
        "parser",
    ];

    for run_type in run_types {
        let mut cmd = langstar_cmd();
        cmd.args(["runs", "query", "--run-type", run_type, "--limit", "1"]);

        let output = cmd.output().expect("Failed to execute command");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !stderr.contains("invalid value"),
            "CLI should accept --run-type {}",
            run_type
        );
    }
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
fn test_runs_query_without_api_key() {
    // Clear the API key to test error handling
    let mut cmd = langstar_cmd();
    cmd.env_remove("LANGSMITH_API_KEY");
    cmd.args(["runs", "query", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The command may succeed (code 0) but should have an authentication error in stderr
    // or it may fail with an auth-related message
    assert!(
        stderr.contains("API")
            || stderr.contains("api")
            || stderr.contains("key")
            || stderr.contains("Key")
            || stderr.contains("auth")
            || stderr.contains("Auth"),
        "Should mention API key, auth, or credential in error: {}",
        stderr
    );
}

#[test]
fn test_runs_query_basic() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--limit", "5", "--is-root"]);

    cmd.assert().success();

    println!("✓ CLI successfully queried runs");
}

#[test]
fn test_runs_query_json_output() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--limit",
        "3",
        "--is-root",
        "--output",
        "json",
    ]);

    let output = cmd.assert().success();
    let stdout_bytes = output.get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    // Output should be valid JSON (array)
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(parsed.is_array(), "JSON output should be an array");

    println!("✓ CLI successfully returned JSON output");
}

#[test]
fn test_runs_query_json_pretty_output() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--limit",
        "3",
        "--is-root",
        "--output",
        "json-pretty",
    ]);

    let output = cmd.assert().success();
    let stdout_bytes = output.get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    // Output should be valid JSON (array) with pretty formatting (has newlines)
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    assert!(parsed.is_array(), "JSON output should be an array");
    assert!(
        stdout.contains('\n'),
        "Pretty JSON should contain newlines for formatting"
    );

    println!("✓ CLI successfully returned pretty JSON output");
}

#[test]
fn test_runs_query_table_output() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--limit",
        "3",
        "--is-root",
        "--output",
        "table",
    ]);

    let output = cmd.assert().success();
    let stdout_bytes = output.get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    // Table output should contain either column headers or "Found X runs" message
    // (If no runs are found, headers might not be shown)
    assert!(
        stdout.contains("ID")
            || stdout.contains("Name")
            || stdout.contains("Type")
            || stdout.contains("Found")
            || stdout.contains("runs"),
        "Table output should contain column headers or summary: {}",
        stdout
    );

    println!("✓ CLI successfully returned table output");
}

#[test]
fn test_runs_query_with_run_type_filter() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--limit",
        "5",
        "--run-type",
        "llm",
        "--output",
        "json",
    ]);

    let output = cmd.assert().success();
    let stdout_bytes = output.get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    // Parse and verify all runs are of type 'llm'
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    if let Some(runs) = parsed.as_array() {
        for run in runs {
            if let Some(run_type) = run.get("run_type").and_then(|v| v.as_str()) {
                assert_eq!(run_type, "llm", "All runs should be of type 'llm'");
            }
        }
    }

    println!("✓ CLI successfully filtered runs by type");
}

#[test]
fn test_runs_query_with_raw_filter() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    // Test with a raw filter expression
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--limit",
        "3",
        "--filter",
        "eq(status, \"success\")",
        "--output",
        "json",
    ]);

    let output = cmd.assert().success();
    let stdout_bytes = output.get_output().stdout.clone();
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    // Parse and verify all runs have status 'success'
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("Output should be valid JSON");

    if let Some(runs) = parsed.as_array() {
        for run in runs {
            if let Some(status) = run.get("status").and_then(|v| v.as_str()) {
                assert_eq!(
                    status, "success",
                    "All runs should have status 'success' due to filter"
                );
            }
        }
    }

    println!("✓ CLI successfully applied raw filter expression");
}

#[test]
fn test_runs_query_with_order_asc() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--limit",
        "5",
        "--order",
        "asc",
        "--is-root",
    ]);

    cmd.assert().success();

    println!("✓ CLI successfully queried runs with ascending order");
}

#[test]
fn test_runs_query_with_order_desc() {
    if !has_api_credentials() {
        println!("Skipping test: LANGSMITH_API_KEY not set");
        return;
    }

    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--limit",
        "5",
        "--order",
        "desc",
        "--is-root",
    ]);

    cmd.assert().success();

    println!("✓ CLI successfully queried runs with descending order");
}
