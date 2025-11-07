# CI/CD Pipeline Documentation

## Overview

This project uses a **validate-on-PR, release-on-tag** CI/CD pipeline strategy, which is the industry best practice for Rust CLI projects. This approach ensures high standards of artifact provenance, reproducibility, traceability, and rollback safety.

## Pipeline Architecture

### CI Workflow (`.github/workflows/ci.yml`)

**Triggers**: Pull requests and pushes to `main`

**Purpose**: Validate code quality and correctness

**Jobs**:
1. **Check** - Formatting and compilation checks
   - `cargo fmt --check` - Verify code formatting
   - `cargo check` - Verify code compiles

2. **Test** - Run test suite
   - `cargo test --workspace` - Run all tests

3. **Clippy** - Linting and code quality
   - `cargo clippy -- -D warnings` - Treat warnings as errors

4. **Build** - Cross-platform build verification
   - Build on Ubuntu and macOS
   - Upload artifacts for manual testing

### Release Workflow (`.github/workflows/release.yml`)

**Triggers**: Version tags matching `v*` (e.g., `v1.2.3`)

**Purpose**: Build release artifacts and publish GitHub releases

**Jobs**:
1. **Create Release**
   - Generate changelog using `git-cliff`
   - Create GitHub Release with changelog
   - Determine if pre-release based on version suffix

2. **Build Release**
   - Build binaries for multiple platforms:
     - `x86_64-unknown-linux-musl` (static Linux)
     - `x86_64-unknown-linux-gnu` (dynamic Linux)
     - `x86_64-apple-darwin` (macOS Intel)
     - `aarch64-apple-darwin` (macOS Apple Silicon)
     - `x86_64-pc-windows-msvc` (Windows)
   - Strip binaries (Unix only)
   - Create archives (.tar.gz for Unix, .zip for Windows)
   - Generate SHA256 checksums
   - Upload to GitHub Release

## Release Process

### Prerequisites

1. **Tools** (pre-installed in devcontainer):
   - `cargo-release` - Version management and release automation
   - `git-cliff` - Changelog generation from conventional commits

   **Note**: If not using the devcontainer, install manually:
   ```bash
   cargo install cargo-release git-cliff
   ```

2. **Ensure clean state**:
   - All changes committed
   - On `main` branch
   - Working directory clean
   - All tests passing

### Creating a Release

#### Option 1: Using cargo-release (Recommended)

```bash
# Dry-run to preview changes
cargo release --dry-run

# Create a patch release (0.1.0 → 0.1.1)
cargo release patch --execute

# Create a minor release (0.1.0 → 0.2.0)
cargo release minor --execute

# Create a major release (0.1.0 → 1.0.0)
cargo release major --execute
```

**What cargo-release does**:
1. Updates version in `Cargo.toml`
2. Runs git-cliff to generate changelog
3. Creates a git commit with version bump
4. Creates an annotated git tag (e.g., `v1.2.3`)
5. Pushes commit and tag to GitHub
6. GitHub Actions automatically triggers release workflow

#### Option 2: Manual Release

```bash
# 1. Update version in Cargo.toml manually
vim Cargo.toml  # Change version field

# 2. Generate changelog
git-cliff --tag v1.2.3 --output CHANGELOG.md

# 3. Commit changes
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "🔖 release: bump version to v1.2.3

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 4. Create annotated tag
git tag -a v1.2.3 -m "Release v1.2.3"

# 5. Push commit and tag
git push && git push origin v1.2.3
```

**What happens next**:
- GitHub Actions detects the tag
- Release workflow builds binaries for all platforms
- GitHub Release is created with changelog and binaries

### Pre-Releases

For alpha, beta, or release candidate versions:

```bash
# Using cargo-release
cargo release --pre-release alpha --execute  # Creates v1.0.0-alpha.1
cargo release --pre-release beta --execute   # Creates v1.0.0-beta.1
cargo release --pre-release rc --execute     # Creates v1.0.0-rc.1

# Manual
git tag -a v1.0.0-alpha.1 -m "Release v1.0.0-alpha.1"
git push origin v1.0.0-alpha.1
```

Pre-releases are automatically detected and marked appropriately on GitHub.

## Configuration Files

### `cliff.toml` - Changelog Generation

Configures `git-cliff` to generate changelogs from Conventional Emoji Commits.

**Key features**:
- Parses commits with emojis and conventional format
- Groups commits by type (Features, Bug Fixes, etc.)
- Links to GitHub PRs automatically
- Prioritizes breaking changes at the top

**Testing locally**:
```bash
# Preview changelog for next release
git-cliff --unreleased

# Preview changelog for specific tag
git-cliff --tag v1.2.3

# Generate changelog and update CHANGELOG.md
git-cliff --tag v1.2.3 --output CHANGELOG.md
```

### `release.toml` - Version Management

Configures `cargo-release` for automated version bumping.

**Key settings**:
- Disables crates.io publishing (GitHub releases only)
- Enables git-cliff integration for changelog
- Configures tag format (`v{{version}}`)
- Allows release from `main` branch
- Pushes tags automatically

**Testing locally**:
```bash
# Dry-run to see what would happen
cargo release patch --dry-run

# See verbose output
cargo release patch --dry-run --verbose
```

## Semantic Versioning

Version bumps are determined by commit types following Conventional Emoji Commits:

