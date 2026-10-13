## Purpose

`langstar prompt` manages the prompts saved in a LangSmith workspace, including writing a prompt to a file and recreating it in another workspace.

## ADDED Requirements

### Requirement: Export a prompt commit to a file

`langstar prompt export <handle>` SHALL write one JSON file holding the prompt's repo details (handle, description, readme, tags, visibility), the commit's manifest as LangSmith returns it with the model included, the commit's model configuration and provider, the commit tags that point at it, and a `source` object naming the workspace, handle and commit hash. `--output <path>` SHALL choose the file; without it the file goes to stdout.

#### Scenario: Export the latest commit

- **WHEN** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes `my-prompt.json` with the manifest of the latest commit, including the model binding
- **AND** `source.commit_hash` holds that commit's hash

#### Scenario: Export a structured prompt

- **WHEN** the prompt is a structured prompt with a nested JSON Schema and the `function_calling` method
- **THEN** the exported manifest holds the schema and the method unchanged

#### Scenario: Prompt not found

- **WHEN** no prompt in the workspace has the handle
- **THEN** langstar exits with status 1 and an error naming the handle

### Requirement: Export a chosen commit or tag

`langstar prompt export` SHALL accept `--commit <hash-or-tag>` and export that commit. Without the flag it SHALL export the latest commit.

#### Scenario: Export a tagged commit

- **WHEN** a user runs `langstar prompt export my-prompt --commit prod --output my-prompt.json`
- **THEN** the file holds the commit the `prod` tag points at

### Requirement: Write the same file for the same commit

Exporting the same commit twice SHALL produce identical files. The file SHALL hold no export time, and langstar SHALL write object keys in a fixed order.

#### Scenario: Export twice

- **WHEN** a user exports the same commit twice to two files
- **THEN** the two files are identical

### Requirement: List the credentials a prompt needs without their values

The export file SHALL list, under `requires.secrets`, the name of every secret the manifest refers to, and under `requires.model_provider`, the model's provider class name as LangSmith returns it, such as `ChatOpenAI`. It SHALL hold no secret values.

#### Scenario: Model reads an API key secret

- **WHEN** the prompt's model refers to the secret `OPENAI_API_KEY`
- **THEN** `requires.secrets` holds `OPENAI_API_KEY`
- **AND** no value of that secret appears in the file

### Requirement: Import names its destination workspace

`langstar prompt import <file>` SHALL require `--workspace-id`. It SHALL refuse to run when the flag is missing, even when `LANGSMITH_WORKSPACE_ID` is set.

#### Scenario: Missing destination workspace

- **WHEN** a user runs `langstar prompt import my-prompt.json` without `--workspace-id`
- **THEN** langstar exits with status 1 and an error asking for `--workspace-id`

### Requirement: Import a prompt that does not exist yet

When the destination workspace has no prompt with the file's handle, `langstar prompt import` SHALL create the prompt with the file's repo details, push the file's manifest as its first commit, and create the file's commit tags on that commit.

#### Scenario: First import

- **WHEN** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **AND** the destination has no prompt named `my-prompt`
- **THEN** langstar creates `my-prompt` in the destination with the file's manifest, model and tags
- **AND** prints the new commit hash

### Requirement: Change an existing prompt only with --update

When the destination prompt exists and its latest commit differs from the file, `langstar prompt import` SHALL change nothing and exit with status 2, listing the differences, unless the user passes `--update`. With `--update` it SHALL push the file's manifest as a new commit whose parent is the destination's latest commit, and move the file's tags to it.

#### Scenario: Destination differs and --update is missing

- **WHEN** the destination prompt's latest commit differs from the file
- **AND** the user runs `langstar prompt import` without `--update`
- **THEN** langstar pushes nothing, lists the differences and exits with status 2

#### Scenario: Update an existing prompt

- **WHEN** the user runs the same import with `--update`
- **THEN** langstar adds a commit to the destination prompt holding the file's manifest

#### Scenario: Destination already matches

- **WHEN** the destination prompt's latest commit matches the file
- **THEN** langstar pushes nothing, reports that the prompt is unchanged and exits with status 0

### Requirement: Record the destination commit

