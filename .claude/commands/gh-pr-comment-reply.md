---
description: Reply to a single GitHub PR review comment using the GitHub API
---

# Reply to GitHub PR Review Comment

Reply to a single PR review comment using the GitHub API.

## Critical Constraints - Session Statelessness

**IMPORTANT:** You are operating in a stateless session. Each Claude Code session is isolated.

**You CANNOT:**

- Track issues across sessions
- Remember to do something later
- Follow up on tasks in the future
- Promise to handle something "in a follow-up"

**You MUST NOT say things like:**

- "I'll track this in a follow-up issue"
- "I'll remember to fix this later"
- "I'll handle this in a subsequent PR"

**Instead, choose ONE of these response patterns:**

### Option 1: Implement Now (Preferred)

When the change is small-ish and worth doing:

1. Implement the fix immediately
2. Commit the change
3. Reply to the comment with: "Fixed in commit {sha}: {brief description}"

### Option 2: Defer with Issue (Expensive - use sparingly)

When the change is large AND worth doing AND not critical to PR:

1. Create a GitHub issue NOW using `gh issue create`
2. Add it to the same milestone as the PR's issue (if applicable)
3. Add it as a sub-issue of the parent ticket using `gh sub-issue add`
4. Reply with: "Created #XYZ to track this. Not addressing in this PR because {reason}."
5. Optionally add a `// TODO(#XYZ): description` code comment

**Only choose this option if:**

- The change is large enough to justify a separate PR
- It's not critical to the current PR's functionality
- The PR is mature (many comments already resolved)

### Option 3: Disagree / Won't Fix

When the suggestion is nitpicky, negligible, or you disagree:

1. Reply explaining why this won't be addressed
2. Be professional and concise

**NEVER use this for:**

- Test failures or errors (these MUST be fixed)
- Security concerns
- Critical functionality issues

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

**Parsing rules** (to handle ambiguity when body contains spaces):

- If exactly 5 arguments: `pr_number`, `comment_id`, `body`, `owner`, `repo`
- If exactly 4 arguments: `pr_number`, `comment_id`, `body`, `owner`
- If 3 or more arguments: `pr_number`, `comment_id`, and the rest is `body`
- **Recommendation:** Always quote the `body` argument if it contains spaces

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
3. Parse remaining arguments as follows:
   - If 5 arguments total: `pr_number`, `comment_id`, `body`, `owner`, `repo`
   - If 4 arguments total: `pr_number`, `comment_id`, `body`, `owner`
   - If 3 or more arguments: `pr_number`, `comment_id`, rest is `body`
   - To avoid ambiguity, always quote the `body` argument if it contains spaces

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

- The endpoint is: `POST /repos/{owner}/{repo}/pulls/{pr_number}/comments`
- Parameters:
  - `body` (string, required): The reply text
  - `in_reply_to` (integer, required): The comment ID to reply to
- Use `-f` for string parameters (`body`)
- Use `-F` for integer parameters (`in_reply_to`)

**Security notes:**

- The `body` content is passed as-is to the GitHub API via `-f` flag escaping
- GitHub's API will handle any necessary sanitization on their end
- Very large body texts could hit API payload limits (typically ~65KB for comments)

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
- **422 Unprocessable Entity**: The `in_reply_to` comment ID does not exist or is not valid for the given PR (not just a format issue)
- **403 Forbidden**: No write access to repository

For any error, display the full GitHub API response to help with debugging.
