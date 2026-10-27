## ADDED Requirements

### Requirement: Export an assistant to a file

`langstar assistant export <assistant-id> --deployment <name-or-id>` SHALL write one JSON file holding the assistant's name, description, graph ID, configuration, context and metadata, and a `source` object naming the deployment, assistant ID and version. `--output <path>` SHALL choose the file; without it stdout SHALL hold only the file, and messages go to stderr. Exporting the same assistant version twice SHALL produce identical files.

#### Scenario: Export an assistant

- **WHEN** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** `support-bot.json` holds the assistant's name, graph ID, configuration, context and metadata
- **AND** `source.assistant_id` holds the source assistant's ID

#### Scenario: Assistant not found

- **WHEN** the deployment has no assistant with that ID
- **THEN** langstar exits with status 1 and an error naming the assistant ID

#### Scenario: Export to stdout

- **WHEN** a user runs `langstar assistant export <assistant-id> --deployment my-agent` without `--output`
- **THEN** stdout holds only the export file's JSON document
- **AND** any progress message goes to stderr

### Requirement: Refuse to export an assistant that holds a credential

Before it writes, `langstar assistant export` SHALL run the credential check over every string at any depth of the `assistant` object, skipping secret references and empty strings. When the check flags a string, export SHALL write no file and nothing to stdout, list the JSON Pointer path and rule of each flagged string, without the string or any part of it, and exit with status 1. Export SHALL NOT redact a flagged string and write the rest.

#### Scenario: Configuration holds a literal API key

- **WHEN** the assistant's `config.configurable.OPENAI_API_KEY` holds a string
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** langstar writes no file, names `/config/configurable/OPENAI_API_KEY` without the string and exits with status 1

#### Scenario: Configuration reads its key from a secret

- **WHEN** the assistant's `config.configurable.openai_api_key` holds a secret reference to `OPENAI_API_KEY`
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** langstar writes the file with the secret reference and exits with status 0

### Requirement: Flag an assistant string by the name of its key

The credential check SHALL flag a string when the key that holds it, or the array it is in, has a name whose words, split at `_`, `-`, `.`, spaces and lower-to-upper case changes and compared without regard to case, include `secret`, `token`, `password`, `passwd`, `pwd`, `credential`, `credentials`, `apikey`, `authorization` or `cookie`, or `key` right after `api`, `access`, `private`, `secret`, `signing` or `account`.

#### Scenario: Context holds an access token

- **WHEN** the assistant's `context.github_access_token` holds a string
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** langstar writes no file, names `/context/github_access_token` without the string and exits with status 1

#### Scenario: Configuration holds a database password

- **WHEN** the assistant's `config.configurable.db.password` holds a string
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** langstar writes no file, names `/config/configurable/db/password` without the string and exits with status 1

#### Scenario: Configuration sends an authorization header

- **WHEN** the assistant's `config.configurable.tool_headers.Authorization` holds `Bearer ` and a token
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** langstar writes no file, names `/config/configurable/tool_headers/Authorization` without the string and exits with status 1

### Requirement: Flag an assistant string by its shape under any key

The credential check SHALL flag a string, under any key, that holds a bearer token, a `Basic ` authorization value, a PEM private key, a URL with a password in its user info, a connection string with a non-empty `Password=`, `Pwd=`, `AccountKey=`, `SharedAccessKey=` or `SharedAccessSignature=`, a JSON Web Token, or a token that starts with a provider prefix the design lists: `sk-`, `lsv2_`, `ghp_`, `gho_`, `ghs_`, `github_pat_`, `xoxb-`, `xoxp-`, `xoxa-`, `AKIA`, `ASIA`, `AIza` or `hf_`.

#### Scenario: Configuration holds a connection string with a password

- **WHEN** the assistant's `config.configurable.checkpoint_uri` holds `postgresql://app:<password>@db.example.com/agent`
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json`
- **THEN** langstar writes no file, names `/config/configurable/checkpoint_uri` as a URL with a password and exits with status 1

#### Scenario: Metadata holds a private key

- **WHEN** the assistant's `metadata.deploy_note` holds a PEM block that begins `-----BEGIN OPENSSH PRIVATE KEY-----`
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent`
- **THEN** stdout is empty, langstar names `/metadata/deploy_note` as a private key and exits with status 1

