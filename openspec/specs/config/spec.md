# config Specification

## Purpose

`langstar config` shows, creates, checks and changes langstar's own settings, which live in a config file and in environment variables.

## Requirements

### Requirement: Settings come from environment variables, then the config file, then defaults

Langstar SHALL read its settings from `~/.config/langstar/config.toml` on macOS and Linux, and from `langstar\config.toml` in the platform config folder on Windows. An environment variable SHALL take precedence over the same setting in the config file, and a built-in default SHALL apply when neither sets it.

#### Scenario: Environment variable wins

- **WHEN** the config file sets `output_format = "table"`
- **AND** `LANGSTAR_OUTPUT_FORMAT=json` is set in the environment
- **THEN** langstar prints its command output as JSON

#### Scenario: Default output format

- **WHEN** neither the config file nor the environment sets the output format
- **THEN** langstar prints its command output as tables

### Requirement: Each setting has one environment variable

`langstar config env` SHALL print every config file key next to the environment variable that sets it, for example `langsmith_api_key` next to `LANGSMITH_API_KEY`.

#### Scenario: Show the mapping

- **WHEN** a user runs `langstar config env`
- **THEN** langstar prints a table of config file keys, environment variables and descriptions
- **AND** notes that environment variables take precedence over the config file

### Requirement: Show the current settings and where each came from

`langstar config show` SHALL print the config file's path, whether the file exists, and each setting's current value followed by its source. It SHALL label a value `(from env: <VARIABLE>)` when the environment variable holds the same text that langstar prints for that setting, and `(from config file or default)` otherwise. For the API key, it SHALL compare the whole key and print only the first 10 characters followed by `...`.

#### Scenario: API key from the environment

- **WHEN** `LANGSMITH_API_KEY` is set to `<your-api-key>`
- **AND** a user runs `langstar config show`
- **THEN** langstar shows the first 10 characters of `<your-api-key>` followed by `...` and `(from env: LANGSMITH_API_KEY)`

#### Scenario: Timezone set to `local` in the environment

- **WHEN** `LANGSTAR_TIMEZONE` is set to `local`
- **AND** a user runs `langstar config show`
- **THEN** langstar shows the timezone as `local (system timezone)` followed by `(from config file or default)`

### Requirement: Name the active scope

`langstar config show` SHALL name the active scope: the workspace when a workspace ID is set, otherwise the organization when an organization ID is set, otherwise none.

#### Scenario: Workspace scope

- **WHEN** both a workspace ID and an organization ID are set
- **THEN** `langstar config show` names the workspace as the active scope

### Requirement: Warn when both organization and workspace IDs are set

When both an organization ID and a workspace ID are set, every langstar command SHALL warn on standard error that the workspace ID takes precedence, and say how to hide the warning. Setting `hide_workspace_and_org_id_message` to true, or `LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE` to `1` or `true`, SHALL hide it.

#### Scenario: Hide the warning

- **WHEN** both IDs are set and `LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE=1`
- **THEN** langstar runs the command without the warning

### Requirement: Create a config file

`langstar config create` SHALL write a config file at the config file path, holding every setting with comments that name its environment variable, and SHALL create the parent folder when it is missing. On Unix it SHALL let only the owner read and write the file. When the file already exists, it SHALL exit with an error unless the user passes `--force`, which replaces the file.

#### Scenario: File already exists

- **WHEN** a config file exists and a user runs `langstar config create`
- **THEN** langstar exits with an error that names the file and suggests `--force`
- **AND** the file stays as it was

### Requirement: Check the settings langstar would use

When the config file exists, `langstar config validate` SHALL load the settings the way every langstar command loads them, from environment variables, the config file and defaults, and print a summary. It SHALL exit with an error when `output_format` is something other than `json` or `table`, or when `timezone` is not a timezone langstar understands. It SHALL warn when no API key is set.

#### Scenario: Invalid timezone

- **WHEN** the config file sets `timezone = "<not-a-timezone>"`
- **AND** a user runs `langstar config validate`
- **THEN** langstar exits with an error that names the invalid timezone

#### Scenario: Invalid output format from the environment

- **WHEN** the config file sets `output_format = "table"`
- **AND** `LANGSTAR_OUTPUT_FORMAT=text` is set in the environment
- **AND** a user runs `langstar config validate`
- **THEN** langstar exits with an error that names the invalid output format

### Requirement: Stop checking when the config file is missing

When the config file is missing, `langstar config validate` SHALL print the config file's path, suggest `langstar config create` and exit successfully, without checking any setting, including the ones set in the environment.

#### Scenario: No config file and an invalid output format in the environment

- **WHEN** no config file exists
- **AND** `LANGSTAR_OUTPUT_FORMAT=text` is set in the environment
- **AND** a user runs `langstar config validate`
- **THEN** langstar says the config file does not exist, suggests `langstar config create` and exits successfully

### Requirement: Change one setting from the command line

`langstar config <setting> set <value>` SHALL check the value and write it to the config file, for the settings `output_format`, `timezone` and `hide_workspace_and_org_id_message`. Without `set`, the command SHALL print the setting's description, current value and example commands.

#### Scenario: Set the timezone

- **WHEN** a user runs `langstar config timezone set America/New_York`
- **THEN** the config file's `timezone` becomes `America/New_York`

#### Scenario: Show help for a setting

- **WHEN** a user runs `langstar config timezone`
- **THEN** langstar prints the timezone setting's description, current value and example `set` commands

### Requirement: Accept only valid setting values

`langstar config output_format set` SHALL accept `json` or `table`. `langstar config timezone set` SHALL accept an IANA timezone name, `local` or `UTC`. `langstar config hide_workspace_and_org_id_message set` SHALL accept `true`, `false`, `1`, `0`, `yes`, `no`, `on` or `off`. Any other value SHALL end the command with an error and leave the config file unchanged.

#### Scenario: Reject an invalid value

- **WHEN** a user runs `langstar config output_format set yaml`
- **THEN** langstar exits with an error and leaves the config file unchanged
