# Structured Output Prompts Research Report

**Issue**: [#398](https://github.com/codekiln/langstar/issues/398) - Research report - scout resources for structured output prompts
**Milestone**: [#7 - ls-prompt-structured-outputs](https://github.com/codekiln/langstar/milestone/7)
**Date**: 2025-11-29

## Executive Summary

This document captures research findings on structured output prompts in LangSmith to inform the design of Langstar's Rust-based structured output prompt support. Structured output prompts allow defining a JSON schema that constrains LLM output format, enabling reliable extraction of typed data.

### Key Findings

1. **StructuredPrompt** is the core class (`langchain_core.prompts.structured.StructuredPrompt`)
2. The Python SDK applies **transform logic** when pushing/pulling prompts to handle the StructuredPrompt ↔ RunnableSequence conversion
3. The API stores prompts as **serialized LangChain objects** (LC-JSON format) in the `manifest` field
4. Langstar currently has **no support** for structured output prompts - this is greenfield work
5. **CRITICAL**: Pydantic classes cannot be serialized - use JSON schema dicts instead (see Section 9)

## Data Sources

| Source | Location | Description |
|--------|----------|-------------|
| LangSmith OpenAPI Spec | `reference/openapi/langchain/langsmith/openapi.json` | API specification |
| LangSmith Python SDK | `reference/repo/langchain-ai/langsmith-sdk/code/` | Official SDK implementation |
| LangSmith MCP Server | `reference/repo/langchain-ai/langsmith-mcp-server/code/` | MCP server prompt handling |
| Experiment Scripts | `reference/experiments/398-structured-output-prompts/` | Test scripts for this research |

## 1. What Are Structured Output Prompts?

Structured output prompts combine:
1. A **prompt template** (messages with variables)
2. A **JSON schema** defining the expected output structure
3. A **method** for how the schema is applied (`json_schema` or `function_calling`)

When used with an LLM, the schema constrains the output to match the defined structure, enabling:
- Type-safe data extraction
- Consistent API responses
- Validated outputs

### Example Use Case

```python
from pydantic import BaseModel
from langchain_core.prompts.structured import StructuredPrompt

class MovieReview(BaseModel):
    title: str
    rating: int  # 1-10
    summary: str
    recommended: bool

prompt = StructuredPrompt(
    messages=[...],
    schema_=MovieReview,
    method="json_schema"
)
```

## 2. LangSmith SDK Implementation

### 2.1 Key Classes and Locations

| Component | Location | Purpose |
|-----------|----------|---------|
| `StructuredPrompt` | `langchain_core.prompts.structured` | Core class for structured prompts |
| `pull_prompt()` | `langsmith/client.py:7696` | Pull prompt from LangSmith |
| `push_prompt()` | `langsmith/client.py:8542` | Push prompt to LangSmith |
| Transform logic (pull) | `langsmith/client.py:7776-7794` | Convert stored format to runnable |
| Transform logic (push) | `langsmith/client.py:8761-8794` | Convert runnable to storage format |

### 2.2 The Transform Problem

LangSmith stores prompts as serialized objects. However, when a structured prompt is bound to a model with `with_structured_output()`, it creates a `RunnableSequence`:

```
ChatPromptTemplate | ChatModel.with_structured_output(schema)
```

This is a 2-step sequence that cannot be directly serialized back to a `StructuredPrompt`.

The SDK handles this with transform logic:

#### Pull Transform (client.py:7776-7794)

When pulling with `include_model=True`:
1. Deserializes the manifest
2. If it's a `StructuredPrompt`, wraps it to create the proper runnable chain
3. Adds an output parser step (2-step → 3-step)

```python
# Pseudo-code from SDK
if isinstance(prompt, StructuredPrompt) and include_model:
    # Add structured output handling
    return prompt | model.with_structured_output(prompt.schema_)
```

#### Push Transform (client.py:8761-8794)

When pushing a prompt:
1. Inspects the incoming object
2. If it's a sequence with structured output, extracts the schema
3. Normalizes to `StructuredPrompt` for storage (3-step → 2-step)

The `ls_structured_output_format` kwarg is used internally to pass the structured output configuration.

### 2.3 Manifest Structure

The `manifest` field in a prompt commit is a flexible JSON object using LangChain's serialization format ("LC-JSON"). For a `StructuredPrompt`:

```json
{
  "lc": 1,
  "type": "constructor",
  "id": ["langchain", "prompts", "structured", "StructuredPrompt"],
  "kwargs": {
    "messages": [
      {
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "chat", "SystemMessagePromptTemplate"],
        "kwargs": {
          "prompt": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
            "kwargs": {
              "template": "You are a helpful assistant.",
              "input_variables": [],
              "template_format": "f-string"
            }
          }
        }
      }
    ],
    "schema_": {
      "type": "object",
      "title": "Response",
      "properties": {
        "answer": {"type": "string"},
        "confidence": {"type": "number"}
      },
      "required": ["answer", "confidence"]
    },
    "method": "json_schema"
  }
}
```

#### Key Fields

| Field | Type | Description |
|-------|------|-------------|
| `lc` | int | LangChain serialization version (always 1) |
| `type` | string | Always "constructor" for class instances |
| `id` | string[] | Module path to the class |
| `kwargs` | object | Constructor arguments |
| `kwargs.messages` | array | Message templates |
| `kwargs.schema_` | object | JSON Schema for output |
| `kwargs.method` | string | "json_schema" or "function_calling" |

## 3. API Endpoints

### 3.1 Prompt Repository Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/repos/` | List prompt repositories |
| POST | `/api/v1/repos/` | Create prompt repository |
| GET | `/api/v1/repos/{owner}/{repo}` | Get repository |
| DELETE | `/api/v1/repos/{owner}/{repo}` | Delete repository |

### 3.2 Commit Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/commits/{owner}/{repo}/` | List commits |
| POST | `/api/v1/commits/{owner}/{repo}/` | Create commit |
| GET | `/api/v1/commits/{owner}/{repo}/{commit}` | Get commit |

**Note**: When using the default owner ("-"), the path simplifies to `/api/v1/commits/{repo}/`. The experiment scripts use this simplified form.

### 3.3 Prompt Pull/Push (SDK Convenience)

The SDK's `pull_prompt()` and `push_prompt()` methods wrap these endpoints with:
- LangChain object serialization/deserialization
- Transform logic for structured outputs
- Model binding support

## 4. Current Langstar Implementation

### 4.1 Existing Prompt Support

Langstar has basic prompt support in:
- `cli/src/commands/prompt.rs` - CLI commands
- `sdk/src/prompts.rs` - SDK types

Current capabilities:
- `prompt list` - List prompts
- `prompt get` - Get prompt details
- `prompt pull` - Pull prompt manifest (raw JSON)
- `prompt push` - Push prompt (limited)

### 4.2 What's Missing

| Feature | Status | Notes |
|---------|--------|-------|
| `StructuredPrompt` type | ❌ Missing | No Rust equivalent |
| Schema handling | ❌ Missing | No JSON Schema support |
| Transform logic | ❌ Missing | No pull/push transforms |
| LC-JSON serialization | ❌ Missing | No LangChain format support |
| Method selection | ❌ Missing | No json_schema/function_calling |

## 5. Implementation Recommendations

### 5.1 Phase 1: Research & Design (This Issue)

- [x] Document Python SDK behavior
- [x] Create experiment scripts
- [x] Run experiments to capture actual manifest formats
- [x] Document edge cases (Pydantic serialization issue)

### 5.2 Phase 2: SDK Types

1. Create `StructuredPrompt` Rust type
2. Implement JSON Schema types (or use `serde_json::Value`)
3. Add LC-JSON serialization support

### 5.3 Phase 3: CLI Support

1. `prompt push --schema <file.json>` - Push with schema
2. `prompt push --schema-from-pydantic <module.Class>` - (Future: Python interop)
3. `prompt pull --include-model` - Pull with model binding support

### 5.4 Phase 4: Transform Logic

Decide whether to:
- **Option A**: Implement full transform logic in Rust (complex, full compatibility)
- **Option B**: Store/retrieve raw manifests only (simpler, partial compatibility)
- **Option C**: Shell out to Python SDK for transforms (pragmatic, full compatibility)

## 6. Open Questions (Answered)

1. **Schema format**: ~~Should we accept Pydantic models (requires Python), JSON Schema files, or both?~~
   - **Answer**: JSON Schema files only. Pydantic classes cannot be serialized to LC-JSON format - they become `null`. Users should run `pydantic_model.model_json_schema()` locally and pass the resulting dict/file.

2. **Transform parity**: ~~How important is it to match Python SDK transforms exactly?~~
   - **Answer**: For basic structured prompt push/pull, we don't need full transform parity. The manifest format is straightforward. Transform logic is mainly needed for `with_structured_output()` model binding scenarios.

3. **Model binding**: ~~Do we need `include_model` support, or just raw prompt storage?~~
   - **Answer**: Start with raw prompt storage only. Model binding is a separate concern that would require implementing the full chain serialization.

4. **Validation**: ~~Should we validate schemas client-side before pushing?~~
   - **Answer**: Yes, basic JSON Schema validation would prevent invalid schemas from being stored. Use a JSON Schema validation crate.

## 7. References

### Code References

- Pull transform: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:7776-7794`
- Push transform: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:8761-8794`
- StructuredPrompt import: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:7725`

### Documentation

- [LangChain Structured Output](https://python.langchain.com/docs/how_to/structured_output/)
- [LangSmith Prompts](https://docs.smith.langchain.com/prompt-engineering)

### Experiment Files

- `reference/experiments/398-structured-output-prompts/README.md`
- `reference/experiments/398-structured-output-prompts/run_test.sh`
- `reference/experiments/398-structured-output-prompts/test_structured_prompts.py`

## 9. Experimental Findings

Experiments were run using `reference/experiments/398-structured-output-prompts/` to validate assumptions.

### 9.1 Schema Serialization Discovery

**Critical Finding**: Pydantic classes **cannot be serialized** to LC-JSON format.

When passing a Pydantic class to `StructuredPrompt.schema_`:
```python
class MovieReview(BaseModel):
    title: str
    rating: int

prompt = StructuredPrompt(messages=[...], schema_=MovieReview, method="json_schema")
```

The serialized manifest contains:
```json
"schema_": {"lc": 1, "type": "not_implemented", "id": ["__main__", "MovieReview"], "repr": "<class '__main__.MovieReview'>"}
```

When stored in LangSmith, `"not_implemented"` becomes `null`, losing the schema entirely.

**Solution**: Pass a JSON schema dict instead of Pydantic class:
```python
json_schema = MovieReview.model_json_schema()  # Get dict from Pydantic
prompt = StructuredPrompt(messages=[...], schema_=json_schema, method="json_schema")
```

This serializes correctly and round-trips through LangSmith.

### 9.2 Validated Manifest Structure

Actual manifest structure captured from LangSmith (with proper JSON schema):

```json
{
  "lc": 1,
  "type": "constructor",
  "id": ["langchain_core", "prompts", "structured", "StructuredPrompt"],
  "kwargs": {
    "input_variables": ["movie_name"],
    "messages": [
      {
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "chat", "SystemMessagePromptTemplate"],
        "kwargs": {
          "prompt": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
            "kwargs": {
              "input_variables": [],
              "template": "You are a movie critic. Provide a structured review.",
              "template_format": "f-string"
            },
            "name": "PromptTemplate"
          }
        }
      },
      {
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "chat", "HumanMessagePromptTemplate"],
        "kwargs": {
          "prompt": {
            "lc": 1,
            "type": "constructor",
            "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
            "kwargs": {
              "input_variables": ["movie_name"],
              "template": "Review the movie: {movie_name}",
              "template_format": "f-string"
            },
            "name": "PromptTemplate"
          }
        }
      }
    ],
    "schema_": {
      "description": "A structured movie review.",
      "properties": {
        "title": {"description": "The movie title", "title": "Title", "type": "string"},
        "rating": {"description": "Rating from 1-10", "maximum": 10, "minimum": 1, "title": "Rating", "type": "integer"},
        "summary": {"description": "Brief summary", "title": "Summary", "type": "string"}
      },
      "required": ["title", "rating", "summary"],
      "title": "MovieReview",
      "type": "object"
    },
    "structured_output_kwargs": {
      "method": "json_schema"
    }
  },
  "name": "StructuredPrompt"
}
```

### 9.3 Key Observations

| Observation | Detail |
|-------------|--------|
| Module path uses underscore | `langchain_core` not `langchain-core` |
| Schema stored in `schema_` | Underscore suffix matches Python kwarg |
| Method stored separately | In `structured_output_kwargs.method` |
| Valid methods | `"json_schema"` or `"function_calling"` |
| `name` field | Redundant with `id[-1]`, present for clarity |
| `input_variables` | Extracted from template placeholders |

### 9.4 Deserialization Works

Prompts created with JSON schema dicts deserialize correctly:
```python
prompt = client.pull_prompt("test-structured-398-v2", include_model=False)
# Returns: StructuredPrompt with schema_ as dict
```

### 9.5 Implications for Langstar

1. **Accept JSON Schema files** - Don't try to support Pydantic classes
2. **Use `serde_json::Value`** - Flexible JSON handling for schema_
3. **Parse `structured_output_kwargs`** - Extract method from nested object
4. **Generate LC-JSON format** - Match the exact structure above

## 10. Next Steps

1. ~~Run experiment scripts to capture actual manifest formats~~ ✅
2. ~~Update this report with findings~~ ✅
3. ~~Create parent epic for structured output prompt implementation~~ ✅ (Issue #402)
4. ~~Break down into sub-issues per phase~~ ✅ (Issues #403-#409)

## 11. Design Decisions

**Issue**: [#403](https://github.com/codekiln/langstar/issues/403) - Design DX consistency and configuration integration
**Date**: 2025-11-29

This section documents the design decisions for integrating structured output prompts into the Langstar CLI, ensuring consistency with existing commands and patterns.

### 11.1 DX Consistency Analysis

#### Existing `prompt push` Command Pattern

The current `prompt push` command uses this flag pattern:

```rust
/// Push/create a prompt in PromptHub
Push {
    #[arg(short, long)]      owner: String,           // -o/--owner
    #[arg(short, long)]      repo: String,            // -r/--repo
    #[arg(short, long)]      template: String,        // -t/--template
    #[arg(short, long)]      input_variables: Option<String>,  // -i/--input-variables
    #[arg(long)]             template_format: String, // --template-format (default: f-string)
    #[arg(long)]             organization_id: Option<String>,  // --organization-id
    #[arg(long)]             workspace_id: Option<String>,     // --workspace-id
}
```

**Observations**:
- Short flags (`-o`, `-r`, `-t`, `-i`) for frequently used options
- Long-only flags for less common options (`--template-format`, `--organization-id`)
- Defaults provided where sensible (`template_format = "f-string"`)

#### File Input Pattern from `dataset import`

The `dataset import` command establishes the file input pattern:

```rust
/// Path to the file to import (JSONL or CSV)
#[arg(long)]
pub file: PathBuf,

