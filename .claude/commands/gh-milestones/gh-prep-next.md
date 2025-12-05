---
description: Automate next issue setup in milestone workflow
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

### Step 1: Parse Arguments and Detect Milestone

```bash
!# Get repository info
REPO_INFO=$(gh repo view --json owner,name --jq '{owner: .owner.login, name: .name}')
OWNER=$(echo "$REPO_INFO" | jq -r '.owner')
REPO=$(echo "$REPO_INFO" | jq -r '.name')

# Parse milestone argument
MILESTONE_ARG="${ARGUMENTS:-}"

# If no argument, try to detect from current branch
if [ -z "$MILESTONE_ARG" ]; then
  CURRENT_BRANCH=$(git branch --show-current)
  # Try to extract issue number from branch name (format: user/NNN-slug)
  ISSUE_NUM=$(echo "$CURRENT_BRANCH" | grep -oE '/[0-9]+' | tr -d '/')

  if [ -n "$ISSUE_NUM" ]; then
    # Get milestone from this issue
    MILESTONE_DATA=$(gh issue view "$ISSUE_NUM" --json milestone --jq '.milestone')
    if [ "$MILESTONE_DATA" != "null" ] && [ -n "$MILESTONE_DATA" ]; then
      MILESTONE_TITLE=$(echo "$MILESTONE_DATA" | jq -r '.title')
      MILESTONE_NUM=$(echo "$MILESTONE_DATA" | jq -r '.number')
      echo "📍 Detected milestone from current branch: $MILESTONE_TITLE (#$MILESTONE_NUM)"
    fi
  fi

  # If still not found, error
  if [ -z "$MILESTONE_TITLE" ]; then
    echo "❌ Error: Could not auto-detect milestone"
    echo "   Usage: /gh-milestones:prep-next [milestone-name-or-number]"
    echo "   Or ensure you're on a branch associated with a milestone issue"
    exit 1
  fi
else
  # Milestone explicitly provided
  if [[ "$MILESTONE_ARG" =~ ^[0-9]+$ ]]; then
    # Numeric milestone number
    MILESTONE_NUM="$MILESTONE_ARG"
    MILESTONE_DATA=$(gh api "repos/$OWNER/$REPO/milestones/$MILESTONE_NUM" 2>/dev/null)
    if [ $? -ne 0 ]; then
      echo "❌ Error: Milestone #$MILESTONE_NUM not found"
      exit 1
    fi
    MILESTONE_TITLE=$(echo "$MILESTONE_DATA" | jq -r '.title')
  else
    # Milestone name
    MILESTONE_TITLE="$MILESTONE_ARG"
    MILESTONE_NUM=$(gh api "repos/$OWNER/$REPO/milestones" --jq ".[] | select(.title == \"$MILESTONE_TITLE\") | .number")
    if [ -z "$MILESTONE_NUM" ]; then
      echo "❌ Error: Milestone '$MILESTONE_TITLE' not found"
      exit 1
    fi
  fi

  echo "📍 Milestone: $MILESTONE_TITLE (#$MILESTONE_NUM)"
fi
```

### Step 2: Get All Issues in Milestone

```bash
!echo ""
echo "📋 Fetching issues in milestone..."

# Get all issues with this milestone (both open and closed)
ALL_ISSUES=$(gh issue list --milestone "$MILESTONE_TITLE" --state all --json number,title,state,labels --jq 'sort_by(.number)')

if [ -z "$ALL_ISSUES" ] || [ "$ALL_ISSUES" = "[]" ]; then
  echo "❌ Error: No issues found in milestone '$MILESTONE_TITLE'"
  exit 1
fi

TOTAL_ISSUES=$(echo "$ALL_ISSUES" | jq 'length')
OPEN_ISSUES=$(echo "$ALL_ISSUES" | jq '[.[] | select(.state == "OPEN")] | length')
CLOSED_ISSUES=$(echo "$ALL_ISSUES" | jq '[.[] | select(.state == "CLOSED")] | length')

echo "   Total issues: $TOTAL_ISSUES"
echo "   Open: $OPEN_ISSUES"
echo "   Closed: $CLOSED_ISSUES"
```

