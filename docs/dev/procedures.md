# Development Procedures and Troubleshooting

This document contains detailed step-by-step procedures and troubleshooting guides. These are operational details that developers may reference as needed, but are not core conventions.

For core conventions and principles, see [README.md](./README.md).

---

## Working with Phased/Sub-Task Issues

When working on multi-phase features that are broken down into sub-issues (e.g., "Phase 1: Research", "Phase 2: Implementation"), **always review prerequisite phases before starting**.

### Procedure for Phase N Issues

When assigned a "Phase N" issue:

1. **Fetch the current issue** to understand the scope:
   ```bash
   gh issue view <current-issue-number> --json title,body,labels
   ```

2. **Fetch the parent issue** to understand the overall feature:
   ```bash
   gh issue view <parent-issue-number> --json title,body,labels
   ```

3. **Check for prerequisite phase issues** (Phase 1 through Phase N-1):
   ```bash
   # List related phase issues
   gh issue list --search "Phase" --state all

   # Or search within parent issue context
   gh issue view <parent-issue-number> --json body | grep -E "Phase [0-9]+"
   ```

4. **Read findings from completed prerequisite phases**:
   ```bash
   # Fetch each prerequisite phase issue with comments
   gh issue view <phase-1-issue> --json title,body,state,comments
   gh issue view <phase-2-issue> --json title,body,state,comments
   # etc.
   ```

5. **Review all comments and findings** from prerequisite phases:
   - Research findings may change implementation approach
   - API behavior discoveries inform design decisions
   - Blockers or limitations discovered in earlier phases
   - Recommendations for subsequent phases

6. **Only then** create your implementation plan and begin work

### Why This Matters

