# Langstar

[![CI](https://github.com/codekiln/langstar/workflows/CI/badge.svg)](https://github.com/codekiln/langstar/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Langstar** is a unified Rust CLI for the LangChain ecosystem, providing ergonomic access to LangSmith, LangGraph Cloud, and other LangChain services.

## Features

- **Type-Safe SDK** - Rust types for LangSmith and LangGraph APIs (OpenAPI generation planned - see [#114](https://github.com/codekiln/langstar/issues/114))
- **Ergonomic CLI** - Built with [clap](https://docs.rs/clap/) for excellent UX
- **Multiple Output Formats** - JSON for scripting, tables for human readability
- **Configuration Management** - Support for config files and environment variables
- **Automation-Friendly** - Designed for both interactive use and AI agent invocation
- **Type-Safe** - Leverages Rust's type system for safety and reliability

## Quick Start

### Installation

#### Quick Install (Recommended for Users)

Install the latest release with our installer script:

```bash
# Quick install (recommended)
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | sh
```

Or download and run manually:

```bash
curl -LO https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh
chmod +x install.sh
./install.sh
```

**Install options:**
```bash
# Install specific version
./install.sh --version 0.2.0

# Install to custom location
./install.sh --prefix ~/.local/bin

# See all options
./install.sh --help
```

The installer script:
- ✅ Downloads pre-built binaries (no compilation needed)
- ✅ Verifies SHA256 checksums
- ✅ Supports Linux (x86_64) and macOS (Intel/Apple Silicon)
- ✅ Installs to `/usr/local/bin` or `~/.local/bin`
- ✅ Handles updates automatically

For detailed installation instructions, see [docs/installation.md](./docs/installation.md).

#### Build from Source (For Development)

If you want to contribute or build from source:

```bash
# Clone the repository
git clone https://github.com/codekiln/langstar.git
cd langstar

# Build and install
cargo install --path cli
```

### Configuration Quick Start

> **⚠️ Important**: Langstar commands have different configuration requirements depending on which service you're using.

Langstar provides access to **LangSmith services** including both prompts and LangGraph Cloud deployments (assistants):

#### For LangSmith Prompts (`langstar prompt *`)

**Required:**
- `LANGSMITH_API_KEY` - Your LangSmith API key ([get one here](https://smith.langchain.com))

**Optional (for organization/workspace scoping):**
- `LANGSMITH_ORGANIZATION_ID` - Scope operations to a specific organization
- `LANGSMITH_WORKSPACE_ID` - Scope operations to a specific workspace

**Example:**
```bash
# Minimal setup (personal prompts)
export LANGSMITH_API_KEY="<your-api-key>"
langstar prompt list

# With workspace scoping (team prompts)
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
langstar prompt list
```

#### For LangGraph Assistants (`langstar assistant *`)

**Required:**
- `LANGSMITH_API_KEY` - Same API key as prompts (LangGraph Cloud is part of LangSmith)

**Not Used:**
- ❌ Organization/workspace IDs are **not applicable** for assistants
- ❌ Assistants are **deployment-level resources**, not organization-scoped

**Example:**
```bash
# Simple setup - no scoping needed
export LANGSMITH_API_KEY="<your-api-key>"
langstar assistant list
```

> **Why the difference?** LangSmith uses a hierarchical organization/workspace model for prompts, while LangGraph assistants are deployment-level resources. Access to assistants is controlled entirely by your API key and deployment permissions.

For complete configuration details, see the [Configuration Guide](#configuration).

### Usage Examples

#### General Commands

```bash
# Show help
langstar --help

# Show current configuration
langstar config
```

#### LangSmith Prompts (Organization/Workspace Scoped)

```bash
# List all accessible prompts
langstar prompt list

# Get details of a specific prompt
langstar prompt get owner/prompt-name

# Search for prompts
langstar prompt search "query"

# Organization-scoped operations
langstar prompt list --organization-id "<your-org-id>"  # Private prompts in org
langstar prompt list --organization-id "<your-org-id>" --public  # Public prompts in org

# Workspace-scoped operations (narrower scope)
langstar prompt search "rag" --workspace-id "<your-workspace-id>"

# Output as JSON for scripting
langstar prompt list --format json
```

#### LangGraph Assistants (Deployment-Level)

```bash
# List all assistants (scoped to your API key/deployment)
langstar assistant list

# List with pagination
langstar assistant list --limit 10 --offset 20

# Search for assistants by name
langstar assistant search "customer-service"

# Get details of a specific assistant
langstar assistant get <assistant-id>

# Create a new assistant
langstar assistant create --graph-id <graph-id> --name "My Assistant"

# Create with configuration
langstar assistant create --graph-id <graph-id> --name "Configured Bot" \
  --config '{"temperature": 0.7}'

# Update an assistant
langstar assistant update <assistant-id> --name "Updated Name"

# Delete an assistant
langstar assistant delete <assistant-id>
langstar assistant delete <assistant-id> --force  # Skip confirmation

# JSON output
langstar assistant list --format json
```

#### LangGraph Deployments (Control Plane)

```bash
# List all deployments
langstar graph list

# List with filters
langstar graph list --limit 20 --status READY --deployment-type prod

# Create a new deployment
langstar graph create \
  --name "my-deployment" \
  --source github \
  --repo-url https://github.com/owner/repo \
  --branch main \
  --deployment-type dev_free

# Create and wait for deployment to be READY
langstar graph create \
  --name "my-deployment" \
  --source github \
  --repo-url https://github.com/owner/repo \
  --branch main \
  --deployment-type dev_free \
  --wait

# Create with environment variables
langstar graph create \
  --name "production-deployment" \
  --source github \
  --repo-url https://github.com/owner/repo \
  --branch main \
  --deployment-type prod \
  --env "API_KEY=value1" \
  --env "DEBUG=true"

# Delete a deployment (with confirmation)
langstar graph delete <deployment-id>

# Delete without confirmation
langstar graph delete <deployment-id> --yes

# JSON output
langstar graph list --format json
```

**Deployment Types:**
- `dev_free` - Free development deployment
- `dev` - Paid development deployment
- `prod` - Production deployment with HA and autoscaling

**Source Types:**
- `github` - Deploy from a GitHub repository (requires `--repo-url` and `--branch`)
- `external_docker` - Deploy from an external Docker image

## Configuration

This section provides detailed configuration options for both LangSmith and LangGraph services.

### Configuration Methods

Langstar supports three configuration methods, in order of precedence:

1. **Command-line flags** (highest priority)
2. **Configuration file** (`~/.langstar/config.toml`)
3. **Environment variables** (lowest priority)

### Configuration File Format

Create a configuration file at `~/.langstar/config.toml`:

```toml
[langstar]
# Output format (table or json)
output_format = "table"

# LangSmith configuration (for both prompt and assistant commands)
langsmith_api_key = "<your-api-key>"
organization_id = "<your-org-id>"        # Optional: scope to organization (prompts only)
workspace_id = "<your-workspace-id>"     # Optional: scope to workspace (prompts only)
```

### Environment Variables

#### LangSmith Service (for `langstar prompt *` commands)

```bash
# Required
export LANGSMITH_API_KEY="<your-api-key>"

# Optional: Organization/Workspace scoping
export LANGSMITH_ORGANIZATION_ID="<your-org-id>"
export LANGSMITH_ORGANIZATION_NAME="<org-name>"      # Informational only
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
export LANGSMITH_WORKSPACE_NAME="<workspace-name>"   # Informational only
```

**Scoping Behavior:**
- When scoped (org/workspace ID set), operations **default to private prompts only**
- Use `--public` flag to explicitly access public prompts when scoped
- Without scoping, all prompts (public and private) are accessible

#### LangGraph Service (for `langstar assistant *` commands)

```bash
# Required (same as prompts)
export LANGSMITH_API_KEY="<your-api-key>"
```

**No Additional Configuration Needed:**
- ❌ No organization ID
- ❌ No workspace ID
- ❌ No deployment configuration
- ✅ Assistants are automatically scoped to your API key and deployment
- ✅ Uses the same `LANGSMITH_API_KEY` as prompt commands

### Viewing Current Configuration

Check your current configuration at any time:

```bash
langstar config
```

This displays:
- Configuration file location
- Which API keys are configured (without showing the actual keys)
- Organization/workspace scoping status
- Output format settings

**Example output:**
```
Configuration file: ~/.langstar/config.toml

LangSmith Configuration:
  API key: configured
  Organization ID: <your-org-id> (scopes prompt operations)
  Workspace ID: <your-workspace-id> (narrows scope further)

  → Prompt commands will use workspace-scoped resources
  → Assistant commands use deployment-level resources (same API key, no org/workspace scoping)
```

### Troubleshooting Configuration

**"Authentication failed" errors:**
1. Verify you have `LANGSMITH_API_KEY` set (used for both prompts and assistants)
2. Ensure your API key is valid and not expired
3. Check that the key has access to the resources you're trying to access

**"No assistants found" but I have assistants:**
- Assistants are deployment-level resources
- Ensure your `LANGSMITH_API_KEY` has access to the deployment
- Unlike prompts, assistants do NOT support org/workspace scoping

For more troubleshooting help, see the [Troubleshooting Guide](./docs/troubleshooting.md).

## Architecture

Langstar follows a **spec-driven, thin-wrapper architecture** and implements a **multi-service SDK** that cleanly separates LangSmith and LangGraph concerns.

```
langstar-rs/
├── sdk/                    # Rust SDK (generated + ergonomic wrappers)
│   ├── src/
│   │   ├── auth.rs        # Authentication helpers
│   │   ├── client.rs      # HTTP client configuration
│   │   ├── error.rs       # Error types
│   │   ├── prompts.rs     # LangSmith Prompts API (org/workspace scoped)
│   │   ├── assistants.rs  # LangGraph Assistants API (deployment-level)
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

1. **Spec-Driven Development** - Design goal to generate code from OpenAPI specs (tracked in [#114](https://github.com/codekiln/langstar/issues/114))
2. **Thin Wrapper Pattern** - Add only lightweight ergonomic helpers, no business logic duplication
3. **Automation-First** - Design for both human and AI agent usage
4. **Zero Surprises** - Type-safe, predictable behavior with clear error messages
5. **Service Separation** - Clean boundaries between LangSmith and LangGraph APIs

### Resource Scoping Models

Langstar interacts with two LangChain services that have fundamentally different resource scoping models:

| Service | Scope Level | Headers Used | Multi-tenancy |
|---------|-------------|--------------|---------------|
| **LangSmith (Prompts)** | Organization/Workspace | `x-api-key`, `x-organization-id`, `X-Tenant-Id` | Yes |
| **LangGraph (Assistants)** | Deployment-level | `x-api-key` only | No |

#### LangSmith (Organization/Workspace Model)

LangSmith uses hierarchical multi-tenancy:
- Organizations contain multiple workspaces
- Workspaces contain prompts
- API requests can be scoped to org or workspace via headers
- Headers: `x-organization-id`, `X-Tenant-Id`

**SDK Implementation:** The `langsmith_*()` methods in `client.rs` add organization and workspace headers when configured.

#### LangGraph (Deployment Model)

LangGraph uses deployment-level resources:
- Assistants belong to a specific deployment
- Access controlled by API key (tied to deployment)
- No additional scoping headers needed
- Simpler model for graph-based applications

**SDK Implementation:** The `langgraph_*()` methods in `client.rs` do NOT add scoping headers, as assistants are deployment-level resources.

**Key Insight**: This architectural difference is reflected throughout the codebase:
- CLI flag design (prompts have `--organization-id`/`--workspace-id`, assistants don't)
- Configuration file structure (separate sections for each service)
- Error messages (guide users to correct API key for each service)
- Documentation (emphasizes the scoping difference)

For detailed architecture documentation, see [docs/architecture.md](./docs/architecture.md).

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
