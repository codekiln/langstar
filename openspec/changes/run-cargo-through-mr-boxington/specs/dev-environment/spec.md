# Spec Delta

## Purpose

`dev-environment` covers how a langstar developer and langstar's CI build the workspace: which tools mise installs and how Cargo's compiled work is cached and reused.

## ADDED Requirements

### Requirement: Cargo started through mise runs through Mr Boxington

The `rust` entry in `mise.toml` SHALL set `mr_boxington = true`, so that Cargo started through mise (a `mise run` task, `mise exec -- cargo`, a mise shim or a shell with mise activated) runs through `mbx`.

#### Scenario: Cargo under mise exec prints an mbx summary

- **WHEN** a developer runs `mise exec -- cargo clippy --workspace` in a langstar checkout
- **THEN** after Cargo's own output, mbx prints a line starting `mbx[cache]:` that counts cache hits and misses

#### Scenario: A second worktree reuses compiled work

- **WHEN** a developer has built langstar with `mise exec -- cargo build` in one worktree
- **AND** runs the same command in a new worktree of the same commit
- **THEN** mbx restores compilations from its cache instead of compiling them again
- **AND** `target` in the new worktree is a symlink into mbx's cache directory

### Requirement: mise pins Mr Boxington to an exact release at least five days old

`mise.toml` SHALL pin `mr-boxington` to an exact version. That version SHALL have been released at least as long ago as the `minimum_release_age` in `mise.toml`'s `[settings]` when it was pinned, because mise does not apply that setting to an exact pin.

#### Scenario: mise installs the pinned release

- **WHEN** a developer runs `mise install` in a langstar checkout
- **THEN** mise installs `mr-boxington` 1.28.0
- **AND** `mise exec -- mbx --version` prints 1.28.0

### Requirement: CI and release workflows compile with plain Cargo

The GitHub Actions workflows SHALL run plain `cargo`, not `mbx`, and SHALL cache Cargo's target directory and downloads with `Swatinem/rust-cache`. The release workflow SHALL NOT restore compiler outputs through `jdx/mr-boxington-action` or a remote mbx cache.

#### Scenario: A CI job compiles with plain cargo

- **WHEN** CI runs the Test job on a pull request
- **THEN** the job restores its entry with `Swatinem/rust-cache`
- **AND** it runs `cargo nextest run`, not `mbx nextest run`

#### Scenario: The release build compiles with plain cargo

- **WHEN** the release workflow builds a binary for a GitHub release
- **THEN** it runs plain `cargo build`, not `mbx build`
