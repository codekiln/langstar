# langsmith-sdk

## Repository Information

- **Repository**: [langchain-ai/langsmith-sdk](https://github.com/langchain-ai/langsmith-sdk)
- **Date Created**: 2025-11-19
- **Cloned to**: `/workspace/reference/repo/langchain-ai/langsmith-sdk/code`

## Purpose

Study the official LangSmith Python SDK to understand:
- How to implement LangSmith API clients
- Proper patterns for prompts, datasets, and tracing
- API parameter structures and serialization
- Reference implementation for Langstar Rust SDK

## Structured Output Prompts (Issue #398 Research)

### Overview

Structured output prompts allow defining a JSON schema that the LLM output must conform to. The langsmith-sdk handles this through:

1. `StructuredPrompt` class from `langchain_core.prompts.structured`
2. `with_structured_output()` method on chat models
3. Special transform logic in `pull_prompt` and `create_commit`

### Key Classes

**`langchain_core.prompts.structured.StructuredPrompt`**
- Core class for defining structured output constraints
- Imported in `client.py:7725`

**`ls_structured_output_format`**
- Keyword argument used to pass structured output format between transforms
- Found in `client.py:8779-8782`

### Transform Logic

#### Pull Transform (client.py:7776-7794)

When pulling a prompt with `include_model=True`, transforms 2-step sequence to 3-step.

**Purpose**: Adds output parser step for proper structured output handling.

#### Push Transform (client.py:8761-8794)

When pushing a prompt, transforms 3-step back to 2-step.

**Purpose**: Normalizes the chain for storage, extracting structured output config into the prompt.

### Manifest Structure

The `manifest` field in prompt commits is a flexible JSON object. For structured output prompts, expected structure:

```json
{
  "lc": 1,
  "type": "constructor",
  "id": ["langchain", "prompts", "structured", "StructuredPrompt"],
  "kwargs": {
    "messages": [...],
    "schema_": {...},
    "method": "json_schema"
  }
}
```

### What's NOT in Langstar Yet

- No `StructuredPrompt` type in Rust SDK
- No structured output handling in CLI `prompt push`
- No schema validation
- No transform logic equivalent to Python SDK

### Open Questions

1. Exact JSON Schema format in the manifest?
2. How is `method` (json_schema vs function_calling) determined?
3. CLI UX for defining structured output schema?

See: `reference/experiments/398-structured-output-prompts/` for experiments.
