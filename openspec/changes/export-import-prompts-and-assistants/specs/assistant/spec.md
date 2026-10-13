## ADDED Requirements

### Requirement: Export an assistant to a file

`langstar assistant export <assistant-id> --deployment <name-or-id>` SHALL write one JSON file holding the assistant's name, description, graph ID, configuration, context and metadata, and a `source` object naming the deployment, assistant ID and version. `--output <path>` SHALL choose the file; without it the file goes to stdout. Exporting the same assistant version twice SHALL produce identical files.

#### Scenario: Export an assistant

- **WHEN** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** `support-bot.json` holds the assistant's name, graph ID, configuration, context and metadata
- **AND** `source.assistant_id` holds the source assistant's ID

#### Scenario: Assistant not found

- **WHEN** the deployment has no assistant with that ID
- **THEN** langstar exits with status 1 and an error naming the assistant ID

### Requirement: Record the prompts an assistant refers to

`langstar assistant export` SHALL accept `--prompt-ref <json-pointer>`, repeatable, naming a string in the assistant's configuration or context that holds a prompt name. The file SHALL list each pointer and the prompt name it holds under `requires.prompts`. A pointer that resolves to no string SHALL be an error.

#### Scenario: Assistant names its system prompt

- **WHEN** a user runs `langstar assistant export <assistant-id> --deployment my-agent --prompt-ref /config/configurable/system_prompt`
- **AND** that value is `support-system:prod`
- **THEN** `requires.prompts` holds the pointer and `support-system:prod`

#### Scenario: Pointer names nothing

- **WHEN** the pointer resolves to no value in the assistant
- **THEN** langstar exits with status 1 and an error naming the pointer

### Requirement: Import an assistant into a deployment

`langstar assistant import <file> --deployment <name-or-id>` SHALL create an assistant on the deployment with the file's name, description, graph ID, configuration, context and metadata, and print the new assistant ID. It SHALL record the deployment and the new assistant ID in the destination record file, `<file stem>.destinations.json` unless `--record <path>` names another.

#### Scenario: First import

- **WHEN** a user runs `langstar assistant import support-bot.json --deployment my-agent-prod`
- **AND** the destination record has no entry for `my-agent-prod`
- **THEN** langstar creates the assistant and prints its new ID
- **AND** `support-bot.destinations.json` holds `my-agent-prod` with that ID

### Requirement: Find the earlier copy instead of creating another

When the destination record holds an assistant ID for the deployment, `langstar assistant import` SHALL compare the file with that assistant and create nothing. It SHALL exit with status 0 when they match, and list the differences and exit with status 2 when they differ, unless the user passes `--update`. When the recorded assistant no longer exists, it SHALL exit with status 1 and say so.

#### Scenario: Import the same file again

- **WHEN** a user imports `support-bot.json` into `my-agent-prod` a second time with no changes
- **THEN** langstar creates no assistant and reports that the assistant is unchanged

#### Scenario: Recorded assistant was deleted

- **WHEN** the recorded assistant ID no longer exists on the deployment
- **THEN** langstar exits with status 1 and names the missing assistant ID

### Requirement: Update an existing assistant only on request

`langstar assistant import --update` SHALL update the recorded assistant, or the assistant named by `--assistant-id <id>`, with the file's name, description, graph ID, configuration, context and metadata. `--assistant-id` SHALL take precedence over the destination record.

#### Scenario: Update the recorded assistant

- **WHEN** the file's configuration differs from the recorded assistant
- **AND** the user runs `langstar assistant import support-bot.json --deployment my-agent-prod --update`
- **THEN** langstar updates that assistant and creates no other

#### Scenario: Update an assistant chosen by ID

- **WHEN** the user runs `langstar assistant import support-bot.json --deployment my-agent-prod --update --assistant-id <dest-assistant-id>`
- **THEN** langstar updates that assistant and records its ID for `my-agent-prod`

### Requirement: Dry run an assistant import

`langstar assistant import --dry-run` SHALL report whether it would create the assistant, update it, or leave it unchanged, list the differences, and change nothing on the deployment or in the destination record.

#### Scenario: Dry run of a first import

- **WHEN** a user runs `langstar assistant import support-bot.json --deployment my-agent-prod --dry-run`
- **THEN** langstar reports that it would create the assistant and creates nothing

### Requirement: Read the assistant back after import

After writing, `langstar assistant import` SHALL read the assistant back from the deployment and compare it with the file. When they differ it SHALL list the differences and exit with status 2.

#### Scenario: Deployment drops a configuration key

- **WHEN** the assistant read back lacks a configuration key the file holds
- **THEN** langstar lists that key as a difference and exits with status 2

### Requirement: Compare an assistant with its export file

`langstar assistant compare <file> --deployment <name-or-id>` SHALL compare the file with the recorded assistant, or the one named by `--assistant-id`, and list each difference by its JSON path with both values. Fields the server sets, such as the assistant ID, version and timestamps, SHALL be left out of the comparison.

#### Scenario: Metadata changed on the deployment

- **WHEN** someone changed a metadata value on the deployment's assistant
- **THEN** langstar lists that metadata path with both values and exits with status 2

### Requirement: Verify that an assistant can run on a deployment

`langstar assistant verify <file> --deployment <name-or-id>` SHALL do what `compare` does and also check that the deployment serves the file's graph ID and that the workspace, from `--workspace-id` or the configured workspace, has each prompt under `requires.prompts`. It SHALL name each missing graph and prompt and exit with status 2 when any check fails.

#### Scenario: Graph missing on the destination

- **WHEN** the destination deployment serves no graph with the file's graph ID
- **THEN** langstar reports the missing graph ID and exits with status 2

#### Scenario: Referenced prompt missing

- **WHEN** the file lists `support-system:prod` under `requires.prompts`
- **AND** the workspace has no `support-system` prompt with a `prod` tag
- **THEN** langstar reports `support-system:prod` as missing and exits with status 2

### Requirement: Assistant compare, verify and import use the prompt commands' exit statuses and JSON report

`assistant compare`, `assistant verify` and `assistant import` SHALL use the exit statuses and the JSON report shape of `prompt compare`, `prompt verify` and `prompt import`.

#### Scenario: Machine-readable verify

- **WHEN** a user runs `langstar -f json assistant verify support-bot.json --deployment my-agent-prod`
- **THEN** stdout holds one JSON object with `status`, `differences` and `checks`
