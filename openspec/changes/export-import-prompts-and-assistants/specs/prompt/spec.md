## Purpose

`langstar prompt` manages the prompts saved in a LangSmith workspace, including writing a prompt to a file and recreating it in another workspace.

## ADDED Requirements

### Requirement: Export a prompt commit to a file

`langstar prompt export <handle>` SHALL write one JSON file holding the repo details (handle, description, readme, tags, visibility), the commit's description, its manifest as LangSmith returns it with the model included, its model configuration and provider, the tags that point at it, and a `source` object naming the workspace, handle and commit hash. `--output <path>` SHALL choose the file; without it stdout SHALL hold only the file, and messages go to stderr.

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

#### Scenario: Export to stdout

- **WHEN** a user runs `langstar prompt export my-prompt` without `--output`
- **THEN** stdout holds only the export file's JSON document
- **AND** any progress message goes to stderr

### Requirement: Export a chosen commit or tag

`langstar prompt export` SHALL accept `--commit <hash-or-tag>` and export that commit. Without the flag it SHALL export the latest commit.

#### Scenario: Export a tagged commit

- **WHEN** a user runs `langstar prompt export my-prompt --commit prod --output my-prompt.json`
- **THEN** the file holds the commit the `prod` tag points at

### Requirement: Write the same file for the same commit

Exporting the same commit twice SHALL produce identical files when nobody changed the repo details or the tags that point at the commit in between, since those change without a new commit. The file SHALL hold no export time, and langstar SHALL write object keys in a fixed order and sort `commit.tags`, `repo.tags` and `requires.secrets` by name. Compare SHALL sort the destination's tags the same way before comparing.

#### Scenario: Export twice

- **WHEN** a user exports the same commit twice to two files
- **AND** nobody changed the repo details or the commit's tags between the two exports
- **THEN** the two files are identical

### Requirement: List the credentials a prompt needs without their values

The export file SHALL list, under `requires.secrets`, the name of every secret the manifest refers to, and under `requires.model_class`, the class name that ends the chat model constructor's `id`, such as `ChatOpenAI`. It SHALL hold no value of a secret the manifest refers to. Import and verify SHALL read both again from the file's manifest, and when either differs from `requires.secrets` or `requires.model_class`, write nothing, name the field and exit with status 1.

#### Scenario: Model reads an API key secret

- **WHEN** the prompt's model refers to the secret `OPENAI_API_KEY`
- **THEN** `requires.secrets` holds `OPENAI_API_KEY`
- **AND** no value of that secret appears in the file

#### Scenario: Manifest edited without its requires list

- **WHEN** someone edits the file's manifest so its model refers to the secret `ANTHROPIC_API_KEY`, and leaves `requires.secrets` holding only `OPENAI_API_KEY`
- **AND** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **THEN** langstar creates no prompt, names `requires.secrets` and exits with status 1

### Requirement: Refuse to export a prompt that holds a credential

Before it writes, `langstar prompt export` SHALL run the credential check over every string in the `repo` and `commit` objects it would write, skipping LangChain secret references and empty strings. When the check flags a string, export SHALL write no file and nothing to stdout, list the JSON Pointer path of each flagged string and the rule that flagged it, without the string or any part of it, and exit with status 1. Export SHALL NOT write a file with `<redacted>` in place of a flagged string.

#### Scenario: Model holds a literal API key

- **WHEN** the prompt's `ChatOpenAI` constructor holds a string under `openai_api_key`
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes no file, names the path of `openai_api_key` without its value and exits with status 1

#### Scenario: Model holds a literal API key under an upper-case name

- **WHEN** the prompt's `ChatOpenAI` constructor holds a string under `OPENAI_API_KEY`
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes no file, names the path of `OPENAI_API_KEY` without its value and exits with status 1

#### Scenario: Readme holds a private key

- **WHEN** the repo's readme holds a PEM block that begins `-----BEGIN PRIVATE KEY-----`
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes no file, names `/repo/readme` and exits with status 1

#### Scenario: Model reads its key from a secret

- **WHEN** the prompt's chat model refers to the secret `OPENAI_API_KEY` under `openai_api_key`
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes the file with the secret reference and exits with status 0

### Requirement: Flag a prompt string by the name of its key

The credential check SHALL flag a string when the key that holds it, or the array it is in, has a name whose words, split at `_`, `-`, `.`, spaces and lower-to-upper case changes and compared without regard to case, include `secret`, `token`, `password`, `passwd`, `pwd`, `credential`, `credentials`, `apikey`, `authorization` or `cookie`, or `key` or `keys` right after `api`, `access`, `private`, `secret`, `signing` or `account`.

#### Scenario: Model sends an authorization header

