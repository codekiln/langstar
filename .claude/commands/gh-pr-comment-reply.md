---
description: Reply to a single GitHub PR review comment using the GitHub API
---

# Reply to GitHub PR Review Comment

Reply to a single PR review comment using the GitHub API.

## Arguments

Arguments are passed via `$ARGUMENTS` in the format:
```
<pr_number> <comment_id> <body> [owner] [repo]
```

**Required:**
- `pr_number`: The PR number (integer)
- `comment_id`: The review comment ID to reply to (integer)
- `body`: The reply text (string, may contain spaces)

**Optional:**
- `owner`: Repository owner (defaults to current repo owner)
- `repo`: Repository name (defaults to current repo name)

## Example Usage

```bash
# Reply to comment 2565891355 on PR #300
/gh-pr-comment-reply 300 2565891355 "Fixed in commit abc123"

# With explicit owner/repo
/gh-pr-comment-reply 300 2565891355 "Fixed" owner repo
```

## Implementation

Parse the arguments from `$ARGUMENTS`:

```text
$ARGUMENTS
```

**Step 1:** Parse arguments

Extract:
1. First argument: `pr_number`
2. Second argument: `comment_id`
3. Remaining arguments (may contain spaces): `body`

If owner/repo are not provided, detect from current repository:
```bash
gh repo view --json owner,name --jq '{owner: .owner.login, name: .name}'
```

**Step 2:** Execute the GitHub API call

Use the `gh api` command to reply to the PR comment:

```bash
gh api repos/{owner}/{repo}/pulls/{pr_number}/comments \
  -f body="{body}" \
  -F in_reply_to={comment_id}
```

**Important:**
- The endpoint is: `POST /repos/{owner}/{repo}/pulls/{pull_number}/comments`
- Parameters:
  - `body` (string, required): The reply text
  - `in_reply_to` (integer, required): The comment ID to reply to
- Use `-f` for string parameters (`body`)
- Use `-F` for integer parameters (`in_reply_to`)

**Step 3:** Report success or failure

On success, output:
```
Successfully replied to comment {comment_id} on PR #{pr_number}
```

On failure, output the error message and suggest checking:
- The PR number exists
- The comment ID is valid
- You have write access to the repository
- The GitHub API authentication is working

## Error Handling

Common errors:
- **404 Not Found**: PR or comment doesn't exist, or no repository access
- **422 Unprocessable Entity**: Invalid parameters (e.g., wrong comment ID format)
- **403 Forbidden**: No write access to repository

For any error, display the full GitHub API response to help with debugging.
