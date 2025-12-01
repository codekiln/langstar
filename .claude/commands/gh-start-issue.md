---
description: Create a git worktree from main for a GitHub issue branch and get started on the work
argument-hint: <issue_number>
allowed-tools: [Bash, Read, Write, Glob, Grep]
---

# gh-start-issue - Automated Issue Workflow

Automates the workflow for starting work on a GitHub issue by creating a worktree, updating the tmux window name, and setting up context.

## Arguments

Arguments are passed via `$ARGUMENTS` in the format:
```
<issue_number>
```

**Required:**
- `issue_number`: The GitHub issue number (integer)

## Example Usage

```bash
# Start work on issue #42
/gh-start-issue 42

# Start work on issue #123
/gh-start-issue 123
```

## User Input

```text
$ARGUMENTS
```

## Implementation

**Step 1:** Validate issue number and fetch details

```bash
!ISSUE_NUM="$ARGUMENTS"

# Validate issue number format
if [ -z "$ISSUE_NUM" ] || ! [[ "$ISSUE_NUM" =~ ^[0-9]+$ ]]; then
  echo "❌ Error: Please provide a valid issue number"
  echo "Usage: /gh-start-issue <issue_number>"
  exit 1
fi

# Fetch issue details and validate existence
ISSUE_DATA=$(gh issue view "$ISSUE_NUM" --json title,body,state,milestone 2>/dev/null)
if [ $? -ne 0 ]; then
  echo "❌ Error: Issue #$ISSUE_NUM not found or you don't have access"
  exit 1
fi

ISSUE_TITLE=$(echo "$ISSUE_DATA" | jq -r .title)
ISSUE_STATE=$(echo "$ISSUE_DATA" | jq -r .state)

if [ "$ISSUE_STATE" = "CLOSED" ]; then
  echo "⚠️  Warning: Issue #$ISSUE_NUM is already closed"
fi

echo "📋 Issue #$ISSUE_NUM: $ISSUE_TITLE"
```

**Step 2:** Determine target branch (check for parent issues)

```bash
!# Check if issue has a parent (for hierarchical branching)
TARGET_BRANCH="main"
PARENT_OUTPUT=$(gh sub-issue list "$ISSUE_NUM" --relation parent 2>/dev/null || echo "")

if [ -n "$PARENT_OUTPUT" ] && ! echo "$PARENT_OUTPUT" | grep -q "No parent issue found"; then
  echo "🔍 Detected parent issue - this is a sub-issue"
  echo "$PARENT_OUTPUT"
  echo "⚠️  Note: You may want to branch from the parent's branch instead of main"
  echo "ℹ️  Defaulting to main for now (you can manually change if needed)"
fi

# Fetch latest main
echo "📥 Fetching latest $TARGET_BRANCH..."
git fetch origin "$TARGET_BRANCH"
```

**Step 3:** Generate branch name following project conventions

```bash
!# Create slug from title (lowercase, replace spaces/special chars with hyphens)
ISSUE_SLUG=$(echo "$ISSUE_TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9 -]//g' | tr -s ' ' '-' | sed 's/^-//;s/-$//' | cut -c1-50)
if [ -z "$ISSUE_SLUG" ]; then
  ISSUE_SLUG="issue"
fi

# Use 'claude' as username for Claude Code
USERNAME="claude"

# Branch: <username>/<issue_num>-<issue_slug>
BRANCH_NAME="${USERNAME}/${ISSUE_NUM}-${ISSUE_SLUG}"

# Worktree: wip/<username>-<issue_num>-<issue_slug>
WORKTREE_PATH="wip/${USERNAME}-${ISSUE_NUM}-${ISSUE_SLUG}"

echo "🌿 Branch: $BRANCH_NAME"
echo "📂 Worktree: $WORKTREE_PATH"
```

**Step 4:** Create worktree

```bash
!# Check if worktree directory already exists
if [ -d "$WORKTREE_PATH" ]; then
  echo "❌ Error: Worktree already exists at $WORKTREE_PATH"
  echo "Remove it first with: git worktree remove $WORKTREE_PATH"
  exit 1
fi

# Check if branch already exists locally or remotely
if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME" || git ls-remote --heads origin "$BRANCH_NAME" | grep -q "$BRANCH_NAME"; then
  echo "❌ Error: Branch $BRANCH_NAME already exists"
  echo "To delete it:"
  echo "  Local:  git branch -D $BRANCH_NAME"
  echo "  Remote: git push origin --delete $BRANCH_NAME"
  exit 1
fi

# Create new worktree with new branch from target
echo "🔧 Creating worktree from $TARGET_BRANCH..."
if ! git worktree add -b "$BRANCH_NAME" "$WORKTREE_PATH" "origin/$TARGET_BRANCH"; then
  echo "❌ Error: Failed to create worktree"
  exit 1
fi

echo "✅ Worktree created successfully"
```

**Step 5:** Update tmux window name (if in tmux)

```bash
!# Update tmux window name if in tmux session
# Format: <emoji><issue_num> where emoji indicates phase:
# 💻 = coding (initial phase when starting issue)
# See pr-lifecycle skill for all phase emojis
if [ -n "$TMUX" ]; then
  TMUX_NAME="💻${ISSUE_NUM}"
  tmux rename-window "$TMUX_NAME" 2>/dev/null && echo "✅ Tmux window renamed to: $TMUX_NAME" || echo "ℹ️  Could not rename tmux window"
else
  echo "ℹ️  Not in tmux session - skipping window rename"
fi
```

**Step 6:** Display issue context and next steps

```bash
!echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Issue #$ISSUE_NUM: $ISSUE_TITLE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Display issue body
echo "$ISSUE_DATA" | jq -r '.body // "(No description provided)"'

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Ready to Start"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📂 Worktree: $WORKTREE_PATH"
echo "🌿 Branch: $BRANCH_NAME"
echo "🎯 Target: $TARGET_BRANCH"
echo ""
echo "Next steps:"
echo "  1. Navigate: cd $WORKTREE_PATH"
echo "  2. Implement changes per the issue requirements"
echo "  3. Use /pr-workflow when ready to create PR and submit"
echo ""
```

## Next Steps After Running

After this command completes:

1. **Navigate to worktree**: `cd <worktree_path>`
2. **Implement changes**: Make code changes for the issue
3. **Commit changes**: Follow conventional commit format
4. **Push branch**: `git push -u origin <branch_name>`
5. **Create PR**: Use `/pr-workflow` or `gh pr create`

## Error Handling

Common errors and solutions:

- **Issue not found**: Verify the issue number exists
- **Worktree already exists**: Choose to delete/recreate or use existing
- **Branch already exists**: Choose to use existing or delete first
- **Git fetch fails**: Check network connection and repository access
- **Permission denied**: Verify git credentials and repository access

## Integration with Other Commands

**Use with `/pr-workflow`**: After implementing changes in the worktree, use `/pr-workflow` to automate PR creation and CI monitoring.

**Use with `git-worktrees` skill**: For more advanced worktree management and cleanup.

## See Also

- **git-worktrees skill** - `.claude/skills/git-worktrees/SKILL.md`
- **pr-workflow command** - `.claude/commands/pr-workflow.md`
- **GitHub Workflow Documentation** - `@docs/dev/github-workflow.md`
- **Branch Naming Convention** - `@docs/dev/github-workflow.md#branch-naming-convention`
