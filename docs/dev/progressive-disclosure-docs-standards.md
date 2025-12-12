# Progressive Disclosure Documentation Standards

Standards for writing documentation that AI coding agents can efficiently navigate without overloading their context windows.

## Core Principle

**Documentation must be modular.** Large monolithic documents waste AI context on irrelevant sections. Instead, structure docs as:

1. **Table of Contents (TOC)** - Brief index pointing to topic files (~10-15 lines)
2. **Topic Files** - Focused documents on single subjects (<500 lines)
3. **Deep Dives** - Detailed reference material loaded only when needed

## Why This Matters

AI coding agents have finite context windows. Every line loaded consumes tokens that could be used for reasoning about your actual task. Poor documentation structure leads to:

- **Context pollution** - Loading 5000 tokens when 500 would suffice
- **Reduced accuracy** - Important details buried in irrelevant content
- **Slower responses** - Processing unnecessary information

**Goal:** Load only what's relevant to the current task.

## Reading Documentation as an Agent

### The TOC-First Pattern

```
1. Load AGENTS.md → find reference to topic area
2. Load topic TOC (e.g., docs/dev/testing/README.md)
3. Identify 1-3 relevant docs from TOC
4. Load only those specific docs
5. Optionally scan with `grep '^#'` before full read
```

### Grep Before Read

Before loading a full document, scan its structure:

```bash
# See all headings in a file
grep '^#' docs/dev/testing/cli-integration-tests.md

# Find specific section
grep -n '## Prerequisites' docs/dev/testing/cli-integration-tests.md
```

This helps identify whether a document contains what you need before committing context to it.

### Example: Writing CLI Integration Tests

**Task:** "Add integration tests for new `deployment list` command"

**Efficient approach:**

1. Load `AGENTS.md` → find testing reference
2. Load `docs/dev/testing/README.md` (TOC, ~15 lines)
3. Identify relevant docs: `cli-integration-tests.md`, `crud-lifecycle-pattern.md`
4. Load only those 2 docs (~400 lines total)

**Inefficient approach:**

- Load all 8 testing docs (~2000 lines)
- 75% of content irrelevant to CLI testing

## Writing Documentation as an Agent

When creating or updating docs, follow these principles:

### File Size Limits

| Document Type | Target Size   | Maximum   |
| ------------- | ------------- | --------- |
| TOC/Index     | 10-15 lines   | 20 lines  |
| Standard doc  | 200-300 lines | 500 lines |
| Reference     | 300-500 lines | 800 lines |

### Heading Structure

Use clear, descriptive headings that enable grep scanning:

```markdown
# Document Title

## Overview <- What this doc covers

## Prerequisites <- What you need first

## Quick Reference <- TL;DR for experienced users

## Detailed Guide <- Step-by-step instructions

## Troubleshooting <- Common problems and solutions

## References <- Links to related docs
```

### One Topic Per File

Each document should cover **one specific topic**. If you find yourself writing "see also" sections that are essential to understanding the content, consider:

1. Consolidating into a single document
2. Creating a parent document that loads children
3. Adding explicit "load this next" guidance

### Avoid Content Duplication

**Never repeat content across files.** Instead:

```markdown
## Prerequisites

See `docs/dev/testing/test-fixtures.md` for test deployment setup.
```

Not:

```markdown
## Prerequisites

To set up test fixtures, first ensure you have LANGSMITH_API_KEY...
[100 lines of duplicated setup instructions]
```

## Repository Structure Example

```
docs/dev/testing/
├── README.md                     # TOC only (~15 lines)
├── HIGH_LEVEL_TESTING_GUIDELINES.md   # Principles, PR requirements
├── crud-lifecycle-pattern.md     # CLI → SDK verification pattern
├── sdk-integration-tests.md      # SDK-specific test standards
├── cli-integration-tests.md      # CLI-specific test standards
├── devcontainer-feature-tests.md # Infrastructure testing
├── test-fixtures.md              # Test deployment management
├── mocking-patterns.md           # When/how to use httpmock
├── debugging-tests.md            # Common failures, debugging
└── post-mortems/                 # Historical case studies
    └── NNN-issue-description.md  # Format: issue number + description
```

## Working Pattern

```
┌─────────────────────────────────────────────────────────┐
│                     AGENTS.md                            │
│  "For testing docs, see @docs/dev/testing/README.md"    │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│              docs/dev/testing/README.md                  │
│  - HIGH_LEVEL_TESTING_GUIDELINES.md - principles         │
│  - crud-lifecycle-pattern.md - CLI verification          │
│  - sdk-integration-tests.md - SDK tests                  │
│  - cli-integration-tests.md - CLI tests                  │
│  ...                                                     │
└─────────────────────────────────────────────────────────┘
                            │
              (load 1-3 relevant docs)
                            │
                ┌───────────┴───────────┐
                ▼                       ▼
┌───────────────────────┐  ┌───────────────────────┐
│ cli-integration-      │  │ crud-lifecycle-       │
│ tests.md              │  │ pattern.md            │
│ (~200 lines)          │  │ (~200 lines)          │
└───────────────────────┘  └───────────────────────┘
```

## Anti-Patterns

### Monolithic Documentation

```markdown
# Testing Guide

[2000 lines covering SDK, CLI, infrastructure, mocking,
debugging, and historical lessons all in one file]
```

**Problem:** Forces loading irrelevant content. Agent working on CLI tests must load SDK and infrastructure sections.

### Repeated Content

```markdown
# File: sdk-integration-tests.md

## Prerequisites

Set LANGSMITH_API_KEY and ensure test deployment exists...

# File: cli-integration-tests.md

## Prerequisites

Set LANGSMITH_API_KEY and ensure test deployment exists...
```

**Problem:** Updates must happen in multiple places. Content drift creates inconsistency.

**Solution:** Reference shared prerequisites from a single source:

```markdown
## Prerequisites

See `test-fixtures.md` for required environment setup.
```

### Vague TOC Entries

```markdown
- testing.md - Testing stuff
- utils.md - Utilities
```

**Problem:** Agent cannot determine relevance without loading each file.

**Solution:** Descriptive one-liners:

```markdown
- sdk-integration-tests.md - SDK integration test standards and patterns
- mocking-patterns.md - When and how to use httpmock for API mocking
```

### Hidden Prerequisites

```markdown
# CLI Integration Tests

Run this command to test:
cargo test --features integration-tests
```

**Problem:** Missing context about required environment variables causes test failures.

**Solution:** Always start with prerequisites:

```markdown
# CLI Integration Tests

## Prerequisites

- `LANGSMITH_API_KEY` environment variable set
- Test deployment created (see `test-fixtures.md`)

## Running Tests

...
```

## Metrics and Validation

### Target Metrics

- **TOC files:** ≤15 lines
- **Standard docs:** <500 lines
- **Context per task:** <3000 tokens (vs >5000 without progressive disclosure)
- **Relevance ratio:** >80% of loaded content should be relevant to task

### Validation Checklist

When creating documentation:

- [ ] TOC exists and is ≤15 lines
- [ ] Each doc covers exactly one topic
- [ ] No content is duplicated across files
- [ ] Headings support `grep '^#'` scanning
- [ ] Prerequisites are explicit, not assumed
- [ ] File sizes are within limits
- [ ] Clear "when to load" guidance in TOC

## References

- [Nielsen Norman Group: Progressive Disclosure](https://www.nngroup.com/articles/progressive-disclosure/) - UX principles that inform this approach
- [Anthropic Claude Documentation](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching) - Context window management
- [LangChain llms.txt Pattern](https://langchain-ai.github.io/langgraph/llms.txt) - Documentation index for AI consumption
