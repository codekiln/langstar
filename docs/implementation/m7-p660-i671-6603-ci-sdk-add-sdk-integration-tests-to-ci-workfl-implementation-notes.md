# Implementation Notes: m7-p660-i671-6603-ci-sdk-add-sdk-integration-tests-to-ci-workfl

**Branch:** m7-p660-i671-6603-ci-sdk-add-sdk-integration-tests-to-ci-workfl
**PR:** #680
**Issue:** #671
**Date:** 2025-12-11

## Current Status

**Blocking:** 5 test failures prevent merge per Toyota Andon Cord principle.

### Test Failures
- SDK integration tests: 4 failures out of 277 tests
- CLI integration tests: 1 failure out of 299 tests

**Failure links:**
- SDK: https://github.com/codekiln/langstar/runs/57780043067
- CLI: https://github.com/codekiln/langstar/runs/57780117362

### Commits Made

1. **94d6fb4** - Address Copilot review feedback and standardize on cargo-nextest
   - Fixed all 4 Copilot review comments in CI workflow
   - Updated testing documentation to use cargo-nextest
   - Replied to all review comments in-thread

2. **feb54ab** - Enable test-utils feature for SDK integration tests
   - Fixed compilation error: SDK tests require both `integration-tests` and `test-utils` features
   - Tests now compile successfully

### Work Completed

✅ All Copilot review comments addressed:
- Fixed condition reference (needs.changes.outputs.code → needs.changes.outputs.should_run)
- Added conditional logic: SDK tests only run on PRs or main branch
- Added test job dependency to SDK integration tests (needs: [changes, test])
- Added LANGSMITH_WORKSPACE_ID environment variable to SDK tests

✅ cargo-nextest standardization:
- Installed cargo-nextest v0.9.114 locally
- Updated CLAUDE.md pre-commit checklist
- Updated docs/dev/README.md pre-commit checklist
- Updated docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md

✅ SDK integration test compilation fixed

### Root Cause Analysis (Session 2 - 2025-12-11)

**Key Findings:**

1. **Tests were never run before** - PR #678 (665d66c) removed `#[ignore]` attributes, but tests weren't added to CI until this PR
2. **Tests fail both locally AND in CI** - Not a CI-specific issue
3. **Main branch CI passed** - Failures are NOT pre-existing (last success: 665d66c on Dec 10)

**Test Failure Details:**

SDK Tests (4 failures in `structured_prompts_integration_test.rs`):
- Error: `404 {"error":"Repository not found"}`
- Tests: `test_push_structured_prompt_integration`, `test_pull_structured_prompt_integration`, `test_structured_prompt_round_trip_integration`, `test_push_function_calling_method`
- Original issue: Tests used public repo `codekiln/langstar-structured-test` without proper CRUD ordering

CLI Test (1 failure in `prompt_scoping_test.rs`):
- Error: `TimedOut` (30s timeout creating repo via SDK)
- Test: `test_prompt_search_crud_lifecycle`

**Attempted Fix #1: Convert to private prompts with serial execution**

Changes made to `sdk/tests/structured_prompts_integration_test.rs`:
- Changed `TEST_OWNER` from `"codekiln"` to `"-"` (private prompts)
- Removed repo creation logic (lines 149-172)
- Added `#[serial_test::serial]` to all 4 tests for CRUD ordering
- Updated docs to explain private prompt usage

Result: **Still fails with 404 "Repository not found"** when pushing to `-/langstar-structured-test`

### Outstanding Work

**MUST FIX BEFORE MERGE:**

1. ✅ Determine if failures are pre-existing → NOT pre-existing
2. ✅ Identify root cause → Tests use wrong repo creation pattern
3. ⚠️ Fix all failures → **IN PROGRESS**

**Current Blocker:**
Tests fail with 404 when pushing to `-/langstar-structured-test`. Need to determine:
- Do private prompt repos need to be created first? (see `/workspace/reference/experiments/398-structured-output-prompts/test_structured_prompts.py:225-238`)
- Or is there a different API pattern for private prompts?

**Research needed:**
- Review `/workspace/reference/experiments/398-structured-output-prompts/test_structured_prompts.py`
- Check `/workspace/reference/repo/langchain-ai/langsmith-mcp-server/code/langsmith_mcp_server/services/tools/prompts.py`
- Review `/workspace/docs/research/398-structured-output-prompts-scout.md`
- Understand: Python experiment DOES create private repos first (POST /repos/ with `is_public: False`)

**Files modified in this PR:**
- .github/workflows/ci.yml (CI config)
- CLAUDE.md (pre-commit checklist)
- docs/dev/README.md (pre-commit checklist)
- docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md (cargo-nextest)
- sdk/tests/structured_prompts_integration_test.rs (ATTEMPT 1: private prompts + serial)

**Next Steps:**
1. Investigate if SDK `create_repo()` works for private prompts with owner="-"
2. Possibly restore repo creation logic but use private pattern
3. Consider if tests need different test data (existing private prompt repo)
4. Run full local test suite to verify CLI test timeout issue