### Requirement: Let the user export a flagged assistant string that holds no secret

`langstar assistant export` SHALL take `--not-a-secret <json-pointer>` any number of times. For each pointer at a string the credential check flags, export SHALL write that string and print a warning to stderr naming the pointer. A flagged string at any path not named SHALL still make export refuse. When a pointer names a path the check does not flag, export SHALL write nothing, name the pointer and exit with status 1. No flag SHALL turn the check off.

#### Scenario: Configuration names a token budget

- **WHEN** the assistant's `config.configurable.token_budget_policy` holds the string `strict`
- **AND** a user runs `langstar assistant export <assistant-id> --deployment my-agent --output support-bot.json --not-a-secret /config/configurable/token_budget_policy`
- **THEN** langstar writes the file with `strict` at that path, prints a warning naming the path to stderr and exits with status 0

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

`langstar assistant import <file> --deployment <name-or-id>` SHALL choose a new assistant ID, write it as pending under the deployment in the destination record (`<file stem>.destinations.json` unless `--record <path>` names another), then create the assistant with that `assistant_id`, `if_exists: do_nothing` and the file's name, description, graph ID, configuration, context and metadata, mark the entry created, and print the ID. A pending entry SHALL make the next import repeat the create.

#### Scenario: First import

- **WHEN** a user runs `langstar assistant import support-bot.json --deployment my-agent-prod`
- **AND** the destination record has no entry for `my-agent-prod`
- **THEN** langstar creates the assistant and prints its new ID
- **AND** `support-bot.destinations.json` holds the ID of the `my-agent-prod` deployment, with the new assistant ID under it, marked created

#### Scenario: Import stopped after the create and before marking the ID created

- **WHEN** the destination record holds a pending assistant ID for `my-agent-prod`
- **AND** the deployment already has an assistant with that ID
- **AND** a user runs `langstar assistant import support-bot.json --deployment my-agent-prod`
- **THEN** langstar sends the create with that ID and `if_exists` set to `do_nothing`, so the deployment still holds one assistant from the file
- **AND** the destination record marks that ID created

### Requirement: Replace the destination record in one step

`langstar assistant import` SHALL write each new destination record to a temporary file in the record's directory and rename it over the old record, so that a langstar that stops during a write leaves the earlier record whole, with its pending assistant IDs.

#### Scenario: Import stopped while marking the ID created

- **WHEN** langstar stops while it writes the record that marks a new assistant ID created
- **THEN** the destination record still holds that ID marked pending
- **AND** the next `langstar assistant import support-bot.json --deployment my-agent-prod` sends the create with that ID and creates no second assistant

### Requirement: Check the destination before importing an assistant

Before it writes, `langstar assistant import` SHALL run the graph check and the prompt check of `assistant verify`, reading prompts from `--workspace-id` or the configured workspace. When a check fails it SHALL write nothing, name each missing graph and prompt, and exit with status 3.

#### Scenario: Referenced prompt missing at import

- **WHEN** the file lists `support-system:prod` under `requires.prompts`
- **AND** the workspace has no `support-system` prompt with a `prod` tag
- **AND** a user runs `langstar assistant import support-bot.json --deployment my-agent-prod`
- **THEN** langstar creates no assistant, reports `support-system:prod` as missing and exits with status 3

### Requirement: Find the earlier copy instead of creating another

When the destination record holds a created assistant ID for the deployment, `langstar assistant import` SHALL compare the file with that assistant and create nothing. It SHALL exit with status 0 when they match, and list the differences and exit with status 3 when they differ, unless the user passes `--update`. When the recorded assistant no longer exists, it SHALL exit with status 1 and say so.

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

### Requirement: Import with --update asks the Agent Server to remove the metadata, description and context the file lacks

`langstar assistant import --update` SHALL ask the Agent Server to remove each metadata key the destination assistant has and the file lacks, and to clear a description or context the file lacks, then read the assistant back. When such a key or field remains after the update, it SHALL list it as a difference and exit with status 3.

#### Scenario: Remove a metadata key the file lacks

