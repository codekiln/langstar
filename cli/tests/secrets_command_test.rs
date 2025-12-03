//! CLI tests for `langstar secrets` commands.
//!
//! These tests verify:
//! - CLI argument parsing and validation
//! - Help text completeness
//! - Error handling for invalid inputs
//! - Security feature validation (no `--value` flag)
//! - Mutual exclusivity of input method flags
//!
//! **Test Categories:**
//!
//! 1. **Unit tests** (no API access): Verify CLI parsing, help text, error handling
//! 2. **Integration tests** (requires API): Verify actual secrets operations
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY` environment variable
//!
//! Run with: `cargo test --test secrets_command_test`

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
fn test_secrets_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage LangSmith workspace secrets",
        ))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_secrets_list_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "list", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "List all workspace secret keys",
        ))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_secrets_set_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "set", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Set or update a workspace secret"))
        .stdout(predicate::str::contains("<KEY>"))
        .stdout(predicate::str::contains("--from-file"))
        .stdout(predicate::str::contains("--from-env"))
        .stdout(predicate::str::contains("--interactive"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_secrets_delete_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "delete", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Delete a workspace secret"))
        .stdout(predicate::str::contains("<KEY>"))
        .stdout(predicate::str::contains("--format"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Security Feature Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_secrets_set_no_value_flag() {
    // Verify that --value flag does NOT exist (security requirement)
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "set", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--value").not());
}

#[test]
fn test_secrets_set_value_flag_not_recognized() {
    // Attempting to use --value should fail
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "set", "TEST_KEY", "--value", "test"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Argument Validation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_secrets_set_missing_key() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "set"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_secrets_delete_missing_key() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "delete"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Mutual Exclusivity Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_secrets_set_from_file_and_from_env_conflict() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-file",
        "/tmp/test",
        "--from-env",
        "TEST_VAR",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_secrets_set_from_file_and_interactive_conflict() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-file",
        "/tmp/test",
        "--interactive",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_secrets_set_from_env_and_interactive_conflict() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-env",
        "TEST_VAR",
        "--interactive",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Output Format Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_secrets_list_accepts_format_flag() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "list", "--format", "json"]);

    // Will fail due to missing API key, but should parse arguments correctly
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--format").not()); // No format parsing error
}

#[test]
fn test_secrets_set_accepts_format_flag() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-env",
        "TEST_VAR",
        "--format",
        "json",
    ]);

    // Will fail due to missing API key, but should parse arguments correctly
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--format").not()); // No format parsing error
}

#[test]
fn test_secrets_delete_accepts_format_flag() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "delete", "TEST_KEY", "--format", "json"]);

    // Will fail due to missing API key, but should parse arguments correctly
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--format").not()); // No format parsing error
}
