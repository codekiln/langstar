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

### `test_push_prompt_to_prompthub` ✅

**Status**: Working (requires prompt repository to exist first)

Creates a new commit for a prompt in the LangSmith PromptHub.

**Prerequisites**:
1. Valid `LANGSMITH_API_KEY` with **write permissions**
2. The prompt repository must already exist in your PromptHub
   - Create at: https://smith.langchain.com/prompts
   - Test uses: `codekiln/langstar-integration-test`

**What it tests**:
- Fetching current organization information
- Setting X-Organization-Id header for write operations
- Creating a commit using `POST /api/v1/commits/{owner}/{repo}`
- Proper request body format (manifest, parent_commit, example_run_ids)

**Expected behavior**:
- If repository exists: ✅ Returns commit hash
- If repository doesn't exist: ❌ Returns 404 "Repository not found"

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

## Assistant Integration Tests

### Prerequisites

**1. Deploy Test LangGraph Application**

Before running assistant integration tests, you must deploy the test graph:

1. Follow the deployment guide: `../../tests/fixtures/test-graph-deployment/DEPLOYMENT_GUIDE.md`
2. Deploy to LangGraph Cloud via LangSmith UI
3. Note the **Graph ID** from the deployment

**2. Set Required Environment Variables**

```bash
# Required for all tests
export LANGSMITH_API_KEY="<your-api-key>"
export LANGCHAIN_WORKSPACE_ID="<your-workspace-id>"

# Required for assistant integration tests
export TEST_GRAPH_ID="<graph-id-from-deployment>"

# Optional: For specific test scenarios
export TEST_DEPLOYMENT_ID="<deployment-id>"
export TEST_ASSISTANT_ID="<existing-assistant-id>"
```

**Where to find these values:**

- **LANGSMITH_API_KEY**: https://smith.langchain.com/settings → "API Keys"
- **LANGCHAIN_WORKSPACE_ID**: LangSmith UI → Settings → Workspace ID
- **TEST_GRAPH_ID**: Deployment details page → "Graph ID" field (after deploying test graph)
- **TEST_DEPLOYMENT_ID**: Deployment details page (optional)

### Running Assistant Tests

```bash
# Run all assistant integration tests
cargo test --test assistant_integration_test -- --ignored --nocapture

# Run specific assistant test
cargo test --test assistant_integration_test test_assistant_lifecycle -- --ignored --nocapture
```

### Available Assistant Tests

**Note**: Assistant integration tests are part of Phase 5 (Issue #94) and will be implemented after Phase 4 deployment setup is complete.

**Planned tests include:**

- `test_assistant_lifecycle` - Full CRUD lifecycle (create, get, update, delete)
- `test_assistant_search_exact_match` - Search by exact name
- `test_assistant_search_partial_match` - Search with partial name matching
- `test_assistant_search_no_results` - Search with no matching results
- `test_assistant_list` - List all assistants with pagination
- `test_assistant_error_handling` - 404, auth failures, invalid configs

**Test Cleanup:**

Assistant tests use timestamped names (e.g., `test-assistant-1234567890`) to avoid conflicts. Tests include cleanup steps to delete created assistants. If tests fail midway, you may need to manually clean up:

```bash
# List test assistants (once CLI is implemented)
langstar assistant list | grep "test-assistant"

# Delete manually via LangSmith UI or CLI
langstar assistant delete <assistant-id>
```

### Troubleshooting Assistant Tests

**Error:** "TEST_GRAPH_ID environment variable not set"

**Solution:**
1. Deploy test graph (see `../../tests/fixtures/test-graph-deployment/DEPLOYMENT_GUIDE.md`)
2. Set `TEST_GRAPH_ID` environment variable

---

**Error:** "Invalid graph_id"

**Solution:**
1. Verify deployment is active in LangSmith UI
2. Check Graph ID matches exactly (case-sensitive)
3. Ensure workspace ID is correct

---

**Error:** "Authentication failed"

**Solution:**
1. Verify `LANGSMITH_API_KEY` is valid
2. Check API key has "Assistants" permissions
3. Verify `LANGCHAIN_WORKSPACE_ID` matches your workspace

---

**Error:** "404 Not Found" when creating assistant

**Solution:**
1. Verify test graph deployment is active
2. Check deployment status in LangSmith UI
3. Confirm Graph ID is from an active deployment

---

## Future Integration Tests

Potential tests to add:

- [ ] Test retrieving a specific prompt by handle
- [ ] Test searching prompts with query parameters
- [ ] Test LangGraph Cloud deployment listing and details
- [ ] Test authentication error handling
- [ ] Test rate limiting behavior
- [ ] Test pagination for large result sets
- [x] Test assistant CRUD operations (Phase 5)
- [x] Test assistant search functionality (Phase 5)
