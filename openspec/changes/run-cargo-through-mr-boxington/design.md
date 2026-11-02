# Design

## Context

langstar pins Rust 1.99.0 in both `mise.toml` and `rust-toolchain.toml`. mise's `mr_boxington` option on a `rust` entry makes Cargo started through mise run through `mbx`, but a project's `rust` entry replaces the global one, so codekiln's global opt-in stopped at langstar's pin.

Before this change, CI's Rust jobs installed Rust with `dtolnay/rust-toolchain@stable`, cached with `Swatinem/rust-cache@v2` and ran plain `cargo`. rustup reads `rust-toolchain.toml`, so those jobs already compiled with 1.99.0. rust-cache keys each job's entry by the job's name, and saves on any run whose key it did not restore exactly, pull requests included.

The Mr Boxington sources read for this design are the local clone of [jdx/mr-boxington](https://github.com/jdx/mr-boxington) at mbx 1.29.1 (`crates/mbx/Cargo.toml:3`), and [jdx/mr-boxington-action](https://github.com/jdx/mr-boxington-action) at tag v1.7.3, read through the GitHub API. Citations below of the form `mr-boxington/<path>:<line>` and `mr-boxington-action/<path>:<line>` point into those two trees.

## Goals / Non-Goals

**Goals:**

- A new langstar worktree restores most of its build from mbx's cache when Cargo starts through mise.
- CI compiles through the same mbx version that developers install, with no job slower on a warm run than it was with rust-cache.

**Non-Goals:**

- Run the cleanup workflow's `cargo build --release` in `.github/workflows/cleanup-test-deployments.yml` through mbx.

## Decisions

### mise turns mbx on for langstar, pinned to 1.28.0

The `rust` entry in `mise.toml` sets `mr_boxington = true` beside `version = "1.99.0"`, and `[tools]` pins `mr-boxington = "1.28.0"`. mise skips `minimum_release_age` for an exact pin, as the comment above `[settings]` says, so 1.28.0 was chosen by hand as the newest release older than five days when pinned; 1.29.x was newer. `mise exec -- cargo clippy --workspace --all-targets --all-features -- -D warnings` in this worktree ended with `mbx[cache]: 286 hits, 13 misses`, so local builds through mise reach mbx.

### CI replaces rust-cache with jdx/mr-boxington-action in the compiling jobs

The Check, Test, Integration Tests, SDK Integration Tests, Clippy and Build jobs replace `Swatinem/rust-cache@v2` with `jdx/mr-boxington-action`, and run `mbx check`, `mbx nextest run`, `mbx clippy` and `mbx build` where they ran `cargo`. mbx's guide for this case, `mr-boxington/docs/cookbook/migrate.md:10-32`, does the same swap: the action's default `target` payload restores the workspace's `target/` and Cargo's registry and git downloads, which is what rust-cache carried (`mr-boxington/docs/github-action.md:61-66`, `mr-boxington-action/README.md:36-41`). `mbx nextest run` works because mbx hands any Cargo subcommand it does not define to Cargo (`mr-boxington/crates/mbx/src/cli/mod.rs:213-214`).

With the `target` payload the action turns off mbx's managed target directories, native link caching and any remote cache for the rest of the job (`mr-boxington/docs/github-action.md:105-115`, `mr-boxington-action/src/index.ts:420-423`). So `target/` stays a real directory in CI, the JUnit files stay at `target/nextest/...` where the publish steps read them, and mbx leaves an existing `target/` in place in every CI build anyway (`mr-boxington/docs/managed-targets.md:41-45`). mbx also turns Cargo's incremental mode off in CI (`mr-boxington/crates/mbx/src/policy.rs:41-53`). Test binaries run outside mbx's scheduler unless `MBX_SCHEDULER_TESTS=1` is set (`mr-boxington/docs/scheduling.md:325-328`), so nextest's `slow-timeout` in `.config/nextest.toml` keeps its meaning (`mr-boxington/docs/scheduling.md:374-381`).

The action's `objects` payload, which shares single compilations across jobs, is not used: it leaves out the registry, and its author measured it about ten seconds slower than the `target` payload on a small edit (`mr-boxington-action/README.md:53-68`). The `remote` backend is not used either, since it needs a cache server or an S3 bucket (`mr-boxington/docs/github-action.md:151-222`), which langstar does not have.

### The action is pinned to its v1.7.3 commit and installs mbx 1.28.0

`uses:` names commit `88acf2bb50552c7c508cbac13e5f8c576b95b1de` with a `# v1.7.3` comment, as `mr-boxington/docs/github-action.md:371-374` asks. v1.7.3 was tagged on 2026-10-26, a week before this change, so it meets the five-day rule `mise.toml` applies to its own tools. `version: 1.28.0` installs the release `mise.toml` pins; without it the action installs the latest release (`mr-boxington/docs/github-action.md:17-20`), which was 1.29.x. The action accepts a release only when GitHub marks it immutable and lists an asset digest (`mr-boxington-action/README.md:135-138`).

### Each job keeps its own cache entry through cache-generation

The action's generated key is `<os>-<arch>-mbx-<generation>-<toolchain>-<commit>`, with no job name in it (`mr-boxington-action/src/lib.ts:286-294`), and its restore key drops only the commit (`mr-boxington-action/src/lib.ts:322-329`). Six jobs on one Linux runner image would then save under one key, and each would restore whichever job saved last, so a debug test build could start from a release build's `target/`. Each job sets `cache-generation: v1-<job>`, which goes into both the key and the restore key (`mr-boxington-action/src/lib.ts:385-392`) and is how the action's README says to keep one set of entries apart from another (`mr-boxington-action/README.md:209`). `cache-key-suffix` would not do it, because it leaves the restore keys shared (`mr-boxington-action/README.md:162-178`). The macOS Build job has its own key already, since the key starts with the platform.

The key also carries a hash of `rustc -vV` from the checkout (`mr-boxington-action/src/lib.ts:418-436`, `mr-boxington/docs/github-action.md:141-149`), and rustup answers that from `rust-toolchain.toml`, so the key follows the 1.99.0 pin and a toolchain bump starts new entries.

### Pull requests save their own entries; fork pull requests save nothing

The action saves only after a successful push to the default branch unless told otherwise (`mr-boxington/docs/github-action.md:78-88`). Each job sets `save-on-pull-request: true` so a pull request's second push restores what its first push compiled, as rust-cache let it. The action saves a same-repository pull request under that pull request's merge ref, which GitHub restores only to later runs of the same pull request, never to `main` or another pull request (`mr-boxington-action/README.md:191`, `mr-boxington-action/src/lib.ts:507-528`). A fork pull request never saves (`mr-boxington-action/src/lib.ts:519`), so code from a fork cannot place compiler outputs that another run restores.

Secrets reach the integration jobs as before, through each step's `env`, and none of the action's inputs reads them.

### Security Audit, the release workflow and the cleanup workflow keep rust-cache and plain cargo

`cargo audit` compiles nothing, so the action would find no target directory and save no entry (`mr-boxington-action/src/index.ts:683-686`); the job keeps rust-cache for its downloads. `.github/workflows/release.yml` builds the binaries langstar publishes, and mbx's docs say a release build should not restore compiler outputs from a shared cache, so that a poisoned entry cannot reach a published binary (`mr-boxington/docs/github-action.md:312-324`). Release jobs therefore keep plain `cargo`. The cleanup workflow is left alone; Non-Goals names it.

### CI timings

Wall time per job, from job start to job end, in three runs of the CI workflow on this pull request:

- The last rust-cache run, [run 41504840394](https://github.com/codekiln/langstar/actions/runs/41504840394), restored warm rust-cache entries from `main`.
- The first mbx run, [run 41509409458](https://github.com/codekiln/langstar/actions/runs/41509409458), found no mbx entry and compiled everything, then saved one entry per job under this pull request.
- The second mbx run, RUN2, restored those entries.

Times (rust-cache warm / mbx cold / mbx warm):

- Check: 26s / 68s / CHECK2
- Test: 45s / 78s / TEST2
- Clippy: 27s / 74s / CLIPPY2
- Integration Tests: 73s / 145s / INT2
- SDK Integration Tests: 63s / 99s / SDK2
- Build (ubuntu-latest): 50s / 110s / BUILDL2
- Build (macos-latest): 79s / 220s / BUILDM2

TIMING-SUMMARY

## Risks / Trade-offs

- [The first run of each job on `main` after merge compiles from nothing, because the rust-cache entries do not carry over] → One slower run per job; the next pull request restores from it.
- [langstar's GitHub Actions cache was at its 10 GB limit before this change, and a saving pull request writes a new entry on every push (`mr-boxington-action/src/lib.ts:444-452`)] → GitHub evicts the least recently used entries, and `main`'s entries stay in use as long as pull requests restore them. [Question 2](#2---should-pull-requests-keep-saving-cache-entries-when-the-repositorys-cache-is-full) asks whether pull requests should keep saving.
- [mbx is a newer tool than rust-cache, and a bug in it could break a build that plain Cargo would pass] → Reverting `.github/workflows/ci.yml` restores rust-cache; nothing else depends on mbx in CI.

## Resolved Questions

None.

## Open Questions

### 1 - If a new Claude Code session still runs ~/.cargo/bin/cargo, should someone file a mise issue?

> Context from the drafting agent, for question 1.
> In a terminal, `command -v cargo` prints mise's `command-wrappers/bin/cargo`, so Cargo goes through Mr Boxington. In a Claude Code session started earlier, `command -v cargo` prints `~/.cargo/bin/cargo` instead, and Cargo runs without the cache: builds still work but reuse nothing. A new terminal or a new Claude Code session should fix it. If it does not, the cause is in how mise orders `PATH`, and the fix belongs in a mise issue, not in this repository. `docs/dev/getting-started.md` tells developers to check with `command -v cargo`.

<ANSWER_HERE>

### 2 - Should pull requests keep saving cache entries when the repository's cache is full?

> Context from the drafting agent, for question 2.
> `save-on-pull-request: true` lets a pull request's second push restore what its first push compiled. Without it, every push to a pull request restores `main`'s entry and compiles the pull request's changes again. Before this change the repository's cache held 10.6 GB, over GitHub's 10 GB default, and each saving push adds about one entry per changed job.

<ANSWER_HERE>
