# Test Plan: ls-prompt-structured-outputs Final Testing Verification

**Milestone**: ls-prompt-structured-outputs (#402)
**Parent Ticket**: #660 (Final testing verification)
**Plan Ticket**: #661 (Audit and plan)
**Created**: 2025-12-10

## Overview

This test plan addresses the 4 critical issues identified in the retroactive test audit for the ls-prompt-structured-outputs milestone. The plan breaks down remediation into logical sub-tickets that can be implemented independently (where possible) or sequentially (where dependencies exist).

## Audit Summary

See companion document: `ls-prompt-structured-outputs-test-audit.md`

**Critical Issues Found:**

1. SDK integration tests use unconditional `#[ignore]`
2. SDK integration tests not run in CI
3. CLI tests don't follow CRUD lifecycle pattern
4. Silent skip pattern in CLI tests

**Goal**: Fix all critical issues so milestone can be marked as released with confidence.

---

## Sub-Ticket Breakdown

### Ticket 1: Fix SDK Integration Test Annotations

**Title**: `660.2-sdk-ignore Fix SDK integration test ignore attributes`

**Description**:
Replace unconditional `#[ignore]` attributes with conditional `#[cfg_attr(not(feature = "integration-tests"), ignore)]` in SDK integration tests.

**Files to Modify**:

- `sdk/tests/structured_prompts_integration_test.rs`
  - Line 138: `test_push_structured_prompt_integration`
  - Line 193: `test_pull_structured_prompt_integration`
  - Line 246: `test_structured_prompt_round_trip_integration`
  - Line 303: `test_push_function_calling_method`

**Changes Required**:

```rust
// BEFORE (4 occurrences)
#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_push_structured_prompt_integration() { ... }

// AFTER
#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
async fn test_push_structured_prompt_integration() { ... }
```

**Acceptance Criteria**:

- [ ] All 4 test functions updated with conditional ignore
- [ ] Tests run with `cargo test --features integration-tests -p langstar-sdk --test structured_prompts_integration_test`
- [ ] Tests are ignored without the feature flag
- [ ] Pre-commit checklist passes

**Dependencies**: None (can be done first)

**Estimated Complexity**: LOW (10 minutes)

**Verification**:

```bash
# Should run 4 tests
cargo test --features integration-tests -p langstar-sdk --test structured_prompts_integration_test

# Should ignore 4 tests
cargo test -p langstar-sdk --test structured_prompts_integration_test
```

---

### Ticket 2: Add SDK Integration Tests to CI

**Title**: `660.3-ci-sdk Add SDK integration tests to CI workflow`

**Description**:
Update CI configuration to run SDK integration tests in addition to CLI integration tests.

**Files to Modify**:

- `.github/workflows/ci.yml`

**Current State** (line 198):

```yaml
run: cargo nextest run --profile integration -p langstar --features integration-tests
```

**Option A - Separate Job (Recommended)**:

Add new job after existing `integration-tests` job:

```yaml
  integration-tests-sdk:
    name: SDK Integration Tests
    runs-on: ubuntu-latest
    needs: changes
    if: needs.changes.outputs.code == 'true'

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-nextest
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Run SDK integration tests
        run: cargo nextest run --profile integration -p langstar-sdk --features integration-tests
        timeout-minutes: 15
        env:
          LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
          LANGSMITH_ORGANIZATION_ID: ${{ secrets.LANGSMITH_ORGANIZATION_ID }}

      - name: Publish SDK Integration Test Results
        uses: EnricoMi/publish-unit-test-result-action@v2
        if: always()
        with:
          files: target/nextest/integration/junit-integration.xml
          check_name: "SDK Integration Test Results"
          comment_title: "SDK Integration Test Results"
```

**Option B - Extend Existing Job**:

Modify line 198 to include SDK package:

```yaml
run: cargo nextest run --profile integration --features integration-tests -p langstar -p langstar-sdk
```

**Acceptance Criteria**:

- [ ] SDK integration tests run in CI
- [ ] CI shows 4 SDK integration test results
- [ ] CI fails if SDK integration tests fail
- [ ] Test results published to PR checks
- [ ] Option A or Option B implemented (document which)

**Dependencies**: Ticket 1 (must fix ignore attributes first)

**Estimated Complexity**: MEDIUM (20-30 minutes)

**Verification**:

- Push to PR and verify CI runs SDK tests
- Check GitHub Actions logs for "structured_prompts_integration_test"
- Verify 4 tests execute and results are reported

---

### Ticket 3: Remove Silent Skip Pattern from CLI Tests

**Title**: `660.4-cli-skip Remove silent skip pattern from CLI integration tests`

**Description**:
Replace the `check_env_vars()` silent skip pattern with explicit `.expect()` calls that fail when required environment variables are missing.

**Files to Modify**:

- `cli/tests/prompt_structured_test.rs`
  - Lines 38-74: Remove `check_env_vars()` function
  - Lines 119-123: Update test to use `.expect()`
  - Update all 9 integration tests that call `check_env_vars()`

**Current Pattern (WRONG)**:

```rust
fn check_env_vars() -> bool {
    let has_api_key = std::env::var("LANGSMITH_API_KEY").is_ok();
    if !has_api_key {
        return false;
    }
    // ... more checks
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_structured_prompt() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;  // ❌ Silent skip
    }
    // test body...
}
```

**Required Pattern (CORRECT)**:

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_structured_prompt() {
    // Explicit failures for missing env vars
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");

    // Fail if workspace ID is set (incompatible with owner/repo format)
    if std::env::var("LANGSMITH_WORKSPACE_ID").is_ok() {
        panic!("LANGSMITH_WORKSPACE_ID must NOT be set for these tests (use owner/repo format)");
    }

    // test body...
}
```

**Tests to Update** (9 total):

1. `test_cli_push_structured_prompt` (line 119)
2. `test_cli_push_invalid_schema_file` (line 160)
3. `test_cli_push_missing_schema_file` (line 191)
4. `test_cli_push_invalid_method` (line 221)
5. `test_cli_push_function_calling_method` (line 255)
6. `test_cli_pull_structured_prompt` (line 290)
7. `test_cli_structured_prompt_round_trip` (line 313)
8. `test_cli_push_structured_prompt_json_output` (line 376)
9. `test_cli_pull_structured_prompt_json_output` (line 422)

**Acceptance Criteria**:

- [ ] `check_env_vars()` function removed
- [ ] All 9 tests updated with explicit `.expect()` calls
- [ ] Tests fail explicitly (not silently) when env vars missing
- [ ] Tests still pass when env vars are set
- [ ] Pre-commit checklist passes

**Dependencies**: None (can be done in parallel with Ticket 1-2)

**Estimated Complexity**: MEDIUM (30-45 minutes)

**Verification**:

```bash
# Without env vars - should fail explicitly
unset LANGSMITH_API_KEY
cargo test --features integration-tests -p langstar --test prompt_structured_test
# Should see: "LANGSMITH_API_KEY must be set for integration tests"

# With env vars - should pass
export LANGSMITH_API_KEY="..."
export LANGSMITH_ORGANIZATION_ID="..."
cargo test --features integration-tests -p langstar --test prompt_structured_test
```

---

### Ticket 4: Add CRUD Lifecycle Verification to CLI Round-Trip Test

**Title**: `660.5-cli-crud Add SDK verification to CLI round-trip test`

**Description**:
Update the main CLI round-trip test to follow the CRUD lifecycle pattern by adding SDK verification after CLI operations.

**Files to Modify**:

- `cli/tests/prompt_structured_test.rs:313-372` - `test_cli_structured_prompt_round_trip`

**Why Start Here**:
Focus on one representative test first to establish the pattern. Other tests can follow in subsequent tickets if needed.

**Current Test Flow**:

1. Push via CLI
2. Pull via CLI
3. Verify CLI output

**Required Test Flow (CRUD Pattern)**:

1. Push via CLI
2. **Verify via SDK** (new step)
3. Pull via CLI
4. **Verify via SDK** (new step)
5. **Cleanup via SDK** (new step)

**Implementation Guidance**:

Add SDK client helper:

```rust
// Add to top of file or in test
fn create_sdk_client() -> Result<LangchainClient, String> {
    let auth = AuthConfig::from_env()
        .map_err(|e| format!("Auth config error: {}", e))?;
    LangchainClient::new(auth)
        .map_err(|e| format!("Client creation error: {}", e))
}

fn create_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("Failed to create runtime")
}
```

Update test:

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_structured_prompt_round_trip() {
    // ... existing env var checks ...

    let runtime = create_runtime();
    let sdk_client = create_sdk_client().expect("SDK client required");

    // Step 1: Push via CLI
    let schema_file = create_temp_schema_file();
    // ... existing push code ...
    let push_output = push_cmd.output().expect("Push failed");
    assert!(push_output.status.success());
    let commit_hash = extract_commit_hash(&push_output.stdout);

    // Step 2: VERIFY VIA SDK (NEW!)
    let pushed_prompt = runtime.block_on(async {
        sdk_client.prompts()
            .pull_structured_prompt(TEST_OWNER, TEST_REPO, &commit_hash)
            .await
    }).expect("SDK verification failed");

    // Parse expected schema from temp file
    let expected_schema: Value = serde_json::from_str(
        &std::fs::read_to_string(schema_file.path()).unwrap()
    ).unwrap();

    assert_eq!(pushed_prompt.schema_, expected_schema,
        "SDK should show schema that was pushed via CLI");
    assert_eq!(pushed_prompt.structured_output_kwargs.method, "json_schema",
        "SDK should show correct method");

    // Step 3: Pull via CLI
    // ... existing pull code ...

    // Step 4: VERIFY via SDK again (confirm data still correct)
    let final_prompt = runtime.block_on(async {
        sdk_client.prompts()
            .pull_structured_prompt(TEST_OWNER, TEST_REPO, "latest")
            .await
    }).expect("Final SDK verification failed");

    assert_eq!(final_prompt.schema_, expected_schema,
        "Schema should remain consistent");

    // Step 5: CLEANUP via SDK
    runtime.block_on(async {
        sdk_client.prompts()
            .delete(&format!("{}/{}", TEST_OWNER, TEST_REPO))
            .await
    }).ok(); // Ignore cleanup errors
}
```

