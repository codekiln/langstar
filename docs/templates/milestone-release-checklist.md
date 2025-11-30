# Milestone Release Checklist (Phase 9)

Use this checklist when releasing a milestone to ensure all steps are completed correctly.

---

## Milestone Information

**Milestone**: {milestone-name} (#{milestone-num})
**Version**: v{X.Y.Z}
**Parent Issue**: #{parent-issue-num} - {parent-issue-title}
**Release Date**: {YYYY-MM-DD}

---

## Phase 1: Pre-Release Validation

### Sub-Issue Completion

- [ ] All sub-issues are closed
  ```bash
  # Verify:
  gh issue list --milestone "{milestone-name}" --state open
  # Should return: no issues
  ```

- [ ] All PRs merged to main
  ```bash
  # Verify:
  gh pr list --search "milestone:{milestone-name} is:open"
  # Should return: no PRs
  ```

### Code Quality

- [ ] CI/CD passing on main branch
  ```bash
  # Verify:
  gh run list --branch main --limit 1
  ```

- [ ] Local build successful
  ```bash
  cargo build --workspace --all-features
  ```

- [ ] All tests passing
  ```bash
  cargo test --workspace --all-features
  ```

- [ ] No clippy warnings
  ```bash
  cargo clippy --workspace --all-features -- -D warnings
  ```

- [ ] Code formatted
  ```bash
  cargo fmt --check
  ```

### Documentation

- [ ] CHANGELOG.md updated (if manual versioning)
- [ ] README.md updated with new features
- [ ] Implementation docs committed to `docs/implementation/`
- [ ] Research reports committed to `docs/research/` or `reference/research/`

---

## Phase 2: Create GitHub Release

### Version Bump (if manual)

- [ ] Version updated in `Cargo.toml` files
  ```bash
  # Update version in all workspace members
  # Verify:
  grep -r "version = \"" --include="Cargo.toml"
  ```

- [ ] Git tag created
  ```bash
  git checkout main && git pull
  git tag -a v{X.Y.Z} -m "Release v{X.Y.Z}"
  git push origin v{X.Y.Z}
  ```

### GitHub Release

- [ ] Release created
  ```bash
  gh release create v{X.Y.Z} --generate-notes
  # Or manually create via GitHub UI
  ```

- [ ] Release notes reviewed and edited (if needed)

- [ ] Release published (not draft)
  ```bash
  # Verify:
  gh release view v{X.Y.Z}
  ```

- [ ] Binary assets uploaded (if applicable)

---

## Phase 3: Milestone Cleanup (Automated)

### Run `/ls-release-milestone` Command

```bash
/ls-release-milestone "{milestone-name}" v{X.Y.Z}
```

**Alternative (if using URL)**:
```bash
/ls-release-milestone https://github.com/{owner}/{repo}/milestone/{milestone-num} v{X.Y.Z}
```

### Expected Output

```
✅ **Milestone Release Tracking Complete**

📍 Milestone: {milestone-name} (#{milestone-num})
🔗 Parent Issue: #{parent-issue-num} - {parent-issue-title}
📦 Release: v{X.Y.Z}
🔗 Release URL: https://github.com/{owner}/{repo}/releases/tag/v{X.Y.Z}

**Actions Completed:**
✅ Verified release v{X.Y.Z} exists
✅ Validated sub-issue completion
✅ Milestone marked as closed
✅ Milestone description updated with release information
✅ Parent issue #{parent-issue-num} closed with release comment
```

### Manual Override (if needed)

If sub-issues are intentionally still open:
```bash
FORCE_RELEASE=true /ls-release-milestone "{milestone-name}" v{X.Y.Z}
```

**⚠️ Warning**: Only use `FORCE_RELEASE` if you have a valid reason. Best practice is to close all sub-issues before release.

---

## Phase 4: Verification

### Milestone Verification

- [ ] Milestone is marked as closed
  ```bash
  gh api /repos/{owner}/{repo}/milestones/{milestone-num} --jq '.state'
  # Should output: closed
  ```

- [ ] Milestone description contains release link
  ```bash
  gh api /repos/{owner}/{repo}/milestones/{milestone-num} --jq '.description'
  # Should contain: "Shipped in [release v{X.Y.Z}](...)"
  ```

- [ ] Milestone URL accessible:
  ```
  https://github.com/{owner}/{repo}/milestone/{milestone-num}
  ```

### Parent Issue Verification

- [ ] Parent issue is closed
  ```bash
  gh issue view {parent-issue-num} --json state --jq '.state'
  # Should output: CLOSED
  ```

- [ ] Parent issue has release comment
  ```bash
  gh issue view {parent-issue-num} --json comments --jq '.comments[-1].body'
  # Should contain: "Shipped in [release v{X.Y.Z}](...)"
  ```

- [ ] Parent issue URL accessible:
  ```
  https://github.com/{owner}/{repo}/issues/{parent-issue-num}
  ```

### Release Verification

- [ ] Release is published (not draft)
- [ ] Release has correct tag: `v{X.Y.Z}`
- [ ] Release notes are complete
- [ ] Release URL accessible:
  ```
  https://github.com/{owner}/{repo}/releases/tag/v{X.Y.Z}
  ```

---

## Phase 5: Post-Release Cleanup

### Local Cleanup

- [ ] Update local main branch
  ```bash
  git checkout main && git pull origin main
  ```

- [ ] Delete feature branches (if using worktrees)
  ```bash
  # If using git-worktrees skill
  cd /workspace  # Main worktree
  git worktree remove wip/{branch-name}
  git branch -d {branch-name}
  git worktree prune
  ```

- [ ] Clean up any temporary files or experiments

### Team Communication

- [ ] Announce release in team chat (if applicable)
- [ ] Update internal documentation (if applicable)
- [ ] Notify stakeholders of new features (if applicable)

---

## Phase 6: Celebrate! 🎉

- [ ] Milestone successfully released!
- [ ] Features deployed to production
- [ ] Users can now benefit from the new functionality

---

## Troubleshooting

### Common Issues

**`/ls-release-milestone` command not found**:
- Verify slash command exists: `ls .claude/commands/ls-release-milestone.md`
- May need to reload Claude Code or IDE

**"Release not found" error**:
- Verify release exists: `gh release list`
- Ensure release is published (not draft)
- Check version format: must be `vX.Y.Z`

**"Open sub-issues detected" error**:
- Close remaining sub-issues: `gh issue list --milestone "{milestone-name}" --state open`
- Or force release: `FORCE_RELEASE=true /ls-release-milestone ...`

**"gh-sub-issue extension not installed" warning**:
- Command will still work, but skips sub-issue validation
- Install extension: `gh extension install https://github.com/codekiln/gh-sub-issue`

**Milestone not closing**:
- Check GitHub API rate limits: `gh api rate_limit`
- Verify GitHub CLI authentication: `gh auth status`
- Retry command

---

## References

- **Phase 9 Documentation**: `@docs/dev/feature-development-process.md#phase-9-milestone-release`
- **Command Documentation**: `.claude/commands/ls-release-milestone.md`
- **GitHub Workflow**: `@docs/dev/github-workflow.md`
- **Example**: Milestone #7 (ls-prompt-structured-outputs) released in v0.10.0

---

## Template Usage

**To use this template**:
1. Copy this file to your workspace
2. Replace all `{placeholders}` with actual values
3. Work through checklist sections in order
4. Check off items as completed
5. Archive or delete when release is complete

**Typical timeline**:
- Phases 1-2: 30-60 minutes (manual steps)
- Phase 3: <10 seconds (automated)
- Phases 4-5: 10-15 minutes (verification and cleanup)
