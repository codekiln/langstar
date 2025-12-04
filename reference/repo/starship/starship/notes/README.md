# starship

## Repository Information

- **Repository**: [starship/starship](https://github.com/starship/starship)
- **Date Created**: 2025-11-22
- **Cloned to**: `/workspace/reference/repo/starship/starship/code`

## Purpose

Studying starship's distribution patterns, specifically:
1. How a major Rust CLI tool handles installation without built-in self-update
2. Install script patterns that could benefit langstar
3. Platform detection and privilege handling approaches

## Key Findings

### No Built-in Self-Update

Like other major Rust CLI tools (ripgrep, bat, fd, eza), starship does **not** have a built-in self-update command. Instead, it relies on:
- Package managers (brew, apt, cargo, etc.)
- The `install.sh` script for initial install and updates

### Install Script Excellence

Starship's `install/install.sh` is the most sophisticated install script among analyzed Rust CLI tools. Key patterns:

#### 1. Writability Checks
The script checks if the target directory is writable before attempting installation, avoiding cryptic permission errors.

#### 2. Privilege Escalation
Detects when sudo is needed and handles elevation gracefully, rather than failing unexpectedly.

#### 3. Download Fallbacks
Supports both `curl` and `wget`, maximizing compatibility across systems.

#### 4. Platform Detection
Comprehensive detection including:
- Architecture (x86_64, aarch64, arm)
- Operating system (Linux, macOS, Windows/WSL)
- libc variant (glibc vs musl)

## Architecture

```
starship/
├── install/
│   └── install.sh      # Sophisticated install script
├── src/
│   ├── main.rs         # Entry point
│   ├── modules/        # Prompt modules
│   └── ...
├── docs/               # Documentation
└── Cargo.toml
```

## Notes

### Patterns to Adopt for Langstar

1. **Pre-flight writability check** - Check directory permissions before attempting binary replacement
2. **Clear elevation guidance** - Tell users when/how to use sudo
3. **Download resilience** - Support multiple download tools (curl/wget)
4. **Graceful degradation** - Provide clear errors instead of cryptic failures

### Why Starship Doesn't Have Self-Update

The starship team chose to:
- Focus on core functionality (prompt customization)
- Leverage existing package manager ecosystem
- Avoid security complexity of self-modifying binaries
- Keep the binary lightweight

This is a reasonable design choice for their use case but doesn't preclude langstar from adding self-update, given langstar's different distribution model (GitHub Releases primary, DevContainer focus).

## Related Research

- [Issue #542: Rust project self-update precedents](https://github.com/codekiln/langstar/issues/542)
- [Scout Report: Langstar Auto-Update Feasibility](../../../../../docs/research/535-langstar-auto-update-scout.md)
- [Research Addendum: Rust Self-Update Precedents](../../../../../docs/research/542-rust-self-update-precedents.md)
