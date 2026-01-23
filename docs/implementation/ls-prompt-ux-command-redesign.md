# Prompt Command Structure Redesign for AI-First UX

**Issue**: #679
**Milestone**: #16 (ls-prompt-ux)
**Date**: 2026-01-20
**Last Updated**: 2026-01-22

## Executive Summary

This document proposes a redesign of the `langstar prompt` command structure to address fundamental UX issues. The primary goal is to create an **AI-first interface** where Claude can reliably choose the correct command using **progressive disclosure help** and **CRUD vocabulary** that avoids confusing non-coders with git-like metaphors.

### Key Recommendations

1. **Progressive Disclosure Help System**: Replace `--help` flags with `help` subcommands
   - `langstar prompt help` - overview/table of contents
   - `langstar prompt list help` - command overview
   - `langstar prompt list help details` - detailed help

2. **CRUD Vocabulary**: Replace git-like terms (pull, push) with standard CRUD verbs
   - **Create**: `langstar prompt create <name>`
   - **Read**: `langstar prompt get <name>` with modifiers
   - **Update**: `langstar prompt update <name>` with modifiers
   - **Delete**: (future consideration)
   - **List**: `langstar prompt list [term]` with `--all` flag

3. **Eliminate Duplicates**: Merge `list` and `search` into single command

---

## Problem Statement

### Current Command Confusion

```bash
$ langstar prompt help
Commands:
  list    List all prompts
  get     Get details of a specific prompt          # ← Metadata only (not obvious)
  search  Search for prompts                        # ← vs list?
  push    Push/create a prompt in PromptHub         # ← "push" confuses non-coders
  pull    Pull a prompt from the PromptHub          # ← "pull" requires git knowledge
```

### Specific Problems

| Problem | Impact | Example |
|---------|--------|---------|
| **Git terminology** | Confuses non-coders | "Pull" and "push" are git concepts, not intuitive for prompt management |
| **`get` unclear** | Ambiguous scope | Does "get" return text, metadata, or everything? |
| **`list` vs `search` duplicate** | Unnecessary choice | Two commands for same functionality |
| **Flat help system** | No progressive disclosure | All help at once, no guidance for exploration |
| **Missing CRUD pattern** | Inconsistent with rest of CLI | Other commands use create/get/update verbs |

### Real User Scenario (Failure Case)

**User request**: "Show me my prompt"

**Claude's confusion**:
- Should I use `get`? (might be metadata only)
- Should I use `pull`? (what does "pull" mean for a prompt?)
- Should I use `cat`? (is that just text?)

**Result**: Claude guesses wrong, user gets frustrated ❌

---

## Recommended Command Structure

### Progressive Disclosure Help System

**Level 1: Overview Help**
```bash
$ langstar prompt help

Manage LangSmith prompts

Commands:
  list      List or search prompts
  get       Read prompt content (text, metadata, model, schema)
  create    Create a new prompt
  update    Update existing prompt (text, metadata, model, schema)
  help      Show help for commands

Examples:
  langstar prompt list              # List all prompts (first page)
  langstar prompt get my-prompt     # Get prompt text
  langstar prompt create my-prompt  # Create new prompt

For detailed help on a command:
  langstar prompt list help
  langstar prompt get help details
```

**Level 2: Command Overview**
```bash
$ langstar prompt list help

List or search prompts in your workspace

Usage:
  langstar prompt list [search-term] [flags]

Flags:
  --all    Show all pages (default: first page only)

Examples:
  langstar prompt list                 # First page of prompts
  langstar prompt list rag             # Search for "rag"
  langstar prompt list --all           # All prompts (all pages)
  langstar prompt list rag --all       # All "rag" results

For more details:
  langstar prompt list help details
```

**Level 3: Detailed Help**
```bash
$ langstar prompt list help details

Detailed usage for 'langstar prompt list'

Behavior:
  - Without arguments: Lists first page of prompts in workspace
  - With search term: Server-side search, first page of results
  - With --all flag: Fetches all pages until exhaustion

Pagination:
  - Default page size: 20 prompts
  - --all: Continues fetching until no more results

Search:
  - Multi-word terms: Use quotes: "structured output"
  - Case-insensitive
  - Searches: name, description, tags

Output Format:
  - Table: Handle, Type, Downloads, Description
  - Type: "Regular" or "Structured" (has JSON schema)
  - Use --output json for programmatic access

Scoping:
  - Uses workspace from environment or config
  - Override with --workspace-id flag
```

