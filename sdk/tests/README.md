# Integration Tests

This directory contains integration tests that make real API calls to LangSmith and other LangChain services.

## Running Integration Tests

Integration tests are marked with `#[ignore]` and require API keys to run.

### Prerequisites

Set the `LANGSMITH_API_KEY` environment variable:

```bash
export LANGSMITH_API_KEY="your-api-key-here"
```

### Run All Integration Tests

```bash
# Run all ignored (integration) tests
cargo test --test integration_test -- --ignored --nocapture

# Run specific integration test
cargo test --test integration_test test_list_prompts_from_prompthub -- --ignored --nocapture
```

### Run Unit Tests Only (Default)

```bash
# This skips integration tests (they're ignored by default)
cargo test
```

### Run Everything

```bash
# Run both unit and integration tests
cargo test -- --include-ignored
```

## Available Integration Tests

### `test_list_prompts_from_prompthub` ✅

**Status**: Working

Tests the ability to list prompts from the LangSmith PromptHub.

**Requirements**:
- Valid `LANGSMITH_API_KEY`
- Read permissions

**What it tests**:
- Authentication with LangSmith API
- Fetching and parsing paginated prompt list
- API response deserialization

### `test_push_prompt_to_prompthub` ⚠️

**Status**: Not working (405 Method Not Allowed)

Attempts to create/update a prompt in the LangSmith PromptHub.

**Current Issue**:
The LangSmith API returns 405 Method Not Allowed when attempting to create prompts via
`PUT /api/v1/repos/{handle}`. The correct endpoint or authentication method needs to be
determined.

**Possible causes**:
1. API key lacks write permissions
2. Different endpoint required (e.g., `/api/v1/commits`)
3. Additional authentication required
4. Prompts must be created via LangSmith web UI first

**Requirements** (when working):
- Valid `LANGSMITH_API_KEY` with **write permissions**
- Network connectivity to `api.smith.langchain.com`

## CI/CD Integration

Integration tests are **not** run in CI by default since they require API keys and make real API calls.

To run integration tests in CI:

1. Add `LANGSMITH_API_KEY` to GitHub Actions secrets
2. Update `.github/workflows/ci.yml` to include integration test job:

```yaml
integration-test:
  name: Integration Tests
  runs-on: ubuntu-latest
  if: github.event_name == 'push' && github.ref == 'refs/heads/main'
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - name: Run integration tests
      env:
        LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
      run: cargo test --test integration_test -- --ignored
```

## Best Practices

1. **Idempotency**: Integration tests should be safe to run multiple times
2. **Cleanup**: Tests should clean up any resources they create (when possible)
3. **Test Data**: Use clearly named test resources (e.g., `langstar-integration-test`)
4. **Timeouts**: Integration tests may be slower due to network calls
5. **Error Messages**: Provide helpful error messages for common failure modes

## Debugging Integration Tests

Run with full output to see detailed information:

```bash
RUST_LOG=debug cargo test --test integration_test -- --ignored --nocapture
```

Add backtrace for error details:

```bash
RUST_BACKTRACE=1 cargo test --test integration_test -- --ignored --nocapture
```

## Future Integration Tests

Potential tests to add:

- [ ] Test retrieving a specific prompt by handle
- [ ] Test searching prompts with query parameters
- [ ] Test LangGraph Cloud API endpoints
- [ ] Test authentication error handling
- [ ] Test rate limiting behavior
- [ ] Test pagination for large result sets
