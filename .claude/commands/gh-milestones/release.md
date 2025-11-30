---
description: Mark milestone as done and update parent issue with release information
args: <milestone> <version>
---

# Mark Milestone as Released

Automates the process of marking milestones as completed and updating related issues when a milestone is released.

## Critical Constraints - Session Statelessness

**IMPORTANT:** You are operating in a stateless session. Each Claude Code session is isolated.

**You CANNOT:**
- Track issues across sessions
- Remember to do something later
- Follow up on tasks in the future
- Promise to handle something "in a follow-up"

**You MUST complete all milestone release steps in this session or notify the user of any blockers immediately.**

## Arguments

Arguments are passed via `$ARGUMENTS` in the format:
```
<milestone> <version>
```

**Required:**
- `milestone`: Either a GitHub milestone URL or a milestone name (string)
- `version`: The release version tag (e.g., `v0.4.1`)

**Parsing rules:**
- If first argument starts with `https://github.com/`: treat as URL and extract milestone number
- Otherwise: treat as milestone name
- Second argument is always the version string

**Examples:**
```bash
# Using milestone URL
/gh-milestones/release https://github.com/codekiln/langstar/milestone/5 v0.4.1

# Using milestone name
/gh-milestones/release "devcontainer-feature" v0.4.1

# With single-word milestone name (no quotes needed)
/gh-milestones/release ls-evals-basic v0.10.0
```

## User Input

```text
$ARGUMENTS
```

## Implementation

### Step 1: Get Current Repository

First, detect the current repository owner and name (needed for subsequent API calls):

```bash
REPO_INFO=$(gh repo view --json owner,name --jq '{owner: .owner.login, name: .name}')
OWNER=$(echo "$REPO_INFO" | jq -r '.owner')
REPO=$(echo "$REPO_INFO" | jq -r '.name')
```

### Step 2: Parse Arguments

Extract milestone identifier and version from `$ARGUMENTS`:

```bash
# Read arguments (using read to properly handle quoted milestone names with spaces)
read -r MILESTONE_ARG VERSION <<< "$ARGUMENTS"

# Validate version format (should start with 'v')
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "❌ Error: Version must be in format vX.Y.Z (e.g., v0.4.1)"
  exit 1
fi

# Determine if milestone is URL or name
if [[ "$MILESTONE_ARG" =~ ^https://github\.com/ ]]; then
  # Extract milestone number from URL
  # URL format: https://github.com/owner/repo/milestone/NUMBER
  MILESTONE_NUM=$(echo "$MILESTONE_ARG" | grep -oE '[0-9]+$')
  if [ -z "$MILESTONE_NUM" ]; then
    echo "❌ Error: Could not extract milestone number from URL"
    exit 1
  fi
else
  # Treat as milestone name - will resolve using $OWNER/$REPO
  MILESTONE_NAME="$MILESTONE_ARG"
fi
```

**Important:** If milestone is provided as a name, look it up using the repository info:

```bash
if [ -n "$MILESTONE_NAME" ]; then
  # List milestones and find matching name
  MILESTONE_NUM=$(gh api "repos/$OWNER/$REPO/milestones" --jq ".[] | select(.title == \"$MILESTONE_NAME\") | .number")

  if [ -z "$MILESTONE_NUM" ]; then
    echo "❌ Error: Milestone '$MILESTONE_NAME' not found"
    exit 1
  fi
fi
```

### Step 3: Validate Release Exists

Before proceeding, verify the GitHub release exists and is published:

```bash
# Check if release exists
RELEASE_INFO=$(gh release view "$VERSION" --json tagName,url,isDraft,isPrerelease 2>&1)

if [ $? -ne 0 ]; then
  echo "❌ Error: Release '$VERSION' not found in repository"
  echo "   Please create the release first: gh release create $VERSION"
  exit 1
fi

# Check if release is a draft
IS_DRAFT=$(echo "$RELEASE_INFO" | jq -r '.isDraft')
if [ "$IS_DRAFT" = "true" ]; then
  echo "⚠️  Warning: Release '$VERSION' is still a draft"
  echo "   Consider publishing it first"
fi

# Get release URL for later use
RELEASE_URL=$(echo "$RELEASE_INFO" | jq -r '.url')
```