| Commit Type | Version Bump | Example |
|-------------|--------------|---------|
| `🚨 BREAKING CHANGE` or `BREAKING CHANGE:` in body | **MAJOR** (1.0.0 → 2.0.0) | Breaking API changes |
| `!` before `:` in subject | **MAJOR** (1.0.0 → 2.0.0) | `feat!: redesign API` |
| `✨ feat` | **MINOR** (1.0.0 → 1.1.0) | New features |
| `🩹 fix`, `⚡️ perf` | **PATCH** (1.0.0 → 1.0.1) | Bug fixes, performance |
| Other types (docs, style, refactor, test, build, ci, chore) | **No bump** | Non-releasable changes |

**Priority**: When multiple commits exist, use the highest priority: MAJOR > MINOR > PATCH

**Examples**:
```bash
# PATCH release (bug fix)
git commit -m "🩹 fix: resolve memory leak in client"
cargo release patch --execute

# MINOR release (new feature)
git commit -m "✨ feat: add deployment assistant support"
cargo release minor --execute

# MAJOR release (breaking change)
git commit -m "🚨 BREAKING CHANGE: remove deprecated API

Old endpoints /api/v1/* are no longer supported.
Migrate to /api/v2/* endpoints."
cargo release major --execute
```

## Troubleshooting

### Release workflow failed to build

**Check**:
1. Does the code compile locally? `cargo build --release`
2. Are all tests passing? `cargo test --workspace`
3. Check the Actions logs on GitHub for specific errors

**Fix**:
1. Fix the issue locally
2. Create a new patch version
3. Push the new tag

### Changelog is empty or incorrect

**Check**:
1. Are commits following Conventional Emoji Commits format?
2. Test locally: `git-cliff --unreleased`

**Fix**:
1. Verify `cliff.toml` configuration
2. Check commit messages: `git log --oneline`
3. Ensure commits have proper emoji/type prefixes

### cargo-release fails with "working directory is dirty"

**Check**:
```bash
git status
```

**Fix**:
```bash
# Commit or stash uncommitted changes
git add .
git commit -m "🔧 build: prepare for release"

# Or stash
git stash
```

### Tag was pushed but release workflow didn't trigger

**Check**:
1. Was it an annotated tag? `git tag -v v1.2.3`
2. Does the tag match `v*` pattern?
3. Check Actions tab on GitHub

**Fix**:
1. Delete and recreate as annotated tag:
   ```bash
   git tag -d v1.2.3
   git push origin :refs/tags/v1.2.3
   git tag -a v1.2.3 -m "Release v1.2.3"
   git push origin v1.2.3
   ```

### Need to undo a release

**Before pushing** (tag is local only):
```bash
git tag -d v1.2.3
git reset --hard HEAD~1  # If you made a commit
```

**After pushing** (tag is on GitHub):
```bash
# 1. Delete GitHub Release
gh release delete v1.2.3 --yes

# 2. Delete remote tag
git push origin :refs/tags/v1.2.3

# 3. Delete local tag
git tag -d v1.2.3

# 4. Revert release commit (if needed)
git revert HEAD
git push

# 5. Fix issues and create new release
```

## Best Practices

### Before Each Release

1. **Run full checks locally**:
   ```bash
   cargo fmt && \
   cargo check --workspace --all-features && \
   cargo clippy --workspace --all-features -- -D warnings && \
   cargo test --workspace --all-features
   ```

2. **Review commits since last release**:
   ```bash
   git log $(git describe --tags --abbrev=0)..HEAD --oneline
   ```

3. **Preview changelog**:
   ```bash
   git-cliff --unreleased
   ```

4. **Dry-run release**:
   ```bash
   cargo release patch --dry-run
   ```

### Commit Message Quality

- Follow Conventional Emoji Commits format strictly
- Use descriptive commit messages
- Include scope when helpful: `feat(cli):` vs `feat:`
- Document breaking changes thoroughly
- Reference issue numbers: `Fixes #42`

### Release Cadence

- **Patch releases**: Bug fixes, performance improvements (as needed)
- **Minor releases**: New features, non-breaking changes (monthly or as needed)
- **Major releases**: Breaking changes (carefully planned, with migration guides)

### Security

- Never commit secrets to the repository
- GitHub Actions uses `GITHUB_TOKEN` with minimal permissions
- Release artifacts include SHA256 checksums for verification
- Consider GPG signing tags for critical releases

## Monitoring

### Check CI Status

- **PR Status**: Check the PR page for green checkmarks
- **Actions Tab**: https://github.com/codekiln/langstar/actions
- **Release Page**: https://github.com/codekiln/langstar/releases

### Verify Releases

After creating a release:

1. **Check GitHub Release page** for new release
2. **Download and verify artifacts**:
   ```bash
   # Download release
   wget https://github.com/codekiln/langstar/releases/download/v1.2.3/langstar-1.2.3-x86_64-linux-musl.tar.gz
   wget https://github.com/codekiln/langstar/releases/download/v1.2.3/langstar-1.2.3-x86_64-linux-musl.tar.gz.sha256

   # Verify checksum
   sha256sum -c langstar-1.2.3-x86_64-linux-musl.tar.gz.sha256
   ```

3. **Test the binary**:
   ```bash
   tar xzf langstar-1.2.3-x86_64-linux-musl.tar.gz
   ./langstar --version
   ./langstar --help
   ```

## References

- [Conventional Emoji Commits](https://conventional-emoji-commits.site/)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [cargo-release Documentation](https://github.com/crate-ci/cargo-release)
- [git-cliff Documentation](https://git-cliff.org/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Issue #9: Release Automation Research](https://github.com/codekiln/langstar/issues/9)
