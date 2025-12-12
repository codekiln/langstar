<!--
  SIZE LIMIT: This file SHOULD remain under 200 lines (currently ~220).
  Last checked: 2025-12-06
  If significantly exceeding limit, extract content to sub-document.
-->

# High-Level Testing Guidelines

This document establishes core testing principles for langstar. Load this document when designing any tests or reviewing PRs.

## Toyota Andon Cord Principle

In Toyota manufacturing, any worker can pull the "andon cord" to stop the entire production line when detecting a defect. This prevents defects from propagating downstream.

**For Langstar:** Any failing test stops the merge process. Period.

### Never Acceptable

- ❌ "My changes didn't introduce this failure"
- ❌ "It's a flaky test, we can ignore it"
- ❌ "The test is wrong, not the code"
- ❌ "We'll fix it in a follow-up PR"
- ❌ "The test passes on my machine"
- ❌ "This failure is unrelated to my changes"
- ❌ "The test should only run in CI where the proper environment is configured"
- ❌ "This is a pre-existing failure" (without CI proof)
- ❌ "The CLI test timeout is a separate issue (not introduced by this PR)" (without verifiable proof)

### Verifying "Pre-Existing" Claims

**Never claim a failure is "pre-existing" without objective proof.** Before making this claim:

1. Check CI status on main: https://github.com/codekiln/langstar/commits/main/
2. All checks must show ❌ for that specific test on main branch
3. Link to the failing CI run as evidence

If CI is green on main, the failure is **your responsibility** to fix, not a pre-existing issue.

### Always Required

- ✅ Fix the failure before merge
- ✅ If test is wrong, fix test then verify code
- ✅ If failure is in unrelated code, fix it or revert the commit that broke it
- ✅ If test is flaky, fix the flakiness first
- ✅ All CI checks must be green before merge
- ✅ All integration tests need to run locally the same as they run in CI

### Why This Matters

**Example from #536:** `langstar prompt list` returned zero results for private prompts. Tests only checked exit codes, not actual behavior. The bug shipped despite "passing" tests.

**Prevention:** Never merge without verifying tests actually exercise the feature.

## Pre-Merge Testing Requirements

### Local Pre-Commit Checklist

**ALWAYS run before committing:**

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo nextest run --profile ci --all-features --workspace && \
cargo fmt --check
```

**Why each step matters:**

| Command             | Purpose                  | What it catches                 |
| ------------------- | ------------------------ | ------------------------------- |
| `cargo fmt`         | Auto-format code         | Style inconsistencies           |
| `cargo check`       | Type checking            | Compile errors across workspace |
| `cargo clippy`      | Lint analysis            | Common mistakes, code smells    |
| `cargo nextest run` | Run all tests (parallel) | Regressions, broken features    |
| `cargo fmt --check` | Verify formatting        | Uncommitted format changes      |

**Time investment:** ~1-2 minutes locally vs 10-20 minutes CI roundtrips

**Note:** We use `cargo-nextest` instead of `cargo test` for faster parallel execution and better output. CI and local testing use the same tool for consistency. The devcontainer automatically installs `cargo-nextest` via `post-create.sh`. If not using the devcontainer, install it with `cargo install cargo-nextest --locked`.

### Integration Test Requirements

When a PR adds or modifies features that interact with APIs:

- [ ] Integration tests exist and pass
- [ ] Tests verify actual behavior (not just exit codes)
- [ ] Tests use CRUD lifecycle pattern (see `crud-lifecycle-pattern.md`)
- [ ] Test data is cleaned up properly
- [ ] Tests use explicit failures (`.expect()`) for missing env vars

#### Required Environment Variables

**Core required env vars for integration tests:**

- `LANGSMITH_API_KEY` - API key for LangSmith/LangGraph services
- `LANGSMITH_ORGANIZATION_ID` - Organization ID for scoped operations
- `LANGSMITH_WORKSPACE_ID` - Workspace ID for scoped operations

**See `docs/dev/environment-variables.md` for complete mapping of environment variables to API headers and which APIs require which variables.**

**CRITICAL - No Silent Skips (Issue #647):**
Tests MUST fail explicitly when required env vars are missing:

```rust
// ❌ WRONG - Silent skip pattern
fn get_org_id_or_skip() -> Option<String> {
    std::env::var("LANGSMITH_ORGANIZATION_ID").ok()
}
let Some(org_id) = get_org_id_or_skip() else {
    println!("Skipping test...");
    return;
};

// ✅ CORRECT - Explicit failure
let org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
    .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
