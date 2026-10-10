# graph Specification

## Purpose

`langstar graph` shows the graphs a LangSmith deployment serves and the structure of each one, through the deployment's Agent Server API.

## Requirements

### Requirement: Every graph command targets one deployment

Every `langstar graph` subcommand SHALL take a deployment name or ID: `graph list` as its positional argument, and `graph get` through a required `--deployment` flag. Langstar SHALL look the deployment up through the control plane API and send the graph request to that deployment's URL. It SHALL report a deployment it cannot find, or one with no URL, the same way `langstar assistant` does.

#### Scenario: List graphs by deployment name

- **WHEN** a user runs `langstar graph list my-agent`
- **AND** the workspace has a deployment named `my-agent`
- **THEN** langstar lists the graphs on that deployment

#### Scenario: Deployment not found

- **WHEN** a user names a deployment that matches no deployment name or ID in the workspace
- **THEN** langstar exits with an error that names the deployment and suggests `langstar deployment list`

### Requirement: List the graphs in a deployment

`langstar graph list` SHALL list each graph that at least one assistant on the deployment uses, with the names and the number of the assistants that use it. With `--show-nodes`, it SHALL also fetch each graph's structure and list its node names. In JSON format it SHALL print an array of graph summaries.

#### Scenario: Show nodes

- **WHEN** a user runs `langstar graph list my-agent --show-nodes`
- **THEN** each row shows the graph ID, its assistants, the number of assistants and the graph's node names

#### Scenario: Deployment with no assistants

- **WHEN** the deployment has no assistants
- **THEN** langstar says it found no graphs in the deployment

### Requirement: Show the structure of one graph

`langstar graph get <graph-id>` SHALL fetch the graph's nodes and edges. With `--xray`, it SHALL include the nodes and edges of subgraphs. In JSON format it SHALL print the graph as the Agent Server returns it. In table and text format it SHALL print a JSON summary with the node and edge counts and the IDs of the nodes other than `__start__` and `__end__`.

#### Scenario: Summarize a graph

- **WHEN** a user runs `langstar graph get agent --deployment my-agent`
- **THEN** langstar prints the graph ID, the number of nodes and edges, and the graph's own node IDs

#### Scenario: Include subgraphs

- **WHEN** a user runs `langstar graph get agent --deployment my-agent --xray -f json`
- **THEN** the output includes the nodes and edges of the graph's subgraphs
