/// Integration tests for CLI structured prompt commands
///
/// These tests verify the `langstar prompt push/pull` commands with --schema flag.
///
/// **Test Coverage:**
/// - PRIVATE prompts (99% use case): Format `-/repo`, requires LANGSMITH_WORKSPACE_ID
/// - PUBLIC prompts (1% use case): Format `owner/repo`, must NOT have LANGSMITH_WORKSPACE_ID
///
/// **Test Pattern:**
/// Each test follows CRUD lifecycle with unique repo names to avoid conflicts:
/// 1. CREATE - Generate unique repo name and create via SDK
/// 2. TEST - Execute CLI command under test
/// 3. VERIFY - Verify CLI output and API state
/// 4. CLEANUP - Delete repo via SDK (even on test failure)
///
/// **Prerequisites:**
/// - LANGSMITH_API_KEY environment variable
/// - LANGSMITH_ORGANIZATION_ID environment variable
/// - LANGSMITH_WORKSPACE_ID environment variable (for private prompt tests)
///
/// Run with: cargo test --features integration-tests --test prompt_structured_test -- --nocapture
use assert_cmd::Command;
use escargot::CargoBuild;
use langstar_sdk::auth::AuthConfig;
use langstar_sdk::client::LangchainClient;
use predicates::prelude::*;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

const TEST_OWNER: &str = "codekiln";

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

/// Helper to create a tokio runtime for SDK operations
fn create_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
}

/// Helper to create an SDK client for CRUD operations
fn create_sdk_client() -> Result<LangchainClient, String> {
    let auth = AuthConfig::from_env().map_err(|e| format!("Auth config error: {}", e))?;
    LangchainClient::new(auth).map_err(|e| format!("Client creation error: {}", e))
}

/// Generate a unique test repo name to avoid conflicts between tests
fn generate_unique_repo_name(prefix: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}-{}", prefix, timestamp)
}

/// Test fixture that creates a unique repo and cleans it up on drop
struct PromptRepoFixture {
    repo_name: String,
    runtime: tokio::runtime::Runtime,
    client: LangchainClient,
    is_private: bool,
}

impl PromptRepoFixture {
    /// Create a new private prompt repo fixture
    fn new_private(prefix: &str) -> Self {
        let repo_name = generate_unique_repo_name(prefix);
        let runtime = create_runtime();
        let client = create_sdk_client().expect("Failed to create SDK client");

        // Create the repo via SDK
        println!("[SETUP] Creating private repo: -/{}", repo_name);
        runtime.block_on(async {
            client
                .prompts()
                .create_repo(
                    &repo_name,
                    Some(format!("Test repo for {}", prefix)),
                    None,
                    false, // is_public = false (private)
                    None,
                )
                .await
                .unwrap_or_else(|_| panic!("Failed to create test repo: {}", repo_name));
        });

        Self {
            repo_name,
            runtime,
            client,
            is_private: true,
        }
    }

    /// Create a new public prompt repo fixture
    fn new_public(prefix: &str) -> Self {
        let repo_name = generate_unique_repo_name(prefix);
        let runtime = create_runtime();
        let client = create_sdk_client().expect("Failed to create SDK client");

        // Create the repo via SDK
        println!("[SETUP] Creating public repo: {}/{}", TEST_OWNER, repo_name);
        runtime.block_on(async {
            client
                .prompts()
                .create_repo(
                    &repo_name,
                    Some(format!("Test repo for {}", prefix)),
                    None,
                    true, // is_public = true
                    None,
                )
                .await
                .unwrap_or_else(|_| panic!("Failed to create test repo: {}", repo_name));
        });

        Self {
            repo_name,
            runtime,
            client,
            is_private: false,
        }
    }

    /// Get the repo handle for CLI commands
    fn handle(&self) -> String {
        if self.is_private {
            format!("-/{}", self.repo_name)
        } else {
            format!("{}/{}", TEST_OWNER, self.repo_name)
        }
    }

    /// Get just the repo name
    fn repo_name(&self) -> &str {
        &self.repo_name
    }
}

