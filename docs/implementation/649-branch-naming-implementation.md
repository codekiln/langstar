# Issue #649: Branch Naming Convention Update

## Implementation Status

✅ **Complete** - Documentation and script implementation

## Overview

Updated branch naming convention from `<username>/<issue_num>-<slug>` to milestone/parent-aware format:
- `m<milestone_id>-p<parent_issue_id>-i<issue_num>-<slug>` (full)
- `p<parent_issue_id>-i<issue_num>-<slug>` (parent only)
- `m<milestone_id>-i<issue_num>-<slug>` (milestone only)
- `i<issue_num>-<slug>` (standalone)

## Changes Implemented

### Documentation Updates
- ✅ `.claude/commands/` - gh-start-issue.md, pr-workflow.md
- ✅ `.claude/skills/` - pr-lifecycle, git-worktrees
- ✅ `docs/dev/` - github-workflow.md, claude-code-branch-naming.md, README.md
- ✅ Root docs - AGENTS.md, README.md
- ✅ Config files - .github/copilot-instructions.md

### Script Implementation
- ✅ `scripts/cleanup-closed-issue-worktrees.sh`
  - Updated regex to match new `i<number>` pattern
  - Added backward compatibility for old format
  - Updated header comments

- ✅ `scripts/gh-milestones/prep-next.py`
  - Fetch milestone ID from issue
  - Fetch parent issue ID via `gh sub-issue`
  - Generate branch name with appropriate prefixes
  - Update worktree path generation

### Additional Fixes During Review
- ✅ Fixed issue extraction regex in pr-workflow.md
- ✅ Corrected `gh pr checks` field names (state vs conclusion)
- ✅ Clarified parameter meanings (milestone_id, parent_issue_id, issue_num)

## Testing Notes

Script changes are documentation/comment updates and regex enhancements - no impact on test suite. Test failures in #581 were unrelated to this change and have since been resolved.
