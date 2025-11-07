---
name: git-worktrees
description: Manage git worktrees for parallel feature development following Langstar's issue-driven workflow. Create worktrees with correct target branches based on issue hierarchy, handle cleanup after PR merge, and maintain worktree hygiene. Use when setting up parallel development environments, working on multiple issues simultaneously, or when users mention worktrees, parallel branches, or issue-based development.
---

# Git Worktrees for Issue-Driven Development

Manage git worktrees for parallel feature development in the Langstar project. Create worktrees with correct target branches based on issue hierarchy, handle cleanup after PR merge, and maintain worktree hygiene.

## Overview

Git worktrees enable working on multiple branches simultaneously without switching context. In Langstar's issue-driven workflow, worktrees support parallel development while respecting the hierarchical PR flow where child issues branch from parent issue branches.

**Key Benefits:**
- Work on multiple issues simultaneously without branch switching
- Maintain correct branch hierarchy for sub-issues
- Isolated working directories prevent context confusion
- Clean separation between main worktree and feature branches

## Project Branching Conventions

Understanding Langstar's branching structure is critical for correct worktree usage.

### Branch Naming Convention

Format: `<username>/<issue_num>-<issue_slug>`

**Examples:**
- `alice/7-add-authentication`
- `codekiln/129-gh-sub-issue`
- `claude/132-create-git-worktrees-skill`

### Issue Hierarchy and PR Flow

**Key Rule:** Each PR closes exactly one issue. When issues have parent-child relationships, PRs follow the hierarchy:

```
Issue #333 (parent)
  ↑ PR from: user/333-kingdom
  ↑ Based on: user/666-phylum

Issue #666 (child of #333)
  ↑ PR from: user/666-phylum
  ↑ Based on: user/999-class

Issue #999 (child of #666)
  ↑ PR from: user/999-class
  ↑ Based on: main (or release/v0.2.0)
```

**Implications for Worktrees:**
- Sub-issues must branch from parent issue's feature branch
- Top-level issues branch from main or release branch
- Target branch selection depends on issue hierarchy

## Worktree Location Convention

**Standard location:** `wip/` directory (gitignored)

**Naming pattern:** `wip/<simplified-branch-name>/`

**Example mapping:**
- Branch: `codekiln/130-add-authentication`
- Worktree: `wip/codekiln-130-add-authentication/`

The `wip/` directory is already configured in `.gitignore`, keeping worktrees out of version control.

## Core Commands

### Create Worktree

**Command:** `git worktree add -b <branch-name> <path> <target-branch>`

Creates a new worktree with a new branch based on the specified target branch.

**Template:**
```bash
git worktree add -b <username>/<issue_num>-<issue_slug> wip/<simplified-name> <target-branch>
```

**Example 1: Top-level issue branching from release branch**
```bash
# Issue #130: Add authentication (top-level)
# Target: release/v0.2.0
git worktree add -b codekiln/130-add-authentication wip/codekiln-130-add-authentication release/v0.2.0
```

**Example 2: Sub-issue branching from parent's feature branch**
```bash
# Issue #135: Implement JWT (child of #130)
# Target: codekiln/130-add-authentication (parent branch)
git worktree add -b codekiln/135-implement-jwt wip/codekiln-135-implement-jwt codekiln/130-add-authentication
```

**What it does:**
- Creates new directory at specified path
- Creates new branch from target branch
- Checks out the new branch in the worktree
- Links worktree to main repository

### List Worktrees

**Command:** `git worktree list`

Displays all worktrees with path, commit hash, branch name, and status.

**Usage:**
```bash
git worktree list           # Basic list
git worktree list -v        # Verbose with lock status
git worktree list --porcelain  # Script-friendly format
```

### Remove Worktree

**Command:** `git worktree remove <path>`

Removes a worktree after PR merge. Must be run from outside the worktree being removed.

**Usage:**
```bash
git worktree remove wip/codekiln-130-auth              # Standard
git worktree remove --force wip/codekiln-130-auth     # Force (with changes)
git worktree remove --force --force wip/locked        # Remove locked
```

### Prune Stale Worktrees

**Command:** `git worktree prune`

Cleans up administrative files for manually deleted worktrees.

**Usage:**
```bash
git worktree prune --dry-run    # Preview
git worktree prune --verbose    # Prune with output
```

## Workflows

### Workflow 1: Create Worktree for Top-Level Issue

**Scenario:** Working on issue that branches directly from main or release branch.

**Steps:**

```bash
# Step 1: Identify target branch
# For features: main
# For releases: release/vX.Y.Z
TARGET_BRANCH="release/v0.2.0"

# Step 2: Determine branch name from issue
# Issue #130: "Add user authentication"
BRANCH_NAME="codekiln/130-add-authentication"
WORKTREE_PATH="wip/codekiln-130-add-authentication"

# Step 3: Create worktree
git worktree add -b $BRANCH_NAME $WORKTREE_PATH $TARGET_BRANCH

# Step 4: Navigate to worktree
cd $WORKTREE_PATH

# Step 5: Verify setup
git status
git log -1
```

**Result:** New worktree ready for development on issue #130.

### Workflow 2: Create Worktree for Sub-Issue

**Scenario:** Working on sub-issue that must branch from parent issue's feature branch.

**Steps:**

```bash
# Step 1: Query parent issue
# Issue #135 is child of #130
gh sub-issue list 135 --relation parent

# Output shows: #130 - Add user authentication

# Step 2: Identify parent branch
PARENT_BRANCH="codekiln/130-add-authentication"

# Step 3: Verify parent branch exists locally
git branch --list $PARENT_BRANCH

# If not exists, fetch it
if [ -z "$(git branch --list $PARENT_BRANCH)" ]; then
  git fetch origin $PARENT_BRANCH:$PARENT_BRANCH
fi

# Step 4: Create worktree from parent branch
BRANCH_NAME="codekiln/135-implement-jwt"
WORKTREE_PATH="wip/codekiln-135-implement-jwt"

git worktree add -b $BRANCH_NAME $WORKTREE_PATH $PARENT_BRANCH

# Step 5: Link issues (if not already linked)
gh sub-issue add 130 135

# Step 6: Navigate and verify
cd $WORKTREE_PATH
git log -1
```

**Result:** Worktree for sub-issue #135 correctly branching from parent #130's branch.

### Workflow 3: Cleanup After PR Merge

**Scenario:** PR has been merged, need to clean up worktree.

**Steps:**

```bash
# Step 1: Ensure you're in main worktree
cd /workspace

# Step 2: Pull latest changes from remote
git checkout release/v0.2.0
git pull origin release/v0.2.0

# Step 3: Remove merged worktree
git worktree remove wip/codekiln-130-add-authentication

# Step 4: Delete local branch (optional)
git branch -d codekiln/130-add-authentication

# If branch not fully merged to current HEAD, force delete
# git branch -D codekiln/130-add-authentication

# Step 5: Prune stale references
git worktree prune
```

**Result:** Clean workspace with merged work removed.

### Workflow 4: Handle Stale Worktrees

**Scenario:** Worktree directories deleted manually or PR merged remotely.

**Steps:**

```bash
# Step 1: List all worktrees
git worktree list

# Identify stale worktrees (directories no longer exist)

# Step 2: Preview pruning
git worktree prune --dry-run --verbose

# Step 3: Prune stale references
git worktree prune --verbose

# Step 4: Remove any remaining worktree directories manually
# (if needed)
rm -rf wip/old-worktree-name

# Step 5: Verify cleanup
git worktree list
```

**Result:** All stale worktree references cleaned up.


## Integration with Other Skills

### With `gh-sub-issue` Skill

Use `gh-sub-issue` to determine correct target branches for sub-issues.

**Query parent to find target branch:**
```bash
gh sub-issue list 135 --relation parent
# Output: #130 - Add user authentication
# Branch from: codekiln/130-add-authentication
```

**Create worktree and link:**
```bash
git worktree add -b codekiln/135-jwt wip/codekiln-135-jwt codekiln/130-add-authentication
gh sub-issue add 130 135  # Link if not already linked
```

### With `github-issue-breakdown` Skill

After breaking down epic into sub-issues, create worktrees from parent branch:

```bash
# Epic #100 broken down into #101, #102, #103
git worktree add -b codekiln/100-auth wip/codekiln-100-auth main
git worktree add -b codekiln/101-research wip/codekiln-101-research codekiln/100-auth
git worktree add -b codekiln/102-implement wip/codekiln-102-implement codekiln/100-auth
```

### With Project GitHub Workflow

