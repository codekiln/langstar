# Research: --plain Mode for AI-Friendly CLI Output

**Issue**: #581
**Parent**: #529 (ls-cli-output-dx milestone)
**Date**: 2025-12-05 (Updated: 2025-12-10)

## Executive Summary

This research evaluates approaches for scriptable CLI output in langstar and resolves the `-o` short flag conflict discovered during implementation of #587.

**Original recommendation:** `-o/--output` flag with `--columns` for field selection.

**Updated recommendation after conflict discovery:** Use the existing **`-f/--format` global flag** (no short flag for output) and extend it with `text` format. This avoids conflicts with existing `-o` short flags used for pagination `--offset` in multiple commands.

## Post-Implementation Finding: `-o` Short Flag Conflict (2025-12-10)

During implementation of #587, a constraint was discovered: **`-o` is already used for `--offset` in pagination contexts**. This section documents the audit and design decision.

### Audit of `-o` Short Flag Usage

**Commands using `-o` for `--offset` (pagination):**

1. **`prompt list`** (`cli/src/commands/prompt.rs:24`)
   - `#[arg(short, long, default_value = "0")]`
   - Pagination parameter: number of prompts to skip
2. **`assistant list`** (`cli/src/commands/assistant.rs:24`)
   - `#[arg(short, long, default_value = "0")]`
   - Pagination parameter: number of assistants to skip
3. **`model-config list`** (`cli/src/commands/model_config.rs:25`)
   - `#[arg(short, long, default_value = "0")]`
   - Pagination parameter: number of items to skip

**Commands using `-o` for `--output` (format selection):**
4. **`runs query`** (`cli/src/commands/runs.rs:136`)

- `#[arg(short = 'o', long = "output", default_value = "table", value_enum)]`
- Note: Intentionally uses `-o` to avoid conflict with global `-f/--format` flag
- Has its own `RunsOutputFormat` enum (Table, Json, JsonPretty)

**Commands with `--offset` but NO short flag:**
5. **`deployment list`** (`cli/src/commands/deployment.rs:29`)

- `#[arg(long, default_value = "0")]`
- Only long form, no short flag

**Global format flag:**

- **`main.rs:27`**: `-f/--format` is global for all commands
- Environment variable: `LANGSTAR_OUTPUT_FORMAT`
- Current formats: `json`, `table`

**Summary:**

- **3 commands** use `-o` for `--offset`
- **1 command** uses `-o` for `--output` (special case for `runs query`)
- **1 command** has `--offset` with no short flag
- Global `-f` is the standard format flag

### Design Decision: Use Existing `-f/--format` Global Flag

**Decision:** Extend the existing `-f/--format` global flag with a `text` format option. Do NOT introduce `-o/--output` as a new flag or reclaim `-o` from pagination.

**Rationale:**

1. **Minimize Breaking Changes**
   - Removing `-o` from 3 pagination commands would break existing user scripts
   - Users expect pagination flags to be consistent (`-l` for limit, `-o` for offset)

2. **Leverage Existing Architecture**
   - `-f/--format` is already global and well-established
   - Environment variable `LANGSTAR_OUTPUT_FORMAT` already exists
   - `OutputFormat` enum in `cli/src/output.rs` is the single source of truth

3. **Consistency with Pagination**
   - Short flags for pagination should be uniform across commands
   - `deployment list` is the outlier (should gain `-o` for consistency)

4. **Special Case for `runs query`**
   - `runs query` can keep its `-o/--output` since it's command-specific
   - Its `RunsOutputFormat` offers more granular control (Table, Json, JsonPretty)
   - Document that `runs` is an exception due to its unique output needs

5. **CLI Precedent Alignment (Partial)**
   - While kubectl uses `-o`, we already committed to `-f` in initial design
   - AWS CLI uses `--output` (long form only)
   - Our `-f` is closer to Docker CLI's `--format`
   - Consistency within langstar > mimicking kubectl exactly

**Trade-offs Accepted:**

- ❌ Not as terse as `-o` for output format
- ✅ Maintains backward compatibility
- ✅ Avoids flag conflicts and user confusion
- ✅ Uses existing, tested infrastructure

