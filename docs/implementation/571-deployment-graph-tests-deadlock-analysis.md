# Issue #571: Deployment and Graph Tests - Deadlock Analysis

**Status:** In Progress
**Branch:** `claude/571-5279-testing-comprehensive-tests-for-deployment-an`
**Issue:** https://github.com/codekiln/langstar/issues/571

## Problem Summary

Tests in `cli/tests/deployment_command_test.rs` are deadlocking or pausing indefinitely when run with the `integration-tests` feature flag.

## Investigation Findings

### What Was Added

This branch adds comprehensive tests for deployment and graph commands:

1. **cli/tests/deployment_command_test.rs** (658 lines)
   - CLI integration tests for `langstar deployment` commands
   - Uses shared `OnceLock<TestDeployment>` pattern

2. **sdk/tests/graph_test.rs** (579 lines)
   - SDK unit tests with mocked HTTP responses
   - Uses `#[tokio::test]` - these should work fine

3. **sdk/tests/graph_integration_test.rs** (321 new lines)
   - SDK integration tests for real API calls
   - Uses `#[tokio::test]` - these should work fine

### Root Cause: Missing `#[serial]` Attribute

**The Deadlock Pattern:**

The file `cli/tests/deployment_command_test.rs`:
- Uses `static TEST_DEPLOYMENT: OnceLock<TestDeployment>` (line 26)
- Has THREE tests that call `get_test_deployment()`:
  - `test_deployment_get_basic` (line 313)
  - `test_deployment_get_json_output` (line 351)
  - `test_deployment_secrets_redacted` (line 618)
- **Does NOT import `serial_test::serial`**
- **Does NOT mark tests with `#[serial]`**

**Why This Causes Deadlocks:**

1. **Concurrent OnceLock initialization**: When tests run in parallel (default cargo test behavior), multiple tests simultaneously attempt to initialize `TEST_DEPLOYMENT` via `OnceLock::get_or_init()`

2. **Nested tokio runtime creation**: `TestDeployment::create()` creates a NEW tokio runtime and blocks:
   ```rust
   // From cli/tests/common/fixtures.rs:92
   let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
   let result = runtime.block_on(async {
       // Long-running API calls here
       get_or_create_deployment(&client, &config).await
   });
   ```

3. **Long-running API operations**: `get_or_create_deployment()` can take 1-30 minutes as it:
   - Lists existing deployments
   - Waits for revision status to reach `Deployed`
   - Or creates a new deployment and waits

4. **Race condition**: Multiple concurrent attempts to:
   - Initialize the same OnceLock
   - Create separate tokio runtimes
   - Block on long-running async operations
   - Results in deadlock/indefinite hang

### Evidence: Working Pattern in assistant_command_test.rs

The `cli/tests/assistant_command_test.rs` file uses the **IDENTICAL pattern** but works correctly because:

```rust
// Line 6: Imports serial_test
use serial_test::serial;

// Line 12: Uses same OnceLock pattern
static TEST_DEPLOYMENT: OnceLock<TestDeployment> = OnceLock::new();

// Line 95-96: Marks tests with #[serial]
#[test]
#[serial]  // ← This is the key difference!
fn test_assistant_create_basic() {
    let Some((deployment_name, graph_id)) = get_test_deployment() else {
        println!("Skipping test: Required environment variables not set");
        return;
    };
    // ... test code
}
```

### Documentation Support

From `docs/dev/testing/cli-integration-tests.md` lines 76-92:

> **Which tests are marked `#[serial]`?**
>
> Tests in `assistant_command_test.rs` that use the shared `TEST_DEPLOYMENT` via `OnceLock`:
> - `test_assistant_create_basic`
> - `test_assistant_lifecycle`
> - ... (all tests using shared deployment)
>
> **Why these tests need to be serial:**
> - They share a single `TestDeployment` via `OnceLock<TestDeployment>`
> - Parallel execution could cause resource conflicts on the shared deployment
> - The `#[serial]` attribute ensures only one of these tests runs at a time

## Solution

Add serialization to tests that use the shared `OnceLock<TestDeployment>`:

### Step 1: Add Import

```rust
// At top of cli/tests/deployment_command_test.rs
use serial_test::serial;
```

### Step 2: Mark Affected Tests

Mark these three tests with `#[serial]`:

```rust
#[test]
#[serial]  // ← Add this
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_get_basic() { ... }

#[test]
#[serial]  // ← Add this
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_get_json_output() { ... }

#[test]
#[serial]  // ← Add this
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_deployment_secrets_redacted() { ... }
```

### Why This Works

1. **Sequential execution**: `#[serial]` ensures only one test using the shared resource runs at a time
2. **Single runtime creation**: Only one test creates a tokio runtime at a time
3. **No race conditions**: OnceLock initialization happens once, safely
4. **Matches established pattern**: Follows the same approach as `assistant_command_test.rs`

### Tests That Don't Need `#[serial]`

These tests do NOT call `get_test_deployment()` and can run in parallel:
- `test_deployment_list_basic`
- `test_deployment_list_with_limit`
- `test_deployment_list_json_output`
- `test_deployment_list_filter_by_type`
- `test_deployment_list_filter_by_status`
- `test_deployment_list_filter_by_name`
- `test_deployment_commands_help`
- `test_deployment_error_handling`

These tests make direct API calls without using the shared fixture, so parallel execution is safe.

## Dependencies

- `serial_test = "3"` is already in `cli/Cargo.toml` (line 55)
- No new dependencies needed

## Testing Strategy

After applying fix:

1. **Run tests sequentially to verify fix**:
   ```bash
   cd /workspace/wip/claude-571-5279-testing-comprehensive-tests-for-deployment-an
   source /workspace/.devcontainer/.env
   cargo test --features integration-tests --test deployment_command_test -- --nocapture
   ```

2. **Verify serialization works**:
   - Tests with `#[serial]` should run one at a time
   - Watch for "Initializing test deployment" message appearing only once
   - All tests should pass without hanging

3. **Run full pre-commit checks**:
   ```bash
   source /workspace/.devcontainer/.env
   cargo fmt && \
   cargo check --workspace --all-features && \
   cargo clippy --workspace --all-features -- -D warnings && \
   cargo test --workspace --all-features && \
   cargo fmt --check
   ```

## Lessons Learned

1. **Shared mutable state requires serialization**: When using `OnceLock` with long-running initialization, tests must be serialized

2. **Document the pattern**: The testing documentation already covers this pattern - we just need to follow it consistently

3. **Watch for nested runtime creation**: Creating new tokio runtimes in `block_on()` from synchronous tests requires careful coordination

4. **Pre-commit testing catches this**: If tests had been run locally before pushing, the deadlock would have been discovered

## References

- **Pattern documentation**: `docs/dev/testing/cli-integration-tests.md` lines 76-92
- **Working example**: `cli/tests/assistant_command_test.rs` lines 6, 95-96
- **Testing principles**: `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md`
- **Worktree testing**: `.claude/skills/test-runner-worktree/SKILL.md`

## Related Issues

- Issue #571: Parent issue for comprehensive testing
- Issue #527: Parent milestone (ls-graph-deployments-separation)
