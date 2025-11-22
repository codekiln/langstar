# Architecture of Production-Grade Rust CLI Release Pipelines with GitHub Actions

## Overview

Large, production-grade Rust CLI projects such as ripgrep, bat, fd, exa, and starship have established robust GitHub Actions architectures for cross-platform releases. Their CI/CD pipelines balance safety, reliability, and automation by combining strict CI preconditions, systematic cross-compilation, changelog automation with git-cliff, secure asset publishing, and carefully designed release gating mechanisms. This report synthesizes how these projects address full CI enforcement, cross-compilation (Linux glibc/musl, macOS, Windows), changelog automation with git-cliff, artifact uploading and possible signing/notarization, and release gating, referencing their publicly available workflow files.

## CI Preconditions: Ensuring Code Quality Before Any Release

All examined projects enforce a two-stage workflow:

* **Stage 1: Comprehensive CI Checks**

  * **Code formatting:** `cargo fmt` (via `--check`) ensures code style compliance.
  * **Linting:** `cargo clippy` with `-D warnings` to disallow all lints.
  * **Testing:** `cargo test`—both unit and integration tests must pass.
  * **Security vetting:** `cargo audit` is run to detect known vulnerabilities.
* **Stage 2: Conditional Artifact Building**

  * Release and artifact-building jobs only execute if all above checks pass.
  * Technically, this is enforced via GitHub Actions’ `needs:` directive, making build/release steps depend on the successful completion of earlier CI jobs.

This approach is consistent across all surveyed repositories, using either a monolithic workflow or separate CI/release workflows tied together via job dependencies and strict branch protection settings. Branch protection on `main` or `master` commonly requires all relevant status checks to succeed before merges or releases proceed.

**Examples:**

* [ripgrep's workflows](https://github.com/BurntSushi/ripgrep/tree/master/.github/workflows)
* [bat's workflows](https://github.com/sharkdp/bat/tree/master/.github/workflows)
* [fd's workflows](https://github.com/sharkdp/fd/tree/master/.github/workflows)

## Cross-Compilation and Release Artifact Matrix

### Cross-Compilation Strategy

All major Rust CLIs deliver cross-platform binaries for:

* **Linux:**

  * **glibc (amd64):** Binaries built using native Ubuntu runners.
  * **musl (static, amd64):** For ultra-portability, via the `cross` tool or custom Docker containers.
* **macOS:** Native runners.
* **Windows:** Native runners.

### Build Matrix

* Each workflow defines a `strategy.matrix` that enumerates all target platforms.
* Artifacts for each matrix axis are built in parallel jobs.

**Examples:**

* [starship’s release matrix](https://github.com/starship/starship/tree/master/.github/workflows)
* [exa’s workflows](https://github.com/ogham/exa/tree/master/.github/workflows)
* [bat’s workflows](https://github.com/sharkdp/bat/tree/master/.github/workflows)

## Automated Changelog Generation with git-cliff

### Automation Flow

* **git-cliff** parses commit history since the last tag.
* [orhun/git-cliff-action](https://github.com/orhun/git-cliff-action) is widely used.
* Appears in PR previews and release workflows.

**Examples:**

* [ripgrep](https://github.com/BurntSushi/ripgrep/tree/master/.github/workflows)
* [bat](https://github.com/sharkdp/bat/tree/master/.github/workflows)
* [fd](https://github.com/sharkdp/fd/tree/master/.github/workflows)

## Artifact Upload, Signing, and Notarization

### Asset Publishing

Common tools:

* [svenstaro/upload-release-action](https://github.com/svenstaro/upload-release-action)
* [actions/upload-release-asset](https://github.com/actions/upload-release-asset)

### Signing & Notarization

* [starship](https://github.com/starship/starship/tree/master/.github/workflows) performs macOS notarization.

## Release Gating, Approval, and GITHUB_TOKEN Workarounds

### Gating Safety

* Branch protection
* `workflow_dispatch` triggers
* Environment protection rules

### GITHUB_TOKEN Limitations

* Cannot trigger CI on PRs it creates.
* Projects use PATs or GitHub App tokens for release uploads.

## Comparison of Leading Project Architectures

### ripgrep

* CI gating, matrix build, musl via cross.

### bat

* Similar to ripgrep.

### fd

* Similar architecture.

### exa

* Strong CI + matrix builds.

### starship

* Includes macOS notarization, uses PAT/App tokens.

## Best Practices

* Gate builds on CI
* Use matrix for OS targets
* Automate changelogs
* Secure release uploads with PAT/App tokens
* Consider macOS signing

## Conclusion

Production Rust CLI release pipelines:

* Enforce CI quality gates
* Compile for all major platforms
* Automate changelogs via git-cliff
* Require secure authenticated release uploads
* Use human-in-the-loop gating via workflow_dispatch or environments

## Sources

[1] ripgrep: [https://github.com/BurntSushi/ripgrep/tree/master/.github/workflows](https://github.com/BurntSushi/ripgrep/tree/master/.github/workflows)
[2] bat: [https://github.com/sharkdp/bat/tree/master/.github/workflows](https://github.com/sharkdp/bat/tree/master/.github/workflows)
[3] fd: [https://github.com/sharkdp/fd/tree/master/.github/workflows](https://github.com/sharkdp/fd/tree/master/.github/workflows)
[4] exa: [https://github.com/ogham/exa/tree/master/.github/workflows](https://github.com/ogham/exa/tree/master/.github/workflows)
[5] starship: [https://github.com/starship/starship/tree/master/.github/workflows](https://github.com/starship/starship/tree/master/.github/workflows)
[6] git-cliff-action: [https://github.com/orhun/git-cliff-action](https://github.com/orhun/git-cliff-action)
[7] upload-release-action: [https://github.com/svenstaro/upload-release-action](https://github.com/svenstaro/upload-release-action)
[8] cross: [https://github.com/cross-rs/cross](https://github.com/cross-rs/cross)
[9] upload-release-asset: [https://github.com/actions/upload-release-asset](https://github.com/actions/upload-release-asset)
