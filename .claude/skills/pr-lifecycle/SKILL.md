---
name: pr-lifecycle
description: Enforce project hygiene throughout the PR lifecycle. Use before creating PRs to validate worktrees, branch naming, and issue linking. Use when creating PRs to ensure proper "Fixes #XYZ" keywords, conventional commit titles, and complete PR bodies. Use after PR merge for cleanup guidance and issue closure verification.
---

# PR Lifecycle Management

Enforce project hygiene throughout the pull request lifecycle. This skill provides checklists, validation commands, and templates to ensure PRs properly close issues and follow project conventions.

## Overview

**Project Hygiene Invariant:** Each PR must:
1. Close exactly one GitHub issue
2. Include "Fixes #XYZ" (or similar keyword) in PR body
3. Be created from a proper worktree (not main)
4. Follow branch naming convention: `<username>/<issue_num>-<issue_slug>`
5. Use Conventional Emoji Commits for PR title

**Why This Matters:** PRs #221 and #222 didn't include "Fixes #XYZ" keywords, causing issues to remain open after merge. This skill prevents such "orphan PRs."

## Phase 1: Before Creating PR

### Pre-PR Checklist

Run these validations before creating a PR:

```bash
# 1. Verify you're in a worktree (not main)
git worktree list | grep "$(pwd)"

# 2. Check branch name follows convention
BRANCH=$(git branch --show-current)
echo "Current branch: $BRANCH"
# Should match: <username>/<issue_num>-<issue_slug>

# 3. Extract issue number from branch
ISSUE_NUM=$(echo "$BRANCH" | grep -oP '\d+' | head -1)
echo "Issue number: $ISSUE_NUM"

# 4. Verify issue exists and is open
gh issue view "$ISSUE_NUM" --json state,title

# 5. Check commits for "Fixes #" keyword
git log origin/main..HEAD --oneline | grep -i "fixes #\|closes #\|resolves #" || echo "WARNING: No 'Fixes #' keyword found in commits"
```

### Validation Details

#### Worktree Verification

**Why:** Feature work should happen in worktrees, not the main worktree.

```bash
# List all worktrees
git worktree list

# Verify current directory is in wip/
pwd | grep -q "wip/" && echo "In worktree" || echo "WARNING: Not in wip/ worktree"
```

**Expected:** You should be in a `wip/<branch-name>/` directory.

#### Branch Naming Convention

**Format:** `<username>/<issue_num>-<issue_slug>`

**Examples:**
- `alice/42-add-authentication`
- `claude/225-pr-lifecycle-skill`
- `codekiln/130-add-user-profile`

**Validation:**
```bash
BRANCH=$(git branch --show-current)

# Check format (username/number-slug)
if [[ "$BRANCH" =~ ^[a-zA-Z0-9_-]+/[0-9]+-[a-zA-Z0-9_-]+$ ]]; then
  echo "Branch name follows convention"
else
  echo "WARNING: Branch name '$BRANCH' doesn't follow convention"
fi
```

#### Issue Verification

**Check issue exists and is open:**
```bash
ISSUE_NUM=$(git branch --show-current | grep -oP '\d+' | head -1)

# View issue details
gh issue view "$ISSUE_NUM" --json number,title,state,labels

# Verify it's open
STATE=$(gh issue view "$ISSUE_NUM" --json state -q '.state')
if [ "$STATE" = "OPEN" ]; then
  echo "Issue #$ISSUE_NUM is open"
else
  echo "WARNING: Issue #$ISSUE_NUM is $STATE"
fi
```

#### Commit Message Check

**Verify commits reference the issue:**
```bash
# Check for GitHub closing keywords in commits
git log origin/main..HEAD --pretty=format:"%s" | \
  grep -iE "(fix(es)?|close[sd]?|resolve[sd]?)\s*#\d+" || \
  echo "WARNING: No GitHub closing keywords found in commit messages"
```

**GitHub closing keywords:** `close`, `closes`, `closed`, `fix`, `fixes`, `fixed`, `resolve`, `resolves`, `resolved`

## Phase 2: Creating PR

### PR Title Convention

**Format:** `<emoji> <type>[scope]: <description>`

