# Report a config file langstar cannot look up, read or parse

Issue: [#810 🐛 fix(config): make config validate fail on a config file that does not parse](https://github.com/codekiln/langstar/issues/810)

## Why

`langstar config validate` passed a `config.toml` that does not parse: loading the settings swallowed the parse error and fell back to the defaults, so a user saw a clean report for a broken file. Every other command that loads the settings ran on the defaults the same way, without saying so.

## What Changes

- `langstar config validate` exits with an error that names the config file and says why when langstar cannot look it up, read it or parse it, before checking any setting. For a parse error it also prints the line the error points at.
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

- `cli/src/config.rs`: loading the settings returns the lookup, read and parse errors instead of falling back silently, and `config validate`, `config <setting> set` and `config show` report them.
- `cli/tests/config_command_test.rs`: integration tests for each scenario.
- `docs/configuration.md`: a troubleshooting entry for "Warning: langstar is using the default settings".

## Citations

- [My/Principle/Explicit is better than Implicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Explicit%20is%20better%20than%20Implicit.md)
