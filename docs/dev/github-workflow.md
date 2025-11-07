# GitHub Issue-Driven Development Workflow

## Key Concept

1. Each github pull request should be opened from a feature branch that closes a single github issue.
2. Github issues use sub-tasks for hierarchy, representing parent child relationships.
   There may be more than one level to the hierarchy. 
3. If github issue "#333 implement kingdom" depends on "#666 implement phylum" depends on #999 implement class, then
   user/999-implement-class will PR into user/666-implement-phylum, and user/666-implement-phylum will PR into user/333-implement-kingdom.

## Workflow Overview

1. **Create GitHub Issue** - Document the feature, bug, or task. Ensure parent child issue relationships are in place, if applicable.
2. **Branch Creation** - Create or use automated branch creation for the issue
3. **Development** - Implement changes on the issue branch
4. **Pull Request** - Submit a PR referencing the original issue
5. **Review & Merge** - Review, approve, and merge the PR (closes the issue)

---

## Step 1: Create GitHub Issue

Start by creating a GitHub issue that describes what needs to be done.

### Issue Best Practices

- **Clear title**: Use descriptive titles (e.g., "Add user authentication" vs "Fix bug")
- **Description**: Provide context, requirements, and acceptance criteria
- **Labels**: Apply appropriate labels (bug, enhancement, documentation, etc.)
- **Assignment**: Assign to yourself or relevant team member
- **References**: Link to related issues or PRs using `#issue-number`

### Example Issue

```markdown
## Description
Implement user authentication using JWT tokens

## Requirements
- [ ] Create login endpoint
- [ ] Create registration endpoint
- [ ] Implement JWT token generation
- [ ] Add authentication middleware

## Acceptance Criteria
- Users can register with email/password
- Users can login and receive JWT token
- Protected routes require valid JWT token
```

---

## Step 2: Branch Creation

### Branch Naming Convention

Branches should follow this format:

```
<username>/<issue_num>-<issue_slug>
```

**Components:**
- `<username>`: Your GitHub username (or `claude` for Claude Code)
- `<issue_num>`: The issue number (e.g., `7`, `42`)
- `<issue_slug>`: A short, kebab-case description derived from the issue title

**Examples:**
- `alice/7-add-user-authentication`
- `bob/42-fix-database-connection`
- `claude/15-update-documentation`

### Creating Branches

#### Option 1: Manual Branch Creation

```bash
# Create and switch to a new branch
git checkout -b <username>/<issue_num>-<issue_slug>

# Example
git checkout -b alice/7-add-user-authentication

# Push the branch to remote
git push -u origin <username>/<issue_num>-<issue_slug>
```

#### Option 2: Claude Code GitHub Actions

Claude Code can automatically create branches when you mention `@claude` in an issue comment:

```markdown
@claude can you start on this?
```

**What Claude Code Does:**
1. Creates a branch automatically
2. Checks out the branch in the workflow environment
3. Can make initial commits if requested
4. Provides a link to create a PR when work is complete

**Claude Code Branch Format:**
When Claude creates branches, it uses a different format than the project convention:
```
claude/issue-<issue_num>-<timestamp>
```

**Example:** `claude/issue-11-20251026-1529`

**Note:** This format differs from the project's preferred convention of `<username>/<issue_num>-<issue_slug>`. The Claude Code Action currently does not support customizing the full branch name pattern beyond the prefix. See [Issue #11](https://github.com/codekiln/langstar/issues/11) for details.

For branches that need to strictly follow project conventions, use manual branch creation (Option 1 above).

---

## Step 3: Development

Make your changes on the issue branch, following the project's coding conventions.

### Commit Guidelines

This project uses **Conventional Emoji Commits**. See [git-scm-conventions.md](./git-scm-conventions.md) for full details.

**Commit Format:**
```
<emoji> <type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Common Types:**
- `✨ feat` - New feature
- `🩹 fix` - Bug fix
- `📚 docs` - Documentation
- `♻️ refactor` - Code refactoring
- `🧪 test` - Tests
- `🔧 build` - Build system changes

**Examples:**
```bash
git commit -m "✨ feat(auth): add JWT authentication"
git commit -m "🩹 fix: resolve database connection timeout"
git commit -m "📚 docs: update API documentation"
```

### Making Changes

```bash
# Make your code changes
# ...

