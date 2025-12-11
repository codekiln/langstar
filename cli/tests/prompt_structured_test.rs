/// Integration tests for CLI structured prompt commands
///
/// These tests verify the `langstar prompt push/pull` commands with --schema flag.
///
/// **Test Coverage:**
/// - PRIVATE prompts (99% use case): Format `-/repo`, requires LANGSMITH_WORKSPACE_ID
/// - PUBLIC prompts (1% use case): Format `owner/repo`, must NOT have LANGSMITH_WORKSPACE_ID
///
/// **Prerequisites:**
/// - LANGSMITH_API_KEY environment variable
/// - LANGSMITH_ORGANIZATION_ID environment variable
/// - LANGSMITH_WORKSPACE_ID environment variable (for private prompt tests)
/// - Test repository: codekiln/langstar-structured-test
///
/// Run with: cargo test --features integration-tests --test prompt_structured_test -- --nocapture
use assert_cmd::Command;
use escargot::CargoBuild;
use predicates::prelude::*;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

const TEST_OWNER: &str = "codekiln";
const TEST_REPO: &str = "langstar-structured-test";
const TEST_REPO_PRIVATE: &str = "-/langstar-structured-test"; // Private prompt format

/// Check environment variables for PRIVATE prompt tests (99% use case)
/// Requires: LANGSMITH_API_KEY, LANGSMITH_ORGANIZATION_ID, LANGSMITH_WORKSPACE_ID
fn check_env_vars_private_prompts() {
    std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for private prompt tests");
}

/// Check environment variables for PUBLIC prompt tests (1% use case)
/// Requires: LANGSMITH_API_KEY, LANGSMITH_ORGANIZATION_ID
/// Unsets: LANGSMITH_WORKSPACE_ID (incompatible with owner/repo format)
fn check_env_vars_public_prompts() {
    std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    // SAFETY: Safe to remove env var in test setup before tests run
    unsafe {
        std::env::remove_var("LANGSMITH_WORKSPACE_ID");
    }
}

/// Helper to build and get the langstar binary path
fn get_langstar_bin() -> std::path::PathBuf {
    CargoBuild::new()
        .bin("langstar")
        .run()
        .expect("Failed to build langstar binary")
        .path()
        .to_owned()
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
fn test_cli_push_private_prompt() {
    check_env_vars_private_prompts();

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
    cmd.args([
        "prompt",
        "push",
        "--owner",
        "-",
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
fn test_cli_push_public_prompt_invalid_schema() {
    check_env_vars_public_prompts();

    let invalid_schema_file = create_temp_invalid_schema_file();
    let schema_path = invalid_schema_file.path().to_str().unwrap();

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
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
fn test_cli_push_public_prompt_missing_schema() {
    check_env_vars_public_prompts();

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
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
fn test_cli_push_public_prompt_invalid_method() {
    check_env_vars_public_prompts();

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
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
fn test_cli_push_function_calling_method_private_prompt() {
    check_env_vars_private_prompts();

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
    cmd.args([
        "prompt",
        "push",
        "--owner",
        "-",
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
fn test_cli_pull_private_prompt() {
    check_env_vars_private_prompts();

    let handle = TEST_REPO_PRIVATE;

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
    cmd.args(["prompt", "pull", "--", handle]);

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
fn test_cli_private_prompt_round_trip() {
    check_env_vars_private_prompts();

    // Step 1: Push a structured prompt
    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let bin = get_langstar_bin();
    let mut push_cmd = Command::new(&bin);
    push_cmd.args([
        "prompt",
        "push",
        "--owner",
        "-",
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
    let handle = TEST_REPO_PRIVATE;

    let bin = get_langstar_bin();
    let mut pull_cmd = Command::new(&bin);
    pull_cmd.args(["prompt", "pull", "--commit", commit_hash, "--", handle]);

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
fn test_cli_push_private_prompt_json_output() {
    check_env_vars_private_prompts();

    let schema_file = create_temp_schema_file();
    let schema_path = schema_file.path().to_str().unwrap();

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
    cmd.args([
        "prompt",
        "push",
        "--owner",
        "-",
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
fn test_cli_pull_private_prompt_json_output() {
    check_env_vars_private_prompts();

    let handle = TEST_REPO_PRIVATE;

    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
    cmd.args(["prompt", "pull", "--format", "json", "--", handle]);

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
