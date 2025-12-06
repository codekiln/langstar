# Post-Mortem: Issue #536 - Prompt List Testing Gap

## Executive Summary

**What happened:** The `langstar prompt list` command returned zero results when querying private prompts in a scoped workspace/organization, despite private prompts existing in the LangSmith API.

**When shipped:** The bug was introduced in commit 9602e3e (default-to-private feature) and shipped in release v0.3.x (exact version TBD).

**How discovered:** User report during manual testing. The bug was not caught by CI/CD integration tests, which passed successfully before and after the feature was merged.

**Impact:** High - Users with organization or workspace IDs configured (the primary production use case) could not list their private prompts. The command succeeded (exit code 0) but returned empty results, creating a confusing user experience.

**Resolution:** Fixed in PR #538 (merged 2025-12-04) by passing the `is_public` query parameter to the LangSmith API instead of doing client-side filtering. Comprehensive CRUD lifecycle tests were added to prevent regression.

**Root cause:** The SDK performed client-side filtering on an incomplete result set from the API, rather than requesting the API to filter server-side. Integration tests only verified command success (exit code 0) without checking actual output content, creating false confidence that the feature worked correctly.

**Key lesson:** Exit-code-only tests are insufficient for integration testing. Tests must verify actual behavior using the CRUD lifecycle pattern (Create → Verify → Read → Verify → Cleanup) to catch bugs where commands succeed but produce incorrect results.

---

## The Bug

### Symptom

`langstar prompt list` returned zero results for private prompts when scoped to an organization or workspace:

```bash
$ langstar prompt list --limit 5
ℹ Scope: Workspace (6f52dd84) | Visibility: private only
ℹ Fetching prompts (limit: 5, offset: 0)...
Found 0 prompts
```

The user had at least 10 private prompts in the workspace (created during integration tests), but none appeared in the results.

### Root Cause

**File:** `sdk/src/prompts.rs:282-320`

The SDK's `list()` method did client-side filtering instead of passing the `is_public` parameter to the API:

```rust
pub async fn list(
    &self,
    limit: Option<u32>,
    offset: Option<u32>,
    visibility: Option<Visibility>,
) -> Result<Vec<Prompt>> {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);
    let visibility = visibility.unwrap_or(Visibility::Any);

    // Line 298: API request MISSING is_public parameter
    let path = format!("/api/v1/repos/?limit={}&offset={}", limit, offset);
    let request = self.client.langsmith_get(&path)?;

    let response: ListReposResponse = self.client.execute(request).await?;

    // Lines 310-318: Client-side filtering (WRONG - filters already-fetched results)
    let filtered = match visibility {
        Visibility::Public => response.repos.into_iter().filter(|p| p.is_public).collect(),
        Visibility::Private => response.repos.into_iter().filter(|p| !p.is_public).collect(),
        Visibility::Any => response.repos,
    };

    Ok(filtered)
}
```

### Why It Broke

1. **API request omitted `is_public` parameter**: The SDK requested `/api/v1/repos/?limit=5&offset=0` without specifying visibility
2. **API returned default subset**: The LangSmith API returned some default subset (likely public prompts or a mixed set)
3. **SDK filtered incomplete result set**: The SDK applied `filter(|p| !p.is_public)` to that subset
4. **No matches in subset**: If the API's default subset contained no private prompts, filtering produced an empty list
5. **Empty result returned**: User saw "Found 0 prompts" despite having private prompts in the API

**The correct approach:** The LangSmith API supports server-side filtering via the `is_public` query parameter (documented in `reference/api-specs/langsmith/prompt-endpoints.json:161-178`). The SDK should have passed this parameter to the API:

```rust
// Correct implementation (from PR #538):
let mut path = format!("/api/v1/repos/?limit={}&offset={}", limit, offset);

match visibility {
    Visibility::Public => path.push_str("&is_public=true"),
    Visibility::Private => path.push_str("&is_public=false"),
    Visibility::Any => {} // Don't add is_public parameter
}

let request = self.client.langsmith_get(&path)?;
let response: ListReposResponse = self.client.execute(request).await?;

// Return all results (API already filtered)
Ok(response.repos)
```

### Scope

The same bug also affected the `search()` method at `sdk/src/prompts.rs:341-379`, which used identical client-side filtering logic.

---

## Why Tests Didn't Catch It

### The Anemic Test

**Test location:** `cli/tests/prompt_scoping_test.rs:42-68`

