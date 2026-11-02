# Run Cargo through Mr Boxington

Issue: [#841 Run Cargo through Mr Boxington so worktrees share compiled work](https://github.com/codekiln/langstar/issues/841)

## Why

codekiln and their coding agents work in many langstar worktrees at once, and each worktree compiles the whole workspace and every dependency into its own `target/` directory, even when the code matches the next worktree's. [Mr Boxington](https://github.com/jdx/mr-boxington) (`mbx`) stores each compiler output once on the machine and restores it in any checkout that compiles the same crate with the same inputs.

codekiln already turns mbx on for their machine in their global mise config, but langstar's own `rust` entry in `mise.toml` replaces the global one, so the opt-in is off inside langstar.

## What Changes

- `mise.toml` sets `mr_boxington = true` on the `rust` entry, keeping the 1.99.0 pin, so Cargo started through mise (tasks, `mise exec`, the shims or an activated shell) runs through mbx.
- `mise.toml` pins `mr-boxington = "1.28.0"`. An exact pin is exempt from the repo's `minimum_release_age = "5d"`, so whoever bumps it picks a release at least five days old by hand, as the comment above `[settings]` says.
- `docs/dev/getting-started.md` says what mbx does, how to check that `cargo` goes through it, and that `target/` becomes a symlink into mbx's cache.
- CI keeps `Swatinem/rust-cache` and plain `cargo`. A trial on this change's pull request moved the compiling jobs to `jdx/mr-boxington-action`, and every one of them ran slower on a warm run than with rust-cache, so the trial was reverted; design.md records the measurements and the reasons.

## Capabilities

### New Capabilities

- `dev-environment`: how a langstar developer and CI build the workspace. No `langstar` command group covers building langstar, so this capability is named for the developer's environment instead.

### Modified Capabilities

None.

## Impact

- `mise.toml`: the `rust` entry and a new `mr-boxington` pin.
- `docs/dev/getting-started.md`: one paragraph.
- `.github/workflows/ci.yml`: unchanged once the trial is reverted.
- A Claude Code session started before Mr Boxington was set up can still run `~/.cargo/bin/cargo`, which skips the cache; [design.md question 1](design.md#1---if-a-new-claude-code-session-still-runs-cargobincargo-should-someone-file-a-mise-issue) covers it.

## Citations

- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Make the Right Thing Easy and the Wrong Thing Hard](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20the%20Right%20Thing%20Easy%20and%20the%20Wrong%20Thing%20Hard.md)
- [My/Principle/Explicit is better than Implicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Explicit%20is%20better%20than%20Implicit.md)
