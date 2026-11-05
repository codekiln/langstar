---
name: bump-release
description: Answer questions about cutting a langstar release, and show what the next release will contain. release-plz keeps a release pull request open. When the maintainer merges it, release-plz pushes the version tag and `release.yml` publishes the GitHub release. Use when someone asks to cut, bump, preview or publish a release, or asks why a release did not appear.
---

# Release langstar

[docs/dev/releases.md](../../../docs/dev/releases.md) explains the release flow, the version rules and the one-time GitHub setup. Read it first.

## Find the release pull request

```bash
gh pr list --label release --state open
```

The pull request is titled `🔖 release: prepare vX.Y.Z`, and its `CHANGELOG.md` section lists what the release contains. release-plz opens it after someone merges a `feat`, `fix`, `perf` or `!` commit that changes files in `cli/` or `sdk/`. When none is open, either main has no such commit since the last tag, or `prepare-release.yml` did not run; see "Find out why a release did not appear" below.

## Ask the maintainer to merge the release pull request

Merging the release pull request releases langstar, and only the maintainer merges it. Ask before merging it, tagging, or starting a release workflow.

## Preview the next release locally

```bash
mise run release:check
release-plz update --config release-plz.toml
git diff Cargo.toml CHANGELOG.md
```

`release-plz update` writes the next version and changelog section to the working tree. Discard them with `git checkout Cargo.toml Cargo.lock CHANGELOG.md` when done.

## Find out why a release did not appear

- No release pull request after a merge: open the latest `Prepare release` run with `gh run list -w prepare-release.yml`. A skipped job means the `RELEASE_ENABLED` variable is missing. A failure at "Create the release App token" means a release App secret is missing.
- A tag with no GitHub release: open the `Release` run for that tag with `gh run list -w release.yml`.
