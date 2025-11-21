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
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"

# Required for assistant integration tests
export TEST_GRAPH_ID="<graph-id-from-deployment>"

# Optional: For specific test scenarios
export TEST_DEPLOYMENT_ID="<deployment-id>"
export TEST_ASSISTANT_ID="<existing-assistant-id>"
```

**Where to find these values:**

- **LANGSMITH_API_KEY**: https://smith.langchain.com/settings → "API Keys"
- **LANGSMITH_WORKSPACE_ID**: LangSmith UI → Settings → Workspace ID
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
3. Verify `LANGSMITH_WORKSPACE_ID` matches your workspace

---

**Error:** "404 Not Found" when creating assistant

**Solution:**
1. Verify test graph deployment is active
2. Check deployment status in LangSmith UI
3. Confirm Graph ID is from an active deployment

---

## Deployment Workflow Integration Tests

### Overview

The `integration_deployment_workflow.rs` file contains tests for LangGraph Cloud deployment operations including:
- Full deployment lifecycle (create, patch, poll, delete)
- GitHub integration discovery and repository access
- Deployment URL extraction helpers
- RAII cleanup guards for resource management

### Prerequisites

**Required Environment Variables:**
```bash
export LANGSMITH_API_KEY="<your-api-key>"              # Required
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"    # Required
```

**Optional Environment Variables:**
```bash
export REPOSITORY_OWNER="codekiln"                     # Default: "codekiln"
export REPOSITORY_NAME="langstar"                      # Default: "langstar"
```

**GitHub Integration Setup:**
- At least one GitHub integration must be configured in your workspace
- Integration must have access to the target repository
- Repository must contain `tests/fixtures/test-graph-deployment/langgraph.json`

### Running Deployment Tests

**Full deployment workflow (5-30 minutes, persistent deployment):**
```bash
cargo test --test integration_deployment_workflow test_deployment_workflow -- --ignored --nocapture
```

**Full lifecycle deployment workflow (pre-release validation, 20-30 minutes):**
```bash
cargo test --test integration_deployment_workflow test_deployment_workflow_full_lifecycle -- --ignored --nocapture
```

**List deployments:**
```bash
cargo test --test integration_deployment_workflow test_list_deployments -- --ignored --nocapture
```

**List GitHub integrations:**
```bash
cargo test --test integration_deployment_workflow test_list_github_integrations -- --ignored --nocapture
```

**List repositories:**
```bash
cargo test --test integration_deployment_workflow test_list_github_repositories -- --ignored --nocapture
```

**Find integration for repo:**
```bash
cargo test --test integration_deployment_workflow test_find_integration_for_repo -- --ignored --nocapture
```

**Unit test (no API, <1s):**
```bash
cargo test --test integration_deployment_workflow test_deployment_url_extraction
```

### Available Deployment Tests

#### `test_deployment_workflow` (Persistent Deployment)

**Duration:** 5-30 minutes (first run: ~22 min, subsequent runs: ~6 min)

**What it tests:**
1. Finding GitHub integration ID dynamically
2. Using a persistent deployment named "langstar-integration-test" with get-or-create pattern
3. Listing revisions and polling first revision to DEPLOYED status
4. Patching deployment (triggers new revision)
5. Polling second revision to DEPLOYED status
6. Validating deployment source, URL, and final status
7. Reuses existing deployment; does not perform cleanup

**Validations:**
- ✅ Deployment source is "github"
- ✅ Deployment has custom_url in source_config
- ✅ Final revision status is Deployed

**Performance:**
- First run creates deployment (~22 minutes)
- Subsequent runs reuse deployment (~6 minutes, 73% time reduction)

#### `test_deployment_workflow_full_lifecycle` (Complete Lifecycle)

**Duration:** 20-30 minutes

**What it tests:**
1. Finding GitHub integration ID dynamically
2. Creating deployment with timestamp-based unique name
3. Listing revisions and polling first revision to DEPLOYED status
4. Patching deployment (triggers new revision)
5. Polling second revision to DEPLOYED status
6. Validating deployment source, URL, and final status
7. Cleanup with RAII guard (deletes deployment after test)

**Validations:**
- ✅ Deployment source is "github"
- ✅ Deployment has custom_url in source_config
- ✅ Final revision status is Deployed
- ✅ Automatic cleanup on test failure

**Use Cases:**
- Pre-release validation requiring complete create/delete cycle
- Testing deployment cleanup functionality
- Scenarios requiring a fresh deployment each run

Creates a uniquely-named deployment and performs a complete lifecycle test including cleanup. This test creates a new deployment each run with RAII guard protection.

#### `test_list_deployments` (Read-Only)

**Duration:** 1-3 seconds

Lists deployments with limit parameter to verify basic API functionality.

#### `test_list_github_integrations` (Read-Only)

**Duration:** 1-3 seconds

Lists all configured GitHub integrations for the workspace. Shows integration ID, name, provider, and organization.

#### `test_list_github_repositories` (Read-Only)

**Duration:** 2-5 seconds

Lists all repositories accessible through a GitHub integration. Validates integration permissions and repository access.

#### `test_find_integration_for_repo` (Key Operation)

**Duration:** 2-5 seconds

Finds the correct GitHub integration for a given repository owner and name. This is the core operation used by the deployment workflow to dynamically discover integration IDs.

#### `test_deployment_url_extraction` (Unit Test)

**Duration:** <1 second

Pure unit test that validates the `custom_url()` helper method for extracting deployment URLs from source_config JSON. No API calls.

### DeploymentGuard (RAII Cleanup)

The `DeploymentGuard` struct provides automatic cleanup to prevent orphaned deployments when tests fail:

```rust
// Create deployment
let deployment = client.deployments().create(&request).await?;

// Guard ensures cleanup reminder on test failure
let mut guard = DeploymentGuard::new(deployment.id.clone());

// ... test operations that might fail ...

// Manually delete and disarm guard on success
client.deployments().delete(&deployment.id).await?;
guard.disarm();  // Prevents warning about manual cleanup
```

**Features:**
- Implements Drop trait for warning-based cleanup reminders
- Drop implementation prints warnings via `eprintln!` (no automatic cleanup performed)
- Provides `disarm()` method to skip warning after manual deletion
- Prints deployment ID and cleanup instructions for debugging
- Note: Automatic cleanup from Drop is not supported in async contexts

### Troubleshooting Deployment Tests

**"Failed to find GitHub integration for repository"**

Solution:
1. Verify GitHub integration exists in LangSmith UI
2. Check integration has repository access configured
3. Verify `REPOSITORY_OWNER` and `REPOSITORY_NAME` environment variables

**"Timeout waiting for revision to be DEPLOYED after 30 minutes"**

Solution:
1. Check deployment status in LangSmith UI
2. Review deployment logs for build errors
3. Verify `langgraph.json` is valid at the specified path
4. Confirm GitHub integration has proper permissions

**"Failed to list GitHub repositories: Forbidden"**

Solution:
1. Verify API key has integration read permissions
2. Check workspace ID is correct
3. Ensure integration is in the same workspace

## Future Integration Tests

Potential tests to add:

- [ ] Test retrieving a specific prompt by handle
- [ ] Test searching prompts with query parameters
- [ ] Test authentication error handling
- [ ] Test rate limiting behavior
- [ ] Test pagination for large result sets
- [x] Test assistant CRUD operations (Phase 5)
- [x] Test assistant search functionality (Phase 5)
- [x] Test LangGraph Cloud deployment workflow (Issue #185)
- [x] Test GitHub integration operations (Issue #186)
- [x] Test RAII cleanup guards (Issue #186)
