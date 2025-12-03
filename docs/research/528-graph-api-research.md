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
| Count | Count of assistants using this graph |
| Nodes | Node names from `/graph?xray=true` (excluding control nodes) |

**Example output:**
```
╭──────────────┬─────────────────────┬───────┬─────────────────────╮
│ Graph ID     │ Assistants          │ Count │ Nodes               │
├──────────────┼─────────────────────┼───────┼─────────────────────┤
│ agent        │ default, custom-v1  │ 2     │ Responder, Feedback │
│ rag_pipeline │ rag-assistant       │ 1     │ Retriever, Generate │
╰──────────────┴─────────────────────┴───────┴─────────────────────╯
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

## Implementation Roadmap

This research completes **Phase 1** of the `ls-graph-deployments-separation` milestone (#527). The following analysis maps remaining work to the standard feature development process documented in `docs/dev/feature-development-process.md`.

### Phase Status

| Phase | Name | Status | Notes |
|-------|------|--------|-------|
| 0.0 | Pre-Epic Scout | N/A | Would use `/gh-milestones:scout graph-commands` for new features |
| 0 | Epic Setup | ✅ Complete | Parent #527, milestone created |
| 1 | Research | ✅ Complete | This report (#528) |
| 2 | Design | 🔲 Needed | DX consistency, configuration |
| 3 | OpenAPI Validation | ⚠️ Special | Agent Server API is per-deployment |
| 4 | SDK Types | 🔲 Needed | Graph structure types |
| 5 | SDK Client | 🔲 Needed | Extend AssistantsClient |
| 6 | CLI Commands | 🔲 Needed | `graph list`, `graph get` |
| 7 | Testing | 🔲 Needed | Unit + integration tests |
| 8 | Documentation | 🔲 Needed | README, usage docs |
| 9 | Milestone Release | 🔲 Final | `/gh-milestones:release` |

### Recommended Sub-Issues

Following the `{parent}.{phase}-{slug}` naming convention:

#### 527.2-design: Design DX consistency for graph commands

**Scope:**
- Analyze existing `langstar assistant` commands for consistency patterns
- Define flag conventions (`--deployment`, `--xray`, `--format`)
- Document configuration integration (env vars, precedence)
- Decide on deployment name resolution pattern (reuse from assistant.rs)

**Deliverable:** Design section added to this research report

#### 527.3-refactor: Rename graph.rs to deployment.rs

**Scope:**
- Rename `cli/src/commands/graph.rs` → `cli/src/commands/deployment.rs`
- Change CLI command from `langstar graph` → `langstar deployment`
- Update `cli/src/commands/mod.rs` exports
- Update all documentation to reference new command names

**Rationale:** Current `langstar graph list` actually lists deployments. This is a breaking change that establishes correct semantics:
- `langstar deployment list` - list deployments (Control Plane API)
- `langstar graph list <deployment>` - list graphs within a deployment (Agent Server API)

#### 527.4-sdk-types: Implement graph structure types in SDK

**Scope:**
- Create `sdk/src/types/graph.rs` with `GraphStructure`, `GraphNode`, `GraphEdge`
- Add `GraphInfo` for list aggregation
- Register in `sdk/src/lib.rs`

**Reference:** Data types section in this report

#### 527.5-sdk-client: Add graph methods to AssistantsClient

**Scope:**
- `get_graph(id: &str, xray: bool) -> Result<GraphStructure>`
- `list_graphs() -> Result<Vec<GraphInfo>>` (aggregates from assistants)
- Handle per-deployment API URL resolution

**Decision:** Extend `AssistantsClient` rather than creating new client (endpoints are `/assistants/{id}/graph`)

#### 527.6-cli-graph: Implement graph list and get CLI commands

**Scope:**
- `langstar graph list <deployment-name-or-id>`
- `langstar graph get <graph_id> --deployment <name-or-id>`
- Support deployment name resolution (reuse pattern from assistant.rs)
- Output formats: table (default), json

**Example output:**
```
╭──────────────┬─────────────────────┬───────┬─────────────────────╮
│ Graph ID     │ Assistants          │ Count │ Nodes               │
├──────────────┼─────────────────────┼───────┼─────────────────────┤
│ agent        │ default, custom-v1  │ 2     │ Responder, Feedback │
╰──────────────┴─────────────────────┴───────┴─────────────────────╯
```

#### 527.7-testing: Add tests for graph commands

**Scope:**
- Unit tests with httpmock for SDK methods
- CLI integration tests (requires test deployment)
- Test deployment name resolution edge cases

#### 527.8-docs: Documentation for graph commands

**Scope:**
- Update CLI README with graph command examples
- Add to usage documentation
- Document relationship between graphs, assistants, and deployments

### Special Considerations

#### OpenAPI Validation (Phase 3)

Unlike LangSmith API features, the Agent Server API spec is **per-deployment** at `<deployment-url>/openapi.json`. This means:

1. Cannot validate against a static reference spec
2. Schema may vary between deployments/versions
3. Recommend: Fetch spec from test deployment to `reference/openapi/langchain/agent-server/`

```bash
# Example: Fetch from test deployment
curl -H "x-api-key: $LANGSMITH_API_KEY" \
  "https://<test-deployment-url>/openapi.json" \
  -o reference/openapi/langchain/agent-server/openapi.json
```

### Starting Fresh: Scout Command

For future similar features, use the scout command before creating a milestone:

```bash
/gh-milestones:scout graph-commands
```

This creates a Phase 0.0 scout issue to validate feasibility before committing to the full 8-phase process. For this milestone, research was done directly as sub-issue #528 since the parent epic already existed.
