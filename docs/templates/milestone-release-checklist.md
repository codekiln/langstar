# Milestone Release Checklist (Phase 9)

Reference checklist for releasing a milestone. Most steps are automated via `/ls-release-milestone`.

## Milestone Info

- **Milestone**: {milestone-name}
- **Version**: v{X.Y.Z}
- **Parent Issue**: #{parent-issue-num}

## Pre-Release Validation

### Issue/PR Status

```bash
# Verify all issues closed
gh issue list --milestone "{milestone-name}" --state open
# Expected: no issues

# Verify all PRs merged
gh pr list --search "milestone:{milestone-name}" --state open
# Expected: no PRs
```

### CI Status

```bash
# Verify CI passed on main
gh run list --branch main --limit 1 --json conclusion --jq '.[0].conclusion'
# Expected: success
```

CI automatically runs: `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `cargo audit`, and cross-platform builds.

## Release (Automated)

Version bumping and tagging is handled by the `prepare-release` workflow. Manual bumps only in exceptional cases (see `docs/dev/README.md`).

```bash
gh release create v{X.Y.Z} --generate-notes
```

## Milestone Cleanup (Automated)

```bash
/ls-release-milestone "{milestone-name}" v{X.Y.Z}
```

This command:
- Verifies release exists
- Validates sub-issue completion
- Closes milestone with release link
- Closes parent issue with release comment

Override for open sub-issues: `FORCE_RELEASE=true /ls-release-milestone ...`

## Verification

```bash
# Milestone closed
gh api /repos/{owner}/{repo}/milestones/{num} --jq '.state'
# Expected: closed

# Parent issue closed
gh issue view {parent-issue-num} --json state --jq '.state'
# Expected: CLOSED
```

## Post-Release

```bash
git checkout main && git pull
# If using worktrees:
git worktree remove wip/{branch}
git branch -d {branch}
```

## Troubleshooting

| Error | Solution |
|-------|----------|
| Release not found | Create first: `gh release create v{X.Y.Z}` |
| Open sub-issues | Close them or use `FORCE_RELEASE=true` |
| gh-sub-issue missing | Install: `gh extension install https://github.com/cli/gh-sub-issue` |

## References

- Command: `.claude/commands/ls-release-milestone.md`
- Workflow: `docs/dev/github-workflow.md`
- CI: `.github/workflows/ci.yml`
