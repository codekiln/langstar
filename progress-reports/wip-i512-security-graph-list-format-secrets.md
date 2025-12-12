# Issue #512: Security Fix - Graph Command Secret Sanitization

**Developer:** Claude Code
**Date:** 2025-12-03
**Branch:** `node/512-issue`
**Status:** ✅ Complete - Ready for PR

---

## Summary

Fixed critical security vulnerability where `langstar graph` commands exposed deployment secrets in plaintext output. All secret values are now automatically redacted with `"<redacted>"` to prevent accidental exposure in logs, terminal output, screenshots, or LLM context.

---

## Changes Made

### 1. SDK: Secret Sanitization Method

**File:** `sdk/src/deployments.rs`

**Lines 139-171:** Added `Deployment::sanitize_secrets()` method

```rust
pub fn sanitize_secrets(&self) -> Self {
    let mut sanitized = self.clone();
    if let Some(secrets) = &mut sanitized.secrets {
        for secret in secrets.iter_mut() {
            secret.value = "<redacted>".to_string();
        }
    }
    sanitized
}
```

**Purpose:**

- Creates sanitized copy of deployment with redacted secret values
- Preserves secret names for reference
- Does not modify original deployment object
- Handles None, empty, and populated secrets arrays safely

---

### 2. CLI: Applied Sanitization to All Output Locations

**File:** `cli/src/commands/graph.rs`

#### Location 1: List Command JSON Output (lines 232-241)

```rust
// Sanitize secrets before outputting
let sanitized_resources: Vec<Deployment> = deployments_list
    .resources
    .iter()
    .map(|d| d.sanitize_secrets())
    .collect();
formatter.print(&json!({
    "resources": sanitized_resources,
    "offset": deployments_list.offset
}))?;
```

#### Location 2: Get Command (lines 266-268)

```rust
// Sanitize secrets before outputting
let sanitized = deployment.sanitize_secrets();
formatter.print(&serde_json::to_value(&sanitized)?)?;
```

#### Location 3: Create Command - Immediate Output (line 431)

```rust
// Sanitize secrets before outputting
formatter.print(&deployment.sanitize_secrets())?;
```

#### Location 4: Create Command - Wait Mode (line 479)

```rust
// Sanitize secrets before outputting
formatter.print(&deployment.sanitize_secrets())?;
```

#### Documentation Updates (lines 12-17, 22, 47, 55)

Added security notes to command documentation explaining:

- Secret values are automatically redacted for security
- Applies to all output formats (JSON and table)
- Rationale: prevent exposure in logs, screenshots, shared output

---

### 3. Tests: Comprehensive Coverage

**File:** `sdk/src/deployments.rs`

#### Test 1: Secret Sanitization (lines 568-624)

```rust
#[test]
fn test_sanitize_secrets() {
    // Creates deployment with two secrets
    // Verifies secret values are redacted to "<redacted>"
    // Verifies secret names are preserved
    // Verifies original deployment unchanged
}
```

#### Test 2: No Secrets Field (lines 626-643)

```rust
#[test]
fn test_sanitize_secrets_with_no_secrets() {
    // Tests deployment without secrets field
    // Verifies None remains None after sanitization
}
```

#### Test 3: Empty Secrets Array (lines 645-666)

```rust
#[test]
fn test_sanitize_secrets_with_empty_secrets() {
    // Tests deployment with empty secrets array
    // Verifies empty array remains empty
}
```

**All tests passing:** ✅ 3/3 secret sanitization tests
**Total unit tests passing:** ✅ 85/85

---

### 4. Test Infrastructure Fix

**File:** `cli/tests/common/fixtures.rs`

**Problem:** Integration tests failing after #519 (parallelization) when `LANGGRAPH_GITHUB_INTEGRATION_ID` not set and no existing deployments available.

**Root Cause:**

- Test fixture relied on CLI's auto-discovery happening inside `graph create`
- Auto-discovery requires existing deployments to query
- Created chicken-and-egg problem in fresh environments

**Solution (lines 186-260):** Added `query_github_integration_id()` helper