- **WHEN** the prompt's chat model holds `default_headers` with `Authorization` set to `Bearer ` and a token
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes no file, names `/commit/manifest/kwargs/last/kwargs/default_headers/Authorization` without its value and exits with status 1

#### Scenario: Model sets a token limit

- **WHEN** the prompt's chat model holds the number 1024 under `max_tokens`
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes the file with `max_tokens` unchanged and exits with status 0

### Requirement: Flag a prompt string by its shape under any key

The credential check SHALL flag a string, under any key, that holds a bearer token, a `Basic ` authorization value, a PEM private key, a URL with a password in its user info, a connection string with a non-empty `Password=`, `Pwd=`, `AccountKey=`, `SharedAccessKey=` or `SharedAccessSignature=`, a JSON Web Token, or a token that starts with a provider prefix the design lists: `sk-`, `lsv2_`, `ghp_`, `gho_`, `ghs_`, `github_pat_`, `xoxb-`, `xoxp-`, `xoxa-`, `AKIA`, `ASIA`, `AIza` or `hf_`.

#### Scenario: Model setting holds a token under a neutral name

- **WHEN** the prompt's chat model holds `sk-` followed by a 40-character token under `model_kwargs.extra`
- **AND** a user runs `langstar prompt export my-prompt`
- **THEN** stdout is empty, langstar names the path of `extra` with the rule that flagged it and exits with status 1

#### Scenario: Prompt metadata holds a connection string with a password

- **WHEN** the manifest's `metadata` holds `postgresql://app:<password>@db.example.com/prod` under `source_db`
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json`
- **THEN** langstar writes no file, names the path of `source_db` as a URL with a password and exits with status 1

### Requirement: Write each flagged prompt string the user names with --not-a-secret

`langstar prompt export` SHALL take `--not-a-secret <json-pointer>` any number of times. For each pointer at a string the credential check flags, export SHALL write that string and print a warning to stderr naming the pointer. A flagged string at any path not named SHALL still make export refuse. When a pointer names a string the check lets through, export SHALL write nothing, name the pointer and exit with status 1. Export SHALL run the check on every run.

#### Scenario: Message template shows an example token

- **WHEN** a message template holds `Authorization: Bearer ` followed by an example token
- **AND** a user runs `langstar prompt export my-prompt --output my-prompt.json --not-a-secret <path of that template>`
- **THEN** langstar writes the file with the template unchanged, prints a warning naming the path to stderr and exits with status 0

#### Scenario: User names an unflagged string with --not-a-secret

- **WHEN** a user runs `langstar prompt export my-prompt --output my-prompt.json --not-a-secret /repo/description`
- **AND** the check flags nothing at `/repo/description`
- **THEN** langstar writes no file, names `/repo/description` and exits with status 1

### Requirement: Export, import and verify a prompt without a model

When the manifest holds no chat model constructor, `langstar prompt export` SHALL write `requires.model_class` as null. Import and verify SHALL accept null when the manifest they read again holds no chat model, skip the saved model configuration check, report no note for it, and still run the secret check.

#### Scenario: Prompt saved without a model

- **WHEN** a user exports a prompt saved without a model and imports the file with `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **THEN** the file holds null under `requires.model_class`
- **AND** langstar creates the prompt without a saved model configuration check and exits with status 0
- **AND** `langstar prompt verify my-prompt.json` against the destination reports no saved model configuration note and exits with status 0

### Requirement: Import requires --workspace-id

`langstar prompt import <file>` SHALL require `--workspace-id`. It SHALL refuse to run when the flag is missing, even when `LANGSMITH_WORKSPACE_ID` is set, and exit with status 1 and an error naming the flag.

#### Scenario: Missing destination workspace

- **WHEN** a user runs `langstar prompt import my-prompt.json` without `--workspace-id`
- **AND** `LANGSMITH_WORKSPACE_ID` is set
- **THEN** langstar exits with status 1 and an error naming `--workspace-id`

### Requirement: Check the destination before importing a prompt

Before it writes, `langstar prompt import` SHALL run the secret check and the saved model configuration check of `prompt verify` against the destination workspace. When the workspace lacks a secret in `requires.secrets` it SHALL write nothing, name each missing secret and exit with status 3. A missing saved model configuration SHALL be a note, as in `prompt verify`.

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

### Requirement: Finish an import that stopped before its first commit

When the destination workspace has a prompt with the file's handle and no commit, `langstar prompt import` SHALL push the file's manifest and commit description as the first commit and create the file's commit tags, as for a prompt that does not exist yet, without `--update`. Repo details that differ from the file SHALL still need `--update`.

#### Scenario: Earlier import created the repo and stopped

