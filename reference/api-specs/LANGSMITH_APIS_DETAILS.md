# LangSmith API Specifications Catalog

This document provides a comprehensive catalog of all LangSmith/LangGraph API specifications stored in this repository.

## Primary OpenAPI Specifications

| API | File Path | Size | Version | Committed | Base URL (US) | Base URL (EU) | Description |
|-----|-----------|------|---------|-----------|---------------|---------------|-------------|
| LangSmith API | `../openapi/langchain/langsmith/openapi.json` | 635K | 0.1.0 | 2025-11-26 | `https://api.smith.langchain.com` | `https://eu.api.smith.langchain.com` | Core LangSmith API - datasets, examples, runs, projects, tracing, org management |
| LangSmith Deployment Control Plane API | `../openapi/langchain/control-plane/openapi.json` | 70K | 0.1.0 | 2025-11-20 | `https://api.host.langchain.com` | `https://eu.api.host.langchain.com` | Deployment management for LangGraph Server |
| LangSmith Deployment Agent Server API | `../openapi/langchain/agent-server/openapi.json` | 95K | 0.1.0 | 2025-12-05 | Per-deployment | Per-deployment | Runtime API for assistants, threads, runs, graphs (OpenAPI at `/openapi.json` on each deployment) |
| SCIM API | SCIM 2.0 compliant | N/A | SCIM 2.0 | N/A | `https://api.smith.langchain.com/scim/v2` | `https://eu.api.smith.langchain.com/scim/v2` | User provisioning (Enterprise only) |

## Extracted Schema Files

These are subsets extracted from the main OpenAPI specs for focused reference. See respective `FRAGMENTS.md` files for jq extraction commands.

### LangSmith API Fragments

See [langsmith/FRAGMENTS.md](./langsmith/FRAGMENTS.md) for extraction commands.

| Schema | File Path | Size | Purpose |
|--------|-----------|------|---------|
| Annotation Queue Endpoints | `langsmith/annotation-queue-endpoints.json` | 2.3K | HITL annotation queue API endpoints |
| Annotation Queue Schemas | `langsmith/annotation-queue-schemas.json` | 45K | Data types for annotation queues |
| Run Schema | `langsmith/run-schema.json` | 9.8K | Run/trace data structures |
| Runs Query Endpoint | `langsmith/runs-query-endpoint.json` | 1.0K | POST /runs/query endpoint spec |
| Runs Query Request | `langsmith/runs-query-request-schema.json` | 5.5K | Request payload for runs query |
| Runs Query Response | `langsmith/runs-query-response-schema.json` | 1.1K | Response format for runs query |

### Agent Server API Fragments

See [agent-server/FRAGMENTS.md](./agent-server/FRAGMENTS.md) for extraction commands.

| Schema | File Path | Size | Purpose |
|--------|-----------|------|---------|
| Graph Endpoint | `agent-server/graph-endpoint.json` | 6K | GET /assistants/{id}/graph |
| Subgraphs Endpoints | `agent-server/subgraphs-endpoints.json` | 3.5K | GET /assistants/{id}/subgraphs |
| Assistants Search Endpoint | `agent-server/assistants-search-endpoint.json` | 1.5K | POST /assistants/search |
| Schemas Endpoint | `agent-server/schemas-endpoint.json` | 1.5K | GET /assistants/{id}/schemas |
| Graph Schemas | `agent-server/graph-schemas.json` | 3K | GraphSchema, Subgraphs types |
| Assistant Schema | `agent-server/assistant-schema.json` | 2K | Assistant type with graph_id field |
| Assistant Search Request | `agent-server/assistant-search-request-schema.json` | 2K | Search request payload |

## Remote Sources

| API | OpenAPI Spec URL | Interactive Docs | Notes |
|-----|------------------|------------------|-------|
| LangSmith API | `https://api.smith.langchain.com/openapi.json` | `https://api.smith.langchain.com/redoc` | Primary LangSmith API |
| LangSmith Deployment Control Plane API | `https://api.host.langchain.com/openapi.json` | `https://api.host.langchain.com/docs` | LangGraph deployment management API |
| LangSmith Deployment Agent Server API | `/docs` on each deployment | `/docs` on each deployment | Per-deployment runtime API |
| SCIM API | SCIM 2.0 compliant (RFC 7644) | N/A | Enterprise user provisioning |

## Additional Endpoints

These endpoints are part of the LangSmith API but serve specialized purposes:

| Endpoint Type | Base URL (US) | Base URL (EU) | Purpose |
|---------------|---------------|---------------|---------|
| OpenTelemetry Traces | `https://api.smith.langchain.com/otel/v1/traces` | `https://eu.api.smith.langchain.com/otel/v1/traces` | Ingest traces using OTLP |
| OpenTelemetry Logs | `https://api.smith.langchain.com/otel/v1/logs` | `https://eu.api.smith.langchain.com/otel/v1/logs` | Ingest logs using OTLP |
| OpenTelemetry Claude Code | `https://api.smith.langchain.com/otel/v1/claude_code` | `https://eu.api.smith.langchain.com/otel/v1/claude_code` | Claude Code telemetry |
| Public JSON Schemas - Messages | `https://api.smith.langchain.com/public/schemas/v1/message.json` | `https://eu.api.smith.langchain.com/public/schemas/v1/message.json` | OpenAI-compatible message schema |
| Public JSON Schemas - Tools | `https://api.smith.langchain.com/public/schemas/v1/tooldef.json` | `https://eu.api.smith.langchain.com/public/schemas/v1/tooldef.json` | Tool definition schema |

## Usage

### Verifying Endpoints with jq

All jq queries in research documents can be run against local spec files:

```bash
# List all dataset-related endpoints
jq '.paths | keys | map(select(contains("dataset")))' ../openapi/langchain/langsmith/openapi.json

# Get endpoint summary
jq '.paths["/api/v1/datasets"].get.summary' ../openapi/langchain/langsmith/openapi.json

# List all schema definitions
jq '.components.schemas | keys' ../openapi/langchain/langsmith/openapi.json
```

### Updating Specifications

To refresh local specs from remote sources:

```bash
# Update LangSmith API spec
curl -o ../openapi/langchain/langsmith/openapi.json https://api.smith.langchain.com/openapi.json

# Update LangGraph Cloud spec
curl -o ../openapi/langchain/control-plane/openapi.json https://api.host.langchain.com/openapi.json
```

## API Coverage by Domain

### LangSmith Core API (`../openapi/langchain/langsmith/openapi.json`)

| Domain | Endpoints | Key Operations |
|--------|-----------|----------------|
| Datasets | `/api/v1/datasets/*` | CRUD, versioning, sharing, import/export |
| Examples | `/api/v1/examples/*` | CRUD, bulk operations, validation |
| Runs | `/api/v1/runs/*` | Query, stats, feedback, sharing |
| Projects | `/api/v1/sessions/*` | Project management, stats |
| Annotation Queues | `/api/v1/annotation-queues/*` | HITL workflow management |
| Feedback | `/api/v1/feedback/*` | Run feedback, scores |

### LangSmith Deployment Control Plane API (`../openapi/langchain/control-plane/openapi.json`)

| Domain | Endpoints | Key Operations |
|--------|-----------|----------------|
| Deployments | `/v2/deployments/*` | Create, update, delete LangGraph deployments |
| Revisions | `/v2/deployments/{id}/revisions/*` | Deployment versioning, rollback |
| Integrations | `/v2/integrations/*` | GitHub, Docker registry configuration |
| Monitoring | Various | Health status, metrics, logs |

### LangSmith Deployment Agent Server API (Runtime)

| Domain | Endpoints | Key Operations |
|--------|-----------|----------------|
| Assistants | `/assistants/*` | Create, list, get, update, delete, search assistant configurations |
| Threads | `/threads/*` | Manage conversation threads and state |
| Runs | `/threads/{thread_id}/runs/*` | Create, stream, wait, list, get execution runs |
| Store | `/threads/{thread_id}/store/*` | Key-value storage within threads |
| Cron Jobs | `/crons/*` | Schedule recurring assistant executions |
| Webhooks | `/webhooks/*` | Event-driven notifications |

### SCIM API (Enterprise)

| Domain | Endpoints | Key Operations |
|--------|-----------|----------------|
| Users | `/Users/*` | Create, read, update, delete user provisioning |
| Groups | `/Groups/*` | Create, read, update, delete group management |
| Token Management | `/v1/platform/orgs/current/scim/tokens/*` | Generate, list, revoke SCIM tokens |

## Authentication

| API | Authentication Method | Headers Required |
|-----|----------------------|------------------|
| LangSmith API | API Key | `X-Api-Key` |
| LangSmith Deployment Control Plane API | API Key + Tenant ID | `X-Api-Key`, `X-Tenant-Id` |
| LangSmith Deployment Agent Server API | API Key | `X-Api-Key` |
| SCIM API | Bearer Token | `Authorization: Bearer <token>` |
| OpenTelemetry Endpoints | API Key | `X-Api-Key` |

## Regional Support

All APIs support:
- **US Region**: Primary endpoints (see Base URL columns above)
- **EU Region**: EU-specific endpoints (see Base URL columns above)
- **Self-Hosted**: Custom host configuration (varies by deployment)

## See Also

- [LANGSMITH_API_OVERVIEW.md](./LANGSMITH_API_OVERVIEW.md) - Quick reference for the four core APIs
- [langchain-apis.md](../../docs/langchain-apis.md) - Comprehensive overview of all LangChain HTTP/REST APIs
- [346-dataset-api-research.md](../docs/research/346-dataset-api-research.md) - Dataset API research with jq citations
- [298-openapi-validation.md](../research/298-openapi-validation.md) - OpenAPI validation report
- [334-openapi-validation.md](../research/334-openapi-validation.md) - Annotation queues validation

