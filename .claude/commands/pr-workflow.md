---
description: Guide Claude agents through the complete pull request lifecycle, from pre-PR validation through to successful merge
---

# PR Workflow Command

Guide Claude agents through creating and managing a pull request from start to finish, with autonomous monitoring and iterative fixes until the PR is ready to merge.

## Critical Constraints - Session Statelessness

**IMPORTANT:** You are operating in a stateless session. Each Claude Code session is isolated. Every GitHub issue gets a fresh Claude Code context.

**You CANNOT:**

- Track issues across sessions
- Remember to do something later
- Follow up on tasks in the future
- Promise to handle something "in a follow-up"

**You MUST NOT say things like:**

- "I'll track this in a follow-up issue"
- "I'll remember to fix this later"
- "I'll handle this in a subsequent PR"

## PR Comment Response Decision Framework

When addressing review comments, choose ONE of these options:

### Option 1: Implement Now (Preferred)

**When:** The change is small-ish and worth doing.
**Action:**

1. Implement the fix immediately
2. Commit with reference: `🩹 fix: address review feedback - {description}`
3. Reply: "Fixed in commit {sha}: {brief description}"

### Option 2: Defer with Issue (Expensive - use sparingly)

**When:** Change is large AND worth doing AND not critical to current PR.
**Action:**

1. Create GitHub issue NOW: `gh issue create --title "..." --body "..."`
2. Add to same milestone: `gh issue edit <num> --milestone "<milestone>"`
3. Add as sub-issue: `gh sub-issue add <parent> <new-issue>`
4. Reply: "Created #XYZ to track this. Not addressing in this PR because {reason}."
5. Optionally add `// TODO(#XYZ): description` code comment

**Only if:**

- Change is large enough to justify separate PR overhead
- Not critical to current PR's functionality
- PR is mature (many comments already resolved, not early review phase)

### Option 3: Disagree / Won't Fix

**When:** Suggestion is nitpicky, negligible, or you disagree with premise.
**Action:** Reply explaining why this won't be addressed. Be professional and concise.

**NEVER use for:**

- Test failures or errors (MUST be fixed)
- Security concerns
- Critical functionality issues

## Arguments

Arguments are passed via `$ARGUMENTS` in the format:

```
[issue-number]
```

**Optional:**

- `issue-number`: The GitHub issue number to create a PR for (defaults to extracting from current branch)

## User Input

```text
$ARGUMENTS
```

If arguments are provided, parse the issue number. Otherwise, extract from the current branch name.

## Tmux Status Helper

Throughout the workflow, update tmux window name to reflect current phase.

```bash
!# Helper function to update tmux status
# Usage: update_tmux_status <emoji> <prefix> <number>
# Examples:
#   update_tmux_status "💻" "i" "483"  -> 💻i483 (coding on issue #483)
#   update_tmux_status "🔧" "pr" "485" -> 🔧pr485 (maintaining PR #485)
update_tmux_status() {
  local EMOJI="$1"
  local PREFIX="$2"
  local NUMBER="$3"

  if [ -n "$TMUX" ]; then
    TMUX_NAME="${EMOJI}${PREFIX}${NUMBER}"
    tmux rename-window "$TMUX_NAME" 2>/dev/null
  fi
}

# Phase emojis:
# 🔍 = gathering information
# 💻 = coding
# ⏳ = waiting for tests
# ❓ = waiting for user (need more info)
# 🚀 = submitting pr
# 🔧 = pr maintenance
# 🧹 = cleanup

# Prefix conventions:
# i = issue number (e.g., i483 for issue #483)
# pr = pull request number (e.g., pr485 for PR #485)
```

## Overview

This command provides **highly autonomous** PR management, reducing cognitive load by:

- Validating environment and branch setup
- Analyzing all commits (not just the latest) for comprehensive PR description
- Creating PR with proper formatting and milestone
- Continuously monitoring CI/CD checks
- Detecting and guiding through rebase needs
- Resolving PR review comments in parallel
- Iteratively fixing issues until all checks pass

## Workflow Phases

### Phase 1: Pre-PR Validation

**Goal:** Ensure environment is properly configured before creating PR.

**Actions:**

1. **Check working directory:**
   ```bash
   pwd | grep -q "wip/" && echo "✅ In worktree" || echo "❌ Not in wip/ worktree"
   ```
   - **MUST** be in a `wip/` worktree, not `/workspace`
   - If not in worktree, **STOP** and instruct user to use `git-worktrees` skill