**Common Types:**
| Type | Emoji | Use For |
|------|-------|---------|
| feat | ✨ | New features |
| fix | 🩹 | Bug fixes |
| docs | 📚 | Documentation |
| refactor | ♻️ | Code refactoring |
| test | 🧪 | Tests |
| build | 🔧 | Build system |
| perf | ⚡️ | Performance |
| release | 🔖 | Releases |

**Examples:**
- `✨ feat(cli): add deployment management commands`
- `🩹 fix: resolve race condition in thread handler`
- `📚 docs: add PR lifecycle skill`

### PR Body Template

**Always include "Fixes #XYZ" to auto-close the issue:**

```markdown
## Summary
<1-3 bullet points describing the changes>

## Changes
- <specific change 1>
- <specific change 2>

## Related Issues
Fixes #<issue_number>

## Test Plan
- [ ] <test item 1>
- [ ] <test item 2>

---
Generated with [Claude Code](https://claude.ai/code)
```

### Create PR Command

**Using gh CLI with proper body:**
```bash
ISSUE_NUM=$(git branch --show-current | grep -oP '\d+' | head -1)

gh pr create \
  --title "✨ feat: <description>" \
  --body "$(cat <<EOF
## Summary
- <summary point>

## Changes
- <change 1>
- <change 2>

## Related Issues
Fixes #$ISSUE_NUM

## Test Plan
- [ ] <test item>

---
Generated with [Claude Code](https://claude.ai/code)
EOF
)"
```

### Verify PR Will Close Issue

**After creating PR, verify the link:**
```bash
# Get PR number
PR_NUM=$(gh pr view --json number -q '.number')

# Check closing issues
gh pr view "$PR_NUM" --json closingIssuesReferences -q '.closingIssuesReferences[].number'

# Should output the issue number
```

## Phase 3: After PR Creation (Optional)

### Monitor for Automated Reviews

Copilot and other automated reviewers may add comments. Monitor and address them:

```bash
PR_NUM=$(gh pr view --json number -q '.number')

# Check for Copilot comments
gh api repos/{owner}/{repo}/pulls/$PR_NUM/comments \
  --jq '.[] | select(.user.login == "copilot") | {id, body, path, line}'

# Check for all review comments
gh api repos/{owner}/{repo}/pulls/$PR_NUM/comments \
  --jq '.[] | {id, user: .user.login, body: .body[0:100]}'
```

### Reply to Review Comments

**After addressing feedback, reply with commit reference:**

```bash
COMMENT_ID=<comment_id>
COMMIT_SHA=$(git rev-parse --short HEAD)

gh api repos/{owner}/{repo}/pulls/$PR_NUM/comments/$COMMENT_ID/replies \
  -f body="Fixed in commit $COMMIT_SHA: <brief description>"
```

### Check PR Status

```bash
PR_NUM=$(gh pr view --json number -q '.number')

# View PR checks
gh pr checks "$PR_NUM"

# View PR status
gh pr view "$PR_NUM" --json state,mergeable,reviewDecision
```

## Phase 4: After PR Merge

### Verify Issue Closure

**Check if the issue was auto-closed:**
```bash
ISSUE_NUM=<issue_number>

# Check issue state
STATE=$(gh issue view "$ISSUE_NUM" --json state -q '.state')
echo "Issue #$ISSUE_NUM state: $STATE"

if [ "$STATE" = "CLOSED" ]; then
  echo "Issue #$ISSUE_NUM was auto-closed"
else
  echo "WARNING: Issue #$ISSUE_NUM is still OPEN"
  echo "Manually close with: gh issue close $ISSUE_NUM"
fi
```

### Manual Issue Closure (If Needed)

**If the issue wasn't auto-closed:**
```bash
ISSUE_NUM=<issue_number>
PR_NUM=<pr_number>

# Close with reference to PR
gh issue close "$ISSUE_NUM" --comment "Closed via PR #$PR_NUM"
```

### Cleanup: Remove Worktree

**After PR merge, clean up the worktree:**
```bash
# Switch to main worktree first
cd /workspace

# Remove the worktree
WORKTREE_PATH="wip/<branch-name>"
git worktree remove "$WORKTREE_PATH"

# Prune stale references
git worktree prune --verbose
```

### Cleanup: Delete Branch

