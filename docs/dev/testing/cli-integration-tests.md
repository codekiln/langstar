# CLI Integration Tests

> **📍 Centralized Testing Documentation**
>
> This document is part of the centralized testing documentation suite. See `docs/dev/testing/README.md` for the complete TOC.

## Overview

Integration tests verify that the Langstar CLI works correctly end-to-end by:
- Creating real deployments via the LangGraph Control Plane API
- Running CLI commands against those deployments
- Validating output and behavior
- Cleaning up test resources automatically

## Test Infrastructure

### Self-Sufficient Tests

All integration tests are **self-sufficient** - they manage their own test infrastructure:

- **`assistant_command_test.rs`**: Creates a shared test deployment for all assistant tests
- **`graph_command_test.rs`**: Contains lifecycle tests that create/delete their own deployments

### Test Fixtures (`common/fixtures.rs`)

The `TestDeployment` fixture provides automated deployment lifecycle management:

```rust
use common::fixtures::TestDeployment;

let deployment = TestDeployment::create();
// Use deployment.name and deployment.id in tests
// Deployment is automatically cleaned up when dropped
```

Features:
- Creates unique deployments with timestamp-based names
- Polls deployment until READY status
- Automatic cleanup on drop (RAII pattern)
- Detailed progress logging

## Running Tests

### Prerequisites

Set required environment variables:

```bash
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
```

### Running Locally

**Unit tests only** (fast, no API calls):
```bash
cargo test --workspace --lib
```

**Integration tests** (requires API access, creates real deployments):
```bash
cargo test --features integration-tests --test assistant_command_test --test graph_command_test -- --nocapture
```

**Specific test**:
```bash
cargo test --features integration-tests --test assistant_command_test test_assistant_create_basic -- --nocapture
```

### Test Parallelization

Integration tests support **selective parallelization** using the `serial_test` crate:

- **Most tests run in parallel** by default, improving CI performance
- **Tests with shared resources** are marked with `#[serial]` and run sequentially
- No need for `--test-threads=1` - the `#[serial]` attribute handles serialization automatically

**Which tests are marked `#[serial]`?**

Tests in `assistant_command_test.rs` that use the shared `TEST_DEPLOYMENT` via `OnceLock`:
- `test_assistant_create_basic`
- `test_assistant_lifecycle`
- `test_assistant_output_formats`
- `test_deployment_discovery_workflow`
- `test_error_handling_nonexistent_deployment`
- `test_assistant_list`
- `test_assistant_search`

**Why these tests need to be serial:**
- They share a single `TestDeployment` via `OnceLock<TestDeployment>`
- Parallel execution could cause resource conflicts on the shared deployment
- The `#[serial]` attribute ensures only one of these tests runs at a time

**Unique naming for parallel safety:**

Test resources use microsecond timestamps + UUID suffix for uniqueness:
```rust
fn generate_test_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .as_micros();
    let uuid_suffix = &Uuid::new_v4().to_string()[..8];
    format!("{}-{}-{}", prefix, timestamp, uuid_suffix)
}
```

This prevents name collisions even when tests run concurrently.

### Running in CI

Integration tests run automatically in GitHub Actions on:
- Pull requests to `main`
- Pushes to `main` branch

See `.github/workflows/ci.yml` for configuration.

## Test Organization

### `assistant_command_test.rs`
Tests for `langstar assistant` commands:
- Create, get, update, delete assistants
- Deployment discovery workflow
- Error handling (missing deployment, invalid inputs)
- Output formats (JSON, table)

**Test Deployment**: Shared across all tests via `OnceLock` pattern
- Created once on first test
- Reused by all subsequent tests
- Cleaned up automatically when test process exits

### `graph_command_test.rs`
Tests for `langstar graph` commands:
- List deployments with filters
- Create deployments (basic and with --wait)
- Delete deployments
- Full lifecycle test (create → list → delete → verify)
- Validation tests (invalid inputs, missing parameters)

**Test Deployments**: Uses standardized `TestDeploymentConfig` naming
- All graph command tests use `TestDeploymentConfig::for_release_tests()` for naming
- Creates `release-integration-test-{timestamp}` deployments
- Each test creates/deletes its own deployment (full lifecycle)
- Tests are marked with `#[cfg_attr(not(feature = "integration-tests"), ignore)]`
- Only enabled when running with `--features integration-tests`

### Standardized Test Deployment Naming

All CLI tests use the SDK's `TestDeploymentConfig` for consistent naming:

| Type | Pattern | Usage |
|------|---------|-------|
| **Release** | `release-integration-test-{timestamp}` | Graph command tests (full lifecycle) |

The `TestDeploymentConfig::for_release_tests()` creates deployments that:
- Have unique timestamp-based names
- Are self-cleaning (deleted after test completion)
- Are captured by the periodic cleanup workflow (4hr threshold)

### `prompt_scoping_test.rs`
Tests for LangSmith prompt scoping (org/workspace):
- These are unit tests and don't require test deployments
- Test configuration and scoping behavior

## Design Principles

### 1. Self-Sufficiency
Tests create and clean up their own resources. No manual setup required.

### 2. Isolation
Tests use unique deployment names (microsecond timestamp + UUID) to avoid collisions.

### 3. Idempotency
Tests can be run multiple times without side effects.

### 4. Cleanup
All test deployments are automatically deleted:
- Via `Drop` implementation on `TestDeployment`
- Via explicit cleanup in lifecycle tests

