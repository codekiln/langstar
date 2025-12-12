# Test Plan: Project Commands (Issue #596)

**Parent Issue:** #586 (ls-projects milestone)
**Test Issue:** #596 (586.7-project-testing)
**Milestone:** #15

## Overview

This test plan provides comprehensive test coverage for the `langstar project` CLI commands and underlying SDK methods following the CRUD lifecycle pattern and Toyota andon cord principle.

## Testing Principles (from docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md)

- ✅ **Toyota Andon Cord**: Any failing test blocks merge
- ✅ **Verify Actual Behavior**: Test output content, not just exit codes
- ✅ **CRUD Lifecycle Pattern**: CLI → SDK bidirectional verification
- ✅ **Explicit Failures**: Use `.expect()` for missing env vars (no silent skips)
- ✅ **Cleanup**: Always clean up test data, even on failure

## Test File Structure

```
sdk/tests/
  └── project_test.rs          # SDK integration tests with httpmock

cli/tests/
  └── project_command_test.rs  # CLI integration tests with real API
```

## Required Environment Variables

All integration tests require (per docs/dev/environment-variables.md):

- `LANGSMITH_API_KEY` - API authentication
- `LANGSMITH_ORGANIZATION_ID` - Organization scoping
- `LANGSMITH_WORKSPACE_ID` - Workspace scoping

## SDK Integration Tests (sdk/tests/project_test.rs)

### Purpose

Verify SDK methods work correctly using httpmock to mock API responses.

### Test Cases

#### 1. Create Project Tests

**test_create_project_minimal**

- Mock: `POST /api/v1/sessions` with minimal body `{name: "Test Project"}`
- Verify: Response parsed correctly, returns `Project` with expected fields
- Pattern: Based on `dataset_test.rs::test_create_dataset()`

**test_create_project_with_description**

- Mock: `POST /api/v1/sessions` with name + description
- Verify: Description field preserved in response

**test_create_project_with_metadata**

- Mock: `POST /api/v1/sessions` with name + extra metadata JSON
- Verify: Metadata serialized correctly, response contains metadata

#### 2. List Projects Tests

**test_list_projects_all**

- Mock: `GET /api/v1/sessions` returns array of 2 projects
- Verify: Returns `Vec<Project>` with correct length, fields populated

**test_list_projects_with_name_filter**

- Mock: `GET /api/v1/sessions?name=my-project`
- Verify: Query parameter added correctly to request

**test_list_projects_with_name_contains_filter**

- Mock: `GET /api/v1/sessions?name_contains=prod`
- Verify: Query parameter added correctly

**test_list_projects_with_limit**

- Mock: `GET /api/v1/sessions?limit=5`
- Verify: Limit parameter passed correctly

**test_list_projects_with_include_stats**

- Mock: `GET /api/v1/sessions?include_stats=true`
- Mock response includes: `run_count`, `latency_p50`, `latency_p99`
- Verify: Stats fields populated in response

**test_list_projects_empty**

- Mock: `GET /api/v1/sessions` returns `[]`
- Verify: Returns empty vector, no errors

#### 3. Get Project Tests

**test_get_project_by_id**

- Mock: `GET /api/v1/sessions/{uuid}`
- Verify: Project returned with correct ID

**test_get_project_not_found**

- Mock: `GET /api/v1/sessions/{uuid}` returns 404
- Verify: Returns appropriate error

#### 4. Update Project Tests

**test_update_project_description**

- Mock: `PATCH /api/v1/sessions/{uuid}` with `{description: "Updated"}`
- Verify: Request body correct, response parsed

**test_update_project_name**

- Mock: `PATCH /api/v1/sessions/{uuid}` with `{name: "New Name"}`
- Verify: Name update request sent correctly

**test_update_project_metadata**

- Mock: `PATCH /api/v1/sessions/{uuid}` with `{extra: {...}}`
- Verify: Metadata serialized correctly

#### 5. Delete Project Tests

**test_delete_project**

- Mock: `DELETE /api/v1/sessions/{uuid}` returns 204
- Verify: Request sent correctly, no errors

**test_delete_project_not_found**

- Mock: `DELETE /api/v1/sessions/{uuid}` returns 404
- Verify: Returns appropriate error

### SDK Test Patterns (from dataset_test.rs)

