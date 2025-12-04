# Scout Report: Langstar Auto-Update Feasibility

**Issue**: #535
**Date**: 2025-12-04
**Status**: Complete

---

## Executive Summary

**Recommendation: GO** - Self-update functionality is highly feasible for langstar.

The existing infrastructure (GitHub releases, install.sh script, SHA256 checksums) provides an excellent foundation. The `self_update` Rust crate offers a mature, well-maintained solution that integrates directly with GitHub releases. Implementation complexity is **small** (1-2 phases).

---

## 1. Existing Langstar Analysis

### Current Version Management

- **Version source**: `Cargo.toml` workspace (`version = "0.13.0"`)
- **CLI access**: `langstar version` command using `env!("CARGO_PKG_VERSION")`
- **No existing update mechanism**: The CLI has no self-update functionality

### Current Distribution Channels

| Channel | Description | Update Method |
|---------|-------------|---------------|
| GitHub Releases | Pre-built binaries | Manual download or install.sh |
| install.sh | Official installer script | Re-run with `--version` |
| Devcontainer Feature | For development environments | Feature version bump |

### GitHub Release Assets (v0.13.0)

```
langstar-0.13.0-x86_64-linux-musl.tar.gz      + .sha256
langstar-0.13.0-aarch64-linux-musl.tar.gz     + .sha256
langstar-0.13.0-aarch64-macos.tar.gz          + .sha256
```

### install.sh Capabilities

The existing `scripts/install.sh` already supports:
- `--version VERSION` - Install specific version (e.g., `--version 0.2.0`)
- `--prefix DIR` - Custom installation directory
- Platform auto-detection (Linux x86_64/aarch64, macOS x86_64/arm64)
- SHA256 checksum verification
- Upgrade detection (compares installed vs target version)

**Key insight**: The install.sh script is essentially a complete update mechanism. Self-update could invoke it or reimplement its logic in Rust.

---

## 2. Self-Update Pattern Analysis

### Recommended Library: `self_update` Crate

**Repository**: https://github.com/jaemk/self_update
**Documentation**: https://docs.rs/self_update/latest/self_update/

#### Features

| Feature | Description |
|---------|-------------|
| GitHub Backend | Native GitHub Releases API integration |
| Archive Support | tar.gz and zip extraction |
| Binary Replacement | Uses `self_replace` crate for atomic replacement |
| Version Comparison | Semver-based comparison with current version |
| Checksum Verification | Optional SHA256/signatures via `signatures` feature |
| Progress Indicators | Built-in download progress via `indicatif` |

#### Basic Usage Pattern

```rust
use self_update::backends::github::Update;

let status = Update::configure()
    .repo_owner("codekiln")
    .repo_name("langstar")
    .bin_name("langstar")
    .current_version(cargo_crate_version!())
    .build()?
    .update()?;

println!("Update status: {:?}", status);
```

#### Asset Naming Convention

The `self_update` crate expects assets in the format:
```
<asset>-<semver>-<platform>.<ext>
```

Current langstar releases use:
```
langstar-0.13.0-x86_64-linux-musl.tar.gz
```

**Compatibility**: Langstar's existing naming convention is compatible with `self_update`.

### Alternative Approaches

| Approach | Pros | Cons |
|----------|------|------|
| `self_update` crate | Mature, GitHub-native, minimal code | Additional dependency |
| Shell out to install.sh | Zero new dependencies, battle-tested | Platform-specific, shell dependency |
| Custom implementation | Full control | Significant development effort |

**Recommendation**: Use `self_update` crate for the cleanest, most maintainable solution.

---

## 3. Claude CLI Update Patterns

Claude Code's update mechanism provides UX inspiration:

### Commands
- `claude update` - Manual update to latest stable
- Background auto-updates every 10 minutes
- `DISABLE_AUTOUPDATER=1` to disable auto-updates

### UX Patterns Worth Adopting

1. **Non-blocking background checks** - Don't interrupt user workflow
2. **Version comparison output** - Show current vs available
3. **Graceful degradation** - Continue working even if update fails
4. **Diagnostic command** - `claude doctor` for troubleshooting

### Proposed Langstar Commands

```bash
langstar update              # Update to latest stable
langstar update latest       # Same as above (explicit)
langstar update v0.12.0      # Install specific version
langstar update --check      # Check for updates without installing
```

---

## 4. Platform & Distribution Considerations

### Currently Supported Platforms

| Platform | Target | Status |
|----------|--------|--------|
| Linux x86_64 | x86_64-unknown-linux-musl | Released |
| Linux ARM64 | aarch64-unknown-linux-musl | Released |
| macOS ARM64 | aarch64-apple-darwin | Released |
| macOS x86_64 | x86_64-apple-darwin | TODO in workflow |
| Windows | x86_64-pc-windows-msvc | TODO in workflow |

### Platform Detection

The `self_update` crate handles platform detection automatically using Rust's `target_triple`. For langstar's custom platform naming (e.g., `x86_64-linux-musl` vs `x86_64-unknown-linux-musl`), a mapping function will be needed:

```rust
fn get_target_platform() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-linux-musl";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "aarch64-linux-musl";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-macos";
    // ... etc
}
```

### Installation Context Considerations

| Context | Self-Update Appropriate? | Notes |
|---------|-------------------------|-------|
| User-local (`~/.local/bin`) | Yes | User has write permission |
| System-wide (`/usr/local/bin`) | Conditional | May need sudo |
| Devcontainer Feature | No | Should use feature version |
| cargo install | Conditional | Prefer `cargo install langstar` |

**Recommendation**: Self-update should detect installation context and warn or refuse if updating would fail (e.g., no write permissions).

---

## 5. Security Considerations

### Current Security Measures

1. **SHA256 checksums** - Every release asset has `.sha256` file
2. **HTTPS downloads** - Enforced in install.sh
3. **GitHub Release API** - Trusted source

### Additional Security Options

| Option | Effort | Value |
|--------|--------|-------|
| Verify checksums | Low | High - already have .sha256 files |
| GPG signatures | Medium | Medium - adds complexity |
| Code signing (macOS) | High | Medium - notarization requirements |

**Recommendation**: Implement checksum verification (already supported by install.sh, leverage existing .sha256 files).

---

## 6. Complexity Assessment

### Technical Challenges

| Challenge | Difficulty | Mitigation |
|-----------|------------|------------|
| Binary replacement while running | Low | `self_replace` handles this |
| Platform detection | Low | Compile-time conditionals |
| Permission issues | Medium | Detect and warn user |
| Network failures | Low | Retry logic, clear errors |
| Checksum verification | Low | Already have .sha256 files |

### Estimated Scope

**Small (1-2 implementation phases)**

1. **Phase 1**: Basic `langstar update` command
   - Add `self_update` dependency
   - Implement update subcommand
   - Version checking and download
   - Basic error handling

2. **Phase 2**: Polish and edge cases
   - `--check` flag for dry-run
   - Specific version installation
   - Permission detection and warnings
   - Integration tests

### Prerequisites

None identified. All infrastructure exists.

### Potential Blockers

| Blocker | Risk | Mitigation |
|---------|------|------------|
| musl binary compatibility | Low | Already releasing musl builds |
| GitHub rate limiting | Low | Use authenticated requests if needed |
| macOS Gatekeeper | Medium | Consider code signing later |

---

## 7. Open Questions for Implementation

1. **Auto-update vs manual only?**
   - Start with manual `langstar update` command
   - Auto-update could be Phase 3

2. **What happens if update fails mid-download?**
   - `self_update` handles atomic replacement
   - Original binary preserved until new one verified

3. **Should devcontainer installs be excluded?**
   - Detect via environment variable or path
   - Warn but don't block

4. **Version rollback support?**
   - Defer to future phase
   - User can use `langstar update v0.X.Y` for now

---

## Deliverables Checklist

### Research Report (This Document)

- [x] Executive summary with go/no-go recommendation
- [x] Existing langstar analysis (installation and versioning)
- [x] Self-update pattern analysis (Rust ecosystem)
- [x] Claude CLI update UX patterns
- [x] Platform and distribution considerations
- [x] Security considerations
- [x] Complexity assessment
- [x] Open questions for implementation

### Optional: Proof-of-Concept

Not required - the `self_update` crate is well-documented with working examples. A PoC would add minimal value given the low complexity.

---

## Recommendation

### Verdict: GO

Self-update is technically feasible and well-aligned with langstar's architecture:

1. **Infrastructure Ready**: GitHub releases, checksums, platform binaries all exist
2. **Library Mature**: `self_update` crate is battle-tested
3. **Scope Small**: 1-2 phases of work
4. **No Blockers**: No significant technical or architectural barriers

### Suggested Next Steps

1. Create GitHub milestone: `ls-auto-update`
2. Create parent issue following epic format
3. Create Phase 1 issue: Basic update command
4. Create Phase 2 issue: Polish and edge cases
5. (Optional) Phase 3: Auto-update background checks

### High-Level Phase Breakdown

```
Phase 1: Basic Update Command (~1-2 days)
├── Add self_update dependency with required features
├── Create cli/src/commands/update.rs
├── Implement Update subcommand in main.rs
├── Add platform-to-asset mapping
├── Basic success/failure output
└── Unit tests for version comparison

Phase 2: Polish & Edge Cases (~1-2 days)
├── --check flag for update availability
├── Specific version installation (langstar update v0.12.0)
├── Permission detection and user-friendly errors
├── Network timeout/retry handling
├── Integration tests
└── Documentation updates

Phase 3 (Optional): Auto-Update (~2-3 days)
├── Background update checks
├── User notification on available updates
├── Configuration option to disable
└── Enterprise policy support
```

---

## References

- [self_update crate docs](https://docs.rs/self_update/latest/self_update/)
- [self_update GitHub](https://github.com/jaemk/self_update)
- [Langstar install.sh](../scripts/install.sh)
- [Langstar release workflow](../.github/workflows/release.yml)
- [Claude Code update documentation](https://docs.anthropic.com/en/docs/claude-code)