# Stage changes
git add .

# Commit with conventional format
git commit -m "✨ feat: add user authentication endpoints"

# Push to remote
git push origin <username>/<issue_num>-<issue_slug>
```

---

## Step 4: Pull Request

When your changes are ready, create a pull request.

### Creating a PR

#### Option 1: GitHub Web Interface

1. Navigate to the repository on GitHub
2. Click "Pull requests" → "New pull request"
3. Select your branch as the compare branch
4. Fill in the PR template (if available)
5. Click "Create pull request"

#### Option 2: GitHub CLI

```bash
gh pr create --title "✨ feat: add user authentication" \
  --body "Fixes #7

## Summary
- Implemented JWT authentication
- Added login and registration endpoints
- Added authentication middleware

## Test Plan
- [x] Manual testing of login flow
- [x] Manual testing of registration
- [x] Verified protected routes require auth"
```

#### Option 3: Claude Code Generated Link

When Claude Code completes work, it provides a pre-filled PR creation link:

```markdown
[Create PR →](https://github.com/owner/repo/compare/main...branch?quick_pull=1&title=...&body=...)
```

### PR Best Practices

**Title Format:**
Follow the same conventional commit format:
```
<emoji> <type>[scope]: <description>
```

**PR Description Should Include:**
- **Summary**: What changes were made
- **Related Issues**: Use keywords to link issues (see below)
- **Test Plan**: How the changes were tested
- **Screenshots**: If UI changes are involved
- **Breaking Changes**: If applicable

### Linking PRs to Issues

Use GitHub keywords in your PR description to automatically close issues when the PR is merged:

**Keywords:**
- `Fixes #7`
- `Closes #7`
- `Resolves #7`

**Example PR Description:**
```markdown
## Summary
Implements user authentication using JWT tokens.

## Changes
- Added login endpoint at `/api/auth/login`
- Added registration endpoint at `/api/auth/register`
- Implemented JWT token generation and validation
- Added authentication middleware for protected routes

## Related Issues
Fixes #7

## Test Plan
- [x] Tested login with valid credentials
- [x] Tested login with invalid credentials
- [x] Tested registration with new user
- [x] Verified JWT token validation
- [x] Verified protected routes require authentication

---
Generated with [Claude Code](https://claude.ai/code)
```

---

## Step 5: Review & Merge

### Code Review Process

1. **Automated Checks**: Ensure CI/CD pipelines pass
2. **Peer Review**: Request reviews from team members
3. **Address Feedback**: Make requested changes and push updates
4. **Approval**: Obtain required approvals
5. **Merge**: Merge the PR using the appropriate strategy

### Merge Strategies

- **Squash and merge** (recommended): Combines all commits into one
- **Rebase and merge**: Maintains individual commits
- **Merge commit**: Creates a merge commit

### After Merging

1. **Issue Closure**: If you used `Fixes #N`, the issue closes automatically
2. **Branch Cleanup**: Delete the merged branch (GitHub offers this option)
3. **Local Cleanup**: Update your local repository

```bash
# Switch to main branch
git checkout main

# Pull latest changes
git pull origin main

# Delete local branch
git branch -d <username>/<issue_num>-<issue_slug>
```

---

## Claude Code GitHub Actions Integration

### Capabilities

Claude Code can assist with:

1. **Branch Creation**: Automatically creates appropriately named branches
2. **Code Implementation**: Makes changes based on issue requirements
3. **Commit Creation**: Creates well-formatted commits following conventions
4. **PR Preparation**: Provides pre-filled PR creation links

### Triggering Claude Code

Mention `@claude` in an issue or PR comment:

```markdown
@claude can you implement the user authentication feature?
```

```markdown
@claude please fix the failing tests in this PR
```

```markdown
@claude can you review this code for security issues?
```

### What Claude Code Does

1. **Analyzes Context**: Reads the issue/PR description and comments
2. **Creates Plan**: Breaks down work into tasks
3. **Implements Changes**: Makes code changes following project conventions
4. **Commits & Pushes**: Creates commits and pushes to the branch
5. **Reports Progress**: Updates the comment with progress and results
6. **Provides PR Link**: Gives you a pre-filled PR creation link

### Limitations

Claude Code **cannot**:
- Submit formal GitHub PR reviews
- Approve pull requests
- Merge pull requests
- Modify workflow files in `.github/workflows`

For more information, see the [Claude Code GitHub Actions FAQ](https://github.com/anthropics/claude-code-action/blob/main/docs/faq.md).

---

## Best Practices

### For Issues

- Create issues before starting work
- Use clear, descriptive titles
- Include acceptance criteria
- Add appropriate labels
- Reference related issues

### For Branches

- Follow the naming convention: `<username>/<issue_num>-<issue_slug>`
- Create branches from the latest main branch
- Keep branches focused on a single issue
- Delete branches after merging

### For Commits

- Follow Conventional Emoji Commits format
- Write clear, descriptive commit messages
- Make atomic commits (one logical change per commit)
- Reference issue numbers in commit messages when relevant

### For Pull Requests

- Use descriptive titles following conventional format
- Fill out PR description templates completely
- Link to related issues using keywords
- Keep PRs focused and reasonably sized
- Request reviews from appropriate team members
- Respond promptly to review feedback

---

## Troubleshooting

### Branch Already Exists

If a branch already exists for an issue:

```bash
# Check out the existing branch
git checkout <username>/<issue_num>-<issue_slug>

# Pull latest changes
git pull origin <username>/<issue_num>-<issue_slug>
```

### Merge Conflicts

If you encounter merge conflicts:

```bash
# Update your branch with latest main
git checkout main
git pull origin main
git checkout <username>/<issue_num>-<issue_slug>
git merge main

# Resolve conflicts in your editor
# Stage resolved files
git add .

# Complete the merge
git commit
```

### CI/CD Failures

If automated checks fail:

1. Review the error messages in the GitHub Actions logs
2. Fix the issues locally
3. Commit and push the fixes
4. Wait for checks to run again

---

## Example Workflow

Here's a complete example from start to finish:

### 1. Create Issue

**Issue #7**: "Document GitHub issue-driven development workflow"

### 2. Create Branch

```bash
git checkout -b alice/7-document-workflow
git push -u origin alice/7-document-workflow
```

Or mention `@claude` in the issue to have Claude create the branch.

### 3. Make Changes

```bash
# Create documentation file
touch docs/dev/github-workflow.md
# Edit the file...

# Stage and commit
git add docs/dev/github-workflow.md
git commit -m "📚 docs: add GitHub workflow documentation

Fixes #7"

# Push changes
git push origin alice/7-document-workflow
```

### 4. Create Pull Request

```bash
gh pr create --title "📚 docs: add GitHub workflow documentation" \
  --body "## Summary
Adds comprehensive documentation for the GitHub issue-driven development workflow.

## Changes
- Created docs/dev/github-workflow.md with complete workflow guide
- Documented branch naming conventions
- Included Claude Code integration details

## Related Issues
Fixes #7

## Test Plan
- [x] Documentation is complete and accurate
- [x] All links work correctly
- [x] Examples are clear and helpful"
```

### 5. Review & Merge

1. Team members review the PR
2. Address any feedback
3. Obtain approval
4. Merge the PR
5. Issue #7 automatically closes

---

## Summary

This workflow ensures:
- Every change is tracked to an issue
- Branches follow a consistent naming convention
- Commits use conventional format for clarity
- PRs are well-documented and properly linked
- The team maintains a clear history of all work

By following this process, we maintain high code quality, clear communication, and efficient collaboration.
- Each PR should fix exactly one github issue. The first line of the PR and the PR commit should contain special github syntax which, upon PR merge, will mark the issue as fixed. As per [Using keywords in issues and pull requests - GitHub Enterprise Cloud Docs](https://docs.github.com/en/enterprise-cloud@latest/get-started/writing-on-github/working-with-advanced-formatting/using-keywords-in-issues-and-pull-requests), any of the following can be used: 
close
closes
closed
fix
fixes
fixed
resolve
resolves
resolved
For example, Closes #10 or Fixes octo-org/octo-repo#100.