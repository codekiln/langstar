# Test Fixtures and Test Deployments

> **📍 Centralized Testing Documentation**
>
> This document is part of the centralized testing documentation suite. See `docs/dev/testing/README.md` for the complete TOC.

## Overview

This document describes the test deployment fixtures used across SDK and CLI integration tests, including naming conventions, lifecycle management, and cleanup strategies.

## Test Deployment Naming Standards

Integration tests use exactly **two deployment types** with standardized naming:

| Type | Pattern | Behavior | Usage |
|------|---------|----------|-------|
| **PR/Dev** | `pr-integration-test-{timestamp}` | Get-or-create by prefix, cleaned by cron (4hr threshold) | SDK tests, shared deployments |
| **Release** | `release-integration-test-{timestamp}` | Always fresh, self-deleting | CLI graph tests, full lifecycle tests |

### Using TestDeploymentConfig

**SDK Integration Tests** use `TestDeploymentConfig` from the SDK:

```rust
// PR/Dev: Reuse existing deployment for faster iteration
let config = TestDeploymentConfig::default();

// Release: Create fresh deployment each time
let config = TestDeploymentConfig::for_release_tests();
```

**Behavior:**
- `TestDeploymentConfig::default()` - Creates PR/dev config with prefix-based reuse
  - Searches for existing `pr-integration-test-*` deployment
  - Reuses if found (saves ~16 minutes per test run)
  - Creates new one if none exists

- `TestDeploymentConfig::for_release_tests()` - Creates release config (always fresh)
  - Always creates new `release-integration-test-{timestamp}` deployment
  - No reuse, complete lifecycle validation
  - Self-deleting via RAII guard

### When to Use Each Type

**Use PR/Dev deployments for:**
- Development and testing during PR review
- Tests that don't modify deployment state
- Faster iteration (reuses existing deployments)
- SDK integration tests (default behavior)

**Use Release deployments for:**
- Pre-release validation requiring complete create/delete cycle
- Testing deployment cleanup functionality
- CLI graph command tests (full lifecycle)
- Scenarios requiring a fresh deployment each run

## Test Graph Deployment

### Overview

The `tests/fixtures/test-graph-deployment/` directory contains a minimal LangGraph application used for integration testing.

### Structure

```
test-graph-deployment/
├── langgraph.json          # LangGraph configuration
├── requirements.txt        # Python dependencies
├── .env.example           # Environment variable template
├── README.md              # Quick start guide
├── DEPLOYMENT_GUIDE.md    # Detailed deployment instructions
└── test_agent/            # Python module
    ├── __init__.py
    └── agent.py           # Minimal echo graph implementation
```

### Graph Implementation

The test graph is intentionally minimal:

- **Single Node**: An `echo` node that prefixes messages with "Echo: "
- **Simple State**: Contains only a `message` field
- **No Dependencies**: Does not require API keys or external services
- **Fast Execution**: Completes immediately for quick test cycles

**Graph Flow:**
```
START → echo_node → END
```

**Example:**

Input:
```json
{
  "message": "Hello, World!"
}
```

Output:
```json
{
  "message": "Echo: Hello, World!"
}
```

### Deployment Setup

**Prerequisites:**
1. LangSmith account
2. GitHub integration configured in LangSmith
3. Repository access for the integration

**Deploy to LangGraph Cloud:**

See `tests/fixtures/test-graph-deployment/DEPLOYMENT_GUIDE.md` for detailed step-by-step instructions.

