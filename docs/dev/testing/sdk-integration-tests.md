# SDK Integration Tests

> **📍 Centralized Testing Documentation**
>
> This document is part of the centralized testing documentation suite. See `docs/dev/testing/README.md` for the complete TOC.

## Overview

The SDK contains integration tests that make real API calls to LangSmith and other LangChain services. These tests verify end-to-end functionality of the SDK against live APIs.

## Key Concepts

### Deployment vs Revision Status

LangGraph Cloud has two distinct status types - see [langgraph-deployments-and-revisions.md](../../langgraph-deployments-and-revisions.md) for details:

- **DeploymentStatus** (e.g., `Ready`) - Overall deployment state
- **RevisionStatus** (e.g., `Deployed`) - Build/deploy state of a specific revision

Test fixtures wait for `RevisionStatus::Deployed`, not `DeploymentStatus::Ready`.

### Test Deployment Naming

Integration tests use exactly **two deployment types** with standardized naming:

| Type | Pattern | Behavior |
|------|---------|----------|
| **PR/Dev** | `pr-integration-test-{timestamp}` | Get-or-create by prefix, cleaned by cron (4hr threshold) |
| **Release** | `release-integration-test-{timestamp}` | Always fresh, self-deleting |

**Using TestDeploymentConfig:**
- `TestDeploymentConfig::default()` - Creates PR/dev config with prefix-based reuse
- `TestDeploymentConfig::for_release_tests()` - Creates release config (always fresh)

The `get_or_create_deployment()` function handles:
- **With prefix**: Searches for existing `pr-integration-test-*` deployment, reuses if found
- **Without prefix**: Always creates a new deployment (for release tests)

## Running Integration Tests

Integration tests are marked with `#[ignore]` and require API keys to run.

### Prerequisites

Set the `LANGSMITH_API_KEY` environment variable:

```bash
export LANGSMITH_API_KEY="<your-api-key>"
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

### PromptHub Tests

#### `test_list_prompts_from_prompthub` ✅

**Status**: Working

Tests the ability to list prompts from the LangSmith PromptHub.

**Requirements**:
- Valid `LANGSMITH_API_KEY`
- Read permissions

**What it tests**:
- Authentication with LangSmith API
- Fetching and parsing paginated prompt list
- API response deserialization

#### `test_push_prompt_to_prompthub` ✅

**Status**: Working

Creates a new commit for a private prompt in the LangSmith PromptHub using the null repository (`-`).

**Prerequisites**:
1. Valid `LANGSMITH_API_KEY` with **write permissions**

**What it tests**:
- Fetching current organization information
- Setting X-Organization-Id header for write operations
- Creating a commit using `POST /api/v1/commits/{owner}/{repo}`
- Proper request body format (manifest, parent_commit, example_run_ids)

**Expected behavior**:
- If repository exists: ✅ Returns commit hash
- If repository doesn't exist: ❌ Returns 404 "Repository not found"

### Assistant Tests

#### Prerequisites

**1. Deploy Test LangGraph Application**

Before running assistant integration tests, you must deploy the test graph:

1. Follow the deployment guide: `../../../tests/fixtures/test-graph-deployment/DEPLOYMENT_GUIDE.md`
2. Deploy to LangGraph Cloud via LangSmith UI
3. Note the **Graph ID** from the deployment

**2. Set Required Environment Variables**

```bash
# Required for all tests
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
```

**Note:** Tests auto-discover GitHub integrations and create deployments as needed. Manual TEST_GRAPH_ID configuration is no longer required.

#### Running Assistant Tests

```bash
# Run all assistant integration tests
cargo test --test assistant_integration_test -- --ignored --nocapture

# Run specific assistant test
cargo test --test assistant_integration_test test_assistant_lifecycle -- --ignored --nocapture
```

#### Available Assistant Tests

**Available tests:**

- `test_assistant_lifecycle` - Full CRUD lifecycle (create, get, update, delete)
- `test_assistant_search` - Search functionality

**Test Cleanup:**

Assistant tests use timestamped names (e.g., `test-assistant-1234567890`) to avoid conflicts. Tests include cleanup steps to delete created assistants. If tests fail midway, you may need to manually clean up:

```bash
# List test assistants (once CLI is implemented)
langstar assistant list | grep "test-assistant"

