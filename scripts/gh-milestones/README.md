# GitHub Milestones Scripts

Python scripts for automating GitHub milestone workflows.

## Scripts

### `prep-next.py`

Automates moving to the next issue in an active milestone after completing the current task.

**Usage:**

```bash
# Auto-detect milestone from current branch
python3 scripts/gh-milestones/prep-next.py

# Specify milestone name
python3 scripts/gh-milestones/prep-next.py "cc-pr-auto"

# Specify milestone number
python3 scripts/gh-milestones/prep-next.py 10
```

**Features:**

- Finds last completed issue in milestone
- Removes `wip` label from completed issue
- Uses intelligent traversal to find next issue (sibling-first depth-first)
- Applies `ready` label to next issue
- Creates worktree with proper naming conventions
- Displays comprehensive issue context

**Requirements:**

- Python 3.6+
- GitHub CLI (`gh`)
- Git
- `gh-sub-issue` extension (optional, degrades gracefully)

**Called by:** `.claude/commands/gh-milestones/gh-prep-next.md`

## Development

When adding new scripts:

1. Use Python 3.6+ for compatibility
2. Include proper error handling
3. Add comprehensive docstrings
4. Use type hints where appropriate
5. Make scripts executable: `chmod +x script.py`
6. Add entry to this README

## Testing

Test scripts individually:

```bash
# Check syntax
python3 -m py_compile scripts/gh-milestones/prep-next.py

# Run with --help
python3 scripts/gh-milestones/prep-next.py --help

# Test with actual milestone
python3 scripts/gh-milestones/prep-next.py "milestone-name"
```
