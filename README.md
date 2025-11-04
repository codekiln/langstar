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

#### Organization and Workspace Scoping

Langstar supports scoping operations to a specific organization or workspace. This is useful when working with team prompts or enterprise deployments.

**Configuration Methods:**

1. **Environment Variables:**
   ```bash
   export LANGSMITH_ORGANIZATION_ID="your-org-id"
   export LANGSMITH_WORKSPACE_ID="your-workspace-id"
   ```

2. **Config File (`~/.config/langstar/config.toml`):**
   ```toml
   organization_id = "your-org-id"
   workspace_id = "your-workspace-id"
   ```

3. **CLI Flags (per command):**
   ```bash
   langstar prompt list --organization-id "your-org-id"
   langstar prompt list --workspace-id "your-workspace-id"
   ```

**Precedence Order:** CLI flags → config file → environment variables

**Default Behavior:**
- When scoped (org/workspace ID set), operations **default to private prompts only**
- Use `--public` flag to explicitly access public prompts when scoped
- Without scoping, all prompts (public and private) are accessible

For detailed documentation on scoping, see [docs/usage/scoping.md](./docs/usage/scoping.md).

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

# Scoped operations (organization/workspace)
langstar prompt list --organization-id "your-org-id"  # List private prompts in org
langstar prompt list --organization-id "your-org-id" --public  # List public prompts in org
langstar prompt search "rag" --workspace-id "your-workspace-id"  # Search within workspace
```

## LangGraph Deployments and Assistants

Langstar provides commands for managing LangGraph deployments and assistants. The workflow uses **auto-discovery** via the Control Plane API - you don't need to manually register deployments.

### Prerequisites

For deployment and assistant operations, you need:

```bash
export LANGSMITH_API_KEY="your-api-key"
export LANGCHAIN_WORKSPACE_ID="your-workspace-id"
```

Or in `~/.config/langstar/config.toml`:

```toml
langsmith_api_key = "your-api-key"
workspace_id = "your-workspace-id"
```

### Discovering Deployments

First, list your available LangGraph deployments:

```bash
# List all deployments
langstar graph list

# Output example:
# Name                    ID                   Status  Created
# my-prod-deployment      abc-123e4567...      READY   2024-01-15
# my-staging-deployment   def-456e4567...      READY   2024-01-20
```

You can filter and search:

```bash
# Filter by deployment type
langstar graph list --deployment-type prod

# Filter by status
langstar graph list --status READY

# Filter by name
langstar graph list --name-contains "production"

# JSON output for scripting
langstar graph list --format json
```

### Managing Assistants

All assistant commands require the `--deployment` flag to specify which deployment to target. You can use either the deployment **name** or **ID** from `langstar graph list`.

#### List Assistants

```bash
# List assistants in a deployment (by name)
langstar assistant list --deployment my-prod-deployment

# Or by ID
langstar assistant list --deployment abc-123e4567

# With pagination
langstar assistant list --deployment my-prod-deployment --limit 10 --offset 20

# JSON output
langstar assistant list --deployment my-prod-deployment --format json
```

#### Search Assistants

```bash
# Search by name
langstar assistant search "customer" --deployment my-prod-deployment

# With result limit
langstar assistant search "bot" --deployment my-staging --limit 5
```

#### Get Assistant Details

```bash
# Get details of a specific assistant
langstar assistant get <assistant-id> --deployment my-prod-deployment
```

#### Create Assistant

```bash
# Create a new assistant
langstar assistant create \
  --deployment my-staging-deployment \
  --graph-id my-graph-id \
  --name "My Assistant"

# With inline configuration
langstar assistant create \
  --deployment my-staging \
  --graph-id my-graph \
  --name "Configured Bot" \
  --config '{"temperature": 0.7}'

# With configuration from file
langstar assistant create \
  --deployment my-staging \
  --graph-id my-graph \
  --name "File Config Bot" \
  --config-file ./assistant-config.json
```

#### Update Assistant

```bash
# Update assistant name
langstar assistant update <assistant-id> \
  --deployment my-staging \
  --name "Updated Name"

# Update configuration
langstar assistant update <assistant-id> \
  --deployment my-staging \
  --config '{"temperature": 0.9}'

# Update from file
langstar assistant update <assistant-id> \
  --deployment my-staging \
  --config-file ./new-config.json
```

#### Delete Assistant

```bash
# Delete with confirmation prompt
langstar assistant delete <assistant-id> --deployment my-staging

# Force delete (skip confirmation)
langstar assistant delete <assistant-id> --deployment my-staging --force
```

### How Deployment Resolution Works

When you specify `--deployment <name-or-id>`:

1. Langstar queries the Control Plane API to list your deployments
2. Finds the deployment by matching name or ID
3. Extracts the deployment's `custom_url` from the API response
4. Uses that URL for all assistant API calls

This means:
- ✅ No manual configuration needed
- ✅ Always up-to-date with your actual deployments
- ✅ Clear error messages if deployment not found

### Error Handling

If a deployment isn't found:

```bash
$ langstar assistant list --deployment nonexistent
Error: Deployment 'nonexistent' not found. Run 'langstar graph list' to see available deployments.
```

If a deployment has no URL (rare):

```bash
$ langstar assistant list --deployment my-deployment
Error: Deployment 'my-deployment' has no custom_url in source_config
```

### Example Workflow

```bash
# 1. Discover available deployments
$ langstar graph list
Name                ID                  Status  Created
prod-chatbot        abc-123...          READY   2024-01-15
staging-chatbot     def-456...          READY   2024-01-20

# 2. List assistants in production
$ langstar assistant list --deployment prod-chatbot
ID              Name            Graph ID        Created
a1b2c3...       Support Bot     graph-123...    2024-01-15
d4e5f6...       Sales Bot       graph-456...    2024-01-16

# 3. Create new assistant in staging
$ langstar assistant create \
  --deployment staging-chatbot \
  --graph-id graph-789 \
  --name "Test Bot"

# 4. Test the assistant in staging
$ langstar assistant get a7b8c9 --deployment staging-chatbot

# 5. When ready, create in production
$ langstar assistant create \
  --deployment prod-chatbot \
  --graph-id graph-789 \
  --name "Production Bot"
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
