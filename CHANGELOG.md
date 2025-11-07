# Changelog

All notable changes to Langstar will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-01-07

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

- **Separate API keys** - Now supports distinct keys for each service:
  - `LANGSMITH_API_KEY` for LangSmith prompts
  - `LANGGRAPH_API_KEY` for LangGraph assistants (with fallback to `LANGSMITH_API_KEY`)

- **Configuration file structure** - Added LangGraph section:
  ```toml
  [langstar]
  # LangSmith configuration
  langsmith_api_key = "<key>"
  organization_id = "<org-id>"    # Optional
  workspace_id = "<workspace-id>" # Optional

  # LangGraph configuration (new)
  langgraph_api_key = "<key>"
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

1. Set `LANGGRAPH_API_KEY` (or use `LANGSMITH_API_KEY` as fallback)
2. Use `langstar assistant` commands

**Configuration Migration:**

```bash
# v0.1.0 (still works)
export LANGSMITH_API_KEY="<key>"
langstar prompt list

# v0.2.0 (recommended for assistants)
export LANGSMITH_API_KEY="<langsmith-key>"
export LANGGRAPH_API_KEY="<langgraph-key>"
langstar prompt list      # Uses LANGSMITH_API_KEY
langstar assistant list   # Uses LANGGRAPH_API_KEY
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