- **WHEN** an earlier import created `my-prompt` in the destination and stopped before it pushed the commit
- **AND** the user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>` again
- **THEN** langstar pushes the file's manifest as the prompt's first commit and creates the file's tags
- **AND** exits with status 0

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

`prompt compare` and `prompt import` SHALL list as a difference a file tag the destination lacks or points at another commit, and a tag on the destination's matching commit that the file lacks; import SHALL then exit with status 3. With `--update`, import SHALL create or move each file tag to that commit, delete each tag on it the file lacks, and add no commit when the latest commit already matches.

#### Scenario: Destination tag points at an older commit

- **WHEN** the destination prompt's latest commit matches the file
- **AND** the destination's `prod` tag points at an earlier commit
- **AND** the user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id> --update`
- **THEN** langstar moves `prod` to the latest commit and adds no commit
- **AND** exits with status 0

#### Scenario: Destination commit has a tag the file lacks

- **WHEN** the destination prompt's latest commit matches the file
- **AND** that commit carries a `staging` tag that the file's `commit.tags` lacks
- **AND** the user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id> --update`
- **THEN** langstar deletes the destination's `staging` tag and adds no commit
- **AND** exits with status 0

### Requirement: Record the destination commit

After import writes a commit, langstar SHALL record the destination workspace, handle and commit hash in a destination record file, `<file stem>.destinations.json` next to the export unless `--record <path>` names another, before it reads the commit back. Each import SHALL add or replace only the entry for its own destination workspace.

#### Scenario: Import into two workspaces

- **WHEN** a user imports `my-prompt.json` into a staging workspace and then into a production workspace
- **THEN** `my-prompt.destinations.json` holds one entry for each workspace with its own commit hash
- **AND** `my-prompt.json` is unchanged

### Requirement: Replace the prompt destination record in one step

`langstar prompt import` SHALL write each new destination record to a temporary file in the record's directory and rename it over the old record, so that a langstar that stops during a write leaves the earlier record whole.

#### Scenario: Import stopped while writing the record

- **WHEN** `my-prompt.destinations.json` holds commit hashes for a staging and a production workspace
- **AND** langstar stops while it writes the record after an import into the production workspace
- **THEN** `my-prompt.destinations.json` still holds both workspaces' entries as they were before that import

### Requirement: Warn when the destination changed since the last import

When the destination record holds a commit for the destination workspace, and the destination's latest commit is not that commit and differs from the file, `langstar prompt import` SHALL say that someone changed the prompt after the last import, and list the differences.

#### Scenario: Someone edited the destination prompt

- **WHEN** a user edited the destination prompt in the web interface after the last import
- **AND** runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>`
- **THEN** langstar reports that the destination changed since the last recorded import and exits with status 3

### Requirement: Record a matching commit that an earlier import pushed

When the destination's latest commit matches the file, `langstar prompt import` without `--dry-run` SHALL write that commit's hash to the destination record under the destination workspace. When the commit tags match the file too, it SHALL report the prompt unchanged and exit with status 0.

#### Scenario: Earlier import stopped before it wrote the record

- **WHEN** an earlier import pushed a commit and stopped before it wrote that commit's hash to `my-prompt.destinations.json`
- **AND** the user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>` again
- **THEN** langstar reports that the prompt is unchanged, writes the latest commit's hash to the record and exits with status 0

#### Scenario: First import stopped before it wrote the record

- **WHEN** the first import into a workspace pushed the prompt and stopped before it wrote `my-prompt.destinations.json`
- **AND** the user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id>` again
- **THEN** langstar reports that the prompt is unchanged, writes the latest commit's hash to the record under that workspace and exits with status 0

### Requirement: Dry run an import

`langstar prompt import --dry-run` SHALL report whether it would create the prompt, add a commit, or leave it unchanged, list the differences, and change nothing in the workspace or the destination record.

#### Scenario: Dry run against a new workspace

- **WHEN** a user runs `langstar prompt import my-prompt.json --workspace-id <dest-workspace-id> --dry-run`
- **AND** the destination has no prompt named `my-prompt`
- **THEN** langstar reports that it would create the prompt and creates nothing

### Requirement: Read the prompt back after import

After writing, `langstar prompt import` SHALL record the commit hash in the destination record, then read the destination commit back with the model included and compare it with the file as `langstar prompt compare` does. When they differ it SHALL list the differences and exit with status 3, and the record SHALL keep the hash, so that the next import reports a change only when someone pushed a commit after the one langstar wrote.

#### Scenario: LangSmith drops a field

- **WHEN** the destination commit read back after import lacks a field the file holds
- **THEN** langstar lists that field as a difference and exits with status 3
- **AND** the destination record holds the new commit hash

### Requirement: Redact a credential when reporting how a prompt differs