**Acceptance Criteria**:

- [ ] SDK client creation helper added
- [ ] Tokio runtime helper added
- [ ] SDK verification added after CLI push
- [ ] SDK verification added after CLI pull
- [ ] SDK cleanup added at end
- [ ] Test verifies schema content via SDK
- [ ] Test verifies method via SDK
- [ ] Test still passes when run with integration-tests feature
- [ ] Pre-commit checklist passes

**Dependencies**: Ticket 3 (silent skip removal should be done first)

**Estimated Complexity**: MEDIUM-HIGH (1-2 hours)

**Verification**:

```bash
cargo test --features integration-tests -p langstar --test prompt_structured_test test_cli_structured_prompt_round_trip -- --nocapture
```

**Reference**:

- `cli/tests/prompt_scoping_test.rs:test_prompt_crud_lifecycle_private_visibility`
- `docs/dev/testing/crud-lifecycle-pattern.md`

---

### Ticket 5: Add Comprehensive SDK→CLI→SDK Round-Trip Test (Optional)

**Title**: `660.6-full-roundtrip Add comprehensive SDK→CLI→SDK verification test`

**Description**:
Add a new test that creates via SDK, verifies via CLI, confirms via SDK. This provides the strongest guarantee of CLI/SDK consistency.

**Files to Modify**:

