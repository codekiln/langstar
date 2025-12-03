# Integration Test "Get or Create" Fixture Analysis: v0.10.0 → v0.11.0

**Issue:** #512
**Date:** 2025-12-03
**Purpose:** Clarify how integration test fixtures handle deployment "get or create" logic and GitHub integration ID discovery

---

## Executive Summary

The test fixture in `cli/tests/common/fixtures.rs` implements a **"get or create"** pattern for test deployments. The key insight is:

1. **v0.10.0**: Relied on the CLI's internal auto-discovery for GitHub integration ID during `graph create` (worked if existing deployments existed)
2. **v0.11.0 (PR #519)**: Added test parallelization but introduced a subtle issue - the fixture still relied on CLI auto-discovery
3. **PR #522 fix**: Added explicit `query_github_integration_id()` to the fixture, making it self-sufficient

The fixture **does NOT** check if a test deployment exists before creating one. Instead, it:
1. First attempts to **find and reuse** an existing READY test deployment
2. Only creates a new deployment if none exist
3. When creating, it needs the GitHub integration ID

---

## How the "Get or Create" Pattern Works

### Step 1: Find Existing Deployment (lines 84-188 in v0.11.0)

```rust
fn find_active_test_deployment() -> Option<Self> {
    // Query: langstar graph list --name-contains "test-deployment-" --status READY --format json
    // Returns first matching deployment, or None
}
```

This checks for deployments where:
- Name starts with `test-deployment-`
- Status is `READY`

### Step 2: Create New Deployment (if none found)

When `find_active_test_deployment()` returns `None`, the fixture calls `create_new_deployment()` which:
1. Generates a unique name: `test-deployment-{timestamp}`
2. **Discovers GitHub integration ID** (see below)
3. Runs: `langstar graph create --name <name> --source github --integration-id <id> ...`
4. Waits for READY status

---

## GitHub Integration ID Discovery: The Critical Difference

### v0.10.0 Behavior (Pre-Parallelization)

The fixture passed the `--integration-id` flag **only if** `LANGGRAPH_GITHUB_INTEGRATION_ID` was set:

```rust
// v0.10.0 - simplified logic
cmd.args(["graph", "create", "--name", &name, "--source", "github", ...]);
// No --integration-id unless env var set
// Relied on CLI's internal auto-discovery
```

**How CLI auto-discovery worked:**
- CLI would query existing deployments internally
- Extract `integration_id` from `source_config` of GitHub deployments
- Use the first found integration ID

**Why it worked in CI:**
- CI environments accumulated test deployments from previous runs
- These existing deployments provided the integration ID via auto-discovery
- Fresh environments would fail if no deployments existed

### v0.11.0 PR #519 Changes

PR #519 introduced test parallelization using `serial_test` crate:
- Tests using shared `TEST_DEPLOYMENT` marked with `#[serial]`
- Other tests run in parallel
- CI workflow removed `--test-threads=1`

**No changes to fixtures.rs in PR #519** - the fixture logic remained the same.

### v0.11.0 PR #522 Fix (Current State)

PR #522 added explicit integration ID discovery to the fixture itself:

```rust
// v0.11.0 with PR #522 - 3-tier discovery
let integration_id = std::env::var("LANGGRAPH_GITHUB_INTEGRATION_ID")
    .ok()
    .filter(|s| !s.is_empty())
    .or_else(|| {
        println!("LANGGRAPH_GITHUB_INTEGRATION_ID not set, querying API...");
        Self::query_github_integration_id()  // NEW: Fixture does its own discovery
    })
    .expect("GitHub integration ID required...");

// Now always pass --integration-id explicitly
cmd.args(["--integration-id", &integration_id, ...]);
```

**New `query_github_integration_id()` function (lines 191-271):**
```rust
fn query_github_integration_id() -> Option<String> {
    // 1. Run: langstar graph list --limit 100 --format json
    // 2. Find first deployment with source == "github"
    // 3. Extract source_config.integration_id
    // 4. Return Some(id) or None
}
```

---

## Key Behavioral Differences

| Aspect | v0.10.0 | v0.11.0 (main) | v0.11.0 (PR #522) |
|--------|---------|----------------|-------------------|
| Integration ID from env var | Yes | Yes | Yes |
| Integration ID from API query | No (relied on CLI) | No (relied on CLI) | Yes (fixture does it) |
| Explicit --integration-id flag | Only if env var set | Only if env var set | Always |
| Works in fresh environment | No (needs existing deployments) | No (needs existing deployments) | Yes (with either env var OR existing deployments) |

---

## Correction to PR #522 Progress Report

The progress report (`wip-i512-security-graph-list-format-secrets.md`) contains this statement:

> **Root Cause Analysis:**
> 1. Recent commit #519 introduced test parallelization
> 2. Tests switched from creating individual deployments to sharing via `OnceLock`

**Clarification:** PR #519 did NOT change how deployments are created or shared. The `OnceLock` pattern for shared deployments existed before PR #519. PR #519 only:
1. Added `#[serial]` attributes to tests using the shared deployment
2. Changed test name generation to use microseconds + UUID
3. Removed `--test-threads=1` from CI

The actual root cause for integration test failures in fresh environments:
- The fixture **always** relied on CLI's internal auto-discovery for integration ID
- This worked when existing deployments existed (providing the ID)
- This failed in fresh environments with no existing deployments
- PR #522's fix adds explicit API query in the fixture itself

---

## Expected Fixture Behavior (Post PR #522)

### Scenario 1: CI Environment (Has Existing Deployments)

1. `TestDeployment::create()` called
2. `find_active_test_deployment()` finds existing `test-deployment-*` in READY status
3. Returns existing deployment (no creation needed)
4. Integration ID discovery never triggered

### Scenario 2: CI Environment (No READY Test Deployments)

1. `TestDeployment::create()` called
2. `find_active_test_deployment()` returns `None`
3. `create_new_deployment()` called
4. Check `LANGGRAPH_GITHUB_INTEGRATION_ID` env var - not set
5. Call `query_github_integration_id()` - finds ID from other GitHub deployments
6. Create new deployment with explicit `--integration-id`

### Scenario 3: Fresh Environment (No Deployments At All)

1. `TestDeployment::create()` called
2. `find_active_test_deployment()` returns `None`
3. `create_new_deployment()` called
4. Check `LANGGRAPH_GITHUB_INTEGRATION_ID` env var - **must be set**
5. If not set and no existing deployments - **panic with helpful message**

---

## Summary of Changes in v0.11.0 Affecting Test Infrastructure

### PR #503: CI Test Auto-Discovery (Issue #499)
**This is the key change for ensuring local/CI test parity.**

Changed CI from hard-coded test file list to auto-discovery:

**Before:**
```yaml
cargo test --features integration-tests --test assistant_command_test --test graph_command_test
```

**After:**
```yaml
cargo test -p langstar --features integration-tests
```

Also added `LANGGRAPH_GITHUB_INTEGRATION_ID` to CI secrets (line 92 of `ci.yml`).

**Why this matters for fixtures:**
- With auto-discovery, ALL tests in the package run in CI
- Tests must be self-sufficient (can't rely on "not being run")
- The CI secret for integration ID provides a reliable fallback

### PR #518: DateTime Deserialization Fix
- Fixed `DateTime<Utc>` parsing for API responses without timezone suffix
- No impact on fixture logic

### PR #519: Test Parallelization
- Added `serial_test` crate dependency
- Marked 7 tests with `#[serial]` attribute
- Changed `generate_test_name()` to use microseconds + UUID
- Removed `--test-threads=1` from CI
- **No changes to fixtures.rs**

### PR #522 (Draft): Security + Fixture Fix
- Security: Added `sanitize_secrets()` for graph command output
- Fixture: Added `query_github_integration_id()` for self-sufficient integration ID discovery
- Fixture: Now always passes `--integration-id` explicitly to `graph create`

---

## The Proper Way to List GitHub Integration IDs

**Historical context:** `LANGGRAPH_GITHUB_INTEGRATION_ID` was a workaround before the integration listing API was discovered. Finding this ID in the UI requires opening Chrome DevTools Network tab in LangSmith and inspecting API calls - extremely inconvenient. **This env var should be removed - do not rely on it.**

### SDK Integration API (Added in PR #185)

**File:** `sdk/src/integrations.rs`

**Endpoint:** `GET /v1/integrations/github/install`

```rust
// SDK method - USE THIS IN TEST FIXTURES
let integrations = client.integrations().list_github_integrations().await?;
// Returns Vec<GitHubIntegration> with id and name fields
```

**Response structure:**
```json
[
  { "id": "uuid-of-integration", "name": "optional-name" },
  ...
]
```

**Additional endpoints:**
- `GET /v1/integrations/github/{integration_id}/repos` - List repos for a specific integration

### Current Fixture Implementation Issues

**PR #522 fixture (`query_github_integration_id`):**
- Runs CLI commands via `Command::new()` instead of using SDK directly
- Queries `graph list` and extracts `integration_id` from existing deployments
- Falls back to `LANGGRAPH_GITHUB_INTEGRATION_ID` env var (should be removed)

**Problems with this approach:**
1. Indirect - requires existing deployments instead of using the integrations API
2. Relies on env var that shouldn't exist
3. No distinction between PR vs release check contexts
4. Doesn't handle "deployment in progress" state correctly

### Critical Bug: "In Progress" Deployments Not Detected

**Filed as Issue #524:** https://github.com/codekiln/langstar/issues/524

**Current behavior in `find_active_test_deployment()` (lines 91-94):**
```rust
cmd.args([
    "--status",
    "READY",  // PROBLEM: Only finds READY deployments!
    ...
]);
```

**The bug:** If a deployment is Building, Deploying, or Queued, the fixture won't find it because it filters `--status READY`. It then creates a NEW deployment, resulting in:
- Wasted resources (multiple deployments created)
- Race conditions between parallel test runs
- Unnecessary API calls and longer test times

**Expected behavior:**
1. Query for ANY test deployment (not just READY)
2. If found with status Building/Deploying/Queued → **wait for it**
3. If found with status READY → use it
4. Only create new if no test deployments exist at all

---

## Intended Test Fixture Behavior

### Principle 1: Tests that don't need deployments shouldn't wait for them

Not all integration tests require a deployment. Don't make tests wait unnecessarily.

### Principle 2: Context-aware behavior (PR vs Release)

**In PR context:**
- a) Look for a READY deployment → use it if exists
- b) If a deployment exists but is NOT READY (being created) → **wait for it** (don't create another)
- c) Only create new if no deployments exist at all

**In release checks:**
- Full creation and teardown lifecycle
- NO get-or-create - always create fresh deployment
- Ensures we have complete picture, not relying on stale fixtures

### Principle 3: Context-aware cleanup (Already Implemented)

The cleanup behavior differs by context and is **already correctly implemented** in CI:

**PR test fixtures:**
- Leave deployment running after tests complete
- Cleaned up periodically by a cron job in CI
- Allows fast test runs by reusing existing deployments

**Release integration tests:**
- Full lifecycle: create → test → teardown in single job
- Ensures complete picture, not relying on stale fixtures
- Validates the full deployment workflow

### Principle 4: SDK for fixtures, CLI for integration tests

- **Test fixture creation/management:** Use SDK directly (e.g., `client.integrations().list_github_integrations()`)
- **Integration tests of CLI commands:** Shelling out to CLI is appropriate (that's what we're testing)

---

## Canonical Implementation: `sdk/tests/integration_deployment_workflow.rs`

This file demonstrates the **correct** implementation of the intended behavior.

### Two Test Variants (PR vs Release Context)

**1. `test_deployment_workflow` (PR-style, lines 87-299):**
- Get-or-create pattern
- Uses persistent deployment name: `langstar-integration-test`
- Leaves deployment running for reuse
- Faster iteration during development

**2. `test_deployment_workflow_full_lifecycle` (Release-style, lines 337-535):**
- Always creates fresh deployment with timestamp-based name
- Full create → test → teardown cycle
- Validates complete workflow
- Uses `DeploymentGuard` for cleanup on failure

### Proper Get-or-Create Pattern (lines 145-178)

```rust
// Try to find existing deployment first
let filters = DeploymentFilters {
    name_contains: Some(deployment_name.clone()),
    ..Default::default()
};
let deployments = client
    .deployments()
    .list(Some(100), None, Some(filters))
    .await?;

let deployment = if let Some(existing) = deployments
    .resources
    .iter()
    .find(|d| d.name == deployment_name)
{
    // Reuse existing deployment
    existing.clone()
} else {
    // Create new deployment
    client.deployments().create(&create_request).await?
};
```

**Key differences from `cli/tests/common/fixtures.rs`:**
- Uses SDK directly (not CLI commands)
- Uses `client.integrations().find_integration_for_repo()` for integration ID
- Filters by name, not status - can find in-progress deployments
- Has proper wait_for_deployment polling

### Integration ID Discovery (lines 114-119)

```rust
let integration_id = client
    .integrations()
    .find_integration_for_repo(&repository_owner, &repository_name)
    .await?;
```

### wait_for_deployment Helper (lines 552-603)

Polls revision status every 60 seconds:
- DEPLOYED → success
- BuildFailed/DeployFailed/Cancelled → error
- Other (Queued, Building, Deploying) → wait and poll again
- 30 minute timeout

---

## What Needs to Change in PR #522

1. **Remove `LANGGRAPH_GITHUB_INTEGRATION_ID` fallback** - Use SDK's `client.integrations().list_github_integrations()` instead
2. **Use SDK for fixture management** - Fixture creation should use SDK directly, not shell out to CLI
3. **Handle "in progress" deployments (Issue #524)** - If a deployment exists but isn't READY, wait for it instead of creating another

**Already correctly implemented:**
- Context awareness (PR vs release) - handled in CI workflow configuration
- Cleanup behavior - PR fixtures cleaned by cron, release fixtures do full lifecycle

---

## References

- PR #185: https://github.com/codekiln/langstar/pull/185 (SDK integration API - `list_github_integrations()`)
- PR #503: https://github.com/codekiln/langstar/pull/503 (CI auto-discovery)
- Issue #499: https://github.com/codekiln/langstar/issues/499 (eval tests not running in CI)
- PR #519: https://github.com/codekiln/langstar/pull/519 (test parallelization)
- PR #522: https://github.com/codekiln/langstar/pull/522 (security fix + fixture improvement)
- Issue #512: https://github.com/codekiln/langstar/issues/512 (security: secrets in plaintext)
- Issue #517: https://github.com/codekiln/langstar/issues/517 (parallelize tests)
- **Issue #524: https://github.com/codekiln/langstar/issues/524 (fixture should wait for in-progress deployments)**
- v0.10.0 Release: https://github.com/codekiln/langstar/releases/tag/v0.10.0
- v0.11.0 Release: https://github.com/codekiln/langstar/releases/tag/v0.11.0
