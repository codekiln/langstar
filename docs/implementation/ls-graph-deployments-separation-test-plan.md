# Test Plan: ls-graph-deployments-separation Milestone

**Issue:** #571 (Comprehensive tests for deployment and graph commands)
**Milestone:** ls-graph-deployments-separation (#11)
**Feature Type:** Combined SDK + CLI
**Date:** 2025-12-06

## Test Strategy Overview

### Milestone Summary

This milestone introduces semantic separation between `langstar deployment` and `langstar graph` commands:
- **`langstar deployment`**: Operations on LangGraph Cloud deployments (list, get, create, delete)
- **`langstar graph`**: Operations on LangGraph graphs within deployments (list, get)

The separation requires testing at both SDK and CLI layers to verify:
1. SDK types deserialize correctly from API responses
2. SDK methods return expected data structures
3. CLI commands produce correct output formats
4. Error handling works across layers
5. CRUD lifecycle operations work end-to-end

### Testing Approach

| Layer | Test Type | Location | Purpose |
|-------|-----------|----------|---------|
| SDK | Unit tests (mocked) | `sdk/tests/graph_test.rs` | Verify deserialization, error handling |
| SDK | Integration tests | `sdk/tests/graph_integration_test.rs` | Verify real API behavior |
| CLI | Unit tests | `cli/tests/deployment_command_test.rs` | Verify help, output parsing |
| CLI | Integration tests | `cli/tests/graph_command_test.rs` | CRUD lifecycle verification |

## Existing WIP Review

### ✅ Already Implemented

#### SDK Unit Tests (`sdk/tests/graph_test.rs`)
- [x] `test_list_graphs_returns_unique_graph_ids` - Graph deduplication
- [x] `test_list_graphs_with_structure_fetches_nodes` - Node fetching
- [x] `test_list_graphs_empty_deployment` - Empty results handling
- [x] `test_list_graphs_handles_pagination` - Pagination logic
- [x] `test_get_graph_returns_structure` - Node/edge parsing
- [x] `test_get_graph_with_conditional_edges` - Conditional edge handling
- [x] `test_get_graph_without_xray` - xray parameter handling
- [x] `test_get_graph_not_found` - 404 error handling
- [x] `test_list_graphs_api_error` - 500 error handling
- [x] `test_list_graphs_structure_fetch_failure_graceful` - Graceful degradation
- [x] `test_get_subgraphs` - Subgraph fetching
- [x] `test_get_subgraphs_empty` - Empty subgraphs

#### CLI Tests (`cli/tests/deployment_command_test.rs`)
- [x] `test_deployment_list_basic` - Basic list command
- [x] `test_deployment_list_with_limit` - Limit parameter
- [x] `test_deployment_list_json_output` - JSON output format
- [x] `test_deployment_list_filter_by_type` - Type filtering
- [x] `test_deployment_list_filter_by_status` - Status filtering
- [x] `test_deployment_list_filter_by_name` - Name filtering
- [x] `test_deployment_list_invalid_type` - Invalid type rejection
- [x] `test_deployment_list_invalid_status` - Invalid status rejection
- [x] `test_deployment_get_basic` - Basic get command
- [x] `test_deployment_get_json_output` - JSON output for get
- [x] `test_deployment_get_invalid_id` - Invalid ID handling
- [x] `test_deployment_commands_help` - Help output verification
- [x] `test_deployment_workflow_list_then_get` - Workflow test
- [x] `test_deployment_secrets_redacted` - Secret redaction

### ❌ Missing Tests (To Be Implemented)

#### SDK Integration Tests (`sdk/tests/graph_integration_test.rs`)
- [ ] `test_graph_list_real_deployment` - List graphs from real deployment
- [ ] `test_graph_get_real_deployment` - Get specific graph from real deployment
- [ ] `test_graph_list_with_structure` - List with node names populated

#### CLI Graph Tests (`cli/tests/graph_command_test.rs`)
- [ ] `test_graph_list_basic` - Basic graph list command
- [ ] `test_graph_list_with_deployment` - List graphs for specific deployment
- [ ] `test_graph_list_json_output` - JSON output verification
- [ ] `test_graph_get_basic` - Get specific graph
- [ ] `test_graph_get_json_output` - JSON output for graph structure
- [ ] `test_graph_commands_help` - Help output verification
- [ ] `test_graph_list_missing_deployment` - Error when deployment not specified
- [ ] `test_graph_workflow_crud` - Full CRUD lifecycle

#### Deprecation Warning Tests
- [ ] `test_old_graph_list_shows_deprecation` - Old `graph list` warns about `deployment list`
- [ ] Verify deprecation messages in stderr

## Test Implementation Details

### 1. SDK Unit Tests (Mocked)

**Location:** `sdk/tests/graph_test.rs`

**Status:** ✅ Complete - Uses mockito for HTTP mocking

**Coverage:**
- Graph deserialization from JSON
- GraphNode/GraphEdge parsing
- Pagination handling
- Error conditions (404, 500)
- Graceful degradation on partial failures

### 2. SDK Integration Tests

**Location:** `sdk/tests/graph_integration_test.rs`

**Prerequisites:**
```bash
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
```

**Test Cases:**

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_graph_list_real_deployment() {
    // 1. Get or create test deployment
    // 2. Call client.graphs().list(deployment_url, false)
    // 3. Verify returns Vec<GraphSummary>
    // 4. Verify graph_id and assistant_count populated
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_graph_get_real_deployment() {
    // 1. Get or create test deployment
    // 2. List graphs to get a graph_id
    // 3. Call client.graphs().get(deployment_url, graph_id, true)
    // 4. Verify returns Graph with nodes and edges
}
```

### 3. CLI Integration Tests

**Location:** `cli/tests/graph_command_test.rs`

**Prerequisites:**
- Same as SDK integration tests
- Test deployment with known graph structure

**Test Cases:**

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_graph_list_basic() {
    // 1. Create/get test deployment
    // 2. Run: langstar graph list --deployment <name>
    // 3. Verify output contains graph table headers
    // 4. Verify exit code 0
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_graph_list_json_output() {
    // 1. Create/get test deployment
    // 2. Run: langstar graph list --deployment <name> --format json
    // 3. Parse JSON output
    // 4. Verify structure has expected fields (graph_id, assistant_count, node_names)
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_graph_get_json_output() {
    // 1. Create/get test deployment
    // 2. List graphs to get graph_id
    // 3. Run: langstar graph get <graph_id> --deployment <name> --format json
    // 4. Parse JSON output
    // 5. Verify structure has nodes and edges arrays
}

#[test]
fn test_graph_commands_help() {
    // Test --help for graph, graph list, graph get
    // Verify expected options documented
}
```

### 4. CRUD Lifecycle Tests

**Location:** `cli/tests/graph_command_test.rs`

Following the CRUD Lifecycle Pattern from `docs/dev/testing/crud-lifecycle-pattern.md`:

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_graph_workflow_crud() {
    // ═══════════════════════════════════════════════════════════════════
    // Step 1: CREATE - Get or create test deployment via SDK
    // ═══════════════════════════════════════════════════════════════════
    let deployment = get_test_deployment();

    // ═══════════════════════════════════════════════════════════════════
    // Step 2: VERIFY - Confirm deployment exists via SDK
    // ═══════════════════════════════════════════════════════════════════
    assert!(deployment.id.len() > 0);

    // ═══════════════════════════════════════════════════════════════════
    // Step 3: READ - List graphs via CLI
    // ═══════════════════════════════════════════════════════════════════
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list", "--deployment", &deployment.name, "--format", "json"]);
    let output = cmd.output()?;

    // ═══════════════════════════════════════════════════════════════════
    // Step 4: VERIFY - Parse output, verify graphs appear
    // ═══════════════════════════════════════════════════════════════════
    let graphs: Vec<Value> = serde_json::from_str(&stdout)?;
    assert!(!graphs.is_empty(), "BUG: Expected graphs in deployment");

    // ═══════════════════════════════════════════════════════════════════
    // Step 5: READ - Get specific graph via CLI
    // ═══════════════════════════════════════════════════════════════════
    let graph_id = graphs[0]["graph_id"].as_str().unwrap();
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "get", graph_id, "--deployment", &deployment.name, "--format", "json"]);
    let output = cmd.output()?;

    // ═══════════════════════════════════════════════════════════════════
    // Step 6: VERIFY - Parse output, verify structure
    // ═══════════════════════════════════════════════════════════════════
    let graph: Value = serde_json::from_str(&stdout)?;
    assert!(graph.get("nodes").is_some(), "Graph should have nodes");
    assert!(graph.get("edges").is_some(), "Graph should have edges");
}
```

### 5. Deprecation Warning Tests

```rust
#[test]
fn test_old_graph_list_shows_deprecation() {
    // If old syntax is still supported with deprecation warning
    let mut cmd = langstar_cmd();
    cmd.args(["graph", "list"]); // Old syntax without --deployment

    let output = cmd.output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should warn about deprecated usage
    assert!(
        stderr.contains("deprecated") || stderr.contains("DEPRECATED"),
        "Should show deprecation warning"
    );
}
```

## Test Data & Fixtures

### Test Deployment

Tests use the shared test deployment from `tests/fixtures/test-graph-deployment/`:
- Minimal echo graph for fast deployment
- Known graph structure: `START → echo → END`

### Naming Conventions

| Type | Pattern | Usage |
|------|---------|-------|
| PR/Dev | `pr-integration-test-{timestamp}` | Shared deployment for faster tests |
| Release | `release-integration-test-{timestamp}` | Fresh deployment for full lifecycle |

### Test Resource Cleanup

- `TestDeployment` struct implements RAII cleanup
- Periodic cron job cleans deployments older than 4 hours
- Integration tests use `#[cfg_attr(not(feature = "integration-tests"), ignore)]`

