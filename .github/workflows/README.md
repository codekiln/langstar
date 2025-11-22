# GitHub Workflows

This directory contains GitHub Actions workflows for CI/CD automation.

## Workflows

- **ci.yml** - Continuous Integration (format, test, lint, build, audit)
- **prepare-release.yml** - Automated release PR generation
- **auto-tag-release.yml** - Automatic git tag creation on release PR merge
- **release.yml** - Build and publish release artifacts

## Critical: The "All Jobs" Aggregation Gate

### What is it?

The `all-jobs` job in `ci.yml` aggregates all CI check results into a single status check.

**Why it exists**:
- Matrix builds create multiple status checks (build-linux, build-macos, etc.)
- Branch protection can only require ONE status check
- Solution: Aggregate all jobs → single "All Jobs" check

**Location**: `ci.yml` lines 150-164

```yaml
all-jobs:
  name: All Jobs
  if: always()
  runs-on: ubuntu-latest
  needs:
    - check
    - test
    - integration-tests
    - clippy
    - audit
    - build
  steps:
    - name: Check all job results
      run: |
        jq --exit-status 'all(.result == "success" or .result == "skipped")' <<< '${{ toJson(needs) }}'
```

### Branch Protection Dependency

**CRITICAL**: Main branch protection **requires** "All Jobs" status check.

**Settings**: https://github.com/codekiln/langstar/settings/rules/9196293

This means:
- ✅ "All Jobs" check MUST pass before merge
- ❌ Cannot merge if "All Jobs" never ran
- ⚠️ If you change ci.yml, "All Jobs" must still exist

## Modifying CI Workflows: Required Procedure

### Adding or Removing CI Jobs

When you add/remove jobs from `ci.yml`:

1. **Update the `needs` list in `all-jobs`**
   - Location: `ci.yml` line 154-160
   - Add new job names to the list
   - Remove deleted job names from the list

2. **Test the change**
   - Push to PR branch
   - Verify "All Jobs" check appears in status checks
   - Verify "All Jobs" passes if all jobs pass

3. **NEVER remove the `all-jobs` job entirely**
   - Branch protection requires it
   - Would block ALL merges to main

### Adding Required Status Checks

⚠️ **LESSON LEARNED FROM ISSUE #235**

When adding a NEW required status check to branch protection:

#### Option A: Correct Order (Recommended)

1. **Add the check to workflow first** (merge to main)
2. **Wait for open PRs to re-run CI** with new workflow
   - Verify check appears on open PRs
   - May need to close/reopen PRs or push empty commits
3. **Then add branch protection requirement**

#### Option B: If You Add Protection First (like we did in #235)

1. **Document the change**: Comment on all open PRs
2. **Re-trigger CI**: Close/reopen PRs or push empty commits
   ```bash
   # Option 1: Close and reopen
   gh pr close <PR>
   gh pr reopen <PR>

   # Option 2: Push empty commit
   gh pr checkout <PR>
   git commit --allow-empty -m "🔧 ci: trigger workflow re-run"
   git push
   ```
3. **Verify**: All open PRs now have the new check

#### Why This Matters

**What happened in issue #235**:
- Added "All Jobs" to branch protection
- Added `all-jobs` job to ci.yml
- PR #223 created BEFORE `all-jobs` merged
- PR #223 ran old workflow (no all-jobs gate)
- **Result**: PR #223 permanently blocked (protection requires check that never ran)

**Impact**: Any PR created between steps 1 and 2 will be blocked forever unless CI is manually re-triggered.

## Troubleshooting

### PR Blocked: "Waiting on All Jobs"

**Symptom**: PR shows all checks passing but merge blocked

**Cause**: PR ran CI before all-jobs gate was added

**Solution**:
```bash
# Option 1: Close and reopen PR
gh pr close <PR_NUMBER>
gh pr reopen <PR_NUMBER>

# Option 2: Push empty commit to PR branch
gh pr checkout <PR_NUMBER>
git commit --allow-empty -m "🔧 ci: trigger CI re-run for all-jobs gate"
git push
```

### "All Jobs" Check Not Appearing

**Possible causes**:
1. PR created before all-jobs gate merged
   - **Solution**: Re-trigger CI (see above)

2. `all-jobs` job has wrong dependencies
   - **Check**: `needs` list in `all-jobs` job
   - **Fix**: Ensure all CI jobs are listed

3. `all-jobs` job failing silently
   - **Check**: Workflow logs for all-jobs step
   - **Debug**: Look for jq errors in "Check all job results" step

### Changing Branch Protection

**Before changing** https://github.com/codekiln/langstar/settings/rules/9196293:

1. **Check if removing required checks**
   - Will unblock stuck PRs
   - But removes quality gate (not recommended)

2. **Check if adding required checks**
   - Follow "Option A" procedure above
   - Verify check exists in workflow FIRST

## References

- Issue #235: All Jobs check not running (lesson learned)
- Issue #199: Automated release PR generation (parent)
- Issue #230: CI quality gates implementation
- PR #233: Added all-jobs gate to ci.yml
