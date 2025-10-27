# Langstar

[![CI](https://github.com/codekiln/langstar/workflows/CI/badge.svg)](https://github.com/codekiln/langstar/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Langstar** is a unified Rust CLI for the LangChain ecosystem, providing ergonomic access to LangSmith, LangGraph Cloud, and other LangChain services.

## Features

- **Spec-Driven SDK** - Generated directly from OpenAPI specifications for guaranteed API coverage
- **Ergonomic CLI** - Built with [clap](https://docs.rs/clap/) for excellent UX
- **Multiple Output Formats** - JSON for scripting, tables for human readability
- **Configuration Management** - Support for config files and environment variables
- **Automation-Friendly** - Designed for both interactive use and AI agent invocation
- **Type-Safe** - Leverages Rust's type system for safety and reliability

## Quick Start

### Prerequisites

- Rust 1.78+ (or latest stable)
- LangSmith API key (get one at [smith.langchain.com](https://smith.langchain.com))

### Installation

#### From Source

```bash
# Clone the repository
git clone https://github.com/codekiln/langstar.git
cd langstar

# Build and install
cargo install --path cli
```

### Configuration

Set your API key via environment variable:

```bash
export LANGSMITH_API_KEY="your-api-key-here"
```

Or create a config file at `~/.config/langstar/config.toml`:

```toml
langsmith_api_key = "your-api-key-here"
output_format = "table"
```

### Usage

```bash
# Show help
langstar --help

# List prompts from LangSmith
langstar prompt list

# Get details of a specific prompt
langstar prompt get owner/prompt-name

# Search for prompts
langstar prompt search "query"

# Output as JSON for scripting
langstar prompt list --format json

# Show configuration
langstar config
```

## Architecture

Langstar follows a **spec-driven, thin-wrapper architecture**:

```
langstar-rs/
├── sdk/                    # Rust SDK (generated + ergonomic wrappers)
│   ├── src/
│   │   ├── auth.rs        # Authentication helpers
│   │   ├── client.rs      # HTTP client configuration
│   │   ├── error.rs       # Error types
│   │   ├── prompts.rs     # LangSmith Prompts API
│   │   ├── generated/     # OpenAPI-generated code
│   │   └── lib.rs
│   └── Cargo.toml
├── cli/                    # User-facing CLI binary
│   ├── src/
│   │   ├── commands/      # Subcommand implementations
│   │   ├── config.rs      # Configuration management
│   │   ├── output.rs      # Output formatting
│   │   └── main.rs
│   └── Cargo.toml
└── tools/
    └── generate_sdk.sh    # OpenAPI code generation
```

### Design Principles

1. **Spec-Driven Development** - Generate code from OpenAPI specs, don't hand-write API wrappers
2. **Thin Wrapper Pattern** - Add only lightweight ergonomic helpers, no business logic duplication
3. **Automation-First** - Design for both human and AI agent usage
4. **Zero Surprises** - Type-safe, predictable behavior with clear error messages

## Development

### Building

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

### Project Structure

- **`sdk/`** - Rust SDK with authentication, client wrappers, and API bindings
- **`cli/`** - Command-line interface built with clap
- **`tools/`** - Development tools (OpenAPI code generation, etc.)
- **`.github/workflows/`** - CI/CD pipelines

### OpenAPI Code Generation

Generate Rust clients from OpenAPI specifications:

```bash
./tools/generate_sdk.sh
```

See [tools/README.md](tools/README.md) for details on the generation process.

### Running the CLI Locally

```bash
# Run without installing
cargo run --bin langstar -- prompt list

# With environment variable
LANGSMITH_API_KEY="your-key" cargo run --bin langstar -- prompt list --format json
```

## Contributing

Contributions are welcome! This project follows a GitHub issue-driven development workflow.

### Development Workflow

1. Create a GitHub issue describing the feature or bug
2. Create a branch following the convention: `<username>/<issue_num>-<issue_slug>`
3. Make your changes following the coding conventions
4. Use Conventional Emoji Commits for commit messages
5. Submit a pull request referencing the issue

For complete details, see:
- [GitHub Workflow](./docs/dev/github-workflow.md)
- [Git SCM Conventions](./docs/dev/git-scm-conventions.md)
- [Developer Documentation](./docs/dev/)

### Commit Message Format

This project uses [Conventional Emoji Commits](https://conventional-emoji-commits.site):

```
✨ feat: add new feature
🩹 fix: resolve bug
📚 docs: update documentation
♻️ refactor: refactor code
🧪 test: add tests
🔧 build: update build configuration
```

## Roadmap

### Current (Prototype)

- ✅ Cargo workspace with `sdk` and `cli` crates
- ✅ Authentication and configuration management
- ✅ Basic LangSmith Prompts API (`list`, `get`, `search`)
- ✅ JSON and table output formats
- ✅ CI/CD pipeline
- ✅ OpenAPI generation tooling

### Next Steps

1. **Full LangSmith API Coverage**
   - Projects, Datasets, Traces, Runs, Feedback
   - Evaluations and Testing

2. **LangGraph Cloud Integration**
   - Deployments, Assistants, Threads
   - Streaming and real-time updates

3. **OpenAPI Integration**
   - Automated SDK generation from specs
   - CI/CD automation for spec updates

4. **Enhanced CLI Features**
   - Shell completion
   - Interactive prompts
   - Progress bars for long operations

5. **Advanced Features**
   - Retry logic and rate limiting
   - Caching and offline mode
   - Plugin system

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Resources

- [LangSmith Documentation](https://docs.smith.langchain.com/)
- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)
- [LangChain Documentation](https://python.langchain.com/)

## Developer Documentation

For coding conventions, best practices, and development guidelines, see the [Developer Documentation](./docs/dev/).

Key documentation:
- [GitHub Workflow](./docs/dev/github-workflow.md) - Issue-driven development process
- [Git SCM Conventions](./docs/dev/git-scm-conventions.md) - Commit message standards
- [Spec-Kit Integration](./docs/dev/spec-kit.md) - Spec-driven development with GitHub Spec-Kit

## Setup

This project uses a devcontainer for consistent development environment. See [.devcontainer](./.devcontainer) for configuration details.

For more information about the project setup and configuration, see [CLAUDE.md](./CLAUDE.md).

---

Built with ❤️ using Rust
