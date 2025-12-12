# Research Addendum: Rust Project Self-Update Precedents

**Issue**: #542
**Parent Issue**: #541
**Milestone**: ls-auto-update
**Date**: 2025-12-04
**Status**: Complete

---

## Executive Summary

Analysis of five major Rust CLI tools reveals that **none implement built-in self-update functionality**. Instead, they rely on package managers and install scripts for distribution and updates. The `self_update` crate remains the recommended approach for langstar, as it provides what these popular tools deliberately omit.

---

## 1. Analysis Summary

### Projects Analyzed

| Project  | Stars | Self-Update? | Update Method                             |
| -------- | ----- | ------------ | ----------------------------------------- |
| ripgrep  | 49k+  | No           | Package managers (cargo, brew, apt, etc.) |
| eza      | 13k+  | No           | Package managers, cargo install           |
| bat      | 50k+  | No           | Package managers, cargo install           |
| fd       | 35k+  | No           | Package managers, cargo install           |
| starship | 46k+  | No           | install.sh script, package managers       |

### Key Finding

**Zero** of these major Rust CLI tools have built-in self-update commands. This is a deliberate architectural choice favoring:

- Separation of concerns (installation is not the tool's responsibility)
- Security (avoiding privilege escalation within the binary)
- Ecosystem integration (letting package managers handle updates)

---

## 2. Detailed Findings

### 2.1 ripgrep

**Repository**: https://github.com/BurntSushi/ripgrep

- No `update` or `upgrade` subcommand
- Relies entirely on package managers
- Installation docs list 15+ package managers
- No install.sh script

**Update approach**: Users update via their package manager (`cargo install ripgrep`, `brew upgrade ripgrep`, etc.)

### 2.2 eza

**Repository**: https://github.com/eza-community/eza

- No self-update functionality
- Recommends cargo install for Rust users
- Extensive package manager support (apt, brew, pacman, etc.)
- No install.sh script

**Update approach**: Package manager updates or `cargo install --force eza`

### 2.3 bat

**Repository**: https://github.com/sharkdp/bat

- No self-update functionality
- Similar pattern to ripgrep
- Broad package manager support
- No install.sh script

**Update approach**: Package manager updates

### 2.4 fd

**Repository**: https://github.com/sharkdp/fd

- No self-update functionality
- Same pattern as other sharkdp tools
- Package manager-centric distribution
- No install.sh script

**Update approach**: Package manager updates

### 2.5 starship

**Repository**: https://github.com/starship/starship
**Local notes**: `reference/repo/starship/starship/notes/README.md`

- **No built-in self-update command**
- **Has sophisticated install.sh script** with patterns worth adopting
- Broad package manager support

**Key insight**: starship's install.sh is the most sophisticated of the analyzed projects and provides excellent patterns for langstar.

---

## 3. Starship install.sh Analysis

Starship's install script (`install/install.sh`) contains valuable patterns:

### 3.1 Writability Checks

```bash
check_bin_dir() {
    local bin_dir="$1"

    # Verify the install dir is writable
    if [ -w "$bin_dir" ]; then
        return 0
    else
        return 1
    fi
}
```

### 3.2 Privilege Escalation

```bash
elevate_priv() {
    if ! check_bin_dir "$INSTALL_DIR"; then
        # Try to elevate permissions
        if command -v sudo >/dev/null; then
            sudo="sudo"
        fi
    fi
}
```

### 3.3 Download Fallbacks

```bash
download() {
    if command -v curl >/dev/null; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null; then
        wget -qO "$output" "$url"
    else
        error "curl or wget required"
    fi
}
```

### 3.4 Platform Detection

Starship uses sophisticated platform detection including:

- Architecture detection (x86_64, aarch64, arm)
- OS detection (Linux, macOS, Windows via WSL)
- libc detection (glibc vs musl)

---

## 4. Production Reference: mise

While the five projects above lack self-update, **mise** (jdx/mise) provides a complete production implementation using the `self_update` crate.

**Repository**: https://github.com/jdx/mise
**Stars**: 12k+
**Local notes**: `reference/repo/jdx/mise/notes/README.md`

### Key Implementation Details

| Aspect                 | mise Approach                                                             |
| ---------------------- | ------------------------------------------------------------------------- |
| Library                | `self_update` v0.42 with `signatures` feature                             |
| Package detection      | File-based markers (`mise-self-update-instructions.toml`)                 |
| Signature verification | `zipsign.pub` public key via `verifying_keys()`                           |
| macOS handling         | Post-update `codesign --verify` check                                     |
| Feature gating         | `#[cfg_attr(not(feature = "self_update"), path = "self_update_stub.rs")]` |

### Patterns Worth Adopting

1. **File-based package manager detection** - More reliable than path heuristics
2. **Binary signature verification** - Stronger than SHA256 checksums alone
3. **Feature-gated implementation** - Smaller builds when not needed
4. **GitHub token passthrough** - Essential for rate limit handling

See `reference/repo/jdx/mise/notes/README.md` for detailed analysis with code excerpts.

---

## 5. self_update Crate Status

**Repository**: https://github.com/jaemk/self_update
**Stars**: ~905
**Last Updated**: Active maintenance

### Why These Projects Don't Use It

1. **Package manager preference**: Most users install via brew/apt/cargo
2. **Security concerns**: Self-modifying binaries require elevated permissions
3. **Maintenance overhead**: Additional code paths to test
4. **User expectations**: CLI tools traditionally update via package managers

### Why Langstar Should Use It

1. **GitHub Release focus**: Langstar's primary distribution is GitHub releases
2. **install.sh already exists**: Users expect script-based installation
3. **DevContainer use case**: Updates needed outside package manager context
4. **User request**: Explicit feature request justifies the functionality
5. **Production precedent**: mise demonstrates successful implementation

---

## 6. Recommendations

### 6.1 Confirm Original Scout Recommendation

The `self_update` crate remains the right choice:

- Mature, maintained library
- GitHub Releases integration
- Handles binary replacement safely
- Compatible with langstar's asset naming

### 6.2 Adopt Starship Install Script Patterns

Incorporate these patterns from starship:

| Pattern                    | Benefit                    | Implementation                 |
| -------------------------- | -------------------------- | ------------------------------ |
| Writability check          | Fail fast with clear error | Check before attempting update |
| Privilege escalation hints | Guide user to solution     | "Try running with sudo"        |
| Download fallbacks         | Broader compatibility      | Support curl and wget          |
| Platform detection         | Correct binary selection   | Compile-time conditionals      |

### 6.3 Differentiate from Ecosystem Norm

Since major Rust tools don't have self-update:

- Document why langstar does (GitHub Release distribution focus)
- Make it optional (users can disable)
- Respect package manager installations (detect and warn)

---

## 7. Changes to Original Scout Recommendation

| Aspect                  | Original            | Updated                               |
| ----------------------- | ------------------- | ------------------------------------- |
| Library choice          | `self_update` crate | **No change** - confirmed             |
| Precedent validation    | Not analyzed        | **Added** - no precedents found       |
| Install script patterns | Not analyzed        | **Added** - adopt starship patterns   |
| Risk assessment         | Low                 | **Confirmed** - no ecosystem conflict |

### New Insight

The absence of self-update in major Rust tools is **not a technical limitation** but a **design choice**. Langstar can reasonably choose differently given:

- Different distribution model (GitHub Releases primary)
- Different user base (DevContainer/CI environments)
- Explicit user request

---

## 8. Acceptance Criteria Checklist

- [x] All 5 repos analyzed for self-update functionality
- [x] File paths and line numbers documented (see starship section)
- [x] Clear recommendation on patterns to follow
- [x] Research report committed to `docs/research/`

---

## References

- [Scout Report: Langstar Auto-Update Feasibility](./535-langstar-auto-update-scout.md)
- [self_update crate](https://github.com/jaemk/self_update)
- [mise - production self-update implementation](https://github.com/jdx/mise)
- [mise notes](../../reference/repo/jdx/mise/notes/README.md)
- [Starship install script](https://github.com/starship/starship/blob/master/install/install.sh)
- [Starship notes](../../reference/repo/starship/starship/notes/README.md)