```rust
use langstar_sdk::{AuthConfig, LangchainClient, ListProjectsParams, ProjectCreate};
use mockito::{Matcher, Server};
use serde_json::json;
use uuid::Uuid;

fn create_test_client(server_url: &str) -> LangchainClient {
    let auth = AuthConfig::new(Some("test_langsmith_key".to_string()), None, None);
    LangchainClient::with_base_urls(
        auth,
        server_url.to_string(),
        "https://api.langgraph.cloud".to_string(),
        "https://api.host.langchain.com".to_string(),
    )
    .expect("Failed to create test client")
}

#[tokio::test]
async fn test_create_project_minimal() {
    let mut server = Server::new_async().await;

    let response_body = json!({
        "id": "12345678-1234-1234-1234-123456789012",
        "name": "Test Project",
        "tenant_id": "87654321-4321-4321-4321-210987654321",
        "start_time": "2024-01-01T12:00:00Z"
    });

    let mock = server
        .mock("POST", "/api/v1/sessions")
        .match_body(Matcher::PartialJson(json!({"name": "Test Project"})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body.to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());
    let request = ProjectCreate {
        name: Some("Test Project".to_string()),
        ..Default::default()
    };

    let project = client.create_project(request).await
        .expect("create_project failed");

    assert_eq!(project.name, Some("Test Project".to_string()));
    mock.assert_async().await;
}
```

## CLI Integration Tests (cli/tests/project_command_test.rs)

### Purpose

Verify CLI commands work end-to-end with real API using CRUD lifecycle pattern.

### Test Organization

```rust
// ═══════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════

fn langstar_cmd() -> Command { /* ... */ }
fn create_sdk_client() -> Result<LangchainClient, String> { /* ... */ }
fn create_runtime() -> tokio::runtime::Runtime { /* ... */ }

fn generate_test_project_name() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("test-crud-lifecycle-{}", timestamp)
}

// ═══════════════════════════════════════════════════════════════════
// Help and Documentation Tests (No API required)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_project_help() { /* ... */ }

#[test]
fn test_project_list_help() { /* ... */ }

#[test]
fn test_project_create_help() { /* ... */ }

// ═══════════════════════════════════════════════════════════════════
// Integration Tests (Requires API credentials)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_project_crud_lifecycle() { /* Full lifecycle test */ }

#[test]
fn test_project_list_filtering() { /* List with filters */ }

#[test]
fn test_project_output_formats() { /* JSON vs table output */ }
```

### Test Cases

#### 1. Help Tests (No API Required)

**test_project_help**

- Run: `langstar project --help`
- Verify stdout contains:
  - "Manage LangSmith projects"
  - "list", "get", "create", "update", "delete"

**test_project_list_help**

- Run: `langstar project list --help`
- Verify stdout contains:
  - "--name", "--name-contains", "--limit", "--include-stats"
  - "-f, --format" (not `-o`)

**test_project_create_help**

- Run: `langstar project create --help`
- Verify stdout contains:
  - "name" (positional argument)
  - "--description", "--metadata"
  - "-f, --format"

**test_project_get_help**

- Run: `langstar project get --help`
- Verify: Help text mentions ID or name lookup

**test_project_update_help**

- Run: `langstar project update --help`
- Verify: Lists update fields (--name, --description)

**test_project_delete_help**

- Run: `langstar project delete --help`
- Verify: Shows --force flag

#### 2. CRUD Lifecycle Test (Full Integration)

**test_project_crud_lifecycle** - The critical test following the pattern from prompt_scoping_test.rs

