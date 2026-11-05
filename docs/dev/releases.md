# Releases

To release langstar, merge the release pull request. release-plz opens it, picks the version and writes the changelog from the commit titles on main, and tags the version after the merge. cargo-dist then builds the binaries and publishes the GitHub release.

## Release flow

1. The maintainer merges a pull request into main. release-plz reads the title of each commit the merge adds to main, such as `🐛 fix(sdk): send the workspace header to the Agent Server (#839)`, to pick the next version and write the changelog. A squash merge adds one commit, and GitHub ends its title with the pull request number, which the changelog turns into a link. A merge commit or a rebase adds every commit on the branch, and the changelog lists each `feat`, `fix` and `perf` commit without a link.
2. `prepare-release.yml` runs on main with a token from the release GitHub App. release-plz opens or updates one pull request titled `🔖 release: prepare vX.Y.Z` on a `release-plz-*` branch. The pull request bumps the workspace version in `Cargo.toml` and `Cargo.lock` and adds the version's section to `CHANGELOG.md`.
3. CI runs its checks and Copilot reviews the release pull request, as on any other pull request.
4. The maintainer reads the changelog section, edits it on the release branch if needed, and merges the pull request once its checks pass.
5. `prepare-release.yml` runs again, and release-plz pushes the `vX.Y.Z` tag.
6. GitHub starts `release.yml`, which cargo-dist generates, for the new tag. It runs `release-checks.yml`, builds every target, writes SHA-256 checksums, and publishes the GitHub release with the version's `CHANGELOG.md` section as its notes.

release-plz opens the pull request and pushes the tag with the App token because GitHub starts no workflow for a pull request or tag that `GITHUB_TOKEN` creates.

## Versions

The CLI (`langstar`) and the SDK (`langstar-sdk`) share the workspace version, and only the CLI gets a tag. `CHANGELOG.md` lists commits from both packages.

| Commit title | Version change |
|---|---|
| Any type with `!`, such as `✨ feat(runs)!: ...` | major |
| `✨ feat: ...` | minor |
| `🩹 fix: ...`, `🐛 fix: ...`, `⚡ perf: ...` | patch |
| Any other type, such as `📚 docs: ...` or `🔧 build: ...` | no release |

release-plz reads only the commit title, so it ignores a `BREAKING CHANGE:` footer in the body. The changelog shows each commit's title with a link to its pull request.

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

`release:check` runs two tasks. `release:impact-test` commits each kind of title to a temporary copy of `tests/fixtures/release-impact` and checks the version and changelog line that the pinned release-plz writes. `dist:check` checks that `release.yml` matches `dist-workspace.toml` and lists the release archives.

cargo-dist generates `release.yml` from `dist-workspace.toml`. After changing `dist-workspace.toml` or the `cargo-dist` version in `mise.toml`, regenerate `release.yml` and commit it:

```sh
mise run dist:generate
```

## Deployment lifecycle test

`test_deployment_workflow_full_lifecycle` creates a LangSmith deployment, patches it, waits for the new revision to deploy, and deletes the deployment. It runs only when someone starts it:

```sh
gh workflow run deployment-lifecycle-test.yml
```

## When something fails

- `prepare-release.yml` fails: fix the cause and rerun the failed run. release-plz updates the open release pull request.
- A check fails on the release pull request: no tag exists yet. Merge the fix to main, and release-plz updates the pull request.
- `release.yml` fails after release-plz pushes the tag: the tag stays, and GitHub shows no release for that version until a run succeeds. When a runner caused the failure, rerun the failed jobs, which build the tagged commit again. When the code or `release.yml` needs a fix, merge the fix to main and leave the failed version's tag where it is. release-plz opens a release pull request for the next patch version only after a commit whose title is `feat`, `fix` or `perf`, or carries a `!`, changes files in `cli/` or `sdk/`. A fix to `release.yml` alone changes neither folder, so the next release waits for such a commit.