/// File format: jsonl or csv (auto-detected from extension if not specified)
#[arg(long)]
pub format: Option<String>,
```

**Observations**:
- Uses `--file <path>` (long flag only) for file paths
- Auto-detects format from extension with optional `--format` override
- Uses `PathBuf` type for proper path handling

#### Recommended CLI Interface Design

**For `prompt push` with structured output support:**

```rust
/// Push/create a prompt in PromptHub
Push {
    // Existing flags (unchanged)
    #[arg(short, long)]
    owner: String,

    #[arg(short, long)]
    repo: String,

    #[arg(short, long)]
    template: String,

    #[arg(short, long)]
    input_variables: Option<String>,

    #[arg(long, default_value = "f-string")]
    template_format: String,

    // NEW: Schema support
    /// Path to JSON Schema file for structured output
    #[arg(long, value_name = "FILE")]
    schema: Option<PathBuf>,

    /// Structured output method: json_schema or function_calling
    #[arg(long, default_value = "json_schema")]
    schema_method: String,

    // Existing scoping (unchanged)
    #[arg(long)]
    organization_id: Option<String>,

    #[arg(long)]
    workspace_id: Option<String>,
}
```

**Design Rationale**:

| Decision | Rationale |
|----------|-----------|
| `--schema <FILE>` (long only) | Matches `--file` pattern from dataset commands; not used frequently enough for short flag |
| `--schema-method` (not `--method`) | Explicit naming avoids ambiguity; clearly indicates it relates to schema handling |
| Default `json_schema` | Most common method; matches Python SDK defaults |
| `PathBuf` type | Proper path handling, consistent with dataset import |
| Optional schema | Backward compatible; existing prompts don't require schema |

**Usage Examples**:

```bash
# Push regular prompt (existing behavior)
langstar prompt push -o owner -r repo -t "Hello {name}"