### Step 4: Get Milestone Details

Fetch milestone information including associated issues:

```bash
# Get milestone details
MILESTONE_DATA=$(gh api "repos/$OWNER/$REPO/milestones/$MILESTONE_NUM" \
  --jq '{title: .title, state: .state, description: .description, number: .number}')

MILESTONE_TITLE=$(echo "$MILESTONE_DATA" | jq -r '.title')
MILESTONE_STATE=$(echo "$MILESTONE_DATA" | jq -r '.state')
MILESTONE_DESC=$(echo "$MILESTONE_DATA" | jq -r '.description // ""')

echo "📍 Milestone: $MILESTONE_TITLE (#$MILESTONE_NUM)"
echo "   State: $MILESTONE_STATE"
```

### Step 5: Find Parent Issue

Find the issue associated with this milestone (typically the epic/parent issue):

```bash
# List all issues with this milestone and get the lowest-numbered one
# Heuristic: The parent issue is usually the lowest-numbered issue with this milestone
PARENT_ISSUE=$(gh issue list --milestone "$MILESTONE_TITLE" --json number,title,state --jq 'sort_by(.number) | .[0].number')

if [ -z "$PARENT_ISSUE" ]; then
  echo "❌ Error: Could not find parent issue for milestone '$MILESTONE_TITLE'"
  echo "   No issues found with this milestone attached"
  exit 1
fi

PARENT_TITLE=$(gh issue list --milestone "$MILESTONE_TITLE" --json number,title,state --jq 'sort_by(.number) | .[0].title')

echo "🔗 Parent Issue: #$PARENT_ISSUE - $PARENT_TITLE"
```

### Step 6: Validate All Sub-Issues Are Closed

Check if all child issues of the parent are closed:

```bash
echo ""
echo "🔍 Validating sub-issue completion..."

# Check if gh-sub-issue extension is available
if ! gh extension list | grep -q "gh-sub-issue"; then
  echo "⚠️  Warning: gh-sub-issue extension not installed"
  echo "   Skipping sub-issue validation"
  echo "   Install with: gh extension install https://github.com/cli/gh-sub-issue"
  SKIP_SUBISSUE_CHECK=true
else
  # List all sub-issues of the parent
  SUB_ISSUES=$(gh sub-issue list "$PARENT_ISSUE" --relation children --state all --json number,title,state 2>&1)

  if [ $? -ne 0 ]; then
    echo "⚠️  Warning: Could not fetch sub-issues for #$PARENT_ISSUE"
    echo "   Skipping sub-issue validation"
    SKIP_SUBISSUE_CHECK=true
  else
    # Check if any sub-issues are open
    OPEN_COUNT=$(echo "$SUB_ISSUES" | jq '[.[] | select(.state == "OPEN")] | length')
    TOTAL_COUNT=$(echo "$SUB_ISSUES" | jq 'length')

    echo "   Total sub-issues: $TOTAL_COUNT"
    echo "   Open sub-issues: $OPEN_COUNT"
    echo ""

    if [ "$OPEN_COUNT" -gt 0 ]; then
      echo "⚠️  Warning: Found $OPEN_COUNT open sub-issue(s):"
      echo "$SUB_ISSUES" | jq -r '.[] | select(.state == "OPEN") | "   - #\(.number): \(.title)"'
      echo ""
      echo "It's recommended to close all sub-issues before releasing the milestone."
      echo ""
      echo "To continue despite open sub-issues, set FORCE_RELEASE=true and rerun:"
      echo "  FORCE_RELEASE=true /gh-milestones/release ..."
      echo ""

      # Check if user explicitly set FORCE_RELEASE to override
      if [ "$FORCE_RELEASE" != "true" ]; then
        echo "❌ Aborted: Open sub-issues detected"
        exit 1
      fi
      echo "⚠️  FORCE_RELEASE=true detected, continuing despite open sub-issues..."
    else
      echo "✅ All sub-issues are closed"
    fi
  fi
fi
```