After a successful import, langstar SHALL record the destination workspace, handle and commit hash in a destination record file, `<file stem>.destinations.json` next to the export unless `--record <path>` names another. Each import SHALL add or replace only the entry for its own destination workspace.

#### Scenario: Import into two workspaces

- **WHEN** a user imports `my-prompt.json` into a staging workspace and then into a production workspace
- **THEN** `my-prompt.destinations.json` holds one entry for each workspace with its own commit hash
- **AND** `my-prompt.json` is unchanged

### Requirement: Warn when the destination changed since the last import

When the destination record holds a commit for the destination workspace and the destination's latest commit is a different one, `langstar prompt import` SHALL say that someone changed the prompt after the last import, in addition to listing the differences.

#### Scenario: Someone edited the destination prompt

- **WHEN** a user edited the destination prompt in the web interface after the last import
- **AND** runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **THEN** langstar reports that the destination changed since the last recorded import and exits with status 2

### Requirement: Dry run an import

`langstar prompt import --dry-run` SHALL report whether it would create the prompt, add a commit, or leave it unchanged, list the differences, and change nothing in the workspace or the destination record.

#### Scenario: Dry run against a new workspace

- **WHEN** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id> --dry-run`
- **AND** the destination has no prompt named `my-prompt`
- **THEN** langstar reports that it would create the prompt and creates nothing

### Requirement: Read the prompt back after import

After writing, `langstar prompt import` SHALL read the destination commit back with the model included and compare it with the file. When they differ it SHALL list the differences and exit with status 2.

#### Scenario: LangSmith drops a field

- **WHEN** the destination commit read back after import lacks a field the file holds
- **THEN** langstar lists that field as a difference and exits with status 2

### Requirement: Compare a prompt with its export file

`langstar prompt compare <file>` SHALL read the prompt with the file's handle from the workspace, with the model included, and list each difference by its JSON path with the file's value and the workspace's value. `--commit <hash-or-tag>` SHALL choose the commit; without it langstar SHALL read the latest commit. Fields the server sets, such as IDs and timestamps, SHALL be left out of the comparison.

#### Scenario: Prompt matches

- **WHEN** the workspace's prompt matches the file
- **THEN** langstar reports no differences and exits with status 0

#### Scenario: Model temperature changed

- **WHEN** someone changed the model temperature in the workspace's prompt
- **THEN** langstar lists the temperature's JSON path with both values and exits with status 2

### Requirement: Verify that a prompt can run in a workspace

`langstar prompt verify <file>` SHALL do what `compare` does and also check that the workspace has every secret in `requires.secrets`. It SHALL name each missing secret and exit with status 2 when any secret is missing. It SHALL also look for a saved model configuration whose settings `id` ends in the class name `requires.model_provider` holds, and report a missing one as a note that leaves the exit status unchanged, because the prompt runs with the model its manifest carries.

#### Scenario: Secret missing in the destination

- **WHEN** the destination workspace has no secret named `OPENAI_API_KEY`
- **AND** the file lists `OPENAI_API_KEY` under `requires.secrets`
- **THEN** langstar reports `OPENAI_API_KEY` as missing and exits with status 2

#### Scenario: No saved model configuration for the provider

- **WHEN** the file's `requires.model_provider` is `ChatOpenAI`
- **AND** no saved model configuration in the destination workspace has a settings `id` ending in `ChatOpenAI`
- **AND** every secret in `requires.secrets` exists and the prompt matches the file
- **THEN** langstar notes that no saved model configuration matches `ChatOpenAI` and exits with status 0

### Requirement: Exit with 0 for a match, 2 for a difference and 1 for an error

`prompt compare`, `prompt verify` and `prompt import` SHALL exit with status 0 when the prompt matches and every check passes, 2 when they find a difference or a failed check, and 1 on any other error. With `-f json` each SHALL print one JSON object with `status`, `differences` and `checks`.

#### Scenario: Machine-readable compare

- **WHEN** a user runs `langstar -f json prompt compare my-prompt.json`
- **THEN** stdout holds one JSON object with `status`, `differences` and `checks`
- **AND** progress messages go to stderr
