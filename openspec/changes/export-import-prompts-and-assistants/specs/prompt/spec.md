## Purpose

`langstar prompt` manages the prompts saved in a LangSmith workspace, including writing a prompt to a file and recreating it in another workspace.

## ADDED Requirements

### Requirement: Export a prompt commit to a file

`langstar prompt export <handle>` SHALL write one JSON file holding the prompt's repo details (handle, description, readme, tags, visibility), the commit's description, the commit's manifest as LangSmith returns it with the model included, the commit's model configuration and provider, the commit tags that point at it, and a `source` object naming the workspace, handle and commit hash. `--output <path>` SHALL choose the file; without it the file goes to stdout.

#### Scenario: Export the latest commit

- **WHEN** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes `my-prompt.json` with the manifest of the latest commit, including the model binding
- **AND** `source.commit_hash` holds that commit's hash
- **AND** `commit.description` holds that commit's description

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

The export file SHALL list, under `requires.secrets`, the name of every secret the manifest refers to, and under `requires.model_provider`, the class name at the end of the chat model constructor's `id` in the manifest, such as `ChatOpenAI`. It SHALL hold no secret values. Import and verify SHALL work out both lists again from the file's manifest, and when either differs from the file's `requires`, write nothing, name the field and exit with status 1.

#### Scenario: Model reads an API key secret

- **WHEN** the prompt's model refers to the secret `OPENAI_API_KEY`
- **THEN** `requires.secrets` holds `OPENAI_API_KEY`
- **AND** no value of that secret appears in the file

#### Scenario: Manifest edited without its requires list

- **WHEN** someone edits the file's manifest so its model refers to the secret `ANTHROPIC_API_KEY`, and leaves `requires.secrets` holding only `OPENAI_API_KEY`
- **AND** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **THEN** langstar creates no prompt, names `requires.secrets` and exits with status 1

### Requirement: Import names its destination workspace

`langstar prompt import <file>` SHALL require `--workspace-id`. It SHALL refuse to run when the flag is missing, even when `LANGSMITH_WORKSPACE_ID` is set, and exit with status 1 and an error naming the flag.

#### Scenario: Missing destination workspace

- **WHEN** a user runs `langstar prompt import my-prompt.json` without `--workspace-id`
- **AND** `LANGSMITH_WORKSPACE_ID` is set
- **THEN** langstar exits with status 1 and an error naming `--workspace-id`

### Requirement: Check the destination before importing a prompt

Before it writes, `langstar prompt import` SHALL run the secret check and the saved model configuration check of `prompt verify` against the destination workspace, and not its comparison, because the destination may have no prompt yet. When the workspace lacks a secret in `requires.secrets` it SHALL write nothing, name each missing secret and exit with status 3. A missing saved model configuration SHALL be a note, as in `prompt verify`.

#### Scenario: Secret missing at import

- **WHEN** the file lists `OPENAI_API_KEY` under `requires.secrets`
- **AND** the destination workspace has no secret named `OPENAI_API_KEY`
- **AND** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **THEN** langstar creates no prompt, reports `OPENAI_API_KEY` as missing and exits with status 3

### Requirement: Import a prompt that does not exist yet

When the destination workspace has no prompt with the file's handle, `langstar prompt import` SHALL create the prompt with the file's repo details, push the file's manifest and commit description as its first commit, and create the file's commit tags on that commit.

#### Scenario: First import

- **WHEN** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **AND** the destination has no prompt named `my-prompt`
- **THEN** langstar creates `my-prompt` in the destination with the file's manifest, model and tags
- **AND** prints the new commit hash

### Requirement: Change an existing prompt only with --update

When the destination prompt exists and its latest commit differs from the file, `langstar prompt import` SHALL change nothing and exit with status 3, listing the differences, unless the user passes `--update`. With `--update` it SHALL push the file's manifest and commit description as a new commit whose parent is the destination's latest commit, and move the file's tags to it.

