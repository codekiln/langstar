# Design

## Context

langstar pins Rust 1.99.0 in both `mise.toml` and `rust-toolchain.toml`. mise's `mr_boxington` option on a `rust` entry makes Cargo started through mise run through `mbx`, but a project's `rust` entry replaces the global one, so codekiln's global `mr_boxington = true` did not apply inside langstar.

CI's Rust jobs install Rust with `dtolnay/rust-toolchain@stable`, cache with `Swatinem/rust-cache@v2` and run plain `cargo`. rustup reads `rust-toolchain.toml`, so those jobs compile with 1.99.0. rust-cache keys each job's entry by the job's name.

The Mr Boxington sources read for this design are the local clone of [jdx/mr-boxington](https://github.com/jdx/mr-boxington) at mbx 1.29.1 (`crates/mbx/Cargo.toml:3`), and [jdx/mr-boxington-action](https://github.com/jdx/mr-boxington-action) at tag v1.7.3, read through the GitHub API. The citations below link to those two tags, v1.29.1 and v1.7.3.

## Goals / Non-Goals

**Goals:**

- A new langstar worktree restores most of its build from mbx's cache when Cargo starts through mise.
- CI changes only if Mr Boxington makes its jobs faster than rust-cache does.

**Non-Goals:**

- Try the action's `objects` payload in CI, which shares single compilations between jobs instead of carrying each job's `target/`.

## Decisions

### mise turns mbx on for langstar, pinned to 1.28.0

The `rust` entry in `mise.toml` sets `mr_boxington = true` beside `version = "1.99.0"`, and `[tools]` pins `mr-boxington = "1.28.0"`. mise skips `minimum_release_age` for an exact pin, as the comment above `[settings]` says, so the developer chose 1.28.0 by hand as the newest release older than five days when pinned; 1.29.x was newer. `mise exec -- cargo clippy --workspace --all-targets --all-features -- -D warnings` in this worktree ended with `mbx[cache]: 286 hits, 13 misses`, so local builds through mise reach mbx.

### CI keeps rust-cache and plain cargo

CI stays as it was. On this pull request, [460f1a7 build(ci): run CI's Cargo steps through Mr Boxington](https://github.com/codekiln/langstar/commit/460f1a7bd) switched the Check, Test, Integration Tests, SDK Integration Tests, Clippy and Build jobs to `jdx/mr-boxington-action` and `mbx` commands, and [f0795fd build(ci): go back to rust-cache and plain cargo in CI](https://github.com/codekiln/langstar/commit/f0795fdfc) reverted that, because on a warm run every one of those jobs was slower than with rust-cache and saved a larger cache entry. [CI timings](#ci-timings) has the numbers.

The action's default `target` payload restores the workspace's `target/` and Cargo's registry and git downloads ([mr-boxington/docs/github-action.md:61-66](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L61-L66), [mr-boxington-action/README.md:36-41](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/README.md#L36-L41)), which is what rust-cache carries; mbx's own guide for leaving rust-cache makes that swap ([mr-boxington/docs/cookbook/migrate.md:10-32](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/cookbook/migrate.md#L10-L32)). With that payload the action turns off mbx's managed targets, native link caching and remote cache ([mr-boxington/docs/github-action.md:105-115](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L105-L115), [mr-boxington-action/src/index.ts:420-423](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/src/index.ts#L420-L423)), and it does not carry mbx's own store. Each CI job therefore starts with an empty mbx store, and every job's summary in both mbx runs reported `0 hits`. Any reuse came from Cargo finding its restored `target/` fresh, which rust-cache already gives; mbx's summary says it does not count that reuse ([mr-boxington/docs/github-action.md:51-56](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L51-L56)). What mbx added on top was the work of recording every compilation into a store the job then threw away, and a larger entry to save.

The two ways to carry mbx's store between jobs do not fit langstar now:

- The `objects` payload carries the store, not `target/`, and leaves out the registry, so Cargo downloads crates again; the action's author measured it about ten seconds slower than the `target` payload on a small edit ([mr-boxington-action/README.md:53-68](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/README.md#L53-L68)). Non-Goals names it as a separate experiment.
- The `remote` backend needs a cache server or an S3 bucket ([mr-boxington/docs/github-action.md:151-222](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L151-L222)), which langstar does not have.

The trial also showed what a later attempt needs. The action's generated key has no job name ([mr-boxington-action/src/lib.ts:286-294](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/src/lib.ts#L286-L294)) and its restore key drops only the commit ([mr-boxington-action/src/lib.ts:322-329](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/src/lib.ts#L322-L329)), so each job needs its own `cache-generation`, which goes into both ([mr-boxington-action/src/lib.ts:385-392](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/src/lib.ts#L385-L392), [mr-boxington-action/README.md:209](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/README.md#L209)). With `cache-generation: v1-<job>`, every job in the second run restored the entry its own job saved. `cache-key-suffix` would not separate them, because it leaves the restore keys shared ([mr-boxington-action/README.md:162-178](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/README.md#L162-L178)). A pull request saves only with `save-on-pull-request: true`, into its own merge-ref scope, and a fork pull request never saves ([mr-boxington-action/README.md:191](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/README.md#L191), [mr-boxington-action/src/lib.ts:507-528](https://github.com/jdx/mr-boxington-action/blob/v1.7.3/src/lib.ts#L507-L528)). The action should be pinned to a commit ([mr-boxington/docs/github-action.md:371-374](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L371-L374)) and given `version:` so it installs the release `mise.toml` pins rather than the latest ([mr-boxington/docs/github-action.md:17-20](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L17-L20)).

### Release builds stay off mbx

`.github/workflows/release.yml` builds the binaries langstar publishes. mbx's docs say a release build should not restore compiler outputs from a shared cache, so that a poisoned entry cannot reach a published binary ([mr-boxington/docs/github-action.md:312-324](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/github-action.md#L312-L324)). Any later move of CI to mbx leaves release jobs on plain `cargo`.

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

Size of each job's saved cache entry, rust-cache then mbx: Check 159 MB and 193 MB, Clippy 159 MB and 193 MB, Test 266 MB and 303 MB, Build on Linux 202 MB and 240 MB, Build on macOS 197 MB and 232 MB, Integration Tests 284 MB and 593 MB, SDK Integration Tests 263 MB and 726 MB.

Part of the warm gap is the save: rust-cache found an exact entry and saved nothing, while each mbx job spent 3 to 16 seconds saving a new pull request entry. The compile steps were slower too, for example 11 seconds against 8 for Test's unit tests and 58 against 47 for the macOS build. The integration test steps call LangSmith, so their times vary from run to run for reasons outside the build.

## Risks / Trade-offs

- [The trial runs saved 14 mbx entries, about 5 GB, under this pull request's scope, while the repository's cache was already over GitHub's 10 GB default] → GitHub evicts the least recently used entries, so `main`'s rust-cache entries may be evicted once and rebuilt on the next push to `main`.
- [Plain `cargo` in a shell whose `PATH` does not start with mise's wrapper skips mbx, and on a filesystem that cannot clone files, such as ext4, it can fail with `output file ... is not writeable` on a unit mbx restored as a read-only hard link ([mr-boxington/docs/troubleshooting.md:182-189](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/troubleshooting.md#L182-L189), [mr-boxington/docs/how-it-works.md:182-188](https://github.com/jdx/mr-boxington/blob/v1.29.1/docs/how-it-works.md#L182-L188))] → `docs/dev/getting-started.md` says to check with `command -v cargo`, to use `mise exec -- cargo ...` in such a shell, and to recover with `mbx clean`. macOS's APFS clones files, so Macs do not hit the error. The open question [If a new Claude Code session still runs ~/.cargo/bin/cargo, should someone file a mise issue?](#1---if-a-new-claude-code-session-still-runs-cargobincargo-should-someone-file-a-mise-issue) covers the shell seen so far.

## Resolved Questions

None.

## Open Questions

### 1 - If a new Claude Code session still runs ~/.cargo/bin/cargo, should someone file a mise issue?

> Context from the drafting agent, for question 1.
> In a terminal, `command -v cargo` prints mise's `command-wrappers/bin/cargo`, so Cargo goes through Mr Boxington. In a Claude Code session started earlier, `command -v cargo` prints `~/.cargo/bin/cargo` instead, and plain `cargo` there skips mbx: it reuses nothing, and on ext4 it can fail on a read-only restored output. `mise exec -- cargo ...` in that session still printed an `mbx[cache]:` line on 2026-11-02, while `mise exec -- sh -c 'command -v cargo'` printed `~/.cargo/bin/cargo`, so scripts that mise starts skip mbx too. A new terminal or a new Claude Code session should fix it. If it does not, the cause is in how mise orders `PATH`, and the fix belongs in a mise issue. `docs/dev/getting-started.md` tells developers to check with `command -v cargo`.

<ANSWER_HERE>
