---
name: test-runner-worktree
description: Run the Rust checks and tests in a git worktree with `mise run check`, which supplies the LangSmith credentials through fnox. Use before committing, when tests fail with authentication errors or missing LANGSMITH_* variables, or when test results differ between runs.
---

# Test Runner for Worktrees

Run every check from the worktree you are working in, not from the root checkout. Run `pwd` and `git branch --show-current` first to confirm which worktree and branch you are in. Worktrees normally live under `.worktrees/<branch-name>`, and Claude Desktop puts them under `.claude/worktrees/`.

## Set up secrets once per worktree

The integration tests need `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID` and `LANGSMITH_WORKSPACE_ID`. `fnox` supplies them. A new worktree has no `fnox.local.toml`, so link it to the main clone's cache once:

```bash
mise run worktree:link-secrets
```

[Per-checkout setup](https://github.com/codekiln/langstar/blob/main/docs/dev/environment-variables.md#per-checkout-setup) explains how `fnox.local.toml` links a worktree to the main clone's cache.

## Run the full pre-commit chain

```bash
mise run check
```

This runs `cargo fmt`, `cargo check`, `cargo clippy` and the nextest suite under `fnox exec`, then `cargo fmt --check`, and stops at the first failure. Run it before every commit that changes Rust code. If only Markdown changed, skip it.

## Run fmt, check, clippy or the tests on their own

```bash
mise run rust:fmt
mise run rust:check
mise run rust:clippy
mise run rust:test
```

`rust:test` passes extra arguments to `cargo nextest run`, and stops with an error naming the missing secret before any test starts.

## Run a single test

```bash
fnox exec -- cargo nextest run --profile ci --all-features --workspace <test_name>
```

`fnox exec` supplies the LangSmith credentials; `cargo nextest run` on its own fails the integration tests with authentication errors.

## Troubleshooting

- **Authentication errors or a missing-secret message**: run `mise run worktree:link-secrets` in this worktree, then retry.
- **Different results in the worktree and the root checkout**: you ran from the wrong directory. Run `pwd` and `git branch --show-current` to confirm.
- **A test fails that your change did not touch**: it still blocks the merge. Fix it or raise an issue; do not skip it.
