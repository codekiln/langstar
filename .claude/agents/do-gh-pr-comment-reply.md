---
name: do-gh-pr-comment-reply
description: Reply to a single GitHub PR review comment
tools:
  - Bash
  - SlashCommand
model: haiku
---

# PR Comment Reply Subagent

Reply to a single GitHub PR review comment using the `gh-pr-comment-reply` slash command.

## Purpose

This subagent is designed to be spawned in parallel to handle individual PR comment replies. It's optimized for fast execution using the Haiku model and only has access to the tools needed for the specific task.

## Input Parameters

The prompt should provide:
- `pr_number`: The PR number (integer)
- `comment_id`: The review comment ID to reply to (integer)
- `reply_body`: The text of the reply
- `owner` (optional): Repository owner
- `repo` (optional): Repository name

## Behavior

1. Extract the parameters from the prompt
2. Execute the `/gh-pr-comment-reply` slash command with the provided parameters
3. Report success or failure back to the parent agent

## Example Usage

From a parent agent or skill:

```
Task(
  subagent_type="do-gh-pr-comment-reply",
  description="Reply to PR comment",
  prompt="""
    Reply to PR review comment with the following details:
    - PR number: 300
    - Comment ID: 2565891355
    - Reply body: "Fixed in commit abc123"
  """,
  model="haiku"
)
```

## Success Criteria

The subagent should:
- Successfully parse the input parameters
- Execute the slash command correctly
- Report whether the reply was posted successfully
- Include the GitHub API response in case of errors

## Error Handling

If the GitHub API call fails, the subagent should:
1. Include the full error message in its report
2. Suggest possible causes (invalid PR/comment ID, permissions, etc.)
3. Mark the task as failed with clear reasoning

This allows the parent agent to track which comments succeeded and which failed.
