# Langstar

## Project Overview

Langstar is a **Rust project**. 

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
