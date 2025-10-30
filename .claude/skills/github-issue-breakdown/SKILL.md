---
name: github-issue-breakdown
description: Break down GitHub issues into official GitHub sub-issues by parsing task lists from parent issue descriptions. Use this skill when a user wants to convert a parent issue with tasks into linked sub-issues, when managing complex features that need hierarchical breakdown, or when converting Spec-Kit task lists into GitHub sub-issues. Accepts issue number as parameter or prompts user if not provided.
---

# GitHub Issue Breakdown

## Overview

This skill automates breaking down GitHub issues into official GitHub sub-issues using the GitHub GraphQL API. It parses various task list formats from parent issue descriptions, shows a preview of proposed sub-issues, and creates them with proper parent-child relationships while inheriting metadata like labels and assignees.

## When to Use This Skill

Use this skill when:
- User requests breaking down an issue into sub-tasks (e.g., "break down issue #47 into sub-issues")
- User mentions converting a task list into sub-issues
- User is working with complex features that need hierarchical task management
- User wants to convert Spec-Kit generated task lists into GitHub sub-issues
- User needs to create multiple related child issues from a parent issue

**Invocation patterns:**
- "Break down issue #42 into sub-tasks"
- "Create sub-issues from issue 47"
- "Convert the task list in issue #42 to sub-issues"
- "Use github-issue-breakdown on issue 42"

## Quick Start

### Prerequisites

1. **Environment Variables:**
   ```bash
   export GITHUB_TOKEN="ghp_xxxx"      # GitHub token with repo scope
   ```

2. **Token Scopes:**
   - `GITHUB_TOKEN` - Requires `repo` scope for issue operations
   - Must have write access to the repository

3. **Setup Location:**
   Configure in `.devcontainer/.env` (gitignored):
   ```bash
   GITHUB_TOKEN=your_token_here
   ```

### Basic Usage

```bash
# Break down an issue (interactive - will show preview)
python /workspace/.claude/skills/github-issue-breakdown/scripts/create_subissues.py --issue 42

# Specify repository explicitly
python /workspace/.claude/skills/github-issue-breakdown/scripts/create_subissues.py --issue 42 --repo owner/repo

# Dry-run mode (preview only, no creation)
python /workspace/.claude/skills/github-issue-breakdown/scripts/create_subissues.py --issue 42 --dry-run
```

## Task Workflows

### Task 1: Break Down Issue with Parameter

When user provides an issue number in their request:

1. **Extract issue number from user request:**
   ```
   User: "Break down issue #42 into sub-tasks"
   ```

2. **Execute the script:**
   ```bash
   cd /workspace/.claude/skills/github-issue-breakdown
   python scripts/create_subissues.py --issue 42
   ```

3. **Script workflow:**
   - Auto-detects current repository from git context
   - Fetches parent issue content
   - Parses task lists (markdown checkboxes, numbered lists, bullets)
   - Shows preview of proposed sub-issues
   - Asks for confirmation
   - Creates sub-issues with parent-child relationships
   - Reports results with URLs

4. **Example output:**
   ```
   Repository: codekiln/langstar
   Parent Issue: #42 - Add user authentication

   Found 4 tasks to convert into sub-issues:

   1. Create login endpoint
   2. Create registration endpoint
   3. Implement JWT token generation
   4. Add authentication middleware

   Create these 4 sub-issues? (y/n):
   ```

### Task 2: Break Down Issue Without Parameter

When user requests breakdown but doesn't specify which issue:

1. **Recognize the request:**
   ```
   User: "Break down this issue into sub-tasks"
   User: "Create sub-issues from the task list"
   ```

2. **Ask user for issue number:**
   Use the AskUserQuestion tool to prompt for the issue number.

3. **Execute with provided issue number:**
   ```bash
   python scripts/create_subissues.py --issue <user_provided_number>
   ```

### Task 3: Preview Sub-Issues (Dry Run)

When user wants to see what would be created without actually creating:

1. **Execute in dry-run mode:**
   ```bash
   python scripts/create_subissues.py --issue 42 --dry-run
   ```

2. **Script will:**
   - Show all parsed tasks
   - Display what sub-issues would be created
   - Show inherited metadata (labels, assignees)
   - Exit without creating anything

### Task 4: Integration with Spec-Kit

When user has generated tasks using Spec-Kit:

1. **Spec-Kit generates task list:**
   ```
   User: "Run /speckit.tasks to generate task list"
   ```

2. **Create issue from task list:**
   Copy tasks from `.specify/tasks/` into new GitHub issue

3. **Break down the issue:**
   ```bash
   python scripts/create_subissues.py --issue <new_issue_number>
   ```

4. **Workflow integration:**
   - `/speckit.specify` → Define requirements
   - `/speckit.plan` → Create technical plan
   - `/speckit.tasks` → Generate task list
   - **Create parent issue with tasks**
   - **Use this skill to create sub-issues**
   - `/speckit.implement` → Work on sub-issues