## CLI Precedent Survey

### GitHub CLI (gh)

**Pattern**: JSON-first with post-processing

```bash
# List fields available
gh issue list --help | grep json

# Select specific fields
gh issue list --json number,title,state

# Post-process with jq
gh issue list --json number,title --jq '.[] | [.number, .title] | @tsv'
```

**Key characteristics**:

- `--json <fields>` takes comma-separated field list
- `--jq <expr>` for filtering/transforming output
- `--template <tmpl>` for Go template formatting
- Fields are discoverable via help text

**Example output**:

```
581	529.1-cli-output-research Research: --plain mode for AI-friendly CLI output
573	556.8-integration Update AGENTS.md/CLAUDE.md and validate progressive disclosure
```

### kubectl

**Pattern**: Output format flag with field selectors

```bash
# Multiple output formats
kubectl get pods -o json          # JSON format
kubectl get pods -o yaml          # YAML format
kubectl get pods -o wide          # Extended table

# Custom columns with field paths
kubectl get node -o custom-columns='NODE_NAME:.metadata.name,STATUS:.status.conditions[?(@.type=="Ready")].status'

# JSONPath for field selection
kubectl get pods -o jsonpath='{.items[*].metadata.name}'
```

**Key characteristics**:

- `-o/--output` flag controls format (json, yaml, wide, custom-columns, jsonpath, go-template)
- `custom-columns` format: `NAME:FIELD_PATH,NAME2:FIELD_PATH2`
- JSONPath provides powerful field selection
- Format names are descriptive (not abbreviations)

### AWS CLI

**Pattern**: Multiple output formats with query language

```bash
# Text format for scripting (tab-separated)
aws iam list-users --output text

# Query specific fields
aws iam list-users --output text --query 'Users[*].[UserName,Arn,CreateDate]'

# JSON with query (default)
aws iam list-users --query 'Users[*].{Name:UserName,ID:UserId}'
```

**Key characteristics**:

- `--output` flag: json (default), yaml, yaml-stream, text, table
- **`text` format is tab-separated** - closest to scriptable output
- `--query` parameter uses JMESPath for field selection
- AWS recommends always using `--query` with `text` format
- Columns are alphabetically sorted in text format

**Example output** (text format):

```
Admin    arn:aws:iam::123456789012:user/Admin    2014-10-16T16:03:09+00:00
```

### Summary of Patterns

| Tool    | Format Flag         | Field Selection              | Delimiter          | Notable Features                |
| ------- | ------------------- | ---------------------------- | ------------------ | ------------------------------- |
| gh      | `--json <fields>`   | Comma-separated list in flag | newline/tsv via jq | JSON-first, post-process        |
| kubectl | `-o <format>`       | `custom-columns='COL:PATH'`  | table/custom       | Rich format ecosystem           |
| aws     | `--output <format>` | `--query 'JMESPath'`         | tabs (text mode)   | Text format + query recommended |

**Common patterns**:

- Format selection via dedicated flag (`-o`, `--output`, `--json`)
- Field selection separate from format selection
- Tab-separated or structured delimiters for scripting
- Field names discoverable via help/docs

**No major CLI uses**:

- Individual `--<field-name>` boolean flags
- A single `--plain` flag for format switching

## Current Langstar Architecture

### Existing Output System

**File**: `cli/src/output.rs`

```rust
pub enum OutputFormat {
    Json,
    Table,
}
```

**Usage** in `cli/src/main.rs`:

```rust
/// Output format (json or table)
#[arg(short = 'f', long, global = true, env = "LANGSTAR_OUTPUT_FORMAT")]
format: Option<String>,
```

**Current state**:

- `-f/--format` flag (short form available)
- Global flag inherited by all commands
- Environment variable support: `LANGSTAR_OUTPUT_FORMAT`
- Two formats: json, table

### Example Command: `prompt list`

**Available columns** (from `cli/src/commands/prompt.rs:145-168`):

