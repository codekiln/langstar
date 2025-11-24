# GitHub Workflows

This directory contains GitHub Actions workflows for CI/CD automation.

## Workflows

- **ci.yml** - Continuous Integration (format, test, lint, build, audit)
- **prepare-release.yml** - Automated release PR generation
- **auto-tag-release.yml** - Automatic git tag creation on release PR merge
- **release.yml** - Build and publish release artifacts

## Release Workflow: Draft Releases

### Overview

The release workflow creates **draft releases** to enable human review before publishing. This follows the ripgrep pattern and prevents accidental publication of problematic releases.

### Complete Release Flow

```
1. Developer clicks "Prepare Release" → PR created automatically
   ↓
2. CI checks enforced by branch protection
   ↓
3. Developer reviews and merges PR → auto-tag-release creates git tag
   ↓
4. Tag triggers release.yml with version validation
   ↓
5. Build artifacts and create DRAFT release
   ↓
6. Developer reviews draft and clicks "Publish" ✅
```

### Version Validation (Implemented in Issue #232)

Before creating a release, the workflow validates that the git tag matches `Cargo.toml` version:

**Location**: `release.yml` lines 77-92

```yaml
- name: Validate tag matches Cargo.toml version
  run: |
    TAG_VERSION="${{ steps.get_version.outputs.version }}"
    CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)

    if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
      echo "❌ Version mismatch detected!" >&2
      # ... error details ...
      exit 1
    fi

    echo "✅ Version validation passed: $TAG_VERSION"
```

**Why this matters**: Prevents publishing releases with mismatched versions (learned from PR #239). If the tag is v0.4.3 but Cargo.toml says v0.4.4, the workflow fails with a clear error.

### Draft Release Process

**Why drafts?**
- Review artifacts before making them public
- Fix issues without embarrassing re-releases
- Verify checksums and download links
- Test installation before announcement

**Location**: `release.yml` line 115 (`draft: true`)

### How to Review and Publish a Draft Release

1. **After PR merge**, the release workflow runs automatically
2. **Check GitHub releases page**: https://github.com/codekiln/langstar/releases
3. **Find the draft release** (will have "Draft" badge)
4. **Review the release**:
   - [ ] Changelog is accurate
   - [ ] All artifacts uploaded (3 .tar.gz + 3 .sha256 files)
   - [ ] SHA256 checksums present
   - [ ] Version number correct
5. **Test installation** (optional but recommended):
   ```bash
   # Download artifact from draft
   # Verify SHA256
   # Test binary works
   ```
6. **Click "Publish release"** when satisfied
7. **Release becomes public** and appears on releases page

### What Gets Built

For each release, the workflow builds:

| Platform | Target | Artifact Format |
|----------|--------|----------------|
| Linux x86_64 | `x86_64-unknown-linux-musl` | .tar.gz + .sha256 |
| Linux ARM64 | `aarch64-unknown-linux-musl` | .tar.gz + .sha256 |
| macOS ARM64 | `aarch64-apple-darwin` | .tar.gz + .sha256 |

**Total**: 6 files (3 archives + 3 checksums)

### Troubleshooting

#### Version Mismatch Error

**Symptom**: Release workflow fails with "Version mismatch detected!"

**Cause**: Git tag version doesn't match `Cargo.toml` version

**Solution**:
```bash
# Delete the incorrect tag
git push --delete origin vX.Y.Z
git tag -d vX.Y.Z

# Fix Cargo.toml version OR create correct tag
# If using prepare-release workflow, it should handle this automatically
```

#### Draft Release Not Appearing

**Symptom**: Workflow runs but no draft release visible

**Possible causes**:
1. Workflow failed - Check Actions logs
2. Permission issues - Verify GITHUB_TOKEN has write permissions
3. Tag format incorrect - Must be `v*` (e.g., `v0.4.3`, not `0.4.3`)

#### Artifacts Not Uploading

**Symptom**: Draft release exists but no artifacts attached

**Check**:
1. Build jobs completed successfully - View workflow run
2. Matrix builds ran for all platforms - Check job logs
3. Upload step succeeded - Look for errors in "Upload release asset" steps

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
