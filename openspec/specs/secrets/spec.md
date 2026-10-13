# secrets Specification

## Purpose

`langstar secrets` manages the secrets of a LangSmith workspace through the LangSmith API.

## Requirements

### Requirement: Print secret keys only

Every `langstar secrets` subcommand SHALL print secret keys only. No subcommand SHALL print a secret's value, in any output format.

#### Scenario: Set a secret in JSON format

- **WHEN** a user runs `langstar secrets set <key> --from-env <var> -f json`
- **THEN** the output holds a `status`, the `key` and a `message`
- **AND** the output omits the secret's value

### Requirement: List secret keys

`langstar secrets list` SHALL list the keys of the workspace's secrets. In table format it SHALL show one key per row and report how many secrets it found. In JSON format it SHALL print the list of keys.

#### Scenario: List keys

- **WHEN** a user runs `langstar secrets list`
- **AND** the workspace holds secrets under the keys `<key-a>` and `<key-b>`
- **THEN** langstar shows `<key-a>` and `<key-b>` and reports 2 secrets

### Requirement: Set a secret from one value source

`langstar secrets set <key>` SHALL create the secret or replace its value. It SHALL read the value from exactly one source: a file through `--from-file`, an environment variable through `--from-env`, a masked prompt through `--interactive`, or standard input when the user passes none of these flags. It SHALL trim whitespace from a value read from a file or standard input.

#### Scenario: Value from standard input

- **WHEN** a user runs `printf '<your-secret-value>' | langstar secrets set <key>`
- **THEN** langstar stores `<your-secret-value>` under `<key>` and says the secret was set

#### Scenario: Two value sources

- **WHEN** a user passes more than one of `--from-file`, `--from-env` and `--interactive`
- **THEN** langstar exits with an argument error before calling the API

### Requirement: Reject a missing or empty value

`langstar secrets set` SHALL exit with an error and leave the workspace unchanged when the value is empty, when standard input holds nothing, or when the variable named by `--from-env` is unset.

#### Scenario: Unset environment variable

- **WHEN** a user runs `langstar secrets set <key> --from-env <var>` and `<var>` is unset
- **THEN** langstar exits with an error saying the environment variable was not found

#### Scenario: Empty standard input

- **WHEN** a user runs `langstar secrets set <key>` with nothing on standard input
- **THEN** langstar exits with an error that lists the value sources it accepts

### Requirement: Delete a secret

`langstar secrets delete <key>` SHALL delete the secret stored under the key, without asking for confirmation, and say that it deleted the secret.

#### Scenario: Delete a secret

- **WHEN** a user runs `langstar secrets delete <key>`
- **THEN** the workspace no longer holds a secret under `<key>`
- **AND** langstar says the secret `<key>` was deleted
