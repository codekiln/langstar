# Implementation: Graph/Deployment Command Separation

**Issue:** #527 (ls-graph-deployments-separation milestone)
**Date:** 2025-12-06

## Executive Summary

This document summarizes the semantic separation of `langstar graph` and `langstar deployment` commands, completed as part of milestone #527.

### Problem

Previously, `langstar graph list` listed LangGraph Cloud **deployments**, which was semantically confusing. Users expected `langstar graph list` to list graphs, not deployments.

### Solution

- **`langstar deployment`** - Manages LangGraph Cloud deployments via Control Plane API
- **`langstar graph`** - Inspects LangGraph graphs within deployments via Agent Server API

---

## Architecture

### API Mapping

| CLI Command | API | Endpoint | Description |
|-------------|-----|----------|-------------|
| `deployment list` | Control Plane | `GET /v2/deployments` | List all deployments |
| `deployment get` | Control Plane | `GET /v2/deployments/{id}` | Get deployment details |
| `deployment create` | Control Plane | `POST /v2/deployments` | Create new deployment |
| `deployment delete` | Control Plane | `DELETE /v2/deployments/{id}` | Delete deployment |
| `graph list` | Agent Server | `POST /assistants/search` | Derive unique graph_ids from assistants |
| `graph get` | Agent Server | `GET /assistants/{graph_id}/graph` | Get graph structure |

### Key Finding: Graphs are Derived Entities

Graphs are **not first-class API resources**. There is no `/graphs` endpoint. Instead:

1. Graphs are defined in `langgraph.json` at deployment time
2. Each assistant has a `graph_id` field linking it to its underlying graph
3. Multiple assistants can share the same graph (with different configurations)
4. Graph structure is accessible via `/assistants/{id}/graph?xray=true`

---

## Command Reference

### `langstar deployment` (Control Plane API)

```bash
# List all deployments
langstar deployment list [--limit N] [--offset N] [--deployment-type TYPE] [--status STATUS]

# Get deployment details
langstar deployment get <deployment_id>

# Create new deployment
langstar deployment create --name NAME --source github --repo-url URL --branch BRANCH [--wait]

# Delete deployment
langstar deployment delete <deployment_id> [--yes]
```

### `langstar graph` (Agent Server API)

```bash
# List graphs in a deployment (requires deployment name or ID)
langstar graph list <deployment> [--show-nodes]

# Get graph structure
langstar graph get <graph_id> --deployment <deployment> [--xray]
```

---

## Migration Guide

```
OLD COMMAND              →  NEW COMMAND
────────────────────────────────────────────────────────────
langstar graph list      →  langstar deployment list
langstar graph get       →  langstar deployment get
langstar graph create    →  langstar deployment create
langstar graph delete    →  langstar deployment delete

NEW COMMANDS (no previous equivalent)
────────────────────────────────────────────────────────────
langstar graph list <deployment>      # List graphs in deployment
langstar graph get <id> --deployment  # Get graph structure
```

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

## Research References

| Phase | Issue | Deliverable |
|-------|-------|-------------|
| Research | #528 | `docs/research/528-graph-api-research.md` |
| Design | #562 | `docs/research/527.2-design-dx-consistency.md` |
| OpenAPI Validation | #564 | `docs/research/527.3-openapi-validation.md` |
| SDK Types | #566 | `sdk/src/graph.rs` |
| SDK Client | #567 | `sdk/src/graph_client.rs` |
| CLI Graph | #569 | `cli/src/commands/graph.rs` |
| Documentation | #572 | This document |

---

## API Specifications

- **Control Plane API:** `reference/openapi/langchain/control-plane/openapi.json`
- **Agent Server API:** `reference/openapi/langchain/agent-server/openapi.json`
- **Fragment Index:** `reference/api-specs/agent-server/FRAGMENTS.md`

---

## Parent Issue

See [#527](https://github.com/codekiln/langstar/issues/527) for the full milestone tracking.
