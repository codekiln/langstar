# Design

## Context

codekiln's global mise config turns on `mr_boxington` for Rust, but that setting never reached langstar. mise uses langstar's own `rust` entry in `mise.toml`, which pins 1.99.0, in place of the global one. With `mr_boxington = true` on a `rust` entry, mise runs Cargo through `mbx` whenever Cargo starts through mise.

CI's Rust jobs install Rust with `dtolnay/rust-toolchain@stable`, cache with `Swatinem/rust-cache@v2` and run plain `cargo`. rustup reads the same 1.99.0 pin from `rust-toolchain.toml`, so those jobs compile with 1.99.0. rust-cache keys each job's entry by the job's name.

The links to Mr Boxington's docs and source below point at mbx v1.29.1 and mr-boxington-action v1.7.3. langstar pins mbx 1.28.0.

## Goals / Non-Goals

**Goals:**

- When a developer or an agent runs Cargo through mise in a new langstar worktree, mbx restores most of the build from its cache.
- Measure whether Mr Boxington makes CI's jobs faster than rust-cache does, and switch CI to it only if it does.

**Non-Goals:**

- Try the action's `objects` setting in CI, which saves mbx's store between jobs in place of each job's `target/`.

## Decisions

### mise turns mbx on for langstar, pinned to 1.28.0

The `rust` entry in `mise.toml` sets `mr_boxington = true` beside `version = "1.99.0"`, and `[tools]` pins `mr-boxington = "1.28.0"`. mise skips `minimum_release_age` for an exact pin, as the comment above `[settings]` says, so the developer chose 1.28.0 by hand as the newest release older than five days when pinned. `mise exec -- cargo clippy --workspace --all-targets --all-features -- -D warnings` in this worktree ended with `mbx[cache]: 286 hits, 13 misses`, so local builds through mise reach mbx.

### CI keeps rust-cache and plain cargo

