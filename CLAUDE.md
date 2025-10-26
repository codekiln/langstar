# Langstar

## Project Overview

Langstar is a **Rust project** for language learning and translation services. The project leverages Rust's performance, safety, and modern tooling ecosystem to build reliable and efficient language processing capabilities.

### Technology Stack

- **Primary Language**: Rust
- **Build System**: Cargo (Rust's package manager and build tool)
- **Development Environment**: Devcontainer-based setup for consistency

## Dev Setup
* see .devcontainer
* git access is provided via a github fine-grained personal access token
  * it's locked down to this repo

## Tooling Preferences

This project prefers **Rust-based tools** wherever possible for better performance and integration with the Rust ecosystem.

### Recommended Rust Tools

- **[ripgrep](https://github.com/BurntSushi/ripgrep)** (`rg`) - Fast recursive search tool (replacement for `grep`)
- **[fd](https://github.com/sharkdp/fd)** - Fast and user-friendly alternative to `find`
- **[bat](https://github.com/sharkdp/bat)** - Cat clone with syntax highlighting
- **[exa](https://github.com/ogham/exa)** or **[eza](https://github.com/eza-community/eza)** - Modern replacement for `ls`
- **[tokei](https://github.com/XAMPPRocky/tokei)** - Fast code statistics tool
- **[hyperfine](https://github.com/sharkdp/hyperfine)** - Command-line benchmarking tool
- **[git-delta](https://github.com/dandavison/delta)** - Syntax-highlighting pager for git (already installed in devcontainer)

### Cargo Workflows

Common Cargo commands for development:

```bash
# Build the project
cargo build

# Build with optimizations (release mode)
cargo build --release

# Run the project
cargo run

# Run tests
cargo test

# Check code without building (faster)
cargo check

# Format code according to Rust style guidelines
cargo fmt

# Run the linter (clippy)
cargo clippy

# Generate and open documentation
cargo doc --open

# Update dependencies
cargo update

# Clean build artifacts
cargo clean
```

## Claude Code Plugin Configuration

This project uses project-based plugin configuration (preferred for devcontainers).

### Configuration Files

- **`.claude/settings.json`** - Version-controlled configuration
  - Defines plugin marketplaces and enabled plugins
  - Automatically installs plugins when team members trust the repository
  - Commit this file to share configuration with the team

- **`.claude/settings.local.json`** - Local overrides (gitignored)
  - Contains credentials, API keys, and personal settings
  - Not committed to version control
  - Used for environment-specific configuration

### Installed Plugins

The project uses plugins from the [anthropics/skills](https://github.com/anthropics/skills) marketplace:

- **example-skills** - Skill creation, MCP building, design, art, communications, web testing, etc.
- **document-skills** - Excel, Word, PowerPoint, and PDF processing capabilities

### Settings Merge Behavior

When both `settings.json` and `settings.local.json` exist, the files are merged with local settings taking precedence. This allows you to:
- Keep shared team configuration in `settings.json`
- Override specific settings locally in `settings.local.json`
- Maintain separate credentials without committing them

## Development Workflow

This project follows a GitHub issue-driven development workflow. For complete details, see @docs/dev/github-workflow.md

Key points:
- Create GitHub issues for all work
- Use branch naming convention: `<username>/<issue_num>-<issue_slug>`
- Follow Conventional Emoji Commits for commit messages
- Link PRs to issues using `Fixes #N` or `Closes #N`

## Coding Conventions

All coding conventions and development guidelines can be found in @docs/dev/README.md

For commit message formatting, please follow @docs/dev/git-scm-conventions.md