```

**Why:** Silent skips create false confidence - "289 tests passing" should mean 289 tests actually tested something, not that some silently skipped.

### CI/CD Requirements

- [ ] All CI jobs pass (check, test, clippy, build)
- [ ] No warnings in clippy output (`-D warnings` flag)
- [ ] Code coverage doesn't decrease
- [ ] Integration tests pass in CI environment

## Test Types and When to Use Them

### Unit Tests (Fast, Isolated)

**When to use:**

- Testing pure functions (no I/O)
- Testing data transformations and serialization
- Testing type conversions
- Testing validation logic
- Testing error handling paths

**Location:** In-module `#[cfg(test)]` blocks

**Example pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_schema_rejects_invalid_type() {
        let invalid = json!({"type": "invalid_type"});
        let result = validate_json_schema(&invalid);
        assert!(result.is_err());
    }
}
```

**Reference:** `sdk/src/prompts.rs` tests module (search for `mod tests`)

### Integration Tests (Real API Calls)

**When to use:**

- Testing end-to-end workflows
- Verifying API integration actually works
- Testing CRUD operations
- Validating CLI commands produce expected results

**Location:** `sdk/tests/*_test.rs` or `cli/tests/*_command_test.rs`

**Mark with:** `#[cfg_attr(not(feature = "integration-tests"), ignore)]`

**Example pattern:**

```rust
#[test]
fn test_prompt_crud_lifecycle() {
    // 1. CREATE - Create resource via SDK
    // 2. VERIFY - Confirm via SDK read
    // 3. READ - Test CLI command
    // 4. VERIFY - Parse output, confirm matches
    // 5. DELETE - Clean up
    // See crud-lifecycle-pattern.md for full pattern
}
```

**Reference:** `cli/tests/prompt_scoping_test.rs` function `test_prompt_crud_lifecycle_private_visibility`

### When Both Are Needed

For most features, you need BOTH:

1. **Unit tests** to verify logic in isolation (fast feedback)
2. **Integration tests** to verify actual API behavior (confidence)

**Example:** For `langstar prompt list`:

- Unit tests: Verify URL building, pagination logic, filtering logic
- Integration tests: Create prompt via SDK, verify it appears in list results

## Test Design Anti-Patterns

### Anti-Pattern: Exit Code Only Tests

**BAD** - Only checks that command doesn't crash:

```rust
// ❌ INSUFFICIENT - This is the bug from #536!
cmd.assert().success();  // Only checks exit code 0
```

**GOOD** - Verifies actual behavior:

```rust
// ✅ PROPER - Verifies expected output
let output = cmd.output()?;
assert!(output.status.success());
let prompts: Vec<Prompt> = serde_json::from_slice(&output.stdout)?;
assert!(!prompts.is_empty(), "Expected prompts in output");
assert!(prompts.iter().any(|p| p.repo_handle == expected_handle),
    "Expected prompt not found in output");
```

### Anti-Pattern: No Cleanup

**BAD** - Leaves test data behind:

```rust
// ❌ Creates data but never cleans up
client.prompts().create_repo("test-prompt", ...).await?;
// Test ends without deletion
```

**GOOD** - Uses RAII or explicit cleanup:

```rust
// ✅ Always clean up, even on failure
let prompt = client.prompts().create_repo("test-prompt", ...).await?;
// ... run tests ...
client.prompts().delete(&prompt.repo_handle).await?;
```

**Reference:** `cli/tests/common/fixtures.rs` struct `TestDeployment`

## Test Design Review Checklist

Before marking a test as complete, verify:

- [ ] Does the test verify actual behavior, not just exit codes?
- [ ] Does the test use SDK to verify CLI actions persisted correctly?
- [ ] Does the test clean up created resources?
- [ ] Does the test cover error cases?
- [ ] Would this test catch the bug if implementation was wrong?
- [ ] Is the test deterministic (no race conditions, no flaky behavior)?
- [ ] Are test names descriptive of what they verify?

## Using /gh-milestones:test-plan

Before implementing tests for a milestone, generate a comprehensive test plan:

```bash
/gh-milestones:test-plan <milestone-name-or-number>
```

**What it does:**

- Analyzes milestone type (SDK/CLI/Infrastructure/Docs)
- Loads relevant testing docs (progressive disclosure, <5000 tokens)
- Generates test plan at `docs/implementation/<milestone>-test-plan.md`
- Ensures compliance with these guidelines

**Example:** `/gh-milestones:test-plan ls-runs-query`

**When to use:** At the start of any milestone before writing tests

## Related Documentation

- **CRUD Lifecycle Pattern:** `crud-lifecycle-pattern.md` - CLI→SDK verification pattern
- **Mocking Patterns:** `mocking-patterns.md` - When and how to mock
- **Debugging Tests:** `debugging-tests.md` - Common failures and troubleshooting
- **CLI Integration Tests:** `cli-integration-tests.md` - CLI-specific patterns
- **SDK Integration Tests:** `sdk-integration-tests.md` - SDK-specific patterns
