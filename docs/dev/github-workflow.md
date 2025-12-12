# GitHub Issue-Driven Development Workflow

## Key Concept

1. Each github pull request should be opened from a feature branch that closes a single github issue.
2. Github issues use sub-tasks for hierarchy, representing parent child relationships.
   There may be more than one level to the hierarchy.
3. If github issue "#333 implement kingdom" depends on "#666 implement phylum" depends on #999 implement class, then
   user/999-implement-class will PR into user/666-implement-phylum, and user/666-implement-phylum will PR into user/333-implement-kingdom.

### Choosing PR Target: Main vs. Hierarchical Merging

Not all sub-issues require hierarchical merging. Choose your PR target based on the nature of the changes:

#### PR Directly to Main (Preferred for Most Work)

Use direct-to-main PRs for:

- **Incremental, non-breaking changes** - Research docs, small features, tests
- **Independent sub-tasks** - Work that doesn't require other sub-tasks to be complete first
- **Documentation and research** - Reports, specs, and reference materials
- **Bug fixes** - Isolated fixes that don't affect other in-progress work
- **Refactoring** - Changes that maintain backward compatibility

**Example**: A research sub-issue (#299) under an epic (#298) can PR directly to main because:

- The research report is standalone and useful immediately
- It doesn't break anything or require other code to exist first
- Other developers benefit from having it merged sooner

#### Hierarchical Merging (For Breaking Changes)

Use hierarchical merging (PR into parent branch) for:

- **Breaking API changes** - Changes that would break other in-progress branches
- **Interdependent features** - Sub-tasks that build on each other sequentially
- **Large refactors** - Structural changes that affect multiple components
- **Feature flags** - When changes must be released together atomically

**Example**: If implementing a new CLI command (#999) requires a new SDK method (#666), and the SDK method changes existing interfaces:

- PR #999 into #666's branch (not main)
- PR #666 into main only when both are complete
- This prevents main from having incomplete features

#### Decision Flowchart

```
Is this change breaking to main?
├── No → PR to main
└── Yes → Does it require other sub-tasks first?
    ├── No → PR to main (with feature flag if needed)
    └── Yes → PR to parent branch (hierarchical)
```

**Rule of thumb**: When in doubt, prefer PRing to main. Hierarchical merging adds complexity and should only be used when truly necessary.

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

### Epic and Sub-Issue Naming Conventions

For complex, multi-phase features, this project uses a hierarchical naming convention for issues. This helps maintain clear relationships between parent and child issues.

**See [Issue #258](https://github.com/codekiln/langstar/issues/258) for complete documentation.**

#### Three-Level Hierarchy

**Level 1: Epic/Milestone Issue**

- Descriptive title without prefix
- Example: `devcontainer-feature milestone - Create devcontainer feature for langstar CLI installation` (#201)
- Create matching GitHub Milestone
- Attach milestone to issue

**Level 2: Phase Issues** (children of epic)

- Format: `{epic-num}.{phase-num}-{slug} {description}`
- Example: `201.3-devcontainer-feature-ci automated CI testing for devcontainer features` (#240)
- Attach same milestone as epic
- Reference parent issue in description

**Level 3: Task Issues** (children of phase)

- Format: `{phase-num}.{task-num}-{slug} {description}`
- Example: `240.1-devcontainer-feature Create Automated Testing Workflow` (#247)
- Attach same milestone as epic and phase
- Reference parent phase in description

#### Key Requirements

1. **ALL issues at ALL levels must have the milestone attached**
   - Enables accurate progress tracking
   - Allows filtering all related work in milestone view
   - Supports project management and burndown charts

2. **Use `gh-sub-issue` skill to establish formal parent-child relationships**
   ```bash
   # Link existing issue as sub-issue
   gh sub-issue add <parent-issue-num> <child-issue-num>

   # Create new sub-issue
   gh sub-issue create --parent <parent-num> --title "Issue title"
   ```
   See `.claude/skills/gh-sub-issue/SKILL.md` for details

#### Complete Example

```
Epic: #201 (devcontainer-feature milestone)
├── Milestone: "devcontainer-feature"
└── Phase: #240 (201.3-devcontainer-feature-ci)
    ├── Milestone: "devcontainer-feature"
    └── Tasks:
        ├── #247: 240.1-devcontainer-feature Create Automated Testing Workflow
        ├── #248: 240.2-devcontainer-feature Implement Dev Container CLI Testing
        └── ... (all with milestone: "devcontainer-feature")
```

**Note**: This convention evolved over the project's lifetime. Earlier issues may not follow this pattern, which is acceptable for historical issues. Use this convention for all new epics going forward.

---

## Step 2: Branch Creation

### Branch Naming Convention

Branches use prefixes to indicate milestone, parent issue, and issue number:

**Prefix Meanings:**

- `m<id>` = milestone ID
- `p<id>` = parent issue ID
- `i<id>` = GitHub issue number

**Format Combinations:**

1. **With milestone and parent issue:**
   - Format: `m<milestone_id>-p<parent_id>-i<issue_num>-<issue_slug>`
   - Example: `m8-p123-i234-add-user-auth` (milestone #8, parent #123, issue #234)

2. **With parent issue only (no milestone):**
   - Format: `p<parent_id>-i<issue_num>-<issue_slug>`
   - Example: `p123-i234-add-user-auth` (parent #123, issue #234)

3. **With milestone only (no parent):**
   - Format: `m<milestone_id>-i<issue_num>-<issue_slug>`
   - Example: `m8-i234-add-user-auth` (milestone #8, issue #234)

4. **Neither milestone nor parent:**
   - Format: `i<issue_num>-<issue_slug>`
   - Example: `i234-add-user-auth` (issue #234)

### Creating Branches

#### Option 1: Manual Branch Creation

```bash
# Create and switch to a new branch
git checkout -b <branch_name>

# Example: issue #234 in milestone #8 with parent #123
git checkout -b m8-p123-i234-add-user-auth

# Example: standalone issue #42
git checkout -b i42-fix-database-connection

# Push the branch to remote
git push -u origin <branch_name>
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
git push origin <branch_name>
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

**IMPORTANT: Always Add Milestone**
If the related issue has a milestone attached, **you MUST add the same milestone to the PR**:

- Use GitHub web UI: Settings → Milestone
- Use GitHub CLI: `gh pr edit <pr-num> --milestone "<milestone-name>"`

**Why milestones matter:**

- Enables progress tracking across the epic
- Allows filtering all PRs related to a milestone
- Supports project management and burndown charts
- Maintains consistency with issue hierarchy

**Example:**

```bash
# After creating PR #421 for issue #373
gh pr edit 421 --milestone "ls-evals-basic"
```

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
git branch -d <branch_name>
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

- Follow the naming convention: `m<milestone>-p<parent>-i<issue>-<slug>` (with appropriate variations)
- Include milestone and parent prefixes when applicable
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
git checkout <branch_name>

# Pull latest changes
git pull origin <branch_name>
```

### Merge Conflicts

If you encounter merge conflicts:

```bash
# Update your branch with latest main
git checkout main
git pull origin main
git checkout <branch_name>
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
# Assuming issue #7 is standalone (no milestone or parent)
git checkout -b i7-document-workflow
git push -u origin i7-document-workflow
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
git push origin i7-document-workflow
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
