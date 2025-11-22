# Rust CLI Release Pipeline Patterns - Synthesis

## Overview

Research for #228 (sub-issue of #199): Analysis of production-grade Rust CLI release pipelines to inform langstar's automated release PR generation with CI checks.

**Projects Analyzed:**
1. **ripgrep** (BurntSushi/ripgrep) - 50k+ stars, grep alternative
2. **bat** (sharkdp/bat) - 50k+ stars, cat clone with syntax highlighting

**Date:** 2025-11-22

## Executive Summary

After deep analysis of ripgrep and bat, two of the most widely-used Rust CLI tools, clear patterns emerge:

**Key Insight:** Combine bat's comprehensive CI quality gates with ripgrep's release safety mechanisms for optimal production pipeline.

### Critical Patterns Identified

1. **All-Jobs Gate** (bat) - Essential for branch protection with matrix builds
2. **Version Validation** (ripgrep) - Prevents mismatched tag/Cargo.toml releases
3. **Cargo Audit** (bat) - Security vulnerability scanning
4. **Clippy in CI** (bat) - Catches common mistakes
5. **Draft Releases** (ripgrep) - Review artifacts before publishing
6. **SHA256 Checksums** (ripgrep) - Artifact integrity verification
7. **Cross-compilation** (both) - Use taiki-e/install-action (bat) over manual curl (ripgrep)
8. **Cargo Metadata Extraction** (bat) - Single source of truth for versions

## Comparison Matrix

