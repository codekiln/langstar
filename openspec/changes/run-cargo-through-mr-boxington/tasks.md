# Tasks

Both groups land in one pull request, [#843 🔧 build(mise): run Cargo through Mr Boxington](https://github.com/codekiln/langstar/pull/843). The change touches only configuration and documentation, and the existing test suites, run through mbx locally, test it.

## 1. Local builds through mise

- [x] 1.1 Set `mr_boxington = true` on the `rust` entry in `mise.toml`, keeping `version = "1.99.0"`; verify that `mise exec -- cargo clippy --workspace --all-targets --all-features -- -D warnings` passes and ends with an `mbx[cache]:` line.
- [x] 1.2 Pin `mr-boxington = "1.28.0"` in `mise.toml` with a comment saying it was the newest release older than `minimum_release_age` when pinned; verify that `mise exec -- mbx --version` prints `mbx 1.28.0`.
- [x] 1.3 Add a paragraph to `docs/dev/getting-started.md` saying that Cargo started through mise runs through mbx, how to check which `cargo` a shell runs with `command -v cargo`, and that `target/` becomes a symlink into mbx's cache; verify that `mise exec -- cargo nextest run --workspace --lib` passes and prints an `mbx[cache]:` line.

## 2. Decide whether CI uses Mr Boxington

- [x] 2.1 Switch the Check, Test, Integration Tests, SDK Integration Tests, Clippy and Build jobs in `.github/workflows/ci.yml` to `jdx/mr-boxington-action` pinned to its v1.7.3 commit, with `version: 1.28.0`, `cache-generation: v1-<job>` and `save-on-pull-request: true`, and run their compile commands through `mbx`; verify with two passing CI runs, the second restoring each job's own entry, and record their job times and entry sizes beside rust-cache's in [design.md, CI timings](design.md#ci-timings).
- [x] 2.2 Revert that switch, since every job ran slower on a warm run than with rust-cache, and record why in [design.md, CI keeps rust-cache and plain cargo](design.md#ci-keeps-rust-cache-and-plain-cargo); verify that `git diff bc3dc21 -- .github/workflows/ci.yml` is empty and that CI passes on the pull request.

## Workflow follow-up

- codekiln answers the open question [If a new Claude Code session still runs ~/.cargo/bin/cargo, should someone file a mise issue?](design.md#1---if-a-new-claude-code-session-still-runs-cargobincargo-should-someone-file-a-mise-issue).
- After the pull request merges, archive the change with `openspec archive run-cargo-through-mr-boxington`, which syncs `specs/dev-environment/spec.md` into `openspec/specs/`.
