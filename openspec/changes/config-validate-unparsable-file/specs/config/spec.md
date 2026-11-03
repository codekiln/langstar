## ADDED Requirements

### Requirement: `config validate` names a config file it cannot look up, read or parse

When langstar cannot look up the config file, cannot read it or cannot parse it, `langstar config validate` SHALL exit with an error that names the file and says why, before checking any setting. When the file does not parse, it SHALL also print the line of the file that the parse error points at.

#### Scenario: Unparsable config file

- **WHEN** the config file holds `output_format = ` with no value
- **AND** a user runs `langstar config validate`
- **THEN** langstar exits with an error that names the config file and the TOML parse error

#### Scenario: Config file that links to itself

- **WHEN** `config.toml` is a symbolic link to itself
- **AND** a user runs `langstar config validate`
- **THEN** langstar exits with an error that names the config file and says langstar could not look it up

### Requirement: `config <setting> set` names a config file it cannot look up, read or parse, and leaves it unchanged

When langstar cannot look up the config file, cannot read it or cannot parse it, `langstar config <setting> set` SHALL exit with an error that names the file and says why, and leave the file unchanged. When the file does not parse, it SHALL also print the line of the file that the parse error points at.

#### Scenario: Setting a value in an unparsable config file

- **WHEN** the config file does not parse
- **AND** a user runs `langstar config output_format set json`
- **THEN** langstar exits with the parse error and leaves the config file unchanged

#### Scenario: Setting a value when the config file links to itself

- **WHEN** `config.toml` is a symbolic link to itself
- **AND** a user runs `langstar config timezone set UTC`
- **THEN** langstar exits with an error that names the config file and says langstar could not look it up

#### Scenario: Setting a value when the config file cannot be read

- **WHEN** the config path is a directory
- **AND** a user runs `langstar config timezone set UTC`
- **THEN** langstar exits with an error that names the config file and says langstar could not read it

### Requirement: `config <setting> set` refuses while another setting holds a value of the wrong type

`langstar config <setting> set` SHALL exit with the parse error and leave the file unchanged when a different setting in the file holds a value of the wrong type, such as `timezone = 5`. It SHALL replace a wrong-type value of the setting it sets.

#### Scenario: Setting a value that replaces a value of the wrong type

- **WHEN** the config file sets `output_format = 5`
- **AND** a user runs `langstar config output_format set json`
- **THEN** langstar replaces the value with `output_format = "json"` and exits successfully

#### Scenario: Setting a value while another setting holds a value of the wrong type

- **WHEN** the config file sets `timezone = 5`
- **AND** a user runs `langstar config output_format set json`
- **THEN** langstar exits with the parse error and leaves the config file unchanged

### Requirement: Warn on stderr and use the defaults when a command that loads the settings cannot look up, read or parse the config file

A langstar command that loads the settings SHALL warn on stderr and run on the defaults and the environment variables when langstar cannot look up the config file, cannot read it or cannot parse it. The warning gives the file's path and the error message, with the line and column of a parse error, and leaves out the quoted line of the file and the value the message quotes, because either can hold the API key.

#### Scenario: `config show` warns and shows the defaults when the config file does not parse

- **WHEN** the config file does not parse
- **AND** a user runs `langstar config show`
- **THEN** langstar warns on stderr with the file's path and the line, column and message of the parse error
- **AND** the warning says langstar is using the default settings
- **AND** the warning leaves out the quoted line of the file and the value the message quotes
- **AND** langstar prints the default settings and exits successfully

### Requirement: Every command outside the `config` group, `config show` and `config <setting>` load the settings

Every command outside the `config` group SHALL load the settings, and so SHALL `langstar config show` and `langstar config <setting>` with no subcommand. `langstar config show` warns and still prints the config file's path. `langstar config create` and `langstar config env` never load the settings and print no warning.

#### Scenario: `config show` when the config file links to itself

- **WHEN** `config.toml` is a symbolic link to itself
- **AND** a user runs `langstar config show`
- **THEN** langstar prints `File exists: unknown (langstar could not check; see the warning above)` and exits successfully

### Requirement: Exit with an error when a command that loads the settings cannot determine where the config file belongs

A langstar command that loads the settings SHALL exit with an error when langstar cannot determine where the config file belongs. That happens when `LANGSTAR_CONFIG_DIR` is unset and the operating system reports no home directory on macOS and Linux, or no config directory on Windows.

#### Scenario: `config show` fails on macOS and Linux when langstar cannot determine the home directory

- **WHEN** `LANGSTAR_CONFIG_DIR` is unset
- **AND** langstar cannot determine the home directory
- **AND** a user runs `langstar config show`
- **THEN** langstar exits with an error that says langstar cannot find the home directory and says to set `HOME` or `LANGSTAR_CONFIG_DIR`
