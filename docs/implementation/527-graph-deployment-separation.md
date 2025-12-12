# Implementation: Graph/Deployment Command Separation

**Issue:** [#527](https://github.com/codekiln/langstar/issues/527) (ls-graph-deployments-separation milestone)
**Milestone:** [ls-graph-deployments-separation](https://github.com/codekiln/langstar/milestone/11)
**Date:** 2025-12-06
**Status:** ✅ Completed

## Executive Summary

Semantic separation of `langstar graph` and `langstar deployment` commands to correctly reflect the underlying LangGraph Cloud API structure.

### What Was Built

| Component                 | Description                                                          |
| ------------------------- | -------------------------------------------------------------------- |
| **`langstar deployment`** | CRUD commands for LangGraph Cloud deployments (Control Plane API)    |
| **`langstar graph`**      | Inspection commands for graphs within deployments (Agent Server API) |
| **SDK Types**             | `Graph`, `GraphNode`, `GraphEdge`, `GraphSummary` structs            |
| **SDK Client**            | `GraphClient` with `list()`, `get()`, `subgraphs()` methods          |
| **User Docs**             | `docs/deployments.md`, `docs/graphs.md`                              |

### Key Deliverables

| Phase        | Issue                                                   | Status | Description                      |
| ------------ | ------------------------------------------------------- | ------ | -------------------------------- |
| 1 Research   | [#528](https://github.com/codekiln/langstar/issues/528) | ✅     | Agent Server API research        |
| 2 Design     | [#562](https://github.com/codekiln/langstar/issues/562) | ✅     | DX consistency design            |
| 3 OpenAPI    | [#564](https://github.com/codekiln/langstar/issues/564) | ✅     | Agent Server API spec validation |
| 4 SDK Types  | [#566](https://github.com/codekiln/langstar/issues/566) | ✅     | Graph structure types            |
| 5 SDK Client | [#567](https://github.com/codekiln/langstar/issues/567) | ✅     | GraphClient implementation       |
| 6 CLI Graph  | [#569](https://github.com/codekiln/langstar/issues/569) | ✅     | `graph list/get` commands        |
| 7 Testing    | [#571](https://github.com/codekiln/langstar/issues/571) | ✅     | Integration tests                |
| 8 Docs       | [#572](https://github.com/codekiln/langstar/issues/572) | ✅     | User and implementation docs     |

### Problem Solved

Previously, `langstar graph list` listed **deployments**, which was semantically confusing. Users expected `langstar graph list` to list graphs, not deployments.

### Solution

- **`langstar deployment`** - Manages LangGraph Cloud deployments via Control Plane API
- **`langstar graph`** - Inspects LangGraph graphs within deployments via Agent Server API

---

## Architecture

### API Mapping

| CLI Command         | API           | Endpoint                           | Description                             |
| ------------------- | ------------- | ---------------------------------- | --------------------------------------- |
| `deployment list`   | Control Plane | `GET /v2/deployments`              | List all deployments                    |
| `deployment get`    | Control Plane | `GET /v2/deployments/{id}`         | Get deployment details                  |
| `deployment create` | Control Plane | `POST /v2/deployments`             | Create new deployment                   |
| `deployment delete` | Control Plane | `DELETE /v2/deployments/{id}`      | Delete deployment                       |
| `graph list`        | Agent Server  | `POST /assistants/search`          | Derive unique graph_ids from assistants |
| `graph get`         | Agent Server  | `GET /assistants/{graph_id}/graph` | Get graph structure                     |

### Key Finding: Graphs are Derived Entities

Graphs are **not first-class API resources**. There is no `/graphs` endpoint. Instead:

1. Graphs are defined in `langgraph.json` at deployment time
2. Each assistant has a `graph_id` field linking it to its underlying graph
3. Multiple assistants can share the same graph (with different configurations)
4. Graph structure is accessible via `/assistants/{id}/graph?xray=true`

---

## Implementation Details

### SDK Types (`sdk/src/graph.rs`)

```rust
/// Graph structure returned by `/assistants/{id}/graph`
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Summary of a graph derived from assistants
pub struct GraphSummary {
    pub graph_id: String,
    pub assistant_names: Vec<String>,
    pub assistant_count: usize,
    pub node_names: Vec<String>,
}
```

### SDK Client (`sdk/src/graph_client.rs`)

```rust
impl GraphClient {
    /// List all unique graphs in a deployment
    pub async fn list(&self, include_structure: bool) -> Result<Vec<GraphSummary>>;

    /// Get graph structure by graph_id
    pub async fn get(&self, graph_id: &str, xray: bool) -> Result<Graph>;

    /// Get subgraphs for a graph
    pub async fn subgraphs(&self, graph_id: &str) -> Result<Vec<Graph>>;
}
```

### CLI Commands (`cli/src/commands/`)

- `deployment.rs` - Control Plane API commands for deployment lifecycle
- `graph.rs` - Agent Server API commands for graph inspection

---

## Documentation

### User Documentation

For detailed usage, see:

- **[Deployments Guide](../deployments.md)** - Complete `langstar deployment` command reference
- **[Graphs Guide](../graphs.md)** - Complete `langstar graph` command reference

### Research & Design Artifacts

| Document                                       | Purpose                            |
| ---------------------------------------------- | ---------------------------------- |
| `docs/research/528-graph-api-research.md`      | Agent Server API research findings |
| `docs/research/527.2-design-dx-consistency.md` | DX and UX design decisions         |
| `docs/research/527.3-openapi-validation.md`    | OpenAPI spec validation            |

### API Specifications

| Spec              | Path                                                     |
| ----------------- | -------------------------------------------------------- |
| Control Plane API | `reference/openapi/langchain/control-plane/openapi.json` |
| Agent Server API  | `reference/openapi/langchain/agent-server/openapi.json`  |
| Fragment Index    | `reference/api-specs/agent-server/FRAGMENTS.md`          |

---

## References

- **Parent Issue:** [#527](https://github.com/codekiln/langstar/issues/527)
- **Milestone:** [ls-graph-deployments-separation](https://github.com/codekiln/langstar/milestone/11)
