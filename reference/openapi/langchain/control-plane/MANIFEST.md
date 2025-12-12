# LangSmith Deployment Control Plane API OpenAPI Specification

## Source

- **Remote URL**: `https://api.host.langchain.com/openapi.json`
- **Interactive Docs**: `https://api.host.langchain.com/docs`
- **Local File**: `openapi.json`

## API Overview

The Deployment Control Plane API manages LangGraph Server deployments:

- Create, update, delete deployments
- Manage deployment revisions and rollbacks
- Configure integrations (GitHub, Docker registries)
- Monitor deployment health and metrics

## Base URLs

| Region | Base URL                            |
| ------ | ----------------------------------- |
| US     | `https://api.host.langchain.com`    |
| EU     | `https://eu.api.host.langchain.com` |

## Authentication

- **Method**: API Key + Tenant ID
- **Headers**: `X-Api-Key`, `X-Tenant-Id`

## Provenance

| Date       | Action        | Size | Notes        |
| ---------- | ------------- | ---- | ------------ |
| 2025-11-20 | Initial fetch | 70K  | v0.1.0       |
| 2025-12-11 | Refresh       | 71K  | Updated spec |

## Refresh Command

```bash
curl -o openapi.json https://api.host.langchain.com/openapi.json
```

## Related Files

- **Extracted fragments**: `../../api-specs/control-plane/` (4 files, see FRAGMENTS.md)
- **Schema documentation**: `../../api-specs/control-plane/FRAGMENTS.md` - Deployment schema details
- **API overview**: `../../api-specs/LANGSMITH_API_OVERVIEW.md`
- **Detailed catalog**: `../../api-specs/LANGSMITH_APIS_DETAILS.md`

## Notes

This file is moderate size (~70K) and is tracked in version control.

### Key Schema Insights (2025-12-11)

**Deployment Schema**: The `deployment_type` field is **not** a top-level field in deployment responses. It's nested inside the `source_config` JSON object, which the SDK stores as unparsed `Option<serde_json::Value>`. This architectural detail affects CLI output formatting decisions - see `FRAGMENTS.md` for full explanation.
