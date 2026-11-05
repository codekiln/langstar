# Releases

To release langstar, merge the release pull request. release-plz opens it, picks the version and writes the changelog from the commit titles on main, and tags the version after the merge. cargo-dist then builds the binaries and publishes the GitHub release.

## Release flow

1. A pull request merges into main. Its squash commit title, such as `✨ feat(runs): ...`, decides what the release needs.
2. `prepare-release.yml` runs on main with a token from the release GitHub App. release-plz opens or updates one pull request titled `🔖 release: prepare vX.Y.Z` on a `release-plz-*` branch. The pull request bumps the workspace version in `Cargo.toml` and `Cargo.lock` and adds the version's section to `CHANGELOG.md`.
3. CI and Copilot review the release pull request like any other.
4. The maintainer reads the changelog section, edits it on the release branch if needed, and merges the pull request once its checks pass.
5. `prepare-release.yml` runs again, and release-plz pushes the `vX.Y.Z` tag.
6. The tag starts `release.yml`, which cargo-dist generates. It runs `release-checks.yml`, builds every target, writes SHA-256 checksums, and publishes the GitHub release with the version's `CHANGELOG.md` section as its notes.

The App token matters in steps 2 and 5: GitHub starts no workflow for a pull request or tag that `GITHUB_TOKEN` creates.

## Versions

The CLI (`langstar`) and the SDK (`langstar-sdk`) share the workspace version, and only the CLI gets a tag. `CHANGELOG.md` lists commits from both packages.

| Commit title | Version change |
|---|---|
| Any type with `!`, such as `✨ feat(runs)!: ...` | major |
| `✨ feat: ...` | minor |
| `🩹 fix: ...`, `🐛 fix: ...`, `⚡ perf: ...` | patch |
| Any other type, such as `📚 docs: ...` or `🔧 build: ...` | no release |

Only the title counts. A `BREAKING CHANGE:` footer in the body changes nothing, and the changelog shows each commit's title with a link to its pull request.

## Release assets

| Platform | Archive |
|---|---|
| Linux x86-64 (static, musl) | `langstar-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (static, musl) | `langstar-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple silicon | `langstar-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `langstar-x86_64-apple-darwin.tar.gz` |

Each archive has a `.sha256` file next to it. `scripts/install.sh` downloads these names, and falls back to the older `langstar-<version>-<platform>.tar.gz` names for releases up to v3.0.0. The devcontainer feature runs `scripts/install.sh`.

## One-time GitHub setup

1. [Create a GitHub App](https://github.com/settings/apps/new) owned by codekiln, with its webhook turned off and these repository permissions:
   - Contents: read and write
   - Pull requests: read and write
   - Metadata: read-only, which GitHub adds
2. Generate a private key on the App's settings page, and [install the App](https://github.com/settings/installations) on `codekiln/langstar` only.
3. Add two [Actions repository secrets](https://github.com/codekiln/langstar/settings/secrets/actions):
   - `RELEASE_APP_ID`: the App ID from the App's settings page
   - `RELEASE_APP_PRIVATE_KEY`: the whole `.pem` private key
4. Delete the downloaded `.pem` file.
5. Add the [Actions repository variable](https://github.com/codekiln/langstar/settings/variables/actions) `RELEASE_ENABLED` with the value `true`. Until it exists, `prepare-release.yml` skips its job.

## Checks and maintenance

```sh
mise run release:check
```

This runs `release:impact-test`, which commits each kind of title to a scratch copy of `tests/fixtures/release-impact` and checks the version and changelog line that the pinned release-plz writes, and `dist:check`.

After changing `dist-workspace.toml` or the `cargo-dist` version in `mise.toml`, regenerate the release workflow and commit it:

```sh
mise run dist:generate
```

`release.yml` is generated, so change `dist-workspace.toml` and regenerate it rather than editing the file.

## Deployment lifecycle test

`test_deployment_workflow_full_lifecycle` creates a LangSmith deployment, exercises it and deletes it. It runs only when someone starts it:

```sh
gh workflow run deployment-lifecycle-test.yml
```

## When something fails

- `prepare-release.yml` fails: fix the cause and rerun the failed run. release-plz updates the open release pull request.
- A check fails on the release pull request: no tag exists yet. Fix main, and release-plz updates the pull request.
- `release.yml` fails after the tag: the tag stays, and no GitHub release exists until the workflow succeeds. A rerun builds the tagged commit again, so it helps when the cause was outside the code, such as a runner. When the code needs a fix, merge it to main and release it under the next version. Keep the existing tag where it is.