# Push structured prompt with schema
langstar prompt push -o owner -r repo -t "Analyze {topic}" \
  --schema ./schemas/analysis.json

# Push structured prompt with function_calling method
langstar prompt push -o owner -r repo -t "Extract {data}" \
  --schema ./schemas/extraction.json \
  --schema-method function_calling
```

#### Intentional Deviations from Existing Patterns

| Deviation | Rationale |
|-----------|-----------|
| No `-s` short flag for `--schema` | `-s` conflicts with potential future `--search` or `--sort` flags |
| `--schema-method` instead of `--schema-format` | "method" matches LangSmith terminology exactly (`json_schema` vs `function_calling` are methods, not formats) |

### 11.2 Configuration Integration

#### Environment Variables (Existing)

The structured output feature uses **only existing environment variables**:

| Variable | Purpose | Required |
|----------|---------|----------|
| `LANGSMITH_API_KEY` | API authentication | Yes |
| `LANGSMITH_WORKSPACE_ID` | Workspace scoping | No |
| `LANGSMITH_ORGANIZATION_ID` | Organization scoping | No |

**No new environment variables required.** The schema file path is provided via CLI flag, not environment configuration.

#### Configuration Precedence

Following the established pattern in `prompt.rs`:

```
CLI flags → Environment variables → Config file → Defaults
```

**Specific to structured outputs**:
- `--schema`: CLI only (no env var - file paths shouldn't be in env)
- `--schema-method`: CLI flag value is used if provided; otherwise, the default `json_schema` is used (follows precedence: CLI flag > default)

#### Error Handling for Missing Configuration

Following existing patterns, error messages should guide users:

```
Error: Schema file not found: ./schemas/analysis.json