2. **Verify branch naming convention:**
   ```bash
   BRANCH=$(git branch --show-current)
   # Should match: m<id>-p<id>-i<num>-<slug> or variants
   ```
   - Extract issue number: `ISSUE_NUM=$(echo "$BRANCH" | grep -oP 'i\K[0-9]+' || echo "$BRANCH" | grep -oE '[0-9]+' | head -1)`
   - If no issue number in branch, **STOP** and ask user which issue this PR fixes

3. **Verify issue exists and is open:**
   ```bash
   gh issue view "$ISSUE_NUM" --json state,title,milestone
   ```
   - If issue is closed, **WARN** and ask user to confirm
   - Store milestone information for later (will add to PR)

4. **Check for uncommitted changes:**
   ```bash
   git status --porcelain
   ```
   - If there are uncommitted changes, **STOP** and ask user to commit them first

5. **Verify branch has commits:**
   ```bash
   # Check if branch has commits ahead of base
   git rev-list --count origin/main..HEAD
   ```
   - If count is 0, the PR would be empty and should be prevented
   - **STOP** and inform user the branch has no new commits

6. **Verify remote branch exists or can be pushed:**
   ```bash
   git push -n origin $(git branch --show-current) 2>&1
   ```
   - Dry-run push to check if branch can be pushed
   - If fails, investigate and resolve

7. **Check if PR already exists:**
   ```bash
   gh pr list --head $(git branch --show-current) --json number,title,url,state
   ```
   - If PR exists and is **OPEN**: Skip to Phase 4 (CI/CD Monitoring Loop)
   - If PR exists and is **CLOSED**: Warn user and ask whether to create new PR
   - If PR exists and is **MERGED**: Inform user and stop (branch should be cleaned up)
   - If no PR exists: Continue to Phase 2

**Validation Summary:**

- ✅ In worktree
- ✅ Branch follows convention
- ✅ Issue exists and is open
- ✅ No uncommitted changes
- ✅ Branch has commits
- ✅ Can push to remote
- ✅ PR existence checked

**If PR already exists:**

```
✅ **Found Existing PR**

📍 PR #$PR_NUM: <title>
🔗 URL: <pr_url>
📊 State: OPEN

Skipping to monitoring phase. I'll:
1. Address any unresolved review comments
2. Check if rebase to main is needed
3. Monitor CI/CD checks until all pass

Starting autonomous PR management...
```

If any validation fails, **STOP** and provide clear instructions to fix the issue.

### Phase 2: PR Creation Preparation

**Goal:** Analyze all changes since branch divergence and draft comprehensive PR description.

**Actions:**

1. **Determine base branch:**
   ```bash
   # Check if this issue has a parent (for hierarchical merging)
   gh sub-issue list "$ISSUE_NUM" --relation parent
   ```
   - If parent exists: PR should target parent's branch
   - If no parent: PR should target `main` (or current release branch)

2. **Analyze complete commit history:**
   ```bash
   # Get base branch (main or parent branch)
   BASE_BRANCH="main"  # or parent branch if applicable

   # View all commits in this branch
   git log "$BASE_BRANCH"..HEAD --oneline

   # View full diff
   # Use three dots (...) to show the diff between the merge-base of BASE_BRANCH and HEAD.
   # This is the standard approach for PR diffs: it shows only changes introduced by this branch since it diverged from BASE_BRANCH (i.e., changes unique to this branch), and does not include changes made in BASE_BRANCH after divergence.
   git diff "$BASE_BRANCH"...HEAD
   ```
   - **IMPORTANT:** Analyze ALL commits, not just the latest
   - Summarize the nature of changes (feature, fix, refactor, etc.)

3. **Draft PR title:**
   - Follow conventional emoji commit format: `<emoji> <type>[scope]: <description>`
   - Match the primary nature of changes:
     - ✨ `feat` - New features
     - 🩹 `fix` - Bug fixes (standard fix emoji per project conventions)
     - 📚 `docs` - Documentation
     - ♻️ `refactor` - Code refactoring
     - 🧪 `test` - Tests
     - 🔧 `build` - Build system changes

