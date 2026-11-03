# Run Cargo through Mr Boxington

Issue: [#841 Run Cargo through Mr Boxington so worktrees share compiled work](https://github.com/codekiln/langstar/issues/841)

## Why

codekiln and their coding agents work in many langstar worktrees at once, and Cargo compiles the whole workspace and every dependency from scratch in each new worktree, even when another worktree already compiled the same code. [Mr Boxington](https://github.com/jdx/mr-boxington) (`mbx`) keeps one copy of each compiled crate on the machine, so a new worktree's first build restores that work instead of repeating it.

codekiln's global mise config sets `mr_boxington = true` on its `rust` entry, but mise reads langstar's own `rust` entry in `mise.toml` in place of the global one, so that setting never reaches langstar.

## What Changes

- `mise.toml` sets `mr_boxington = true` on the `rust` entry, keeping the 1.99.0 pin, so Cargo started through mise (tasks, `mise exec`, the shims or an activated shell) runs through mbx.
- `mise.toml` pins `mr-boxington = "1.28.0"`. An exact pin is exempt from the repo's `minimum_release_age = "5d"`, so whoever bumps it picks a release at least five days old by hand, as the comment above `[settings]` says.
- `docs/dev/getting-started.md` says what mbx does, how to check that `cargo` goes through it, and that `target/` becomes a symlink into mbx's cache.
- CI keeps `Swatinem/rust-cache` and plain `cargo`. The agent working on this change switched the compiling CI jobs to `jdx/mr-boxington-action`, found every job slower on a warm run than with rust-cache, and switched them back; [design.md, CI keeps rust-cache and plain cargo](design.md#ci-keeps-rust-cache-and-plain-cargo) has the measurements.

## Capabilities

### New Capabilities

- `dev-environment`: how a langstar developer and CI build the workspace.

### Modified Capabilities

None.

## Impact

- `mise.toml`: the `rust` entry and a new `mr-boxington` pin.
- `docs/dev/getting-started.md`: three paragraphs on mbx.
- A Claude Code session started before Mr Boxington was set up can still run `~/.cargo/bin/cargo`, which skips the cache. A Claude Code session started on 2026-11-03 ran mise's wrapper instead, so starting a new session fixes it, as [design.md, A new Claude Code session runs Cargo through mise's wrapper](design.md#a-new-claude-code-session-runs-cargo-through-mises-wrapper) says.

## Citations

- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Make the Right Thing Easy and the Wrong Thing Hard](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20the%20Right%20Thing%20Easy%20and%20the%20Wrong%20Thing%20Hard.md)
- [My/Principle/Explicit is better than Implicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Explicit%20is%20better%20than%20Implicit.md)
