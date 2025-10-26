# GitHub Issue-Driven Development Workflow

## Overview

This document describes the preferred development workflow for Langstar, which leverages GitHub issues and Claude Code GitHub Actions for streamlined branch management and pull requests.

## Workflow Steps

### 1. Create GitHub Issue

Start every feature, bug fix, or task by creating a GitHub issue:

1. Navigate to the repository's Issues tab
2. Click "New Issue"
3. Provide a clear title and detailed description
4. Add relevant labels (bug, enhancement, documentation, etc.)
5. Assign the issue if applicable

**Example Issue:**
```
Title: Add user authentication middleware
Labels: enhancement, backend
Description: Implement JWT-based authentication middleware to protect API endpoints...
```

### 2. Branch Creation with Claude Code

Use Claude Code's GitHub Actions integration to automatically create a branch for the issue:

**Method 1: Comment on the Issue**
Comment `@claude` on the issue to trigger Claude Code. Claude will automatically create a branch following the naming convention and start working on the issue.

**Method 2: Manual Branch Creation**
If you prefer to create the branch manually or Claude creates one automatically:

```bash
# Branch naming convention: claude/issue-N-YYYYMMDD-HHMM
# Example: claude/issue-42-20251026-1400
git checkout -b claude/issue-42-20251026-1400
```