**Note:** If `gh-sub-issue` extension is not available, skip this validation step with a warning.

### Step 7: Update Milestone Description

Prepend release information to the milestone description:

```bash
echo ""
echo "📝 Updating milestone description..."

# Construct new description (prepend release info)
NEW_DESC="Shipped in [release $VERSION]($RELEASE_URL)

$MILESTONE_DESC"

# Update milestone description and close it
gh api "repos/$OWNER/$REPO/milestones/$MILESTONE_NUM" \
  -X PATCH \
  -f description="$NEW_DESC" \
  -f state="closed"

if [ $? -eq 0 ]; then
  echo "✅ Milestone description updated"
  echo "✅ Milestone marked as closed"
else
  echo "❌ Error: Failed to update milestone"
  exit 1
fi
```

### Step 8: Close Parent Issue with Release Comment

Add a release comment to the parent issue and close it:

```bash
echo ""
echo "📝 Updating parent issue #$PARENT_ISSUE..."

# Create comment body
COMMENT_BODY="Shipped in [release $VERSION]($RELEASE_URL)"

# Add comment to issue
gh issue comment "$PARENT_ISSUE" --body "$COMMENT_BODY"

if [ $? -eq 0 ]; then
  echo "✅ Release comment added to issue #$PARENT_ISSUE"
else
  echo "⚠️  Warning: Failed to add comment to issue"
fi

# Close the issue if not already closed
PARENT_STATE=$(gh issue view "$PARENT_ISSUE" --json state --jq '.state')

if [ "$PARENT_STATE" = "OPEN" ]; then
  gh issue close "$PARENT_ISSUE"

  if [ $? -eq 0 ]; then
    echo "✅ Parent issue #$PARENT_ISSUE closed"
  else
    echo "❌ Error: Failed to close parent issue"
    exit 1
  fi
else
  echo "ℹ️  Parent issue #$PARENT_ISSUE was already closed"
fi
```

### Step 9: Display Summary

Show a comprehensive summary of what was done:

```
✅ **Milestone Release Tracking Complete**

📍 Milestone: $MILESTONE_TITLE (#$MILESTONE_NUM)
🔗 Parent Issue: #$PARENT_ISSUE - $PARENT_TITLE
📦 Release: $VERSION
🔗 Release URL: $RELEASE_URL

**Actions Completed:**
✅ Verified release $VERSION exists
✅ Validated sub-issue completion (if applicable)
✅ Milestone marked as closed
✅ Milestone description updated with release information
✅ Parent issue #$PARENT_ISSUE closed with release comment

**View Updated Milestone:**
https://github.com/$OWNER/$REPO/milestone/$MILESTONE_NUM

**View Parent Issue:**
https://github.com/$OWNER/$REPO/issues/$PARENT_ISSUE
```

## Error Handling

### Common Errors

**Invalid version format:**
```
❌ Error: Version must be in format vX.Y.Z (e.g., v0.4.1)
```
- Ensure version starts with 'v' and follows semantic versioning

**Milestone not found:**
```
❌ Error: Milestone 'milestone-name' not found
```
- Check milestone name spelling
- Verify milestone exists: `gh api repos/$OWNER/$REPO/milestones` (e.g., `gh api repos/octocat/Hello-World/milestones`)
- If using URL, ensure URL is correct format

**Release not found:**
```
❌ Error: Release 'v0.4.1' not found in repository
   Please create the release first: gh release create v0.4.1
```
- Create the GitHub release before running this command
- Verify release tag exists: `gh release list`

**Parent issue not found:**
```
❌ Error: Could not find parent issue for milestone 'milestone-name'
   No issues found with this milestone attached
```
- Ensure at least one issue has the milestone attached
- The issue with the lowest number is assumed to be the parent
- Manually verify issues: `gh issue list --milestone "milestone-name"`