4. **Draft PR body using template:**
   ```markdown
   ## Summary

   - <High-level summary point 1>
   - <High-level summary point 2>
   - <High-level summary point 3>

   ## Changes

   - <Specific change 1 with file references>
   - <Specific change 2 with file references>
   - <Specific change 3 with file references>

   ## Related Issues

   Fixes #$ISSUE_NUM

   ## Test Plan

   - [ ] <Test item 1>
   - [ ] <Test item 2>
   - [ ] <Test item 3>

   ---

   🤖 Generated with [Claude Code](https://claude.com/claude-code)

   Co-Authored-By: Claude <noreply@anthropic.com>
   ```
   - Be specific about files changed and why
   - Include test plan based on what was modified
   - **MUST** include `Fixes #$ISSUE_NUM` for auto-close

5. **Show draft to user and confirm:**
   ```
   📋 **Draft PR Title:**
   <title>

   📄 **Draft PR Body:**
   <body>

   🎯 **Target Branch:** <base_branch>
   🏷️ **Milestone:** <milestone if exists>

   Proceed with PR creation? (I'll create the PR and begin monitoring)
   ```

### Phase 3: PR Creation

**Goal:** Create PR with proper formatting and configuration.

**Actions:**

1. **Update tmux status to "submitting PR":**
   ```bash
   !update_tmux_status "🚀" "i" "$ISSUE_NUM"
   ```

2. **Push branch to remote (if not already pushed):**
   ```bash
   git push -u origin $(git branch --show-current)
   ```

3. **Create PR using gh CLI:**
   ```bash
   ISSUE_NUM=<extracted_issue_num>
   BASE_BRANCH=<determined_base_branch>

   # Capture PR URL and extract PR number from it
   PR_URL=$(gh pr create \
     --title "<title>" \
     --body "$(cat <<EOF
   <body>
   EOF
   )" \
     --base "$BASE_BRANCH")

   # Extract PR number from the URL (e.g., https://github.com/owner/repo/pull/123 -> 123)
   PR_NUM=$(echo "$PR_URL" | grep -oE '[0-9]+$')
   ```

4. **Add milestone to PR (if issue has milestone):**
   ```bash
   MILESTONE=$(gh issue view "$ISSUE_NUM" --json milestone -q '.milestone.title')

   if [ -n "$MILESTONE" ]; then
     gh pr edit "$PR_NUM" --milestone "$MILESTONE"
   fi
   ```

5. **Verify PR will close issue:**
   ```bash
   gh pr view "$PR_NUM" --json closingIssuesReferences -q '.closingIssuesReferences[].number'
   ```
   - Should output the issue number
   - If not, **WARN** and fix PR body

6. **Report PR creation:**
   ```
   ✅ **PR Created:** #$PR_NUM
   📍 URL: <pr_url>
   🎯 Target: $BASE_BRANCH
   🏷️ Milestone: $MILESTONE

   Now monitoring CI/CD checks and review comments...
   ```

### Phase 4: CI/CD Monitoring Loop

**Goal:** Continuously monitor PR status and automatically fix issues until ready to merge.

**This is a LOOP - continue until PR is ready or user intervenes.**

**IMPORTANT - Idempotent Design:**

- This phase can be run multiple times safely
- Each run checks current state and only acts on what's needed
- Safe to restart if interrupted - will pick up where it left off
- Safe to run in parallel with manual changes - will sync and continue

**Update tmux status to "PR maintenance" (using PR number):**

```bash
!# After PR is created, switch from issue number to PR number
!update_tmux_status "🔧" "pr" "$PR_NUM"
```

**Order of operations (priority):**

1. Review comments FIRST (most important - human feedback)
2. Rebase check (must be up-to-date before CI is meaningful)
3. CI/CD checks (verify code quality)
4. Monitor for 5-7 minutes after all fixes to ensure stability

**Actions in each iteration:**

1. **Check for review comments (HIGHEST PRIORITY):**
   ```bash
   # Fetch unresolved review comments using the GitHub Reviews API
   # NOTE: GitHub API returns `null` for unresolved comments, not `false`
   # IMPORTANT: Use --paginate to get ALL comments (default page size is 30)
   REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
   gh api repos/$REPO/pulls/$PR_NUM/comments --paginate \
     --jq '.[] | select(.resolved == null) | {id, path, line, user: .user.login, body}'
   ```
   - Count unresolved review comments (where `.resolved == null`)
   - **IMPORTANT:** Include comments from both human reviewers AND Copilot
   - If unresolved comments exist, handle them automatically:
     ```
     💬 **Found Unresolved Review Comments:** 5

     1. Comment 2565891355 in .claude/commands/pr-workflow.md:45 by @copilot
        "Consider adding error handling here"

     2. Comment 2565891356 in .claude/commands/pr-workflow.md:120 by @reviewer
        "This logic could be simplified"

     Addressing each comment now...
     ```

