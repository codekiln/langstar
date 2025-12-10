# Issue #649 Progress Update

## Status

Creating PR with **documentation changes only**. Script implementation deferred due to test failures.

## Background

While implementing the branch naming convention updates, test failures were discovered that are being addressed in issue #581. See: https://github.com/codekiln/langstar/issues/581#issuecomment-3636154374

## Script Changes (Deferred)

The following script changes have been implemented in this worktree but will NOT be included in this PR:

### `scripts/cleanup-closed-issue-worktrees.sh`
- Updated regex to support new branch naming format
- Added backward compatibility for old format

### `scripts/gh-milestones/prep-next.py`
- Implemented milestone ID detection
- Implemented parent issue ID detection via `gh sub-issue`
- Updated branch name generation logic to use `m<milestone>-p<parent>-i<issue>-<slug>` format

**These changes will be included in a future PR after #581 is resolved.**

## This PR Scope

This PR includes ONLY documentation updates to reflect the new branch naming convention:
- `.claude/` command and skill documentation
- `docs/dev/` developer documentation
- Root-level documentation (README.md, AGENTS.md)
- Configuration files (.cursor/, .github/)

## Next Steps

1. Merge this documentation PR
2. Complete #581 (output format research)
3. Submit separate PR with script implementation