**Authentication error:**
```
❌ Error: HTTP 401: Unauthorized
```
- Check GitHub CLI authentication: `gh auth status`
- Re-authenticate if needed: `gh auth login`

**Permission error:**
```
❌ Error: HTTP 403: Forbidden
```
- Verify you have write access to the repository
- Ensure you have permission to close issues and edit milestones

### Handling Edge Cases

**Milestone already closed:**
- The command will still update the description and parent issue
- Output will note: `ℹ️  Milestone was already closed`

**Parent issue already closed:**
- The command will still add the release comment
- Output will note: `ℹ️  Parent issue #N was already closed`

**Release is a draft:**
- Command will warn but continue: `⚠️  Warning: Release 'vX.Y.Z' is still a draft`
- Consider publishing the release first for consistency

**Sub-issues are open:**
- Command will warn and ask for confirmation
- If you proceed, milestone will still be closed
- Best practice: close all sub-issues before releasing

**gh-sub-issue extension not installed:**
- Sub-issue validation will be skipped with a warning
- All other steps will complete normally
- Install extension: Check `.claude/skills/gh-sub-issue/SKILL.md` for installation

## Example Workflow

### Example 1: Using Milestone URL

```bash
# Command
/gh-milestones/release https://github.com/codekiln/langstar/milestone/5 v0.4.1

# Output
📍 Milestone: devcontainer-feature (#5)
   State: open
🔗 Parent Issue: #201 - devcontainer-feature milestone

🔍 Validating sub-issue completion...
   Total sub-issues: 3
   Open sub-issues: 0

✅ All sub-issues are closed

📝 Updating milestone description...
✅ Milestone description updated
✅ Milestone marked as closed

📝 Updating parent issue #201...
✅ Release comment added to issue #201
✅ Parent issue #201 closed

✅ **Milestone Release Tracking Complete**

📍 Milestone: devcontainer-feature (#5)
🔗 Parent Issue: #201 - devcontainer-feature milestone
📦 Release: v0.4.1
🔗 Release URL: https://github.com/codekiln/langstar/releases/tag/v0.4.1

**Actions Completed:**
✅ Verified release v0.4.1 exists
✅ Validated sub-issue completion
✅ Milestone marked as closed
✅ Milestone description updated with release information
✅ Parent issue #201 closed with release comment
```

### Example 2: Using Milestone Name with Open Sub-Issues

```bash
# Command
/gh-milestones/release ls-evals-basic v0.10.0

# Output
📍 Milestone: ls-evals-basic (#8)
   State: open
🔗 Parent Issue: #298 - ls-evals-basic milestone

🔍 Validating sub-issue completion...
   Total sub-issues: 4
   Open sub-issues: 1

⚠️  Warning: Found 1 open sub-issue(s):
   - #305: Implement batch evaluation CLI

It's recommended to close all sub-issues before releasing the milestone.
Continue anyway? (y/n): y

📝 Updating milestone description...
✅ Milestone description updated
✅ Milestone marked as closed

📝 Updating parent issue #298...
✅ Release comment added to issue #298
✅ Parent issue #298 closed

✅ **Milestone Release Tracking Complete**
[... rest of output ...]
```

## Integration with Project Workflow

This command is typically used after:
1. **All work for the milestone is complete** - All features implemented, tests passing
2. **PR is merged to main** - The milestone's code is now in the main branch
3. **Release is created** - GitHub release has been published with release notes

**Typical release workflow:**
```bash
# 1. Merge final PR for milestone
gh pr merge 385 --squash

# 2. Create GitHub release (or automated via CI)
gh release create v0.4.1 --generate-notes

# 3. Mark milestone as released
/gh-milestones/release "milestone-name" v0.4.1
```

## See Also

- **GitHub Workflow Documentation** - `@docs/dev/github-workflow.md`
- **gh-sub-issue skill** - `.claude/skills/gh-sub-issue/SKILL.md`
- **Milestone Management** - `@docs/dev/github-projects.md`
- **Release Workflow** - `.github/workflows/auto-tag-release.yml`