| Column      | Field           | Type   | Notes                   |
| ----------- | --------------- | ------ | ----------------------- |
| Handle      | `repo_handle`   | String | First column, truncated |
| Likes       | `num_likes`     | u32    | Only in --full mode     |
| Downloads   | `num_downloads` | u32    | Always shown            |
| Public      | `is_public`     | String | Only in --full mode     |
| Description | `description`   | String | Truncated to fit        |

**Current behavior**:

- Default view: Handle, Downloads, Description
- `--full` flag: All columns including Likes and Public
- No column customization beyond --full toggle

### Commands That Would Benefit

All list commands in langstar (from `cli/src/main.rs:33-72`):

1. ✅ **`prompt list`** - Already has compact/full modes
2. ✅ **`assistant list`** - Likely similar structure
3. ✅ **`graph list`** (deployments) - Mentioned in #529
4. ✅ **`runs query`** - Large result sets, many columns
5. ✅ **`queue list`** - Annotation queues
6. ✅ **`dataset list`** - Dataset management
7. ✅ **`eval list`** - Evaluation results
8. ✅ **`secrets list`** - Sensitive data, minimal output needed
9. ✅ **`model-config list`** - Model configurations

**Priority candidates**:

- `prompt list` - Most mature, good starting point
- `runs query` - Large datasets, most scriptability benefit
- `secrets list` - Security-sensitive, minimal output critical

## Design Recommendation

### Recommended Approach: Extend Existing `-f/--format` Global Flag

**Rationale**: Uses established langstar architecture while avoiding conflicts with pagination flags.

#### Proposed Changes

**1. Add `text` output format** (like AWS CLI)

```rust
pub enum OutputFormat {
    Json,      // Existing
    Table,     // Existing
    Text,      // NEW: tab-separated values
    Records,   // NEW: vertical format (from #529)
}
```

**2. Add `--columns` flag** (like kubectl custom-columns)

```bash
# List available columns
langstar prompt list --show-columns
# Output: handle, likes, downloads, public, description, created_at

# Select specific columns (using global -f flag)
langstar prompt list -f text --columns handle,downloads

# Output (tab-separated):
# my-prompt	123
# another-prompt	456
```

**3. Implementation pattern**

Each command defines column metadata:

```rust
pub struct ColumnMetadata {
    name: &'static str,           // "handle"
    display_name: &'static str,   // "Handle"
    accessor: fn(&T) -> String,   // Extract value from data
    default_visible: bool,        // Show by default?
    width_hint: Option<usize>,    // For table formatting
}
```

### Comparison: Approach A vs Approach B

| Aspect              | Approach A: `-f text --columns` | Approach B: `--plain --<col>` |
| ------------------- | ------------------------------- | ----------------------------- |
| **Precedent**       | ✅ Similar to Docker --format   | ❌ No major CLI uses this     |
| **Extensibility**   | ✅ Easy to add formats          | ⚠️ Plain is terminal state     |
| **Integration**     | ✅ Uses existing `-f` flag      | ⚠️ New parallel system         |
| **Flag count**      | ✅ Minimal (2 flags)            | ⚠️ O(n) flags per command      |
| **Discovery**       | ✅ `--show-columns` clear       | ⚠️ `--help` gets cluttered     |
| **AI-friendliness** | ✅ Single introspection point   | ⚠️ Must parse help text        |
| **Consistency**     | ✅ Uniform across commands      | ⚠️ Hard to maintain            |
| **Delimiter**       | ✅ Configurable/standard        | ⚠️ Unspecified in proposal     |
| **Compatibility**   | ✅ No pagination conflicts      | ❌ Would need to reclaim `-o` |

### Why Not `--plain` with `--<column>` Flags?

While creative, this approach has significant drawbacks:

1. **No CLI precedent**: Would make langstar unique but unfamiliar to users
2. **Flag explosion**: Each column becomes a CLI flag, cluttering `--help`
3. **Clap complexity**: Boolean flags per column harder to validate than single `--columns` string
4. **Non-extensible**: `--plain` suggests binary toggle, hard to add future formats
5. **Discovery friction**: AI must parse help text vs querying `--show-columns`

## Implementation Plan

### Phase 1: Core Infrastructure

**File**: `cli/src/output.rs`