### Step 3: Find Last Completed Issue

```bash
!echo ""
echo "🔍 Finding last completed issue..."

# Get most recently closed issue (highest issue number that's closed)
LAST_CLOSED=$(echo "$ALL_ISSUES" | jq -r '[.[] | select(.state == "CLOSED")] | sort_by(.number) | .[-1]')

if [ -z "$LAST_CLOSED" ] || [ "$LAST_CLOSED" = "null" ]; then
  echo "⚠️  No closed issues found in milestone"
  echo "   Starting from first open issue..."
  LAST_CLOSED_NUM=""
else
  LAST_CLOSED_NUM=$(echo "$LAST_CLOSED" | jq -r '.number')
  LAST_CLOSED_TITLE=$(echo "$LAST_CLOSED" | jq -r '.title')
  echo "   Last completed: #$LAST_CLOSED_NUM - $LAST_CLOSED_TITLE"

  # Check if it has wip label
  HAS_WIP=$(echo "$LAST_CLOSED" | jq -r '[.labels[].name] | any(. == "wip")')
  if [ "$HAS_WIP" = "true" ]; then
    echo "   Removing 'wip' label from #$LAST_CLOSED_NUM..."
    gh issue edit "$LAST_CLOSED_NUM" --remove-label "wip" 2>/dev/null || echo "   ⚠️  Could not remove wip label"
  fi
fi
```

### Step 4: Build Parent-Child Relationship Tree

```bash
!echo ""
echo "🌳 Building issue hierarchy..."

# Check if gh-sub-issue extension is available
if ! gh extension list | grep -q "gh-sub-issue"; then
  echo "⚠️  Warning: gh-sub-issue extension not installed"
  echo "   Install with: gh extension install https://github.com/cli/gh-sub-issue"
  echo "   Falling back to simple sequential ordering..."
  USE_SIMPLE_ORDERING=true
else
  USE_SIMPLE_ORDERING=false
fi

# Build parent-child mappings
declare -A PARENT_MAP  # child_num -> parent_num
declare -A CHILDREN_MAP  # parent_num -> space-separated child numbers

if [ "$USE_SIMPLE_ORDERING" = "false" ]; then
  # For each issue, get its parent and children
  for ISSUE_NUM_ITER in $(echo "$ALL_ISSUES" | jq -r '.[].number'); do
    # Get parent
    PARENT_OUTPUT=$(gh sub-issue list "$ISSUE_NUM_ITER" --relation parent --json number 2>/dev/null || echo "[]")
    PARENT_NUM=$(echo "$PARENT_OUTPUT" | jq -r '.[0].number // empty')
    if [ -n "$PARENT_NUM" ]; then
      PARENT_MAP[$ISSUE_NUM_ITER]="$PARENT_NUM"
    fi

    # Get children
    CHILDREN_OUTPUT=$(gh sub-issue list "$ISSUE_NUM_ITER" --relation children --json number,state 2>/dev/null || echo "[]")
    CHILDREN_NUMS=$(echo "$CHILDREN_OUTPUT" | jq -r '.[].number' | tr '\n' ' ')
    if [ -n "$CHILDREN_NUMS" ]; then
      CHILDREN_MAP[$ISSUE_NUM_ITER]="$CHILDREN_NUMS"
    fi
  done

  echo "   ✓ Hierarchy built successfully"
fi
```

### Step 5: Find Next Issue Using Intelligent Traversal

