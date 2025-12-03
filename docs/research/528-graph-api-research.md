# Research: Agent Server API for Graph Listing

**Issue:** #528
**Parent:** #527 (ls-graph-deployments-separation milestone)
**Date:** 2025-12-03

## Executive Summary

LangGraph graphs are **not a first-class API resource**. There is no `/graphs` endpoint. Instead, graphs are discovered indirectly through assistants, where each assistant has a `graph_id` field identifying the graph it's based on.

## Key Finding: Graph Discovery via Assistants

### How Graphs Work in LangGraph Cloud

1. **Graphs are defined at deployment time** in `langgraph.json` config file
2. **An assistant is auto-created for each graph** registered in the config
3. **Each assistant has a `graph_id` field** identifying which graph it uses
4. **Multiple assistants can share the same graph** (with different configurations)

### Implications

- To list graphs, we must list assistants and extract unique `graph_id` values
- Graph structure is available via the assistant's graph endpoint
- The graph endpoint accepts either `assistant_id` OR `graph_id` as the path parameter

## API Endpoints

### APIs Investigated

| API | Has Graph Endpoints? | Notes |
|-----|---------------------|-------|
| Control Plane API | ❌ No | Only deployment management |
| LangSmith API | ❌ No | Tracing, datasets, evaluations |
| Agent Server API | ✅ Yes | Per-deployment, graph structure endpoints |

### Agent Server API Graph Endpoints

The Agent Server API is deployed per-deployment at the deployment's runtime URL.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/assistants/{id}/graph` | GET | Get graph structure (accepts assistant_id OR graph_id) |
| `/assistants/{id}/graph?xray=true` | GET | Include subgraph representation |
| `/assistants/{id}/subgraphs` | GET | List subgraphs |
| `/assistants/{id}/schemas` | GET | Get graph input/output schemas |

### Authentication

Agent Server API uses the same `x-api-key` header as LangSmith API:

```bash
curl -H "x-api-key: $LANGSMITH_API_KEY" \
  "https://<deployment-url>/assistants/{id}/graph?xray=true"
```

## Graph Structure Response

The `/assistants/{id}/graph?xray=true` endpoint returns graph topology:

```json
{
  "nodes": [
    { "id": "__start__", "type": "runnable", "data": { "name": "__start__" } },
    { "id": "Responder", "type": "runnable", "data": { "name": "Responder" } },
    { "id": "Feedback", "type": "runnable", "data": { "name": "Feedback" } },
    { "id": "__end__" }
  ],
  "edges": [
    { "source": "Responder", "target": "Feedback", "conditional": true },
    { "source": "Responder", "target": "__end__", "conditional": true },
    { "source": "__start__", "target": "Feedback", "conditional": true },
    { "source": "__start__", "target": "Responder", "conditional": true },
    { "source": "Feedback", "target": "__end__" }
  ]
}
```

**Key elements:**
- `__start__` and `__end__` are special control flow nodes (should be hidden in CLI output)
- Node `id` and `data.name` identify graph steps
- Edges show transitions, with `conditional: true` for branching logic

## Recommended CLI Implementation

### `langstar graph list <deployment-name-or-id>`

**Algorithm:**
1. Resolve deployment name to ID (if name provided instead of UUID)
2. Get deployment details to obtain runtime URL
3. Call `POST /assistants/search` on deployment's Agent Server API
4. Extract unique `graph_id` values from assistants
5. For each unique graph, call `/assistants/{graph_id}/graph?xray=true`
6. Parse node names (excluding `__start__` and `__end__`)

**Output columns:**

| Column | Source |
|--------|--------|
| Graph ID | `assistant.graph_id` (deduplicated) |
| Assistant Names | Comma-separated list of assistants using this graph |
| Assistant Count | Count of assistants using this graph |
| Nodes | Node names from `/graph?xray=true` (excluding control nodes) |

