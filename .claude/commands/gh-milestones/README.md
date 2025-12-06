# gh-milestones Namespace Commands

Commands for managing GitHub milestones and milestone-driven workflows.

## Available Commands

### /gh-milestones:gh-prep-next

Move to the next issue within an active milestone after completing the current task.

**Usage:** `/gh-milestones:gh-prep-next [milestone-name-or-number]`

**Description:** Automates the workflow of cleaning up completed issues, finding the next issue in the milestone hierarchy, applying labels, and creating a worktree for the next task.

**Documentation:** See `gh-prep-next.md`

**Example:**
```bash
# Auto-detect from current branch
/gh-milestones:gh-prep-next

# Explicit milestone
/gh-milestones:gh-prep-next "ls-test-improvement"
```

---

### /gh-milestones:release

Mark milestone as done and update parent issue with release information.

**Usage:** `/gh-milestones:release <milestone> <version>`

**Description:** Finalizes a milestone by updating the parent issue with release details and marking the milestone as complete.

**Documentation:** See `release.md`

**Example:**
```bash
/gh-milestones:release "ls-runs-query" v0.5.0
```

---

### /gh-milestones:scout

Create a Phase 0.0 scout issue for feasibility research.

**Usage:** `/gh-milestones:scout <feature-name>`

**Description:** Creates a scout issue and performs AI-driven feasibility research before committing to a full 8-phase milestone.

**Documentation:** See `scout.md`

**Example:**
```bash
/gh-milestones:scout "dataset-management"
```

---

### /gh-milestones:test-plan

Generate comprehensive test plan for a milestone.

**Usage:** `/gh-milestones:test-plan <milestone-name-or-number>`

**Description:** Analyzes a milestone, identifies the feature type, loads relevant testing documentation, and generates a comprehensive test plan following project standards.

**Documentation:** See `test-plan.md`

**Example:**
```bash
/gh-milestones:test-plan "ls-runs-query"
/gh-milestones:test-plan 14
```

---

## Workflow Integration

These commands work together to support the milestone-driven development process:

1. **Scout** (optional) - Research feasibility before creating full milestone
2. **Work** - Implement features through issue-driven workflow
3. **Test Plan** - Generate comprehensive testing strategy
4. **Prep Next** - Move between issues within milestone
5. **Release** - Finalize and document milestone completion

## See Also

- [GitHub Workflow Documentation](../../docs/dev/github-workflow.md)
- [Feature Development Process](../../docs/dev/feature-development-process.md)
- [Testing Standards](../../docs/dev/testing/README.md)