### CRUD Command Structure

**Create**
```bash
langstar prompt create <prompt-name> [options]

Options:
  --text <TEXT>               Prompt template text
  --text-file <FILE>          Read template from file
  --metadata <JSON>           Metadata as JSON
  --model <MODEL>             Associated model name
  --structured-output <FILE>  JSON schema file
```

**Read (Get)**
```bash
# Get prompt text only (default)
langstar prompt get <prompt-name>

# Get specific aspects
langstar prompt get metadata <prompt-name>
langstar prompt get model <prompt-name>
langstar prompt get structured-output <prompt-name>

# Get everything
langstar prompt get <prompt-name> --all
```

**Update**
```bash
# Update prompt text
langstar prompt update <prompt-name> --text "new template"
langstar prompt update <prompt-name> --text-file template.txt

# Update specific aspects
langstar prompt update metadata <prompt-name> --json '{...}'
langstar prompt update model <prompt-name> --model claude-3-5-sonnet
langstar prompt update structured-output <prompt-name> --schema schema.json
```

**List/Search**
```bash
# List prompts
langstar prompt list                 # First page
langstar prompt list --all           # All pages

# Search prompts
langstar prompt list rag             # Search "rag" (first page)
langstar prompt list "structured output" --all  # Multi-word, all pages
```

---

## Detailed Command Specifications

### 1. `list [term]` - Browse and Search

**Purpose**: Unified list/search with pagination control

**Syntax**:
```bash
langstar prompt list [search-term] [--all]
```

**Arguments**:
- `search-term` (optional): Search query (use quotes for multi-word)

**Flags**:
- `--all`: Fetch all pages (default: first page only)
- `--workspace-id <ID>`: Workspace scope override

**Examples**:
```bash
langstar prompt list                    # First 20 prompts
langstar prompt list --all              # All prompts (paginate till end)
langstar prompt list rag                # Search "rag" (first page)
langstar prompt list "structured output" --all  # All search results
```

**Output**:
```
Handle                Type        Downloads  Description
my-rag-prompt         Regular     1,234      RAG Q&A prompt
data-extractor        Structured  567        Extract structured data
```

**Help Access**:
```bash
langstar prompt list help           # Overview
langstar prompt list help details   # Full details
```

---

### 2. `get` - Read Prompt Content

**Purpose**: Read prompt content with modifiers for specific aspects

**Syntax**:
```bash
# Default: get prompt text
langstar prompt get <prompt-name>

# Get specific aspect
langstar prompt get {metadata|model|structured-output} <prompt-name>

# Get everything
langstar prompt get <prompt-name> --all
```

**Modifiers**:
- None (default): Return prompt template text only
- `metadata`: Return metadata (created, likes, downloads, public/private)
- `model`: Return associated model information
- `structured-output`: Return JSON schema (if structured prompt)

**Flags**:
- `--all`: Return everything (text + metadata + model + schema)
- `--workspace-id <ID>`: Workspace scope override

**Examples**:
```bash
# Get just the prompt text
langstar prompt get my-prompt

# Get metadata
langstar prompt get metadata my-prompt

# Get model info
langstar prompt get model my-prompt

# Get schema (structured prompts)
langstar prompt get structured-output my-prompt

# Get everything
langstar prompt get my-prompt --all
```

**Output (default - text only)**:
```
You are a helpful assistant that answers questions.

Context: {context}
Question: {question}

Please provide a detailed answer.
```

**Output (metadata)**:
```
Prompt: my-prompt
Created: 2026-01-15T10:30:00Z
Likes: 12
Downloads: 567
Public: false
Type: Regular
Tags: qa, assistant
```

**Output (structured-output)**:
```json
{
  "type": "object",
  "title": "ExtractionResult",
  "properties": {
    "title": {"type": "string"},
    "rating": {"type": "integer", "minimum": 1, "maximum": 10},
    "summary": {"type": "string"}
  },
  "required": ["title", "rating", "summary"]
}
```

**Output (--all)**:
```
=== Metadata ===
Prompt: my-prompt
Created: 2026-01-15T10:30:00Z
Type: Structured
Likes: 12, Downloads: 567, Public: false

=== Model ===
Model: claude-3-5-sonnet-20241022
Temperature: 0.7

=== Template ===
You are a helpful assistant...

=== Structured Output Schema ===
{
  "type": "object",
  ...
}
```