```rust
#[test]
fn test_prompt_list_with_org_id_from_env() {
    // Requires LANGSMITH_ORGANIZATION_ID to be set
    let org_id = match get_org_id_or_skip() {
        Some(id) => id,
        None => {
            println!("⚠️  Skipping test: LANGSMITH_ORGANIZATION_ID not set");
            println!("   Set this environment variable to run organization-scoped tests");
            return;
        }
    };

    println!(
        "Testing prompt list with org ID from environment: {}",
        org_id
    );

    let mut cmd = langstar_cmd();
    cmd.args(["prompt", "list", "--limit", "5"]);

    // Run the command
    let assert = cmd.assert();

    // Should succeed
    assert.success();  // ← ONLY CHECKS EXIT CODE!

    println!("✓ CLI successfully listed prompts with org ID from environment");
}
```

### What Was Verified

- ✅ Command doesn't crash
- ✅ Exit code is 0
- ✅ Command runs without errors

### What Was NOT Verified

- ❌ Actual prompts are returned in output
- ❌ Correct prompts are returned (private vs public)
- ❌ Output matches expected data structure
- ❌ API receives correct query parameters
- ❌ Output content is parseable and non-empty

### False Confidence

The test passed with a green checkmark (✓), giving the false impression that the feature worked correctly. In reality, the test only verified that the command didn't crash. The bug—returning zero results—was completely invisible to the test suite.

**Worse:** There were 9 similar tests in `prompt_scoping_test.rs` (lines 42-494), all using the same exit-code-only pattern. Every single one passed, reinforcing the false confidence.

### Comments Acknowledged the Gap

Ironically, the test code itself acknowledged this limitation:

```rust
// Lines 191-193
// Note: We can't easily verify the output contains only private prompts
// without parsing JSON output. The unit tests verify the logic.
// This test just confirms the command runs successfully.
```

This comment reveals the developer knew the test was incomplete but chose to proceed anyway, trusting that "unit tests verify the logic." Unfortunately, unit tests cannot verify end-to-end integration with the LangSmith API—only integration tests can do that.

---

## What Should Have Been Done: CRUD Lifecycle Pattern

### The Comprehensive Test