CI stays as it was. On this pull request, the agent working on this change switched the Check, Test, Integration Tests, SDK Integration Tests, Clippy and Build jobs to `jdx/mr-boxington-action` and `mbx` commands in [460f1a7 build(ci): run CI's Cargo steps through Mr Boxington](https://github.com/codekiln/langstar/commit/460f1a7bd), then switched them back in [f0795fd build(ci): go back to rust-cache and plain cargo in CI](https://github.com/codekiln/langstar/commit/f0795fdfc), because on a warm run every one of those jobs was slower than with rust-cache and saved a larger cache entry. [CI timings](#ci-timings) has the numbers.

In CI, mbx had nothing to reuse: every job started with an empty mbx store, and every job's summary in both mbx runs reported `0 hits`. By default, `jdx/mr-boxington-action` saves and restores what rust-cache does, the workspace's `target/` and Cargo's registry and git downloads, and leaves mbx's own store out ([mr-boxington/docs/github-action.md:61-66](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L61-L66), [mr-boxington-action/README.md:36-41](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/README.md#L36-L41), [mr-boxington/docs/cookbook/migrate.md:10-32](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/cookbook/migrate.md#L10-L32)). With that default the action also turns off mbx's managed targets, native link caching and remote cache ([mr-boxington/docs/github-action.md:105-115](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L105-L115), [mr-boxington-action/src/index.ts:420-423](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/src/index.ts#L420-L423)). The reuse those jobs did get came from Cargo finding the restored `target/` up to date, which rust-cache already gives and which mbx's summary leaves out of its count ([mr-boxington/docs/github-action.md:51-56](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L51-L56)). mbx added the time spent recording every compilation into a store the job then discarded, and a larger cache entry to save.

The two ways to carry mbx's store between jobs do not fit langstar now:

- The action's `objects` setting saves mbx's store instead of `target/`, and leaves out the registry, so Cargo downloads crates again; the action's author measured it about ten seconds slower than the default setting on a small edit ([mr-boxington-action/README.md:53-68](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/README.md#L53-L68)). Non-Goals names it as a separate experiment.
- The `remote` backend needs a cache server or an S3 bucket ([mr-boxington/docs/github-action.md:151-222](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L151-L222)), which langstar does not have.

Anyone who tries mbx in CI again has to give each job its own `cache-generation`, because the action's cache key leaves out the job name ([mr-boxington-action/src/lib.ts:286-294](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/src/lib.ts#L286-L294)). With `cache-generation: v1-<job>`, every job in the second mbx run restored the entry its own job saved.

### Release builds stay off mbx

`.github/workflows/release.yml` builds the binaries langstar publishes. mbx's docs say a release build should not restore compiler outputs from a shared cache, so that a poisoned entry cannot reach a published binary ([mr-boxington/docs/github-action.md:312-324](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L312-L324)). If the developers later switch CI to mbx, they keep the release jobs on plain `cargo`.

### CI timings

Wall time per job in three CI runs on this pull request:

- [The rust-cache run](https://github.com/codekiln/langstar/actions/runs/41504840394) restored warm entries from `main`.
- [The first mbx run](https://github.com/codekiln/langstar/actions/runs/41509409458) found no mbx entry, compiled everything and saved one entry per job under this pull request.
- [The second mbx run](https://github.com/codekiln/langstar/actions/runs/41510364255) restored those entries.

Seconds for each job, as rust-cache warm, then mbx cold, then mbx warm:

- Check: 26, 68, 28
- Test: 45, 78, 60
- Clippy: 27, 74, 31
- Integration Tests: 73, 145, 112
- SDK Integration Tests: 63, 99, 80
- Build (ubuntu-latest): 50, 110, 59
- Build (macos-latest): 79, 220, 135

mbx saved a larger cache entry for every job: about 35 MB larger for Check, Clippy, Test and both Builds, and more than twice the size for Integration Tests (284 MB to 593 MB) and SDK Integration Tests (263 MB to 726 MB).

Saving the cache explains part of why mbx's second run was slower than rust-cache: rust-cache found an exact entry and saved nothing, while each mbx job spent 3 to 16 seconds saving a new entry for the pull request. The compile steps were slower too, for example 11 seconds against 8 for Test's unit tests and 58 against 47 for the macOS build. The integration test steps call LangSmith, so their times vary from run to run for reasons outside the build.

## Risks / Trade-offs

- [The trial runs saved 14 mbx entries, about 5 GB, under this pull request's scope, while the repository's cache was already over GitHub's 10 GB default] → GitHub evicts the least recently used entries, so `main`'s rust-cache entries may be evicted once and rebuilt on the next push to `main`.
- [Cargo that runs as `~/.cargo/bin/cargo` instead of through mise's wrapper skips mbx. On a Linux filesystem that cannot clone files, such as ext4, it can also fail with `output file ... is not writeable`, because mbx restores outputs as read-only hard links ([mr-boxington/docs/troubleshooting.md:182-189](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/troubleshooting.md#L182-L189), [mr-boxington/docs/how-it-works.md:182-188](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/how-it-works.md#L182-L188))] → `docs/dev/getting-started.md` says to check which Cargo a shell runs with `command -v cargo`, to use `mise exec -- cargo ...` when it is not mise's wrapper, and to recover with `mbx clean`. Macs use APFS, which clones files, so they do not hit the error. A Claude Code session started before Mr Boxington was set up is the only shell seen running `~/.cargo/bin/cargo`, and starting a new session fixes it ([A new Claude Code session runs Cargo through mise's wrapper](#a-new-claude-code-session-runs-cargo-through-mises-wrapper)).

## Resolved Questions

### A new Claude Code session runs Cargo through mise's wrapper

A Claude Code session started on 2026-11-03, after Mr Boxington was set up, printed `~/.local/share/mise/command-wrappers/bin/cargo` for `command -v cargo`, because mise's wrapper directory came before `~/.cargo/bin` on that session's `PATH`. A session started before the setup can still run `~/.cargo/bin/cargo`, and starting a new session fixes it.

## Open Questions

None.