**Branch Naming Convention:**
- Format: `claude/issue-N-YYYYMMDD-HHMM`
- `N` is the issue number
- Date/time helps with uniqueness and tracking
- Examples:
  - `claude/issue-7-20251026-1344` (for issue #7)
  - `claude/issue-123-20251030-0915` (for issue #123)

### 3. Development

Make your changes on the issue branch:

1. **Make code changes** following the project's coding conventions (see [README.md](./README.md))
2. **Test your changes** locally
3. **Commit frequently** using Conventional Emoji Commits (see [git-scm-conventions.md](./git-scm-conventions.md))

**Example commits:**
```bash
git add src/auth/middleware.js
git commit -m "✨ feat(auth): add JWT authentication middleware"

git add tests/auth/middleware.test.js
git commit -m "🧪 test(auth): add tests for JWT middleware"

git add docs/api/authentication.md
git commit -m "📚 docs(auth): document JWT authentication flow"
```

### 4. Create Pull Request

When your changes are ready for review, create a pull request:

**Method 1: Using Claude Code**
If Claude Code worked on the issue, it will provide a pre-filled PR creation link in its comment.

**Method 2: Manual PR Creation**
1. Push your branch to the remote:
   ```bash
   git push origin claude/issue-N-YYYYMMDD-HHMM
   ```

2. Navigate to the repository on GitHub
3. Click "Compare & pull request" or use the pre-filled URL format:
   ```
   https://github.com/codekiln/langstar/compare/main...claude/issue-N-YYYYMMDD-HHMM?quick_pull=1&title=<encoded-title>&body=<encoded-body>
   ```

**Pull Request Guidelines:**

**Title Format:**
Follow the same Conventional Emoji Commits format:
```
✨ feat(auth): add JWT authentication middleware
```

**Body Template:**
```markdown
## Summary
Brief description of what this PR accomplishes.

## Changes
- Bullet point list of main changes
- Each change should be clear and concise
- Include any breaking changes

## Related Issues
Fixes #N (this automatically closes issue #N when PR is merged)

## Test Plan
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing completed

## Screenshots (if applicable)
[Add screenshots for UI changes]

---
Generated with [Claude Code](https://claude.ai/code)
```

**Linking PRs to Issues:**
- Use `Fixes #N` or `Closes #N` in the PR description to automatically close the issue when merged
- Use `Related to #N` or `Addresses #N` if the PR doesn't fully close the issue
- GitHub supports these keywords: `close`, `closes`, `closed`, `fix`, `fixes`, `fixed`, `resolve`, `resolves`, `resolved`

### 5. Review & Merge

**Review Process:**
1. **CI Checks**: Ensure all automated checks pass (tests, linting, builds)
2. **Code Review**: At least one team member should review the code
3. **Address Feedback**: Make requested changes and push new commits
4. **Final Approval**: Obtain approval from reviewer(s)

**Merge Guidelines:**
- **Squash and Merge**: Preferred for feature branches to keep history clean
- **Merge Message**: Should follow Conventional Emoji Commits format
- **Delete Branch**: After merging, delete the feature branch
- **Issue Closure**: Verify that linked issues are automatically closed

## Claude Code GitHub Actions Integration

### Triggering Claude Code

Claude Code can be triggered in several ways:

1. **Issue Comment**: Comment `@claude` on any issue
2. **Issue Assignment**: Assign an issue with the label `claude`
3. **PR Comment**: Comment `@claude` on any PR for assistance

### What Claude Code Does

When triggered on an issue:
1. **Reads the issue** to understand the request
2. **Creates a branch** automatically using the naming convention
3. **Makes code changes** based on the issue description
4. **Commits changes** following git-scm-conventions.md
5. **Pushes to remote** branch
6. **Provides PR link** with pre-filled title and body

### Claude Code Capabilities

**Can Do:**
- Answer questions about code
- Perform code reviews and provide feedback
- Implement code changes (simple to moderate complexity)
- Create pull requests
- Update documentation
- Run tests and fix issues

**Cannot Do:**
- Approve pull requests (security limitation)
- Merge branches
- Modify GitHub Actions workflows (permission limitation)

### Example Workflow with Claude Code

```
1. Create issue #42: "Add rate limiting to API endpoints"

2. Comment on issue: "@claude please implement rate limiting using express-rate-limit"

3. Claude Code:
   - Creates branch: claude/issue-42-20251026-1500
   - Implements rate limiting middleware
   - Adds tests
   - Updates documentation
   - Commits with proper format
   - Pushes changes
   - Comments with PR creation link

4. You:
   - Review Claude's changes
   - Click PR creation link
   - Submit PR
   - Review and merge

5. GitHub automatically closes issue #42
```

## Benefits

### Traceability
- Every change traces back to a specific issue
- Clear audit trail of decisions and implementations
- Easy to understand why changes were made

### Automation
- Automated branch creation and naming
- Consistent commit message formatting
- Automatic PR linking to issues
- Streamlined workflow reduces manual steps

### Consistency
- Standardized process across the team
- Predictable branch names and structure
- Uniform commit and PR formatting
- Easier onboarding for new team members

### History
- Complete context for every change
- Easier debugging and troubleshooting
- Better project documentation
- Simplified maintenance and updates

## Best Practices

### Issue Creation
- **Be specific**: Clearly describe the problem or feature
- **Include examples**: Provide code examples or expected behavior
- **Add acceptance criteria**: Define what "done" looks like
- **Use templates**: Create issue templates for common types

### Branch Management
- **One issue, one branch**: Each issue should have its own branch
- **Short-lived branches**: Merge frequently to avoid conflicts
- **Keep branches updated**: Regularly rebase or merge from main
- **Clean up**: Delete branches after merging

### Commit Practices
- **Atomic commits**: Each commit should represent one logical change
- **Clear messages**: Follow Conventional Emoji Commits format
- **Commit often**: Small, frequent commits are easier to review
- **Test before commit**: Ensure code works before committing

### Pull Request Etiquette
- **Small PRs**: Easier to review and less prone to conflicts
- **Self-review**: Review your own PR before requesting review
- **Respond promptly**: Address review feedback quickly
- **Update tests**: Always update or add tests for changes
- **Update docs**: Keep documentation in sync with code

## Troubleshooting

### Branch Name Conflicts
If you encounter a branch name conflict:
```bash
# Use a different timestamp or add a suffix
git checkout -b claude/issue-N-YYYYMMDD-HHMM-v2
```

### Claude Code Not Responding
If Claude Code doesn't respond to `@claude`:
1. Verify the repository has Claude Code GitHub Actions installed
2. Check that the issue or PR is in a public repository or Claude has access
3. Review GitHub Actions logs for errors
4. Try commenting again or create a new comment

### PR Not Closing Issue
If the PR doesn't automatically close the issue:
1. Verify you used the correct keyword (`Fixes #N`, `Closes #N`)
2. Ensure the keyword is in the PR description (not just comments)
3. Check that the issue number is correct
4. Manually close the issue after merging if needed

## Additional Resources

- [Conventional Emoji Commits Documentation](https://conventional-emoji-commits.site/)
- [GitHub Issues Documentation](https://docs.github.com/en/issues)
- [GitHub Pull Requests Documentation](https://docs.github.com/en/pull-requests)
- [Claude Code Documentation](https://docs.claude.ai/code)
- [Linking PRs to Issues](https://docs.github.com/en/issues/tracking-your-work-with-issues/linking-a-pull-request-to-an-issue)

## Summary

The GitHub issue-driven development workflow provides:
1. **Clear process**: Every change starts with an issue
2. **Automation**: Claude Code handles branch creation and boilerplate
3. **Traceability**: Full audit trail from issue to merged code
4. **Consistency**: Standardized naming, commits, and PRs
5. **Efficiency**: Streamlined workflow saves time and reduces errors

Follow this workflow for all development work to maintain consistency and quality across the Langstar project.
