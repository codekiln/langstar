# Documentation Approximation Standards

**Purpose:** Maintain accuracy in documentation without excessive churn from minor changes.

**Issue context:** #573 - Copilot review feedback highlighted inconsistencies that hurt credibility, but we want to avoid PR churn over single-line differences.

## Core Principle

**Balance accuracy with maintainability.** Documentation should be accurate enough to be useful and credible, but not so precise that every small change requires updates across multiple files.

## Approximation Guidelines

### 1. Small Numbers (< 20)

**Use ranges** to accommodate minor variations:

- ✅ Good: "~10-15 lines", "~5-10 files"
- ❌ Avoid: "exactly 14 lines" (requires update when 15th line added)

**Acceptable variance:** ±3 lines/items

**Example:**

```markdown
TOC file: ~10-15 lines
```

This covers 10-15 lines without requiring updates if it grows from 14 to 15 lines.

### 2. Medium Numbers (20-100)

**Round to nearest 5 or 10:**

- ✅ Good: "~50 lines", "~75 files"
- ❌ Avoid: "53 lines"

**Acceptable variance:** ±10%

**Example:**

```markdown
Typical integration test: ~50 lines
```

### 3. Large Numbers (> 100)

**Round to nearest hundred:**

- ✅ Good: "~3,000 lines", "~500 files"
- ❌ Avoid: "3,079 lines"

**Acceptable variance:** ±10%

**Example:**

```markdown
Total testing documentation: ~3,000 lines
```

This covers 2,700-3,300 lines without requiring updates.

### 4. Token Estimates

**Use ranges with round numbers:**

- ✅ Good: "~24,000-30,000 tokens", "~4,000-5,000 tokens"
- ❌ Avoid: "27,438 tokens"

**Acceptable variance:** ±20% (tokens are estimates, not measurements)

**Example:**

```markdown
Full load: ~24,000-30,000 tokens
Per-task load: ~4,000-5,000 tokens
```

### 5. Percentages

**Round to nearest 5% for estimates:**

- ✅ Good: "~80-85%", "~70-75%"
- ❌ Avoid: "73.4% reduction"

**Use precise percentages only for:**

- Measured benchmarks
- SLA requirements
- Financial/legal contexts

**Example:**

```markdown
Context savings: ~70-75% reduction
```

## When to Update Numbers

### ✅ Update Required

Update documentation numbers when:

- **Structural changes:** New major doc added/removed
- **Order of magnitude:** 10 files → 20 files, 3,000 lines → 6,000 lines
- **Outside variance:** Number falls outside acceptable range
- **Invalidates examples:** "Load 2-3 docs" but now requires 5+

### ⏸️ No Update Needed

**Don't** update for:

- **Minor changes:** 14 lines → 15 lines (covered by "~10-15")
- **Within variance:** 3,050 lines when documented as "~3,000"
- **Rounding boundary:** 2,980 lines still "~3,000" (not "~2,900")
- **Token estimate drift:** ±20% is normal for estimates

## File Count Rules

### Count Consistency

**Be explicit about what's counted:**

```markdown
# ✅ Clear

Total: 10 markdown files in docs/dev/testing/

# ❌ Ambiguous

Total: 9 testing docs
```

### What to Count

**Include in count:**

- All files directly referenced in documentation
- Files in subdirectories if they're part of the documented set
- Index/TOC files

**Example:**

```markdown
Testing documentation (docs/dev/testing/):

- 1 TOC (README.md)
- 8 specialized guides
- 1 post-mortem
  Total: 10 markdown files
```

## Cross-File Consistency

### Single Source of Truth

**Pick one file as the authoritative source:**

- Other files may reference it
- Updates happen in authoritative file first
- References stay in sync

**Example:**

```markdown
<!-- In docs/research/validation-report.md (authoritative) -->

Total: 10 markdown files, ~3,000 lines

<!-- In AGENTS.md (reference) -->

Testing docs total ~3,000 lines
See docs/research/556-integration-validation-report.md for details.
```

### Reconciliation Check

**Before committing, verify consistency:**

```bash
# Quick check for common metrics
grep -r "testing doc" docs/ AGENTS.md CLAUDE.md | grep -E "[0-9]+ (files|lines|tokens)"
```

Look for:

- Same numbers using same approximation style
- Ranges that overlap appropriately
- No exact numbers contradicting approximate ones

## Review Feedback Response

### When Reviewer Notes Inconsistency

**If numbers are factually wrong:**

- Fix immediately (Option 1 from PR workflow)
- Verify with actual counts (`wc -l`, `ls | wc -l`)
- Update all files for consistency

**If numbers are within acceptable variance:**

- Explain the approximation standard
- Optionally tighten ranges if it helps clarity
- Reference this document

**Example response:**

```markdown
The variance is within our approximation standards (docs/dev/documentation-approximation-standards.md).
"~3,000 lines" covers 2,700-3,300 lines (±10%), and actual count is 3,079.
If you feel this hurts clarity, I can tighten to "~3,000-3,100 lines".
```

## Benefits of This Approach

### Reduced Churn

- Minor doc additions don't trigger doc updates
- Changes stay focused on actual work
- Fewer "fix documentation numbers" commits

### Maintained Credibility

- Numbers are accurate enough to be useful
- Consistency across files is explicit
- Variance is planned, not sloppy

### Clear Intent

- Ranges signal "approximate, may vary"
- Exact numbers signal "measured, important"
- Reviewers know what to check

## Anti-Patterns to Avoid

### ❌ False Precision

```markdown
Context savings: 73.246% reduction
Total lines: 3,079
```

Implies measurement precision we don't have.

### ❌ Inconsistent Styles

```markdown
File A: ~3,000 lines
File B: 3,079 lines
File C: approximately 3000 lines
```

Mixing styles looks uncoordinated.

### ❌ Outdated Ranges

```markdown
TOC: ~10-15 lines (actual: 47 lines)
```

Range doesn't cover reality.

### ❌ Over-Updating

```markdown
Commit: "docs: update line count from 3,079 to 3,082"
```

Churn without value.

## Validation Checklist

Before committing documentation with numbers:

- [ ] Small numbers use ranges (~10-15)
- [ ] Large numbers rounded to nearest hundred (~3,000)
- [ ] Token estimates use ranges (~24,000-30,000)
- [ ] Percentages rounded to nearest 5% (~75%)
- [ ] Cross-file consistency verified
- [ ] File counts are explicit and accurate
- [ ] Numbers are within acceptable variance
- [ ] No false precision (exact numbers where estimates appropriate)

## See Also

- Progressive Disclosure Standards: `docs/dev/progressive-disclosure-docs-standards.md`
- GitHub Workflow: `docs/dev/github-workflow.md`
- Issue #573: Copilot review feedback and resolution