```rust
#[test]
fn test_project_crud_lifecycle() {
    // ═══════════════════════════════════════════════════════════════
    // Setup - Explicit failure if env vars missing
    // ═══════════════════════════════════════════════════════════════
    let api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let runtime = create_runtime();
    let client = create_sdk_client().expect("SDK client required");
    let test_project_name = generate_test_project_name();

    // ═══════════════════════════════════════════════════════════════
    // Step 1: CREATE - Create project via SDK
    // ═══════════════════════════════════════════════════════════════
    println!("[CREATE] Creating test project via SDK: {}", test_project_name);

    let project = runtime.block_on(async {
        client.create_project(ProjectCreate {
            name: Some(test_project_name.clone()),
            description: Some("Test project for CRUD lifecycle".to_string()),
            ..Default::default()
        }).await
    }).expect("Failed to create test project");

    let project_id = project.id;
    println!("  ✓ Created project with ID: {}", project_id);

    // ═══════════════════════════════════════════════════════════════
    // Step 2: VERIFY - Confirm project exists via SDK read
    // ═══════════════════════════════════════════════════════════════
    println!("[VERIFY] Reading project via SDK...");

    let read_project = runtime.block_on(async {
        client.get_project(project_id).await
    }).expect("Failed to read project");

    assert_eq!(read_project.id, project_id);
    assert_eq!(read_project.name, Some(test_project_name.clone()));
    println!("  ✓ Verified project exists");

    // ═══════════════════════════════════════════════════════════════
    // Step 3: READ - Execute CLI list command
    // ═══════════════════════════════════════════════════════════════
    println!("[READ] Running CLI 'project list'...");

    let mut list_cmd = langstar_cmd();
    list_cmd.args([
        "project", "list",
        "--name", &test_project_name,
        "-f", "json",
    ]);

    let output = list_cmd.output().expect("Failed to execute CLI");
    assert!(output.status.success(), "CLI list command failed");

    // ═══════════════════════════════════════════════════════════════
    // Step 4: VERIFY - Parse output, confirm our project appears
    // ═══════════════════════════════════════════════════════════════
    println!("[VERIFY] Checking CLI output contains test project...");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let projects: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .expect("Failed to parse CLI JSON output");

    let found = projects.iter().any(|p| {
        p.get("id")
            .and_then(|v| v.as_str())
            .map(|id| id == project_id.to_string())
            .unwrap_or(false)
    });

    assert!(found,
        "BUG: Created project '{}' not found in CLI list output",
        test_project_name
    );
    println!("  ✓ Verified project appears in CLI list");

    // ═══════════════════════════════════════════════════════════════
    // Step 5: READ - Get specific project via CLI
    // ═══════════════════════════════════════════════════════════════
    println!("[READ] Running CLI 'project get'...");

    let mut get_cmd = langstar_cmd();
    get_cmd.args([
        "project", "get",
        &test_project_name,
        "-f", "json",
    ]);

    let output = get_cmd.output().expect("Failed to execute CLI get");
    assert!(output.status.success(), "CLI get command failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let got_project: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Failed to parse CLI get JSON output");

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
        "project", "update",
        &test_project_name,
        "--description", updated_description,
        "-f", "json",
    ]);

    let output = update_cmd.output().expect("Failed to execute CLI update");
    assert!(output.status.success(), "CLI update command failed");
    println!("  ✓ Update command succeeded");

    // ═══════════════════════════════════════════════════════════════
    // Step 7: VERIFY - Confirm update via SDK read
    // ═══════════════════════════════════════════════════════════════
    println!("[VERIFY] Confirming update via SDK...");

    let updated_project = runtime.block_on(async {
        client.get_project(project_id).await
    }).expect("Failed to read updated project");

    assert_eq!(
        updated_project.description,
        Some(updated_description.to_string())
    );
    println!("  ✓ Verified description updated");

    // ═══════════════════════════════════════════════════════════════
    // Step 8: DELETE - Clean up test data
    // ═══════════════════════════════════════════════════════════════
    println!("[DELETE] Cleaning up test project...");

    let _cleanup = runtime.block_on(async {
        client.delete_project(project_id).await
    });

    // Note: Don't panic on cleanup failure - test already passed
    println!("  ✓ Cleanup completed");
}
```

#### 3. List Filtering Tests

**test_project_list_with_name_contains**

- Create: 2 projects via SDK (`test-alpha-123`, `test-beta-456`)
- Run: `langstar project list --name-contains "alpha" -f json`
- Verify: Only `test-alpha-123` in results
- Cleanup: Delete both projects

**test_project_list_with_limit**

- Create: 5 projects via SDK
- Run: `langstar project list --limit 2 -f json`
- Verify: Exactly 2 projects returned
- Cleanup: Delete all 5 projects

**test_project_list_with_include_stats**

- Run: `langstar project list --include-stats -f json | head -1`
- Verify: Output includes fields like `run_count`, `latency_p50`
- Note: Don't verify exact values, just field presence

#### 4. Output Format Tests

**test_project_list_json_output**

- Run: `langstar project list -f json --limit 1`
- Verify:
  - Output is valid JSON array
  - Each object has `id`, `name`, `tenant_id` fields

**test_project_list_table_output**

- Run: `langstar project list -f table --limit 5`
- Verify:
  - Output contains table headers: "ID", "Name", "Description"
  - Output has row separators

**test_project_get_json_output**

- Create: 1 project via SDK
- Run: `langstar project get <name> -f json`
- Verify: Output is single JSON object (not array)
- Cleanup: Delete project

#### 5. Error Handling Tests

**test_project_get_not_found**

- Run: `langstar project get nonexistent-project-12345`
- Verify:
  - Exit code non-zero
  - Stderr contains "not found" or similar error message

