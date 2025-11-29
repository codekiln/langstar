# Experiment: Structured Output Prompts in LangSmith

**Date**: 2025-11-29
**Issue**: [#398 - Research report - scout resources for structured output prompts](https://github.com/codekiln/langstar/issues/398)
**Milestone**: [ls-prompt-structured-outputs (#7)](https://github.com/codekiln/langstar/milestone/7)

## Objective

Understand how structured output prompts work in LangSmith by:

1. **Creating** a prompt with structured output schema
2. **Pushing** it to LangSmith via the Python SDK
3. **Pulling** it back and examining the manifest structure
4. **Documenting** the API request/response format

## Key Questions

1. What does the `manifest` field look like for a structured output prompt?
2. How is the JSON Schema embedded in the manifest?
3. What's the difference between `json_schema` and `function_calling` methods?
4. Can we create structured output prompts via raw API calls (without SDK transforms)?

## Prerequisites

- `LANGSMITH_API_KEY` environment variable set
- Python 3.10+ with langsmith and langchain-core installed
- Optional: `ANTHROPIC_API_KEY` for testing with Claude

## Files

| File | Purpose |
|------|---------|
| `README.md` | This file - experiment overview |
| `run_test.sh` | Shell wrapper with environment setup |
| `test_structured_prompts.py` | Python script for experiments |

## Usage

```bash
# Run the experiment
./run_test.sh

# Or run specific tests
./run_test.sh create    # Create and push a structured output prompt
./run_test.sh pull      # Pull an existing prompt and examine manifest
./run_test.sh raw       # Test raw API call (no SDK transforms)
```

## Findings

### Manifest Structure

*To be filled in after running experiments*

### API Request/Response

*To be filled in after running experiments*

### Transform Logic Observations

*To be filled in after running experiments*

## References

- `reference/repo/langchain-ai/langsmith-sdk/notes/README.md` - SDK analysis
- `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:7776-7794` - Pull transform
- `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:8761-8794` - Push transform
- `reference/openapi/langchain/langsmith/openapi.json` - API spec
