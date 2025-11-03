---
name: gh-issue-link-parent-to-child
description: Link existing GitHub child issues to parent issues, establishing official parent-child sub-task relationships. Use when issues were created separately but need to be connected, or when fixing missing sub-task relationships. Requires both parent and child issue numbers.
---

# Link GitHub Child Issues to Parent Issues

Link an existing GitHub issue as an official sub-task of another issue, establishing a proper parent-child relationship.

## When to Use This Skill

Use this skill when you need to:

1. **Link existing issues** as sub-tasks without creating new issues
2. **Establish parent-child relationships** for issues that were created separately
3. **Fix missing sub-task relationships** when issues weren't properly linked initially
4. **Create two-level hierarchies** (Epic → Phase → Sub-tasks)

## What This Skill Does

- Links an existing "child" issue as a tracked sub-issue of a "parent" issue
- Uses GitHub's official GraphQL API to create proper parent-child relationships
- Shows the relationship in GitHub's UI (child appears in parent's "Tracked by" section)
- Validates both issues exist before attempting to link them

## Usage

### Basic Usage

```bash
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py --parent 92 --child 103
```

### With Repository Specification

```bash
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py \
  --parent 92 \
  --child 103 \
  --repo owner/repo
```

### Dry Run (Preview Without Changes)

```bash
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py \
  --parent 92 \
  --child 103 \
  --dry-run
```

## Parameters

- `--parent` (required): Issue number of the parent issue (the one that will track)
- `--child` (required): Issue number of the child issue (the sub-task to be tracked)
- `--repo` (optional): Repository in `owner/name` format (auto-detected from git remote if omitted)
- `--dry-run` (optional): Show what would happen without making changes

## Environment Variables

Requires GitHub authentication:

- `GITHUB_TOKEN` or `GH_TOKEN` - GitHub personal access token with `repo` scope

## Examples

### Example 1: Link Sub-Task to Phase

```bash
# Link issue #103 as a sub-task of issue #92
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py --parent 92 --child 103
```

**Output:**
```
Repository: codekiln/langstar
Parent Issue: #92
Child Issue: #103

Fetching parent issue...
  #92: Phase 3: CLI Commands Implementation
  State: OPEN

Fetching child issue...
  #103: feat(cli): Add deployment management for LangGraph assistants
  State: OPEN

Linking issues...
✓ Successfully linked issues
  Parent: #92 - Phase 3: CLI Commands Implementation
  Tracked issues:
    - #103: feat(cli): Add deployment management for LangGraph assistants

✓ Link operation completed successfully
  View parent issue: https://github.com/codekiln/langstar/issues/92
  View child issue: https://github.com/codekiln/langstar/issues/103
```

### Example 2: Preview Before Linking

```bash
# Preview what will happen
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py \
  --parent 92 \
  --child 103 \
  --dry-run
```

## Differences from github-issue-breakdown

| Feature | github-issue-breakdown | gh-issue-link-parent-to-child |
|---------|----------------------|-------------------|
| **Creates new issues** | ✓ Yes | ✗ No |
| **Links existing issues** | ✗ No | ✓ Yes |
| **Parses task lists** | ✓ Yes | ✗ No |
| **One-to-many linking** | ✓ Yes (creates multiple) | ✗ No (one at a time) |
| **Use case** | Convert task list to sub-issues | Link pre-existing issues |

## When to Use Each Skill

**Use `github-issue-breakdown`:**
- When you have a task list in an issue and want to create sub-issues from it
- When creating a new Epic/parent issue that needs sub-issues
- When you want to generate multiple sub-issues at once

**Use `gh-issue-link-parent-to-child`:**
- When you already have issues created but they're not properly linked
- When you need to establish parent-child relationships after the fact
- When you created issues separately and now want to connect them
- When fixing incorrect or missing parent-child relationships

## Common Workflows

### Workflow 1: Fixing Missing Relationships

1. You created issue #103 separately
2. Later realized #103 should be a sub-task of #92
3. Use `gh-issue-link-parent-to-child` to establish the relationship:

```bash
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py --parent 92 --child 103
```

### Workflow 2: Creating Two-Level Hierarchy

1. Epic #83 tracks multiple phases (#90-95)
2. Phase #92 needs its own sub-tasks (#102, #103)
3. Link each sub-task to the phase:

```bash
# Link #102 to #92 (if #102 were an issue)
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py --parent 92 --child 102

# Link #103 to #92
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py --parent 92 --child 103
```

Result: Epic #83 → Phase #92 → Sub-tasks #102, #103

## Error Handling

The script will exit with an error if:

- GitHub token is not found in environment
- Repository cannot be detected or is invalid
- Parent issue doesn't exist
- Child issue doesn't exist
- GraphQL mutation fails (e.g., permissions issue)

## Requirements

- **gh CLI**: GitHub CLI tool installed and authenticated
- **Python 3.6+**: Script uses modern Python features
- **GitHub Token**: With `repo` scope for accessing private repositories

## Troubleshooting

### "Could not detect repository"

**Cause:** Not running from within a git repository, or no `origin` remote.

**Solution:** Either:
- Run from within your git repository, or
- Specify repository explicitly: `--repo owner/name`

### "Error: No GitHub token found"

**Cause:** `GITHUB_TOKEN` or `GH_TOKEN` not set.

**Solution:**
```bash
export GITHUB_TOKEN=ghp_your_token_here
# Or use gh CLI's authentication
gh auth login
```

### "Could not fetch issue #N"

**Cause:** Issue doesn't exist, or you don't have access to it.

**Solution:**
- Verify issue number is correct
- Check you have access to the repository
- Ensure issue hasn't been deleted

## Implementation Details

The skill uses GitHub's GraphQL API mutation:

```graphql
mutation($parentId: ID!, $childId: ID!) {
  updateIssue(input: {
    id: $parentId
    trackedIssueIds: [$childId]
  }) {
    issue {
      number
      title
      trackedIssues(first: 10) {
        nodes {
          number
          title
        }
      }
    }
  }
}
```

This creates an official parent-child relationship that:
- Shows child in parent's "Tracked by" section in GitHub UI
- Updates parent's task list checkboxes when child is closed
- Creates proper issue hierarchy for project management

## See Also

- `github-issue-breakdown` - Create new sub-issues from task lists
- `update-github-issue-project-status` - Update GitHub Projects status fields