**test_project_create_without_name**

- Run: `langstar project create` (missing required name argument)
- Verify: Exit code non-zero, shows usage help

**test_project_delete_confirmation**

- Create: 1 project via SDK
- Run: `langstar project delete <name>` (without --force)
- Verify: Prompts for confirmation (or fails if stdin not available in test)
- Cleanup: Delete with --force

**test_project_delete_force**

- Create: 1 project via SDK
- Run: `langstar project delete <name> --force`
- Verify:
  - Exit code 0
  - No confirmation prompt
  - Project deleted (verify via SDK)

#### 6. Metadata Tests

**test_project_create_with_metadata**

- Run: `langstar project create test-metadata --metadata '{"env":"staging"}' -f json`
- Verify:
  - Exit code 0
  - SDK read shows `extra` field contains `{"env":"staging"}`
- Cleanup: Delete project

**test_project_create_with_invalid_metadata_json**

- Run: `langstar project create test-bad-meta --metadata 'not-valid-json'`
- Verify:
  - Exit code non-zero
  - Error message mentions "invalid JSON"

## Test Coverage Summary

### SDK Tests (sdk/tests/project_test.rs)

- ✅ 5 create tests (minimal, description, metadata, error cases)
- ✅ 6 list tests (all, filters, pagination, empty)
- ✅ 2 get tests (success, not found)
- ✅ 3 update tests (description, name, metadata)
- ✅ 2 delete tests (success, not found)
- **Total: ~18 SDK tests**

### CLI Tests (cli/tests/project_command_test.rs)

- ✅ 6 help tests (one per subcommand)
- ✅ 1 full CRUD lifecycle test (CREATE → VERIFY → READ → VERIFY → UPDATE → VERIFY → DELETE)
- ✅ 3 list filtering tests
- ✅ 3 output format tests
- ✅ 4 error handling tests
- ✅ 2 metadata tests
- **Total: ~19 CLI tests**

### Anti-Patterns to Avoid

❌ **Exit code only verification**

```rust
// BAD
cmd.assert().success();
```

✅ **Verify actual behavior**

```rust
// GOOD
let output = cmd.output()?;
assert!(output.status.success());
let projects: Vec<Project> = serde_json::from_slice(&output.stdout)?;
assert!(!projects.is_empty());
assert_eq!(projects[0].name, Some(expected_name));
```

❌ **Silent skips for missing env vars**

```rust
// BAD
let Some(api_key) = std::env::var("LANGSMITH_API_KEY").ok() else {
    return; // Silent skip
};
```

✅ **Explicit failures**

```rust
// GOOD
let api_key = std::env::var("LANGSMITH_API_KEY")
    .expect("LANGSMITH_API_KEY must be set for integration tests");
```

❌ **No cleanup after test**

```rust
// BAD
let project = client.create_project(...).await?;
// Test ends without deletion
```

✅ **Always cleanup**

```rust
// GOOD
let project = client.create_project(...).await?;
// ... run tests ...
let _ = client.delete_project(project.id).await; // Best effort cleanup
```

## Implementation Order

1. ✅ **SDK tests first** (sdk/tests/project_test.rs)
   - Faster to implement (mocked responses)
   - Validates SDK methods independently
   - No API credentials required during development

2. ✅ **CLI help tests** (cli/tests/project_command_test.rs)
   - No API required
   - Fast feedback on CLI structure

3. ✅ **CLI CRUD lifecycle test** (cli/tests/project_command_test.rs)
   - The most critical test (prevents #536-style bugs)
   - Requires API credentials

4. ✅ **CLI additional tests** (cli/tests/project_command_test.rs)
   - Filtering, formats, error handling
   - Build on CRUD lifecycle foundation

## Pre-Commit Checklist

Before committing, run:

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo nextest run --profile ci --all-features --workspace && \
cargo fmt --check
```

**Time investment:** ~1-2 minutes locally vs 10-20 minutes CI roundtrips

## References

- **Testing Guidelines**: `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md`
- **CRUD Pattern**: `docs/dev/testing/crud-lifecycle-pattern.md`
- **Example SDK Test**: `sdk/tests/dataset_test.rs`
- **Example CLI Test**: `cli/tests/dataset_command_test.rs`
- **Example CRUD Test**: `cli/tests/prompt_scoping_test.rs::test_prompt_crud_lifecycle_private_visibility`
- **Environment Variables**: `docs/dev/environment-variables.md`
- **Issue #536 Post-Mortem**: `docs/dev/testing/post-mortems/536-prompt-list-testing-gap.md`
