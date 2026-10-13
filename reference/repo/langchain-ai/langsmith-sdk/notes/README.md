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

## Playground Settings (Issue #471 Research)

### Overview

Research for implementing the `/api/v1/playground-settings` API in Langstar SDK.

**Key findings:**
- Python SDK has NO implementation for playground settings
- Langstar will be first SDK to implement this API
- Should follow existing Langstar patterns from datasets/prompts
- Settings field uses LangChain serialization format (similar to prompts)

### Patterns

See: `playground-settings-patterns.md` for complete SDK implementation patterns including:
- Method signatures for CRUD operations
- Type definitions
- Pagination strategy
- Error handling approach
- Comparison with Python SDK patterns

## How the Python and JS clients retry a 429

Research for [langstar #801: name the upstream LangChain projects to check for compatibility](https://github.com/codekiln/langstar/issues/801).

Read at commit [47c553c](https://github.com/langchain-ai/langsmith-sdk/commit/47c553caf08c1989e2be3ff1886da3eb801f709f):

- Layer 1 is the adapter. `Client` mounts `_LangSmithHttpAdapter` ([python/langsmith/client.py:898](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L898), mounted at [:1469](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L1469)) with the `Retry` that `_default_retry_config` ([:672](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L672)) builds: `total=3, backoff_factor=0.5, status_forcelist=[429, 500, 502, 503, 504, 408, 425], respect_retry_after_header=True, raise_on_status=False, raise_on_redirect=False`, plus `allowed_methods=None` (every method) when urllib3 is 1.26 or newer. It retries a 429 even at the default `stop_after_attempt=1`, honoring `Retry-After`.
- Layer 2 is `request_with_retries` ([:1979](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L1979)). It sees a 429 only after the adapter gives up, and retries only when the caller passes `stop_after_attempt` above its default of 1. It then reads the `retry-after` header, defaults to 30 seconds when it is missing or invalid, and sleeps `retry_after * 2**idx` plus up to one second of jitter, so the wait doubles on each further attempt.
- JS: `handle429` in [js/src/client.ts](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/js/src/client.ts#L813) waits `retry-after` seconds, defaulting to 10 when the header is missing, then retries. It is installed only as `onFailedResponseHook` of `batchIngestCaller` (`js/src/client.ts` line 1488), the caller that sends batched trace uploads; the general caller has no such hook.
