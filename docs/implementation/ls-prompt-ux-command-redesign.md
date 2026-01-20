# Prompt Command Structure Redesign for AI-First UX

**Issue**: #679
**Milestone**: #16 (ls-prompt-ux)
**Date**: 2026-01-20

## Executive Summary

This document proposes a redesign of the `langstar prompt` command structure to address fundamental UX issues. The primary goal is to create an **AI-first interface** where Claude can reliably choose the correct command by reading `--help` text, while optimizing for the **prompt engineering workflow** (get text → edit → push) as the primary use case.

### Key Recommendations

1. **Consolidate `list` and `search`** into single `list [query]` command
2. **Rename `get` to `info`** for clarity (metadata only)
3. **Add `cat` command** for template-only output (enables editing workflow)
4. **Enhance `pull`** to show friendly formatted view with schema
5. **Add type indicator** to list/search output for structured prompts
6. **Keep `push`** unchanged (already supports structured outputs)

---

## Problem Statement

### Current Command Confusion

```bash
$ langstar prompt --help
Commands:
  list    List all prompts
  get     Get details of a specific prompt          # ← Metadata only (not obvious)
  search  Search for prompts                        # ← vs list?
  push    Push/create a prompt in PromptHub
  pull    Pull a prompt from the PromptHub          # ← Full content (not obvious)
```

### Specific Problems

| Problem | Impact | Example |
|---------|--------|---------|
| **`get` vs `pull` unclear** | Claude picks wrong command | User: "Show me the schema" → Claude runs `get` (no schema) ❌ |
| **`list` vs `search` duplicate** | Unnecessary choice | User: "Find prompts about RAG" → Which command? |
| **Missing editing workflow** | Can't extract template text | No way to get just the prompt text for editing |
| **Schema not visible** | Structured prompts look regular | Can't tell which prompts have schemas |
| **Raw manifest dump** | `pull` shows debugging format | Unfriendly JSON dump |

### Real User Scenario (Failure Case)

**User request**: "Show me the schema for my structured prompt"

**Claude's reasoning**:
- Reads help: `get - Get details of a specific prompt`
- "Details" sounds like it includes everything
- Runs: `langstar prompt get owner/prompt-name`

**Result**: Shows metadata (likes, downloads) but NO schema ❌

---

## Recommended Command Structure

### Option D (Selected): Consolidate and Clarify

```bash
langstar prompt --help

Commands:
  list [query]      List or search prompts (optional search query)
  info <handle>     Show prompt metadata (likes, downloads, description)
  cat <handle>      Output prompt template text only (for editing)
  pull <handle>     Pull full prompt details with formatted schema
  push              Push/create a prompt in PromptHub
```

### Rationale

| Decision | Rationale |
|----------|-----------|
| `list [query]` | Eliminates `list` vs `search` confusion |
| `info` (not `get`) | "info" clearly means metadata |
| `cat` | Follows Unix convention; obvious output |
| `pull` enhanced | Familiar name, improved formatting |
| `push` unchanged | Already works well |

---

## Implementation Plan

### Phase 1: Add New Commands (Non-Breaking)

**Issues**: 668.1-668.3

**Changes**:
1. Add `PromptCommands::Cat` variant
2. Implement template extraction in `cat` command
3. Enhance `pull` formatting with schema display
4. Add `--raw` flag to `pull` for backward compatibility
5. Add "Type" column to list output (Regular | Structured)

### Phase 2: Consolidate `list`/`search` (Breaking)

**Issue**: 668.4

**Changes**:
1. Add optional `query` positional argument to `list`
2. Deprecate `search` with warning message

**Migration**:
```bash
# Old: langstar prompt search "my query"
# New: langstar prompt list "my query"
```

### Phase 3: Rename `get` to `info` (Breaking)

**Issue**: 668.5

**Changes**:
1. Rename `PromptCommands::Get` to `PromptCommands::Info`
2. Keep `get` as hidden alias with deprecation warning

**Migration**:
```bash
# Old: langstar prompt get owner/prompt-name
# New: langstar prompt info owner/prompt-name
```

### Phase 4: Remove Deprecated Commands (Major Version)

**Issue**: 668.6

**Version**: v2.0.0 (breaking)

---

## Success Criteria

- [ ] Every primary workflow has clear, unambiguous command
- [ ] Claude can reliably choose correct command from `--help`
- [ ] Prompt engineering workflow (get → edit → push) is easy
- [ ] Structured prompt schemas visible
- [ ] No overlapping/confusing command names
- [ ] Clear migration path for breaking changes
- [ ] Help text self-explanatory

---

## References

- Parent issue: #668 (ls-prompt-ux milestone)
- Structured outputs research: docs/research/398-structured-output-prompts-scout.md
- CLI output DX: #529 (systematic CLI patterns)
- Current implementation: cli/src/commands/prompt.rs
