## RENAMED Requirements

- FROM: `### Requirement: List the runs in a queue`
- TO: `### Requirement: List the items in a queue`

## MODIFIED Requirements

### Requirement: Remove a run from a queue

`langstar queue remove-run <queue-id> <run-id>` SHALL remove one run from the queue and print the run and queue IDs. If the queue holds no item for the run, it SHALL exit with an error that says the run is not in the queue.

#### Scenario: Remove a run

- **WHEN** a user runs `langstar queue remove-run <queue-id> <run-id>`
- **THEN** the run leaves the queue

#### Scenario: Remove a run that is not in the queue

- **WHEN** a user runs `langstar queue remove-run <queue-id> <run-id>` for a run the queue does not hold
- **THEN** langstar exits with an error and says the run is not in the queue

### Requirement: List the items in a queue

`langstar queue items <queue-id>` SHALL list up to `--limit` items (default 100), one review section after another. With `--status needs-my-review`, `--status needs-others-review` or `--status archived`, it SHALL list only that section. The table and the JSON `status` field name each item's section with underscores: `needs_my_review`, `needs_others_review`, `archived`. The table SHALL show each item's short ID, short run ID, type, review section and the date it was added, followed by how many items it found.

#### Scenario: Queue shorter than the limit

- **WHEN** a queue holds 5 items and the user runs `langstar queue items <queue-id>`
- **THEN** langstar lists the 5 items and says it found 5 items

#### Scenario: One review section

- **WHEN** a queue holds 3 archived items and 2 items that need review, and the user runs `langstar queue items <queue-id> --status archived`
- **THEN** langstar lists only the 3 archived items and says it found 3 items
