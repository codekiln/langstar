---
description: Create a Phase 0.0 scout issue to validate feasibility before committing to a milestone
args: <feature-name>
---

# Scout Milestone Feasibility (Phase 0.0)

Automates the creation of a pre-epic scouting issue to validate feasibility of a new API feature before committing to a full 8-phase milestone.

## Critical Constraints - Session Statelessness

**IMPORTANT:** You are operating in a stateless session. Each Claude Code session is isolated.

**You CANNOT:**
- Track issues across sessions
- Remember to do something later
- Follow up on tasks in the future
- Promise to handle something "in a follow-up"

**You MUST complete all scout issue creation steps in this session or notify the user of any blockers immediately.**

## Arguments

Arguments are passed via `$ARGUMENTS` in the format:
```
<feature-name>
```

**Required:**
- `feature-name`: Short descriptive name of the feature (e.g., "structured-output-prompts", "dataset-management")

## User Input

```text
$ARGUMENTS
```

## Overview

This command creates a **Phase 0.0 scout issue** following the pattern established in [Issue #398](https://github.com/codekiln/langstar/issues/398). Scout issues validate feasibility *before* creating a full milestone, reducing risk and informing scope.

**When to use this command:**
- ✅ Adding support for a new LangSmith/LangGraph API feature
- ✅ Implementing functionality with unclear API complexity
- ✅ Uncertain if langstar already partially implements the feature
- ✅ Need to validate feasibility before committing to full 8-phase process

**When NOT to use:**
- ❌ Bug fixes (scope is already known)
- ❌ Small enhancements to existing commands
- ❌ Infrastructure changes
- ❌ Documentation-only changes

## Implementation

### Step 1: Parse Arguments

```bash
# Read feature name from arguments
read -r FEATURE_NAME <<< "$ARGUMENTS"

# Validate feature name provided
if [ -z "$FEATURE_NAME" ]; then
  echo "❌ Error: Feature name is required"
  echo "Usage: /ls-scout-milestone <feature-name>"
  echo "Example: /ls-scout-milestone dataset-management"
  exit 1
fi

# Create slug from feature name (replace spaces/underscores with hyphens, lowercase)
FEATURE_SLUG=$(echo "$FEATURE_NAME" | tr '[:upper:]' '[:lower:]' | tr ' _' '-' | sed 's/--*/-/g')

echo "📍 Feature: $FEATURE_NAME"
echo "🏷️  Slug: $FEATURE_SLUG"
```

### Step 2: Confirm Scout Creation

Display what will be created and confirm with user:

```
📋 **Scout Issue to Create:**

**Title**: [Scout] Research $FEATURE_NAME feasibility and API patterns

**Purpose**: Exploratory research to validate feasibility before committing to full milestone

**Deliverables**:
- Research report at docs/research/{issue-num}-${FEATURE_SLUG}-scout.md
- Updated SDK notes in reference/repo/langchain-ai/langsmith-sdk/notes/
- Optional experiments in reference/experiments/{issue-num}-${FEATURE_SLUG}/

**Labels**: research, scout

**Note**: This issue will NOT have a milestone (created before milestone exists)

Proceed with scout issue creation?
```

Wait for user confirmation before proceeding.

### Step 3: Load Scout Issue Template

```bash
# Load the scout issue template
TEMPLATE_PATH="docs/templates/scout-issue-template.md"

if [ ! -f "$TEMPLATE_PATH" ]; then
  echo "❌ Error: Scout issue template not found at $TEMPLATE_PATH"
  exit 1
fi

# Read template and substitute feature name
ISSUE_BODY=$(cat "$TEMPLATE_PATH" | sed "s/{feature-name}/$FEATURE_NAME/g")
```

### Step 4: Create Scout Issue

```bash
# Create GitHub issue with scout template
ISSUE_URL=$(gh issue create \
  --title "[Scout] Research $FEATURE_NAME feasibility and API patterns" \
  --body "$ISSUE_BODY" \
  --label "research,scout")

# Extract issue number from URL
ISSUE_NUM=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')

if [ -z "$ISSUE_NUM" ]; then
  echo "❌ Error: Failed to create scout issue"
  exit 1
fi

echo "✅ **Scout Issue Created:** #$ISSUE_NUM"
echo "🔗 URL: $ISSUE_URL"
```

### Step 5: Create Research Directory Structure

```bash
# Create research directory if it doesn't exist
mkdir -p "docs/research"

# Create placeholder research report file
RESEARCH_FILE="docs/research/${ISSUE_NUM}-${FEATURE_SLUG}-scout.md"

cat > "$RESEARCH_FILE" <<EOF
# $FEATURE_NAME Feasibility Scout

**Issue**: #${ISSUE_NUM}
**Date**: $(date +%Y-%m-%d)
**Status**: In Progress

---

## Executive Summary

<!-- To be completed after research -->

**Feasibility Assessment**: [Go / No-Go / Conditional]

**Key Findings**:
- TBD

**Recommendation**: TBD

---

## 1. Existing Langstar Implementation

<!-- Check ./cli and ./sdk for partial implementations -->

### CLI Commands

### SDK Types and Methods

---

## 2. LangSmith Python SDK Precedent

<!-- Analyze python/langsmith/client.py -->

### Relevant Methods

### Request/Response Shapes

### Key Patterns

---

## 3. API Endpoints

<!-- Identify REST API endpoints -->

### Endpoints Used

### Request Format

### Response Format

---

## 4. Complexity Assessment

<!-- Rate: Low / Medium / High -->

**Complexity**: TBD

**Technical Challenges**:
- TBD

**Dependencies**:
- TBD

---

## 5. Feasibility Recommendation

### Go / No-Go / Conditional

<!-- Provide clear recommendation -->

**Rationale**: TBD

**Conditions (if conditional)**: TBD

**Next Steps**: TBD

---

## References

- Scout issue: #${ISSUE_NUM}
- Python SDK: \`reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/\`
- LangSmith API docs: https://docs.smith.langchain.com/

EOF

echo "📄 **Research Report Created:** $RESEARCH_FILE"
```

### Step 6: Set Up SDK Research Workspace

Guide user to set up SDK research workspace using the skill:

```
🔧 **Next: Set Up SDK Research Workspace**

To analyze the Python SDK precedent, use the setup-remote-repo-notes-dir skill:

For LangSmith features:
  .claude/skills/setup-remote-repo-notes-dir/scripts/setup_repo_notes.sh https://github.com/langchain-ai/langsmith-sdk

For LangGraph features:
  .claude/skills/setup-remote-repo-notes-dir/scripts/setup_repo_notes.sh https://github.com/langchain-ai/langgraph

This will create:
  reference/repo/langchain-ai/langsmith-sdk/
  ├── notes/          # Your research notes (committed)
  └── code/           # Cloned SDK (gitignored)
```

### Step 7: Create Worktree for Scout Work

```bash
# Offer to create worktree for this scout issue
BRANCH_NAME="codekiln/${ISSUE_NUM}-${FEATURE_SLUG}-scout"

echo ""
echo "🌳 **Create Worktree for Scout Issue?**"
echo ""
echo "Would you like to create a git worktree for this scout issue?"
echo ""
echo "Command:"
echo "  git worktree add -b $BRANCH_NAME wip/codekiln-${ISSUE_NUM}-scout main"
echo ""
echo "This creates an isolated workspace for your scout research."
```

### Step 8: Display Scout Workflow

```
✅ **Scout Issue Ready**

📍 Issue #${ISSUE_NUM}: Research $FEATURE_NAME feasibility
🔗 URL: $ISSUE_URL
📄 Report: $RESEARCH_FILE

## Scout Research Workflow

**Phase 0.0.1: Explore Existing Code**
- Search ./cli and ./sdk for partial implementations
- Document what's already done vs. what's needed

**Phase 0.0.2: Analyze Python SDK**
- Clone langsmith-sdk using setup-remote-repo-notes-dir skill
- Find relevant methods in python/langsmith/client.py
- Document method signatures, parameters, patterns
- Update reference/repo/langchain-ai/langsmith-sdk/notes/README.md

**Phase 0.0.3: Identify API Endpoints**
- Use LangSmith UI or docs to understand feature behavior
- Identify which REST API endpoints are involved
- Document request/response shapes

**Phase 0.0.4: Assess Complexity**
- Rate implementation complexity (low/medium/high)
- Identify technical blockers or unknowns
- List dependencies on unimplemented features

**Phase 0.0.5: Make Recommendation**
- Go: Feasible, proceed to Phase 0 (Epic Setup)
- No-Go: Blockers exist, defer/cancel
- Conditional: Feasible if conditions met

**Phase 0.0.6: Create PR**
- Complete research report at $RESEARCH_FILE
- Update SDK notes in reference/repo/.../notes/README.md
- Optional: Add experiments in reference/experiments/${ISSUE_NUM}-${FEATURE_SLUG}/
- Create PR directly to main (no milestone yet)
- PR title: "📚 docs: scout $FEATURE_NAME feasibility"
- PR body should include: "Fixes #${ISSUE_NUM}"

## After Scout Completion

**If Go Recommendation**:
1. Review scout findings
2. Create GitHub milestone (use naming: ls-${FEATURE_SLUG})
3. Create parent issue for milestone (Phase 0: Epic Setup)
4. Optionally: Retroactively attach scout issue #${ISSUE_NUM} to milestone

**If No-Go Recommendation**:
1. Document blockers in research report
2. Close scout issue with explanation
3. Revisit when blockers resolved

**If Conditional Recommendation**:
1. Address conditions/blockers first
2. Re-assess feasibility
3. Proceed to Go or No-Go

## References

- **Phase 0.0 Documentation**: docs/dev/feature-development-process.md#phase-00-pre-epic-scouting-optional
- **Scout Template**: docs/templates/scout-issue-template.md
- **Example Scout**: Issue #398 (structured output prompts)
- **Research Report Template**: $RESEARCH_FILE
```

### Step 9: Offer Immediate Assistance

```
💡 **Ready to Start Scouting?**

I can help you get started with the research now. Would you like me to:

A) Clone the langsmith-sdk repository for analysis
B) Search existing langstar code for partial implementations
C) Help you understand the feature from LangSmith documentation
D) All of the above (comprehensive scout start)
E) I'll handle it myself

Select an option to begin Phase 0.0 scouting, or proceed independently.
```

## Special Cases and Error Handling

### Case 1: Feature Name Has Spaces or Special Characters

**Scenario**: User provides "Dataset Management API" as feature name.

**Action**:
- Convert to slug: `dataset-management-api`
- Confirm slug with user before creating issue

### Case 2: Scout Issue Already Exists

**Scenario**: A scout issue for this feature already exists.

**Action**:
```bash
# Check if similar scout issue exists
EXISTING=$(gh issue list --label scout --search "$FEATURE_NAME" --json number,title --jq '.[0]')

if [ -n "$EXISTING" ]; then
  EXISTING_NUM=$(echo "$EXISTING" | jq -r '.number')
  EXISTING_TITLE=$(echo "$EXISTING" | jq -r '.title')

  echo "⚠️  **Similar Scout Issue Found**"
  echo ""
  echo "Issue #$EXISTING_NUM: $EXISTING_TITLE"
  echo ""
  echo "Do you want to:"
  echo "A) Continue with new scout issue anyway"
  echo "B) Use existing issue #$EXISTING_NUM"
  echo "C) Cancel"
fi
```

### Case 3: Template Not Found

**Scenario**: Scout template doesn't exist yet.

**Action**:
```
❌ **Scout Template Not Found**

The scout issue template is missing at: docs/templates/scout-issue-template.md

This template should have been created as part of issue #448.

Please ensure:
1. PR #449 has been merged (adds Phase 0.0 documentation)
2. Template exists in docs/templates/

Cannot proceed without template.
```

## Integration with Other Commands

### After Scout Completion

Once scout research is complete and PR is merged, create the full milestone:

```bash
# If scout recommends Go:
# 1. Manually create GitHub milestone via UI or API
gh api repos/:owner/:repo/milestones -f title="ls-${FEATURE_SLUG}" \
  -f description="Parent issue: (to be created)"

# 2. Create parent issue (Phase 0: Epic Setup) - manual or future command
# 3. Create sub-issues for phases 1-8 using gh-sub-issue skill
```

### With setup-remote-repo-notes-dir Skill

This command guides users to use the skill but doesn't invoke it directly:

```bash
# User should run separately:
.claude/skills/setup-remote-repo-notes-dir/scripts/setup_repo_notes.sh https://github.com/langchain-ai/langsmith-sdk
```

### With git-worktrees Skill

Offers to create worktree but doesn't automatically do it:

```bash
# User can run:
git worktree add -b codekiln/${ISSUE_NUM}-${FEATURE_SLUG}-scout \
  wip/codekiln-${ISSUE_NUM}-scout main
```

## Best Practices

### Clear Feature Names

**Good feature names:**
- `dataset-management`
- `annotation-queues`
- `structured-output-prompts`

**Avoid:**
- `new-feature` (too vague)
- `fix-bug` (scout is for new features)
- `api-endpoints` (too generic)

### Scout Issue Scope

**Scout should answer:**
- Is this feature feasible?
- What's the complexity?
- Are there blockers?
- Should we proceed to full milestone?

**Scout should NOT:**
- Implement any code
- Create the milestone
- Define full implementation plan

### Research Report Quality

**Good scout reports:**
- Clear go/no-go recommendation
- Specific technical findings
- Links to SDK code and API docs
- Identified blockers or dependencies

**Incomplete scout reports:**
- Vague "seems doable" conclusions
- No analysis of SDK precedent
- Missing complexity assessment

## Command Reference

### Essential Commands Used

```bash
# Create scout issue
gh issue create --title "..." --body "..." --label "research,scout"

# Check for existing scouts
gh issue list --label scout --search "<feature-name>"

# Create research directory
mkdir -p docs/research

# Create worktree (optional)
git worktree add -b <branch> wip/<worktree-name> main
```

## See Also

- **Phase 0.0 Documentation** - `docs/dev/feature-development-process.md#phase-00-pre-epic-scouting-optional`
- **Scout Issue Template** - `docs/templates/scout-issue-template.md`
- **Example Scout Issue** - Issue #398 (structured output prompts)
- **setup-remote-repo-notes-dir Skill** - `.claude/skills/setup-remote-repo-notes-dir/SKILL.md`
- **Milestone Lifecycle Review** - `docs/research/448-milestone-lifecycle-review.md`
