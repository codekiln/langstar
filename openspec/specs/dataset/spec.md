# dataset Specification

## Purpose

`langstar dataset` manages LangSmith datasets and the examples in them through the LangSmith API.

## Requirements

### Requirement: Choose JSON output with a flag

The `dataset` subcommands that print a dataset or a list SHALL print a table or a plain summary by default, and SHALL print the API's JSON when the user passes `--json`. They SHALL ignore the global `--format` option.

#### Scenario: JSON list

- **WHEN** a user runs `langstar dataset list --json`
- **THEN** langstar prints the datasets as a JSON array

#### Scenario: Global format has no effect

- **WHEN** a user runs `langstar -f json dataset list`
- **THEN** langstar prints the table, as it would without `-f json`

### Requirement: Create a dataset

`langstar dataset create` SHALL create a dataset from a required `--name`, an optional `--description`, and a `--data-type` of `kv`, `llm` or `chat` (default `kv`). On success it SHALL print the new dataset's ID, name and data type.

#### Scenario: Create a chat dataset

- **WHEN** a user runs `langstar dataset create --name support-chats --data-type chat`
- **THEN** langstar creates a chat dataset named `support-chats`
- **AND** prints its ID

#### Scenario: Unknown data type

- **WHEN** a user passes a `--data-type` other than `kv`, `llm` or `chat`
- **THEN** langstar exits with an error that names the three data types

### Requirement: List datasets

`langstar dataset list` SHALL list the workspace's datasets, up to `--limit` (default 100). It SHALL filter by exact name with `--name`, by part of the name with `--name-contains`, and by data type with `--data-type`. The table SHALL show each dataset's ID, name, data type, example count, description and modification date, followed by how many datasets it found.

#### Scenario: Filter by part of the name

- **WHEN** a user runs `langstar dataset list --name-contains support`
- **THEN** langstar lists only the datasets whose names contain `support`

### Requirement: Get one dataset

`langstar dataset get <dataset-id>` SHALL show one dataset, given its UUID: its name, ID, data type, description when set, example count, session count, and creation and modification times.

#### Scenario: Show a dataset

- **WHEN** a user runs `langstar dataset get <dataset-id>`
- **THEN** langstar prints the dataset's name, data type and example count

### Requirement: Update a dataset

`langstar dataset update <dataset-id>` SHALL change the dataset's name with `--name` and its description with `--description`, leaving unchanged any field the user does not pass. When the user passes neither flag, langstar SHALL print a warning and change nothing.

#### Scenario: Rename a dataset

- **WHEN** a user runs `langstar dataset update <dataset-id> --name support-chats-v2`
- **THEN** the dataset's name becomes `support-chats-v2`

#### Scenario: Nothing to update

- **WHEN** a user runs `langstar dataset update <dataset-id>` with neither `--name` nor `--description`
- **THEN** langstar prints a warning naming both flags and leaves the dataset unchanged

### Requirement: Delete a dataset only with --yes

`langstar dataset delete <dataset-id>` SHALL delete the dataset when the user passes `--yes` or `-y`. Without that flag, it SHALL print a message asking whether to delete the dataset, tell the user to pass `--yes`, and exit without deleting.

#### Scenario: Delete without --yes

- **WHEN** a user runs `langstar dataset delete <dataset-id>`
- **THEN** langstar tells the user to pass `--yes` and leaves the dataset in place

#### Scenario: Delete with --yes

- **WHEN** a user runs `langstar dataset delete <dataset-id> --yes`
- **THEN** langstar deletes the dataset and prints its ID

### Requirement: Import examples from JSONL or CSV

`langstar dataset import <dataset-id> --file <path>` SHALL add the examples in the file to the dataset. `--format` SHALL choose `jsonl` or `csv`; without it, langstar SHALL take the format from the file extension. Langstar SHALL skip blank lines, lines starting with `#`, and records it cannot read, warning about each record it skips, then report how many examples it imported.

#### Scenario: Import a JSONL file

- **WHEN** a user runs `langstar dataset import <dataset-id> --file examples.jsonl`
- **AND** each line holds an object with `inputs` and optionally `outputs`, `metadata` and `id`
- **THEN** langstar adds one example per line and prints how many it imported

#### Scenario: Unsupported format

- **WHEN** the format is neither `jsonl` nor `csv`
- **THEN** langstar exits with an error naming the two formats

#### Scenario: No readable examples

- **WHEN** the file holds no example langstar can read
- **THEN** langstar prints that it found no valid examples and adds nothing

### Requirement: Read CSV columns into examples

When importing a CSV file, langstar SHALL read the `id`, `inputs`, `outputs` and `metadata` columns, parsing `inputs`, `outputs` and `metadata` as JSON. A cell in `inputs` or `outputs` that is not JSON SHALL become `{"input": <cell>}` or `{"output": <cell>}`. When the file has no `inputs` column, langstar SHALL build the inputs from every other column.

#### Scenario: CSV without an inputs column

- **WHEN** a user imports a CSV file with columns `question` and `outputs`
- **THEN** each example's inputs hold the `question` value under the key `question`

### Requirement: List the examples in a dataset

`langstar dataset list-examples <dataset-id>` SHALL list the dataset's examples, up to `--limit` (default 100). The table SHALL show each example's ID, name, inputs, outputs and creation date, followed by how many examples it found.

#### Scenario: List examples

- **WHEN** a user runs `langstar dataset list-examples <dataset-id> --limit 10`
- **THEN** langstar shows at most 10 of the dataset's examples

### Requirement: Export examples to JSONL or CSV

`langstar dataset export <dataset-id>` SHALL write the dataset's examples as CSV, or as JSONL when the user passes `--file-format jsonl`. It SHALL write to the file named by `--out`, or to standard output without it. It SHALL export up to `--limit` examples, and up to 100 when `--limit` is absent.

#### Scenario: Export to a CSV file

- **WHEN** a user runs `langstar dataset export <dataset-id> --out examples.csv`
- **THEN** `examples.csv` holds a header row of `id`, `inputs`, `outputs` and `metadata` and one row per example, with each value as JSON
- **AND** langstar reports how many examples it exported

#### Scenario: Export JSONL to standard output

- **WHEN** a user runs `langstar dataset export <dataset-id> --file-format jsonl`
- **THEN** langstar prints one JSON object per example, holding its `id`, `inputs`, `outputs` and `metadata`
