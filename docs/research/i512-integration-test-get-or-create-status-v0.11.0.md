# Integration Test Fixture Analysis: SDK vs CLI

**Issue:** #512, #524
**Date:** 2025-12-03
**Purpose:** Document the current state of test fixtures, the discrepancy between SDK and CLI implementations, and the path forward for consolidation.

---

## Executive Summary

The project has **two completely separate test fixture implementations** with no code sharing:

| Component | Location | Approach | Status |
|-----------|----------|----------|--------|
| **SDK Tests** | `sdk/tests/integration_deployment_workflow.rs` | Uses SDK directly | ✅ Correct |
| **CLI Tests** | `cli/tests/common/fixtures.rs` | Shells out to CLI | ❌ Broken |

**The core problem:** CLI test fixtures reimplemented deployment management by shelling out to CLI commands instead of using the SDK. This led to bugs, duplication, and inconsistent behavior.

**The solution:** Consolidate test fixtures to use SDK directly. Track in **Issue #524**.

---

## Current Architecture: Two Separate Implementations

### SDK Tests (`sdk/tests/integration_deployment_workflow.rs`)

**Approach:** Uses SDK client directly

```rust
// Integration ID discovery - uses SDK integrations API
let integration_id = client
    .integrations()
    .find_integration_for_repo(&owner, &repo)
    .await?;

// Get-or-create - filters by NAME (finds in-progress deployments)
let filters = DeploymentFilters {
    name_contains: Some(deployment_name.clone()),
    ..Default::default()
};
let deployments = client.deployments().list(Some(100), None, Some(filters)).await?;

// Wait for deployment - polls revision status
wait_for_deployment(&client, &deployment_id, &revision_id).await?;
```

**Features:**
- Uses `client.integrations().find_integration_for_repo()` for GitHub integration ID
- Filters by **name** (not status) - can find in-progress deployments
- Has `wait_for_deployment()` helper that polls until READY
- Two test variants: PR-style (get-or-create) and Release-style (full lifecycle)
- Uses `DeploymentGuard` RAII pattern for cleanup on failure

### CLI Tests (`cli/tests/common/fixtures.rs`)

**Approach:** Shells out to CLI commands

```rust
// Integration ID discovery - parses CLI JSON output
fn query_github_integration_id() -> Option<String> {
    let mut cmd = Command::new(&bin);
    cmd.args(["graph", "list", "--limit", "100", "--format", "json"]);
    // Parse JSON, find deployment with source=="github", extract integration_id
}

// Get-or-create - filters by STATUS (misses in-progress deployments!)
cmd.args([
    "graph", "list",
    "--name-contains", "test-deployment-",
    "--status", "READY",  // BUG: Only finds READY deployments
    "--format", "json",
]);
```

**Problems:**
1. **Shells out to CLI** instead of using SDK - duplicates logic, prone to parsing errors
2. **Filters by `--status READY`** - misses Building/Deploying/Queued deployments
3. **Falls back to `LANGGRAPH_GITHUB_INTEGRATION_ID` env var** - workaround that should be removed
4. **No `wait_for_deployment`** - can't wait for in-progress deployments
5. **No code sharing** with SDK tests

---

## The GitHub Integration ID Problem

### What is a GitHub Integration ID?

To deploy from GitHub, LangGraph Cloud needs a GitHub App integration. Each integration has a UUID that must be passed when creating deployments with `source: "github"`.

### Historical Workarounds

1. **`LANGGRAPH_GITHUB_INTEGRATION_ID` env var** - Required devs to find ID in Chrome DevTools Network tab (extremely inconvenient)
2. **CLI auto-discovery from existing deployments** - Parse `source_config.integration_id` from existing GitHub deployments

### The Correct Solution (SDK)

PR #185 added proper API support:

```rust
// List all GitHub integrations
let integrations = client.integrations().list_github_integrations().await?;

// Find integration for a specific repo
let integration_id = client
    .integrations()
    .find_integration_for_repo("codekiln", "langstar")
    .await?;
```

**API Endpoints:**
- `GET /v1/integrations/github/install` - List all integrations
- `GET /v1/integrations/github/{id}/repos` - List repos for an integration

The CLI fixtures should use this SDK method, not parse CLI output or rely on env vars.

---

## The "In Progress" Bug

### Current CLI Behavior

```rust
// cli/tests/common/fixtures.rs
cmd.args(["--status", "READY", ...]);
```

This only finds READY deployments. If a deployment is Building/Deploying/Queued:
1. Fixture doesn't find it
2. Creates a NEW deployment
3. Results in duplicate deployments and race conditions

