# Tasks

## 1. Report the config file's errors

- [x] 1.1 Make loading the settings in `cli/src/config.rs` return the lookup, read and parse errors, and have `langstar config validate` and `langstar config <setting> set` exit with an error that names the file, before checking a setting or writing the file
- [x] 1.2 Have `langstar config <setting> set` refuse while a different setting holds a value of the wrong type, and replace a wrong-type value of the setting it sets
- [x] 1.3 Have every other command that loads the settings warn on stderr with the path and the error, without the quoted line or value, and run on the defaults; have `langstar config show` print `File exists: unknown` when langstar could not look the file up
- [x] 1.4 Exit with an error when langstar cannot determine where the config file belongs

## 2. Tests

- [x] 2.1 Unit tests in `cli/src/config.rs`, including `test_load_fails_without_a_home_or_config_directory_even_with_settings_in_env` and `test_load_names_a_config_file_it_cannot_look_up`
- [x] 2.2 Integration tests in `cli/tests/config_command_test.rs` for each scenario in the delta spec, including `test_config_validate_fails_on_an_unparsable_file`, `test_config_set_fails_while_another_setting_holds_a_value_of_the_wrong_type` and `test_config_show_prints_unknown_for_a_config_file_it_cannot_look_up`

## 3. Docs

- [x] 3.1 Add the troubleshooting entry "Warning: langstar is using the default settings" to `docs/configuration.md`

## 4. Archive

- [ ] 4.1 After this change merges, archive it so its requirements sync into `openspec/specs/config/spec.md`, running `openspec validate config-validate-unparsable-file --strict` first