```bash
!echo ""
echo "🎯 Finding next issue to work on..."

if [ "$USE_SIMPLE_ORDERING" = "true" ]; then
  # Simple fallback: just get next open issue after last closed
  if [ -n "$LAST_CLOSED_NUM" ]; then
    NEXT_ISSUE=$(echo "$ALL_ISSUES" | jq -r "[.[] | select(.state == \"OPEN\" and .number > $LAST_CLOSED_NUM)] | sort_by(.number) | .[0]")
  else
    NEXT_ISSUE=$(echo "$ALL_ISSUES" | jq -r '[.[] | select(.state == "OPEN")] | sort_by(.number) | .[0]')
  fi
else
  # Intelligent traversal: sibling-first depth-first search
  NEXT_ISSUE=""

  if [ -n "$LAST_CLOSED_NUM" ]; then
    # Start from last closed issue
    CURRENT_NUM="$LAST_CLOSED_NUM"

    # Step 1: Check for next sibling
    PARENT_NUM="${PARENT_MAP[$CURRENT_NUM]}"
    if [ -n "$PARENT_NUM" ]; then
      # Get all siblings (children of same parent)
      SIBLINGS="${CHILDREN_MAP[$PARENT_NUM]}"
      FOUND_SELF=false
      for SIBLING in $SIBLINGS; do
        if [ "$FOUND_SELF" = "true" ]; then
          # Check if this sibling is open
          SIBLING_STATE=$(echo "$ALL_ISSUES" | jq -r ".[] | select(.number == $SIBLING) | .state")
          if [ "$SIBLING_STATE" = "OPEN" ]; then
            NEXT_ISSUE=$(echo "$ALL_ISSUES" | jq ".[] | select(.number == $SIBLING)")
            break
          fi
        fi
        if [ "$SIBLING" = "$CURRENT_NUM" ]; then
          FOUND_SELF=true
        fi
      done
    fi

    # Step 2: If no next sibling, traverse up and look for parent's next sibling
    if [ -z "$NEXT_ISSUE" ] && [ -n "$PARENT_NUM" ]; then
      GRANDPARENT_NUM="${PARENT_MAP[$PARENT_NUM]}"
      if [ -n "$GRANDPARENT_NUM" ]; then
        # Get parent's siblings
        PARENT_SIBLINGS="${CHILDREN_MAP[$GRANDPARENT_NUM]}"
        FOUND_PARENT=false
        for UNCLE in $PARENT_SIBLINGS; do
          if [ "$FOUND_PARENT" = "true" ]; then
            UNCLE_STATE=$(echo "$ALL_ISSUES" | jq -r ".[] | select(.number == $UNCLE) | .state")
            if [ "$UNCLE_STATE" = "OPEN" ]; then
              NEXT_ISSUE=$(echo "$ALL_ISSUES" | jq ".[] | select(.number == $UNCLE)")
              break
            fi
          fi
          if [ "$UNCLE" = "$PARENT_NUM" ]; then
            FOUND_PARENT=true
          fi
        done
      fi
    fi
  fi

  # Step 3: Fallback - just get first open issue
  if [ -z "$NEXT_ISSUE" ] || [ "$NEXT_ISSUE" = "null" ]; then
    NEXT_ISSUE=$(echo "$ALL_ISSUES" | jq -r '[.[] | select(.state == "OPEN")] | sort_by(.number) | .[0]')
  fi
fi

if [ -z "$NEXT_ISSUE" ] || [ "$NEXT_ISSUE" = "null" ]; then
  echo "🎉 No more open issues in milestone!"
  echo "   All issues completed. Milestone ready for release."
  exit 0
fi

NEXT_NUM=$(echo "$NEXT_ISSUE" | jq -r '.number')
NEXT_TITLE=$(echo "$NEXT_ISSUE" | jq -r '.title')

echo "   Next issue: #$NEXT_NUM - $NEXT_TITLE"
```

### Step 6: Apply Ready Label to Next Issue

