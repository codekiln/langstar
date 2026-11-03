# Report a config file langstar cannot look up, read or parse

Issue: [#810 🐛 fix(config): make config validate fail on a config file that does not parse](https://github.com/codekiln/langstar/issues/810)

## Why

When a user's `config.toml` does not parse, most langstar commands run on the default settings without telling them. Only `langstar config validate` and `langstar config <setting> set` stop with an error, and no test checks that `config validate` does.

## What Changes

- `langstar config validate` exits with an error that names the config file and says why when langstar cannot look it up, read it or parse it, before checking any setting. For a parse error it also prints the line the error points at. langstar cannot look up a file when it cannot even check whether the file exists, as when `config.toml` is a symbolic link to itself.
- `langstar config <setting> set` exits with the same error and leaves the file unchanged. It also refuses when a different setting in the file holds a value of the wrong type, such as `timezone = 5`.
- Every other command that loads the settings warns on stderr with the file's path and the error, then runs on the defaults and the environment variables. The warning leaves out the quoted line and value, because either can hold the API key.
- `langstar config show` prints `File exists: unknown` when langstar could not look the file up.
- A command that loads the settings exits with an error when langstar cannot determine where the config file belongs, which happens when `LANGSTAR_CONFIG_DIR` is unset and the system reports no home directory.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `config`: adds requirements for a config file langstar cannot look up, read or parse, and for a missing home directory.

## Impact

- `cli/src/config.rs`: `Config::load` warns on stderr when it cannot look up, read or parse the config file, and `config validate`, `config <setting> set` and `config show` report the error.
- `cli/tests/config_command_test.rs`: integration tests for each scenario.
- `docs/configuration.md`: a troubleshooting entry for "Warning: langstar is using the default settings".

## Citations

- [My/Principle/Explicit is better than Implicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Explicit%20is%20better%20than%20Implicit.md)