2. **Address each review comment using the Decision Framework:**

   **For each comment, apply the PR Comment Response Decision Framework (see above):**

   - **Option 1 (Implement Now):** Read file, make fix, commit, reply with commit SHA
   - **Option 2 (Defer with Issue):** Create issue NOW, add to milestone, link as sub-issue, reply with issue number
   - **Option 3 (Disagree):** Reply explaining why not addressing (NEVER for test failures/errors)

   **CRITICAL:** Do NOT promise to "track this later" or "handle in a follow-up" - you cannot!

   Use the `resolve-pr-comments` skill for parallel handling if 3+ comments.

   **CRITICAL: Always Reply In-Thread Using `/gh-pr-comment-reply`**

   When replying to review comments, you MUST reply directly to the comment thread (not as a top-level PR comment).
   This allows maintainers to mark threads as resolved.

   **Steps:**
   1. Extract the comment URL from the review:
      ```bash
      # From unresolved comments JSON (step 1 above), get the comment ID
      # Build the comment URL: https://github.com/<owner>/<repo>/pull/<pr_num>#discussion_r<comment_id>
      COMMENT_URL="https://github.com/$REPO/pull/$PR_NUM#discussion_r$COMMENT_ID"
      ```

   2. Use the dedicated slash command to reply:
      ```bash
      /gh-pr-comment-reply $COMMENT_URL
      ```
      This command:
      - Extracts the comment ID automatically
      - Uses the correct GitHub API endpoint for in-thread replies
      - Ensures the reply appears in the comment thread (not top-level)
      - Allows maintainers to mark the thread as resolved

   3. **NEVER use these for review comment replies:**
      - ❌ `gh pr comment` - Creates top-level comment (can't be resolved)
      - ❌ Manual `gh api` calls - Error-prone and unnecessary

   **Example commit for Option 1:**
   ```bash
   git commit -m "$(cat <<EOF
   ```

🩹 fix(pr-workflow): address review feedback on error handling

- Added validation before API calls
- Improved error messages for user clarity

Addresses review comment: https://github.com/owner/repo/pull/385#discussion_r123456
EOF
)"