```bash
!echo ""
echo "🏷️  Updating labels..."

# Add 'ready' label to next issue
gh issue edit "$NEXT_NUM" --add-label "ready" 2>/dev/null && \
  echo "   ✓ Added 'ready' label to #$NEXT_NUM" || \
  echo "   ⚠️  Could not add 'ready' label (may not exist in repo)"
```

### Step 7: Create Worktree for Next Issue

```bash
!echo ""
echo "📂 Creating worktree for #$NEXT_NUM..."

# Generate branch name and worktree path following project conventions
ISSUE_SLUG=$(echo "$NEXT_TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9 -]//g' | tr -s ' ' '-' | sed 's/^-//;s/-$//' | cut -c1-50)
if [ -z "$ISSUE_SLUG" ]; then
  ISSUE_SLUG="issue"
fi

USERNAME="claude"
BRANCH_NAME="${USERNAME}/${NEXT_NUM}-${ISSUE_SLUG}"
WORKTREE_PATH="wip/${USERNAME}-${NEXT_NUM}-${ISSUE_SLUG}"

# Check if worktree already exists
if [ -d "$WORKTREE_PATH" ]; then
  echo "⚠️  Worktree already exists at $WORKTREE_PATH"
  echo "   To recreate:"
  echo "     git worktree remove $WORKTREE_PATH"
  echo "     /gh-milestones:prep-next"
  exit 1
fi

# Check if branch already exists
if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME" || git ls-remote --heads origin "$BRANCH_NAME" | grep -q "$BRANCH_NAME"; then
  echo "⚠️  Branch $BRANCH_NAME already exists"
  echo "   To delete:"
  echo "     git branch -D $BRANCH_NAME  # local"
  echo "     git push origin --delete $BRANCH_NAME  # remote"
  exit 1
fi

# Fetch latest main
echo "   Fetching latest main..."
git fetch origin main

# Create worktree
if git worktree add -b "$BRANCH_NAME" "$WORKTREE_PATH" "origin/main"; then
  echo "   ✓ Worktree created successfully"
else
  echo "❌ Error: Failed to create worktree"
  exit 1
fi

# Update tmux window name if in tmux
if [ -n "$TMUX" ]; then
  TMUX_NAME="💻i${NEXT_NUM}"
  tmux rename-window "$TMUX_NAME" 2>/dev/null && \
    echo "   ✓ Tmux window renamed to: $TMUX_NAME"
fi
```

### Step 8: Display Issue Context and Next Steps

```bash
!echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Ready to Start Next Issue"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📍 Milestone: $MILESTONE_TITLE (#$MILESTONE_NUM)"
echo "📋 Issue: #$NEXT_NUM - $NEXT_TITLE"
echo "🌿 Branch: $BRANCH_NAME"
echo "📂 Worktree: $WORKTREE_PATH"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Issue Details"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
gh issue view "$NEXT_NUM" --json body --jq '.body // "(No description provided)"'
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Next Steps"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Navigate to worktree:"
echo "   cd $WORKTREE_PATH"
echo ""
echo "2. Review issue requirements and implement changes"
echo ""
echo "3. When ready to submit:"
echo "   /pr-workflow"
echo ""
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
   Usage: /gh-milestones:prep-next [milestone-name-or-number]
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
   Falling back to simple sequential ordering...
```
- Install: `gh extension install https://github.com/cli/gh-sub-issue`
- Command continues with fallback logic (simple sequential ordering)

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
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ✅ Ready to Start Next Issue
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
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

## See Also

- **GitHub Workflow Documentation** - `@docs/dev/github-workflow.md`
- **gh-sub-issue skill** - `.claude/skills/gh-sub-issue/SKILL.md`
- **git-worktrees skill** - `.claude/skills/git-worktrees/SKILL.md`
- **gh-milestones:release command** - `.claude/commands/gh-milestones/release.md`
- **gh-milestones:scout command** - `.claude/commands/gh-milestones/scout.md`
- **pr-workflow command** - `.claude/commands/pr-workflow.md`
