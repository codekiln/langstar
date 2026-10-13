# model-config Specification

## Purpose

`langstar model-config` manages LangSmith model configurations, which the LangSmith API calls playground settings.

## Requirements

### Requirement: List model configurations

`langstar model-config list` SHALL list the workspace's model configurations, taking `--limit` (default 20) and `--offset` (default 0). In table format it SHALL show each configuration's ID, name, provider and model. In JSON format it SHALL print the configurations as the LangSmith API returns them. When there are none, it SHALL say that no model configurations were found.

#### Scenario: Page through configurations

- **WHEN** a user runs `langstar model-config list --limit 10 --offset 10`
- **THEN** langstar shows at most 10 configurations, starting after the first 10

#### Scenario: Provider and model from the settings

- **WHEN** a configuration's settings name the `anthropic` chat model class and the model `<model-name>`
- **THEN** its row shows `anthropic` as the provider and `<model-name>` as the model

### Requirement: Choose the columns of text output

In text format, `langstar model-config list` SHALL print one tab-separated line per configuration. `--columns` SHALL choose the columns from `id`, `name`, `provider` and `model`; without it, every column appears. `--show-columns` SHALL print the available column names. An unknown column name SHALL end the command with an error that lists the available columns.

#### Scenario: Choose text columns

- **WHEN** a user runs `langstar model-config list -f text --columns id,model`
- **THEN** each output line holds a configuration's ID and model, separated by a tab

### Requirement: Get one model configuration

`langstar model-config get <id>` SHALL show the model configuration with that ID, in the same table and JSON output as `model-config list`. It SHALL look for the ID among the first 5,000 configurations in the workspace, and when none of them has the ID, it SHALL exit with an error that names the ID.

#### Scenario: Unknown ID

- **WHEN** a user runs `langstar model-config get <config-id>` and none of the first 5,000 configurations has that ID
- **THEN** langstar exits with an error saying the model configuration was not found

### Requirement: Create a model configuration from a file

`langstar model-config create --file <path>` SHALL create a model configuration from the JSON request in the file, print it, and report the new configuration's ID.

#### Scenario: Create from a file

- **WHEN** a user runs `langstar model-config create --file config.json`
- **THEN** langstar creates the configuration described in `config.json`
- **AND** prints the new configuration's ID

### Requirement: Update a model configuration

`langstar model-config update <id>` SHALL update the configuration from a JSON request in `--file`, or change only its name and description through `--name` and `--description`. It SHALL reject `--file` combined with `--name` or `--description`, and SHALL exit with an error when the user passes none of the three.

#### Scenario: Rename a configuration

- **WHEN** a user runs `langstar model-config update <config-id> --name <new-name>`
- **THEN** the configuration's name becomes `<new-name>` and langstar reports its ID

#### Scenario: Nothing to update

- **WHEN** a user runs `langstar model-config update <config-id>` with no other flag
- **THEN** langstar exits with an error asking for `--file`, `--name` or `--description`

### Requirement: Delete a model configuration after confirmation

`langstar model-config delete <id>` SHALL ask the user to confirm with `y` or `yes` before deleting the configuration, and SHALL skip the question when the user passes `-y` or `--yes`. When langstar would ask the user to confirm but standard input is not a terminal, as when a script runs the command, it SHALL leave the configuration in place and exit with an error that says to pass `--yes`.

#### Scenario: User declines

- **WHEN** a user runs `langstar model-config delete <config-id>` and answers anything other than `y` or `yes`
- **THEN** langstar prints that it cancelled and leaves the configuration in place

#### Scenario: Standard input is not a terminal

- **WHEN** a user runs `langstar model-config delete <config-id>` with standard input that is not a terminal and without `--yes`
- **THEN** langstar exits with an error saying to pass `--yes`, and deletes nothing
