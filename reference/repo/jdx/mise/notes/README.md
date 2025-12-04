# mise

## Repository Information

- **Repository**: [jdx/mise](https://github.com/jdx/mise)
- **Date Created**: 2025-12-04
- **Cloned to**: `/workspace/reference/repo/jdx/mise/code`

## Purpose

Studying mise's self-update implementation as a **production reference** for langstar's auto-update feature (#541). Unlike ripgrep/eza/bat/fd/starship (which have no self-update), mise has a complete, battle-tested implementation using the `self_update` crate.

**Source**: Initial research via [DeepWiki Q&A](https://deepwiki.com/search/does-mise-have-a-command-line_e9362074-9c1b-4e63-9d21-136f721dcd6b), verified against local clone.

**Why mise matters for langstar:**
- Uses the same `self_update` crate we plan to use
- Handles package manager detection elegantly
- Implements binary signature verification
- Has macOS codesign integration
- Feature-gated implementation pattern

## Key Findings

### 1. Self-Update Implementation Overview

Location: `src/cli/self_update.rs`

**Command signature:**
```
mise self-update [OPTIONS] [VERSION]
```

**Options:**
- `-f, --force` - Update even if already up to date
- `-y, --yes` - Skip confirmation prompt
- `--no-plugins` - Disable auto-updating plugins
- `[VERSION]` - Update to specific version (optional)

### 2. self_update Crate Configuration

**Dependency** (from `Cargo.toml:176-190`):
```toml
# Unix
self_update = { version = "0.42", optional = true, default-features = false, features = [
  "archive-tar",
  "compression-flate2",
  "signatures",
] }

# Windows
self_update = { version = "0.42", optional = true, default-features = false, features = [
  "archive-zip",
  "compression-zip-deflate",
  "signatures",
] }
```

**Key insight**: Different features for Unix vs Windows - tar.gz vs zip archives.

### 3. Core Update Logic

From `src/cli/self_update.rs:105-155`:

```rust
fn do_update(&self) -> Result<Status> {
    let mut update = Update::configure();

    // Use GitHub API token if available (higher rate limits)
    if let Some(token) = &*env::GITHUB_TOKEN {
        update.auth_token(token);
    }

    // Platform-specific binary path in archive
    #[cfg(windows)]
    let bin_path_in_archive = "mise/bin/mise.exe";
    #[cfg(not(windows))]
    let bin_path_in_archive = "mise/bin/mise";

    update
        .repo_owner("jdx")
        .repo_name("mise")
        .bin_name("mise")
        .current_version(cargo_crate_version!())
        .bin_path_in_archive(bin_path_in_archive);

    // Build target string: {OS}-{ARCH}[-musl]
    let target = format!("{}-{}", *OS, *ARCH);
    #[cfg(target_env = "musl")]
    let target = format!("{target}-musl");

    // Final archive name: mise-{version}-{target}.{tar.gz|zip}
    #[cfg(windows)]
    let target = format!("mise-{v}-{target}.zip");
    #[cfg(not(windows))]
    let target = format!("mise-{v}-{target}.tar.gz");

    let status = update
        .verifying_keys([*include_bytes!("../../zipsign.pub")])  // Signature verification!
        .show_download_progress(true)
        .target(&target)
        .no_confirm(settings.is_ok_and(|s| s.yes) || self.yes)
        .build()?
        .update()?;

    // macOS-specific signature verification
    #[cfg(target_os = "macos")]
    if status.updated() {
        Self::verify_macos_signature(&env::MISE_BIN)?;
    }

    Ok(status)
}
```

### 4. Package Manager Detection (Critical Pattern!)

**Problem**: Self-update should NOT run when mise was installed via apt/dnf/brew.

**Solution**: Multi-layered detection in `src/cli/self_update.rs:157-164` and `src/env.rs:211-247`:

```rust
pub fn is_available() -> bool {
    // 1. Explicit override via environment variable
    if let Some(b) = *env::MISE_SELF_UPDATE_AVAILABLE {
        return b;
    }
    // 2. Check for disable marker file
    let has_disable = env::MISE_SELF_UPDATE_DISABLED_PATH.is_some();
    // 3. Check for package manager instructions file
    let has_instructions = env::MISE_SELF_UPDATE_INSTRUCTIONS.is_some();
    !(has_disable || has_instructions)
}
```

**Detection mechanism** (env.rs):
- Searches for `lib/mise-self-update-instructions.toml` near binary
- Searches for `lib/.disable-self-update` marker file
- Package builds create these files (see `scripts/build-deb.sh`)

**Example instructions file** (created by `scripts/build-deb.sh:7-9`):
```toml
message = "To update mise from the APT repository, run:\n\n  sudo apt update && sudo apt install --only-upgrade mise\n"
```

### 5. Feature-Gated Implementation

From `src/cli/mod.rs:55`:
```rust
#[cfg_attr(not(feature = "self_update"), path = "self_update_stub.rs")]
pub mod self_update;
```

**Pattern**: When `self_update` feature is disabled (e.g., package manager builds), a stub module is used instead. This prevents the command from being available at all.

### 6. macOS Codesign Verification

From `src/cli/self_update.rs:166-205`:

```rust
#[cfg(target_os = "macos")]
fn verify_macos_signature(binary_path: &Path) -> Result<()> {
    let output = Command::new("codesign")
        .args([
            "--verify",
            "--deep",
            "--strict",
            "-R=identifier \"dev.jdx.mise\"",
        ])
        .arg(binary_path)
        .output()?;

    if !output.status.success() {
        bail!("macOS binary signature verification failed...");
    }
    Ok(())
}
```

**Key**: Uses hardcoded identifier `dev.jdx.mise` for verification.

### 7. Post-Update Plugin Refresh

From `src/cli/self_update.rs:98-100`:
```rust
if !self.no_plugins {
    cmd!(&*env::MISE_BIN, "plugins", "update").run()?;
}
```

**Pattern**: After self-update, optionally update related components.

## Architecture

```
src/cli/
├── self_update.rs        # Main implementation
├── self_update_stub.rs   # Stub for non-self-update builds
└── mod.rs                # Feature-gated module selection

src/env.rs                # Environment variable handling for:
                          # - MISE_SELF_UPDATE_INSTRUCTIONS
                          # - MISE_SELF_UPDATE_AVAILABLE
                          # - MISE_SELF_UPDATE_DISABLED_PATH

zipsign.pub               # Public key for signature verification

scripts/
├── build-deb.sh          # Creates mise-self-update-instructions.toml
├── build-rpm.sh          # Creates disable marker for RPM
└── ...                   # Other package scripts
```

## Patterns to Adopt for Langstar

### High Priority

| Pattern | Location | Langstar Adaptation |
|---------|----------|---------------------|
| Package manager detection | `is_available()` | Check for devcontainer feature install |
| Signature verification | `verifying_keys()` | Add zipsign.pub to langstar |
| GitHub token support | `auth_token()` | Use existing GITHUB_TOKEN handling |
| Feature-gated build | `cfg_attr` | Optional self_update feature |

### Medium Priority

| Pattern | Location | Langstar Adaptation |
|---------|----------|---------------------|
| macOS codesign verification | `verify_macos_signature()` | Future if we add macOS signing |
| Post-update hooks | `plugins update` | Consider cache refresh |
| Instructions file | `mise-self-update-instructions.toml` | Create for devcontainer feature |

### Lower Priority

| Pattern | Location | Langstar Adaptation |
|---------|----------|---------------------|
| Archive format per-platform | `Cargo.toml` features | Already have tar.gz only |
| Disable marker file | `.disable-self-update` | May not need |

## Comparison: mise vs langstar Scout Recommendations

| Aspect | mise Implementation | Scout Recommendation | Delta |
|--------|---------------------|---------------------|-------|
| Library | `self_update` v0.42 | `self_update` (latest) | Aligned |
| Signature verification | zipsign.pub | SHA256 checksums | mise is stronger |
| Package manager detection | File-based markers | Context detection | mise pattern is better |
| macOS verification | codesign check | Not mentioned | Add to langstar |
| Feature gating | Compile-time | Not mentioned | Good pattern to adopt |

## Notes

### Differences from Langstar's Situation

1. **mise has plugins** - The `--no-plugins` flag and post-update plugin refresh don't apply to langstar
2. **mise has broader distribution** - apt, dnf, brew, cargo, etc. Langstar is primarily GitHub Releases + devcontainer
3. **mise has code signing** - zipsign + macOS codesign. Langstar has SHA256 checksums

### Key Takeaways for Implementation

1. **Use file-based markers** for detecting package manager installs (better than path heuristics)
2. **Consider feature-gating** the self_update dependency for smaller builds
3. **Add signature verification** using `verifying_keys()` - this is better than just checksums
4. **macOS codesign** is a nice-to-have but not essential for initial implementation
5. **GitHub token passthrough** is essential for rate limit handling

### Version Compatibility

mise uses `self_update` v0.42. The langstar scout report didn't specify a version. Should verify v0.42 is compatible with our asset naming.

## Related Research

- [DeepWiki Q&A: mise self-update implementation](https://deepwiki.com/search/does-mise-have-a-command-line_e9362074-9c1b-4e63-9d21-136f721dcd6b) - Source for initial research
- [Issue #542: Rust project self-update precedents](https://github.com/codekiln/langstar/issues/542)
- [Issue #541: Auto-update milestone](https://github.com/codekiln/langstar/issues/541)
- [Scout Report: Langstar Auto-Update Feasibility](../../../../../docs/research/535-langstar-auto-update-scout.md)
- [Research Addendum: Rust Self-Update Precedents](../../../../../docs/research/542-rust-self-update-precedents.md)