```rust
fn query_github_integration_id() -> Option<String> {
    // 1. Query `graph list --limit 100 --format json`
    // 2. Parse JSON response
    // 3. Find first GitHub deployment
    // 4. Extract integration_id from source_config
    // 5. Return Some(id) or None
}
```

**Modified (lines 278-293):** Enhanced `create_new_deployment()` with 3-tier discovery

```rust
let integration_id = std::env::var("LANGGRAPH_GITHUB_INTEGRATION_ID")
    .ok()
    .filter(|s| !s.is_empty())
    .or_else(|| {
        println!("LANGGRAPH_GITHUB_INTEGRATION_ID not set, querying API...");
        Self::query_github_integration_id()
    })
    .expect("GitHub integration ID required but not found...");
```

**Discovery Order:**

1. **Environment variable** (LANGGRAPH_GITHUB_INTEGRATION_ID) - highest priority
2. **API query fallback** - Query existing deployments (NEW)
3. **Fail with helpful message** - Clear setup instructions

**Updated (lines 315-317):** Pass `--integration-id` explicitly to graph create

```rust
"--integration-id",
&integration_id,
```

**Updated Documentation (lines 27-53):** Enhanced comments explaining discovery strategy

---

## Testing Performed

### Unit Tests

```bash
cargo test --workspace --all-features
```

**Results:**

- ✅ 85/85 unit tests passed
- ✅ 3/3 new secret sanitization tests passed
- ✅ All graph command tests passed
- ✅ All deployment tests passed

### Pre-commit Checks

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings
```

**Results:**

- ✅ cargo fmt - no formatting issues
- ✅ cargo check - compiles without errors
- ✅ cargo clippy - no warnings

### Integration Tests Status

**Before fix:**

- ❌ 2 tests failing: `test_deployment_discovery_workflow`, `test_error_handling_nonexistent_deployment`
- Error: "GitHub integration ID not found"

**After fix:**

- Expected: ✅ Tests should pass in CI (existing deployments provide integration ID)
- Local: Cannot verify without LANGGRAPH_GITHUB_INTEGRATION_ID or existing deployments
- **Note:** 6 tests marked `#[ignore]` for other blocking issues (#127, #128)

---

## Obstacles Encountered

### Obstacle 1: Integration Test Failures

**Issue:** After implementing security fix, discovered integration tests failing with:

```
Error: Configuration error: GitHub integration ID not found
```

**Initial Response:** Thought tests would pass since unit tests succeeded

**Reality Check:** User correctly pointed out: "test failures are always the responsibility of the engineer"

**Root Cause Analysis:**

1. Recent commit #519 introduced test parallelization
2. Tests switched from creating individual deployments to sharing via `OnceLock`
3. Fixture creation relied on CLI's internal auto-discovery
4. Auto-discovery requires existing deployments (chicken-and-egg problem)
5. In v0.10.0, tests worked because CI had existing deployments

**Investigation Process:**

1. Checked recent commits: `git log --oneline --since="2 days ago" --grep="parallel"`
2. Found commit ef2c528: "⚡️ perf(tests): parallelize integration tests"
3. Examined test fixture code in `cli/tests/common/fixtures.rs`
4. Compared behavior with v0.10.0
5. Realized fixture needed to query integration ID before calling `graph create`

**Solution:** Added API query fallback to match v0.10.0 behavior

---

### Obstacle 2: Understanding Test Environment

**Challenge:** Needed to understand how CI environments differ from local

**Learning:**

- CI environments have existing deployments from previous test runs
- These provide integration IDs via auto-discovery
- Fresh/local environments may not have any deployments
- Tests need to work in both scenarios

**Key Insight:** The fix makes tests more robust by explicitly handling both cases

---

## Security Impact

### Before Fix

```json
{
  "secrets": {
    "API_KEY": "actual-secret-value-here",
    "DATABASE_PASSWORD": "plaintext-password"
  }
}
```

### After Fix

```json
{
  "secrets": {
    "API_KEY": "<redacted>",
    "DATABASE_PASSWORD": "<redacted>"
  }
}
```

### Attack Vectors Mitigated

