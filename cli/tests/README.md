# Integration Tests

This directory contains integration tests for the Langstar CLI.

## Overview

Integration tests verify that the CLI works correctly end-to-end by:
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
cargo test --features integration-tests --test assistant_command_test --test graph_command_test -- --test-threads=1 --nocapture
```

**Specific test**:
```bash
cargo test --features integration-tests --test assistant_command_test test_assistant_create_basic -- --nocapture
```

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

**Test Deployments**: Each test creates its own deployment
- Validates create/delete commands work correctly
- Tests are marked with `#[cfg_attr(not(feature = "integration-tests"), ignore)]`
- Only enabled when running with `--features integration-tests`

### `prompt_scoping_test.rs`
Tests for LangSmith prompt scoping (org/workspace):
- These are unit tests and don't require test deployments
- Test configuration and scoping behavior

## Design Principles

### 1. Self-Sufficiency
Tests create and clean up their own resources. No manual setup required.

### 2. Isolation
Tests use unique deployment names (timestamp-based) to avoid collisions.

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
- Use `--test-threads=1` to run tests sequentially (required for assistant tests)
- Consider running only specific tests during development
- Integration tests are optimized for CI, not local iteration

## Contributing

When adding new integration tests:

1. **Use test fixtures** for deployment management
2. **Clean up resources** - always delete test deployments
3. **Use unique names** - include timestamp or random ID
4. **Document prerequisites** - list required env vars
5. **Handle missing credentials** - skip tests gracefully if env vars not set
6. **Add to CI** - ensure new tests run in GitHub Actions

## Related Documentation

- [GitHub Issue #160](https://github.com/codekiln/langstar/issues/160) - Deployment create/delete implementation
- [CLI Integration Testing Discussion](https://github.com/codekiln/langstar/issues/160#issuecomment-3547720154) - Integration test infrastructure design
- [LangGraph Control Plane API Docs](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref/)