Hint: Ensure the file exists and path is correct:
  langstar prompt push --schema ./schemas/analysis.json ...
```

```
Error: Invalid schema method 'invalid'. Valid methods: json_schema, function_calling

Hint: Use --schema-method to specify the structured output method:
  langstar prompt push --schema-method json_schema ...
```

### 11.3 Business Purpose and User Scenarios

#### What Structured Output Prompts Enable

Structured output prompts solve a critical problem: **ensuring LLM outputs conform to a predictable, typed format**. This enables:

1. **Reliable data extraction** - Parse LLM responses as typed objects
2. **API integration** - LLM outputs match expected response schemas
3. **Validation** - Reject invalid outputs before downstream processing
4. **Consistency** - Same prompt produces same output structure every time

#### UI Workflow Correspondence

In the LangSmith UI, creating a structured output prompt involves:

1. Navigate to Prompt Hub
2. Create/edit a prompt
3. Add messages (system, human, etc.)
4. Enable "Structured Output" toggle
5. Define JSON Schema (manual JSON entry or Pydantic class)
6. Select method (json_schema or function_calling)
7. Save/commit the prompt

**CLI Equivalent**:

```bash
# Equivalent to UI workflow
langstar prompt push \
  -o my-org -r my-prompt \
  -t "You are a data extractor. Extract: {input}" \
  -i "input" \
  --schema ./schemas/extraction.json \
  --schema-method json_schema
