# LangSmith API OpenAPI Specification

## Source

- **Remote URL**: `https://api.smith.langchain.com/openapi.json`
- **Interactive Docs**: `https://api.smith.langchain.com/redoc`
- **Local File**: `openapi.json`

## API Overview

The core LangSmith API provides endpoints for:
- Tracing and observability
- Datasets and examples management
- Runs and feedback
- Projects (sessions)
- Annotation queues (HITL workflows)
- Organization management

## Base URLs

| Region | Base URL |
|--------|----------|
| US | `https://api.smith.langchain.com` |
| EU | `https://eu.api.smith.langchain.com` |

## Authentication

- **Method**: API Key
- **Header**: `X-Api-Key`

## Provenance

| Date | Action | Size | Notes |
|------|--------|------|-------|
| 2025-11-26 | Initial fetch | 635K | v0.1.0 |

## Refresh Command

```bash
curl -o openapi.json https://api.smith.langchain.com/openapi.json
```

## Related Files

- **Extracted fragments**: `../../api-specs/langsmith/`
- **API overview**: `../../api-specs/LANGSMITH_API_OVERVIEW.md`
- **Detailed catalog**: `../../api-specs/LANGSMITH_APIS_DETAILS.md`

## Notes

This file is large (~635K) and is tracked in version control. Consider gitignoring if size becomes problematic. Fragments in `api-specs/langsmith/` are preferred for AI context grounding.