impl Drop for PromptRepoFixture {
    fn drop(&mut self) {
        // Clean up the repo when the fixture goes out of scope
        println!("[CLEANUP] Deleting repo: {}", self.handle());
        let _ = self
            .runtime
            .block_on(async { self.client.prompts().delete(&self.repo_name).await });
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

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_private("test-push-private");

    // TEST: Push structured prompt via CLI
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
        fixture.repo_name(),
        "--template",
        "Answer this question: {question}",
        "--input-variables",
        "question",
        "--schema",
        schema_path,
        "--schema-method",
        "json_schema",
    ]);

    // VERIFY: CLI command succeeds with expected output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Prompt commit pushed successfully",
        ))
        .stdout(predicate::str::contains("Commit hash:"))
        .stdout(predicate::str::contains(
            "Structured prompt with JSON schema",
        ));

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_public_prompt_invalid_schema() {
    check_env_vars_public_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_public("test-invalid-schema");

    // TEST: Push with invalid schema file
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
        fixture.repo_name(),
        "--template",
        "Test template",
        "--schema",
        schema_path,
    ]);

    // VERIFY: CLI command fails with schema validation error
    cmd.assert().failure().stderr(
        predicate::str::contains("Schema validation failed")
            .or(predicate::str::contains("InvalidSchemaError")),
    );

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_public_prompt_missing_schema() {
    check_env_vars_public_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_public("test-missing-schema");

    // TEST: Push with nonexistent schema file path
    let bin = get_langstar_bin();
    let mut cmd = Command::new(&bin);
    cmd.args([
        "prompt",
        "push",
        "--owner",
        TEST_OWNER,
        "--repo",
        fixture.repo_name(),
        "--template",
        "Test template",
        "--schema",
        "/nonexistent/path/to/schema.json",
    ]);

    // VERIFY: CLI command fails with file not found error
    cmd.assert().failure().stderr(
        predicate::str::contains("Failed to read schema file")
            .or(predicate::str::contains("No such file or directory")),
    );

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_public_prompt_invalid_method() {
    check_env_vars_public_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_public("test-invalid-method");

    // TEST: Push with invalid schema method
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
        fixture.repo_name(),
        "--template",
        "Test template",
        "--schema",
        schema_path,
        "--schema-method",
        "invalid_method",
    ]);

    // VERIFY: CLI command fails with invalid method error
    cmd.assert().failure().stderr(
        predicate::str::contains("invalid_method")
            .or(predicate::str::contains("InvalidMethodError")),
    );

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_function_calling_method_private_prompt() {
    check_env_vars_private_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_private("test-function-calling");

    // TEST: Push structured prompt with function_calling method
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
        fixture.repo_name(),
        "--template",
        "Test with function calling",
        "--input-variables",
        "question",
        "--schema",
        schema_path,
        "--schema-method",
        "function_calling",
    ]);

    // VERIFY: CLI command succeeds
    cmd.assert().success().stdout(predicate::str::contains(
        "Prompt commit pushed successfully",
    ));

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_pull_private_prompt() {
    check_env_vars_private_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_private("test-pull-private");

    // Setup: First push a structured prompt to pull
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
        fixture.repo_name(),
        "--template",
        "Test pull: {input}",
        "--input-variables",
        "input",
        "--schema",
        schema_path,
    ]);
    push_cmd.assert().success();

    // TEST: Pull the structured prompt
    let mut cmd = Command::new(&bin);
    cmd.args(["prompt", "pull", "--", &fixture.handle()]);

    // VERIFY: CLI command succeeds with expected output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PROMPT MANIFEST"))
        .stdout(predicate::str::contains(
            "Structured prompt with JSON schema",
        ));

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_private_prompt_round_trip() {
    check_env_vars_private_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_private("test-round-trip");

    // TEST Step 1: Push a structured prompt
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
        fixture.repo_name(),
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

    // TEST Step 2: Pull it back
    let mut pull_cmd = Command::new(&bin);
    pull_cmd.args([
        "prompt",
        "pull",
        "--commit",
        commit_hash,
        "--",
        &fixture.handle(),
    ]);

    // VERIFY: Round-trip preserves structured prompt data
    pull_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Structured prompt with JSON schema",
        ))
        .stdout(predicate::str::contains("json_schema"))
        .stdout(predicate::str::contains("Round-trip test"));

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_private_prompt_json_output() {
    check_env_vars_private_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_private("test-json-output-push");

    // TEST: Push structured prompt with JSON output format
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
        fixture.repo_name(),
        "--template",
        "JSON output test",
        "--schema",
        schema_path,
        "--format",
        "json",
    ]);

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // VERIFY: JSON output can be parsed and has expected structure
    // Skip any non-JSON lines at the beginning (e.g., "✓ Repository exists")
    let json_str = stdout
        .lines()
        .skip_while(|line| !line.trim_start().starts_with('{'))
        .collect::<Vec<_>>()
        .join("\n");

    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
    assert!(json_result.is_ok(), "Output should contain valid JSON");

    let json = json_result.unwrap();
    assert!(
        json.get("commit").is_some(),
        "JSON should have commit field"
    );
    assert!(
        json["commit"].get("commit_hash").is_some(),
        "JSON should have commit_hash"
    );

    // CLEANUP: Automatic via Drop trait
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_pull_private_prompt_json_output() {
    check_env_vars_private_prompts();

    // CREATE: Setup test repo (cleaned up automatically on drop)
    let fixture = PromptRepoFixture::new_private("test-json-output-pull");

    // Setup: First push a structured prompt to pull
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
        fixture.repo_name(),
        "--template",
        "Test JSON pull: {input}",
        "--input-variables",
        "input",
        "--schema",
        schema_path,
    ]);
    push_cmd.assert().success();

    // TEST: Pull with JSON output format
    let mut cmd = Command::new(&bin);
    cmd.args([
        "prompt",
        "pull",
        "--format",
        "json",
        "--",
        &fixture.handle(),
    ]);

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // VERIFY: JSON output can be parsed and has expected structure
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(json_result.is_ok(), "Output should be valid JSON");

    let json = json_result.unwrap();
    // Should contain LC-JSON structure
    assert!(
        json.get("lc").is_some() || json.get("manifest").is_some(),
        "JSON should have lc or manifest field"
    );

    // CLEANUP: Automatic via Drop trait
}
