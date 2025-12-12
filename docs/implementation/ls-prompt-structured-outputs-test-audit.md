# Test Audit Report: ls-prompt-structured-outputs

**Milestone**: ls-prompt-structured-outputs (#402)
**Audit Date**: 2025-12-10
**Audit Tool**: `/gh-milestones:test-audit`
**Related Ticket**: #661

## Executive Summary

This milestone implemented structured output prompt support for the langstar CLI and SDK. The implementation was completed before the test-plan workflow existed, making this a retroactive audit.

**Key Findings:**

- ✅ 22 tests implemented with good functional coverage
- ❌ 4 critical issues preventing tests from running in CI
- ⚠️ 3 warnings about test quality and patterns
- **Compliance Rate**: ~65%
- **Recommendation**: Fix critical issues before marking milestone as released

---

## Summary

- **Tests Planned**: 8 test categories (from issue #408)
- **Tests Implemented**: 22 tests total
  - 4 SDK integration tests (real API)
  - 6 SDK mocked HTTP tests
  - 9 SDK unit tests
  - 9 CLI integration tests
- **Compliance Rate**: ~65% (critical issues found)
- **Critical Issues**: 4 (must fix before release)
- **Warnings**: 3 (should address)

---

## Critical Issues (Must Fix Before Release)

### Issue 1: SDK Integration Tests Use Unconditional `#[ignore]`

**Severity**: CRITICAL
**Toyota Andon Cord**: ✅ VIOLATION

**Location**: `sdk/tests/structured_prompts_integration_test.rs`

- Line 138: `test_push_structured_prompt_integration`
- Line 193: `test_pull_structured_prompt_integration`
- Line 246: `test_structured_prompt_round_trip_integration`
- Line 303: `test_push_function_calling_method`

**Problem**: Tests marked with `#[ignore]` without conditional feature flag.

**Impact**: These 4 integration tests NEVER run in CI, even with `--features integration-tests` enabled. The tests exist and appear to provide coverage, but they create false confidence because they don't execute.

**Current Code (WRONG)**:

```rust
#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_push_structured_prompt_integration() { ... }
```

**Required Fix (CORRECT)**:

```rust
#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
async fn test_push_structured_prompt_integration() { ... }
```

**Remediation**:

```bash
# Replace all 4 unconditional #[ignore] with conditional version
# Lines: 138, 193, 246, 303
```

**Reference**: `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md:166`

---

### Issue 2: SDK Integration Tests NOT Run in CI

**Severity**: CRITICAL
**Toyota Andon Cord**: ✅ VIOLATION

**Location**: `.github/workflows/ci.yml:198`

**Problem**: CI configuration only runs integration tests for `-p langstar` (CLI package), not `-p langstar-sdk` (SDK package).

**Impact**: The 4 SDK integration tests with real API calls never execute in CI. Combined with Issue #1, this creates a complete blind spot in CI coverage.

**Current CI Command**:

```yaml
run: cargo nextest run --profile integration -p langstar --features integration-tests
```

**What's Missing**: SDK integration tests in the `langstar-sdk` package are not executed.

**Remediation Options**:

**Option A (Recommended)**: Add separate SDK integration test job:

```yaml
integration-tests-sdk:
  name: SDK Integration Tests
  runs-on: ubuntu-latest
  needs: changes
  if: needs.changes.outputs.code == 'true'
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
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
```

**Option B**: Extend existing integration test job to include SDK:

```yaml
run: cargo nextest run --profile integration --workspace --features integration-tests
```

**Reference**: `.github/workflows/ci.yml:185-203`

---

### Issue 3: CLI Tests Don't Follow CRUD Lifecycle Pattern

**Severity**: CRITICAL
**Toyota Andon Cord**: ⚠️ PARTIAL VIOLATION

**Location**: All 9 tests in `cli/tests/prompt_structured_test.rs`

**Problem**: Tests verify CLI → CLI, not CLI → SDK → CLI. They check that CLI commands produce output, but don't verify via SDK that the correct data was persisted to the API.

**Impact**: Cannot catch bugs like issue #536 where CLI command succeeds and produces output, but the wrong data is sent to or retrieved from the API.

**Current Pattern (Insufficient)**:

```rust
// Test: test_cli_structured_prompt_round_trip (line 313)

// Push via CLI
push_cmd.assert().success().stdout(contains("Commit hash:"));

// Pull via CLI
pull_cmd.assert().success().stdout(contains("json_schema"));

// ❌ MISSING: Verification via SDK that data was actually persisted correctly
```

**Required Pattern**:

```rust
// 1. CREATE via CLI
let push_output = push_cmd.output()?;
assert!(push_output.status.success());
let commit_hash = extract_commit_hash(&push_output.stdout);

// 2. VERIFY via SDK (this is the missing step!)
let client = create_sdk_client()?;
let pulled_prompt = client.prompts()
    .pull_structured_prompt(owner, repo, &commit_hash).await?;

assert_eq!(pulled_prompt.schema_, expected_schema,
    "Schema should match what was pushed");
assert_eq!(pulled_prompt.structured_output_kwargs.method, "json_schema",
    "Method should match");

// 3. CLEANUP via SDK
client.prompts().delete(&repo_handle).await?;
```

**Files Needing Updates**:

- `cli/tests/prompt_structured_test.rs:313` - `test_cli_structured_prompt_round_trip`
- `cli/tests/prompt_structured_test.rs:119` - `test_cli_push_structured_prompt`
- `cli/tests/prompt_structured_test.rs:290` - `test_cli_pull_structured_prompt`

**Why This Matters**:

- Issue #536 post-mortem: `langstar prompt list` returned zero results for private prompts despite tests passing
- Tests only checked exit codes, not actual API state
- CRUD lifecycle pattern would have caught the bug

**Reference**:

- `docs/dev/testing/crud-lifecycle-pattern.md:57-72`
- `docs/dev/testing/post-mortems/536-prompt-list-testing-gap.md`

---

### Issue 4: Silent Skip Pattern in CLI Tests

**Severity**: CRITICAL
**Toyota Andon Cord**: ⚠️ VIOLATION (Issue #647 pattern)

**Location**: `cli/tests/prompt_structured_test.rs`

- Line 48: `check_env_vars()` helper function
- Lines 120-123: Example usage in `test_cli_push_structured_prompt`

**Problem**: Tests use early return pattern instead of failing explicitly when required environment variables are missing.

**Impact**:

- Creates false confidence in test counts ("289 tests passing" may include silently skipped tests)
- Violates Issue #647 resolution: integration tests MUST fail explicitly when env vars missing
- Makes it unclear whether tests actually ran or just skipped

**Current Code (WRONG)**:

```rust
fn check_env_vars() -> bool {
    let has_api_key = std::env::var("LANGSMITH_API_KEY").is_ok();
    if !has_api_key {
        return false;  // ❌ Silent skip
    }
    let has_org_id = std::env::var("LANGSMITH_ORGANIZATION_ID").is_ok();
    if !has_org_id {
        println!("⚠️  Skipping test: LANGSMITH_ORGANIZATION_ID not set");
        return false;  // ❌ Silent skip
    }
    // Check workspace_id logic...
    true
}

#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_structured_prompt() {
    if !check_env_vars() {
        println!("Skipping test: Required environment variables not set");
        return;  // ❌ Test appears to pass but didn't run
    }
    // ... test implementation
}
```

**Required Fix (CORRECT)**:

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_cli_push_structured_prompt() {
    // Explicit failure if env vars missing
    let api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");

    // Workspace ID check with clear error message
    if std::env::var("LANGSMITH_WORKSPACE_ID").is_ok() {
        panic!("LANGSMITH_WORKSPACE_ID must NOT be set for these tests (they use owner/repo format)");
    }

    // Test implementation...
}
```

**Remediation**:

1. Remove `check_env_vars()` helper function entirely
2. Use `.expect()` with clear error messages in each test
3. Follow pattern from `HIGH_LEVEL_TESTING_GUIDELINES.md:99-112`

**Reference**:

- `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md:96-114`
- Issue #647: Silent skip anti-pattern resolution

---

## Warnings (Should Address)

### Warning 1: Missing Comprehensive SDK→CLI→SDK Round-Trip Test

**Severity**: MEDIUM

**Current State**: Tests verify push→pull but not the full bidirectional pattern.

**Suggestion**: Add test that creates via SDK, verifies via CLI, confirms via SDK. This provides the strongest guarantee that CLI and SDK are in sync.

**Recommended Test Structure**:

```rust
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_structured_prompt_full_lifecycle_sdk_cli_sdk() {
    let client = create_sdk_client().expect("SDK client required");
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // 1. CREATE via SDK
    let structured_prompt = create_test_movie_review_prompt();
    let push_result = runtime.block_on(async {
        client.prompts()
            .push_structured_prompt(OWNER, REPO, structured_prompt.clone(), None)
            .await
    }).expect("SDK push failed");

    let commit_hash = push_result.commit.commit_hash;

    // 2. VERIFY via CLI
    let mut cmd = Command::new(get_langstar_bin());
    cmd.args(["prompt", "pull", &format!("{}/{}", OWNER, REPO),
              "--commit", &commit_hash, "--format", "json"]);
    let output = cmd.output().expect("CLI failed");
    assert!(output.status.success(), "CLI should succeed");

    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .expect("CLI output should be valid JSON");

    // Verify CLI shows correct schema
    let cli_schema = cli_json["kwargs"]["schema_"].clone();
    assert_eq!(cli_schema, structured_prompt.schema_);

    // 3. VERIFY via SDK (round-trip confirmation)
    let verified_prompt = runtime.block_on(async {
        client.prompts()
            .pull_structured_prompt(OWNER, REPO, &commit_hash)
            .await
    }).expect("SDK verification failed");

    assert_eq!(verified_prompt.schema_, structured_prompt.schema_,
        "Schema should survive full round-trip");
    assert_eq!(verified_prompt.structured_output_kwargs.method,
        structured_prompt.structured_output_kwargs.method,
        "Method should survive full round-trip");

    // 4. CLEANUP
    runtime.block_on(async {
        client.prompts().delete(&format!("{}/{}", OWNER, REPO)).await
    }).ok();
}
```

**Benefit**: Catches discrepancies between SDK serialization and CLI parsing.

---

### Warning 2: Limited Error Type Verification

**Severity**: LOW

**Location**:

- `sdk/tests/structured_prompts_test.rs:131-133` - `test_push_structured_prompt_invalid_schema`
- `sdk/tests/structured_prompts_test.rs:151-153` - `test_push_structured_prompt_invalid_method`

**Problem**: Error tests check for string content in debug output instead of verifying specific error types.

**Current Pattern**:

```rust
assert!(result.is_err());
let err_str = format!("{:?}", result.unwrap_err());
assert!(err_str.contains("InvalidSchemaError"));
```

**Better Pattern**:

```rust
let err = result.unwrap_err();
assert!(matches!(err, LangchainError::InvalidSchemaError(_)),
    "Expected InvalidSchemaError, got: {:?}", err);
```

**Benefit**: More precise error verification, catches error type changes.

---

### Warning 3: Hardcoded Test Repository

**Severity**: LOW

**Location**: Multiple test files use:

- `TEST_OWNER = "codekiln"`
- `TEST_REPO = "langstar-structured-test"`

**Problem**: Using the same repository for all tests could cause conflicts if tests run in parallel or if test cleanup fails.

**Current Approach**: Works fine if tests are serialized and cleanup is reliable.

**Better Approach**: Generate unique repository names per test run:

```rust
fn generate_test_repo_name() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("langstar-structured-test-{}", timestamp)
}
```

**Benefit**: Tests are more isolated and can run in parallel without conflicts.

**Note**: Not critical since tests appear to be serialized currently.

---

## Test Plan Coverage Matrix

| Test Case (from issue #408)    | Implemented? | File:Line                                            | Status                           |
| ------------------------------ | ------------ | ---------------------------------------------------- | -------------------------------- |
| **Unit Tests**                 |              |                                                      |                                  |
| StructuredPrompt serialization | ✅ Yes       | sdk/src/prompts.rs:838-933                           | ✅ Working                       |
| Schema validation              | ✅ Yes       | sdk/tests/structured_prompts_test.rs:116-133         | ✅ Working                       |
| **Mocked HTTP Tests**          |              |                                                      |                                  |
| Client push method             | ✅ Yes       | sdk/tests/structured_prompts_test.rs:78-113          | ✅ Working                       |
| Client pull method             | ✅ Yes       | sdk/tests/structured_prompts_test.rs:193-270         | ✅ Working                       |
| Error cases (invalid schema)   | ✅ Yes       | sdk/tests/structured_prompts_test.rs:116-133         | ✅ Working                       |
| Error cases (invalid method)   | ✅ Yes       | sdk/tests/structured_prompts_test.rs:136-153         | ✅ Working                       |
| Both method types              | ✅ Yes       | sdk/tests/structured_prompts_test.rs:156-190         | ✅ Working                       |
| Round-trip (mock)              | ✅ Yes       | sdk/tests/structured_prompts_test.rs:309-379         | ✅ Working                       |
| **SDK Integration Tests**      |              |                                                      |                                  |
| Push to LangSmith              | ✅ Yes       | sdk/tests/structured_prompts_integration_test.rs:139 | ❌ Not in CI (Issue #1, #2)      |
| Pull from LangSmith            | ✅ Yes       | sdk/tests/structured_prompts_integration_test.rs:194 | ❌ Not in CI (Issue #1, #2)      |
| Round-trip (real API)          | ✅ Yes       | sdk/tests/structured_prompts_integration_test.rs:247 | ❌ Not in CI (Issue #1, #2)      |
| Function_calling method        | ✅ Yes       | sdk/tests/structured_prompts_integration_test.rs:304 | ❌ Not in CI (Issue #1, #2)      |
| **CLI Integration Tests**      |              |                                                      |                                  |
| CLI push command               | ✅ Yes       | cli/tests/prompt_structured_test.rs:119              | ⚠️ No SDK verification (Issue #3) |
| CLI pull command               | ✅ Yes       | cli/tests/prompt_structured_test.rs:290              | ⚠️ No SDK verification (Issue #3) |
| CLI round-trip                 | ✅ Yes       | cli/tests/prompt_structured_test.rs:313              | ⚠️ No SDK verification (Issue #3) |
| Error: invalid schema file     | ✅ Yes       | cli/tests/prompt_structured_test.rs:160              | ✅ Working                       |
| Error: missing schema file     | ✅ Yes       | cli/tests/prompt_structured_test.rs:191              | ✅ Working                       |
| Error: invalid method          | ✅ Yes       | cli/tests/prompt_structured_test.rs:221              | ✅ Working                       |
| Function_calling method        | ✅ Yes       | cli/tests/prompt_structured_test.rs:255              | ✅ Working                       |
| JSON output format             | ✅ Yes       | cli/tests/prompt_structured_test.rs:376,422          | ✅ Working                       |

**Legend**:

- ✅ Working: Test implemented and runs correctly
- ❌ Not in CI: Test exists but doesn't run in CI (critical)
- ⚠️ No SDK verification: Test runs but missing CRUD lifecycle verification (critical)

---

## CI Configuration Status

**Integration Test Job**: `.github/workflows/ci.yml:185-230`

Checklist:

- [x] LANGSMITH_API_KEY in workflow secrets (line 201)
- [x] LANGSMITH_ORGANIZATION_ID in workflow secrets (line 202)
- [x] LANGSMITH_WORKSPACE_ID in workflow secrets (line 203)
- [x] `integration-tests` feature enabled in CI (line 198)
- [x] CLI integration test job configured (line 185)
- [ ] **MISSING**: SDK integration tests in CI job

**Current Command**:

```yaml
run: cargo nextest run --profile integration -p langstar --features integration-tests
```

**Problem**: Only tests the `langstar` package (CLI), not `langstar-sdk` package.

---

## Anti-Pattern Detection

| Pattern                   | Found?    | Location                                                       | Severity |
| ------------------------- | --------- | -------------------------------------------------------------- | -------- |
| Unconditional `#[ignore]` | ❌ Yes    | sdk/tests/structured_prompts_integration_test.rs (all 4 tests) | CRITICAL |
| Exit-code-only assertions | ✅ No     | -                                                              | -        |
| Missing cleanup           | ⚠️ Partial | CLI tests don't clean up created prompts                       | MEDIUM   |
| Hardcoded test data       | ⚠️ Yes     | All tests use same owner/repo                                  | LOW      |
| Silent skip pattern       | ❌ Yes    | cli/tests/prompt_structured_test.rs:48                         | CRITICAL |
| No SDK verification       | ❌ Yes    | All CLI tests (9 tests)                                        | CRITICAL |

**Legend**:

- ✅ No: Pattern not found (good)
- ❌ Yes: Pattern found (needs fixing)
- ⚠️ Partial: Pattern partially present or low-risk variant

---

## Recommendations

### Immediate Actions (Before Release)

1. **Fix Critical Issue #1** (Est: 10 minutes)
   - File: `sdk/tests/structured_prompts_integration_test.rs`
   - Lines: 138, 193, 246, 303
   - Action: Replace `#[ignore]` with `#[cfg_attr(not(feature = "integration-tests"), ignore)]`

2. **Fix Critical Issue #2** (Est: 20 minutes)
   - File: `.github/workflows/ci.yml`
   - Action: Add SDK integration test job or extend existing job to include `-p langstar-sdk`
   - Verify: Re-run CI and confirm 4 SDK integration tests execute

3. **Fix Critical Issue #3** (Est: 2-3 hours)
   - File: `cli/tests/prompt_structured_test.rs`
   - Focus on 3 main tests: `test_cli_structured_prompt_round_trip`, `test_cli_push_structured_prompt`, `test_cli_pull_structured_prompt`
   - Action: Add SDK verification after CLI operations
   - Pattern: Follow `cli/tests/prompt_scoping_test.rs:test_prompt_crud_lifecycle_private_visibility`

4. **Fix Critical Issue #4** (Est: 30 minutes)
   - File: `cli/tests/prompt_structured_test.rs`
   - Action: Remove `check_env_vars()` helper, use `.expect()` with clear messages
   - Verify: Tests fail explicitly when env vars missing

### Verification Steps

After fixes are implemented:

```bash
# 1. Verify SDK integration tests run locally
cargo test --features integration-tests -p langstar-sdk \
  --test structured_prompts_integration_test -- --nocapture

# 2. Verify all integration tests (CLI + SDK)
cargo test --workspace --features integration-tests -- --nocapture

# 3. Check for silent skips (should be 0 when env vars set)
cargo test --workspace --features integration-tests 2>&1 | grep -i "skipping"

# 4. Run pre-commit checklist
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features

# 5. Verify CI passes
# Push to feature branch and check GitHub Actions
```

### Post-Fix Verification

After all fixes are merged:

1. **Re-run audit**: `/gh-milestones:test-audit ls-prompt-structured-outputs`
2. **Confirm**: 0 critical issues found
3. **Verify**: All integration tests pass in CI
4. **Document**: Update milestone with test improvements made

---

## Toyota Andon Cord Status

**Violations Found**: YES

The following violations of the Toyota Andon Cord principle were identified:

1. ❌ **Tests exist but don't run in CI** (SDK integration tests)
   - False confidence from "all tests passing" when 4 tests don't execute

2. ❌ **Tests that silently skip** (CLI integration tests)
   - False confidence from passing tests that didn't actually run

3. ⚠️ **Tests that don't verify actual API state** (CLI integration tests)
   - Can't catch bugs like issue #536 where CLI succeeds but API state is wrong

**Andon Cord Pull**: These violations MUST be fixed before marking milestone as released.

**Principle**: Any failing test stops the merge process. Tests that don't run or don't verify actual behavior are equivalent to missing tests.

---

## Next Steps

See companion document: `ls-prompt-structured-outputs-test-plan.md`

The test plan breaks down remediation into logical sub-tickets:

1. Fix SDK integration test annotations (Issue #1)
2. Add SDK integration tests to CI (Issue #2)
3. Add CRUD lifecycle verification to CLI tests (Issue #3)
4. Remove silent skip pattern (Issue #4)
5. Add comprehensive SDK→CLI→SDK round-trip test (Warning #1)
6. Final verification and release

---

## References

- **Parent Ticket**: #660 (Final testing verification)
- **Audit Ticket**: #661 (This document)
- **Milestone**: #402 (ls-prompt-structured-outputs)
- **Testing Standards**: `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md`
- **CRUD Pattern**: `docs/dev/testing/crud-lifecycle-pattern.md`
- **Issue #536 Post-Mortem**: `docs/dev/testing/post-mortems/536-prompt-list-testing-gap.md`
- **Issue #647**: Silent skip anti-pattern resolution
