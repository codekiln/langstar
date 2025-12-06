<!--
  SIZE LIMIT: This file SHOULD remain under 300 lines (currently ~310).
  Last checked: 2025-12-06
  If significantly exceeding limit, extract content to sub-document.
-->

# CRUD Lifecycle Testing Pattern

This document describes the CLI→SDK bidirectional verification pattern that prevents bugs like issue #536.

## The Problem: Anemic Testing (Issue #536)

### What Went Wrong

**Bug:** `langstar prompt list` returned zero results for private prompts

**Test coverage:** ✅ Tests existed and passed
**Actual behavior:** ❌ Feature was broken in production

### Why Tests Passed

The tests only checked exit codes, not actual behavior:

```rust
// cli/tests/prompt_scoping_test.rs (BEFORE the fix)

#[test]
fn test_prompt_list_with_org_id_from_env() {
    let mut cmd = langstar_cmd();
    cmd.args(["prompt", "list", "--limit", "5"]);

    let assert = cmd.assert();
    assert.success();  // ← Only checks exit code 0!
}
```

**What this test verified:**

- ✅ Command doesn't crash
- ✅ Exit code is 0

**What this test DIDN'T verify:**

- ❌ Prompts are actually returned
- ❌ Correct prompts are returned
- ❌ API was called with correct parameters
- ❌ Output format is correct

**Result:** Broken feature shipped because tests gave false confidence.

### Root Cause

The SDK was doing client-side filtering instead of passing `is_public` to the API. The API returned a default subset (mostly public prompts), then SDK filtered for private—resulting in zero matches.

See issue #536 for complete analysis.

## The Solution: CLI→SDK Bidirectional Verification

Integration tests must exercise the entire CRUD lifecycle using **both** the CLI and SDK to verify results.

### Pattern Overview

```
1. CREATE  → Create test data (deterministic, unique)
2. VERIFY  → Confirm creation via SDK (API state is correct)
3. READ    → Execute CLI command under test
4. VERIFY  → Parse CLI output, verify it matches expected data
5. UPDATE  → (If applicable) Modify via CLI
6. VERIFY  → Confirm update via SDK
7. DELETE  → Clean up test data
8. VERIFY  → Confirm deletion via SDK
```

### Why This Works

**CLI → SDK verification catches:**

- Commands that succeed but don't produce expected API state
- CLI commands that crash but SDK works
- Authentication/authorization issues

**SDK → CLI verification catches:**

- SDK works but CLI doesn't display results (this was #536!)
- CLI output formatting bugs
- Filtering/pagination issues

### Complete Example: Prompt List Test

This is the test pattern in `cli/tests/prompt_scoping_test.rs` (function `test_prompt_crud_lifecycle_private_visibility`):

```rust
#[test]
fn test_prompt_crud_lifecycle_private_visibility() {
    // Skip if required env vars not set
    let org_id = match get_org_id_or_skip() {
        Some(id) => id,
        None => return,
    };

    let runtime = create_runtime();
    let client = create_sdk_client().expect("SDK client required");
    let test_prompt_name = generate_test_prompt_name();

    // ═══════════════════════════════════════════════════════════════════
    // Step 1: CREATE - Create a private prompt via SDK
    // ═══════════════════════════════════════════════════════════════════
    println!("[CREATE] Creating private test prompt via SDK...");

    let prompt = runtime.block_on(async {
        client.prompts()
            .create_repo(
                &test_prompt_name,
                Some("Test prompt for CRUD lifecycle".to_string()),
                None,
                false,  // is_public = false (private)
                None,
            )
            .await
    }).expect("Failed to create test prompt");

    assert!(!prompt.is_public, "Prompt should be private");

    // ═══════════════════════════════════════════════════════════════════
    // Step 2: VERIFY - Confirm prompt exists via SDK read
    // ═══════════════════════════════════════════════════════════════════
    println!("[VERIFY] Reading prompt via SDK...");

    let read_prompt = runtime.block_on(async {
        client.prompts().get(&prompt.repo_handle).await
    }).expect("Failed to read prompt");

    assert_eq!(read_prompt.repo_handle, prompt.repo_handle);

    // ═══════════════════════════════════════════════════════════════════
    // Step 3: READ - Execute CLI list command
    // ═══════════════════════════════════════════════════════════════════
    println!("[READ] Running CLI 'prompt list'...");

    let mut list_cmd = langstar_cmd();
    list_cmd.args([
        "prompt", "list",
        "--limit", "20",
        "--organization-id", &org_id,
        "--format", "json",
    ]);

    let output = list_cmd.output().expect("Failed to execute CLI");
    assert!(output.status.success(), "CLI list command failed");

    // ═══════════════════════════════════════════════════════════════════
    // Step 4: VERIFY - Parse output, confirm our prompt appears
    // ═══════════════════════════════════════════════════════════════════
    println!("[VERIFY] Checking CLI output contains test prompt...");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_start = stdout.find('[').unwrap_or(0);
    let cli_prompts: Vec<Value> = serde_json::from_str(&stdout[json_start..])
        .expect("Failed to parse CLI JSON output");

    let found = cli_prompts.iter().any(|p| {
        p.get("repo_handle")
            .and_then(|v| v.as_str())
            .map(|h| h.ends_with(&test_prompt_name))
            .unwrap_or(false)
    });

    assert!(found,
        "BUG: Created private prompt '{}' not found in CLI list output. \
         This is the bug that issue #536 was supposed to fix!",
        test_prompt_name
    );

    // ═══════════════════════════════════════════════════════════════════
    // Step 5: DELETE - Clean up test data
    // ═══════════════════════════════════════════════════════════════════
    println!("[DELETE] Cleaning up test prompt...");

    let _ = runtime.block_on(async {
        client.prompts().delete(&test_prompt_name).await
    });
}
```