````
**Example for Option 2 (Defer):**
```bash
# Create issue
ISSUE_URL=$(gh issue create --title "Refactor: improve error handling patterns" \
  --body "Raised in PR #385 review. See: https://github.com/owner/repo/pull/385#discussion_r123456")
NEW_ISSUE=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')

# Add to milestone (if PR has one)
gh issue edit "$NEW_ISSUE" --milestone "current-milestone"

# Link as sub-issue of parent
gh sub-issue add "$PARENT_ISSUE" "$NEW_ISSUE"

# Reply to comment using the slash command
/gh-pr-comment-reply https://github.com/owner/repo/pull/385#discussion_r123456
# Reply body: "Created #$NEW_ISSUE to track this. Not addressing in this PR as it's a larger refactor."
````

3. **Check if branch needs rebasing:**
   ```bash
   # Check if base branch has new commits
   git fetch origin "$BASE_BRANCH"
   BEHIND_COUNT=$(git rev-list --count HEAD..origin/"$BASE_BRANCH")
   ```
   - If `BEHIND_COUNT > 0`, rebase is needed:
     ```bash
     # Attempt automatic rebase
     git rebase origin/"$BASE_BRANCH"

     # If successful:
     git push --force-with-lease origin $(git branch --show-current)

     # If conflicts occur: stop and ask user
     ```
   - Report rebase status:
     ```
     🔄 **Rebased to $BASE_BRANCH**

     Branch was $BEHIND_COUNT commits behind. Successfully rebased and pushed.
     ⏳ Waiting for CI to run on rebased code...
     ```

4. **Check CI/CD status:**
   ```bash
   # Update tmux status to "waiting for tests"
   !update_tmux_status "⏳" "pr" "$PR_NUM"

   # Check CI/CD status and wait for completion
   while true; do
     # Get the status of all checks
     # Note: gh pr checks uses 'state' field with values: SUCCESS, FAILURE, SKIPPED, PENDING, etc.
     checks_running=$(gh pr checks "$PR_NUM" --json state,completedAt --jq '[.[] | select(.completedAt == null)] | length')
     if [ "$checks_running" -gt 0 ]; then
       echo "⏳ Checks still running, waiting 30 seconds..."
       sleep 30
     else
       break
     fi
   done

   # Return to PR maintenance status
   !update_tmux_status "🔧" "pr" "$PR_NUM"
   # Now parse output for failed checks
   - If checks pass: proceed to stability monitoring
   - If checks fail: proceed to failure handling
   ```

5. **If CI/CD checks fail:**
   ```bash
   # Get detailed failure information
   # Note: gh pr checks uses 'state' field (not 'conclusion')
   # Available fields: bucket, completedAt, description, event, link, name, startedAt, state, workflow
   gh pr checks "$PR_NUM" --json name,state,link,workflow \
     --jq '.[] | select(.state == "FAILURE") | {name, workflow, link}' > failed_checks.json

   # Extract run IDs from links (format: https://github.com/owner/repo/actions/runs/12345/job/67890)
   # For each failed check, fetch logs using gh run view
   for link in $(jq -r '.link' failed_checks.json); do
     # Extract run ID from URL
     run_id=$(echo "$link" | grep -oP 'runs/\K[0-9]+')
     if [ -n "$run_id" ]; then
       gh run view "$run_id" --log-failed
     fi
   done
   ```
   - Parse error messages from logs
   - Identify specific failures (clippy, tests, fmt, etc.)
   - Report to user with actionable information:
     ```
     ❌ **CI Checks Failed:**

     1. cargo-clippy: 3 warnings found
        - src/main.rs:45: unused variable `foo`
        - src/lib.rs:120: missing documentation

     2. cargo-test: 2 tests failed
        - test_authentication: assertion failed
        - test_authorization: panic at line 67

     I'll fix these issues now...
     ```

6. **Fix identified CI issues:**
   - Read relevant files
   - Apply fixes based on error messages
   - Commit fixes following conventional commit format:
     ```bash
     git add <files>
     # Replace <specific issue> with actual description (e.g., "clippy warnings")
     # This is a template - interpolate real values when executing
     git commit -m "$(cat <<EOF
     ```

🩹 fix(ci): address <specific issue>

- Fixed clippy warnings in src/main.rs
- Resolved test failures in authentication module
  EOF
  )"

  git push origin $(git branch --show-current)
  if [ $? -ne 0 ]; then
  echo "❌ git push failed due to network or permission issues. Please check your connection and access rights."
  exit 1
  fi
  ```
  - Report what was fixed:
  ```
  ✅ **Fixes Applied:**
  - Removed unused variable `foo` in src/main.rs:45
  - Added missing documentation in src/lib.rs:120
  - Fixed assertion in test_authentication
  - Resolved panic in test_authorization

  🔄 Pushed commit: abc123
  ⏳ Waiting for CI to run again...
  ```
  ```

7. **Stability monitoring (after all fixes):**
   - Once all checks pass, monitor for 5-7 minutes to ensure stability
   - Check every 60 seconds for any new failures or comments
   - If new issues appear, loop back to step 1
   - Report during monitoring:
     ```
     ✅ **All Checks Passing**

     🕐 Monitoring for 5 minutes to ensure stability...
     ⏳ Time remaining: 4:00
     ```

8. **Loop back to step 1** after:
   - Applying fixes and pushing
   - Resolving review comments
   - Any other changes made to PR
   - During stability monitoring if new issues detected

**Exit loop when:**

- ✅ All CI checks passing
- ✅ No unresolved review comments
- ✅ Branch up-to-date with base
- ✅ PR properly linked to issue with milestone
- ✅ Stability monitoring completed (5 minutes with no new issues)

### Phase 5: Completion Verification

**Goal:** Confirm PR is ready for review/merge.

**Actions:**

1. **Final status check:**
   ```bash
   gh pr view "$PR_NUM" --json state,mergeable,reviewDecision,statusCheckRollup
   ```

2. **Display completion checklist:**
   ```
   ✅ **PR Ready for Review/Merge**

   📍 PR #$PR_NUM: <title>
   🔗 URL: <pr_url>

   **Status:**
   ✅ All CI checks passing
   ✅ All review comments resolved
   ✅ Branch up-to-date with $BASE_BRANCH
   ✅ Properly linked to issue #$ISSUE_NUM
   ✅ Milestone: $MILESTONE

   **Next Steps:**
   - PR is ready for review
   - Once approved, you can merge
   - After merge, use `pr-lifecycle` skill for cleanup
   ```

3. **Remind about post-merge cleanup:**
   ```
   📝 **Don't forget after merge:**
   After the PR is merged, remember to clean up with these commands:
   ```

   ```bash
   cd /workspace
   git checkout main
   git pull origin main
   git worktree remove wip/<branch-name>
   git branch -d <branch-name>
   git worktree prune --verbose
   ```

   Or use the `pr-lifecycle` skill's cleanup workflow.

## Special Cases and Handling

### Case 1: No Issue Number Provided and Can't Extract

**Scenario:** User runs `/pr-workflow` but branch doesn't follow naming convention.

**Action:**

```
❌ **Cannot determine issue number**

