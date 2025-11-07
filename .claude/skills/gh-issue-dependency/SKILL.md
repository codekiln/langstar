---
name: gh-issue-dependency
description: Manage GitHub issue dependencies using the gh-issue-dependency extension. Create blocking/blocked-by relationships, visualize dependency chains, and prevent circular dependencies. Use when working with issue dependencies, prerequisites, blockers, or when users mention dependencies, blocking issues, blocked-by relationships, or dependency management.
---

# gh-issue-dependency

Manage GitHub issue dependencies using the `gh-issue-dependency` extension to create blocking/blocked-by relationships between issues.

**Key advantage**: Creates explicit dependency relationships that prevent work from starting on blocked issues until their prerequisites are completed. This is complementary to `gh-sub-issue` (hierarchical relationships) but serves the purpose of enforcing sequential dependencies.

## Installation

Verify the extension is installed:
```bash
gh extension list | grep issue-dependency
```

If not installed:
```bash
gh extension install torynet/gh-issue-dependency
```

**Prerequisites**: GitHub CLI authenticated with `gh auth login`

## Core Commands

### 1. List Dependencies

View all dependencies for an issue:

```bash
gh issue-dependency list <issue>
```

**What it does**: Shows all blocking and blocked-by relationships for the specified issue.

**Example**:
```bash
gh issue-dependency list 92
```

**Output**:
```
Issue #92: Phase 3: Implementation

Blocked by:
  #91 - Phase 2: Design (open)

Blocks:
  #100 - Phase 4: Testing (open)
```

### 2. Add Dependencies

#### Add Blocker (blocked-by)

Mark an issue as blocked by other issues:

```bash
gh issue-dependency add <issue> --blocked-by <blocker1>,<blocker2>
```

**What it does**: Creates a dependency where the issue cannot proceed until the specified blockers are resolved.

**Example**:
```bash
gh issue-dependency add 92 --blocked-by 91
```

**Output**:
```
✓ Added dependency: #92 is blocked by #91
```

**Multiple blockers**:
```bash
gh issue-dependency add 100 --blocked-by 91,92,93
```

#### Add Blocked (blocks)

Mark an issue as blocking other issues:

```bash
gh issue-dependency add <issue> --blocks <blocked1>,<blocked2>
```

**What it does**: Creates a dependency where the specified issues cannot proceed until this issue is resolved.

**Example**:
```bash
gh issue-dependency add 91 --blocks 92
```

**Output**:
```
✓ Added dependency: #91 blocks #92
```

**Note**: `A blocked-by B` is equivalent to `B blocks A`. Use both directions for clarity.

### 3. Remove Dependencies

Remove existing dependency relationships:

```bash
gh issue-dependency remove <issue> --blocked-by <blocker>
gh issue-dependency remove <issue> --blocks <blocked>
```

**What it does**: Deletes the specified dependency relationship.

**Example**:
```bash
gh issue-dependency remove 92 --blocked-by 91
```

**Output**:
```
✓ Removed dependency: #92 is no longer blocked by #91
```

## When to Use This Skill

### Use Case 1: Sequential Phase Dependencies

**Scenario**: Multi-phase project where each phase must complete before the next begins.

**Example**:
```bash
# Phase 2 cannot start until Phase 1 is done
gh issue-dependency add 92 --blocked-by 91

# Phase 3 cannot start until Phase 2 is done
gh issue-dependency add 93 --blocked-by 92
```

### Use Case 2: Prerequisite Features

**Scenario**: Feature B requires Feature A to be implemented first.

**Example**:
```bash
# Authentication API must exist before OAuth can be implemented
gh issue-dependency add 105 --blocked-by 104

# Database migrations must complete before data import
gh issue-dependency add 107 --blocked-by 106
```

### Use Case 3: Visualizing Dependencies

**Scenario**: Understanding which issues are blocking progress.

**Example**:
```bash
# Check what's blocking Phase 3
gh issue-dependency list 93

# See entire dependency chain
gh issue-dependency list 91
gh issue-dependency list 92
gh issue-dependency list 93
```

### Use Case 4: Reorganizing Dependencies

**Scenario**: Requirements changed, need to update dependency structure.

**Example**:
```bash
# Remove old dependency
gh issue-dependency remove 93 --blocked-by 91

# Add new dependency
gh issue-dependency add 93 --blocked-by 92
```

## Comparison: Dependencies vs Sub-Issues