- **WHEN** the recorded assistant's metadata is `{"kept": 1, "stale": 2}`
- **AND** the file's metadata is `{"kept": 1}`
- **AND** the Agent Server removes a metadata key that an update sends with a null value
- **AND** the user runs `langstar assistant import support-bot.json --deployment my-agent-prod --update`
- **THEN** the assistant's metadata read back from the deployment is `{"kept": 1}`
- **AND** langstar exits with status 0

#### Scenario: Deployment keeps a removed metadata key

- **WHEN** the Agent Server keeps `stale` in the assistant's metadata after the update
- **THEN** langstar lists `/metadata/stale` as a difference and exits with status 3

#### Scenario: Deployment keeps a description the file lacks

- **WHEN** the recorded assistant has a description and the file has none
- **AND** the Agent Server keeps that description after the update
- **THEN** langstar lists `/description` as a difference and exits with status 3

### Requirement: Dry run an assistant import

`langstar assistant import --dry-run` SHALL report whether it would create the assistant, update it, or leave it unchanged, list the differences, and change nothing on the deployment or in the destination record.

#### Scenario: Dry run of a first import

- **WHEN** a user runs `langstar assistant import support-bot.json --deployment my-agent-prod --dry-run`
- **THEN** langstar reports that it would create the assistant and creates nothing

### Requirement: Read the assistant back after import

After writing, `langstar assistant import` SHALL record the assistant ID in the destination record, then read the assistant back from the deployment and compare it with the file. When they differ it SHALL list the differences and exit with status 3, and the record SHALL keep the ID so that the next import finds that assistant instead of creating another.

#### Scenario: Deployment drops a configuration key

- **WHEN** the assistant read back lacks a configuration key the file holds
- **THEN** langstar lists that key as a difference and exits with status 3
- **AND** the destination record holds the new assistant ID, so importing the file again creates no assistant

### Requirement: Compare an assistant with its export file

`langstar assistant compare <file> --deployment <name-or-id>` SHALL compare the file with the assistant recorded in the destination record (`<file stem>.destinations.json` unless `--record <path>` names another), or the one named by `--assistant-id`, and list each difference by its JSON path with both values. It SHALL compare only the fields the export file holds, leaving out the assistant ID, version and timestamps the server sets.

#### Scenario: Metadata changed on the deployment

- **WHEN** someone changed a metadata value on the deployment's assistant
- **THEN** langstar lists that metadata path with both values and exits with status 3

#### Scenario: No assistant to compare

- **WHEN** the destination record has no entry for `my-agent-prod`
- **AND** a user runs `langstar assistant compare support-bot.json --deployment my-agent-prod` without `--assistant-id`
- **THEN** langstar exits with status 1 and an error that names the deployment and suggests `--assistant-id`

#### Scenario: Record kept at another path

- **WHEN** a user imported `support-bot.json` with `--record records/prod.json`
- **AND** runs `langstar assistant compare support-bot.json --deployment my-agent-prod --record records/prod.json`
- **THEN** langstar compares the file with the assistant that import recorded in `records/prod.json`

### Requirement: Verify that an assistant can run on a deployment

`langstar assistant verify <file> --deployment <name-or-id>` SHALL do what `compare` does and also check that the deployment serves the file's graph ID and that the workspace, from `--workspace-id` or the configured workspace, has each prompt under `requires.prompts`. It SHALL name each missing graph and prompt and exit with status 3 when any check fails.

#### Scenario: Graph missing on the destination

- **WHEN** the destination deployment serves no graph with the file's graph ID
- **THEN** langstar reports the missing graph ID and exits with status 3

#### Scenario: Referenced prompt missing

- **WHEN** the file lists `support-system:prod` under `requires.prompts`
- **AND** the workspace has no `support-system` prompt with a `prod` tag
- **THEN** langstar reports `support-system:prod` as missing and exits with status 3

### Requirement: Reject an export file of another kind or format version

`assistant import`, `assistant compare` and `assistant verify` SHALL read a file only when its `kind` is `assistant` and its `format_version` is 1. For any other file they SHALL write nothing, exit with status 1 and name the field and the value they found.

#### Scenario: Prompt file given to an assistant command

