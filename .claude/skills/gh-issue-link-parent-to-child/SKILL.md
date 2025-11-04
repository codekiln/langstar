---
name: gh-issue-link-parent-to-child
description: Link existing GitHub child issues to parent issues, establishing official parent-child sub-task relationships. Use when issues were created separately but need to be connected, or when fixing missing sub-task relationships. Requires both parent and child issue numbers. WARNING - Closes and recreates the child issue.
---

# Link GitHub Child Issues to Parent Issues

Link an existing GitHub issue as an official sub-task of another issue, establishing a proper parent-child relationship.

## ⚠️ IMPORTANT: How This Works

**GitHub's API limitation:** You can ONLY set `parentIssueId` when creating an issue, NOT when updating existing issues.

**What this skill does:**
1. Closes the child issue with an explanatory comment
2. Recreates it with identical content but with `parentIssueId` set
3. Preserves title, body, labels, and assignees
4. Adds a note referencing the original issue number

**Result:** The child issue gets a new issue number but establishes a proper parent-child relationship.

## When to Use This Skill

Use this skill when you need to:

1. **Link existing issues** as sub-tasks (they will be closed and recreated)
2. **Establish parent-child relationships** for issues that were created separately
3. **Fix missing sub-task relationships** when issues weren't properly linked initially
4. **Create two-level hierarchies** (Epic → Phase → Sub-tasks)

## What This Skill Does

- Closes the existing "child" issue with an explanatory comment
- Recreates it as a sub-task of the "parent" issue using GraphQL `createIssue` with `parentIssueId`
- Preserves all content, labels, and assignees from the original issue
- Shows the relationship in GitHub's UI (child appears in parent's "Sub-issues" dropdown)
- Validates both issues exist before making changes

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
# Link issue #103 as a sub-task of issue #92 (closes and recreates #103)
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py --parent 92 --child 103
```

**Output:**
```
Repository: codekiln/langstar
Parent Issue: #92
Child Issue: #103

Fetching repository ID...
Fetching parent issue...
  #92: Phase 3: CLI Commands Implementation
  State: OPEN

Fetching child issue...
  #103: feat(cli): Add deployment management for LangGraph assistants
  State: OPEN

⚠️  WARNING: This will close and recreate the child issue!
    Issue #103 will be closed and a new issue created.
    Content, labels, and assignees will be preserved.

Closing issue #103...
  ✓ Closed issue #103

Creating new issue as sub-task of #92...
  ✓ Created issue #122
  URL: https://github.com/codekiln/langstar/issues/122

✓ Link operation completed successfully
  Old issue #103 closed
  New issue #122 created as sub-task of #92
  View parent: https://github.com/codekiln/langstar/issues/92
  View child: https://github.com/codekiln/langstar/issues/122
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
| **Creates new issues** | ✓ Yes (from scratch) | ✓ Yes (by recreating) |
| **Closes issues** | ✗ No | ✓ Yes (child issue) |
| **Preserves issue numbers** | N/A | ✗ No (gets new number) |
| **Parses task lists** | ✓ Yes | ✗ No |
| **One-to-many linking** | ✓ Yes (creates multiple) | ✗ No (one at a time) |
| **Use case** | Convert task list to sub-issues | Retrofit parent-child relationship |

## When to Use Each Skill

**Use `github-issue-breakdown`:**
- When you have a task list in an issue and want to create sub-issues from it
- When creating a new Epic/parent issue that needs sub-issues
- When you want to generate multiple sub-issues at once
- **PREFERRED:** When planning ahead, always use this skill to avoid recreating issues

**Use `gh-issue-link-parent-to-child`:**
- When you already have issues created but they're not properly linked
- When you need to establish parent-child relationships after the fact
- When you created issues separately and now want to connect them
- When fixing incorrect or missing parent-child relationships
- **CAUTION:** This closes and recreates the child issue (new issue number)

**⚠️ Best Practice:** Plan ahead and use `github-issue-breakdown` instead of creating issues manually, to avoid the need to close and recreate them later.

## Common Workflows

### Workflow 1: Fixing Missing Relationships

1. You created issue #103 separately (without parent relationship)
2. Later realized #103 should be a sub-task of #92
3. Use `gh-issue-link-parent-to-child` to establish the relationship:

```bash
python .claude/skills/gh-issue-link-parent-to-child/scripts/link_issue.py --parent 92 --child 103
```

**Result:** Issue #103 is closed and recreated (e.g., as #122) with parent relationship to #92

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

The skill uses GitHub's GraphQL API `createIssue` mutation with `parentIssueId`:

```graphql
mutation($repoId: ID!, $parentId: ID!, $title: String!, $body: String, $labelIds: [ID!], $assigneeIds: [ID!]) {
  createIssue(input: {
    repositoryId: $repoId
    parentIssueId: $parentId
    title: $title
    body: $body
    labelIds: $labelIds
    assigneeIds: $assigneeIds
  }) {
    issue {
      id
      number
      title
      url
    }
  }
}
```

**Why not `updateIssue`?**
GitHub's GraphQL API does NOT support `parentIssueId` in `updateIssue`. You can only set a parent when creating an issue, not when updating an existing one.

This creates an official parent-child relationship that:
- Shows child in parent's "Sub-issues" dropdown in GitHub UI
- Shows parent reference on the child issue
- Creates proper issue hierarchy for project management
- Allows the child to be closed independently while tracking progress

## See Also

- `github-issue-breakdown` - Create new sub-issues from task lists
- `update-github-issue-project-status` - Update GitHub Projects status fields