**Critical distinction**: These are complementary but different concepts.

| Aspect | gh-sub-issue | gh-issue-dependency |
|--------|-------------|---------------------|
| **Relationship** | Parent → Child (hierarchical) | A → B (sequential) |
| **Concept** | Breakdown structure | Blocking dependencies |
| **Example** | Epic → Phase → Task | Phase 1 must complete before Phase 2 |
| **Visualization** | Tree structure | Dependency graph |
| **When to use** | Organizing work hierarchy | Enforcing prerequisites |

**Use both together** for complex projects:

```
Epic #83: User Authentication System
├── Phase 1: Research #90 (no blockers)
├── Phase 2: Design #91 (blocked-by #90)
└── Phase 3: Implementation #92 (blocked-by #91)
    ├── Task: Auth API #103 (no blockers)
    ├── Task: OAuth Integration #104 (blocked-by #103)
    └── Task: Tests #105 (blocked-by #104)
```

## Integration with Other Skills

### With gh-sub-issue

Use both skills for comprehensive project organization:
1. **gh-sub-issue**: Create hierarchical breakdown (Epic → Phases → Tasks)
2. **gh-issue-dependency**: Add sequential dependencies between phases/tasks

**Example**: Create hierarchy, then add dependencies between phases.

### With github-issue-breakdown

After breaking down an issue into sub-tasks, add dependencies:
```bash
gh issue-dependency add 104 --blocked-by 103
gh issue-dependency add 105 --blocked-by 104
```

### With update-github-issue-project-status

After resolving blockers, update project status for now-unblocked issues.

## Workflows

### Workflow 1: Linear Phase Dependencies

Create a linear dependency chain for sequential phases.

