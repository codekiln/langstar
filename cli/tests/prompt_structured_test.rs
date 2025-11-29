/// Integration tests for CLI structured prompt commands
///
/// These tests verify the `langstar prompt push/pull` commands with --schema flag.
///
/// Prerequisites:
/// - LANGSMITH_API_KEY environment variable
/// - LANGSMITH_WORKSPACE_ID environment variable (required per CLI test pattern)
/// - Test repository: codekiln/langstar-structured-test (auto-created if needed)
///
/// Run with: cargo test --features integration-tests --test prompt_structured_test -- --nocapture
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

const TEST_OWNER: &str = "codekiln";
const TEST_REPO: &str = "langstar-structured-test";

/// Helper to check if required environment variables are set
fn check_env_vars() -> bool {
    std::env::var("LANGSMITH_API_KEY").is_ok() && std::env::var("LANGSMITH_WORKSPACE_ID").is_ok()
}

/// Helper to create a temporary valid JSON schema file
fn create_temp_schema_file() -> NamedTempFile {
    let schema = json!({
        "type": "object",
        "title": "TestOutput",
        "properties": {
            "answer": {
                "type": "string",
                "description": "The answer"
            },
            "confidence": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": "Confidence score"
            }
        },
        "required": ["answer"]
    });

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(schema.to_string().as_bytes())
        .expect("Failed to write schema");
    temp_file
}

/// Helper to create a temporary invalid JSON schema file
fn create_temp_invalid_schema_file() -> NamedTempFile {
    let invalid_schema = json!({
        "type": "invalid_type",
        "properties": {}
    });

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(invalid_schema.to_string().as_bytes())
        .expect("Failed to write invalid schema");
    temp_file
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_structured_prompt() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        TEST_REPO,
        "--template",
        "Answer this question: {question}",
        "--input-variables",
        "question",
        "--schema",
        schema_path,
        "--schema-method",
        "json_schema",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Prompt commit pushed successfully",
        ))
        .stdout(predicate::str::contains("Commit hash:"))
        .stdout(predicate::str::contains(
            "Structured prompt with JSON schema",
        ));
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_invalid_schema_file() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let invalid_schema_file = create_temp_invalid_schema_file();
    let schema_path = invalid_schema_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        TEST_REPO,
        "--template",
        "Test template",
        "--schema",
        schema_path,
    ]);

    cmd.assert().failure().stderr(
        predicate::str::contains("Schema validation failed")
            .or(predicate::str::contains("InvalidSchemaError")),
    );
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_missing_schema_file() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        TEST_REPO,
        "--template",
        "Test template",
        "--schema",
        "/nonexistent/path/to/schema.json",
    ]);

    cmd.assert().failure().stderr(
        predicate::str::contains("Failed to read schema file")
            .or(predicate::str::contains("No such file or directory")),
    );
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_invalid_method() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        TEST_REPO,
        "--template",
        "Test template",
        "--schema",
        schema_path,
        "--schema-method",
        "invalid_method",
    ]);

    cmd.assert().failure().stderr(
        predicate::str::contains("invalid_method")
            .or(predicate::str::contains("InvalidMethodError")),
    );
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_function_calling_method() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        TEST_REPO,
        "--template",
        "Test with function calling",
        "--input-variables",
        "question",
        "--schema",
        schema_path,
        "--schema-method",
        "function_calling",
    ]);

    cmd.assert().success().stdout(predicate::str::contains(
        "Prompt commit pushed successfully",
    ));
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_pull_structured_prompt() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let handle = format!("{}/{}", TEST_OWNER, TEST_REPO);

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args(["prompt", "pull", &handle]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Prompt Manifest"))
        .stdout(
            predicate::str::contains("Structured prompt with JSON schema")
                .or(predicate::str::contains("Type:").and(predicate::str::contains("Schema"))),
        );
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_structured_prompt_round_trip() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    // Step 1: Push a structured prompt
    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let mut push_cmd = Command::cargo_bin("langstar").unwrap();
    push_cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        TEST_REPO,
        "--template",
        "Round-trip test: {query}",
        "--input-variables",
        "query",
        "--schema",
        schema_path,
        "--schema-method",
        "json_schema",
    ]);

    let push_output = push_cmd.assert().success();
    let push_stdout = String::from_utf8_lossy(&push_output.get_output().stdout);

    // Extract commit hash from output
    let commit_hash = if let Some(line) = push_stdout.lines().find(|l| l.contains("Commit hash:")) {
        line.split("Commit hash:")
            .nth(1)
            .map(|s| s.trim())
            .expect("Failed to extract commit hash")
    } else {
        panic!("Commit hash not found in push output");
    };

    println!("Pushed with commit hash: {}", commit_hash);

    // Step 2: Pull it back
    let handle = format!("{}/{}", TEST_OWNER, TEST_REPO);

    let mut pull_cmd = Command::cargo_bin("langstar").unwrap();
    pull_cmd.args(["prompt", "pull", &handle, "--commit", commit_hash]);

    pull_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Structured prompt with JSON schema",
        ))
        .stdout(predicate::str::contains("json_schema"))
        .stdout(predicate::str::contains("Round-trip test"));
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_structured_prompt_json_output() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        TEST_REPO,
        "--template",
        "JSON output test",
        "--schema",
        schema_path,
        "--format",
        "json",
    ]);

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify JSON output can be parsed
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(json_result.is_ok(), "Output should be valid JSON");

    let json = json_result.unwrap();
    assert!(
        json.get("commit").is_some(),
        "JSON should have commit field"
    );
    assert!(
        json["commit"].get("commit_hash").is_some(),
        "JSON should have commit_hash"
    );
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_pull_structured_prompt_json_output() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;
    }

    let handle = format!("{}/{}", TEST_OWNER, TEST_REPO);

    let mut cmd = Command::cargo_bin("langstar").unwrap();
    cmd.args(["prompt", "pull", &handle, "--format", "json"]);

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify JSON output can be parsed
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(json_result.is_ok(), "Output should be valid JSON");

    let json = json_result.unwrap();
    // Should contain LC-JSON structure
    assert!(
        json.get("lc").is_some() || json.get("manifest").is_some(),
        "JSON should have lc or manifest field"
    );
}
