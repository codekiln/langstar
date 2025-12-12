# Tmux Window Naming Conventions

This document describes the tmux window naming conventions used in the Langstar project to maximize information density and provide at-a-glance status during development.

## Overview

When working with GitHub issues and pull requests in tmux, window names follow a compact format that conveys:

1. **Current workflow phase** (via emoji)
2. **Type of work** (issue vs PR)
3. **GitHub number** (issue # or PR #)

This approach reduces tmux window titles from 50+ characters (full branch names) to 5-7 characters while maintaining clarity.

## Format

```
<emoji><prefix><number>
```

### Components

| Component  | Description                                | Examples             |
| ---------- | ------------------------------------------ | -------------------- |
| `<emoji>`  | Visual indicator of current workflow phase | 💻 🔍 🚀 🔧 ⏳ ❓ 🧹 |
| `<prefix>` | Type: `i` for issue, `pr` for pull request | `i` `pr`             |
| `<number>` | GitHub issue or PR number                  | `483` `485`          |

### Complete Examples

| Window Name | Meaning                                   |
| ----------- | ----------------------------------------- |
| `💻i483`    | Coding on issue #483                      |
| `🚀i483`    | Submitting PR for issue #483              |
| `🔧pr485`   | Maintaining PR #485                       |
| `⏳pr485`   | Waiting for CI tests on PR #485           |
| `🔍i123`    | Researching/gathering info for issue #123 |
| `🧹pr485`   | Cleaning up after PR #485 merge           |

## Phase Emojis

The workflow consists of 7 distinct phases, each with its own emoji:

| Phase                    | Emoji | When Used                                    | Typical Prefix | Automated By      |
| ------------------------ | ----- | -------------------------------------------- | -------------- | ----------------- |
| 1. Gathering information | 🔍    | Research, reading docs, exploring codebase   | `i`            | Manual            |
| 2. Coding                | 💻    | Active development on issue                  | `i`            | `/gh-start-issue` |
| 3. Waiting for tests     | ⏳    | CI/CD checks running                         | `pr`           | `/pr-workflow`    |
| 4. Waiting for user      | ❓    | Needs user input or clarification            | `i` or `pr`    | Manual            |
| 5. Submitting PR         | 🚀    | Creating and pushing pull request            | `i`            | `/pr-workflow`    |
| 6. PR maintenance        | 🔧    | Addressing review comments, fixing issues    | `pr`           | `/pr-workflow`    |
| 7. Cleanup               | 🧹    | Post-merge cleanup (delete branch, worktree) | `pr`           | Manual            |

## Prefix Conventions

### Issue Prefix: `i`

Use the `i` prefix when working on a GitHub issue before a PR exists.

**Examples:**

- `💻i483` - Coding on issue #483
- `🔍i256` - Researching solution for issue #256
- `🚀i483` - About to submit PR for issue #483

### Pull Request Prefix: `pr`

Use the `pr` prefix once a pull request has been created.

**Examples:**

- `🔧pr485` - Working on feedback for PR #485
- `⏳pr485` - Waiting for CI checks on PR #485
- `🧹pr485` - Cleaning up after PR #485 was merged

### Transition from `i` to `pr`

The transition happens automatically when using `/pr-workflow`:

```
💻i483 (coding)
  ↓
🚀i483 (submitting PR)
  ↓
🔧pr485 (PR #485 created, now in maintenance mode)
  ↓
⏳pr485 (waiting for tests)
  ↓
🔧pr485 (back to maintenance after tests complete)
```

## Automated Updates

### `/gh-start-issue` Command

When starting work on an issue:

```bash
/gh-start-issue 483
```

**Tmux window automatically set to:** `💻i483`

**Rationale:** You're beginning the coding phase on issue #483.

### `/pr-workflow` Command

When creating and managing a PR:

**Phase 1 - Submitting:**

```bash
# Before creating PR
# Tmux: 🚀i483
```

**Phase 2 - After PR Created:**

```bash
# PR #485 created for issue #483
# Tmux automatically updates to: 🔧pr485
```

**Phase 3 - During CI:**

```bash
# While CI checks are running
# Tmux: ⏳pr485
```

**Phase 4 - Back to Maintenance:**

```bash
# After CI completes (pass or fail)
# Tmux: 🔧pr485
```

## Manual Updates

You can manually update tmux window names when commands don't automatically handle it:

### For Issue Work

```bash
# Gathering information phase
ISSUE_NUM=483
tmux rename-window "🔍i${ISSUE_NUM}"

# Waiting for user input
tmux rename-window "❓i${ISSUE_NUM}"
```

### For PR Work

```bash
# Post-merge cleanup
PR_NUM=485
tmux rename-window "🧹pr${PR_NUM}"

# Waiting for user to review
tmux rename-window "❓pr${PR_NUM}"
```

## Visual Styling

Window names are styled for maximum visibility and accessibility (WCAG 2.1 Level AAA):

### Active Window

- **Background:** colour17 `#00005f` (navy blue)
- **Text:** colour15 (white)
- **Contrast ratio:** ~12:1 ✓ WCAG AAA
- **Effect:** Active window (e.g., `💻i483`) appears with navy blue background

### Inactive Windows

- **Background:** colour235 `#262626` (dark grey)
- **Text:** colour250 `#bcbcbc` (light grey)
- **Contrast ratio:** ~10:1 ✓ WCAG AAA
- **Effect:** Inactive windows have subtle grey appearance

### Pane Borders

- **Active:** colour39 `#00afff` (bright cyan-blue)
- **Inactive:** colour240 (dark grey)

Configuration file: `.devcontainer/.tmux.conf`

## Benefits

### 1. Information Density

- **Old format:** `claude/483-gh-start-issue-and-pr-workflow-tmux-clean-up-pane-` (56 chars)
- **New format:** `💻i483` (6 chars including emoji)
- **Space saved:** 50 characters per window

### 2. At-a-Glance Status

Instantly see:

- What phase you're in (emoji)
- Whether it's issue work or PR work (i vs pr)
- Which GitHub number you're working on

### 3. Context Switching

When switching between multiple issues/PRs in different tmux windows, the compact format helps you quickly identify which window to switch to.

### 4. Accessibility

All colors meet WCAG 2.1 Level AAA standards (7:1 contrast minimum), ensuring readability for users with visual impairments.

## Related Files

- `.devcontainer/.tmux.conf` - Tmux configuration with WCAG-compliant colors
- `.claude/commands/gh-start-issue.md` - Automatically sets `💻i<num>` when starting issue work
- `.claude/commands/pr-workflow.md` - Manages transitions through PR lifecycle phases
- `.claude/skills/pr-lifecycle/SKILL.md` - Documents PR lifecycle with tmux integration

## Examples from Real Development

### Example 1: Working on Issue #483

```
# Start working on issue
/gh-start-issue 483
# Tmux: 💻i483

# Make changes, commit code
# Tmux: 💻i483 (still coding)

# Ready to create PR
/pr-workflow
# Tmux: 🚀i483 (submitting)
# PR #485 gets created
# Tmux: 🔧pr485 (now maintaining PR)

# CI checks start running
# Tmux: ⏳pr485 (waiting for tests)

# CI completes, need to fix issues
# Tmux: 🔧pr485 (back to maintenance)

# PR merged, time to clean up
tmux rename-window "🧹pr485"
```

### Example 2: Multiple Issues in Parallel

```
Window 1: 💻i483 (coding on issue 483)
Window 2: 🔍i490 (researching issue 490)
Window 3: 🔧pr485 (maintaining PR 485)
Window 4: ⏳pr487 (waiting for tests on PR 487)
```

With this setup, you can quickly identify which window contains which work without reading full branch names.

## Troubleshooting

### Window name not updating automatically

**Cause:** Not in a tmux session, or tmux config not loaded.

**Solution:**

```bash
# Verify you're in tmux
echo $TMUX

# Reload tmux config
tmux source-file ~/.tmux.conf

# Verify symlink exists
ls -la ~/.tmux.conf
# Should point to: /workspace/.devcontainer/.tmux.conf
```

### Colors not showing correctly

**Cause:** Terminal doesn't support 256 colors.

**Solution:**

```bash
# Check terminal color support
echo $TERM

# Should be one of: screen-256color, tmux-256color, xterm-256color
# If not, add to .tmux.conf:
set -g default-terminal "screen-256color"
```

### Can't see emoji in window names

**Cause:** Font doesn't support emoji characters.

**Solution:** Use a font with emoji support:

- Noto Color Emoji
- Segoe UI Emoji
- Apple Color Emoji
- Or any "Nerd Font" which includes emoji support

## Future Enhancements

Potential future additions to the naming convention:

1. **Branch indicator:** Add prefix for feature vs hotfix branches
2. **Priority indicator:** Use additional emoji for high-priority work
3. **Conflict indicator:** Show when merge conflicts need resolution
4. **Team member indicator:** Show who's working on what (for shared tmux sessions)

## See Also

- [GitHub Workflow Documentation](./github-workflow.md) - Issue-driven development process
- [Git SCM Conventions](./git-scm-conventions.md) - Commit message conventions
- [Procedures](./procedures.md) - Detailed operational procedures
