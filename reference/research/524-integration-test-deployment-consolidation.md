# Research: Integration Test Deployment Naming Consolidation

**Issue:** #524
**Date:** 2025-12-04
**Purpose:** Document all testing documentation, CI workflows, and current deployment naming patterns to inform consolidation to exactly two deployment types.

---

## Executive Summary

The project currently has **8 different test deployment naming patterns** spread across SDK and CLI tests. This creates:

- Confusion about which pattern to use
- Incomplete cleanup (patterns missed by cron job)
- Orphaned resources blocking CI

**Goal:** Consolidate to exactly **two types**:

1. `pr-integration-test-{timestamp}` - Shared deployment for PR/dev testing (get-or-create by prefix)
2. `release-integration-test-{timestamp}` - Full lifecycle test (always creates fresh, self-deleting)

---

## A. Testing Documentation Inventory

### Primary Test Documentation

| File                                                       | Lines | Topic                                                                      |
| ---------------------------------------------------------- | ----- | -------------------------------------------------------------------------- |
| `sdk/tests/README.md`                                      | 1-446 | SDK integration test guide, deployment naming conventions, troubleshooting |
| `cli/tests/README.md`                                      | 1-237 | CLI integration test guide, test parallelization, fixtures                 |
| `tests/fixtures/test-graph-deployment/README.md`           | 1-272 | Test graph deployment setup, environment variables                         |
| `tests/fixtures/test-graph-deployment/DEPLOYMENT_GUIDE.md` | 1-423 | LangGraph Cloud deployment guide (manual + GitHub integration)             |

### Analysis & Research Documents

| File                                                                  | Lines | Topic                                                              |
| --------------------------------------------------------------------- | ----- | ------------------------------------------------------------------ |
| `docs/research/i512-integration-test-get-or-create-status-v0.11.0.md` | 1-258 | SDK vs CLI fixture analysis, identifies duplication problem        |
| `docs/langgraph-deployments-and-revisions.md`                         | Full  | Deployment vs Revision status (DeploymentStatus vs RevisionStatus) |

### CI/CD Documentation

| File                                  | Lines | Topic                                        |
| ------------------------------------- | ----- | -------------------------------------------- |
| `.github/workflows/README.md`         | Full  | CI workflow overview and documentation       |
| `.github/workflows/TEST-ISOLATION.md` | 1-202 | DevContainer feature test isolation strategy |

---

## B. CI Workflow Files

### Integration Test Workflows

| File                                             | Trigger             | Purpose                                                               |
| ------------------------------------------------ | ------------------- | --------------------------------------------------------------------- |
| `.github/workflows/ci.yml`                       | push/PR to main     | Main CI - unit tests (line 54-104) + integration tests (line 106-163) |
| `.github/workflows/cleanup-test-deployments.yml` | cron (4hr) + manual | Cleans up test deployments older than 4 hours                         |

### Key CI Configuration (ci.yml)

**Unit Tests (lines 72-73):**

```yaml
run: cargo nextest run --profile ci --all-features --workspace --lib
```

**Integration Tests (lines 127-137):**

```yaml
run: cargo nextest run --profile integration -p langstar --features integration-tests
env:
  LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
  LANGSMITH_WORKSPACE_ID: ${{ secrets.LANGSMITH_WORKSPACE_ID }}
  LANGGRAPH_GITHUB_INTEGRATION_ID: ${{ secrets.LANGGRAPH_GITHUB_INTEGRATION_ID }}
```

### Cleanup Workflow (cleanup-test-deployments.yml)

**Current pattern matching (lines 67-76):**

```bash
for pattern in "integration-test" "langstar-test" "cli-test"; do
  # Search and merge results
done
```

**Threshold:** 4 hours (line 97)

---

## C. Current Test Deployment Naming Patterns

### Pattern Inventory (8 patterns - PROBLEM)

| Pattern                         | Source File                                    | Line | Type                  | Cleanup     |
| ------------------------------- | ---------------------------------------------- | ---- | --------------------- | ----------- |
| `pr-integration-test`           | `sdk/src/test_utils.rs`                        | 59   | Shared (no timestamp) | Cron        |
| `release-integration-test-{ts}` | `sdk/src/test_utils.rs`                        | 84   | Lifecycle             | Self-delete |
| `langstar-integration-test`     | `sdk/tests/integration_deployment_workflow.rs` | 123  | Persistent            | Manual      |
| `langstar-test-{ts}`            | `sdk/tests/integration_deployment_workflow.rs` | 377  | Lifecycle             | Self-delete |
| `cli-test-deployment-{ts}`      | `cli/tests/graph_command_test.rs`              | 297  | Lifecycle             | Self-delete |
| `cli-test-deployment-wait-{ts}` | `cli/tests/graph_command_test.rs`              | 354  | Lifecycle             | Self-delete |
| `cli-test-lifecycle-{ts}`       | `cli/tests/graph_command_test.rs`              | 426  | Lifecycle             | Self-delete |
| `cli-test-deployment-env-{ts}`  | `cli/tests/graph_command_test.rs`              | 614  | Lifecycle             | Self-delete |

