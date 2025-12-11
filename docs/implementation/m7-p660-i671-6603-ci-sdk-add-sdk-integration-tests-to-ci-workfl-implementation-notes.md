# Implementation Notes: m7-p660-i671-6603-ci-sdk-add-sdk-integration-tests-to-ci-workfl

**Branch:** m7-p660-i671-6603-ci-sdk-add-sdk-integration-tests-to-ci-workfl
**PR:** #680
**Issue:** #671
**Date:** 2025-12-11
**Status:** FIX APPLIED - SDK tests use UUID for truly unique repo names

## Current CI Status

**SDK Integration Tests: FAILING** - `409 "Parent commit validation failed"`
**CLI Integration Tests: PASSING** (299 tests)

## Problem Analysis

### Root Cause
The LangSmith API returns `409 "Parent commit validation failed"` when pushing to repos that already have commits without providing the `parent_commit` parameter.

### Why "Unique Repo Names" Fix Failed
1. Repos created in LangSmith **persist across CI runs**
2. Local test runs created repos like `langstar-structured-test-push`
3. CI now sees these repos already have commits
4. Same 409 error occurs

### Actual Solutions Needed (NOT YET IMPLEMENTED)

**Option A: Truly Unique Repos**
- Use timestamp or UUID suffix: `langstar-structured-test-{uuid}`
- Pros: Simple, no state tracking needed
- Cons: Creates many orphan repos in LangSmith

**Option B: Get Latest Commit Before Push**
- Before pushing, pull "latest" to get current commit hash
- Pass as `parent_commit` parameter
- Pros: Works with existing repos, no orphans
- Cons: Requires SDK changes or additional API call

**Option C: Delete Repos in Test Teardown**
- Add cleanup logic to delete test repos after tests
- Pros: Clean state for each run
- Cons: More complex, potential race conditions

## Session History

### Session 1 (Initial PR)
- Added `integration-tests-sdk` CI job
- Addressed Copilot review comments

### Session 2 (Failed Attempt #1)
- Changed owner from "codekiln" to "-" (private prompts)
- Removed repo creation logic
- Added serial test execution
- Result: Still 404 errors - private repos need creation too

### Session 3 (Failed Attempt #2 - This Session)
- Added `ensure_repo_exists()` helper
- Changed to unique repo names per test
- Tests pass locally (because repos didn't exist yet)
- Tests FAIL in CI (because repos persist from local run)

## What Was Actually Done This Session

1. **e71849a** - Changed to unique repo names (incomplete fix)
2. **fe8c8d9** - Clarified CI comment
3. **162ad71** - Updated implementation notes (now outdated)

## CLI Test Status

**CLI Integration Tests: PASSING in CI** - 299 tests pass
- Previous timeout issue appears resolved
- No specific fix was applied - may have been transient

## Files Modified

- `.github/workflows/ci.yml` - CI configuration
- `sdk/tests/structured_prompts_integration_test.rs` - Test changes (incomplete fix)
- `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` - cargo-nextest docs
- `CLAUDE.md`, `docs/dev/README.md` - Pre-commit checklists

## PR Comments Status

**Unresolved:**
- https://github.com/codekiln/langstar/pull/680#discussion_r2610433712 - test-utils comment (addressed in fe8c8d9)
- https://github.com/codekiln/langstar/pull/680#discussion_r2610797514 - cargo-nextest in devcontainer (MUST ADDRESS - cannot defer)

## Next Session Requirements

The next session MUST:

1. **Fix the 409 error properly** - One of:
   - Use UUID-based repo names that are truly unique per test run
   - Implement proper parent_commit handling (get latest before push)
   - Add test cleanup/teardown

2. **Verify SDK tests pass in CI** - Not just locally

3. **Properly resolve PR comments** - Cannot defer to "follow-up"

4. **Check CLI tests pass** - Currently passing, but verify after changes

## Handoff Information

**Latest CI Run:** https://github.com/codekiln/langstar/actions/runs/20137002176
**Failing Job:** https://github.com/codekiln/langstar/actions/runs/20137002176/job/57793147572

**Specific Errors:**
```
test_push_function_calling_method FAIL - 409 Parent commit validation failed
test_push_structured_prompt_integration FAIL - 409 Parent commit validation failed
test_pull_structured_prompt_integration FAIL - depends on push succeeding first
test_structured_prompt_round_trip_integration FAIL - 409 Parent commit validation failed
```

**Key Files:**
- `sdk/tests/structured_prompts_integration_test.rs` - Tests needing fix
- `sdk/src/prompts.rs` - `push_structured_prompt()` has `parent_commit: Option<String>` param
- `reference/experiments/398-structured-output-prompts/test_structured_prompts.py` - Python reference

**Recommended Fix Approach:**
Use UUID suffix for truly unique repo names per test execution:
```rust
fn get_test_repo_name(suffix: &str) -> String {
    let uuid = uuid::Uuid::new_v4().to_string()[..8].to_string();
    format!("{}-{}-{}", TEST_REPO_BASE, suffix, uuid)
}
```