**Delete local and remote branches:**
```bash
BRANCH="<username>/<issue_num>-<slug>"

# Delete local branch
git branch -d "$BRANCH"

# If not fully merged, force delete
# git branch -D "$BRANCH"

# Delete remote branch (GitHub usually does this automatically)
git push origin --delete "$BRANCH" 2>/dev/null || echo "Remote branch already deleted"
```

### Complete Cleanup Workflow

```bash
# Variables
ISSUE_NUM=225
BRANCH="claude/225-pr-lifecycle-skill"
WORKTREE_PATH="wip/claude-225-pr-lifecycle-skill"

# 1. Verify issue closed
gh issue view "$ISSUE_NUM" --json state -q '.state'

# 2. Switch to main worktree
cd /workspace

# 3. Pull latest changes
git checkout main
git pull origin main

# 4. Remove worktree
git worktree remove "$WORKTREE_PATH"

# 5. Delete local branch
git branch -d "$BRANCH"

# 6. Prune
git worktree prune --verbose

# 7. Verify cleanup
git worktree list
git branch | grep -v "^\*"
```

## Quick Reference

### Pre-PR Checklist

| Check | Command | Expected |
|-------|---------|----------|
| In worktree | `pwd \| grep wip/` | In wip/ directory |
| Branch format | `git branch --show-current` | `user/num-slug` |
| Issue open | `gh issue view N --json state` | `OPEN` |
| Has "Fixes #" | `git log \| grep -i "fixes #"` | Found keyword |

### GitHub Closing Keywords

Any of these in PR body will auto-close the linked issue:
- `close`, `closes`, `closed`
- `fix`, `fixes`, `fixed`
- `resolve`, `resolves`, `resolved`

**Format:** `Fixes #123` or `Fixes owner/repo#123`

### PR Title Emojis

| Emoji | Type | Triggers |
|-------|------|----------|
| ✨ | feat | MINOR bump |
| 🩹 | fix | PATCH bump |
| 🚨 | BREAKING | MAJOR bump |
| 📚 | docs | No bump |
| ♻️ | refactor | No bump |
| 🧪 | test | No bump |
| 🔧 | build | No bump |
| 🔖 | release | N/A |

### Common API Commands

```bash
# View issue
gh issue view <num> --json number,title,state

# View PR
gh pr view --json number,title,state,closingIssuesReferences

# PR comments (for reviews)
gh api repos/{owner}/{repo}/pulls/<num>/comments

# Reply to comment
gh api repos/{owner}/{repo}/pulls/<num>/comments/<id>/replies -f body="message"

# Close issue manually
gh issue close <num> --comment "Closed via PR #N"
```

## Integration with Other Skills

### With `git-worktrees` Skill

Use git-worktrees skill to create proper worktrees before starting work:
```bash
# Create worktree for new issue
git worktree add -b alice/42-new-feature wip/alice-42-new-feature main
```

### With `gh-sub-issue` Skill

For issues with parent-child relationships, verify PR targets:
```bash
# Check if issue has parent
gh sub-issue list 42 --relation parent

# If parent exists, PR should target parent branch, not main
```

## Troubleshooting

### "Issue not closed after merge"

**Cause:** Missing "Fixes #N" keyword in PR body.

**Solution:**
```bash
# Close manually
gh issue close <num> --comment "Closed via PR #<pr_num>"

# Prevent future issues: always include "Fixes #N" in PR body
```

### "Branch name doesn't match issue"

**Cause:** Branch created without following convention.

**Solution:** Rename branch before PR:
```bash
git branch -m old-name user/42-proper-name
git push origin -u user/42-proper-name
git push origin --delete old-name
```

### "Can't determine issue number"

**Cause:** Branch name doesn't contain issue number.

**Solution:** Check branch name, rename if needed:
```bash
git branch --show-current
# If no number, rename branch to include issue number
```

### "PR shows wrong closing issues"

**Cause:** Wrong issue number in PR body.

**Solution:** Edit PR body on GitHub or:
```bash
gh pr edit <num> --body "$(cat updated-body.md)"
```

## See Also

- **git-worktrees skill** - Create and manage worktrees for issue branches
- **gh-sub-issue skill** - Manage parent-child issue relationships
- **GitHub Workflow Documentation** - `@docs/dev/github-workflow.md`
- **Git SCM Conventions** - `@docs/dev/git-scm-conventions.md`
- **GitHub Keywords Docs** - https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue
