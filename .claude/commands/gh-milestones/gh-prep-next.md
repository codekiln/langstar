---
description: Automate next issue setup in milestone workflow
argument-hint: [milestone-name-or-number]
---

# gh-milestones:prep-next - Move to Next Issue in Milestone

Automates the workflow of moving to the next issue within an active milestone after completing the current task.

## Problem

When working through a milestone with multiple issues and sub-issues, developers currently need to manually:

1. Clean up the completed issue (remove `wip` label)
2. Find the next issue to work on in the milestone
3. Apply the `ready` label to that issue
4. Create a worktree for the new issue
5. Get context on what to work on next

This command automates this entire process.

## Arguments

```text
$ARGUMENTS
```

**Optional:**

- `milestone`: Milestone name or number (if omitted, attempts to detect from current branch/context)

**Examples:**

```bash
# Auto-detect milestone
/gh-milestones:prep-next

# Specify milestone name
/gh-milestones:prep-next "cc-pr-auto"

# Specify milestone number
/gh-milestones:prep-next 10
```

## Implementation

This command delegates to a Python script for robust implementation:

```bash
python3 scripts/gh-milestones/prep-next.py $ARGUMENTS
```

The script performs the following steps:

### Step 1: Detect or Parse Milestone

- Auto-detects milestone from current branch's issue if no argument provided
- Or uses explicit milestone name/number argument
- Validates milestone exists

### Step 2: Fetch All Issues in Milestone

- Retrieves all issues (both open and closed) with the milestone
- Displays summary statistics

### Step 3: Find Last Completed Issue

- Identifies most recently closed issue in milestone
- Removes `wip` label if present

### Step 4: Build Issue Hierarchy

- Uses `gh-sub-issue` extension to map parent-child relationships
- Falls back to simple sequential ordering if extension not available

### Step 5: Find Next Issue (Intelligent Traversal)

- Sibling-first depth-first search when hierarchy available
- Checks for next sibling of last completed issue
- If no sibling, traverses up to parent and finds next sibling at that level
- Handles arbitrary nesting depth
- Falls back to first open issue if no hierarchy match

### Step 6: Update Labels

- Applies `ready` label to next issue

### Step 7: Create Worktree

- Generates branch name following project conventions: `claude/<issue_num>-<slug>`
- Creates worktree at `wip/claude-<issue_num>-<slug>`
- Checks for existing branches/worktrees
- Fetches latest main and creates branch from it
- Updates tmux window name if in tmux session

### Step 8: Display Context

- Shows comprehensive summary with issue details
- Provides clear next steps

## Intelligent Issue Traversal Example

The script uses a depth-first approach when finding the next issue:

```
Milestone: Feature X
├── #100 (closed) - Parent
│   ├── #101 (closed) - Child 1 (LAST COMPLETED)
│   ├── #102 (open) - Child 2 ← NEXT (sibling of last completed)
│   └── #103 (open) - Child 3
└── #200 (open) - Parent 2
```

**Traversal logic:**