### Pattern Analysis

**Problems with current patterns:**

1. **No uniqueness in shared pattern**: `pr-integration-test` has no timestamp, causing 409 conflicts when orphaned tracing projects exist

2. **Inconsistent naming**:
   - SDK uses `langstar-*` prefix
   - CLI uses `cli-test-*` prefix
   - Shared uses `*-integration-test` pattern

3. **Cleanup gaps**:
   - Original cleanup only matched `integration-test`
   - Fixed in PR #539 to also match `langstar-test` and `cli-test`
   - But this is a symptom of too many patterns

4. **Documentation confusion**: `sdk/tests/README.md` lists 3 patterns (lines 16-22) but codebase has 8

---

## D. Code Locations for Deployment Creation

### SDK Test Utilities (`sdk/src/test_utils.rs`)

**TestDeploymentConfig struct (line 39-67):**

```rust
pub struct TestDeploymentConfig {
    pub name: String,
    pub repository_owner: String,
    pub repository_name: String,
    pub branch: String,
    pub config_path: String,
}

impl Default for TestDeploymentConfig {
    fn default() -> Self {
        Self {
            name: "pr-integration-test".to_string(),  // Line 59 - no timestamp!
            // ...
        }
    }
}
```

**for_release_tests() (line 70-87):**

```rust
pub fn for_release_tests() -> Self {
    let timestamp = SystemTime::now()...;
    Self {
        name: format!("release-integration-test-{}", timestamp),  // Line 84
        ..Default::default()
    }
}
```

**get_or_create_deployment() (line 310-415):**

- Searches by exact name match (line 306-316)
- Creates if not found (line 326-414)

### SDK Integration Tests (`sdk/tests/integration_deployment_workflow.rs`)

**test_deployment_workflow() (line 87-299):**

- Uses hardcoded `langstar-integration-test` (line 123)
- Does NOT delete after test (persistent)

**test_deployment_workflow_full_lifecycle() (line 337-535):**

- Uses `{repo_name}-test-{timestamp}` pattern (line 377)
- Deletes after test (line 520-525)

### CLI Tests (`cli/tests/graph_command_test.rs`)

**test_graph_create_basic() (line 285-340):**

- Uses `cli-test-deployment-{ts}` (line 297)

**test_graph_create_with_wait() (line 342-423):**

- Uses `cli-test-deployment-wait-{ts}` (line 354)

**test_graph_lifecycle() (line 425-524):**

- Uses `cli-test-lifecycle-{ts}` (line 426)

**test_graph_create_with_env_secrets() (line 600-700):**

- Uses `cli-test-deployment-env-{ts}` (line 614)

---

## E. Related Issues and PRs

### Issues

| Issue | Title                                             | Relevance                   |
| ----- | ------------------------------------------------- | --------------------------- |
| #524  | Consolidate CLI test fixtures to use SDK directly | Parent issue for this work  |
| #512  | Security: secrets in plaintext                    | Led to fixture improvements |
| #517  | Parallelize tests                                 | Added serial_test crate     |

### Pull Requests

| PR   | Title                       | Impact                                 |
| ---- | --------------------------- | -------------------------------------- |
| #185 | SDK integration API         | Added find_integration_for_repo()      |
| #519 | Test parallelization        | Added #[serial] markers                |
| #522 | Security fix                | Added query_github_integration_id()    |
| #526 | Test fixtures consolidation | Moved to SDK test_utils                |
| #539 | Fix cleanup patterns        | Added langstar-test, cli-test patterns |

---

## F. Recommendations

### Target State: Two Deployment Types

| Type        | Pattern                         | Usage                              | Cleanup     |
| ----------- | ------------------------------- | ---------------------------------- | ----------- |
| **PR/Dev**  | `pr-integration-test-{ts}`      | Shared via get-or-create by prefix | Cron (4hr)  |
| **Release** | `release-integration-test-{ts}` | Fresh each run, full lifecycle     | Self-delete |

### Key Changes

1. **Add timestamp to PR pattern** for uniqueness
2. **Add prefix field** to TestDeploymentConfig for get-or-create lookup
3. **Update get_or_create_deployment()** to search by prefix
4. **Migrate all tests** to use TestDeploymentConfig
5. **Simplify cleanup workflow** to single pattern

### Documentation Updates

Files requiring updates:

- `sdk/tests/README.md` (lines 16-22)
- `cli/tests/README.md` (fixtures section)
- `docs/research/i512-integration-test-get-or-create-status-v0.11.0.md`

---

## References

- Implementation plan: `docs/implementation/524-test-deployment-naming-consolidation.md`
- SDK test utilities: `sdk/src/test_utils.rs`
- CI workflow: `.github/workflows/ci.yml`
- Cleanup workflow: `.github/workflows/cleanup-test-deployments.yml`