## Script Details

### Location

```
/workspace/.claude/skills/github-issue-breakdown/scripts/create_subissues.py
/workspace/.claude/skills/github-issue-breakdown/scripts/gh_helpers.py
```

### Arguments

| Argument | Required | Description | Example |
|----------|----------|-------------|---------|
| --issue | Yes | Issue number to break down | `--issue 42` or `--issue 47` |
| --repo | No | Repository (auto-detected if not provided) | `--repo codekiln/langstar` |
| --dry-run | No | Preview mode without creating | `--dry-run` |
| --inherit-labels | No | Inherit labels from parent (default: true) | `--inherit-labels` |
| --inherit-assignees | No | Inherit assignees from parent (default: true) | `--inherit-assignees` |

### Environment Variables

**Required:**
- `GITHUB_TOKEN` - Fine-grained or classic PAT with `repo` scope

**Optional:**
- `GH_TOKEN` - Alternative name for GitHub token (fallback)

### What the Script Does

1. **Auto-detects repository** - Uses git context to determine owner/repo
2. **Fetches parent issue** - Retrieves issue title, body, labels, assignees via GraphQL
3. **Parses task lists** - Supports multiple formats (see Parsing Patterns)
4. **Shows preview** - Displays proposed sub-issues with inherited metadata
5. **Confirms with user** - Waits for confirmation before creating
6. **Creates sub-issues** - Uses GraphQL `createIssue` mutation with `parentIssueId`
7. **Reports results** - Shows success/failure for each sub-issue with URLs

### Script Behavior

- **Interactive** - Shows preview and waits for confirmation
- **Idempotent** - Safe to run dry-run multiple times
- **Error handling** - Reports specific errors for each sub-issue
- **Repository-agnostic** - Works in any repository without modification
- **Portable** - No hardcoded repository or project IDs

## Parsing Patterns

The skill supports multiple task list formats. For detailed documentation, load `references/parsing-patterns.md` into context.

**Quick reference:**

- Markdown checkboxes: `- [ ] Task name`
- Markdown checked boxes: `- [x] Task name` (skipped by default)
- Numbered lists: `1. Task name`
- Bullet points: `* Task name` or `- Task name`
- Spec-Kit format: Custom task delimiters

## Common Use Cases

### Use Case 1: Feature Development with Sub-Tasks

User says: "Break down issue #42 (Add authentication) into sub-tasks"

Execute:
```bash
python scripts/create_subissues.py --issue 42
```

Result: Creates sub-issues for each task in the parent issue description.

### Use Case 2: Spec-Kit Workflow Integration

User says: "I ran /speckit.tasks and created issue #50 with the task list. Create sub-issues from it."

Execute:
```bash
python scripts/create_subissues.py --issue 50
```

Result: Converts Spec-Kit task list into linked sub-issues.

### Use Case 3: Preview Before Creating

User says: "Show me what sub-issues would be created from issue #47"

Execute:
```bash
python scripts/create_subissues.py --issue 47 --dry-run
```

Result: Shows preview without creating anything.

### Use Case 4: Cross-Repository Sub-Issues

User says: "Create sub-issues from issue #10 in my other-repo"

Execute:
```bash
python scripts/create_subissues.py --issue 10 --repo username/other-repo
```

Result: Creates sub-issues in specified repository.

## Tips for Effective Usage

1. **Check parent issue format** - Ensure parent issue has clear task list
2. **Use dry-run first** - Preview before creating to verify parsing
3. **Support multiple formats** - Script handles various task list styles
4. **Inherit metadata by default** - Sub-issues get parent labels and assignees
5. **Repository auto-detection** - Works automatically in any git repository
6. **Confirmation required** - Script always asks before creating
7. **Review created sub-issues** - Check URLs in output to verify
8. **Integration with workflows** - Complements Spec-Kit and GitHub Projects

## Integration with Existing Workflow

This skill enhances the project's GitHub issue-driven development workflow:

**Standard Workflow:**
1. Create parent issue (existing)
2. **NEW: Use this skill to break down into sub-issues**
3. Create branch for sub-issue work (existing)
4. Development (existing)
5. Create PR (existing)
6. Review & Merge (existing)

**Spec-Kit Integration:**
- Run `/speckit.specify` → Create detailed spec
- Run `/speckit.plan` → Plan implementation
- Run `/speckit.tasks` → Generate task list
- Create parent issue with tasks
- **Use this skill to create sub-issues**
- Run `/speckit.implement` on individual sub-issues

## References

This skill includes reference documentation:

- `references/parsing-patterns.md` - Detailed documentation of supported task list formats

Load reference files into context when needing detailed information about parsing logic or troubleshooting parsing issues.