- `cli/tests/prompt_structured_test.rs` (add new test function)

**Test Structure**:

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_structured_prompt_full_lifecycle_sdk_cli_sdk() {
    // Setup
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY required");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID required");

    let sdk_client = create_sdk_client().expect("SDK client required");
    let runtime = create_runtime();

    // 1. CREATE via SDK
    let structured_prompt = create_test_structured_prompt();
    let push_result = runtime.block_on(async {
        sdk_client.prompts()
            .push_structured_prompt(TEST_OWNER, TEST_REPO, structured_prompt.clone(), None)
            .await
    }).expect("SDK push failed");

    let commit_hash = push_result.commit.commit_hash;

    // 2. VERIFY via CLI
    let bin = get_langstar_bin();
    let mut pull_cmd = Command::new(&bin);
    pull_cmd.args([
        "prompt", "pull",
        &format!("{}/{}", TEST_OWNER, TEST_REPO),
        "--commit", &commit_hash,
        "--format", "json"
    ]);

    let output = pull_cmd.output().expect("CLI pull failed");
    assert!(output.status.success(), "CLI should succeed");

    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .expect("CLI output should be valid JSON");

    // Verify CLI shows correct data
    let cli_schema = &cli_json["kwargs"]["schema_"];
    assert_eq!(cli_schema, &structured_prompt.schema_,
        "CLI should display schema pushed via SDK");

    // 3. VERIFY via SDK (round-trip confirmation)
    let verified_prompt = runtime.block_on(async {
        sdk_client.prompts()
            .pull_structured_prompt(TEST_OWNER, TEST_REPO, &commit_hash)
            .await
    }).expect("SDK verification failed");

    assert_eq!(verified_prompt.schema_, structured_prompt.schema_,
        "Schema should survive full SDK→CLI→SDK round-trip");
    assert_eq!(verified_prompt.structured_output_kwargs.method,
        structured_prompt.structured_output_kwargs.method,
        "Method should survive full round-trip");

    // 4. CLEANUP
    runtime.block_on(async {
        sdk_client.prompts()
            .delete(&format!("{}/{}", TEST_OWNER, TEST_REPO))
            .await
    }).ok();
}
```

**Acceptance Criteria**:

- [ ] New test function added
- [ ] Test creates prompt via SDK
- [ ] Test verifies via CLI (JSON output)
- [ ] Test confirms via SDK
- [ ] Test cleans up resources
- [ ] Test passes with integration-tests feature
- [ ] Pre-commit checklist passes

**Dependencies**: Ticket 4 (uses same helper functions)

**Estimated Complexity**: MEDIUM (45-60 minutes)

**Priority**: Optional (enhances coverage but not critical)

**Verification**:

```bash
cargo test --features integration-tests -p langstar --test prompt_structured_test test_structured_prompt_full_lifecycle_sdk_cli_sdk -- --nocapture
```

---

### Ticket 6: Final Verification and Documentation

**Title**: `660.7-verify Final verification and milestone release preparation`

**Description**:
Run final verification steps, re-run audit, and prepare milestone for release.

**Tasks**:

1. **Run Complete Test Suite**:

```bash
# All unit tests
cargo test --workspace --lib

