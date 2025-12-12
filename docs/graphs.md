# LangGraph Graphs

Inspect LangGraph graph structure within deployments using the `langstar graph` commands. Graphs represent the workflow topology defined in your `langgraph.json` configuration.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Quickstart](#quickstart)
- [Command Reference](#command-reference)
- [Understanding Graph Structure](#understanding-graph-structure)
- [Common Workflows](#common-workflows)
- [Troubleshooting](#troubleshooting)

## Overview

Graphs in LangGraph Cloud define the workflow structure of your applications:

- **Nodes**: Individual processing steps or states in the workflow
- **Edges**: Connections between nodes that define execution flow
- **Conditional Edges**: Branching logic based on runtime conditions

### Key Concepts

| Concept   | Description                                                                      |
| --------- | -------------------------------------------------------------------------------- |
| Graph     | A workflow defined in `langgraph.json` at deployment time                        |
| Graph ID  | String identifier matching the key in `langgraph.json`                           |
| Assistant | A runtime instance that uses a specific graph                                    |
| Nodes     | Processing steps in the graph (excludes `__start__` and `__end__` control nodes) |
| Edges     | Connections between nodes (may be conditional)                                   |

### How Graphs Relate to Assistants

- Each graph can have multiple assistants with different configurations
- An assistant's `graph_id` field links it to its underlying graph
- When you create an assistant, you specify which graph it should use

## Prerequisites

1. **LangSmith API Key**: Set your API key:
   ```bash
   export LANGSMITH_API_KEY=<your-api-key>
   ```

2. **Langstar CLI**: Install langstar CLI (see main README for installation)

3. **A Deployment**: Graphs exist within deployments. You need at least one deployment to inspect graphs.

## Quickstart

### List Graphs in a Deployment

```bash
langstar graph list my-deployment
```

Output:

```
╭──────────────┬─────────────────────┬──────────────┬─────────────────────╮
│ Graph ID     │ Assistants          │ # Assistants │ Nodes               │
├──────────────┼─────────────────────┼──────────────┼─────────────────────┤
│ agent        │ default, custom-v1  │ 2            │ Responder, Feedback │
│ rag_pipeline │ rag-assistant       │ 1            │ Retriever, Generate │
╰──────────────┴─────────────────────┴──────────────┴─────────────────────╯
```

### Get Graph Structure

```bash
langstar graph get agent --deployment my-deployment
```

Output:

```json
{
  "graph_id": "agent",
  "nodes_count": 4,
  "edges_count": 4,
  "nodes": ["Responder", "Feedback"],
  "nodes_summary": "4 nodes",
  "edges_summary": "4 edges"
}
```

---

## Command Reference

### `langstar graph list`

List all unique graphs in a deployment by scanning assistants.

```bash
langstar graph list <DEPLOYMENT> [OPTIONS]
```

**Arguments:**

- `<DEPLOYMENT>` - Deployment name or ID (required)

**Options:**
| Option | Description |
|--------|-------------|
| `--show-nodes` | Fetch and display node details for each graph |
| `-f, --format <FORMAT>` | Output format: `table`, `json` |

**Examples:**

```bash
# List graphs by deployment name
langstar graph list my-deployment

# List graphs by deployment ID
langstar graph list 47599969-47ab-49d5-878e-cc6dbcbed059

# Show node details
langstar graph list my-deployment --show-nodes

# JSON output for scripting
langstar graph list my-deployment --format json
```

**Output Columns:**
| Column | Description |
|--------|-------------|
| Graph ID | The graph identifier from `langgraph.json` |
| Assistants | Comma-separated list of assistant names using this graph |
| # Assistants | Count of assistants using this graph |
| Nodes | Node names (excluding control nodes `__start__` and `__end__`) |

### `langstar graph get`

Get the detailed structure of a specific graph.

```bash
langstar graph get <GRAPH_ID> --deployment <DEPLOYMENT> [OPTIONS]
```

**Arguments:**

- `<GRAPH_ID>` - The graph ID to inspect

**Options:**
| Option | Description |
|--------|-------------|
| `--deployment <DEPLOYMENT>` | Deployment name or ID (required) |
| `--xray` | Include subgraph details in response |
| `-f, --format <FORMAT>` | Output format: `table`, `json` |

**Examples:**

```bash
# Get graph structure
langstar graph get agent --deployment my-deployment

# Include subgraph details
langstar graph get agent --deployment my-deployment --xray

# Full JSON output
langstar graph get agent --deployment my-deployment --format json
```

**JSON Output Structure:**

```json
{
  "nodes": [
    { "id": "__start__", "type": "runnable", "data": { "name": "__start__" } },
    { "id": "Responder", "type": "runnable", "data": { "name": "Responder" } },
    { "id": "Feedback", "type": "runnable", "data": { "name": "Feedback" } },
    { "id": "__end__" }
  ],
  "edges": [
    { "source": "__start__", "target": "Responder" },
    { "source": "Responder", "target": "Feedback", "conditional": true },
    { "source": "Responder", "target": "__end__", "conditional": true },
    { "source": "Feedback", "target": "__end__" }
  ]
}
```

---

## Understanding Graph Structure

### Nodes

Nodes represent processing steps in your graph:

| Node Type    | Description                                        |
| ------------ | -------------------------------------------------- |
| `__start__`  | Entry point (control node, hidden in table output) |
| `__end__`    | Exit point (control node, hidden in table output)  |
| User-defined | Your custom nodes (e.g., `Responder`, `Retriever`) |

### Edges

Edges define the flow between nodes:

| Edge Type   | `conditional` | Description                            |
| ----------- | ------------- | -------------------------------------- |
| Direct      | `false`       | Always follows this path               |
| Conditional | `true`        | May follow based on runtime conditions |

### Example: Simple Agent Graph

```
langgraph.json:
{
  "graphs": {
    "agent": "./agent.py:graph"
  }
}
```

Produces a graph like:

```
__start__ → Responder → __end__
              ↓
           Feedback → __end__
```

---

## Common Workflows

### Explore a Deployment's Graphs

```bash
# First, list available deployments
langstar deployment list

# Then inspect graphs in a deployment
langstar graph list my-deployment --show-nodes

# Get details on a specific graph
langstar graph get agent --deployment my-deployment --xray --format json
```

### Validate Graph Structure in CI/CD

```bash
#!/bin/bash
# validate-graph.sh

DEPLOYMENT="$1"
EXPECTED_GRAPH="agent"

# Check if expected graph exists
GRAPHS=$(langstar graph list "$DEPLOYMENT" --format json)
if echo "$GRAPHS" | jq -e ".[] | select(.graph_id == \"$EXPECTED_GRAPH\")" > /dev/null; then
  echo "✓ Graph '$EXPECTED_GRAPH' found in deployment"
else
  echo "✗ Graph '$EXPECTED_GRAPH' not found!"
  exit 1
fi
```

### Compare Graphs Across Deployments

```bash
# Get graph structure from staging
langstar graph get agent --deployment staging --format json > staging-graph.json

# Get graph structure from production
langstar graph get agent --deployment production --format json > prod-graph.json

# Compare
diff staging-graph.json prod-graph.json
```

---

## Troubleshooting

### "Deployment not found"

The deployment name or ID doesn't match any existing deployment.

```bash
# List available deployments
langstar deployment list

# Use exact name or ID from the list
langstar graph list <exact-name-or-id>
```

### "No graphs found in this deployment"

The deployment has no assistants, or hasn't finished initializing.

1. Check deployment status:
   ```bash
   langstar deployment get <deployment-id>
   ```

2. Ensure status is `READY`

3. Verify your `langgraph.json` defines graphs correctly

### Graph shows only control nodes

If `langstar graph list` shows empty "Nodes" column:

1. Use `--show-nodes` flag to fetch node details
2. Check that your graph has user-defined nodes (not just `__start__` and `__end__`)

---

## Migration from Previous Versions

> **Note:** In v0.5.0, `langstar graph` was repurposed for graph inspection. Previous `langstar graph` commands for deployment management moved to `langstar deployment`.

| Old Command (pre-v0.5.0)  | New Command                    |
| ------------------------- | ------------------------------ |
| `langstar graph list`     | `langstar deployment list`     |
| `langstar graph get <id>` | `langstar deployment get <id>` |
| `langstar graph create`   | `langstar deployment create`   |
| `langstar graph delete`   | `langstar deployment delete`   |

See [Command Migration Guide](./README.md#command-migration-v050) for details.

---

## See Also

- [Deployments Documentation](./deployments.md) - Manage deployment lifecycle
- [Assistants Documentation](./README.md#langgraph-assistants-deployment-level) - Manage assistants within deployments
- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)
- [Agent Server API Reference](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref/)