#### Scenario: Destination differs and --update is missing

- **WHEN** the destination prompt's latest commit differs from the file
- **AND** the user runs `langstar prompt import` without `--update`
- **THEN** langstar pushes nothing, lists the differences and exits with status 3

#### Scenario: Update an existing prompt

- **WHEN** the user runs the same import with `--update`
- **THEN** langstar adds a commit to the destination prompt holding the file's manifest


#### Scenario: Destination already matches

- **WHEN** the destination prompt's latest commit and its commit tags match the file
- **THEN** langstar pushes nothing, reports that the prompt is unchanged and exits with status 0

### Requirement: Update the repo details only with --update

When the destination repo's description, readme, tags or visibility differ from the file, `langstar prompt import` SHALL treat them as differences. With `--update` it SHALL set those fields to the file's values. When only those fields differ and the latest commit matches the file, it SHALL update the repo and add no commit.

#### Scenario: Only the repo readme differs

- **WHEN** the destination prompt's latest commit matches the file
- **AND** the destination repo's readme differs from the file's readme
- **AND** the user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id> --update`
- **THEN** langstar sets the destination repo's readme to the file's readme and adds no commit
- **AND** exits with status 0

### Requirement: Move commit tags only with --update

When a commit tag in the file is missing in the destination or points at a different commit, `langstar prompt import` SHALL list the tag as a difference and exit with status 3. With `--update` it SHALL create or move the tag to the destination commit that matches the file, and add no commit when the latest commit already matches.

#### Scenario: Destination tag points at an older commit

- **WHEN** the destination prompt's latest commit matches the file
- **AND** the destination's `prod` tag points at an earlier commit
- **AND** the user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id> --update`
- **THEN** langstar moves `prod` to the latest commit and adds no commit
- **AND** exits with status 0

### Requirement: Record the destination commit

After import writes a commit, langstar SHALL record the destination workspace, handle and commit hash in a destination record file, `<file stem>.destinations.json` next to the export unless `--record <path>` names another, before it reads the commit back. Each import SHALL add or replace only the entry for its own destination workspace.

#### Scenario: Import into two workspaces

- **WHEN** a user imports `my-prompt.json` into a staging workspace and then into a production workspace
- **THEN** `my-prompt.destinations.json` holds one entry for each workspace with its own commit hash
- **AND** `my-prompt.json` is unchanged

### Requirement: Warn when the destination changed since the last import

When the destination record holds a commit for the destination workspace and the destination's latest commit is a different one, `langstar prompt import` SHALL say that someone changed the prompt after the last import, in addition to listing the differences.

#### Scenario: Someone edited the destination prompt

- **WHEN** a user edited the destination prompt in the web interface after the last import
- **AND** runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **THEN** langstar reports that the destination changed since the last recorded import and exits with status 3

### Requirement: Dry run an import

`langstar prompt import --dry-run` SHALL report whether it would create the prompt, add a commit, or leave it unchanged, list the differences, and change nothing in the workspace or the destination record.

#### Scenario: Dry run against a new workspace

