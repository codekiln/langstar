---
description: Audit test implementation compliance against test plan and project guidelines
argument-hint: <milestone-name-or-number>
---

# Audit Test Implementation Compliance

This command verifies that implemented tests comply with both the test plan (Phase 7) and the project's testing guidelines. It catches common issues that slip through even well-intentioned test implementations.

## Usage

```
/gh-milestones:test-audit <milestone-name-or-number>
```

**Examples:**
- `/gh-milestones:test-audit ls-runs-query`
- `/gh-milestones:test-audit 14`
- `/gh-milestones:test-audit https://github.com/codekiln/langstar/milestone/14`

## Why This Phase Exists

Experience has shown that test implementations often deviate from test plans in problematic ways. Common issues include:

- Integration tests marked `#[ignore]` instead of conditionally ignored
- CI not configured with required environment variables
- Anemic tests that only verify exit codes, not actual behavior
- Missing CRUD lifecycle verification (SDK → CLI → SDK)
- Tests that don't clean up resources

See [Issue #637 post-mortem](https://github.com/codekiln/langstar/issues/637) for a real-world example.

## Command Behavior

When this command runs, you should:

### Step 1: Load Testing Documentation

Load the high-level testing guidelines (always required):
- `@docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md`

### Step 2: Locate Test Plan

Find the test plan from Phase 7:
```bash
# Check common locations
ls docs/implementation/*test-plan*.md
```

Or search for test plan in milestone issues:
```bash
gh issue list --milestone "<milestone-name>" --json number,title,body | jq '.[] | select(.title | contains("test"))'
```

### Step 3: Fetch Implemented Tests

Identify test files for the milestone feature:
```bash
# SDK tests
find sdk/tests -name "*<feature>*_test.rs"

# CLI tests
find cli/tests -name "*<feature>*_command_test.rs"

# In-module tests
grep -l "#\[cfg(test)\]" sdk/src/<feature>.rs cli/src/commands/<feature>.rs 2>/dev/null
```

### Step 4: Run Audit Checks

For each test file, verify compliance with these requirements:

#### 4.1 Test Structure Checks

```bash
# Check for unconditional #[ignore] (BAD)
grep -n "#\[ignore\]" <test-file>

# Check for proper conditional ignore (GOOD)
grep -n "cfg_attr(not(feature = \"integration-tests\"), ignore)" <test-file>

# Check for proper test module structure
grep -n "#\[cfg(test)\]" <test-file>
```

#### 4.2 Test Quality Checks (Manual Review)

Review each test for:
- [ ] **Behavior verification**: Does test verify actual behavior, not just exit codes?
- [ ] **Round-trip assertions**: Are SDK operations verified through assertions?
- [ ] **Error type checking**: Do error tests verify specific error types?
- [ ] **Edge case coverage**: Are boundary conditions from test plan covered?

#### 4.3 CRUD Lifecycle Checks

For integration tests, verify:
- [ ] Resources created via SDK (not just CLI)
- [ ] Operations use CLI or SDK under test
- [ ] Results verified via SDK (not just CLI output)
- [ ] Resources cleaned up (even on failure)

#### 4.4 CI Configuration Checks

```bash
# Check workflow files for required env vars
grep -l "LANGSMITH_API_KEY" .github/workflows/*.yml
grep -l "integration-tests" .github/workflows/*.yml

# Verify feature flag is enabled
grep "integration-tests" .github/workflows/*.yml
```

### Step 5: Generate Compliance Report

Create a markdown report with this structure:

```markdown
# Test Audit Report: [Milestone Name]

## Summary
- **Tests Planned**: [count from test plan]
- **Tests Implemented**: [count found]
- **Compliance Rate**: [percentage]
- **Critical Issues**: [count]
- **Warnings**: [count]

## Critical Issues (Must Fix Before Merge)

### Issue 1: [Title]
- **Location**: `file:line`
- **Problem**: [specific issue]
- **Remediation**: [exact fix needed]

### Issue 2: ...

## Warnings (Should Address)

### Warning 1: [Title]
- **Location**: `file:line`
- **Suggestion**: [improvement]

## Test Plan Coverage Matrix

| Test Case (from plan) | Implemented? | File:Line | Notes |
|----------------------|--------------|-----------|-------|
| test_create_run | ✅ Yes | sdk/tests/runs_test.rs:45 | |
| test_query_runs_empty | ❌ No | - | Missing |
| test_query_runs_pagination | ⚠️ Partial | sdk/tests/runs_test.rs:100 | Only tests first page |

## CI Configuration Status

- [ ] LANGSMITH_API_KEY in workflow secrets
- [ ] LANGSMITH_WORKSPACE_ID in workflow secrets
- [ ] `integration-tests` feature enabled in CI
- [ ] Integration test job configured

## Anti-Pattern Detection

| Pattern | Found? | Location |
|---------|--------|----------|
| Unconditional `#[ignore]` | ❌ No | - |
| Exit-code-only assertions | ⚠️ Yes | cli/tests/runs_command_test.rs:55 |
| Missing cleanup | ❌ No | - |
| Hardcoded test data | ⚠️ Yes | sdk/tests/runs_test.rs:20 |

## Recommendations

1. [Specific action item with file:line reference]
2. [Specific action item with file:line reference]

## Next Steps

If critical issues found:
1. Fix all critical issues
2. Re-run audit: `/gh-milestones:test-audit <milestone>`
3. Update test plan if gaps discovered

If all checks pass:
1. Proceed to Phase 10 (Documentation)
2. Ensure tests run in CI before merge
```

### Step 6: Present Results

Output the compliance report with:
- Summary of findings
- Clear distinction between critical issues and warnings
- Actionable remediation steps
- Toyota Andon Cord reminder if critical issues found

## Common Issues and Remediations

### Issue: Unconditional `#[ignore]`

**Problem**: Tests marked with `#[ignore]` won't run in CI.

**Remediation**: Use conditional ignore:
```rust
// WRONG
#[ignore]
#[tokio::test]
async fn test_integration() { ... }

// RIGHT
#[cfg_attr(not(feature = "integration-tests"), ignore)]
#[tokio::test]
async fn test_integration() { ... }
```

### Issue: Exit-Code-Only Assertions

**Problem**: Test only checks `assert!(result.status.success())`.

**Remediation**: Verify actual behavior:
```rust
// WRONG
assert!(result.status.success());

// RIGHT
assert!(result.status.success());
let output = String::from_utf8_lossy(&result.stdout);
assert!(output.contains("expected_data"));
// Or verify via SDK:
let runs = client.query_runs(&request).await?;
assert_eq!(runs.len(), expected_count);
```

### Issue: Missing Cleanup

**Problem**: Test creates resources but doesn't clean up.

**Remediation**: Use cleanup in all paths:
```rust
let project_id = client.create_project(&request).await?;

// Use scopeguard or manual cleanup
let _cleanup = scopeguard::guard((), |_| {
    // Cleanup runs even on panic
    let _ = runtime.block_on(client.delete_project(&project_id));
});

// ... test operations ...
```

### Issue: Missing CI Environment Variables

**Problem**: Integration tests need API keys not configured in CI.

**Remediation**: Update `.github/workflows/test.yml`:
```yaml
integration-tests:
  env:
    LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
    LANGSMITH_WORKSPACE_ID: ${{ secrets.LANGSMITH_WORKSPACE_ID }}
```

## References

- `@docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` - Testing standards
- `@docs/dev/testing/crud-lifecycle-pattern.md` - CRUD test pattern
- `@docs/dev/testing/post-mortems/` - Case studies of testing gaps
- Issue #637 - Post-mortem that motivated this phase