### Correct SDK Behavior

```rust
// sdk/tests/integration_deployment_workflow.rs
let filters = DeploymentFilters {
    name_contains: Some(deployment_name.clone()),
    ..Default::default()  // No status filter!
};
```

Filters by **name only**, then checks status after finding:
- If READY → use it
- If Building/Deploying/Queued → wait for it
- If not found → create new

---

## Two Test Modes: PR vs Release

### PR-Style Tests (Get-or-Create)

**Purpose:** Fast iteration during development and PR CI

**Behavior:**
1. Look for existing deployment by name
2. If found (any status) → wait for READY if needed, then use it
3. If not found → create new
4. Leave deployment running for future test runs
5. Cleanup happens via periodic cron job

**SDK Example:** `test_deployment_workflow()` (lines 87-299)
- Uses persistent name: `langstar-integration-test`
- Leaves deployment running

### Release-Style Tests (Full Lifecycle)

**Purpose:** Pre-release validation of complete workflow

**Behavior:**
1. Always create fresh deployment with timestamp-based unique name
2. Run tests
3. Delete deployment (cleanup in same job)
4. Validates the full create→test→teardown cycle

**SDK Example:** `test_deployment_workflow_full_lifecycle()` (lines 337-535)
- Uses unique name: `langstar-test-{timestamp}`
- Deletes after test
- Uses `DeploymentGuard` for cleanup on failure

---

## Proposed Consolidated Architecture

**Issue #524** tracks this refactoring.

### Target State

```
sdk/src/test_utils.rs (or sdk/tests/common/)
├── TestDeployment struct
├── get_or_create_deployment()  - finds existing or creates new
├── wait_for_deployment()       - polls until READY
├── find_github_integration()   - uses integrations API
└── DeploymentGuard             - RAII cleanup on failure

cli/tests/common/fixtures.rs
└── Re-exports SDK test utilities
    └── Only adds CLI-specific helpers if needed

sdk/tests/*.rs → imports shared utilities
cli/tests/*.rs → imports shared utilities (via fixtures.rs)
```

### Key Changes Needed

1. **Extract shared utilities from SDK tests** into reusable module
2. **Refactor CLI fixtures** to import SDK utilities instead of shelling out
3. **Remove `LANGGRAPH_GITHUB_INTEGRATION_ID`** - use `find_integration_for_repo()` instead
4. **Remove `--status READY` filter** - filter by name, check status after

---

## Summary of v0.10.0 → v0.11.0 Changes

### PR #185: SDK Integration API
- Added `IntegrationClient` with `list_github_integrations()`, `find_integration_for_repo()`
- The correct way to discover GitHub integration IDs

### PR #503: CI Test Auto-Discovery
- Changed from hard-coded test files to `cargo test -p langstar --features integration-tests`
- Added `LANGGRAPH_GITHUB_INTEGRATION_ID` to CI secrets (workaround, should be removed)

### PR #519: Test Parallelization
- Added `serial_test` crate for selective serialization
- Marked shared-deployment tests with `#[serial]`
- No changes to fixtures.rs

### PR #522: Security Fix + Fixture Improvements
- Added `sanitize_secrets()` for graph command output
- Added `query_github_integration_id()` to CLI fixtures (still uses CLI, not SDK)

---

## References

### Issues
- **Issue #524:** https://github.com/codekiln/langstar/issues/524 (Consolidate test fixtures)
- Issue #512: https://github.com/codekiln/langstar/issues/512 (Security: secrets in plaintext)
- Issue #517: https://github.com/codekiln/langstar/issues/517 (Parallelize tests)
- Issue #499: https://github.com/codekiln/langstar/issues/499 (Eval tests not running in CI)

### Pull Requests
- PR #185: https://github.com/codekiln/langstar/pull/185 (SDK integration API)
- PR #503: https://github.com/codekiln/langstar/pull/503 (CI auto-discovery)
- PR #519: https://github.com/codekiln/langstar/pull/519 (Test parallelization)
- PR #522: https://github.com/codekiln/langstar/pull/522 (Security fix)

### Key Files
- `sdk/src/integrations.rs` - GitHub integration API client
- `sdk/tests/integration_deployment_workflow.rs` - Correct SDK-based test fixtures
- `cli/tests/common/fixtures.rs` - CLI fixtures (needs refactoring)

### Releases
- v0.10.0: https://github.com/codekiln/langstar/releases/tag/v0.10.0
- v0.11.0: https://github.com/codekiln/langstar/releases/tag/v0.11.0