## Success Criteria

### Coverage Targets

- [ ] All SDK graph methods have unit tests with mocks
- [ ] All SDK graph methods have integration tests
- [ ] All CLI deployment subcommands have tests
- [ ] All CLI graph subcommands have tests
- [ ] CRUD lifecycle test verifies end-to-end behavior
- [ ] Error handling tested for common failure modes

### CI Requirements

```bash
# All checks must pass
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features && \
cargo fmt --check
```

### Manual Testing Checklist

- [ ] `langstar deployment list` shows deployments
- [ ] `langstar deployment get <id>` shows deployment details
- [ ] `langstar graph list --deployment <name>` shows graphs
- [ ] `langstar graph get <id> --deployment <name>` shows graph structure
- [ ] JSON output is valid and parseable
- [ ] Help text is complete and accurate

## Toyota Andon Cord Reminder

**All tests must pass before merge. No exceptions.**

From `HIGH_LEVEL_TESTING_GUIDELINES.md`:

### Never Acceptable
- ❌ "My changes didn't introduce this failure"
- ❌ "It's a flaky test, we can ignore it"
- ❌ "The test is wrong, not the code"
- ❌ "We'll fix it in a follow-up PR"

### Always Required
- ✅ Fix the failure before merge
- ✅ If test is wrong, fix test then verify code
- ✅ All CI checks must be green before merge