| Feature | ripgrep | bat | Recommendation for Langstar |
|---------|---------|-----|----------------------------|
| **Workflow Structure** | Separate (ci.yml + release.yml) | Combined (CICD.yml) | **Combined** (simpler, DRY) |
| **Quality Gates: rustfmt** | ✅ Yes | ✅ Yes | ✅ **Adopt** |
| **Quality Gates: clippy** | ❌ No | ✅ Yes | ✅ **Adopt** (bat's approach) |
| **Quality Gates: cargo-audit** | ❌ No | ✅ Yes | ✅ **Adopt** (security critical) |
| **Quality Gates: docs** | ✅ Yes | ✅ Yes | ✅ **Adopt** |
| **MSRV Testing** | ❌ No | ✅ Yes | ✅ **Adopt** (compatibility guarantee) |
| **Version Validation** | ✅ Yes (tag vs Cargo.toml) | ❌ No | ✅ **Adopt** (ripgrep's approach) |
| **Draft Releases** | ✅ Yes | ❌ No (immediate) | ✅ **Adopt** (ripgrep's approach) |
| **SHA256 Checksums** | ✅ Yes | ❌ No | ✅ **Adopt** (ripgrep's approach) |
| **Changelog** | Manual (not enforced) | Manual (CI-enforced) | **git-cliff** (automated) |
| **All-Jobs Gate** | ❌ No | ✅ Yes | ✅ **Adopt** (bat's pattern) |
| **Cross Installation** | Manual curl+tar | taiki-e action | **taiki-e** (cleaner) |
| **Cargo Metadata** | Uses tag for version | ✅ Extracts from Cargo.toml | ✅ **Adopt** (bat's DRY approach) |
| **Smoke Tests** | ❌ No | ✅ Yes (runs binary) | ✅ **Adopt** (validation) |
| **Feature Flag Testing** | ❌ No | ✅ Yes | Consider (if multi-feature) |

**Legend:**
- ✅ Has feature / Recommended
- ❌ Missing feature / Not recommended

## Detailed Pattern Analysis

### 1. All-Jobs Gate Pattern (bat)

**Problem:** Branch protection can only check one status, but matrix builds create multiple status checks (build-linux, build-windows, etc.)

**Solution:**
```yaml
all-jobs:
  if: always() # Run even if dependencies fail
  runs-on: ubuntu-latest
  needs: [crate_metadata, lint, min_version, build, ...]
  steps:
    - run: jq --exit-status 'all(.result == "success")' <<< '${{ toJson(needs) }}'
```

**Why critical:**
- Aggregates all matrix results into single checkable status
- Branch protection requires only `all-jobs` check
- Uses `if: always()` to run even when jobs fail

**Recommendation:** **MUST HAVE** for langstar

### 2. Version Validation (ripgrep)

**Problem:** Could tag v1.2.3 but Cargo.toml says 1.2.2 (bat has this weakness)

**Solution:**
```yaml
- name: Check that tag version and Cargo.toml version are the same
  run: |
    if ! grep -q "version = \"$VERSION\"" Cargo.toml; then
      echo "version does not match Cargo.toml" >&2
      exit 1
    fi
```

**Why critical:**
- Catches version mismatches before creating release
- Prevents confusing releases
- Simple grep-based validation

**Recommendation:** **MUST HAVE** for langstar

### 3. Draft Releases (ripgrep)

**Problem:** Once published, releases are permanent (bat publishes immediately)

**Solution:**
```yaml
# Job 1: Create draft
- name: Create GitHub release
  run: gh release create $VERSION --draft --verify-tag --title $VERSION

# Job 2: Build artifacts and upload to draft

# Manual step: Review and publish draft
```

**Why critical:**
- Review artifacts before making public
- Fix issues without embarrassing re-releases
- Two-phase: create → build → review → publish

**Recommendation:** **MUST HAVE** for langstar

### 4. cargo-audit Security Scanning (bat)

**Problem:** Dependencies may have known vulnerabilities (ripgrep doesn't check)

**Solution:**
```yaml
cargo-audit:
  steps:
    - run: cargo install cargo-audit --locked
    - run: cargo audit
```

**Why critical:**
- Identifies CVEs in dependencies
- Essential for production-grade tools
- Blocks PR merge if vulnerabilities found

**Recommendation:** **MUST HAVE** for langstar

### 5. Clippy Linting (bat)

**Problem:** Common mistakes slip through (ripgrep only has rustfmt)

**Solution:**
```yaml
lint:
  steps:
    - run: cargo fmt -- --check
    - run: cargo clippy --locked --all-targets --all-features -- -D warnings
```

**Flags explained:**
- `--locked`: Ensures Cargo.lock is up-to-date
- `--all-targets`: Checks bins, libs, tests, examples
- `--all-features`: Tests all feature combinations
- `-D warnings`: Treat warnings as errors

**Recommendation:** **MUST HAVE** for langstar

### 6. Cargo Metadata Extraction (bat)

**Problem:** Hardcoded versions in workflow (DRY violation)

**Solution:**
```yaml
crate_metadata:
  steps:
    - run: |
        cargo metadata --no-deps --format-version 1 | jq -r '"name=" + .packages[0].name' | tee -a $GITHUB_OUTPUT
        cargo metadata --no-deps --format-version 1 | jq -r '"version=" + .packages[0].version' | tee -a $GITHUB_OUTPUT
  outputs:
    name: ${{ steps.crate_metadata.outputs.name }}
    version: ${{ steps.crate_metadata.outputs.version }}
```

**Why valuable:**
- Single source of truth (Cargo.toml)
- No version duplication in workflow
- Automatically extracts MSRV for testing

**Recommendation:** **ADOPT** for langstar

### 7. Cross-Compilation Installation

**ripgrep's approach (manual):**
```yaml
- run: |
    curl -LO "https://github.com/cross-rs/cross/releases/download/$CROSS_VERSION/cross-x86_64-unknown-linux-musl.tar.gz"
    tar xf cross-x86_64-unknown-linux-musl.tar.gz
```

**bat's approach (action):**
```yaml
- uses: taiki-e/install-action@v2
  with:
    tool: cross
```

**Why bat's is better:**
- Cleaner, more maintainable
- Action handles version pinning
- No manual URL construction

**Recommendation:** **Use taiki-e/install-action** for langstar

### 8. Changelog Generation

**Current approaches:**
- **ripgrep**: Manual CHANGELOG.md (not enforced)
- **bat**: Manual CHANGELOG.md (CI-enforced with PR check)

**Problems:**
- Developer burden
- Prone to forgetting
- Inconsistent formatting

**Solution for langstar:**
Use **git-cliff** for automated changelog generation from conventional commits

**Why better:**
- Automated from commit messages
- Consistent formatting
- No manual maintenance
- Langstar already uses Conventional Emoji Commits (#199)

**Recommendation:** **Use git-cliff** (improvement over both)

## Recommended CI/CD Architecture for Langstar

### Workflow Structure

```
.github/workflows/
└── CICD.yml    # Combined CI and CD (bat's approach)
```

**Single workflow with conditional release:**
```yaml
on:
  workflow_dispatch:  # Manual trigger
  pull_request:       # PR checks
  push:
    branches: [main]  # CI on main
    tags: ['v*']      # Release on v* tags
```

### Job Flow

```
CICD.yml
├── crate_metadata (extract version from Cargo.toml)
├── lint (rustfmt + clippy)
├── audit (cargo-audit for security)
├── docs (rustdoc validation)
├── test (unit + integration tests)
├── build (matrix: Linux x86_64, macOS, Windows)
│   ├── Build binary
│   ├── Run smoke tests
│   ├── Create tarball/zip
│   ├── Generate SHA256 checksums
│   └── Upload artifacts
├── release (if tag)
│   ├── Validate: tag == Cargo.toml version
│   ├── Create draft GitHub release
│   ├── Upload artifacts + checksums
│   └── Generate changelog with git-cliff
└── all-jobs (aggregate status for branch protection)
```

### Quality Gates

**Must Pass Before Merge:**
1. ✅ cargo fmt --check
2. ✅ cargo clippy --locked --all-targets --all-features -- -D warnings
3. ✅ cargo test --workspace --all-features
4. ✅ cargo audit
5. ✅ cargo doc --no-deps -D warnings

**Gate via:** `all-jobs` status check in branch protection

### Release Trigger

**On tag push** (v*):
1. Validate tag matches Cargo.toml version
2. Run all CI quality gates
3. Build for all targets
4. Create draft release
5. Upload artifacts + SHA256 checksums
6. Generate changelog with git-cliff
7. **Manual step:** Review draft, then publish

## Implementation Checklist for #199

### Phase 1: CI Quality Gates
- [ ] Add `all-jobs` gate pattern
- [ ] Add cargo clippy to CI
- [ ] Add cargo audit to CI
- [ ] Add MSRV testing (if rust-version in Cargo.toml)
- [ ] Configure branch protection to require `all-jobs` status

### Phase 2: Release Automation
- [ ] Add cargo metadata extraction job
- [ ] Add version validation (tag vs Cargo.toml)
- [ ] Create draft release on tag push
- [ ] Build artifacts for targets (Linux x86_64 initially)
- [ ] Generate SHA256 checksums
- [ ] Install and configure git-cliff for changelog

### Phase 3: Cross-Platform Support (Future)
- [ ] Add macOS build target
- [ ] Add Windows build target
- [ ] Add Linux ARM64 target (if needed)
- [ ] Use taiki-e/install-action for cross-compilation

### Phase 4: Enhancements (Future)
- [ ] Add smoke tests (run binary with --version, --help)
- [ ] Feature flag testing (if multi-feature)
- [ ] Debian package generation (if distributing via apt)

## Code Examples for Implementation

### 1. All-Jobs Gate

```yaml
all-jobs:
  if: always()
  name: All jobs
  runs-on: ubuntu-latest
  needs:
    - crate_metadata
    - lint
    - audit
    - docs
    - test
    - build
  steps:
    - name: Check all job results
      run: |
        jq --exit-status 'all(.result == "success")' <<< '${{ toJson(needs) }}'
```

### 2. Cargo Metadata Extraction

```yaml
crate_metadata:
  name: Extract crate metadata
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Extract crate information
      id: metadata
      run: |
        cargo metadata --no-deps --format-version 1 | jq -r '"name=" + .packages[0].name' | tee -a $GITHUB_OUTPUT
        cargo metadata --no-deps --format-version 1 | jq -r '"version=" + .packages[0].version' | tee -a $GITHUB_OUTPUT
  outputs:
    name: ${{ steps.metadata.outputs.name }}
    version: ${{ steps.metadata.outputs.version }}
```

### 3. Lint Job (rustfmt + clippy)

```yaml
lint:
  name: Code quality
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt,clippy
    - name: Check formatting
      run: cargo fmt --all --check
    - name: Run clippy
      run: cargo clippy --locked --all-targets --all-features -- -D warnings
```

### 4. Security Audit

```yaml
audit:
  name: Security audit
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install cargo-audit
      run: cargo install cargo-audit --locked
    - name: Run security audit
      run: cargo audit
```

### 5. Version Validation

```yaml
validate-version:
  name: Validate version
  runs-on: ubuntu-latest
  needs: crate_metadata
  if: startsWith(github.ref, 'refs/tags/v')
  steps:
    - uses: actions/checkout@v4
    - name: Extract tag version
      id: tag
      run: echo "VERSION=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT
    - name: Check tag matches Cargo.toml
      run: |
        TAG_VERSION="${{ steps.tag.outputs.VERSION }}"
        CARGO_VERSION="${{ needs.crate_metadata.outputs.version }}"
        if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
          echo "❌ Version mismatch: tag=$TAG_VERSION, Cargo.toml=$CARGO_VERSION" >&2
          exit 1
        fi
        echo "✓ Version validation passed: $TAG_VERSION"
```

### 6. Draft Release Creation

```yaml
create-release:
  name: Create draft release
  runs-on: ubuntu-latest
  needs: [validate-version, all-jobs]
  if: startsWith(github.ref, 'refs/tags/v')
  permissions:
    contents: write
  steps:
    - uses: actions/checkout@v4
    - name: Install git-cliff
      uses: taiki-e/install-action@v2
      with:
        tool: git-cliff
    - name: Generate changelog
      run: git-cliff --tag ${{ github.ref_name }} --output RELEASE_CHANGELOG.md
    - name: Create draft release
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        gh release create ${{ github.ref_name }} \
          --draft \
          --title "${{ github.ref_name }}" \
          --notes-file RELEASE_CHANGELOG.md
```

### 7. Build and Upload Artifacts

```yaml
build:
  name: Build ${{ matrix.target }}
  runs-on: ${{ matrix.os }}
  needs: crate_metadata
  strategy:
    matrix:
      include:
        - { target: x86_64-unknown-linux-gnu, os: ubuntu-latest }
        - { target: x86_64-apple-darwin, os: macos-latest }
        - { target: x86_64-pc-windows-msvc, os: windows-latest }
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}

    - name: Build release binary
      run: cargo build --release --locked --target ${{ matrix.target }}

    - name: Create artifact archive
      shell: bash
      run: |
        NAME="${{ needs.crate_metadata.outputs.name }}"
        VERSION="${{ needs.crate_metadata.outputs.version }}"
        TARGET="${{ matrix.target }}"

        if [[ "${{ matrix.os }}" == windows-* ]]; then
          ARCHIVE="$NAME-v$VERSION-$TARGET.zip"
          BIN="target/$TARGET/release/$NAME.exe"
          7z a "$ARCHIVE" "$BIN"
        else
          ARCHIVE="$NAME-v$VERSION-$TARGET.tar.gz"
          BIN="target/$TARGET/release/$NAME"
          tar czf "$ARCHIVE" -C "target/$TARGET/release" "$NAME"
        fi

        shasum -a 256 "$ARCHIVE" > "$ARCHIVE.sha256"

        echo "ARCHIVE=$ARCHIVE" >> $GITHUB_ENV
        echo "CHECKSUM=$ARCHIVE.sha256" >> $GITHUB_ENV

    - name: Upload to draft release
      if: startsWith(github.ref, 'refs/tags/v')
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        gh release upload ${{ github.ref_name }} "$ARCHIVE" "$CHECKSUM"
```

## Token Handling

**Sufficient for langstar:**
- `GITHUB_TOKEN` (automatic, sufficient for releases)
- No PAT needed
- No GitHub App needed

**Permissions:**
```yaml
# CI jobs
permissions:
  contents: read

# Release jobs
permissions:
  contents: write  # Required for creating releases
```

## Branch Protection Configuration

**Require before merge:**
- Status check: `All jobs` (from all-jobs gate)
- Up-to-date branch

**Do NOT require:**
- Individual matrix job statuses (build-linux, build-windows, etc.)
- Use all-jobs aggregation instead

## git-cliff Configuration

Create `.cliff.toml` in repository root:

```toml
[changelog]
header = """
# Changelog\n
All notable changes to this project will be documented in this file.\n
"""
body = """
{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | upper_first }}
    {% for commit in commits %}
        - {{ commit.message | upper_first }} ({{ commit.id | truncate(length=7, end="") }})\
    {% endfor %}
{% endfor %}\n
"""

[git]
conventional_commits = true
filter_unconventional = true
commit_parsers = [
  { emoji = "✨", group = "Features" },
  { emoji = "🩹", group = "Bug Fixes" },
  { emoji = "📚", group = "Documentation" },
  { emoji = "⚡", group = "Performance" },
  { emoji = "♻️", group = "Refactor" },
  { emoji = "🧪", group = "Testing" },
  { emoji = "🔧", group = "Build" },
]
```

## Comparison to Existing Langstar CI

**Current (.github/workflows/check.yml):**
- ✅ cargo fmt --check
- ✅ cargo check --workspace
- ✅ cargo clippy --workspace -- -D warnings
- ✅ cargo test --workspace
- ❌ No cargo audit
- ❌ No all-jobs gate
- ❌ No release automation

**Improvements needed:**
1. Add cargo-audit job
2. Add all-jobs aggregation gate
3. Add release workflow (draft creation, artifacts, checksums)
4. Add cargo metadata extraction
5. Add version validation
6. Configure git-cliff for changelog

## Resources

### Documentation
- [All-jobs gate pattern](https://docs.github.com/en/actions/using-jobs/using-jobs-in-a-workflow#defining-prerequisite-jobs)
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
- [git-cliff](https://git-cliff.org/)
- [taiki-e/install-action](https://github.com/taiki-e/install-action)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)

### Reference Implementations
- ripgrep: `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- bat: `.github/workflows/CICD.yml`

## Next Steps

1. **Review synthesis with team** - Validate recommendations
2. **Implement Phase 1** - CI quality gates and all-jobs pattern
3. **Test with manual tag push** - Validate release automation
4. **Implement Phases 2-3** - Full release automation
5. **Document process** - Update langstar docs with release workflow

---

**Research conducted by:** Claude Code
**Date:** 2025-11-22
**Issue:** #228 (sub-issue of #199)