# All integration tests
cargo test --workspace --features integration-tests

# Pre-commit checklist
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features
```

2. **Verify CI Passes**:

- Push all changes to feature branch
- Verify all CI jobs pass (including new SDK integration tests)
- Check test result reports

3. **Re-Run Audit**:

```bash
/gh-milestones:test-audit ls-prompt-structured-outputs
```

4. **Document Improvements**:
   Add section to milestone release notes documenting test improvements:

- 4 critical issues resolved
- SDK integration tests now run in CI
- CLI tests follow CRUD lifecycle pattern
- No silent skip patterns
- X% test coverage improvement

5. **Mark Milestone as Released**:
   Use the appropriate milestone release workflow.

**Acceptance Criteria**:

- [ ] All tests pass locally
- [ ] All CI checks pass
- [ ] Audit shows 0 critical issues
- [ ] Test improvements documented
- [ ] Milestone marked as released

**Dependencies**: Tickets 1, 2, 3, 4 (must all be complete)

**Estimated Complexity**: MEDIUM (1 hour)

---

## Implementation Order

### Phase 1: Quick Wins (Can be parallel)

1. **Ticket 1**: Fix SDK ignore attributes (10 min) → MERGE
2. **Ticket 3**: Remove silent skip pattern (30-45 min) → MERGE

### Phase 2: CI Integration (Sequential dependency)

3. **Ticket 2**: Add SDK tests to CI (20-30 min, needs Ticket 1) → MERGE

### Phase 3: CRUD Lifecycle (Sequential)

4. **Ticket 4**: Add CRUD verification to round-trip test (1-2 hours, needs Ticket 3) → MERGE
5. **Ticket 5**: (Optional) Add SDK→CLI→SDK test (45-60 min, needs Ticket 4) → MERGE if time allows

### Phase 4: Release

6. **Ticket 6**: Final verification and release (1 hour, needs all previous) → MERGE

**Total Estimated Time**: 3-5 hours (excluding optional Ticket 5)

---

## Success Criteria

The testing verification is complete when:

- [ ] All 4 critical issues from audit are resolved
- [ ] SDK integration tests run in CI and pass
- [ ] At least 1 CLI test follows CRUD lifecycle pattern
- [ ] No silent skip patterns remain
- [ ] Pre-commit checklist passes
- [ ] All CI checks pass
- [ ] Test audit re-run shows 0 critical issues
- [ ] Milestone can be confidently marked as released

---

## Rollback Plan

If issues arise during implementation:

1. **Ticket 1**: Easy rollback, just revert commit
2. **Ticket 2**: Can disable CI job if causing problems
3. **Ticket 3**: Can temporarily re-add `check_env_vars()` if needed
4. **Ticket 4**: Can skip if too complex, but recommended to complete
5. **Ticket 5**: Optional, can defer to future work

---

## References

- **Audit Report**: `docs/implementation/ls-prompt-structured-outputs-test-audit.md`
- **Parent Ticket**: #660
- **Milestone**: #402 (ls-prompt-structured-outputs)
- **Testing Standards**: `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md`
- **CRUD Pattern**: `docs/dev/testing/crud-lifecycle-pattern.md`
- **Issue #536 Post-Mortem**: Why this work is important
- **Issue #647**: Silent skip anti-pattern context
