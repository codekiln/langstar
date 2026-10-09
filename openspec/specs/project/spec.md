# project Specification

## Purpose

`langstar project` manages LangSmith tracing projects through the LangSmith API.

## Requirements

### Requirement: Name a project by ID or name

`langstar project get`, `update` and `delete` SHALL take a project ID or a project name. When the value is not a UUID, langstar SHALL look the project up by exact name. When no project has that name, langstar SHALL exit with an error that names it.

#### Scenario: Project named by name

- **WHEN** a user runs `langstar project get my-project`
- **AND** the workspace has a project named `my-project`
- **THEN** langstar shows that project

#### Scenario: Project not found

- **WHEN** a user names a project that does not exist
- **THEN** langstar exits with the error `Project not found: <name>`

### Requirement: Choose the output format per command

`langstar project list`, `get`, `create` and `update` SHALL print a table, JSON or tab-separated text. A `--format` passed to the subcommand SHALL take precedence over the global output format.

#### Scenario: Text output

- **WHEN** a user runs `langstar project list --format text`
- **THEN** langstar prints one line per project holding its ID, name and run count, separated by tabs

### Requirement: List projects

`langstar project list` SHALL list the workspace's projects, up to `--limit` (default 100). It SHALL filter by exact name with `--name` and by part of the name with `--name-contains`, and SHALL ask the API for run statistics when the user passes `--include-stats`. The table SHALL show each project's ID, name, description, run count and last run date, followed by how many projects it found.

#### Scenario: List with statistics

- **WHEN** a user runs `langstar project list --include-stats`
- **THEN** the table shows each project's run count and the date of its last run

### Requirement: Get one project

`langstar project get <id-or-name>` SHALL show one project. The table format SHALL show its name, ID, and, when the API returns them, its description, run count, median and 99th percentile latency, error rate and last run time.

#### Scenario: Show a project

- **WHEN** a user runs `langstar project get my-project`
- **THEN** langstar prints the project's name, ID and run count

### Requirement: Create a project

`langstar project create <name>` SHALL create a project with that name, an optional `--description`, and optional `--metadata` given as a JSON object. On success it SHALL print the new project's ID and name.

#### Scenario: Create with metadata

- **WHEN** a user runs `langstar project create my-project --metadata '{"environment": "staging"}'`
- **THEN** langstar creates `my-project` with that metadata and prints its ID

#### Scenario: Metadata is not JSON

- **WHEN** the `--metadata` value is not valid JSON
- **THEN** langstar exits with an `Invalid metadata JSON` error before calling the API

### Requirement: Update a project

`langstar project update <id-or-name>` SHALL change the project's name with `--name` and its description with `--description`, leaving unchanged any field the user does not pass. When the user passes neither flag, langstar SHALL print a warning and change nothing.

#### Scenario: Nothing to update

- **WHEN** a user runs `langstar project update my-project` with neither `--name` nor `--description`
- **THEN** langstar prints a warning naming both flags and leaves the project unchanged

### Requirement: Delete a project after confirmation

`langstar project delete <id-or-name>` SHALL ask the user to type `yes` before deleting the project, and SHALL skip the question when the user passes `--force`.

#### Scenario: User declines

- **WHEN** a user runs `langstar project delete my-project` and types anything other than `yes`
- **THEN** langstar prints that it cancelled the deletion and leaves the project in place

#### Scenario: Forced delete

- **WHEN** a user runs `langstar project delete my-project --force`
- **THEN** langstar deletes the project without asking and prints its ID