**What we needed** (implemented in PR #538 at `cli/tests/prompt_scoping_test.rs:536-747`):

```rust
#[test]
fn test_prompt_crud_lifecycle_private_visibility() {
    let org_id = match get_org_id_or_skip() {
        Some(id) => id,
        None => {
            println!("Skipping: LANGSMITH_ORGANIZATION_ID not set");
            return;
        }
    };

    let runtime = create_runtime();
    let client = match create_sdk_client() {
        Ok(c) => c,
        Err(e) => {
            println!("Skipping: SDK client error - {}", e);
            return;
        }
    };

    let test_prompt_name = generate_test_prompt_name();

    // ═══════════════════════════════════════════════════════════════════════
    // Step 1: CREATE - Create a private prompt via SDK
    // ═══════════════════════════════════════════════════════════════════════
    let created_prompt = runtime.block_on(async {
        client
            .prompts()
            .create_repo(
                &test_prompt_name,
                Some("Test prompt for CRUD lifecycle - issue #536".to_string()),
                None,
                false, // is_public = false (private)
                None,
            )
            .await
    });

    let prompt = match created_prompt {
        Ok(p) => {
            assert!(!p.is_public, "Prompt should be private");
            p
        }
        Err(e) => {
            panic!("Failed to create test prompt: {}", e);
        }
    };

    // ═══════════════════════════════════════════════════════════════════════
    // Step 2: VERIFY CREATION - Read via SDK to confirm API state
    // ═══════════════════════════════════════════════════════════════════════
    let read_result = runtime.block_on(async {
        client.prompts().get(&prompt.repo_handle).await
    });

    match read_result {
        Ok(p) => {
            assert_eq!(p.repo_handle, prompt.repo_handle);
            assert!(!p.is_public, "Prompt should still be private");
        }
        Err(e) => {
            panic!("Failed to read created prompt: {}", e);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Step 3: READ - Verify prompt appears in CLI list output
    // ═══════════════════════════════════════════════════════════════════════
    let mut list_cmd = langstar_cmd();
    list_cmd.args([
        "prompt",
        "list",
        "--limit",
        "20",
        "--organization-id",
        &org_id,
        "--format",
        "json",
    ]);

    let list_output = list_cmd
        .output()
        .expect("Failed to execute CLI list command");
    assert!(
        list_output.status.success(),
        "CLI list command failed: {}",
        String::from_utf8_lossy(&list_output.stderr)
    );

    // Parse JSON output
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    let json_start = list_stdout.find('[').unwrap_or(0);
    let json_str = &list_stdout[json_start..];

    let cli_prompts: Vec<Value> =
        serde_json::from_str(json_str).expect("Failed to parse CLI JSON output");

    // ═══════════════════════════════════════════════════════════════════════
    // Step 4: VERIFY READ - Confirm our prompt is in the list
    // ═══════════════════════════════════════════════════════════════════════
    let found_in_list = cli_prompts.iter().any(|p| {
        p.get("repo_handle")
            .and_then(|v| v.as_str())
            .map(|h| h.ends_with(&test_prompt_name))
            .unwrap_or(false)
    });

    assert!(
        found_in_list,
        "BUG #536 WOULD BE CAUGHT HERE! \
         Created private prompt '{}' not found in CLI list output. \
         CLI returned {} prompts but our test prompt was not among them.",
        test_prompt_name, cli_prompts.len()
    );

    // ═══════════════════════════════════════════════════════════════════════
    // Step 5: CLEANUP - Delete the test prompt
    // ═══════════════════════════════════════════════════════════════════════
    let _ = runtime.block_on(async {
        client.prompts().delete(&test_prompt_name).await
    });
}
```

### Why This Pattern Works

**CRUD Lifecycle Order:**

1. **CREATE**: Create deterministic test data via SDK
   - Uses SDK to ensure data reaches the API
   - Creates specific test prompt with known handle/name
   - Verifies creation succeeded (prompt object returned)

2. **VERIFY CREATION**: Read via SDK to confirm API state
   - Double-checks the prompt exists in the API
   - Confirms visibility is correct (`is_public = false`)
   - Establishes baseline: "data exists in API"

3. **READ**: Execute the CLI command being tested
   - Runs `langstar prompt list` with JSON output
   - Tests the actual user-facing command
   - Captures stdout for verification

4. **VERIFY READ**: Parse and check output content
   - **This is where bug #536 would be caught!**
   - Parses JSON output from CLI
   - Searches for the test prompt by handle
   - **Fails if prompt not found** (zero results = test failure)

5. **CLEANUP**: Delete test data
   - Removes test prompt from API
   - Prevents test data pollution
   - Ensures tests are isolated and repeatable

**Critical difference from anemic test:**

| Anemic Test | CRUD Lifecycle Test |
|-------------|---------------------|
| `assert.success()` | `assert!(found_in_list, "BUG #536 WOULD BE CAUGHT HERE!")` |
| Checks: exit code 0 | Checks: actual prompt appears in output |
| Bug invisible | Bug causes test failure |

**This test would have blocked the merge** and prevented the bug from shipping to production.

---

## Process Failures

### 1. Insufficient Test Review

**Problem:** PR reviewer did not question test quality

When the default-to-private feature PR was reviewed, the reviewer saw 9 passing integration tests and approved the PR. The reviewer did not ask: "Do these tests verify actual behavior, or just exit codes?"

**Should have asked:**
- "Does this test verify that private prompts are actually returned?"
- "How do we know the filtering logic works correctly end-to-end?"
- "Would this test catch a bug where zero results are returned?"

**Prevention:** Add test quality checklist to PR review template

### 2. No CRUD Pattern Requirement

**Problem:** No documented standard for integration test quality

The project had no documentation explaining the CRUD lifecycle pattern or requiring its use for integration tests. Developers and AI agents had no clear guidance on what constitutes a "good" integration test.

**Should have existed:**
- `docs/dev/testing/crud-lifecycle-pattern.md` - Pattern documentation
- `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` - Quality standards
- `docs/dev/testing/README.md` - Central index to all testing docs

**Result:** Developers fell back on simpler patterns (exit-code checks) because they lacked examples and requirements for comprehensive testing.

**Prevention:** This milestone (#556) is creating these missing documents.

### 3. No Output Verification Guidelines

**Problem:** No explicit requirement that tests must verify output content

The project lacked a clear guideline stating: "Exit code checks are insufficient for integration tests. Always verify output content."

**Should have existed:** Rule documented in `HIGH_LEVEL_TESTING_GUIDELINES.md`:

> **Rule: Integration tests must verify actual behavior**
>
> Testing only exit codes (assert.success()) is insufficient. Integration tests must:
> 1. Create deterministic test data
> 2. Execute the command
> 3. Parse and verify output content
> 4. Check that output matches expected data
> 5. Clean up test data

**Prevention:** Add this rule as part of milestone #556.

### 4. Missing High-Level Testing Principles

**Problem:** No "Toyota andon cord" principle documented

The project lacked high-level testing philosophy explaining why test quality matters and empowering developers to "pull the andon cord" (block merges) when tests are insufficient.

**Should have existed:** `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` with principles like:

> **Toyota Andon Cord Principle**
>
> Any developer or reviewer can and should block a PR if tests are insufficient, even if CI passes. Quality over speed. It's better to delay a merge than to ship a bug to production.

**Prevention:** Document testing principles in milestone #556.

### 5. No Centralized Testing Documentation

**Problem:** Testing standards scattered across multiple files, not discoverable

Before milestone #556, testing documentation was fragmented:
- `.devcontainer/features/langstar/TESTING-GITHUB-ACTIONS.md` (371 lines)
- `cli/tests/README.md` (252 lines)
- `sdk/tests/README.md` (453 lines)
- `tests/fixtures/test-graph-deployment/README.md` (272 lines)

**Result:** Developers and AI agents didn't know where to find testing standards, so they:
- Created inconsistent test patterns
- Missed important requirements
- Lacked examples of comprehensive tests

**Should have existed:** `docs/dev/testing/README.md` as a central index with progressive disclosure:

```markdown
# Testing Documentation

## Quick Start
- [High-Level Guidelines](./HIGH_LEVEL_TESTING_GUIDELINES.md) - Principles, andon cord
- [CRUD Lifecycle Pattern](./crud-lifecycle-pattern.md) - Integration test standard

## Detailed Guides
- [CLI Integration Tests](./cli-integration-tests.md) - Load only when writing CLI tests
- [SDK Integration Tests](./sdk-integration-tests.md) - Load only when writing SDK tests
```

**Prevention:** Milestone #556 is creating centralized testing documentation.

---

## Lessons Learned

### For Test Authors

1. **Never check only exit codes**
   - Exit-code-only tests create false confidence
   - Always verify actual behavior: parse output, check content
   - Use CRUD lifecycle pattern for integration tests

2. **Use CRUD lifecycle for API features**
   - Create → Verify → Read → Verify → Cleanup
   - Create deterministic test data with known values
   - Verify command output contains expected data
   - Clean up to keep tests isolated

3. **Test with real API data**
   - Don't rely on mocks for integration tests
   - Create actual test data in the API
   - Verify end-to-end behavior against real API

4. **Verify output content, not just structure**
   - Don't just check that JSON is valid
   - Check that JSON contains expected values
   - Search for your test data in the output

### For PR Reviewers

1. **Question test quality**
   - Ask: "Does this test verify behavior, or just exit codes?"
   - Don't be satisfied with passing tests
   - Require output verification for integration tests

2. **Require output verification**
   - Block PRs with exit-code-only integration tests
   - Ask: "How does this test verify correct output?"
   - Require parsing output and checking content

3. **Check for CRUD pattern**
   - For API features, require Create → Read → Verify cycle
   - Ensure tests use deterministic test data
   - Verify cleanup is included

4. **Pull the andon cord**
   - Block insufficient tests even if CI passes
   - Better to delay merge than ship bugs
   - Quality over speed—always

### For Process

1. **Document testing standards centrally**
   - Create `docs/dev/testing/README.md` as index
   - Use progressive disclosure (load on-demand)
   - Provide clear examples of good tests

2. **Automate test planning**
   - Use `/gh-milestones:test-plan` command
   - Generate test plans for milestones
   - Ensure comprehensive test coverage

3. **Enforce pre-merge checks**
   - Add test quality checklist to PR template
   - Empower reviewers to block insufficient tests
   - Document "andon cord" principle

4. **Learn from failures**
   - Write post-mortems for all shipped bugs
   - Identify root causes and process failures
   - Update standards to prevent recurrence

---

## References

- **Original bug:** [Issue #536](https://github.com/codekiln/langstar/issues/536)
- **Fix PR:** [PR #538](https://github.com/codekiln/langstar/pull/538) (merged 2025-12-04)
- **Testing gap epic:** [Issue #556](https://github.com/codekiln/langstar/issues/556)
- **Research audit:** [Issue #557](https://github.com/codekiln/langstar/issues/557)
- **This post-mortem:** [Issue #568](https://github.com/codekiln/langstar/issues/568)
- **CRUD pattern:** `@docs/dev/testing/crud-lifecycle-pattern.md` (to be created in #556)
- **High-level guidelines:** `@docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` (to be created in #556)
- **Testing index:** `@docs/dev/testing/README.md` (to be created in #556)
- **API specification:** `reference/api-specs/langsmith/prompt-endpoints.json:161-178`
- **Anemic test:** `cli/tests/prompt_scoping_test.rs:42-494` (pre-fix)
- **Comprehensive test:** `cli/tests/prompt_scoping_test.rs:536-881` (post-fix)

---

## Timeline

| Date | Event |
|------|-------|
| ~2025-11 | Default-to-private feature implemented (commit 9602e3e) |
| ~2025-11 | PR merged with 9 passing (but anemic) integration tests |
| ~2025-11 | Bug shipped in release v0.3.x |
| ~2025-12 | Bug discovered via manual testing/user report |
| 2025-12-04 | Issue #536 filed |
| 2025-12-04 | PR #538 created with fix + comprehensive tests |
| 2025-12-04 | PR #538 merged, bug fixed |
| 2025-12-04 | Issue #556 filed (testing documentation gap) |
| 2025-12-06 | This post-mortem created (issue #568) |

---

**Document version:** 1.0
**Last updated:** 2025-12-06
**Author:** Claude Code (issue #568)
**Reviewed by:** TBD
