# bat

## Repository Information

- **Repository**: [sharkdp/bat](https://github.com/sharkdp/bat)
- **Date Created**: 2025-11-22
- **Cloned to**: `/workspace/reference/repo/sharkdp/bat/code`
- **Star Count**: 50k+ stars, widely-used cat(1) clone with syntax highlighting

## Purpose

Analyzing bat's combined CICD pipeline for #199. bat is another major Rust CLI tool by sharkdp (author of fd, hyperfine) with sophisticated automation and quality gates.

## Key Findings

### Combined CICD Workflow (.github/workflows/CICD.yml)

**Single Workflow Philosophy:**
- Unlike ripgrep (separate ci.yml and release.yml), bat combines both in one file
- Conditional execution: builds artifacts on every push, but only publishes on tag
- Benefits: DRY (don't repeat yourself), consistent builds between CI and release

**Triggers:**
```yaml
on:
  workflow_dispatch:    # Manual trigger
  pull_request:         # PR checks
  push:
    branches: [master]  # CI on master
    tags: ['*']         # Release on any tag
```
- More flexible than ripgrep: includes workflow_dispatch for manual runs
- Release on any tag (not just vX.Y.Z pattern like ripgrep)

### All-Jobs Gate Pattern

**Critical Innovation:**
```yaml
all-jobs:
  if: always() # Otherwise this job is skipped if the matrix job fails
  name: all-jobs
  runs-on: ubuntu-latest
  needs: [crate_metadata, lint, min_version, license_checks, ...]
  steps:
    - run: jq --exit-status 'all(.result == "success")' <<< '${{ toJson(needs) }}'
```

**Why this matters:**
- Branch protection rules need a single status check to gate merges
- Matrix jobs create multiple status checks (build-linux, build-windows, etc.)
- `all-jobs` aggregates all results into single checkable gate
- Uses `if: always()` to run even if dependencies fail
- jq checks that all needed jobs succeeded

**Recommendation:** Essential pattern for branch protection with matrix jobs

### Crate Metadata Extraction

**DRY Principle:**
```yaml
crate_metadata:
  steps:
    - run: |
        cargo metadata --no-deps --format-version 1 | jq -r '"name=" + .packages[0].name' | tee -a $GITHUB_OUTPUT
        cargo metadata --no-deps --format-version 1 | jq -r '"version=" + .packages[0].version' | tee -a $GITHUB_OUTPUT
        cargo metadata --no-deps --format-version 1 | jq -r '"msrv=" + .packages[0].rust_version' | tee -a $GITHUB_OUTPUT
  outputs:
    name: ${{ steps.crate_metadata.outputs.name }}
    version: ${{ steps.crate_metadata.outputs.version }}
    msrv: ${{ steps.crate_metadata.outputs.msrv }}
```

**Benefits:**
- Single source of truth (Cargo.toml)
- No hardcoded versions in workflow
- Extracts name, version, maintainer, homepage, MSRV
- Used by downstream jobs for package naming

**vs ripgrep:** ripgrep uses `${{ github.ref_name }}` for version (from tag), bat uses Cargo.toml

### CI Quality Gates

**1. lint Job:**
```yaml
lint:
  steps:
    - run: cargo fmt -- --check
    - run: cargo clippy --locked --all-targets --all-features -- -D warnings
```
- **Both rustfmt AND clippy** (ripgrep missing clippy!)
- `--locked`: Ensures Cargo.lock is up-to-date
- `--all-targets`: Checks bins, libs, tests, benches, examples
- `--all-features`: Tests with all feature combinations
- `-D warnings`: Treat warnings as errors

**2. min_version Job:**
```yaml
min_version:
  needs: crate_metadata
  steps:
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ needs.crate_metadata.outputs.msrv }}
    - run: cargo test --locked ${{ env.MSRV_FEATURES }}
```
- Tests MSRV (minimum supported Rust version) from `rust-version` in Cargo.toml
- Ensures package works on oldest advertised Rust version
- Uses minimal feature set for MSRV compatibility

**3. license_checks Job:**
```yaml
license_checks:
  steps:
    - uses: actions/checkout@v5
      with:
        submodules: true # check submodules too
    - run: tests/scripts/license-checks.sh
```
- Custom script for license validation
- Includes submodules (important for syntax highlighting assets)

**4. cargo-audit Job:**
```yaml
cargo-audit:
  steps:
    - run: cargo install cargo-audit --locked
    - run: cargo audit
```
- **Security vulnerability scanning** (ripgrep doesn't have this!)
- Checks dependencies for known CVEs
- Essential for production-grade tools

**5. documentation Job:**
```yaml
documentation:
  steps:
    - env: RUSTDOCFLAGS: -D warnings
      run: cargo doc --locked --no-deps --document-private-items --all-features
    - run: man $(find . -name bat.1)
```
- Validates all documentation compiles without warnings
- Checks man page generation

**6. test_with_new_syntaxes_and_themes Job:**
- bat-specific: tests with updated syntax highlighting assets
- Ensures new syntaxes don't break existing functionality

**7. test_with_system_config Job:**
- bat-specific: tests system-wide configuration loading

### Build and Release Strategy

**Build Matrix:**
```yaml
matrix:
  job:
    - { target: aarch64-unknown-linux-musl, os: ubuntu-latest, dpkg_arch: arm64, use-cross: true }
    - { target: x86_64-apple-darwin, os: macos-13 }
    - { target: aarch64-apple-darwin, os: macos-14 }
    - { target: x86_64-pc-windows-msvc, os: windows-2025 }
    # ... 12 targets total
```
- 12 targets (fewer than ripgrep's 14)
- Includes dpkg_arch for Debian package naming
- `use-cross` flag controls cross-compilation

**Cross Installation:**
```yaml
- name: Install cross
  if: matrix.job.use-cross
  uses: taiki-e/install-action@v2
  with:
    tool: cross
```
- **Better approach than ripgrep**: uses GitHub Action instead of manual curl/tar
- `taiki-e/install-action` handles cross installation cleanly
- Automatically pins stable version

**Build Command Abstraction:**
```yaml
env:
  BUILD_CMD: cargo
steps:
  - name: Overwrite build command env variable
    if: matrix.job.use-cross
    run: echo "BUILD_CMD=cross" >> $GITHUB_ENV
  - run: $BUILD_CMD build --locked --release --target=${{ matrix.job.target }}
```
- Uses `BUILD_CMD` env variable (cargo vs cross)
- All build/test commands use `$BUILD_CMD` for consistency

**Testing Strategy:**
```yaml
- name: Run tests
  run: $BUILD_CMD test --locked --target=${{ matrix.job.target }} ${{ steps.test-options.outputs.CARGO_TEST_OPTIONS}}
- name: Run bat
  run: $BUILD_CMD run --locked --target=${{ matrix.job.target }} -- --paging=never --color=always --theme=ansi Cargo.toml src/config.rs
- name: Show diagnostics (bat --diagnostic)
  run: $BUILD_CMD run --locked --target=${{ matrix.job.target }} -- ... --diagnostic
```
- Tests run for each target
- Smoke test: actually runs bat on sample files
- Diagnostics: ensures --diagnostic flag works

**Feature Flag Testing:**
```yaml
- name: "Feature check: regex-onig"
  run: $BUILD_CMD check --locked --target=${{ matrix.job.target }} --verbose --lib --no-default-features --features regex-onig
- name: "Feature check: minimal-application"
  run: $BUILD_CMD check --locked --target=${{ matrix.job.target }} --verbose --no-default-features --features minimal-application
```
- Tests multiple feature combinations
- Ensures features compile independently
- Validates minimal builds for embedded use cases

### Artifact Packaging

**Tarball Creation:**
```yaml
PKG_BASENAME=${{ needs.crate_metadata.outputs.name }}-v${{ needs.crate_metadata.outputs.version }}-${{ matrix.job.target }}
PKG_NAME=${PKG_BASENAME}.tar.gz  # or .zip for Windows

ARCHIVE_DIR="${PKG_STAGING}/${PKG_BASENAME}/"
cp "$BIN_PATH" "$ARCHIVE_DIR"
cp "README.md" "LICENSE-MIT" "LICENSE-APACHE" "CHANGELOG.md" "$ARCHIVE_DIR"
cp 'target/.../bat.1' "$ARCHIVE_DIR"  # man page
cp 'target/.../bat.bash' "$ARCHIVE_DIR/autocomplete/"  # shell completions
```
- Uses cargo build.rs generated assets (man page, completions)
- Consistent naming: `bat-v0.24.0-x86_64-unknown-linux-gnu.tar.gz`
- Includes all documentation

**Debian Package Creation:**
```yaml
- name: Create Debian package
  if: startsWith(matrix.job.os, 'ubuntu')
  run: |
    DPKG_BASENAME=bat
    DPKG_CONFLICTS=bat-musl
    case ${{ matrix.job.target }} in *-musl)
      DPKG_BASENAME=bat-musl
      DPKG_CONFLICTS=bat
    ;; esac

    install -Dm755 "$BIN_PATH" "${DPKG_DIR}/usr/bin/bat"
    install -Dm644 'target/.../bat.1' "${DPKG_DIR}/usr/share/man/man1/bat.1"
    # ... more install commands

    fakeroot dpkg-deb --build "${DPKG_DIR}" "${DPKG_PATH}"
```
- **Manual dpkg creation** (not cargo-deb like ripgrep)
- Separate packages for musl vs glibc (bat vs bat-musl)
- Conflict declaration prevents both from being installed
- Proper Debian directory structure

**vs ripgrep:** bat uses manual dpkg, ripgrep uses cargo-deb

### Release Publishing

**Conditional Upload:**
```yaml
- name: Check for release
  id: is-release
  run: |
    unset IS_RELEASE
    if [[ $GITHUB_REF =~ ^refs/tags/v[0-9].* ]]; then
      IS_RELEASE='true'
    fi
    echo "IS_RELEASE=${IS_RELEASE}" >> $GITHUB_OUTPUT

- name: Publish archives and packages
  uses: softprops/action-gh-release@v2
  if: steps.is-release.outputs.IS_RELEASE
  with:
    files: |
      ${{ steps.package.outputs.PKG_PATH }}
      ${{ steps.debian-package.outputs.DPKG_PATH }}
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Key differences from ripgrep:**
1. Uses `softprops/action-gh-release` (ripgrep uses gh CLI)
2. Single step uploads all files (ripgrep uploads per-job)
3. No draft release - published immediately
4. No separate create-release job

**Winget Publishing:**
```yaml
winget:
  needs: build
  if: startsWith(github.ref, 'refs/tags/v')
  steps:
    - uses: vedantmgoyal9/winget-releaser@...
      with:
        identifier: sharkdp.bat
        installers-regex: '-pc-windows-msvc\.zip$'
        token: ${{ secrets.WINGET_TOKEN }}
```
- Automatically publishes to Windows Package Manager
- Requires separate PAT token (WINGET_TOKEN)
- Regex filter selects correct Windows artifact

### Changelog Requirement (.github/workflows/require-changelog-for-PRs.yml)

**Pre-merge Check:**
```yaml
check-changelog:
  if: github.actor != 'dependabot[bot]'
  steps:
    - name: Get PR submitter
      run: curl -sSfL https://api.github.com/repos/sharkdp/bat/pulls/${PR_NUMBER} | jq -r '.user.login'

    - name: Search for added line in changelog
      run: |
        ADDED=$(git diff -U0 "origin/${PR_BASE}" HEAD -- CHANGELOG.md | grep -P '^\+[^\+].+$')
        grep "#${PR_NUMBER}\b.*${PR_SUBMITTER}\b" <<< "$ADDED"
```

**Why this matters:**
- Enforces manual CHANGELOG.md updates for all PRs
- Checks format: must include PR number and submitter
- Example: `- Fix bug in parser (#123, @username)`
- Skips dependabot PRs (automerged)

**Trade-off:**
- Manual changelog vs automated (git-cliff)
- Ensures quality descriptions, but adds developer burden
- bat prefers human-written changelogs

### Token Handling

**Uses:** `GITHUB_TOKEN` (standard) + `WINGET_TOKEN` (PAT for Winget)
- GITHUB_TOKEN sufficient for GitHub releases
- WINGET_TOKEN is PAT with repo scope for Winget PRs
- No GitHub App needed

### Version Management

**No version validation job!**
- Unlike ripgrep's explicit tag-vs-Cargo.toml check
- Relies on developers to keep versions in sync
- Cargo metadata job reads version from Cargo.toml, but doesn't validate against tag

**Potential issue:** Could release v1.2.3 tag with Cargo.toml saying 1.2.2

## Architecture

### Workflow Structure

```
.github/workflows/
├── CICD.yml                            # Combined CI and release
└── require-changelog-for-PRs.yml       # Changelog enforcement
```

### CICD Jobs Diagram

```
CICD.yml (on: pull_request, push, workflow_dispatch)
├── crate_metadata (extracts name, version, msrv)
├── lint (rustfmt + clippy)
├── min_version (MSRV testing)
├── license_checks (custom script)
├── test_with_new_syntaxes_and_themes (bat-specific)
├── test_with_system_config (bat-specific)
├── documentation (rustdoc + man page)
├── cargo-audit (security vulnerabilities)
├── build (matrix: 12 targets)
│   ├── Build binary
│   ├── Run tests
│   ├── Run smoke tests (bat on sample files)
│   ├── Feature flag checks (regex-onig, minimal-application, etc.)
│   ├── Create tarball (.tar.gz / .zip)
│   ├── Create Debian package (.deb)
│   ├── Upload artifacts (always)
│   └── Publish to GitHub releases (if tag)
├── winget (if tag)
│   └── Publish to Windows Package Manager
└── all-jobs (aggregates all results for branch protection)
```

### Release Flow

```
Developer pushes vX.Y.Z tag
    ↓
CICD workflow triggered (on: push: tags)
    ↓
All CI quality gates run (lint, test, audit, etc.)
    ↓
Build job: Create artifacts for 12 targets
    ↓
Upload artifacts to GitHub Actions (always)
    ↓
If IS_RELEASE: Publish artifacts to GitHub release
    ↓
Winget job: Publish to Windows Package Manager
```

**No manual approval step** - fully automated on tag push

## Patterns & Anti-Patterns

### Strengths

1. **All-jobs gate pattern** - Essential for branch protection with matrix builds
2. **Cargo metadata extraction** - DRY, single source of truth
3. **Combined CICD** - Consistent builds between CI and release
4. **cargo-audit** - Security vulnerability scanning
5. **MSRV testing** - Ensures advertised compatibility
6. **Clippy in CI** - Catches common mistakes
7. **Feature flag testing** - Validates feature independence
8. **Changelog enforcement** - Human-readable release notes
9. **Winget publishing** - Windows package manager integration
10. **taiki-e/install-action** - Clean cross installation
11. **Smoke tests** - Actually runs the binary on sample files
12. **Manual Debian packages** - Full control over package structure

### Potential Weaknesses

1. **No version validation** - Could tag v1.2.3 with Cargo.toml at 1.2.2
2. **No draft release** - Immediately published (can't review artifacts first)
3. **Manual changelog** - Developer burden, prone to forgetting
4. **No SHA256 checksums** - Unlike ripgrep
5. **Winget requires PAT** - Extra secret management

### Unique Approaches

1. **All-jobs gate** - Aggregates matrix results for branch protection
2. **Changelog requirement** - Enforces manual updates
3. **Smoke tests** - Runs binary on actual files
4. **Feature matrix testing** - Multiple feature combinations
5. **Manual dpkg** - Full control vs cargo-deb
6. **Winget automation** - Windows ecosystem integration

## Recommendations for Langstar #199

### Adopt

1. **All-jobs gate pattern** - Critical for branch protection with matrix builds
2. **Cargo metadata extraction** - Avoid hardcoding versions
3. **Combined CICD workflow** - Simpler than separate files
4. **cargo-audit** - Security scanning (bat has, ripgrep doesn't)
5. **Clippy in CI** - Code quality (bat has, ripgrep doesn't)
6. **MSRV testing** - Ensures compatibility promises
7. **Feature flag testing** - If langstar has multiple features
8. **taiki-e/install-action** - Cleaner than ripgrep's curl/tar
9. **Smoke tests** - Actually run the binary

### Consider

1. **Version validation** - Add check that tag matches Cargo.toml (bat weakness)
2. **Draft releases** - Review before publishing (ripgrep has, bat doesn't)
3. **SHA256 checksums** - Integrity verification (ripgrep has, bat doesn't)
4. **Changelog automation** - git-cliff instead of manual (bat's manual approach adds burden)
5. **workflow_dispatch** - Manual triggering capability

### Skip (for now)

1. **Winget publishing** - Complex setup, additional PAT needed
2. **Manual changelog enforcement** - Use git-cliff instead
3. **Extensive feature matrix** - Only if langstar has many features
4. **Custom license checks** - Standard licenses don't need this

## Code Examples

### All-Jobs Gate Pattern

From `CICD.yml:17-32`:
```yaml
all-jobs:
  if: always() # Run even if dependencies fail
  name: all-jobs
  runs-on: ubuntu-latest
  needs:
    - crate_metadata
    - lint
    - min_version
    - build
    # ... all other jobs
  steps:
    - run: jq --exit-status 'all(.result == "success")' <<< '${{ toJson(needs) }}'
```

**Why this matters:**
- Matrix jobs create many status checks (build-linux, build-windows, etc.)
- Branch protection can only check one status
- This aggregates all into single `all-jobs` check
- Use `if: always()` so it runs even if some jobs fail

**How to use:**
1. Add `all-jobs` job that needs all other jobs
2. Use jq to check all succeeded
3. In branch protection, require only `all-jobs` status check

### Cargo Metadata Extraction

From `CICD.yml:34-52`:
```yaml
crate_metadata:
  steps:
    - run: |
        cargo metadata --no-deps --format-version 1 | jq -r '"name=" + .packages[0].name' | tee -a $GITHUB_OUTPUT
        cargo metadata --no-deps --format-version 1 | jq -r '"version=" + .packages[0].version' | tee -a $GITHUB_OUTPUT
        cargo metadata --no-deps --format-version 1 | jq -r '"msrv=" + .packages[0].rust_version' | tee -a $GITHUB_OUTPUT
  outputs:
    name: ${{ steps.crate_metadata.outputs.name }}
    version: ${{ steps.crate_metadata.outputs.version }}
    msrv: ${{ steps.crate_metadata.outputs.msrv }}
```

**Why this matters:**
- Single source of truth (Cargo.toml)
- No hardcoded versions in workflow
- Automatically extracts MSRV for testing

**How to use:**
1. First job extracts metadata
2. Other jobs use `needs: crate_metadata`
3. Reference via `${{ needs.crate_metadata.outputs.version }}`

### Changelog Enforcement

From `require-changelog-for-PRs.yml:25-33`:
```yaml
- name: Search for added line in changelog
  run: |
    ADDED=$(git diff -U0 "origin/${PR_BASE}" HEAD -- CHANGELOG.md | grep -P '^\+[^\+].+$')
    echo "Added lines in CHANGELOG.md:"
    echo "$ADDED"
    echo "Grepping for PR info:"
    grep "#${PR_NUMBER}\\b.*${PR_SUBMITTER}\\b" <<< "$ADDED"
```

**Why this matters:**
- Enforces human-written changelogs
- Ensures PR number and author in changelog entry
- Example format: `- Fix parser bug (#123, @username)`

**Trade-off:** Manual effort vs quality descriptions

### Cross Installation with taiki-e

From `CICD.yml:196-200`:
```yaml
- name: Install cross
  if: matrix.job.use-cross
  uses: taiki-e/install-action@v2
  with:
    tool: cross
```

**vs ripgrep's approach:**
```yaml
# ripgrep: manual curl + tar
- run: |
    curl -LO "https://github.com/cross-rs/cross/releases/download/$CROSS_VERSION/cross-x86_64-unknown-linux-musl.tar.gz"
    tar xf cross-x86_64-unknown-linux-musl.tar.gz
```

**Why bat's approach is better:**
- Cleaner, more maintainable
- Action handles version pinning
- No manual URL construction

## Comparison: bat vs ripgrep

| Feature | bat | ripgrep |
|---------|-----|---------|
| Workflow structure | Combined CICD | Separate ci.yml + release.yml |
| Quality gates | rustfmt, clippy, audit, docs | rustfmt, docs (no clippy/audit) |
| Clippy | ✅ Yes | ❌ No |
| cargo-audit | ✅ Yes | ❌ No |
| MSRV testing | ✅ Yes | ❌ No |
| Version validation | ❌ No | ✅ Yes (tag vs Cargo.toml) |
| Draft releases | ❌ No (immediate) | ✅ Yes |
| SHA256 checksums | ❌ No | ✅ Yes |
| Changelog | Manual (enforced by CI) | Manual (not enforced) |
| All-jobs gate | ✅ Yes | ❌ No |
| Cross installation | taiki-e action | curl + tar |
| Release action | softprops/action-gh-release | gh CLI |
| Debian packages | Manual dpkg | cargo-deb |
| Winget | ✅ Yes | ❌ No |
| Smoke tests | ✅ Yes (runs binary) | ❌ No |
| Feature flag testing | ✅ Yes | ❌ No |

**Summary:**
- bat: More comprehensive CI checks, better dev experience
- ripgrep: Better release validation, draft releases, checksums

**Best of both:** Adopt bat's CI quality gates + ripgrep's release safety

## References

- bat CICD: `.github/workflows/CICD.yml`
- bat changelog check: `.github/workflows/require-changelog-for-PRs.yml`
- taiki-e/install-action: https://github.com/taiki-e/install-action
- softprops/action-gh-release: https://github.com/softprops/action-gh-release
- cargo-audit: https://github.com/RustSec/rustsec/tree/main/cargo-audit