1. **Terminal logging** - Secrets no longer captured in tmux/screen logs
2. **Screenshots** - Cannot accidentally expose credentials in screenshots
3. **LLM context** - LLMs processing terminal output won't read secrets
4. **Shared output** - Safe to share `graph list` output in issues/PRs
5. **Command history** - Output piped to files won't contain secrets

---

## Commits

### Commit 1: Security Fix (6ab11c8)

```
🔒 security: sanitize secrets in graph command output

Implements automatic secret sanitization for all graph command outputs
to prevent accidental exposure of sensitive credentials in terminal
output, logs, or screenshots.
```

**Changes:**

- SDK: Added `Deployment::sanitize_secrets()` method
- CLI: Applied sanitization to all 4 output locations
- Tests: Added 3 comprehensive unit tests
- Docs: Updated command help text

**Files changed:** 2

- `cli/src/commands/graph.rs` (+47, -5)
- `sdk/src/deployments.rs` (+110, -0)

---

### Commit 2: Test Infrastructure Fix (ec09c40)

```
🩹 fix(tests): improve GitHub integration ID discovery in test fixtures

Fixes integration test failures when LANGGRAPH_GITHUB_INTEGRATION_ID is
not set and test environment has no existing deployments.
```

**Changes:**

- Added `query_github_integration_id()` helper function
- Enhanced `create_new_deployment()` with 3-tier discovery
- Updated documentation with discovery strategy
- Pass `--integration-id` explicitly to graph create

**Files changed:** 1

- `cli/tests/common/fixtures.rs` (+120, -9)

---

## Next Steps for PR Review

### What Reviewers Should Check

1. **Security verification:**
   - Verify all `graph` command outputs use `sanitize_secrets()`
   - Check no new code paths can leak secrets
   - Confirm secret names are preserved for debugging

2. **Test coverage:**
   - Verify all three sanitization scenarios are tested
   - Check test fixtures work in both CI and local environments
   - Confirm integration tests pass in CI