1. Start at last completed issue (#101)
2. Check for next sibling sub-issue (#102) ← Found, this is NEXT
3. If no sibling, traverse up to parent (#100) and check for its next sibling
4. Continue up the tree until finding an open issue or reaching milestone root

**Example with grandchild:**

```
Milestone: Feature X
├── #100 (closed) - Parent
│   └── #101 (open) - Child
│       ├── #111 (closed) - Grandchild 1 (LAST COMPLETED)
│       ├── #112 (open) - Grandchild 2 ← NEXT (sibling of last completed)
│       └── #113 (open) - Grandchild 3
```

## Error Handling

### Common Errors

**Milestone not found:**

```
❌ Error: Milestone 'milestone-name' not found
```

- Check milestone name spelling
- List milestones: `gh api repos/$OWNER/$REPO/milestones --jq '.[].title'`

**Could not auto-detect milestone:**

```
❌ Error: Could not auto-detect milestone
   Usage: prep-next.py [milestone-name-or-number]
```

- Provide milestone explicitly as argument
- Ensure current branch is from an issue with a milestone

**Worktree already exists:**

```
⚠️  Worktree already exists at wip/claude-NNN-slug
```

- Remove existing worktree: `git worktree remove wip/claude-NNN-slug`
- Or work in existing worktree

**Branch already exists:**

```
⚠️  Branch claude/NNN-slug already exists
```

- Delete branch locally: `git branch -D claude/NNN-slug`
- Delete remotely: `git push origin --delete claude/NNN-slug`

**No more open issues:**

```
🎉 No more open issues in milestone!
   All issues completed. Milestone ready for release.
```

- This is success! Use `/gh-milestones:release` to mark milestone as done

**gh-sub-issue extension not installed:**

```
⚠️  Warning: gh-sub-issue extension not installed
   Install with: gh extension install https://github.com/dlvhdr/gh-sub-issue
   Falling back to simple sequential ordering...
```

- Install: `gh extension install https://github.com/dlvhdr/gh-sub-issue`
- Script continues with fallback logic (simple sequential ordering)

## Example Workflow

### Example 1: Auto-detect Milestone

```bash
# Currently on branch: claude/373-implement-feature
# Issue #373 is part of milestone "ls-evals-basic"

/gh-milestones:prep-next

# Output:
# 📍 Detected milestone from current branch: ls-evals-basic (#8)
#
# 📋 Fetching issues in milestone...
#    Total issues: 5
#    Open: 2
#    Closed: 3
#
# 🔍 Finding last completed issue...
#    Last completed: #373 - Implement feature
#    Removing 'wip' label from #373...
#
# 🌳 Building issue hierarchy...
#    ✓ Hierarchy built successfully
#
# 🎯 Finding next issue to work on...
#    Next issue: #374 - Add tests for feature
#
# 🏷️  Updating labels...
#    ✓ Added 'ready' label to #374
#
# 📂 Creating worktree for #374...
#    Fetching latest main...
#    ✓ Worktree created successfully
#    ✓ Tmux window renamed to: 💻i374
#
# ============================================================
# ✅ Ready to Start Next Issue
# ============================================================
# [... issue details and next steps ...]
```

### Example 2: Explicit Milestone

```bash
/gh-milestones:prep-next "cc-pr-auto"

# Similar output as Example 1, but explicitly uses "cc-pr-auto" milestone
```

### Example 3: Hierarchical Issue Traversal

Milestone has this structure:

```
Milestone: Feature X
├── #100 (closed) - Parent
│   ├── #101 (closed) - Child 1 (LAST COMPLETED)
│   ├── #102 (open) - Child 2 ← NEXT
│   └── #103 (open) - Child 3
└── #200 (open) - Parent 2
```

Command finds #102 as next issue (sibling of last completed #101).

## Integration with Project Workflow

**Typical workflow:**

1. Complete work on current issue
2. Run `/pr-workflow` to submit PR
3. After PR is merged and issue closes, run `/gh-milestones:prep-next`
4. Command automatically sets up next issue
5. Navigate to worktree and start working
6. Repeat until milestone complete

**When milestone complete:**

```bash
# All issues closed
/gh-milestones:prep-next
# Output: 🎉 No more open issues in milestone!

# Mark milestone as released
/gh-milestones:release "milestone-name" vX.Y.Z
```

## Development Notes

**Script location:** `scripts/gh-milestones/prep-next.py`

The Python implementation provides:

- Better error handling than shell scripts
- Cleaner code organization with classes and functions
- Easier testing and maintenance
- Proper JSON parsing without shell escaping issues
- Type hints for better code clarity

**Dependencies:**

- Python 3.6+
- GitHub CLI (`gh`)
- Git
- `gh-sub-issue` extension (optional, degrades gracefully)

## See Also

- **GitHub Workflow Documentation** - `@docs/dev/github-workflow.md`
- **gh-sub-issue skill** - `.claude/skills/gh-sub-issue/SKILL.md`
- **git-worktrees skill** - `.claude/skills/git-worktrees/SKILL.md`
- **gh-milestones:release command** - `.claude/commands/gh-milestones/release.md`
- **gh-milestones:scout command** - `.claude/commands/gh-milestones/scout.md`
- **pr-workflow command** - `.claude/commands/pr-workflow.md`