**Example**: Phase 1 (Research) might discover that:
- An API behaves differently than documented
- Two headers should be used together (not either/or)
- Validation is unnecessary (API doesn't validate)
- A simpler approach is recommended

Starting Phase 2 (Implementation) without this knowledge leads to:
- ❌ Implementing the wrong approach
- ❌ Wasted effort that needs to be redone
- ❌ Missing critical context for design decisions

### Creating Sub-Issues

When creating a "Phase N+1" issue that depends on "Phase N":

1. **Add a reminder in the issue body**:
   ```markdown
   ## Prerequisites

   ⚠️ **Before starting this phase, review findings from:**
   - [ ] #XX - Phase N: [Title]
   - [ ] Read all comments and research findings
   - [ ] Verify recommendations for this phase
   ```

2. **Link to prerequisite issues**:
   ```markdown
   Depends on #XX (Phase N)
   Sub-task of #YY (Parent feature)
   ```

3. **Reference key findings** in the issue description if they're critical

### Quick Checklist

Before starting any Phase N work:

- [ ] Fetched and read current issue
- [ ] Fetched and read parent issue
- [ ] Identified all prerequisite phase issues
- [ ] Read all comments/findings from prerequisite phases
- [ ] Understand recommendations for current phase
- [ ] Verified no blockers from previous phases
- [ ] Created todo list based on informed understanding

**Remember**: Taking 10 minutes to review prior work can save hours of implementing the wrong approach.

---

## Pre-Commit Checklist

Before committing and pushing code, **always run these checks locally** to catch issues before CI fails. The CI runs these exact checks, so running them locally prevents wasted time and unnecessary commits.

### Essential Checks (Run Every Time)

Run these commands from the project root (`/workspace`):

```bash
# 1. Format code (auto-fixes formatting issues)
cargo fmt

# 2. Check compilation for entire workspace
cargo check --workspace --all-features

# 3. Run clippy for linting warnings
cargo clippy --workspace --all-features -- -D warnings

# 4. Run all tests in workspace
cargo test --workspace --all-features

# 5. Check formatting (verifies cargo fmt was run)
cargo fmt --check
```

### Why Each Check Matters

**1. `cargo fmt`** (Auto-format)
- Fixes code formatting to match project style
- **Prevents**: "Check" CI job failures
- **Lesson from #75**: Forgot to run this, had to add formatting commit

**2. `cargo check --workspace`** (Compile check)
- Verifies code compiles across **entire workspace** (not just one crate)
- Much faster than full build
- **Prevents**: Build CI job failures
- **Lesson from #75**: Changed `AuthConfig::new()` signature, only tested SDK, missed CLI usage
- **Critical**: When making breaking changes to SDK, this catches all usages in CLI

**3. `cargo clippy`** (Linting)
- Catches common mistakes and non-idiomatic code
- **Prevents**: Clippy CI job failures
- Use `-- -D warnings` to treat warnings as errors (matches CI)

**4. `cargo test --workspace`** (All tests)
- Runs tests for **all crates** (SDK + CLI)
- **Prevents**: Test CI job failures
- **Lesson from #75**: Only ran `cargo test --lib` in SDK directory, missed workspace-level issues
- **Critical**: Always test at workspace level, not just individual crates

**5. `cargo fmt --check`** (Verify formatting)
- Ensures formatting is correct (no changes needed)
- Should pass after step 1
- **Prevents**: "Check" CI job failures

### Quick Pre-Commit Script

Save time with a one-liner that runs all checks:

```bash
cargo fmt && cargo check --workspace --all-features && cargo clippy --workspace --all-features -- -D warnings && cargo test --workspace --all-features && cargo fmt --check
```

**Exit on first failure** (better for catching issues early):

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features && \
cargo fmt --check
```

### Breaking Changes Checklist

When making **breaking changes** to SDK APIs (changing function signatures, removing fields, etc.):

1. **Before changing**: Search for all usages
   ```bash
   # Example: Find all usages of AuthConfig::new
   grep -r "AuthConfig::new" --include="*.rs"
   ```

2. **After changing**: Update **all** call sites found
   - Don't assume you know where it's used
   - The search will find usages in CLI, tests, examples, etc.

3. **Verify with workspace check**:
   ```bash
   cargo check --workspace --all-features
   ```

4. **Test at workspace level**:
   ```bash
   cargo test --workspace --all-features
   ```

### Common Mistakes to Avoid

❌ **Testing only one crate**: `cargo test` or `cargo test --lib`
- Misses issues in dependent crates (CLI depends on SDK)

✅ **Test entire workspace**: `cargo test --workspace`

---

❌ **Checking only one crate**: `cargo check` in SDK directory
- Misses compilation errors in CLI that uses SDK

✅ **Check entire workspace**: `cargo check --workspace`

---

❌ **Skipping cargo fmt**: "I'll format it later"
- CI will fail on formatting issues

✅ **Format before committing**: `cargo fmt` (takes 1 second)

---

❌ **Changing API without searching for usages**: "I know where it's used"
- Easy to miss usages in tests, examples, or other crates

✅ **Search first, change second**: `grep -r "function_name" --include="*.rs"`

### Time Investment

Running all checks locally takes ~30-60 seconds total:
- `cargo fmt`: <1 second
- `cargo check --workspace`: ~5-15 seconds (cached)
- `cargo clippy --workspace`: ~5-15 seconds (cached)
- `cargo test --workspace`: ~10-30 seconds
- `cargo fmt --check`: <1 second

**Compare to**:
- Waiting for CI to fail: 2-5 minutes
- Fixing issue: 5-10 minutes
- Pushing fix: 1 minute
- Waiting for CI again: 2-5 minutes
- **Total wasted time**: 10-20 minutes per mistake

**Lesson**: Spending 1 minute running checks locally saves 10-20 minutes of CI roundtrips.

### Integration with Git Hooks (Optional)

Consider setting up a pre-commit hook to run these automatically:

```bash
# .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running pre-commit checks..."

echo "→ Formatting code..."
cargo fmt

echo "→ Checking workspace..."
cargo check --workspace --all-features

echo "→ Running clippy..."
cargo clippy --workspace --all-features -- -D warnings

echo "→ Running tests..."
cargo test --workspace --all-features

echo "→ Verifying formatting..."
cargo fmt --check

echo "✓ All checks passed!"
```

Make it executable: `chmod +x .git/hooks/pre-commit`

**Note**: This runs on every commit. If it's too slow, consider a lighter version that only runs `cargo fmt` and `cargo check --workspace`.
