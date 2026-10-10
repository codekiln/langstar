# assistant Specification

## Purpose

`langstar assistant` manages the assistants on a LangSmith deployment through the deployment's Agent Server API.

## Requirements

### Requirement: Every assistant command targets one deployment

Every `langstar assistant` subcommand SHALL take a required `--deployment` flag holding a deployment name or ID. Langstar SHALL look the deployment up through the control plane API and send the assistant request to that deployment's URL.

#### Scenario: Deployment found by name

- **WHEN** a user runs `langstar assistant list --deployment my-agent`
- **AND** the workspace has a deployment named `my-agent`
- **THEN** langstar lists the assistants on that deployment

#### Scenario: Deployment not found

- **WHEN** a user passes a `--deployment` that matches no deployment name or ID in the workspace
- **THEN** langstar exits with an error that names the deployment and suggests `langstar deployment list`

#### Scenario: Deployment has no URL

- **WHEN** the deployment exists but has no custom URL
- **THEN** langstar exits with an error saying the deployment has no URL

### Requirement: List assistants

`langstar assistant list` SHALL list the assistants on the deployment, taking `--limit` (default 20) and `--offset` (default 0). In table format it SHALL show each assistant's ID, name, graph ID and creation date, and report how many it found. In JSON format it SHALL print an object with an `assistants` array.

#### Scenario: Page through assistants

- **WHEN** a user runs `langstar assistant list --deployment my-agent --limit 10 --offset 10`
- **THEN** langstar shows at most 10 assistants, starting after the first 10

### Requirement: Choose the columns of text output

In text format, `langstar assistant list` SHALL print one tab-separated line per assistant. `--columns` SHALL choose the columns from `assistant_id`, `name`, `graph_id`, `description` and `created_at`; without it, every column appears. `--show-columns` SHALL print the available column names.

#### Scenario: Choose text columns

- **WHEN** a user runs `langstar assistant list --deployment my-agent -f text --columns assistant_id,name`
- **THEN** each output line holds an assistant's ID and name, separated by a tab

#### Scenario: Unknown column

- **WHEN** a user passes a column name that is not in the list of available columns
- **THEN** langstar exits with an error that lists the available columns

#### Scenario: Show available columns

- **WHEN** a user runs `langstar assistant list --deployment my-agent --show-columns`
- **THEN** langstar prints the available column names and exits without listing assistants

### Requirement: Search assistants by name

`langstar assistant search <query>` SHALL list the assistants on the deployment whose names match the query, up to `--limit` results (default 20). It SHALL use the same table and JSON output as `assistant list`.

#### Scenario: No match

- **WHEN** no assistant on the deployment matches the query
- **THEN** langstar prints an empty table and says no assistants matched the query

### Requirement: Get one assistant

`langstar assistant get <assistant-id>` SHALL show one assistant. In JSON format it SHALL print the assistant as the Agent Server returns it. In table format it SHALL print the assistant's ID, name, graph ID, and, when set, its description, creation and update times, configuration and metadata.

#### Scenario: Show configuration in table format

- **WHEN** a user gets an assistant that has a configuration
- **THEN** the table output includes the configuration as indented JSON

### Requirement: Create an assistant

`langstar assistant create` SHALL create an assistant from a required `--graph-id` and `--name`, with an optional `--description`. It SHALL take the assistant's configuration as inline JSON through `--config` or from a JSON file through `--config-file`, and SHALL reject a command that passes both. On success it SHALL print the new assistant and its ID.

#### Scenario: Create with a configuration file

- **WHEN** a user runs `langstar assistant create --deployment my-agent --graph-id agent --name support --config-file config.json`
- **THEN** langstar creates an assistant named `support` on graph `agent` with the configuration in `config.json`
- **AND** prints the new assistant's ID

#### Scenario: Both configuration flags

- **WHEN** a user passes both `--config` and `--config-file`
- **THEN** langstar exits with an argument error before calling the API

### Requirement: Update an assistant

`langstar assistant update <assistant-id>` SHALL change the assistant's name, description or configuration, taking `--name`, `--description`, and either `--config` or `--config-file`. It SHALL leave unchanged any field the user does not pass.

#### Scenario: Rename an assistant

- **WHEN** a user runs `langstar assistant update <assistant-id> --deployment my-agent --name support-v2`
- **THEN** the assistant's name becomes `support-v2`
- **AND** its description and configuration stay as they were

### Requirement: Delete an assistant after confirmation

`langstar assistant delete <assistant-id>` SHALL ask the user to confirm with `y` or `yes` before deleting the assistant, and SHALL skip the question when the user passes `-y` or `--force`. When langstar would ask the user to confirm but standard input is not a terminal, as when a script runs the command, it SHALL leave the assistant in place and exit with an error that says to pass `--force`. In JSON format it SHALL print an object whose `deleted` field holds the assistant ID.

#### Scenario: User declines

- **WHEN** a user runs `langstar assistant delete <assistant-id> --deployment my-agent` and answers anything other than `y` or `yes`
- **THEN** langstar prints that it cancelled and leaves the assistant in place

#### Scenario: Forced delete

- **WHEN** a user runs `langstar assistant delete <assistant-id> --deployment my-agent --force`
- **THEN** langstar deletes the assistant without asking

#### Scenario: Standard input is not a terminal

- **WHEN** a user runs `langstar assistant delete <assistant-id> --deployment my-agent` with standard input that is not a terminal and without `--force`
- **THEN** langstar exits with an error saying to pass `--force`, and deletes nothing
