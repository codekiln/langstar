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

For pre-commit testing requirements, see `docs/dev/testing/cli-integration-tests.md`.

**Quick reference:**
```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features && \
cargo fmt --check
```

**Full documentation:** `docs/dev/testing/cli-integration-tests.md`

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

---

## Configuring Repository Rulesets for Mandatory CI Tests

This repository uses **GitHub Repository Rulesets** (not classic branch protection rules) to enforce CI test requirements before merging. This ensures that broken features cannot be merged accidentally.

### Overview

**What are Repository Rulesets?**
- Named lists of rules that apply to repository branches or tags
- Modern replacement for classic branch protection rules
- Multiple rulesets can apply simultaneously (most restrictive rule wins)
- Support flexible enforcement modes (active, disabled, evaluate)

**Key Benefits:**
- ✅ Multiple rulesets can target the same branch
- ✅ Anyone with read access can view active rulesets
- ✅ More granular control than classic branch protection
- ✅ Support for advanced features (commit message validation, email validation, etc.)

**References:**
- [GitHub Rulesets Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)
- [GitHub Rulesets API](https://docs.github.com/en/rest/repos/rules)

### Current Configuration

The repository has three active rulesets:

1. **"main"** (ID: 9196293)
   - Target: Default branch (main)
   - Requires PR approval (1 reviewer + code owner)
   - Requires status checks to pass ("All Jobs")
   - Blocks deletion and force pushes
   - **Strict mode**: Enabled (branches must be up-to-date before merge)

2. **"Release Branch Protection"** (ID: 9347848)
   - Target: Release branches (release/*)
   - Similar protections for release branches

3. **"Copilot review for default branch"** (ID: 10352512)
   - Target: Default branch
   - Enables Copilot code review integration

### How Required Status Checks Work

**Required Status Checks:**
- The "main" ruleset requires "All Jobs" (integration_id: 15368) to pass
- "All Jobs" is a GitHub Actions integration that requires all workflow jobs to succeed
- This includes the `test-features` workflow when it runs

**Workflow Path Filtering:**
- The `test-features` workflow has path filters: `.devcontainer/features/**`
- It only runs when files in that path are modified
- If the workflow runs, it MUST pass before merging (enforced by "All Jobs")
- If the workflow doesn't run (no relevant files changed), it's not required

**Strict Mode:**
- `strict_required_status_checks_policy: true` (enabled)
- Requires branches to be up-to-date with the base branch before merging
- Prevents merge conflicts and integration issues
- Ensures all status checks run on the exact code that will be merged

### Viewing Current Rulesets

**List all rulesets:**
```bash
gh api repos/codekiln/langstar/rulesets | jq '.[] | {id: .id, name: .name, enforcement: .enforcement}'
```

**View a specific ruleset:**
```bash
gh api repos/codekiln/langstar/rulesets/9196293 | jq .
```

**View in GitHub UI:**
- Go to: Settings → Rules → Rulesets
- URL: https://github.com/codekiln/langstar/rules

### Updating Rulesets

**Important:** Updating rulesets requires "Administration" repository permissions.

#### Using the GitHub API

**1. Fetch current ruleset:**
```bash
gh api repos/codekiln/langstar/rulesets/<RULESET_ID> > ruleset.json
```

**2. Prepare update payload (keep only updatable fields):**
```bash
cat ruleset.json | jq '{
  name: .name,
  target: .target,
  enforcement: .enforcement,
  bypass_actors: .bypass_actors,
  conditions: .conditions,
  rules: .rules
}' > updated-ruleset.json
```

**3. Modify the JSON as needed (e.g., enable strict mode):**
```bash
# Example: Enable strict required status checks
cat updated-ruleset.json | jq '
  .rules = (.rules | map(
    if .type == "required_status_checks"
    then .parameters.strict_required_status_checks_policy = true
    else .
    end
  ))
' > final-ruleset.json
```

**4. Apply the update:**
```bash
gh api -X PUT repos/codekiln/langstar/rulesets/<RULESET_ID> \
  --input final-ruleset.json
```

#### Using the GitHub UI

**1. Navigate to Settings → Rules → Rulesets**

**2. Click on the ruleset to edit (e.g., "main")**

**3. Modify settings:**
   - Status checks: Add/remove required checks
   - Strict mode: Toggle "Require branches to be up to date before merging"
   - Review requirements: Change approval count or code owner requirement

**4. Save changes**

### Common Ruleset Modifications

#### Enable Strict Mode for Status Checks

**What it does:** Requires branches to be up-to-date with base branch before merging.

**Why:** Prevents merge conflicts and ensures status checks run on final code.

**How:**
```bash
# Fetch ruleset
gh api repos/codekiln/langstar/rulesets/9196293 > main-ruleset.json

# Update strict mode
cat main-ruleset.json | jq '{
  name: .name,
  target: .target,
  enforcement: .enforcement,
  bypass_actors: .bypass_actors,
  conditions: .conditions,
  rules: (.rules | map(
    if .type == "required_status_checks"
    then .parameters.strict_required_status_checks_policy = true
    else .
    end
  ))
}' > updated-main-ruleset.json

# Apply update
gh api -X PUT repos/codekiln/langstar/rulesets/9196293 \
  --input updated-main-ruleset.json
```

#### Add Explicit Status Check Requirements

**What it does:** Requires specific workflow jobs to pass (instead of "All Jobs").

**Why:** More explicit control over which checks are mandatory.

**How:**
```bash
# Modify the required_status_checks in the JSON:
{
  "type": "required_status_checks",
  "parameters": {
    "strict_required_status_checks_policy": true,
    "do_not_enforce_on_create": false,
    "required_status_checks": [
      {
        "context": "Test Features (ubuntu-22.04)",
        "integration_id": 15368
      },
      {
        "context": "Test Features (debian-12)",
        "integration_id": 15368
      }
    ]
  }
}
```

**Note:** The `context` must match the job name in the workflow. Use `gh api repos/codekiln/langstar/commits/<SHA>/check-runs` to see actual check names.

#### Require More PR Approvals

**What it does:** Increases required approval count before merging.

**How:**
```bash
# Modify the pull_request rule in the JSON:
# Change required_approving_review_count from 1 to 2
{
  "type": "pull_request",
  "parameters": {
    "required_approving_review_count": 2,
    "require_code_owner_review": true,
    "dismiss_stale_reviews_on_push": false
  }
}
```

### Troubleshooting

#### "Resource not accessible by integration"

**Cause:** Insufficient permissions to update rulesets.

**Fix:** Ensure your GitHub token has "Administration" repository permissions.

#### Status check not required but workflow failed

**Cause:** Either the workflow isn't required, or it has path filters that exclude the changes.

**Fix:**
- Check if "All Jobs" is required in the ruleset
- Verify workflow path filters match changed files
- Consider adding explicit status check requirements

#### "Branch is out of date" error

**Cause:** Strict mode is enabled, requiring branches to be up-to-date.

**Fix:**
1. Update your branch from base:
   ```bash
   git fetch origin
   git merge origin/main  # Or: git rebase origin/main
   ```
2. Push updated branch
3. Wait for CI to complete
4. Now you can merge

### Best Practices

✅ **Use "All Jobs" for simplicity**
- Automatically includes all workflow jobs
- No need to update ruleset when adding new workflows

✅ **Enable strict mode**
- Prevents merge conflicts
- Ensures tests run on final code
- Slight inconvenience (need to update branches) is worth it

✅ **Keep rulesets minimal**
- Only add rules that are truly necessary
- Too many rules can slow down development

✅ **Document changes**
- When updating rulesets, document why in commit message
- Consider adding issue/PR references

❌ **Don't disable rulesets casually**
- Rulesets exist to prevent broken code from merging
- If tests are failing, fix the tests (don't disable protection)

❌ **Don't add every workflow as required**
- Some workflows are optional (e.g., release automation)
- Only require workflows that validate code quality

### Related Files

- `.github/workflows/test-features.yml` - The feature testing workflow
- `.github/workflows/ci.yml` - Main CI workflow (tests, clippy, formatting)
- `docs/dev/github-workflow.md` - GitHub issue-driven development workflow

### Further Reading

- [GitHub Rulesets Documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets)
- [GitHub Required Status Checks](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches#require-status-checks-before-merging)
- [GitHub Rulesets API](https://docs.github.com/en/rest/repos/rules)