**Quick summary:**
1. Navigate to [LangSmith](https://smith.langchain.com/)
2. Go to "Deployments" section
3. Click "New Deployment"
4. Connect this repository or upload files
5. Configure deployment settings
6. Deploy
7. Note the **Graph ID** from the deployment details

### Required Environment Variables

After deploying the test graph, set these environment variables:

```bash
# Required for all tests
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"

# Required for assistant tests
export TEST_GRAPH_ID="<graph-id-from-deployment>"

# Optional: For read-only tests
export TEST_DEPLOYMENT_ID="<deployment-id>"
```

**Finding these values:**

- **LANGSMITH_API_KEY**: [LangSmith Settings](https://smith.langchain.com/settings) → "API Keys"
- **LANGSMITH_WORKSPACE_ID**: LangSmith UI → Settings → Workspace ID
- **TEST_GRAPH_ID**: After deployment, go to deployment details → Copy Graph ID
- **TEST_DEPLOYMENT_ID**: Deployment URL or details page

## Deployment Lifecycle Management

### RAII Cleanup Guards

Both SDK and CLI tests use RAII (Resource Acquisition Is Initialization) patterns for automatic cleanup:

**SDK DeploymentGuard:**

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

**CLI TestDeployment:**

```rust
use common::fixtures::TestDeployment;

// Creates deployment and polls until READY
let deployment = TestDeployment::create();

// Use deployment.name and deployment.id in tests
// Deployment is automatically cleaned up when dropped
```

**Features:**
- Implements Drop trait for automatic cleanup
- Provides `disarm()` method to skip cleanup after manual deletion
- Prints deployment ID and cleanup instructions on drop
- Prevents orphaned deployments when tests fail

### Cleanup Strategies

**Strategy 1: RAII Guards (Preferred)**
- Automatic cleanup on test completion or failure
- Used by both SDK and CLI tests
- Most reliable for preventing orphaned resources

**Strategy 2: Periodic Cron Job**
- GitHub Actions workflow runs periodically
- Cleans up deployments older than 4 hours
- Captures deployments from both `pr-integration-test-*` and `release-integration-test-*` patterns
- Backup cleanup mechanism for orphaned deployments

**Strategy 3: Manual Cleanup**
- Use LangSmith UI to bulk delete old test deployments
- Use CLI commands (once implemented):
  ```bash
  # List test deployments
  langstar graph list | grep "integration-test"

  # Delete specific deployment
  langstar graph delete <deployment-id>
  ```

## Shared vs Fresh Deployments

### Shared Deployments (SDK)

**Pattern:** `pr-integration-test-{timestamp}`

**Advantages:**
- Faster test execution (~6 min vs ~22 min)
- Reduced API calls and resource usage
- Better for development iteration

**Disadvantages:**
- Tests must not modify deployment state
- Requires coordination between tests
- May hide deployment lifecycle bugs

**Use for:**
- SDK PromptHub tests
- SDK deployment workflow tests (default path)
- Read-only operations

### Fresh Deployments (CLI, SDK Release Tests)

**Pattern:** `release-integration-test-{timestamp}`

**Advantages:**
- Complete lifecycle validation
- No test interdependencies
- Tests deployment cleanup functionality
- Catches deployment-specific bugs

**Disadvantages:**
- Slower (creates new deployment each run)
- More API calls and resource usage
- Longer test execution time

**Use for:**
- CLI graph command tests
- SDK deployment workflow full lifecycle tests
- Pre-release validation
- CRUD operations that modify state

## Test Isolation

### Unique Naming for Parallel Safety

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

### Selective Serialization

**Most tests run in parallel** by default for better performance.

**Tests with shared resources** are marked with `#[serial]`:
- CLI assistant tests that share a deployment via `OnceLock`
- Tests that modify global state
- Tests with resource conflicts

Use `#[serial]` only when truly necessary - prefer parallel execution for faster CI.

## Maintenance

### Updating the Test Graph

To update the graph logic:

1. Modify `tests/fixtures/test-graph-deployment/test_agent/agent.py`
2. Commit changes
3. Redeploy via LangSmith UI or CI/CD

### Cleanup Test Data

```bash
# List all test assistants (via Langstar CLI)
langstar assistant list | grep "test-assistant"

# Delete test assistants
langstar assistant delete <assistant-id>

# Or use the LangSmith UI to bulk delete
```

## Troubleshooting

### "Failed to create test deployment"

**Causes:**
- Invalid API key or workspace ID
- No access to workspace
- Rate limiting
- Network issues
- GitHub integration not configured

**Solutions:**
- Verify API key and workspace ID are correct
- Check workspace permissions
- Wait a few minutes if rate limited
- Verify GitHub integration exists and has repository access

### "Deployment not found" or "404"

**Solutions:**
- Verify deployment is active in LangSmith UI
- Check deployment hasn't been deleted
- Ensure using correct workspace

### "Timeout waiting for deployment"

**Causes:**
- Deployment build taking longer than expected (>30 minutes)
- Build errors in langgraph.json
- GitHub integration permissions issues

**Solutions:**
- Check deployment logs in LangSmith UI
- Verify `langgraph.json` is valid
- Confirm GitHub integration has proper permissions
- Check for build errors in deployment details

## Related Documentation

- [SDK Integration Tests](./sdk-integration-tests.md) - SDK testing patterns and deployment usage
- [CLI Integration Tests](./cli-integration-tests.md) - CLI testing patterns and test fixtures
- [Test Graph Deployment Guide](../../../tests/fixtures/test-graph-deployment/DEPLOYMENT_GUIDE.md) - Step-by-step deployment instructions
- [Test Graph README](../../../tests/fixtures/test-graph-deployment/README.md) - Quick reference (redirects here for standards)