- **WHEN** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id> --dry-run`
- **AND** the destination has no prompt named `my-prompt`
- **THEN** langstar reports that it would create the prompt and creates nothing

### Requirement: Read the prompt back after import

After writing, `langstar prompt import` SHALL record the commit hash in the destination record, then read the destination commit back with the model included and compare it with the file. When they differ it SHALL list the differences and exit with status 3, and the record SHALL keep the hash, so that the next import reports a change only when someone pushed a commit after the one langstar wrote.

#### Scenario: LangSmith drops a field

- **WHEN** the destination commit read back after import lacks a field the file holds
- **THEN** langstar lists that field as a difference and exits with status 3
- **AND** the destination record holds the new commit hash

### Requirement: Compare a prompt with its export file

`langstar prompt compare <file>` SHALL read the prompt with the file's handle from the workspace, with its model and repo details, and list each difference by its JSON path with both values. `--commit <hash-or-tag>` SHALL choose the commit, and without it langstar SHALL read the latest. It SHALL compare every field in the file's `repo` and `commit`, including every manifest key and its `id` arrays. Those hold no IDs or timestamps LangSmith sets, so compare never reports them.

#### Scenario: Prompt matches

- **WHEN** the workspace's prompt matches the file
- **THEN** langstar reports no differences and exits with status 0

#### Scenario: Model temperature changed

- **WHEN** someone changed the model temperature in the workspace's prompt
- **THEN** langstar lists the temperature's JSON path with both values and exits with status 3

#### Scenario: Model class changed

- **WHEN** someone changed the workspace's prompt to use `ChatAnthropic` where the file's manifest names `ChatOpenAI`
- **THEN** langstar lists the JSON path of the model's `id` array with both values and exits with status 3

#### Scenario: Repo readme changed

- **WHEN** someone changed the readme of the workspace's prompt
- **THEN** langstar lists the readme's JSON path with both values and exits with status 3

### Requirement: Verify that a prompt can run in a workspace

`langstar prompt verify <file>` SHALL do what `compare` does and also check that the workspace has every secret in `requires.secrets`. It SHALL name each missing secret and exit with status 3 when any secret is missing. It SHALL also look for a saved model configuration whose settings `id` ends in the class name `requires.model_provider` holds, and report a missing one as a note that leaves the exit status unchanged, because the prompt runs with the model its manifest carries.

#### Scenario: Secret missing in the destination

- **WHEN** the destination workspace has no secret named `OPENAI_API_KEY`
- **AND** the file lists `OPENAI_API_KEY` under `requires.secrets`
- **THEN** langstar reports `OPENAI_API_KEY` as missing and exits with status 3

#### Scenario: No saved model configuration for the provider

- **WHEN** the file's `requires.model_provider` is `ChatOpenAI`
- **AND** no saved model configuration in the destination workspace has a settings `id` ending in `ChatOpenAI`
- **AND** every secret in `requires.secrets` exists and the prompt matches the file
- **THEN** langstar notes that no saved model configuration matches `ChatOpenAI` and exits with status 0

### Requirement: Reject an export file of another kind or format version

`prompt import`, `prompt compare` and `prompt verify` SHALL read a file only when its `kind` is `prompt` and its `format_version` is 1. For any other file they SHALL write nothing, exit with status 1 and name the field and the value they found.

#### Scenario: File from a newer format

- **WHEN** a user runs `langstar prompt compare my-prompt.json`
- **AND** `my-prompt.json` holds `"format_version": 2`
- **THEN** langstar exits with status 1 and an error naming `format_version` and the value 2

#### Scenario: Assistant file given to a prompt command

- **WHEN** a user runs `langstar prompt import support-bot.json --workspace-id <workspace-id>`
- **AND** `support-bot.json` holds `"kind": "assistant"`
- **THEN** langstar creates no prompt, exits with status 1 and names `kind` and the value `assistant`

### Requirement: Exit with 0 for a match, 3 for a difference and 1 for an error

`prompt compare`, `prompt verify` and `prompt import` SHALL exit with status 0 when the prompt matches and every check passes, 3 when they find a difference or a failed check, and 1 on any other error. A usage error that clap rejects, such as a mistyped flag, SHALL keep clap's status 2. With `-f json` each SHALL print one JSON object with `status`, `differences` and `checks`.

#### Scenario: Machine-readable compare

- **WHEN** a user runs `langstar -f json prompt compare my-prompt.json`
- **THEN** stdout holds one JSON object with `status`, `differences` and `checks`
- **AND** progress messages go to stderr

#### Scenario: Mistyped flag

- **WHEN** a user runs `langstar prompt compare my-prompt.json --wrkspace-id <workspace-id>`
- **THEN** langstar prints clap's usage error and exits with status 2