1. Add `Text` variant to `OutputFormat` enum
2. Implement tab-separated renderer
3. Create `ColumnMetadata` trait/struct
4. Add `--columns` parsing and validation

**Effort**: ~4 hours, 1 PR

### Phase 2: Pilot Command

**File**: `cli/src/commands/prompt.rs`

1. Define column metadata for prompts
2. Implement `--show-columns` flag
3. Support `--columns` for field selection
4. Wire up text output format
5. Update tests

**Effort**: ~6 hours, 1 PR

**Example**:

```bash
# Before
langstar prompt list --full
# ╭─────────────────────┬───────┬───────────┬────────┬─────────────────────╮
# │ Handle              │ Likes │ Downloads │ Public │ Description         │
# ╰─────────────────────┴───────┴───────────┴────────┴─────────────────────╯

# After
langstar prompt list -f text --columns handle,downloads
# my-prompt	123
# another-prompt	456

langstar prompt list --show-columns
# Available columns: handle, likes, downloads, public, description, created_at
```

### Phase 3: Rollout

Apply pattern to remaining 8 commands:

- assistant list
- graph list
- runs query
- queue list
- dataset list
- eval list
- secrets list
- model-config list

**Effort**: ~2 hours per command, 8 PRs or 2-3 batched PRs

### Phase 4: Polish

1. Add `Records` format (psql `\x` style, from #529)
2. Config file defaults: `[output.prompt-list] columns = ["handle", "downloads"]`
3. Add `--hide-columns` as inverse operation
4. Performance optimization for large datasets

**Effort**: ~8 hours, 2-3 PRs

## AI Workflow Examples

### Discovery

```bash
# AI agent discovers columns
langstar prompt list --show-columns
# Output: handle, likes, downloads, public, description, created_at
```

### Extraction

```bash
# Get specific data
langstar prompt list -f text --columns handle | head -5
# Output (one per line):
# anthropics/summarize-v2
# langchain/qa-chain
# openai/code-review
```

### Processing

```bash
# Pipe to xargs for batch operations
langstar prompt list -f text --columns handle --limit 100 | \
  xargs -I {} langstar prompt get {} -f json | \
  jq '.num_downloads' | \
  awk '{sum+=$1} END {print "Total downloads:", sum}'
```

### Filtering

```bash
# Combine with standard Unix tools
langstar prompt list -f text --columns handle,downloads,public | \
  awk -F'\t' '$3=="true" && $2>100 {print $1}' | \
  sort
```

## Open Questions

1. **Delimiter configuration**: Should `-f text` always use tabs, or add `--delimiter` flag?
   - **Recommendation**: Tabs by default, add `--delimiter` later if needed

2. **Column aliases**: Should we support `--columns name,id` as shortcuts for `handle,downloads`?
   - **Recommendation**: No, use full field names for clarity

3. **Default columns**: Should each command have smart defaults when `--columns` not specified?
   - **Recommendation**: Yes, maintain current compact view as default

4. **Header row**: Should `-f text` include a header row?
   - **Recommendation**: No by default, add `--header` flag if needed (like `aws --output text`)

5. **`runs query` exception**: Should `runs query` migrate to global `-f` or keep its `-o/--output`?
   - **Recommendation**: Keep `-o` for now as a documented exception. Revisit if it causes user confusion.

## Success Criteria

- [ ] `-f text` format outputs tab-separated values
- [ ] `--columns` flag selects specific fields
- [ ] `--show-columns` discovers available fields
- [ ] All 9 list commands support new output system
- [ ] AI agents can introspect and extract data without parsing tables
- [ ] Unix pipelines work naturally with output
- [ ] Documentation updated with examples

## Related Issues

- **Parent**: #529 (ls-cli-output-dx milestone)
- **Related**: #554 (table formatting improvements - already merged)

## References

- GitHub CLI: https://cli.github.com/manual/gh_help_formatting
- kubectl output options: https://kubernetes.io/docs/reference/kubectl/quick-reference/
- AWS CLI output: https://docs.aws.amazon.com/cli/latest/userguide/cli-usage-output-format.html
- Langstar output module: `cli/src/output.rs`
