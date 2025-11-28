# LangSmith API Specifications Catalog

This document provides a table of contents for all LangSmith/LangGraph API specifications stored in this repository.

## Primary OpenAPI Specifications

| API | File Path | Size | Version | Committed | Description |
|-----|-----------|------|---------|-----------|-------------|
| LangSmith API | `api-specs/langsmith-openapi.json` | 635K | 0.1.0 | 2025-11-26 | Core LangSmith API - datasets, examples, runs, projects, tracing |
| LangSmith Deployment Control Plane | `openapi/langchain/langsmith-deployment-control-plane-api-openapi.json` | 70K | 0.1.0 | 2025-11-20 | Deployment management for LangGraph Cloud |

## Extracted Schema Files

These are subsets extracted from the main OpenAPI specs for focused reference:

| Schema | File Path | Size | Purpose |
|--------|-----------|------|---------|
| Annotation Queue Endpoints | `api-specs/annotation-queue-endpoints.json` | 2.3K | HITL annotation queue API endpoints |
| Annotation Queue Schemas | `api-specs/annotation-queue-schemas.json` | 45K | Data types for annotation queues |
| Run Schema | `api-specs/run-schema.json` | 9.8K | Run/trace data structures |
| Runs Query Endpoint | `api-specs/runs-query-endpoint.json` | 1.0K | POST /runs/query endpoint spec |
| Runs Query Request | `api-specs/runs-query-request-schema.json` | 5.5K | Request payload for runs query |
| Runs Query Response | `api-specs/runs-query-response-schema.json` | 1.1K | Response format for runs query |

## Remote Sources

| API | URL | Notes |
|-----|-----|-------|
| LangSmith API | `https://api.smith.langchain.com/openapi.json` | Primary LangSmith API |
| LangGraph Cloud | `https://api.host.langchain.com/openapi.json` | LangGraph deployment API |

## Usage

### Verifying Endpoints with jq

All jq queries in research documents can be run against local spec files:

```bash
# List all dataset-related endpoints
jq '.paths | keys | map(select(contains("dataset")))' reference/api-specs/langsmith-openapi.json

# Get endpoint summary
jq '.paths["/api/v1/datasets"].get.summary' reference/api-specs/langsmith-openapi.json

# List all schema definitions
jq '.components.schemas | keys' reference/api-specs/langsmith-openapi.json
```

### Updating Specifications

To refresh local specs from remote sources:

```bash
# Update LangSmith API spec
curl -o reference/api-specs/langsmith-openapi.json https://api.smith.langchain.com/openapi.json

# Update LangGraph Cloud spec
curl -o reference/openapi/langchain/langsmith-deployment-control-plane-api-openapi.json https://api.host.langchain.com/openapi.json
```

## API Coverage by Domain

### LangSmith Core API (`langsmith-openapi.json`)

| Domain | Endpoints | Key Operations |
|--------|-----------|----------------|
| Datasets | `/api/v1/datasets/*` | CRUD, versioning, sharing, import/export |
| Examples | `/api/v1/examples/*` | CRUD, bulk operations, validation |
| Runs | `/api/v1/runs/*` | Query, stats, feedback, sharing |
| Projects | `/api/v1/sessions/*` | Project management, stats |
| Annotation Queues | `/api/v1/annotation-queues/*` | HITL workflow management |
| Feedback | `/api/v1/feedback/*` | Run feedback, scores |

### Deployment Control Plane API

| Domain | Endpoints | Key Operations |
|--------|-----------|----------------|
| Deployments | `/deployments/*` | Create, update, delete LangGraph deployments |
| Revisions | `/revisions/*` | Deployment versioning |
| Crons | `/crons/*` | Scheduled task management |
| Assistants | `/assistants/*` | Assistant configuration |

## See Also

- [346-dataset-api-research.md](../docs/research/346-dataset-api-research.md) - Dataset API research with jq citations
- [298-openapi-validation.md](research/298-openapi-validation.md) - OpenAPI validation report
- [334-openapi-validation.md](research/334-openapi-validation.md) - Annotation queues validation