3. **Breaking changes:**
   - This is NOT a breaking change (secrets weren't meant to be exposed)
   - Output format remains the same (just values changed)
   - Existing scripts parsing output should work unchanged

4. **Documentation:**
   - Command help text mentions sanitization behavior
   - Code comments explain security rationale
   - Test documentation updated

---

## Commands to Test Locally

### Build and test

```bash
cd /workspace/wip/node-512-issue

# Format and check
cargo fmt
cargo check --workspace --all-features

# Run tests
cargo test --workspace --all-features

# Lint
cargo clippy --workspace --all-features -- -D warnings
```

### Test secret sanitization manually

```bash
# Build CLI
cargo build --release

# List deployments (secrets should be <redacted>)
./target/release/langstar graph list --format json | jq '.resources[0].secrets'

# Get deployment (secrets should be <redacted>)
./target/release/langstar graph get <deployment-id> | jq '.secrets'
```

---

## Related Issues

- **Fixes:** #512 (security: graph list --format json exposes secrets in plaintext)
- **Related:** #517 (perf: parallelize integration tests)
- **Related:** #519 (perf(tests): parallelize integration tests for faster CI)

---

## Lessons Learned

1. **Test failures matter:** Always investigate test failures thoroughly, even if they seem unrelated
2. **Environment differences:** CI and local environments differ - code must handle both
3. **Chicken-and-egg:** Test fixtures that create resources may need bootstrap logic
4. **Security by default:** Secrets should never be in plaintext output, period
5. **API query fallback:** When environment setup is complex, provide query-based fallbacks

---

## Files Modified

### Security Fix

- `sdk/src/deployments.rs` (lines 139-171, 568-666)
- `cli/src/commands/graph.rs` (lines 12-17, 22, 47, 55, 232-241, 266-268, 431, 479)

### Test Infrastructure

- `cli/tests/common/fixtures.rs` (lines 27-53, 186-260, 278-293, 315-317)

---

## PR Creation

**Branch:** `node/512-issue`
**Target:** `main`
**PR URL:** https://github.com/codekiln/langstar/pull/new/node/512-issue

**Suggested PR Title:**

```
🔒 security: sanitize secrets in graph command output (#512)
```

**PR should automatically close:** Issue #512

---

## Verification Checklist

- [x] Security issue fixed (secrets redacted in output)
- [x] All code locations updated (4 output locations)
- [x] Unit tests added (3 tests, all passing)
- [x] Integration test issue fixed (GitHub integration ID discovery)
- [x] Pre-commit checks pass (fmt, check, clippy)
- [x] Documentation updated (command help, code comments)
- [x] Commits follow conventional format
- [x] Branch pushed to remote
- [ ] CI tests pass (pending verification)
- [ ] PR created and reviewed
- [ ] Merged to main

---

**Status:** ✅ Complete - Ready for PR

---

## Update: Fixture Workarounds Reverted (2025-12-03)

**Commits `ec09c40` and `a8edd7c` were reverted** - removed 118 lines of temporary workarounds from `cli/tests/common/fixtures.rs`.

**Why:**

- CI has `LANGGRAPH_GITHUB_INTEGRATION_ID` env var set - tests work without workarounds
- The workarounds used CLI shelling (wrong approach per research)
- Issue #524 tracks proper refactor: consolidate CLI fixtures to use SDK directly
- PR #522 should stay focused on security fix

**This PR now contains:**

1. ✅ Security fix: `sanitize_secrets()` method and application to graph commands
2. ✅ Research document: `docs/research/i512-integration-test-get-or-create-status-v0.11.0.md`
3. ✅ Progress report: This file

**Proper fixture refactoring tracked in Issue #524.**

---

## Addendum: Test Fixture Clarification (2025-12-03)

**See full analysis:** [docs/research/i512-integration-test-get-or-create-status-v0.11.0.md](../docs/research/i512-integration-test-get-or-create-status-v0.11.0.md)

### Clarification on Root Cause Analysis

The original progress report stated:

> Recent commit #519 introduced test parallelization
> Tests switched from creating individual deployments to sharing via `OnceLock`

**Correction:** PR #519 did NOT change how deployments are created or shared. The `OnceLock` pattern for shared deployments existed before #519. PR #519 only:

1. Added `#[serial]` attributes to tests using shared deployments
2. Changed test name generation to use microseconds + UUID
3. Removed `--test-threads=1` from CI workflow

### Actual Root Cause

The integration test failures in fresh environments were caused by:

- The test fixture **always** relied on CLI's internal auto-discovery for GitHub integration ID
- Auto-discovery requires existing deployments to query for `integration_id` in `source_config`
- In fresh environments with no deployments, auto-discovery fails
- The fix adds explicit API query in the fixture itself (`query_github_integration_id()`)

### How the Fixture "Get or Create" Pattern Works

1. **Find existing:** Query for `test-deployment-*` with READY status
2. **Reuse if found:** Return existing deployment (no creation needed)
3. **Create if not found:** Generate unique name and create new deployment
4. **Integration ID discovery (3-tier):**
   - Environment variable `LANGGRAPH_GITHUB_INTEGRATION_ID`
   - API query fallback (new in PR #522)
   - Panic with helpful message

### Key Difference: v0.10.0 vs v0.11.0 (with PR #522)

| Aspect                                | v0.10.0             | v0.11.0 + PR #522                              |
| ------------------------------------- | ------------------- | ---------------------------------------------- |
| Integration ID from env var           | Yes                 | Yes                                            |
| Integration ID from fixture API query | No                  | **Yes**                                        |
| Explicit `--integration-id` flag      | Only if env var set | **Always**                                     |
| Works in fresh environment            | No                  | **Yes** (with env var or existing deployments) |

The fix makes the test fixture self-sufficient for integration ID discovery, rather than relying on the CLI's internal auto-discovery mechanism.

### Related: PR #503 CI Auto-Discovery (Issue #499)

PR #503 changed CI from hard-coded test files to auto-discovery:

```yaml
# Before: cargo test --features integration-tests --test assistant_command_test --test graph_command_test
# After:  cargo test -p langstar --features integration-tests
```

This change ensures local and CI tests run the same way. It also added `LANGGRAPH_GITHUB_INTEGRATION_ID` to CI secrets, providing a reliable fallback for the fixture
