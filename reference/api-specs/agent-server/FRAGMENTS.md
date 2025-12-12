# Agent Server API Extracted Fragments

## Source

- **Full Spec**: `../../openapi/langchain/agent-server/openapi.json`
- **Deployment Pattern**: `https://<deployment-url>/openapi.json`
- **Note**: Unlike centralized APIs, this spec is per-deployment

## Purpose

These fragments are extracted subsets of the full OpenAPI spec, optimized for:

- AI context window efficiency (small file sizes)
- Focused reference for graph-related features
- Quick lookup without loading full spec

## Fragments

| File                                   | Size  | Purpose                                  | jq Query  | Last Updated |
| -------------------------------------- | ----- | ---------------------------------------- | --------- | ------------ |
| `graph-endpoint.json`                  | 1KB   | GET /assistants/{id}/graph endpoint      | See below | 2025-12-05   |
| `subgraphs-endpoints.json`             | 3.5KB | GET /assistants/{id}/subgraphs endpoints | See below | 2025-12-05   |
| `assistants-search-endpoint.json`      | 1.5KB | POST /assistants/search endpoint         | See below | 2025-12-05   |
| `schemas-endpoint.json`                | 1.5KB | GET /assistants/{id}/schemas endpoint    | See below | 2025-12-05   |
| `graph-schemas.json`                   | 3KB   | GraphSchema, Subgraphs types             | See below | 2025-12-05   |
| `assistant-schema.json`                | 2KB   | Assistant type (has graph_id)            | See below | 2025-12-05   |
| `assistant-search-request-schema.json` | 2KB   | AssistantSearchRequest type              | See below | 2025-12-05   |

## Extraction Commands

Run these from the repository root (with agent-server openapi.json in place):

```bash
# Graph topology endpoint (only /graph, not /subgraphs)
jq '.paths | with_entries(select(.key | test("/assistants.*/graph$")))' \
  reference/openapi/langchain/agent-server/openapi.json \
  > reference/api-specs/agent-server/graph-endpoint.json

# Subgraphs endpoints
jq '.paths | with_entries(select(.key | test("/assistants.*subgraphs")))' \
  reference/openapi/langchain/agent-server/openapi.json \
  > reference/api-specs/agent-server/subgraphs-endpoints.json

# Assistants search endpoint (for listing graphs via assistant graph_id)
jq '.paths | with_entries(select(.key == "/assistants/search"))' \
  reference/openapi/langchain/agent-server/openapi.json \
  > reference/api-specs/agent-server/assistants-search-endpoint.json

# Schemas endpoint
jq '.paths | with_entries(select(.key | test("/assistants.*schemas")))' \
  reference/openapi/langchain/agent-server/openapi.json \
  > reference/api-specs/agent-server/schemas-endpoint.json

# Graph-related schemas
jq '.components.schemas | with_entries(select(.key | test("Graph|Subgraph"; "i")))' \
  reference/openapi/langchain/agent-server/openapi.json \
  > reference/api-specs/agent-server/graph-schemas.json

# Assistant schema (contains graph_id field)
jq '.components.schemas.Assistant' \
  reference/openapi/langchain/agent-server/openapi.json \
  > reference/api-specs/agent-server/assistant-schema.json

# AssistantSearchRequest schema
jq '.components.schemas.AssistantSearchRequest' \
  reference/openapi/langchain/agent-server/openapi.json \
  > reference/api-specs/agent-server/assistant-search-request-schema.json
```

## Key Endpoints for Graph Operations

### List Graphs (via Assistants)

**Endpoint**: `POST /assistants/search`

No direct `/graphs` endpoint exists. Graphs are discovered by:

1. Searching assistants: `POST /assistants/search`
2. Extracting unique `graph_id` values from assistants

```bash
curl -H "x-api-key: $LANGSMITH_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{}' \
  "https://<deployment>/assistants/search" | jq '[.[].graph_id] | unique'
```

### Get Graph Topology

**Endpoint**: `GET /assistants/{id}/graph?xray={bool|int}`

Returns nodes and edges. Accepts assistant_id OR graph_id as path parameter.

```bash
curl -H "x-api-key: $LANGSMITH_API_KEY" \
  "https://<deployment>/assistants/{graph_id}/graph?xray=true"
```

**Response structure** (not fully documented in OpenAPI):

```json
{
  "nodes": [
    { "id": "node_name", "type": "runnable", "data": { "name": "node_name" } }
  ],
  "edges": [
    { "source": "from_node", "target": "to_node", "conditional": false }
  ]
}
```

### Get Graph Schemas

**Endpoint**: `GET /assistants/{id}/schemas`

Returns input/output/state/config schemas for the graph.

### Get Subgraphs

**Endpoint**: `GET /assistants/{id}/subgraphs?recurse={bool}`

Returns map of subgraph namespace to schema metadata.

## Notes on OpenAPI Spec Quality

The graph topology endpoint (`/assistants/{id}/graph`) has a loose response schema:

```json
{
  "additionalProperties": { "items": { "type": "object" }, "type": "array" },
  "type": "object"
}
```

This doesn't document the actual `nodes[]` and `edges[]` structure. The actual response structure is validated empirically (see validation report).

## Related

- **Full spec**: `../../openapi/langchain/agent-server/openapi.json`
- **Spec manifest**: `../../openapi/langchain/agent-server/MANIFEST.md`
- **Research**: `../../../docs/research/528-graph-api-research.md`
- **Validation**: `../../../docs/research/527.3-openapi-validation.md`
