# Issue Automation

This document describes the automated issue management system that integrates with GitHub Projects V2 and the Claude Code bot.

## Overview

The Langstar project uses GitHub Actions to automatically manage issue assignments and project status updates when the Claude bot begins work and when issues are closed.

## Automated Behaviors

### When Claude Bot Starts Work

When a user mentions `@claude` in an issue comment:

1. **Issue Assignment**: The issue is automatically assigned to `@codekiln`
2. **Status Update**: The issue's status in the langstar GitHub Project is updated to "In Progress"

**Trigger:** Issue comment containing `@claude` by user `codekiln`

**Implementation:** `.github/workflows/issue-status-automation.yml` (job: `update-status-on-claude-trigger`)

### When Issue Is Closed

When an issue is closed (manually or via PR merge):

1. **Status Update**: The issue's status in the langstar GitHub Project is updated to "Done"

**Trigger:** Issue closed event

**Implementation:** `.github/workflows/issue-status-automation.yml` (job: `update-status-on-close`)

## Components

### Workflow: `issue-status-automation.yml`

Location: `.github/workflows/issue-status-automation.yml`

This workflow handles both automation scenarios:

**Job 1: `update-status-on-claude-trigger`**
- Triggers when `@claude` is mentioned in an issue comment
- Assigns the issue to `@codekiln`
- Updates project status to "In Progress"

**Job 2: `update-status-on-close`**
- Triggers when an issue is closed
- Updates project status to "Done"

**Permissions Required:**
- `issues: write` - To assign issues
- `contents: read` - To checkout repository and access scripts

### Script: `update-issue-status.sh`

Location: `.github/scripts/update-issue-status.sh`

A bash script that handles the actual API calls to update issue assignments and project status.

**Usage:**
```bash
./update-issue-status.sh <issue_number> <status> [assignee]
```

**Arguments:**
- `issue_number` - The GitHub issue number (e.g., `15`)
- `status` - The status to set: `in_progress` or `done`
- `assignee` - (Optional) GitHub username to assign (default: `codekiln`)

**Environment Variables Required:**
- `GITHUB_TOKEN` - GitHub token with appropriate permissions
- `GITHUB_REPOSITORY` - Repository in format "owner/repo" (automatically provided by GitHub Actions)

**Examples:**
```bash
# Mark issue 15 as in progress and assign to codekiln
./update-issue-status.sh 15 in_progress codekiln

# Mark issue 15 as done (no assignment needed)
./update-issue-status.sh 15 done
```

## How It Works

### Workflow Execution Flow

#### When @claude is Mentioned

1. User comments `@claude can you help with this?` on an issue
2. GitHub triggers the `issue_comment` event
3. The `update-status-on-claude-trigger` job runs:
   - Checks if the comment contains `@claude` and actor is `codekiln`
   - Checks out the repository
   - Executes `update-issue-status.sh` with parameters:
     - Issue number from `github.event.issue.number`
     - Status: `in_progress`
     - Assignee: `codekiln`
4. The script:
   - Assigns the issue to `@codekiln`
   - Gets the issue's node ID
   - Finds or creates the project item for this issue
   - Updates the project status field to "In Progress"
5. The Claude Code workflow (`.github/workflows/claude.yml`) then runs separately

#### When Issue is Closed

1. Issue is closed (manually via UI or automatically via PR merge with `Fixes #N`)
2. GitHub triggers the `issues` event with action `closed`
3. The `update-status-on-close` job runs:
   - Checks out the repository
   - Executes `update-issue-status.sh` with parameters:
     - Issue number from `github.event.issue.number`
     - Status: `done`
4. The script:
   - Gets the issue's node ID
   - Finds the project item for this issue
   - Updates the project status field to "Done"

### GitHub Project Configuration

The automation uses these GitHub Project V2 identifiers:

| Component | Value | Description |
|-----------|-------|-------------|
| Project ID | `PVT_kwHOAAImgs4BGe4B` | langstar project |
| Status Field ID | `PVTSSF_lAHOAAImgs4BGe4Bzg3g-NQ` | Status field |
| Todo Status | `f75ad846` | Initial status |
| In Progress Status | `47fc9ee4` | Active work status |
| Done Status | `98236657` | Completed status |