**Example output:**
```
╭──────────────┬─────────────────────┬────────────┬─────────────────────╮
│ Graph ID     │ Assistants          │ # Assists  │ Nodes               │
├──────────────┼─────────────────────┼────────────┼─────────────────────┤
│ agent        │ default, custom-v1  │ 2          │ Responder, Feedback │
│ rag_pipeline │ rag-assistant       │ 1          │ Retriever, Generate │
╰──────────────┴─────────────────────┴────────────┴─────────────────────╯
```

### `langstar graph get <graph_id> --deployment <name-or-id>`

**Algorithm:**
1. Resolve deployment name to ID (if needed)
2. Call `GET /assistants/{graph_id}/graph?xray=true`
3. Return formatted graph structure

**Output options:**
- Default: Formatted node list with edges
- `--json`: Raw JSON response
- `--xray`: Include subgraph details (default: true)

### Name Resolution Pattern

Users prefer names over UUIDs. Both commands should support deployment names:

```bash
# By name (preferred UX)
langstar graph list pr-integration-test

# By UUID (also supported)
langstar graph list 47599969-47ab-49d5-878e-cc6dbcbed059
```

**Resolution logic:**
1. Check if argument matches UUID format
2. If not UUID, call deployment list API with name filter
3. If exactly one match, use that deployment's ID
4. If multiple matches, show disambiguation error
5. If no matches, show "deployment not found" error

**Precedent:** This pattern already exists in `cli/src/commands/assistant.rs`

## SDK Implementation Approach

### Option A: Extend Assistants Client (Recommended)

Add graph-related methods to existing `AssistantsClient`:

```rust
impl AssistantsClient {
    /// Get graph structure for an assistant or graph ID
    pub async fn get_graph(&self, id: &str, xray: bool) -> Result<GraphStructure>;

    /// List unique graphs in deployment with their assistants
    pub async fn list_graphs(&self) -> Result<Vec<GraphInfo>>;
}
```

**Pros:**
- Uses existing client infrastructure
- Graph endpoints are under `/assistants/` path
- Consistent authentication handling

### Option B: New Graph Client

Create dedicated `GraphClient` for graph operations.

**Pros:**
- Cleaner separation of concerns
- Can add graph-specific logic

**Cons:**
- Duplicates deployment URL resolution
- Graph endpoints are still `/assistants/{id}/graph`

### Recommendation

**Option A** - Extend `AssistantsClient`. The graph endpoints live under `/assistants/`, so it's semantically appropriate. This avoids code duplication and leverages existing request handling.

## Data Types

### GraphStructure

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct GraphStructure {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: Option<String>,
    pub data: Option<GraphNodeData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphNodeData {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub conditional: bool,
}
```

### GraphInfo (for list output)

```rust
#[derive(Debug)]
pub struct GraphInfo {
    pub graph_id: String,
    pub assistants: Vec<String>,  // Assistant names
    pub assistant_count: usize,
    pub nodes: Vec<String>,       // Node names (excluding __start__/__end__)
}
```

## Files to Modify

### SDK (`sdk/`)

- `sdk/src/assistants.rs` - Add `get_graph()` and `list_graphs()` methods
- `sdk/src/types/graph.rs` (new) - Graph structure types

### CLI (`cli/`)

- `cli/src/commands/graph.rs` - Currently has deployment commands (rename to `deployment.rs`)
- `cli/src/commands/graph.rs` (new) - Graph list/get commands
- `cli/src/commands/mod.rs` - Update module exports

## References

- Control Plane API: `reference/openapi/langchain/control-plane/openapi.json`
- LangSmith API: `reference/openapi/langchain/langsmith/openapi.json`
- Agent Server API: Per-deployment at `/openapi.json`
- Existing assistant commands: `cli/src/commands/assistant.rs`
- Assistant SDK client: `sdk/src/assistants.rs`

## Next Steps

1. **Issue #529** (or sibling): Rename current `graph.rs` to `deployment.rs`
2. **Issue #530** (or sibling): Implement SDK graph methods
3. **Issue #531** (or sibling): Implement CLI `graph list` and `graph get` commands
