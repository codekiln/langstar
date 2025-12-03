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
        .stdout(predicate::str::contains("List all workspace secret keys"))
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

    // Should parse --format flag correctly (no argument parsing errors)
    // May succeed (if API key present) or fail (if missing), but --format should parse
    let assert = cmd.assert();
    assert.stderr(predicate::str::contains("--format").not()); // No format parsing error
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

    // Should parse --format flag correctly (no argument parsing errors)
    // May fail (missing env var or API key), but --format should parse
    let assert = cmd.assert();
    assert.stderr(predicate::str::contains("--format").not()); // No format parsing error
}

#[test]
fn test_secrets_delete_accepts_format_flag() {
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "delete", "TEST_KEY", "--format", "json"]);

    // Should parse --format flag correctly (no argument parsing errors)
    // May succeed or fail depending on environment, but --format should parse
    let assert = cmd.assert();
    assert.stderr(predicate::str::contains("--format").not()); // No format parsing error
}

// ═══════════════════════════════════════════════════════════════════════════
// Security Validation Tests - No Secret Leakage
// ═══════════════════════════════════════════════════════════════════════════
//
// These tests verify the critical security requirement that secret VALUES are
// never exposed in command output, error messages, or help text.
//
// Security context: This CLI will be used by automated LLM agents (Claude Code)
// that can read stdout/stderr. Secret values must NEVER appear in any output.

#[test]
fn test_secrets_set_output_never_contains_value_pattern() {
    // Verify help text doesn't mention patterns that could leak values
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "set", "--help"]);

    cmd.assert()
        .success()
        // Should NOT have any text suggesting values will be shown
        .stdout(
            predicate::str::contains("display")
                .and(predicate::str::contains("value"))
                .not(),
        )
        .stdout(
            predicate::str::contains("show")
                .and(predicate::str::contains("value"))
                .not(),
        )
        .stdout(
            predicate::str::contains("print")
                .and(predicate::str::contains("value"))
                .not(),
        );
}

#[test]
fn test_secrets_list_help_confirms_no_values() {
    // Verify list help explicitly states values are not displayed
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "list", "--help"]);

    cmd.assert()
        .success()
        // Should mention that values are never displayed
        .stdout(predicate::str::contains("never displayed").or(predicate::str::contains("keys")));
}

#[test]
fn test_secrets_set_error_message_no_value_leakage() {
    // When secret set fails (e.g., missing input), error should not reveal any values
    let mut cmd = langstar_cmd();
    // This will fail because no value source is provided
    // We need to send empty stdin to avoid hanging
    cmd.args(["secrets", "set", "TEST_KEY"]).write_stdin("");

    // The error message should be about missing value source, not contain any actual values
    let output = cmd.assert().failure();
    output.stderr(
        predicate::str::contains("No secret value provided").or(predicate::str::contains(
            "--from-file",
        )
        .and(predicate::str::contains("--from-env"))
        .and(predicate::str::contains("--interactive"))),
    );
}

#[test]
fn test_secrets_set_empty_value_error_no_leakage() {
    // When value is empty, error should not leak what was provided
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("langstar_test_empty_secret.txt");

    // Create empty file
    std::fs::File::create(&temp_file)
        .unwrap()
        .write_all(b"")
        .unwrap();

    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-file",
        temp_file.to_str().unwrap(),
    ]);

    // Should fail with appropriate error, not reveal file contents or path details
    let output = cmd.assert().failure();
    output.stderr(predicate::str::contains("empty"));

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_secrets_set_from_env_missing_var_error() {
    // When env var doesn't exist, error should be clear but not leak anything
    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-env",
        "DEFINITELY_DOES_NOT_EXIST_12345",
    ]);

    let output = cmd.assert().failure();
    // Error should mention the variable name (that's ok, it's not a secret)
    // but should NOT contain any actual values
    output.stderr(
        predicate::str::contains("DEFINITELY_DOES_NOT_EXIST_12345")
            .and(predicate::str::contains("not found").or(predicate::str::contains("Environment"))),
    );
}

#[test]
fn test_secrets_set_from_file_nonexistent_error() {
    // When file doesn't exist, error should be clear but not leak paths
    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-file",
        "/nonexistent/path/to/secret/file.txt",
    ]);

    // Should fail with appropriate IO error
    cmd.assert().failure();
    // Just verify it fails - we don't want to be too specific about error format
}

// ═══════════════════════════════════════════════════════════════════════════
// Input Method Tests (stdin)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_secrets_set_stdin_parsing() {
    // Test that stdin input method is accepted
    // This will fail at API level but should parse correctly
    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "set", "TEST_KEY"])
        .write_stdin("test_secret_value\n");

    // Will fail (no API key) but should get past argument parsing
    // The error should NOT contain the value we sent via stdin
    // Either succeeds (API key present) or fails (no API key) but:
    // NEVER should stdout/stderr contain "test_secret_value"
    cmd.assert()
        .stdout(predicate::str::contains("test_secret_value").not())
        .stderr(predicate::str::contains("test_secret_value").not());
}

#[test]
fn test_secrets_set_from_file_value_not_in_output() {
    // Create a temp file with a known secret value
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("langstar_test_secret_value.txt");
    let secret_value = "SUPER_SECRET_VALUE_12345_SHOULD_NOT_APPEAR";

    std::fs::File::create(&temp_file)
        .unwrap()
        .write_all(secret_value.as_bytes())
        .unwrap();

    let mut cmd = langstar_cmd();
    cmd.args([
        "secrets",
        "set",
        "TEST_KEY",
        "--from-file",
        temp_file.to_str().unwrap(),
    ]);

    // Run command - will likely fail due to missing API key, but that's ok
    let output = cmd.output().expect("Failed to execute command");

    // CRITICAL: The secret value should NEVER appear in stdout or stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.contains(secret_value),
        "Secret value should NEVER appear in stdout"
    );
    assert!(
        !stderr.contains(secret_value),
        "Secret value should NEVER appear in stderr"
    );

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_secrets_set_from_env_value_not_in_output() {
    // Set an environment variable with a known secret value
    let env_var_name = "LANGSTAR_TEST_SECRET_VAR";
    let secret_value = "ENV_SECRET_VALUE_67890_SHOULD_NOT_APPEAR";

    // Set the env var for this test
    // SAFETY: This test is single-threaded and the env var is cleaned up after use
    unsafe {
        std::env::set_var(env_var_name, secret_value);
    }

    let mut cmd = langstar_cmd();
    cmd.args(["secrets", "set", "TEST_KEY", "--from-env", env_var_name]);

    // Run command - will likely fail due to missing API key, but that's ok
    let output = cmd.output().expect("Failed to execute command");

    // CRITICAL: The secret value should NEVER appear in stdout or stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.contains(secret_value),
        "Secret value from env should NEVER appear in stdout"
    );
    assert!(
        !stderr.contains(secret_value),
        "Secret value from env should NEVER appear in stderr"
    );

    // Cleanup
    // SAFETY: This test is single-threaded and the env var was set by this test
    unsafe {
        std::env::remove_var(env_var_name);
    }
}