```

#### Key User Scenarios

**Scenario 1: Data Extraction Pipeline**

```bash
# User has a Pydantic model defining their output format
# 1. Export schema from Pydantic (user's Python code)
python -c "from mymodels import Invoice; print(Invoice.model_json_schema())" > invoice.json

# 2. Push structured prompt
langstar prompt push -o team -r invoice-extractor \
  -t "Extract invoice data from: {document}" \
  --schema invoice.json
```

**Scenario 2: API Response Formatting**

```bash
# Ensure API responses match OpenAPI schema
langstar prompt push -o team -r api-formatter \
  -t "Format the following data as an API response: {data}" \
  --schema ./api/response-schema.json \
  --schema-method json_schema
```

**Scenario 3: Structured Analysis**

```bash
# Sentiment analysis with structured output
langstar prompt push -o team -r sentiment-analyzer \
  -t "Analyze sentiment: {text}" \
  --schema ./schemas/sentiment.json
```

#### Getting Structured Prompts

```bash
# Get and view structured prompt
langstar prompt get owner/prompt-name

# Output shows schema information
# Manifest:
# {
#   "id": ["langchain_core", "prompts", "structured", "StructuredPrompt"],
#   "kwargs": {
#     "schema_": { ... },
#     "structured_output_kwargs": { "method": "json_schema" }
#   }
# }
```

### 11.4 CLI Advantage Over UI

| Aspect | UI | CLI |
|--------|----|----|
| **Automation** | Manual clicks each time | Script once, run anywhere |
| **Version Control** | UI-based versioning | Schema files in git |
| **CI/CD Integration** | Not possible | `langstar prompt push` in pipeline |
| **Batch Operations** | One at a time | Loop over multiple schemas |
| **Schema Reuse** | Copy-paste | Reference same file |
| **Diff/Review** | UI comparison | `git diff schema.json` |

**Key CLI advantages for structured outputs specifically**:

1. **Schema file management** - Keep JSON schemas in version control alongside code
2. **Reproducibility** - Same command produces same prompt every time
3. **Pipeline integration** - Update prompts as part of deployment
4. **Testing** - Validate schemas locally before pushing

### 11.5 Implementation Summary

#### SDK Changes Required

1. **New type**: `StructuredPrompt` in `sdk/src/prompts.rs`
2. **LC-JSON serialization**: Generate proper manifest format
3. **Schema validation**: Validate JSON schema before push

#### CLI Changes Required

1. **New flags**: `--schema`, `--schema-method` on `prompt push`
2. **File reading**: Load and validate schema from file
3. **Output display**: Show schema info when getting structured prompts

#### Validation Strategy

**Before push**:
1. Validate schema is valid JSON
2. Validate schema is valid JSON Schema (using `jsonschema` crate)
3. Validate method is `json_schema` or `function_calling`

**Error messages**:
- Invalid JSON: `Schema file contains invalid JSON: <parse error>`
- Invalid schema: `Schema file is not a valid JSON Schema: <validation error>`
- Invalid method: `Invalid schema method. Valid options: json_schema, function_calling`

### 11.6 Design Decisions Summary Table

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Schema input | `--schema <FILE>` | Matches dataset import pattern, file paths via flag |
| Method selection | `--schema-method` with default | Explicit naming, sensible default |
| Default method | `json_schema` | Most common, matches Python SDK |
| New env vars | None | Schema paths shouldn't be in environment |
| Validation | Client-side before push | Fail fast, better error messages |
| Short flags | None for schema options | Avoid conflicts, these are less frequent |
| Backward compatibility | Schema is optional | Existing prompts work unchanged |