- **WHEN** a user runs `langstar assistant import my-prompt.json --deployment my-agent-prod`
- **AND** `my-prompt.json` holds `"kind": "prompt"`
- **THEN** langstar creates no assistant, exits with status 1 and names `kind` and the value `prompt`

### Requirement: Reject a file whose prompt reference no longer matches its assistant

`assistant import`, `assistant compare` and `assistant verify` SHALL resolve each pointer under `requires.prompts` against the file's `assistant` object when they read the file. When a pointer resolves to no string, or to a string other than the prompt name recorded with it, they SHALL write nothing, exit with status 1 and name the pointer and both values.

#### Scenario: Configuration edited after export

- **WHEN** `requires.prompts` records `/config/configurable/system_prompt` with `support-system:prod`
- **AND** the file's `assistant.config.configurable.system_prompt` is `support-system:staging`
- **AND** the user runs `langstar assistant verify support-bot.json --deployment my-agent-prod`
- **THEN** langstar exits with status 1 and names the pointer, `support-system:prod` and `support-system:staging`

### Requirement: Assistant compare, verify and import use the prompt commands' exit statuses and JSON report

`assistant compare`, `assistant verify` and `assistant import` SHALL use the exit statuses and the JSON report shape of `prompt compare`, `prompt verify` and `prompt import`.

#### Scenario: Machine-readable verify

- **WHEN** a user runs `langstar -f json assistant verify support-bot.json --deployment my-agent-prod`
- **THEN** stdout holds one JSON object with `status`, `differences` and `checks`

### Requirement: Redact a credential in an assistant difference

When `assistant compare`, `assistant verify` or `assistant import` prints either side of a difference, in text or JSON, langstar SHALL print `<redacted>` in place of each string the credential check from "Refuse to export an assistant that holds a credential" flags. It SHALL redact the string when the difference is at the string or at an object that contains it, and whether the string came from the destination or from the file.

#### Scenario: Destination configuration holds a literal API key

- **WHEN** someone updated the destination assistant so `config.configurable.openai_api_key` holds a string
- **AND** a user runs `langstar assistant compare support-bot.json --deployment my-agent-prod`
- **THEN** langstar reports `/assistant/config/configurable/openai_api_key` with `<redacted>` in place of the string and exits with status 3

#### Scenario: Destination configuration holds a literal API key under an upper-case name

- **WHEN** someone updated the destination assistant so `config.configurable.OPENAI_API_KEY` holds a string
- **AND** a user runs `langstar assistant compare support-bot.json --deployment my-agent-prod`
- **THEN** langstar reports `/assistant/config/configurable/OPENAI_API_KEY` with `<redacted>` in place of the string and exits with status 3

#### Scenario: Destination context holds a connection string with a password

- **WHEN** someone updated the destination assistant so `context.checkpoint_uri` holds `postgresql://app:<password>@db.example.com/agent`
- **AND** a user runs `langstar assistant compare support-bot.json --deployment my-agent-prod -f json`
- **THEN** the text and the JSON report print `<redacted>` at `/assistant/context/checkpoint_uri`, neither holds the password, and langstar exits with status 3

#### Scenario: Destination adds a configuration object that holds a password

- **WHEN** the file's configuration has no `db` key
- **AND** someone updated the destination assistant so `config.configurable.db` holds `{"user": "app", "password": "<password>"}`
- **AND** a user runs `langstar assistant compare support-bot.json --deployment my-agent-prod`
- **THEN** langstar reports `/assistant/config/configurable/db` with `user` unchanged and `<redacted>` under `password`, and exits with status 3

### Requirement: Print a secret reference in a destination assistant's configuration unchanged

When `assistant compare`, `assistant verify` or `assistant import` prints either side of a difference, in the text or the JSON report, langstar SHALL print a LangChain secret reference unchanged under any key, so the report names the secret. A secret reference looks like `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`.

#### Scenario: Destination configuration reads its key from a secret

- **WHEN** someone updated the destination assistant so `config.configurable.openai_api_key` refers to the secret `OPENAI_API_KEY`
- **AND** a user runs `langstar assistant compare support-bot.json --deployment my-agent-prod`
- **THEN** langstar reports `/assistant/config/configurable/openai_api_key` with the secret reference and `OPENAI_API_KEY` in it, and exits with status 3