These IDs are hardcoded in `.github/scripts/update-issue-status.sh`.

If the project structure changes, update these values in the script.

## Local Testing

You can test the script locally if you have the appropriate permissions:

```bash
# Export your GitHub token
export GITHUB_TOKEN="your-github-token"
export GITHUB_REPOSITORY="codekiln/langstar"

# Test marking an issue as in progress
.github/scripts/update-issue-status.sh 15 in_progress codekiln

# Test marking an issue as done
.github/scripts/update-issue-status.sh 15 done
```

**Note:** Local testing requires a GitHub token with:
- `repo` scope (for issue access)
- `project` scope (for project updates)

The `GITHUB_PROJECT_PAT` defined in `.devcontainer/.env` has these permissions for local testing.

## Troubleshooting

### Issue Not Being Assigned

**Symptom:** Issue remains unassigned after `@claude` is mentioned

**Possible Causes:**
1. The workflow didn't run (check Actions tab)
2. The actor was not `codekiln` (workflow only runs for this user)
3. GitHub token lacks `issues: write` permission
4. The assignee username is invalid

**Solution:**
- Check the GitHub Actions logs for the `update-status-on-claude-trigger` job
- Verify the workflow conditions match the trigger event
- Confirm the `GITHUB_TOKEN` has appropriate permissions

### Project Status Not Updating

**Symptom:** Issue assignment works but project status doesn't change

**Possible Causes:**
1. Issue not added to the project yet
2. GitHub token lacks project permissions
3. Project IDs changed (project was recreated)
4. GraphQL API call failed

**Solution:**
- Check the script output in GitHub Actions logs
- Verify the issue appears in the project board
- Confirm project IDs in the script match current values
- Test the GraphQL queries manually using `gh api graphql`

### Script Fails to Find Project Item

**Symptom:** Script reports "Issue not in project"

**Behavior:** The script automatically attempts to add the issue to the project

**If This Fails:**
1. Check that the project ID is correct
2. Verify the GitHub token has project write permissions
3. Manually add the issue to the project through the UI
4. Re-run the workflow

### Workflow Not Triggering

**Symptom:** Nothing happens when `@claude` is mentioned

**Check:**
1. Is the actor `codekiln`? (workflow is restricted to this user)
2. Does the comment body contain exactly `@claude`?
3. Is the workflow file valid YAML?
4. Are there any workflow errors shown in the Actions tab?

**Debug:**
```bash
# Check workflow syntax locally
gh workflow view issue-status-automation.yml

# List recent workflow runs
gh run list --workflow=issue-status-automation.yml

# View logs for a specific run
gh run view <run-id> --log
```

## Permissions

### GitHub Actions Token

The workflow uses `${{ secrets.GITHUB_TOKEN }}`, which is automatically provided by GitHub.

**Required Permissions:**
```yaml
permissions:
  issues: write      # For assigning issues
  contents: read     # For checking out repo
```

**Note:** The default `GITHUB_TOKEN` does **not** have full project permissions. The script works because it uses the GitHub GraphQL API to update project items through the issue's connection to the project.

### Personal Access Token

For local testing, use the `GITHUB_PROJECT_PAT` defined in `.devcontainer/.env`.

**Required Scopes:**
- `repo` - Full repository access
- `project` - Full project access

## Future Enhancements

Possible improvements to consider:

1. **Configurable Assignee**: Allow specifying assignee via workflow input or based on who triggered `@claude`
2. **Multiple Projects**: Support updating multiple projects simultaneously
3. **Custom Status Names**: Make status names configurable via workflow inputs
4. **Label Automation**: Add/remove labels in addition to project status updates
5. **Slack Notifications**: Send notifications when status changes
6. **Status History**: Track status change history in issue comments
7. **Validation**: Add checks to ensure issue is in correct state before updating

## Related Documentation

- [GitHub Workflow](./github-workflow.md) - Issue-driven development process
- [GitHub Projects](./github-projects.md) - Project configuration and API reference
- [Git SCM Conventions](./git-scm-conventions.md) - Commit message standards

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitHub Projects V2 API](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)
- [GitHub GraphQL API](https://docs.github.com/en/graphql)
- [Claude Code GitHub Actions](https://github.com/anthropics/claude-code-action)