### 5. Performance
- Unit tests run without API calls (fast feedback)
- Integration tests run in CI only on PRs/main (avoid excessive API usage)
- Shared deployments reduce API calls and test time
- Most tests run in parallel for faster CI execution

### 6. Selective Serialization
- Tests with shared resources use `#[serial]` attribute
- Parallel-safe tests run concurrently by default
- No global `--test-threads=1` required

## Troubleshooting

### "Skipping test: Required environment variables not set"

**Cause**: Missing `LANGSMITH_API_KEY` or `LANGSMITH_WORKSPACE_ID`

**Solution**: Set environment variables:
```bash
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
```

### "Failed to create test deployment"

**Causes**:
- Invalid API key
- No access to workspace
- Rate limiting
- Network issues

**Solution**:
- Verify API key is valid
- Check workspace ID is correct
- Wait a few minutes if rate limited
- Check network connectivity

### "Test deployment should be initialized"

**Cause**: Test deployment creation failed in `get_test_deployment()`

**Solution**:
- Check prior error messages for deployment creation failure
- Verify environment variables are set correctly
- Ensure API key has permission to create deployments

### Tests Running Slowly

**Cause**: Deployments take 1-3 minutes to reach READY status

**Solution**:
- Tests with shared resources are marked `#[serial]` and run sequentially automatically
- Most tests run in parallel, improving overall execution time
- Consider running only specific tests during development
- Integration tests are optimized for CI, not local iteration

## Pre-Commit Checklist

Before committing and pushing code, **always run these checks locally** to catch issues before CI fails. The CI runs these exact checks, so running them locally prevents wasted time and unnecessary commits.

### Essential Checks (Run Every Time)

Run these commands from the project root (`/workspace`):

```bash
# 1. Format code (auto-fixes formatting issues)
cargo fmt

# 2. Check compilation for entire workspace
cargo check --workspace --all-features

# 3. Run clippy for linting warnings
cargo clippy --workspace --all-features -- -D warnings

# 4. Run all tests in workspace
cargo test --workspace --all-features

# 5. Check formatting (verifies cargo fmt was run)
cargo fmt --check
```

### Why Each Check Matters

**1. `cargo fmt`** (Auto-format)
- Fixes code formatting to match project style
- **Prevents**: "Check" CI job failures
- **Lesson from #75**: Forgot to run this, had to add formatting commit

**2. `cargo check --workspace`** (Compile check)
- Verifies code compiles across **entire workspace** (not just one crate)
- Much faster than full build
- **Prevents**: Build CI job failures
- **Lesson from #75**: Changed `AuthConfig::new()` signature, only tested SDK, missed CLI usage
- **Critical**: When making breaking changes to SDK, this catches all usages in CLI

**3. `cargo clippy`** (Linting)
- Catches common mistakes and non-idiomatic code
- **Prevents**: Clippy CI job failures
- Use `-- -D warnings` to treat warnings as errors (matches CI)

**4. `cargo test --workspace`** (All tests)
- Runs tests for **all crates** (SDK + CLI)
- **Prevents**: Test CI job failures
- **Lesson from #75**: Only ran `cargo test --lib` in SDK directory, missed workspace-level issues

**5. `cargo fmt --check`** (Verify formatting)
- Confirms formatting is correct (doesn't auto-fix)
- **Prevents**: CI formatting check failures
- Should pass after running `cargo fmt` in step 1

### Quick One-Liner

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features && \
cargo fmt --check
```

**Time investment**: ~1-2 minutes locally vs 10-20 minutes of CI roundtrips

### When to Run

- **Before every commit**: Catch issues early
- **After making changes**: Verify nothing broke
- **Before pushing**: Final check before CI runs
- **After resolving conflicts**: Ensure merge didn't break tests

### Special Considerations

**Working in Git Worktrees:**
- Some tests require environment variable sourcing
- Use the `test-runner-worktree` skill for proper test execution
- See `.claude/skills/test-runner-worktree/SKILL.md`

**Integration Tests:**
- Integration tests are slower (require API calls)
- Consider running unit tests only during rapid iteration:
  ```bash
  cargo test --workspace --lib
  ```
- Run full integration tests before final commit

## Contributing

When adding new integration tests:

1. **Use test fixtures** for deployment management
2. **Clean up resources** - always delete test deployments
3. **Use unique names** - use `generate_test_name()` with microsecond + UUID suffix
4. **Document prerequisites** - list required env vars
5. **Handle missing credentials** - skip tests gracefully if env vars not set
6. **Add to CI** - ensure new tests run in GitHub Actions
7. **Mark shared resource tests** - use `#[serial]` for tests using `OnceLock` or shared state
8. **Prefer parallelization** - only use `#[serial]` when truly necessary

## Related Documentation

- [GitHub Issue #160](https://github.com/codekiln/langstar/issues/160) - Deployment create/delete implementation
- [CLI Integration Testing Discussion](https://github.com/codekiln/langstar/issues/160#issuecomment-3547720154) - Integration test infrastructure design
- [LangGraph Control Plane API Docs](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref/)
- [Test Fixtures](./test-fixtures.md) - Test deployment configuration
- [SDK Integration Tests](./sdk-integration-tests.md) - SDK testing patterns
- [cli/tests/README.md](../../../cli/tests/README.md) - Quick reference (redirects here)
