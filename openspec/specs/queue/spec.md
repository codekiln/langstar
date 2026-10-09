# queue Specification

## Purpose

`langstar queue` manages LangSmith annotation queues and the runs in them.

## Requirements

### Requirement: Queue commands print JSON on request

The `langstar queue` subcommands SHALL print a table or a short summary by default, and SHALL print JSON when the user passes `--json`. They SHALL ignore the global `--format` flag. `queue list`, `create`, `get`, `update` and `items` SHALL take `--json`.

#### Scenario: JSON queue list

- **WHEN** a user runs `langstar queue list --json`
- **THEN** langstar prints the queues as a JSON array

### Requirement: List annotation queues

`langstar queue list` SHALL list the workspace's annotation queues, up to `--limit` (default 100). `--name` SHALL keep only the queue with that exact name, and `--name-contains` SHALL keep the queues whose names contain the given text. The table SHALL show each queue's short ID, name, type, description and creation date, followed by how many queues it found.

#### Scenario: Filter by part of a name

- **WHEN** a user runs `langstar queue list --name-contains review`
- **THEN** langstar lists only the queues whose names contain `review`

### Requirement: Create an annotation queue

`langstar queue create` SHALL create a queue from a required `--name`, with an optional `--description` and `--rubric` of instructions for annotators. `--queue-type` SHALL accept `single` (the default) or `pairwise`. On success it SHALL print the new queue's ID, name, type and creation time.

#### Scenario: Create a pairwise queue

- **WHEN** a user runs `langstar queue create --name compare-answers --queue-type pairwise`
- **THEN** langstar creates a pairwise queue named `compare-answers` and prints its ID

#### Scenario: Unknown queue type

- **WHEN** a user passes a `--queue-type` other than `single` or `pairwise`
- **THEN** langstar exits with an error that names the two accepted types

### Requirement: Get one annotation queue

`langstar queue get <queue-id>` SHALL show the queue's name, ID, type, and, when set, its description, rubric, and creation and update times. Langstar SHALL reject a queue ID that is not a UUID before calling the API.

#### Scenario: Malformed queue ID

- **WHEN** a user runs `langstar queue get not-a-uuid`
- **THEN** langstar exits with an argument error

### Requirement: Update an annotation queue

`langstar queue update <queue-id>` SHALL change the queue's name, description or rubric, taking `--name`, `--description` and `--rubric`. When the user passes none of them, it SHALL print a warning and change nothing.

#### Scenario: Nothing to update

- **WHEN** a user runs `langstar queue update <queue-id>` with no other flags
- **THEN** langstar warns that no updates were given and leaves the queue unchanged

### Requirement: Delete an annotation queue only with force

`langstar queue delete <queue-id>` SHALL delete the queue only when the user passes `--force`. Without `--force`, it SHALL print a message saying to add `--force` and exit without deleting.

#### Scenario: Delete without force

- **WHEN** a user runs `langstar queue delete <queue-id>`
- **THEN** langstar asks the user to rerun with `--force`
- **AND** the queue stays in place

#### Scenario: Delete with force

- **WHEN** a user runs `langstar queue delete <queue-id> --force`
- **THEN** langstar deletes the queue and prints its ID

### Requirement: Add runs to a queue

`langstar queue add-runs <queue-id>` SHALL add the runs whose IDs the user gives as arguments, from a file named by `--runs-file`, or both. The file SHALL hold one run ID per line; langstar SHALL skip blank lines and lines starting with `#`, and warn about and skip lines that are not UUIDs. On success it SHALL print how many runs it added.

#### Scenario: Add runs from a file

- **WHEN** a user runs `langstar queue add-runs <queue-id> --runs-file runs.txt`
- **AND** `runs.txt` holds three run IDs and one comment line
- **THEN** langstar adds the three runs to the queue and says it added 3 runs

### Requirement: Remove a run from a queue

`langstar queue remove-run <queue-id> <run-id>` SHALL remove one run from the queue and print the run and queue IDs.

#### Scenario: Remove a run

- **WHEN** a user runs `langstar queue remove-run <queue-id> <run-id>`
- **THEN** the run leaves the queue

### Requirement: List the runs in a queue

`langstar queue items <queue-id>` SHALL list the runs in the queue in order, up to `--limit` (default 100), stopping at the end of the queue. The table SHALL show each run's position in the queue, short run ID, name, status and the date it was added, followed by how many items it found.

#### Scenario: Queue shorter than the limit

- **WHEN** a queue holds 5 runs and the user runs `langstar queue items <queue-id>`
- **THEN** langstar lists the 5 runs and says it found 5 items