When `prompt compare`, `prompt verify` or `prompt import` prints either side of a difference, in text or JSON, langstar SHALL print `<redacted>` in place of each string the credential check from "Refuse to export a prompt that holds a credential" flags. It SHALL redact the string when the difference is at the string or at an object that contains it, and whether the string came from the destination or from the file.

#### Scenario: Destination model holds a literal API key

- **WHEN** someone saved the destination prompt with an `openai_api_key` written as a string
- **AND** a user runs `langstar prompt compare my-prompt.json`
- **THEN** langstar reports the setting's path with `<redacted>` in place of the string and exits with status 3

#### Scenario: Destination adds a model with a literal API key

- **WHEN** the file holds a prompt saved without a model
- **AND** someone saved the destination prompt with a model whose `openai_api_key` is written as a string
- **AND** a user runs `langstar prompt compare my-prompt.json`
- **THEN** langstar reports the model's path with `<redacted>` in place of the string and exits with status 3

#### Scenario: Destination model sends a bearer token in a header

- **WHEN** someone saved the destination prompt with a model whose `default_headers.Authorization` holds `Bearer ` and a token
- **AND** a user runs `langstar prompt compare my-prompt.json -f json`
- **THEN** the text and the JSON report print `<redacted>` at that header's path, neither holds the token, and langstar exits with status 3

#### Scenario: Destination model holds a token under a neutral name

- **WHEN** someone saved the destination prompt with a model whose `model_kwargs.extra` holds a string that starts with `lsv2_`
- **AND** a user runs `langstar prompt compare my-prompt.json`
- **THEN** langstar reports that path with `<redacted>` in place of the string and exits with status 3

### Requirement: Print a secret reference in a destination prompt's model unchanged

When `prompt compare`, `prompt verify` or `prompt import` prints either side of a difference, in the text or the JSON report, langstar SHALL print a LangChain secret reference unchanged under any key, so the report names the secret. A secret reference looks like `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`.

#### Scenario: Destination adds a model that reads its key from a secret

- **WHEN** the file holds a prompt saved without a model
- **AND** someone saved the destination prompt with a model whose `openai_api_key` refers to the secret `OPENAI_API_KEY`
- **AND** a user runs `langstar prompt compare my-prompt.json`
- **THEN** langstar reports the model's path with the secret reference and `OPENAI_API_KEY` in it, and exits with status 3

### Requirement: Compare a prompt with its export file

`langstar prompt compare <file>` SHALL read the prompt with the file's handle from the workspace, with its model and repo details, and list each difference by its JSON path with both values. `--commit <hash-or-tag>` SHALL choose the commit, and without it the latest. It SHALL compare every field in the file's `repo` and `commit`, including every manifest key and its `id` arrays, except `commit.model_config` and `commit.model_provider`, which import cannot set.

#### Scenario: Prompt matches

- **WHEN** the workspace's prompt matches the file
- **THEN** langstar reports no differences and exits with status 0

#### Scenario: Model temperature changed

- **WHEN** someone changed the model temperature in the workspace's prompt
- **THEN** langstar lists the temperature's JSON path with both values and exits with status 3

#### Scenario: Model class changed

- **WHEN** someone changed the workspace's prompt to use `ChatAnthropic` where the file's manifest names `ChatOpenAI`
- **THEN** langstar lists the JSON path of the model's `id` array with both values and exits with status 3

#### Scenario: Destination fills the model configuration differently

- **WHEN** the workspace's prompt has the file's manifest and a `model_config` or `model_provider` that differs from the file's
- **THEN** langstar reports no differences and exits with status 0

#### Scenario: Repo readme changed

- **WHEN** someone changed the readme of the workspace's prompt
- **THEN** langstar lists the readme's JSON path with both values and exits with status 3

#### Scenario: Prompt missing from the workspace

- **WHEN** a user runs `langstar prompt compare my-prompt.json`
- **AND** the workspace has no prompt named `my-prompt`
- **THEN** langstar exits with status 1 and an error naming the handle

### Requirement: Verify that a prompt can run in a workspace

`langstar prompt verify <file>` SHALL do what `compare` does and also check that the workspace has every secret in `requires.secrets`. It SHALL name each missing secret and exit with status 3 when any secret is missing. When `requires.model_class` is not null, it SHALL also look for a saved model configuration whose settings `id` ends in that class name, and report a missing one as a note that leaves the exit status unchanged, because the prompt runs with the model its manifest carries.

#### Scenario: Secret missing in the destination

- **WHEN** the destination workspace has no secret named `OPENAI_API_KEY`
- **AND** the file lists `OPENAI_API_KEY` under `requires.secrets`
- **THEN** langstar reports `OPENAI_API_KEY` as missing and exits with status 3

#### Scenario: No saved model configuration for the model class

- **WHEN** the file's `requires.model_class` is `ChatOpenAI`
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
