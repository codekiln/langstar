# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ✨ Features

- ✨ feat(sdk): add deployment create/delete methods to SDK (#160)
  - Added `CreateDeploymentRequest` struct with builder pattern
  - Added `DeploymentClient::create()` method for creating deployments
  - Added `DeploymentClient::delete()` method for deleting deployments
  - Added `control_plane_post()` and `control_plane_delete()` methods to HTTP client
  - Export `CreateDeploymentRequest` in SDK public API

- ✨ feat(cli): add deployment create/delete commands (#160)
  - Added `langstar graph create` command with GitHub source support
  - Added `langstar graph delete` command with confirmation prompt
  - Support for environment variables via `--env KEY=VALUE` flag
  - Support for deployment types: `dev_free`, `dev`, `prod`
  - JSON and table output formats
  - Input validation for required fields and source types

### 🧪 Testing

- 🧪 test(cli): add integration tests for deployment lifecycle (#160)
  - Tests for `graph create` with various configurations
  - Tests for `graph delete` with confirmation behavior
  - Tests for validation and error handling
  - Tests for environment variable parsing

### 📚 Documentation

- 📚 docs: update README with graph deployment commands (#160)
  - Added usage examples for create and delete commands
  - Documented deployment types and source types
  - Added examples with environment variables

## [0.3.0] - 2025-11-12

### ✨ Features

- ✨ feat: add automated CI/CD release pipeline with cross-platform builds (#146)

* ✨ feat: add automated CI/CD release pipeline with cross-platform builds

Implements industry best-practice release workflow following the research from issue #9.

## Changes

### GitHub Actions Workflows
- Add release.yml workflow triggered by version tags (v*)
- Builds cross-platform binaries: Linux (musl/gnu), macOS (Intel/ARM), Windows
- Generates changelogs using git-cliff
- Creates GitHub Releases with artifacts and SHA256 checksums
- Automatic pre-release detection (alpha/beta/rc versions)

### Configuration Files
- cliff.toml: git-cliff configuration for Conventional Emoji Commits
  - Parses emoji and conventional commit formats
  - Groups changes by type (Breaking Changes, Features, Bug Fixes, etc.)
  - Links to GitHub PRs automatically

- release.toml: cargo-release configuration for version management
  - Integrates with git-cliff for changelog generation
  - Automates version bumping and tagging
  - Disables crates.io publishing (GitHub releases only)

### Documentation
- docs/dev/ci-cd.md: Comprehensive CI/CD pipeline documentation
  - Release process guide (automated and manual)
  - Semantic versioning rules based on commit types
  - Troubleshooting guide
  - Best practices and security considerations

### Claude Code Skill
- .claude/skills/bump-release/: Local release management skill
  - Custom scripts for commit analysis and version bumping
  - Alternative to cargo-release for manual control
  - Comprehensive workflow documentation

## Release Process

Using cargo-release (recommended):
```bash
cargo install cargo-release git-cliff
cargo release patch --execute  # Bug fixes
cargo release minor --execute  # Features
cargo release major --execute  # Breaking changes
```

Manual process:
```bash
git tag -a v1.2.3 -m "Release v1.2.3"
git push origin v1.2.3
# GitHub Actions handles the rest
```

## Implementation Details

Follows research recommendations from issue #9:
- ✅ Validate on PR, release on tag pattern
- ✅ Uses Rust ecosystem tools (cargo-release, git-cliff)
- ✅ Strong provenance with checksums and tagged releases
- ✅ Cross-platform binary distribution
- ✅ Automated changelog generation
- ✅ Full Conventional Emoji Commits support

Fixes #9

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>

* 🔧 build: install cargo-release and git-cliff in devcontainer

Adds cargo-release and git-cliff to devcontainer postCreateCommand so they
are automatically available to all developers and maintainers.

Changes:
- .devcontainer/devcontainer.json: Add cargo install commands to postCreateCommand
- docs/dev/ci-cd.md: Update prerequisites to note tools are pre-installed

This ensures consistent tooling across all developers using the devcontainer
and removes the manual installation step from the release process.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>

---------

Co-authored-by: Claude <noreply@anthropic.com>
- ✨ feat: add official installer script for langstar CLI (#149)

Implements end-user installer script with comprehensive features:

- Platform detection (Linux x86_64, macOS Intel/ARM64)
- Automatic version detection (latest from GitHub API)
- SHA256 checksum verification
- Idempotent installation (safe to re-run)
- System-wide (/usr/local/bin) or user-local (~/.local/bin) installation
- Custom prefix support via --prefix flag
- Update detection and upgrade support
- Clear error messages and progress output
- Comprehensive help documentation

Changes:
- Added scripts/install.sh (executable installer script)
- Updated README.md with quick install instructions
- Created docs/installation.md (comprehensive guide)
- Added scripts/test-installer.md (testing checklist)

The installer downloads pre-built binaries from GitHub releases,
eliminating the need for Rust toolchain installation for end-users.

Fixes #148

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-authored-by: Claude <noreply@anthropic.com>

### 🔧 Build System

- 🔧 build(devcontainer): use Claude native installer instead of npm (#147)

Replace npm installation with official native installer method as
recommended in Claude Code documentation. This provides:
- Self-contained executable without Node.js dependency
- Improved auto-updater stability
- Follows official best practices

Uses wget (already available in base image) instead of curl to avoid
adding unnecessary dependencies and reduce security surface area.

Fixes #125

# Changelog

All notable changes to Langstar will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0]

### Added

#### LangGraph Assistants Support

- **Complete LangGraph Assistants API support** - Full CRUD operations for managing LangGraph assistants
  - `langstar assistant list` - List all assistants with pagination support
  - `langstar assistant search <query>` - Search assistants by name
  - `langstar assistant get <id>` - Get detailed assistant information
  - `langstar assistant create` - Create new assistants with optional configuration
  - `langstar assistant update <id>` - Update assistant name and configuration
  - `langstar assistant delete <id>` - Delete assistants with optional force flag

- **Deployment-level resource model** - Assistants are scoped to API key/deployment
  - No organization or workspace scoping required
  - Simpler configuration compared to LangSmith prompts
  - Clear separation from LangSmith's hierarchical model

- **Configuration file support** - Assistants can be configured via:
  - Inline JSON: `--config '{"temperature": 0.7}'`
  - Configuration files: `--config-file path/to/config.json`

#### Documentation

- **Comprehensive configuration guide** (`docs/configuration.md`)
  - Environment variables reference
  - Configuration file format documentation
  - Precedence rules explanation
  - Common scenarios and examples
  - Migration guides

- **Workflow examples** for both services:
  - `docs/examples/prompt-workflows.md` - LangSmith prompt patterns
  - `docs/examples/assistant-workflows.md` - LangGraph assistant patterns
  - `docs/examples/multi-service-usage.md` - Using both services together

- **Architecture documentation** (`docs/architecture.md`)
  - Resource scoping models explained
  - Multi-service SDK design
  - HTTP client implementation details
  - Error handling patterns
  - Design principles and trade-offs

- **Troubleshooting guide** (`docs/troubleshooting.md`)
  - Common configuration issues
  - Authentication errors
  - Scoping problems
  - Network and connectivity issues
  - Debug workflows

#### SDK Enhancements

- **Multi-service HTTP client** - Separate header management for each service
  - LangSmith: Adds `x-organization-id` and `X-Tenant-Id` headers when configured
  - LangGraph: API key only, no additional scoping headers

- **Improved error handling** - Service-specific error messages with helpful hints

#### CLI Improvements

- **Enhanced help text** - Clear documentation of service differences in CLI
- **Service-specific commands** - Separate command groups for prompts and assistants
- **Configuration visualization** - `langstar config` shows service-specific settings

### Changed

#### Breaking Changes

None. Version 0.2.0 adds new features without changing existing functionality.

#### Configuration

- **Unified API key** - Uses `LANGSMITH_API_KEY` for both services:
  - `LANGSMITH_API_KEY` for LangSmith prompts
  - `LANGSMITH_API_KEY` for LangGraph assistants (LangGraph Cloud is part of LangSmith)

- **Configuration file structure** - Simplified configuration:
  ```toml
  [langstar]
  # LangSmith configuration (for both prompts and assistants)
  langsmith_api_key = "<key>"
  organization_id = "<org-id>"    # Optional (prompts only)
  workspace_id = "<workspace-id>" # Optional (prompts only)
  ```

#### Documentation

- **README restructured** with prominent "Configuration Quick Start" section
- **Clear service separation** throughout all documentation
- **Enhanced examples** showing real-world usage patterns

### Fixed

- Improved error messages when using wrong API key for a service
- Better handling of missing configuration
- Clearer scoping behavior documentation

## [0.1.0] - Initial Release

### Added

#### LangSmith Prompts Support

- **Core prompt operations**:
  - `langstar prompt list` - List prompts with organization/workspace scoping
  - `langstar prompt get <name>` - Get prompt details
  - `langstar prompt search <query>` - Search prompts by keyword

- **Organization and workspace scoping**:
  - `--organization-id` flag for organization-level operations
  - `--workspace-id` flag for workspace-level operations
  - `--public` flag to access public prompts when scoped

- **Output formats**:
  - Table format (human-readable, default)
  - JSON format (machine-readable, for scripting)

#### Configuration System

- **Environment variables**:
  - `LANGSMITH_API_KEY` - API authentication
  - `LANGSMITH_ORGANIZATION_ID` - Optional organization scoping
  - `LANGSMITH_WORKSPACE_ID` - Optional workspace scoping

- **Configuration file** support (`~/.langstar/config.toml`):
  ```toml
  [langstar]
  langsmith_api_key = "<key>"
  organization_id = "<org-id>"
  workspace_id = "<workspace-id>"
  output_format = "table"
  ```

- **Precedence order**: CLI flags → config file → environment variables

#### SDK Architecture

- **Spec-driven development** - Code generated from OpenAPI specifications
- **Thin wrapper pattern** - Minimal abstraction over upstream APIs
- **Type-safe** - Leverages Rust's type system for correctness
- **HTTP client** - Built on reqwest with proper error handling

#### CLI Features

- **Clap-based** command-line interface
- **Consistent** command structure across all operations
- **Clear error messages** with helpful hints
- **Exit codes** for CI/CD integration

#### Documentation

- README with quick start guide
- Developer documentation in `docs/dev/`:
  - GitHub workflow
  - Git SCM conventions
  - Code style principles

---

## Version Comparison

### v0.2.0 vs v0.1.0

**What's New in v0.2.0:**

1. **LangGraph Assistants** - Full CRUD support for LangGraph assistants
2. **Multi-Service Architecture** - Clear separation between LangSmith and LangGraph
3. **Comprehensive Documentation** - 6 new documentation files covering all aspects
4. **Enhanced Configuration** - Support for service-specific API keys
5. **Better Developer Experience** - Clear error messages, troubleshooting guide, examples

**Upgrade Path:**

No breaking changes. Existing v0.1.0 configurations continue to work. To use new assistant features:

1. Ensure `LANGSMITH_API_KEY` is set (same key works for both prompts and assistants)
2. Use `langstar assistant` commands

**Configuration Migration:**

```bash
# v0.1.0 (still works in v0.2.0)
export LANGSMITH_API_KEY="<key>"
langstar prompt list

# v0.2.0 (same key for both services)
export LANGSMITH_API_KEY="<key>"
langstar prompt list      # Uses LANGSMITH_API_KEY
langstar assistant list   # Uses LANGSMITH_API_KEY (LangGraph is part of LangSmith)
```

---

## Release Notes

### v0.2.0: LangGraph Assistants & Comprehensive Documentation

This release adds complete support for LangGraph assistants and significantly improves documentation and developer experience.

**Key Features:**

- ✅ Full LangGraph Assistants API support (list, get, search, create, update, delete)
- ✅ Multi-service architecture with clear service separation
- ✅ 6 comprehensive documentation files (1000+ lines of docs)
- ✅ Real-world workflow examples for both services
- ✅ Enhanced error messages with service-specific guidance
- ✅ Troubleshooting guide with solutions to common issues

**Documentation Highlights:**

- [Configuration Guide](./docs/configuration.md) - 500+ lines covering all configuration aspects
- [Architecture Documentation](./docs/architecture.md) - Detailed design explanations
- [Workflow Examples](./docs/examples/) - 3 comprehensive example guides
- [Troubleshooting Guide](./docs/troubleshooting.md) - Solutions to common issues

**For Users:**

- Easier to get started with clear configuration quick start
- Better understanding of service differences
- Comprehensive examples for common tasks
- Quick troubleshooting when issues arise

**For Developers:**

- Clear architecture documentation
- Well-documented SDK with inline comments
- Comprehensive test coverage
- Design principles and trade-offs explained

### v0.1.0: Initial Release

First release of Langstar with support for LangSmith prompts.

**Features:**

- List, get, and search prompts
- Organization and workspace scoping
- Configuration via environment variables and config file
- JSON and table output formats
- Type-safe Rust SDK
- Comprehensive CLI with clap

---

## Links

- [GitHub Repository](https://github.com/codekiln/langstar)
- [Issues](https://github.com/codekiln/langstar/issues)
- [Documentation](./docs/)
- [LangSmith Documentation](https://docs.smith.langchain.com/)
- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)

---

## Versioning

We follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html):

- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backwards compatible manner
- **PATCH** version for backwards compatible bug fixes

## Deprecation Policy

Features marked as deprecated will be supported for at least one minor version before removal. Deprecation warnings will appear in:

1. CHANGELOG (this file)
2. CLI warning messages
3. Documentation

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for how to contribute to Langstar.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](./LICENSE) file for details.