Your branch name doesn't follow the convention: `m<milestone>-p<parent>-i<issue>-<slug>` (or variants)

Current branch: <branch_name>

Please provide the issue number manually:
/pr-workflow <issue_number>

Or rename your branch to follow the convention.
```

### Case 2: Issue Has Parent (Hierarchical Merging)

**Scenario:** Issue has a parent issue, indicating it should PR into parent's branch.

**Action:**

```
🔍 **Detected Parent Issue**

This issue (#$ISSUE_NUM) has a parent: #$PARENT_NUM

According to project workflow, this PR should target the parent's branch instead of main.

**Target Branch:** <parent_branch>

Proceeding with PR creation targeting parent branch...
```

### Case 3: CI Failure Can't Be Auto-Fixed

**Scenario:** CI failure is complex and requires manual intervention.

**Action:**

```
❌ **CI Failure Requires Manual Attention**

The following CI check failed with an issue I cannot automatically fix:

**Check:** <check_name>
**Error:** <error_summary>
**Logs:** <log_url>

**Suggested Actions:**
1. Review the error logs at the URL above
2. Make necessary changes locally
3. Commit and push fixes
4. I'll continue monitoring once you push

Would you like me to show you the relevant code sections?
```

### Case 4: Merge Conflicts During Rebase

**Scenario:** Base branch has advanced and rebase results in conflicts.

**Action:**

```
⚠️ **Rebase Required with Potential Conflicts**

Your branch is out of date with $BASE_BRANCH.

**New commits in $BASE_BRANCH:**
- a1b2c3d feat: add new authentication method
- d4e5f6g fix: resolve cache invalidation bug

**Rebase Steps:**
```

```bash
git fetch origin "$BASE_BRANCH"
git rebase origin/"$BASE_BRANCH"

# If conflicts occur:
git status  # See conflicted files
# Resolve conflicts in your editor
git add <resolved_files>
git rebase --continue

git push --force-with-lease origin $(git branch --show-current)
```

I cannot automatically rebase if conflicts occur. Let me know when you've completed the rebase.

### Case 5: Review Comments Need Code Changes

**Scenario:** Review comments indicate code changes are needed, not just replies.

**Action:**

```
💬 **Review Comments Require Code Changes**

I've identified <N> review comments that require code modifications:

1. Comment 2565891355: "Missing error handling in auth.rs:45"
   → Need to add error handling

2. Comment 2565891356: "Performance issue in query.rs:120"
   → Need to optimize query

Would you like me to:
A) Address these comments one by one with your guidance
B) Attempt to fix them automatically based on the feedback
C) Just reply that I'm working on them
```

## Integration with Skills

### pr-lifecycle Skill

Use for initial validation (Phase 1):

- Worktree verification
- Branch naming validation
- Issue linking checks

### resolve-pr-comments Skill

Use in Phase 4 when resolving multiple comments:

- Fetch all unresolved comments
- Reply in parallel using subagents
- Track success/failure of replies

### git-worktrees Skill

Reference for users who aren't in a worktree:

- Explain how to create worktree
- Guide user to proper setup

## Best Practices

### Autonomous Operation

**Be proactive:**

- Don't ask permission for obvious fixes (clippy warnings, fmt issues)
- Automatically retry after pushing fixes
- Continuously monitor without user prompting

**But ask when needed:**

- Complex CI failures that need investigation
- Code changes based on review comments
- Rebasing with potential conflicts

### Communication

**Progress updates:**

- After each phase completion
- When starting long-running operations (CI checks)
- When encountering issues

**Error reporting:**

- Be specific about what failed
- Provide actionable next steps
- Include relevant links (logs, docs)

### Iteration Limits

**Prevent infinite loops:**

- After 5 iterations without progress, stop and ask user
- If same CI check fails 3 times with same fix attempts, ask user
- If unable to push commits, stop immediately

**Tracking mechanism example:**

```bash
iteration_count=0
max_iterations=5
failed_attempts=0
max_failed_attempts=3