**Help Access**:
```bash
langstar prompt get help
langstar prompt get help details
```

---

### 3. `create` - Create New Prompt

**Purpose**: Create a new prompt with text, metadata, model, and optional schema

**Syntax**:
```bash
langstar prompt create <prompt-name> [options]
```

**Options**:
- `--text <TEXT>`: Prompt template text (inline)
- `--text-file <FILE>`: Read template from file
- `--metadata <JSON>`: Metadata as JSON string
- `--model <MODEL>`: Associated model name
- `--structured-output <FILE>`: JSON schema file path
- `--public`: Make prompt public (default: private)
- `--workspace-id <ID>`: Workspace scope override

**Examples**:
```bash
# Create simple prompt
langstar prompt create my-prompt --text "Hello {name}"

# Create from file
langstar prompt create my-prompt --text-file template.txt

# Create with model
langstar prompt create my-prompt \
  --text "Answer: {question}" \
  --model claude-3-5-sonnet-20241022

# Create structured prompt
langstar prompt create data-extractor \
  --text-file extraction-template.txt \
  --structured-output schema.json \
  --model claude-3-5-sonnet-20241022
```

**Help Access**:
```bash
langstar prompt create help
langstar prompt create help details
```

---

### 4. `update` - Update Existing Prompt

**Purpose**: Update prompt content with modifiers for specific aspects

**Syntax**:
```bash
# Update prompt text
langstar prompt update <prompt-name> --text <TEXT>
langstar prompt update <prompt-name> --text-file <FILE>

# Update specific aspect
langstar prompt update {metadata|model|structured-output} <prompt-name> [options]
```

**Options** (for text update):
- `--text <TEXT>`: New template text (inline)
- `--text-file <FILE>`: Read new template from file

**Options** (for metadata update):
- `--json <JSON>`: Metadata as JSON string
- `--public / --private`: Change visibility

**Options** (for model update):
- `--model <MODEL>`: New model name
- `--temperature <N>`: Model temperature
- `--max-tokens <N>`: Max tokens

**Options** (for structured-output update):
- `--schema <FILE>`: New JSON schema file

**Examples**:
```bash
# Update prompt text
langstar prompt update my-prompt --text "New template"
langstar prompt update my-prompt --text-file new-template.txt

# Update metadata
langstar prompt update metadata my-prompt --public

# Update model
langstar prompt update model my-prompt --model claude-opus-4-5

# Update schema
langstar prompt update structured-output my-prompt --schema new-schema.json
```

**Help Access**:
```bash
langstar prompt update help
langstar prompt update help details
```

---

## User Workflow Decision Tree

### Decision Tree

```
User says → Identify intent → Choose command

"List my prompts"
"Show all prompts"
  └→ Intent: Browse → `list`

"Find prompts about X"
"Search for Y"
  └→ Intent: Search → `list <term>`

"Show me my prompt"
"Get the text for X"
  └→ Intent: Read text → `get <name>`

"What's the metadata?"
"When was it created?"
  └→ Intent: Read metadata → `get metadata <name>`

"What model does it use?"
  └→ Intent: Read model → `get model <name>`

"Show me the schema"
"What's the output structure?"
  └→ Intent: Read schema → `get structured-output <name>`

"Show me everything"
  └→ Intent: Read all → `get <name> --all`

"Create a new prompt"
"Make a prompt called X"
  └→ Intent: Create → `create <name> [options]`

"Update my prompt"
"Change the text to X"
  └→ Intent: Update → `update <name> [options]`
```

### Common User Phrases → Command Mapping

| User Says | Command | Why |
|-----------|---------|-----|
| "List prompts" | `list` | Browse all |
| "List everything" | `list --all` | All pages |
| "Find RAG prompts" | `list rag` | Search term |
| "Show me my prompt" | `get <name>` | Default: text only |
| "What's in my prompt?" | `get <name>` | Text only |
| "When was it created?" | `get metadata <name>` | Metadata aspect |
| "What model does it use?" | `get model <name>` | Model aspect |
| "Show me the schema" | `get structured-output <name>` | Schema aspect |
| "Show me everything" | `get <name> --all` | All aspects |
| "Create a prompt" | `create <name> --text "..."` | New prompt |
| "Update my prompt" | `update <name> --text "..."` | Update text |

