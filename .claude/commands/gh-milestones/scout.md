---
description: Create a Phase 0.0 scout issue and perform feasibility research for a new milestone
argument-hint: <feature-name>
---

# Scout Milestone Feasibility (Phase 0.0)

Create a scout issue and perform AI-driven feasibility research before committing to a full 8-phase milestone.

## Arguments

```text
$ARGUMENTS
```

Parse: `<feature-name>` - short descriptive name (e.g., "dataset-management", "annotation-queues")

## When to Use

- Adding support for a new LangSmith/LangGraph API feature
- Implementing functionality with unclear API complexity
- Need to validate feasibility before committing to full 8-phase process

## Execution

### 1. Parse and Validate

```bash
FEATURE_NAME="$ARGUMENTS"
FEATURE_SLUG=$(echo "$FEATURE_NAME" | tr '[:upper:]' '[:lower:]' | tr ' _' '-' | sed 's/--*/-/g')

if [ -z "$FEATURE_NAME" ]; then
  echo "Error: Feature name required. Usage: /gh-milestones/scout <feature-name>"
  exit 1
fi
```

### 2. Check for Existing Scout

```bash
EXISTING=$(gh issue list --label scout --search "$FEATURE_NAME" --state open --json number,title --jq '.[0].number // empty')
if [ -n "$EXISTING" ]; then
  echo "Scout issue #$EXISTING already exists for this feature"
  exit 1
fi
```

### 3. Create Scout Issue

```bash
ISSUE_BODY=$(cat docs/templates/scout-issue-template.md | sed "s/{feature-name}/$FEATURE_NAME/g")

ISSUE_URL=$(gh issue create \
  --title "[Scout] Research $FEATURE_NAME feasibility and API patterns" \
  --body "$ISSUE_BODY" \
  --label "research,scout")

ISSUE_NUM=$(echo "$ISSUE_URL" | grep -oE '[0-9]+$')
```

### 4. Create Research Directory

```bash
mkdir -p docs/research
RESEARCH_FILE="docs/research/${ISSUE_NUM}-${FEATURE_SLUG}-scout.md"

cat > "$RESEARCH_FILE" <<EOF
# $FEATURE_NAME Feasibility Scout

**Issue**: #${ISSUE_NUM}
**Date**: $(date +%Y-%m-%d)
**Status**: In Progress

## Executive Summary

**Feasibility**: [Go / No-Go / Conditional]

## 1. Existing Langstar Code

<!-- Search ./cli and ./sdk -->

## 2. Python SDK Precedent

<!-- Analyze python/langsmith/client.py -->

## 3. API Endpoints

<!-- REST endpoints and shapes -->

## 4. Complexity Assessment

**Complexity**: [Low / Medium / High]

## 5. Experiments

<!-- Document any API experiments run -->

## 6. Recommendation

**Decision**: [Go / No-Go / Conditional]
**Next Steps**:
EOF
```

### 5. Set Up SDK Research (if needed)

Only clone if not already present:

```bash
if [ ! -d "reference/repo/langchain-ai/langsmith-sdk/code" ]; then
  .claude/skills/setup-remote-repo-notes-dir/scripts/setup_repo_notes.sh https://github.com/langchain-ai/langsmith-sdk
fi
```

### 6. Create Worktree

```bash
BRANCH="codekiln/${ISSUE_NUM}-${FEATURE_SLUG}-scout"
git worktree add -b "$BRANCH" "wip/codekiln-${ISSUE_NUM}-scout" main
```

### 7. Perform Scout Research

**Execute these steps automatically:**

1. **Search existing langstar code**
   ```bash
   grep -r "$FEATURE_NAME" ./cli ./sdk 2>/dev/null || echo "No existing implementation found"
   ```

2. **Analyze Python SDK**
   - Read `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py`
   - Find methods related to the feature
   - Document signatures and patterns

3. **Identify API endpoints**
   - Check LangSmith API docs via MCP
   - Document request/response shapes

4. **Run experiments if needed**
   - Create `reference/experiments/${ISSUE_NUM}-${FEATURE_SLUG}/`
   - Write Python scripts to test API behavior
   - Document findings in research report

5. **Assess complexity** and make recommendation

### 8. Complete Research Report

Update `$RESEARCH_FILE` with findings and clear Go/No-Go/Conditional recommendation.

### 9. Create PR

```bash
cd "wip/codekiln-${ISSUE_NUM}-scout"
git add -A
git commit -m "docs: scout $FEATURE_NAME feasibility

Fixes #${ISSUE_NUM}"
gh pr create --title "docs: scout $FEATURE_NAME feasibility" \
  --body "Fixes #${ISSUE_NUM}

## Summary
Feasibility research for $FEATURE_NAME implementation.

## Deliverables
- Research report at docs/research/${ISSUE_NUM}-${FEATURE_SLUG}-scout.md
- SDK analysis notes (if applicable)"
```

## After Scout Completion

**If Go**: Create milestone with `ls-${FEATURE_SLUG}` naming, then Phase 0 parent issue.

**If No-Go**: Close scout issue with explanation.

**If Conditional**: Document conditions, re-assess when resolved.

## References

- Phase 0.0 docs: `docs/dev/feature-development-process.md#phase-00-pre-epic-scouting-optional`
- Scout template: `docs/templates/scout-issue-template.md`
- Example: Issue #398 (structured output prompts)