**Reference:** Full implementation in `cli/tests/prompt_scoping_test.rs` (search for `test_prompt_crud_lifecycle`)

## When to Use CRUD Pattern

### Required For

- ✅ CLI commands that create/modify/delete resources
- ✅ Features with API state changes
- ✅ Commands that list/query resources
- ✅ Any feature where "exit code 0" doesn't prove correctness

### Not Required For

- ❌ Pure CLI utilities (`--help`, `--version`)
- ❌ Configuration commands (no API calls)
- ❌ Commands that only display local state
- ❌ Unit tests for serialization/validation logic

## CRUD Test Checklist

Before considering an integration test complete:

- [ ] Test creates known data with unique identifiers (deterministic)
- [ ] Test verifies creation via SDK before testing CLI
- [ ] Test exercises CLI command with realistic arguments
- [ ] Test parses and verifies CLI output content (not just exit code)
- [ ] Test verifies expected data appears in output
- [ ] Test cleans up all created data (even on failure)
- [ ] Test can run multiple times without conflicts (idempotent)
- [ ] Test has descriptive assertions that explain failures

## Helper Functions

### Generate Unique Test Names

Prevent test collisions with unique names:

```rust
fn generate_test_prompt_name() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("test-crud-lifecycle-{}", timestamp)
}
```

**Reference:** `cli/tests/prompt_scoping_test.rs` function `generate_test_prompt_name`

### Create SDK Client for Verification

Helper to create SDK client from environment:

```rust
fn create_sdk_client() -> Result<LangchainClient, String> {
    let auth = AuthConfig::from_env()
        .map_err(|e| format!("Auth config error: {}", e))?;
    LangchainClient::new(auth)
        .map_err(|e| format!("Client creation error: {}", e))
}
```

**Reference:** `cli/tests/prompt_scoping_test.rs` function `create_sdk_client`

### Graceful Skip for Missing Environment

Allow tests to skip when environment not configured:

```rust
fn get_org_id_or_skip() -> Option<String> {
    match std::env::var("LANGSMITH_ORGANIZATION_ID") {
        Ok(id) if !id.is_empty() => Some(id),
        _ => None,
    }
}

// Usage in test:
let org_id = match get_org_id_or_skip() {
    Some(id) => id,
    None => {
        println!("Skipping: LANGSMITH_ORGANIZATION_ID not set");
        return;
    }
};
```

**Reference:** `cli/tests/prompt_scoping_test.rs` function `get_org_id_or_skip`

## Common Mistakes

### Mistake: Only Verify Creation, Not List

```rust
// ❌ INCOMPLETE - Doesn't test the list command
let prompt = client.prompts().create_repo(...).await?;
let read = client.prompts().get(&handle).await?;
assert_eq!(read.repo_handle, prompt.repo_handle);
// Missing: verify CLI list shows it!
```

### Mistake: Skip Verification After CLI

```rust
// ❌ INCOMPLETE - Doesn't verify CLI output
let mut cmd = langstar_cmd();
cmd.args(["prompt", "list", "--format", "json"]);
cmd.assert().success();  // ← Only exit code!
// Missing: parse JSON, verify expected prompts appear
```

### Mistake: Hard-Coded Test Data

```rust
// ❌ FLAKY - May conflict with other test runs
let name = "test-prompt";  // Same name every time!
```

## Related Documentation

- **High-Level Guidelines:** `HIGH_LEVEL_TESTING_GUIDELINES.md` - Core principles
- **CLI Integration Tests:** `cli-integration-tests.md` - CLI-specific patterns
- **SDK Integration Tests:** `sdk-integration-tests.md` - SDK-specific patterns
- **Post-Mortem #536:** `post-mortems/536-prompt-list-testing-gap.md` - Full analysis
