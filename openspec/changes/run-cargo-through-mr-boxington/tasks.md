# Tasks

Both groups land in one pull request, [#843 🔧 build(mise): run Cargo through Mr Boxington](https://github.com/codekiln/langstar/pull/843). The change adds no langstar code, so it adds no unit or integration tests: the existing suites, run through mbx locally and in CI, are its test.

## 1. Local builds through mise

- [x] 1.1 Set `mr_boxington = true` on the `rust` entry in `mise.toml`, keeping `version = "1.99.0"`; verify that `mise exec -- cargo clippy --workspace --all-targets --all-features -- -D warnings` passes and ends with an `mbx[cache]:` line.
- [x] 1.2 Pin `mr-boxington = "1.28.0"` in `mise.toml` with a comment saying it was the newest release older than `minimum_release_age` when pinned; verify that `mise exec -- mbx --version` prints `mbx 1.28.0`.
- [x] 1.3 Add a paragraph to `docs/dev/getting-started.md` saying that Cargo started through mise runs through mbx, how to check which `cargo` a shell runs, and that `target/` becomes a symlink into mbx's cache; verify that `mise exec -- cargo nextest run --workspace --lib` passes and prints an `mbx[cache]:` line, as the paragraph says.

## 2. CI through Mr Boxington

- [x] 2.1 In `.github/workflows/ci.yml`, replace `Swatinem/rust-cache` with `jdx/mr-boxington-action` pinned to its v1.7.3 commit in the Check, Test, Integration Tests, SDK Integration Tests, Clippy and Build jobs, with `version: 1.28.0`, `cache-generation: v1-<job>` and `save-on-pull-request: true`, and run their compile commands as `mbx check`, `mbx nextest run`, `mbx clippy` and `mbx build`; verify with a passing CI run on the pull request whose Mr Boxington steps print `Set up mbx 1.28.0`.
- [ ] 2.2 Push a second commit to the pull request and verify that each changed job's Mr Boxington step restores the entry the first run saved under its own cache generation; record both runs' job times beside the last rust-cache run in [design.md, CI timings](design.md#ci-timings).

## Workflow follow-up

- codekiln answers the open questions in design.md.
- After the pull request merges, check that the first push to `main` saves one entry per changed job (`gh cache list --key linux-x64-mbx-`), then archive the change with `openspec archive run-cargo-through-mr-boxington`, which syncs `specs/dev-environment/spec.md` into `openspec/specs/`.
