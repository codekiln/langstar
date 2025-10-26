# Claude Code Action Branch Naming

## Issue Summary

The Claude Code GitHub Action creates branches with a different naming convention than the project's standard. This document outlines the issue, findings, and recommendations.

**Related:** [Issue #11](https://github.com/codekiln/langstar/issues/11)

## Branch Naming Comparison

| Format | Pattern | Example |
|--------|---------|---------|
| **Project Convention** | `<username>/<issue_num>-<issue_slug>` | `claude/11-branch-naming-conventions` |
| **Claude Code Actual** | `<prefix>issue-<num>-<timestamp>` | `claude/issue-11-20251026-1529` |

### Key Differences

1. **Issue Prefix**: Claude adds the word "issue-" before the number
2. **Identifier**: Claude uses timestamp (YYYYMMDD-HHMM) instead of issue title slug
3. **Format**: `issue-N-timestamp` vs `N-slug`

## Root Cause

The Claude Code Action (anthropics/claude-code-action@v1) has a hardcoded branch naming format that cannot be fully customized through workflow configuration.

### Available Configuration

According to the Claude Code Action documentation, the only available input for branch naming is:

- **`branch_prefix`** (default: `claude/`)
  - Controls only the prefix portion of the branch name
  - Does NOT control the suffix pattern (issue number, slug, timestamp)
  - Cannot achieve the desired `<username>/<issue_num>-<issue_slug>` format

### What Cannot Be Configured

- The "issue-" prefix before the number
- The timestamp vs. slug decision
- The overall pattern structure

## Current Workflow Configuration

The `.github/workflows/claude.yml` file currently has no branch naming configuration:

```yaml
- name: Run Claude Code
  id: claude
  uses: anthropics/claude-code-action@v1
  with:
    anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }}
```

## Recommendations

### Option 1: Accept Current Behavior (Recommended)

**Pros:**
- No changes required
- Timestamp-based naming ensures uniqueness
- Avoids potential branch name conflicts
- Clear indication that Claude created the branch

**Cons:**
- Inconsistent with manual branch naming
- Timestamps less readable than slugs
- Violates documented project convention

**Action:** Update project documentation to explicitly allow both formats

### Option 2: Update Project Convention

Modify the project's branch naming convention to officially support both patterns:

```markdown
## Branch Naming Convention

### Manual Branches
<username>/<issue_num>-<issue_slug>

### Automated (Claude Code)
claude/issue-<issue_num>-<timestamp>
```

### Option 3: File Upstream Feature Request

Request the Claude Code Action team to add support for custom branch naming patterns.

**Suggested Feature:**
```yaml
branch_name_pattern: "{prefix}{issue_num}-{issue_slug}"
```

**Available Variables:**
- `{prefix}` - Configurable prefix (default: claude/)
- `{issue_num}` - GitHub issue number
- `{issue_slug}` - Kebab-case slug from issue title
- `{timestamp}` - Current timestamp (YYYYMMDD-HHMM)
- `{username}` - GitHub username

**Link:** https://github.com/anthropics/claude-code-action/issues

### Option 4: Modify Workflow (Limited Impact)

While we could add `branch_prefix` to the workflow:

```yaml
- name: Run Claude Code
  uses: anthropics/claude-code-action@v1
  with:
    anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }}
    branch_prefix: "claude/"  # Already the default
```

This would **NOT** solve the problem because:
- It only changes the prefix
- The suffix pattern remains `issue-<num>-<timestamp>`
- Still doesn't match project convention

**Impact:** Minimal - not recommended

### Option 5: Manual Branch Creation

For issues requiring strict naming compliance:

```bash
# Create branch manually before mentioning @claude
git checkout -b claude/11-branch-naming-conventions
git push -u origin claude/11-branch-naming-conventions

# Then mention @claude in the issue
# Claude will detect and use the existing branch
```

**Note:** This approach requires additional manual steps and reduces automation benefits.

## Decision Matrix

| Option | Effort | Impact | Maintains Automation | Recommendation |
|--------|--------|--------|---------------------|----------------|
| Accept Current | Low | Low | ✅ Yes | ⭐ **Recommended** |
| Update Convention | Low | Medium | ✅ Yes | ⭐ **Recommended** |
| Feature Request | Medium | High (long-term) | ✅ Yes | ✓ Suggested |
| Modify Workflow | Low | None | ✅ Yes | ✗ Not effective |
| Manual Creation | High | High | ❌ No | ✗ Defeats purpose |

## Proposed Solution

**Immediate (v1):**
1. ✅ Update documentation to reflect actual Claude Code behavior
2. Update project convention to explicitly allow both naming patterns
3. Document the difference and when each is used

**Long-term (v2):**
4. File feature request with Claude Code Action repository
5. If feature is added, update workflow configuration
6. Standardize on single naming convention

## Implementation

### Documentation Updates

The following files have been updated:

- ✅ `docs/dev/github-workflow.md` - Corrected Claude Code branch format description
- ✅ `docs/dev/claude-code-branch-naming.md` - This detailed analysis (new file)

### Proposed Convention Update

Update `docs/dev/github-workflow.md` to officially support both formats:

```markdown
## Branch Naming Convention

Branches should follow one of these formats:

### Manual Branch Creation
<username>/<issue_num>-<issue_slug>

Examples:
- `alice/7-add-user-authentication`
- `bob/42-fix-database-connection`

### Automated (Claude Code)
claude/issue-<issue_num>-<timestamp>

Examples:
- `claude/issue-11-20251026-1529`
- `claude/issue-6-20251026-1344`

Both formats are acceptable. Manual branches use descriptive slugs,
while automated Claude branches use timestamps for uniqueness.
```

## Testing

To verify the current behavior:

1. Create a new test issue
2. Mention `@claude` in the issue
3. Observe the branch name created
4. Confirm it follows `claude/issue-N-YYYYMMDD-HHMM` format

## References

- [Issue #11: Bug: Claude Code Action branch naming doesn't follow project conventions](https://github.com/codekiln/langstar/issues/11)
- [Claude Code Action Repository](https://github.com/anthropics/claude-code-action)
- [Project Workflow Documentation](./github-workflow.md)
- [Example Issue #6](https://github.com/codekiln/langstar/issues/6#issuecomment-3448596077)

## Conclusion

The Claude Code Action's branch naming cannot be fully customized to match the project's preferred convention. The most pragmatic solution is to:

1. **Accept both naming patterns** as valid
2. **Document the difference** clearly
3. **File a feature request** for future improvement

This maintains the automation benefits while acknowledging the current limitation.