**Steps**:
1. Create or identify phase issues (e.g., #90, #91, #92)
2. Add dependencies in sequence:
   ```bash
   gh issue-dependency add 91 --blocked-by 90
   gh issue-dependency add 92 --blocked-by 91
   ```
3. Verify dependency chain:
   ```bash
   gh issue-dependency list 90  # Blocks #91
   gh issue-dependency list 91  # Blocked by #90, Blocks #92
   gh issue-dependency list 92  # Blocked by #91
   ```
4. Work on Phase 1 (#90) first
5. When #90 closes, verify #91 is unblocked:
   ```bash
   gh issue-dependency list 91  # Should show no blockers
   ```
6. Proceed to Phase 2 (#91)

### Workflow 2: Parallel Features with Shared Blocker

Multiple features depend on a common prerequisite.

**Steps**:
1. Identify prerequisite issue (e.g., #100: "Database schema")
2. Add dependencies:
   ```bash
   gh issue-dependency add 101 --blocked-by 100
   gh issue-dependency add 102 --blocked-by 100
   gh issue-dependency add 103 --blocked-by 100
   ```
3. Work on prerequisite (#100) first
4. When #100 closes, all three features become unblocked

### Workflow 3: Complex Dependency Chain

Multi-level dependencies with branches.

**Example structure**: #90 (Foundation) blocks #91 (API) and #92 (Database). #91 blocks #93, #94. #92 blocks #95.

**Add dependencies**:
```bash
gh issue-dependency add 91 --blocked-by 90
gh issue-dependency add 92 --blocked-by 90
gh issue-dependency add 93 --blocked-by 91
gh issue-dependency add 94 --blocked-by 91
gh issue-dependency add 95 --blocked-by 92
```

**Work order**: #90 → (#91, #92 in parallel) → (#93, #94, #95 in parallel)

### Workflow 4: Reorganize Dependencies

Requirements changed, need to update dependency structure.

**Steps**:
```bash
# Remove obsolete dependency
gh issue-dependency remove 92 --blocked-by 90

# Add new dependency
gh issue-dependency add 92 --blocked-by 91

# Verify changes
gh issue-dependency list 92
```

## Best Practices

### 1. Prefer Linear Over Complex

Keep dependency chains as linear and simple as possible. Complex dependency webs are hard to manage.

**Good**:
```
A → B → C → D (linear chain)
```

**Avoid**:
```
    ↗ B ↘
A →   →  → D  (complex web)
    ↘ C ↗
```

### 2. Document Dependencies in Issue Descriptions

Add dependency information to issue descriptions for visibility:

```markdown
## Dependencies

⚠️ **Blocked by**: #91 (Phase 2: Design must complete first)

**Blocks**: #100 (Phase 4: Testing cannot start until this is done)
```

### 3. Use Both Directions for Clarity

Set dependencies in both directions to make relationships explicit:

```bash
# Set both directions
gh issue-dependency add 92 --blocked-by 91
gh issue-dependency add 91 --blocks 92
```

While technically redundant, this ensures both issues show the relationship.

### 4. Avoid Circular Dependencies

The extension prevents circular dependencies, but plan carefully:

**Invalid** (will be rejected):
```bash
gh issue-dependency add 91 --blocked-by 92
gh issue-dependency add 92 --blocked-by 91  # ERROR: circular dependency
```

### 5. Regular Dependency Review

Periodically review dependencies to ensure they're still valid:

```bash
# List all dependencies for a project's issues
for issue in 90 91 92 93; do
  echo "Issue #$issue:"
  gh issue-dependency list $issue
  echo ""
done
```

### 6. Combine with Sub-Issues for Complex Projects

Use hierarchical breakdown (gh-sub-issue) + sequential dependencies (gh-issue-dependency):

```bash
# Create hierarchy
gh sub-issue create 83 "Phase 1" "Phase 2" "Phase 3"

# Add dependencies
gh issue-dependency add <phase2-issue> --blocked-by <phase1-issue>
gh issue-dependency add <phase3-issue> --blocked-by <phase2-issue>
```

## Troubleshooting

### Extension Not Found

**Error**: `gh: unknown command "issue-dependency"`

**Solution**:
```bash
# Install the extension
gh extension install torynet/gh-issue-dependency

# Verify installation
gh extension list | grep issue-dependency
```

### Circular Dependency Detected

**Error**: `Error: circular dependency detected`

**Solution**: Review your dependency chain. One of the issues in the chain depends on itself (directly or indirectly). Remove the circular dependency:

```bash
# Review dependencies
gh issue-dependency list <issue1>
gh issue-dependency list <issue2>

# Remove the dependency creating the circle
gh issue-dependency remove <issue> --blocked-by <blocker>
```

### Cross-Repository Dependencies

If you need dependencies across repositories, use full issue references:

```bash
gh issue-dependency add owner/repo1#123 --blocked-by owner/repo2#456
```

### API Rate Limiting

**Error**: `API rate limit exceeded`

**Solution**: Wait for rate limit reset or use a GitHub token with higher limits:

```bash
# Check rate limit status
gh api rate_limit

# Wait or authenticate with a different token
gh auth login
```

## Environment Requirements

**Prerequisites**:
- GitHub CLI (`gh`) installed and authenticated
- `gh-issue-dependency` extension installed
- Write access to the repository

**Verify setup**:
```bash
# Check gh CLI
gh --version

# Check authentication
gh auth status

# Check extension
gh extension list | grep issue-dependency
```

## Advanced Features

### Dry Run Mode

Test dependency changes without applying them:

```bash
gh issue-dependency add 92 --blocked-by 91 --dry-run
```

### Output Formats

Get dependency information in different formats:

```bash
# JSON format (for scripting)
gh issue-dependency list 92 --format json

# CSV format
gh issue-dependency list 92 --format csv

# Table format (default)
gh issue-dependency list 92
```

### Batch Operations

Add multiple dependencies at once:

```bash
# Multiple blockers
gh issue-dependency add 100 --blocked-by 91,92,93

# Multiple blocked issues
gh issue-dependency add 90 --blocks 91,92,93
```

## Command Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `list <issue>` | View all dependencies | `gh issue-dependency list 92` |
| `add --blocked-by` | Mark as blocked | `gh issue-dependency add 92 --blocked-by 91` |
| `add --blocks` | Mark as blocker | `gh issue-dependency add 91 --blocks 92` |
| `remove --blocked-by` | Remove blocker | `gh issue-dependency remove 92 --blocked-by 91` |
| `remove --blocks` | Remove blocked | `gh issue-dependency remove 91 --blocks 92` |

**Common flags**:
- `--dry-run` - Preview changes without applying
- `--format json|csv` - Change output format

## See Also

- **gh-sub-issue skill** - Create hierarchical parent-child issue relationships
- **github-issue-breakdown skill** - Convert task lists to sub-issues
- **update-github-issue-project-status skill** - Update GitHub Projects status
- [gh-issue-dependency repository](https://github.com/torynet/gh-issue-dependency)
- [GitHub CLI documentation](https://cli.github.com/manual/)