---

## Implementation Plan

### Phase 1: Help System Infrastructure (Non-Breaking)

**Issue**: 668.1

**Changes**:
1. Add `help` subcommand infrastructure to CLI
2. Implement three-level help system:
   - Level 1: Command overview (`langstar prompt help`)
   - Level 2: Command-specific help (`langstar prompt list help`)
   - Level 3: Detailed help (`langstar prompt list help details`)
3. Keep `--help` flag working alongside `help` subcommand
4. Document progressive disclosure pattern for other commands

**Testing**:
- Verify all help levels render correctly
- Test help for each command
- Ensure `--help` still works

---

### Phase 2: Implement CRUD Commands (Non-Breaking)

**Issue**: 668.2

**Changes**:
1. Add `get` command with modifiers:
   - Default: return text only
   - `get metadata <name>`: metadata
   - `get model <name>`: model info
   - `get structured-output <name>`: schema
   - `get <name> --all`: everything
2. Add `create` command replacing `push`
3. Add `update` command for modifications
4. Keep existing `push` and `pull` as hidden deprecated commands

**Migration Notices**:
- `pull` → Use `get` or `get --all`
- `push` → Use `create` or `update`

---

### Phase 3: Consolidate List/Search (Breaking)

**Issue**: 668.3

**Changes**:
1. Add optional `[term]` positional argument to `list`
2. Add `--all` flag for pagination control
3. Deprecate `search` command with warning
4. Update documentation

**Migration**:
```bash
# Old: langstar prompt search "term"
# New: langstar prompt list "term"

# Old: langstar prompt search "term" --limit 100
# New: langstar prompt list "term" --all
```

---

### Phase 4: Remove Deprecated Commands (Major Version)

**Issue**: 668.4

**Version**: v2.0.0 (breaking)

**Changes**:
1. Remove `search` command entirely
2. Remove `pull` command entirely
3. Remove `push` command entirely
4. Remove `--help` flag (use `help` subcommand only)

---

## Rationale for Design Decisions

### Why Progressive Disclosure Help?

**Problem**: Flat help dumps everything at once
- Overwhelming for new users
- Hard for AI agents to parse
- No guidance for exploration

**Solution**: Hierarchical help with levels
- Level 1: High-level overview (what commands exist)
- Level 2: Command usage (how to use this command)
- Level 3: Deep dive (all details and edge cases)

**Benefits**:
- AI agents can explore progressively
- Users aren't overwhelmed
- Clear path to find specific information

### Why CRUD Vocabulary?

**Problem**: Git terms confuse non-coders
- "Pull" requires git knowledge
- "Push" implies version control
- "Cat" is Unix-specific

**Solution**: Standard CRUD verbs
- Create, Read (Get), Update, Delete
- Widely understood across industries
- Consistent with other CLI tools

**Benefits**:
- Intuitive for non-technical users
- Aligns with REST API patterns
- Consistent with other langstar commands

### Why Modifiers Instead of Separate Commands?

**Problem**: Too many commands creates confusion
- `info`, `cat`, `pull`, `get` all do similar things
- Hard to remember which does what

**Solution**: Single `get` command with modifiers
- `get <name>` - default (text)
- `get metadata <name>` - specific aspect
- `get <name> --all` - everything

**Benefits**:
- Clear mental model: "get" is for reading
- Modifiers make intent explicit
- Fewer commands to remember

---

## Success Criteria

- [ ] Progressive disclosure help system implemented at all levels
- [ ] CRUD vocabulary replaces git-like terms
- [ ] `list` and `search` consolidated into single command
- [ ] AI agents can reliably choose correct command from help
- [ ] Non-coders understand commands without git knowledge
- [ ] Editing workflow is clear: `get <name>` → edit → `update <name>`
- [ ] Structured prompt schemas visible with `get structured-output`
- [ ] Migration path documented for breaking changes

---

## References

- Parent issue: #668 (ls-prompt-ux milestone)
- Structured outputs research: docs/research/398-structured-output-prompts-scout.md
- CLI output DX: #529 (systematic CLI patterns)
- Current implementation: cli/src/commands/prompt.rs
- Review feedback: PR #724 comment r2716746624