**PR creation from worktrees:**
```bash
cd wip/codekiln-130-add-authentication
git push -u origin codekiln/130-add-authentication
gh pr create --title "✨ feat: add authentication" --body "Fixes #130"
```

## Best Practices

### Location and Naming

✅ **Always use `wip/` directory**
- Already gitignored
- Consistent location for all worktrees
- Easy to find and manage

✅ **Use simplified naming**
- Remove forward slash from branch name
- Example: `codekiln/130-add-auth` → `wip/codekiln-130-add-auth`

❌ **Avoid**
- Random locations (`../temp`, `~/worktrees/random`)
- Vague names (`wip/test`, `wip/tmp`)

### Target Branch Selection

✅ **Top-level issues:**
- Branch from `main` for regular features
- Branch from `release/vX.Y.Z` for release-specific work

✅ **Sub-issues:**
- Use `gh sub-issue list <issue> --relation parent` to find parent
- Branch from parent issue's feature branch
- Example: `codekiln/135-jwt` branches from `codekiln/130-auth`

❌ **Avoid**
- Guessing parent branch names
- Branching sub-issues from main (breaks PR hierarchy)
- Creating worktrees without verifying target branch exists

### Cleanup Hygiene

✅ **Remove worktrees after PR merge**
```bash
cd /workspace
git worktree remove wip/codekiln-130-add-authentication
```

✅ **Run `git worktree prune` periodically**
```bash
git worktree prune --verbose
```

✅ **Delete branches after merge (optional)**
```bash
git branch -d codekiln/130-add-authentication
```

❌ **Avoid**
- Leaving stale worktrees around indefinitely
- Accumulating dozens of old worktrees
- Manually deleting worktree directories without `git worktree remove`

### Development Workflow

✅ **Switch to main worktree before cleanup**
```bash
cd /workspace  # Main worktree
git worktree remove wip/old-feature
```

✅ **Verify target branch before creating worktree**
```bash
git branch --list codekiln/130-add-authentication
# If empty, fetch it first
```

✅ **Use descriptive paths matching issue numbers**
- Makes it obvious which worktree corresponds to which issue

❌ **Avoid**
- Removing worktrees while inside them (causes errors)
- Creating worktrees from non-existent target branches
- Mixing conventions (some in `wip/`, some elsewhere)

## Troubleshooting

**"fatal: invalid reference"** - Target branch missing. Fetch it: `git fetch origin <branch>:<branch>`

**"fatal: '<path>' already exists"** - Remove existing: `git worktree remove <path>` or `rm -rf <path>`

**"fatal: '<path>' is not a working tree"** - Run `git worktree prune` then `rm -rf <path>`

**"fatal: cannot remove a locked working tree"** - Unlock: `git worktree unlock <path>` or force: `git worktree remove --force --force <path>`

**Directory deleted manually** - Clean up: `git worktree prune --verbose`

**Wrong base branch** - Remove and recreate with correct target from `gh sub-issue list <issue> --relation parent`

## Environment Requirements

**Prerequisites:** Git 2.5+, `wip/` in `.gitignore`, `gh` CLI (for issue queries)

**Verification:** `git --version` and `git check-ignore wip/` (should output: wip/)

## Command Reference

| Command | Purpose |
|---------|---------|
| `git worktree add -b <branch> <path> <target>` | Create worktree |
| `git worktree list [-v]` | List all worktrees |
| `git worktree remove [--force] <path>` | Remove worktree |
| `git worktree prune [--dry-run]` | Clean stale references |
| `git worktree lock/unlock <path>` | Prevent/allow removal |

### Complete Workflow

```bash
# Find parent (for sub-issues)
gh sub-issue list 135 --relation parent

# Create worktree
git worktree add -b codekiln/135-jwt wip/codekiln-135-jwt codekiln/130-auth

# Work and push
cd wip/codekiln-135-jwt
git push -u origin codekiln/135-jwt
gh pr create --title "✨ feat: JWT" --body "Fixes #135"

# After merge
cd /workspace
git worktree remove wip/codekiln-135-jwt
git branch -d codekiln/135-jwt
git worktree prune
```

## See Also

- **gh-sub-issue skill** - Query parent issues for correct target branches
- **github-issue-breakdown skill** - Create sub-issues from task lists
- **GitHub Workflow Documentation** - `@docs/dev/github-workflow.md`
- **Git Worktree Official Docs** - https://git-scm.com/docs/git-worktree