## Implementation Checklist

### Immediate Actions
- [x] SDK unit tests with mockito (graph_test.rs) - Complete
- [x] CLI deployment tests (deployment_command_test.rs) - Complete
- [ ] CLI graph tests (graph_command_test.rs) - To create
- [ ] SDK integration tests (graph_integration_test.rs) - To create

### Before PR
- [ ] All tests pass locally
- [ ] `cargo fmt && cargo clippy` clean
- [ ] Integration tests run successfully with env vars
- [ ] Help text verified accurate

## References

### Testing Documentation
- `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` - Core principles
- `docs/dev/testing/crud-lifecycle-pattern.md` - CLI→SDK verification
- `docs/dev/testing/cli-integration-tests.md` - CLI patterns
- `docs/dev/testing/sdk-integration-tests.md` - SDK patterns
- `docs/dev/testing/mocking-patterns.md` - When to mock

### Related Issues
- #571 - This testing issue
- #527 - Parent milestone issue
- #569 - CLI graph commands implementation
- #567 - CLI deployment commands implementation
- #566 - SDK graph client implementation

### Test File Examples
- `cli/tests/assistant_command_test.rs` - CLI test patterns
- `sdk/tests/integration_deployment_workflow.rs` - SDK integration patterns
- `cli/tests/prompt_scoping_test.rs` - CRUD lifecycle example