while true; do
  # ... perform CI check or fix attempt ...
  iteration_count=$((iteration_count + 1))

  if [ "$ci_check_failed" = "true" ]; then
    failed_attempts=$((failed_attempts + 1))
  else
    failed_attempts=0
  fi

  if [ $iteration_count -ge $max_iterations ]; then
    echo "⚠️ Iteration Limit Reached"
    # ... handle limit (ask user, stop, etc.) ...
    break
  fi

  if [ $failed_attempts -ge $max_failed_attempts ]; then
    echo "⚠️ CI check failed 3 times with same fix attempts"
    # ... handle repeated failure ...
    break
  fi

  # ... other workflow logic ...
done
```

**Example user message:**

```
⚠️ **Iteration Limit Reached**

I've attempted to fix the CI failures 5 times, but checks are still failing.

**Latest Failure:**
<error summary>

This may require manual investigation. Would you like me to:
A) Try one more time with a different approach
B) Stop and let you investigate
C) Show me the full error logs for analysis
```

## Error Handling

### GitHub API Errors

**401 Unauthorized:**

```bash
gh auth status
# If not authenticated:
gh auth login
```

**404 Not Found:**

- Issue/PR doesn't exist
- Verify issue number
- Verify repository access

**422 Unprocessable:**

- Invalid PR body format
- Review PR description for syntax errors

### Git Errors

**Push rejected:**

- Check if branch is protected
- Verify write access
- Check if force-push is needed (after rebase)

**Diverged branches:**

- Guide user through rebase
- Offer to show diverged commits

## Command Reference

### Essential Commands Used

**PR Management:**

```bash
# Create PR
gh pr create --title "..." --body "..." --base main

# View PR
gh pr view <num> --json <fields>

# Edit PR
gh pr edit <num> --milestone "..." --add-label "..."

# Check PR status
gh pr checks <num>
gh pr view <num> --json state,mergeable,reviewDecision
```

**Issue Management:**

```bash
# View issue
gh issue view <num> --json state,title,milestone

# Check for parent issues
# Requires the custom `gh-sub-issue` extension/skill (see `.claude/skills/gh-sub-issue/SKILL.md`)
gh sub-issue list <num> --relation parent
```

**Git Operations:**

```bash
# Branch info
git branch --show-current
git log <base>..HEAD --oneline
git diff <base>...HEAD

# Push changes
git push -u origin <branch>
git push --force-with-lease origin <branch>

# Rebase
git fetch origin <base>
git rebase origin/<base>
```

**Review Comments:**

```bash
# Fetch unresolved comments (use --paginate to get ALL comments)
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
gh api repos/$REPO/pulls/$PR_NUM/comments --paginate \
  --jq '.[] | select(.resolved == null) | {id, path, line, user: .user.login, body}'

# Reply to a review comment - ALWAYS use the dedicated slash command
# Build comment URL: https://github.com/<owner>/<repo>/pull/<pr_num>#discussion_r<comment_id>
/gh-pr-comment-reply https://github.com/<owner>/<repo>/pull/<pr_num>#discussion_r<comment_id>

# ❌ NEVER use these for review comment replies:
# - gh pr comment <pr_num> --body "..."  (creates top-level comment, can't be resolved)
# - Manual gh api calls (error-prone, use slash command instead)
```

## See Also

- **pr-lifecycle skill** - Pre-PR validation and post-merge cleanup (`.claude/skills/pr-lifecycle/SKILL.md`)
- **resolve-pr-comments skill** - Parallel comment resolution (`.claude/skills/resolve-pr-comments/SKILL.md`)
- **git-worktrees skill** - Worktree management (`.claude/skills/git-worktrees/SKILL.md`)
- **GitHub Workflow Documentation** - `@docs/dev/github-workflow.md`
- **Git SCM Conventions** - `@docs/dev/git-scm-conventions.md`
