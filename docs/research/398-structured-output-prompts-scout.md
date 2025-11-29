# Structured Output Prompts Research Report

**Issue**: [#398](https://github.com/codekiln/langstar/issues/398) - Research report - scout resources for structured output prompts
**Milestone**: [#7 - ls-prompt-structured-outputs](https://github.com/codekiln/langstar/milestone/7)
**Date**: 2025-11-29

## Executive Summary

This document captures research findings on structured output prompts in LangSmith to inform the design of langstar's Rust-based structured output prompt support. Structured output prompts allow defining a JSON schema that constrains LLM output format, enabling reliable extraction of typed data.

### Key Findings

1. **StructuredPrompt** is the core class (`langchain_core.prompts.structured.StructuredPrompt`)
2. The Python SDK applies **transform logic** when pushing/pulling prompts to handle the StructuredPrompt ↔ RunnableSequence conversion
3. The API stores prompts as **serialized LangChain objects** (LC-JSON format) in the `manifest` field
4. Langstar currently has **no support** for structured output prompts - this is greenfield work

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

The `manifest` field in a prompt commit is a flexible JSON object using LangChain's serialization format ("lc-json"). For a `StructuredPrompt`:

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
- [ ] Run experiments to capture actual manifest formats
- [ ] Document edge cases

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

## 6. Open Questions

1. **Schema format**: Should we accept Pydantic models (requires Python), JSON Schema files, or both?
2. **Transform parity**: How important is it to match Python SDK transforms exactly?
3. **Model binding**: Do we need `include_model` support, or just raw prompt storage?
4. **Validation**: Should we validate schemas client-side before pushing?

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

## 8. Next Steps

1. Run experiment scripts to capture actual manifest formats
2. Update this report with findings
3. Create parent epic for structured output prompt implementation
4. Break down into sub-issues per phase