# Delete manually via LangSmith UI or CLI
langstar assistant delete <assistant-id>
```

### Deployment Workflow Integration Tests

#### Overview

The `integration_deployment_workflow.rs` file contains tests for LangGraph Cloud deployment operations including:
- Full deployment lifecycle (create, patch, poll, delete)
- GitHub integration discovery and repository access
- Deployment URL extraction helpers
- RAII cleanup guards for resource management

#### Prerequisites

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

#### Running Deployment Tests

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

#### Available Deployment Tests

##### `test_deployment_workflow` (Shared PR/Dev Deployment)

**Duration:** 5-30 minutes (first run: ~22 min, subsequent runs: ~6 min)

**What it tests:**
1. Get or create deployment using `TestDeploymentConfig::default()` (prefix-based reuse)
2. Validates deployment is ready (RevisionStatus::Deployed)
3. Patching deployment (triggers new revision)
4. Polling new revision to DEPLOYED status
5. Validating deployment source, URL, and final status
6. Leaves deployment running (cleaned by periodic cron job after 4 hours)

**Validations:**
- ✅ Deployment source is "github"
- ✅ Deployment has custom_url in source_config
- ✅ Final revision status is Deployed

**Performance:**
- First run creates `pr-integration-test-{ts}` deployment (~22 minutes)
- Subsequent runs reuse existing `pr-integration-test-*` deployment (~6 minutes, 73% time reduction)

##### `test_deployment_workflow_full_lifecycle` (Release Test - Complete Lifecycle)

**Duration:** 20-30 minutes

**What it tests:**
1. Create fresh deployment using `TestDeploymentConfig::for_release_tests()` (no prefix reuse)
2. Validates deployment is ready (RevisionStatus::Deployed)
3. Patching deployment (triggers new revision)
4. Polling new revision to DEPLOYED status
5. Validating deployment source, URL, and final status
6. Cleanup with RAII guard (deletes deployment after test)

**Validations:**
- ✅ Deployment source is "github"
- ✅ Deployment has custom_url in source_config
- ✅ Final revision status is Deployed
- ✅ Automatic cleanup on test failure (via DeploymentGuard)

**Use Cases:**
- Pre-release validation requiring complete create/delete cycle
- Testing deployment cleanup functionality
- Scenarios requiring a fresh deployment each run

Creates a `release-integration-test-{ts}` deployment and performs a complete lifecycle test including cleanup. This test creates a new deployment each run with RAII guard protection.

##### `test_list_deployments` (Read-Only)

**Duration:** 1-3 seconds

Lists deployments with limit parameter to verify basic API functionality.

##### `test_list_github_integrations` (Read-Only)

**Duration:** 1-3 seconds

Lists all configured GitHub integrations for the workspace. Shows integration ID, name, provider, and organization.

##### `test_list_github_repositories` (Read-Only)

**Duration:** 2-5 seconds

Lists all repositories accessible through a GitHub integration. Validates integration permissions and repository access.

##### `test_find_integration_for_repo` (Key Operation)

**Duration:** 2-5 seconds

Finds the correct GitHub integration for a given repository owner and name. This is the core operation used by the deployment workflow to dynamically discover integration IDs.

##### `test_deployment_url_extraction` (Unit Test)

**Duration:** <1 second

Pure unit test that validates the `custom_url()` helper method for extracting deployment URLs from source_config JSON. No API calls.

#### DeploymentGuard (RAII Cleanup)

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

## CI/CD Integration

Integration tests run automatically in CI for PRs and main branch pushes. See `.github/workflows/ci.yml` for the current configuration.

**Required GitHub Secrets:**
- `LANGSMITH_API_KEY`
- `LANGSMITH_WORKSPACE_ID`

The CI uses `cargo nextest` with the `integration` profile and `integration-tests` feature flag.

## Test Organization

Integration tests are organized by functionality:

- **`integration_test.rs`** - PromptHub operations (list, push)
- **`assistant_integration_test.rs`** - Assistant CRUD operations
- **`integration_deployment_workflow.rs`** - Deployment lifecycle operations

Each test file is self-contained and can be run independently.

## Design Principles

### 1. Idempotency
Integration tests should be safe to run multiple times without side effects.

### 2. Cleanup
Tests should clean up any resources they create when possible.

### 3. Test Data
Use clearly named test resources (e.g., `pr-integration-test`, `release-integration-test`).

### 4. Timeouts
Integration tests may be slower due to network calls. Deployment tests can take 5-30 minutes.

### 5. Error Messages
Provide helpful error messages for common failure modes.

### 6. Resource Reuse
PR/dev tests reuse deployments for faster iteration. Release tests create fresh deployments for complete lifecycle validation.

## Troubleshooting

### Common Issues

#### Assistant Tests

**Error:** "TEST_GRAPH_ID environment variable not set"

**Solution:**
1. Deploy test graph (see `../../../tests/fixtures/test-graph-deployment/DEPLOYMENT_GUIDE.md`)
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

#### Deployment Tests

**"Failed to find GitHub integration for repository"**

Solution:
1. Verify GitHub integration exists in LangSmith UI
2. Check integration has repository access configured
3. Verify `REPOSITORY_OWNER` and `REPOSITORY_NAME` environment variables

---

**"Timeout waiting for revision to be DEPLOYED after 30 minutes"**

Solution:
1. Check deployment status in LangSmith UI
2. Review deployment logs for build errors
3. Verify `langgraph.json` is valid at the specified path
4. Confirm GitHub integration has proper permissions

---

**"Failed to list GitHub repositories: Forbidden"**

Solution:
1. Verify API key has integration read permissions
2. Check workspace ID is correct
3. Ensure integration is in the same workspace

---

### Debugging Integration Tests

Run with full output to see detailed information:

```bash
RUST_LOG=debug cargo test --test integration_test -- --ignored --nocapture
```

Add backtrace for error details:

```bash
RUST_BACKTRACE=1 cargo test --test integration_test -- --ignored --nocapture
```

## Contributing

When adding new integration tests:

1. **Mark tests with `#[ignore]`** - Integration tests should not run by default
2. **Clean up resources** - Always delete test resources when possible
3. **Use unique names** - Use timestamps + UUID suffix for uniqueness
4. **Document prerequisites** - List required env vars clearly
5. **Handle missing credentials** - Skip tests gracefully if env vars not set
6. **Add to CI** - Ensure new tests run in GitHub Actions
7. **Use standardized naming** - Follow `TestDeploymentConfig` patterns

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

## Related Documentation

- [Test Fixtures](./test-fixtures.md) - Test deployment configuration and fixtures
- [CI/CD Documentation](../ci-cd.md) - GitHub Actions integration
- [HIGH_LEVEL_TESTING_GUIDELINES](./HIGH_LEVEL_TESTING_GUIDELINES.md) - Testing principles
- [sdk/tests/README.md](../../../sdk/tests/README.md) - Quick reference (redirects here)
