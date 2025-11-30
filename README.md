# Langstar

[![CI](https://github.com/codekiln/langstar/workflows/CI/badge.svg)](https://github.com/codekiln/langstar/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-alpha-orange.svg)](https://github.com/codekiln/langstar/issues)

> **⚠️ Alpha Status**: This project is in early development. APIs and features may change. Use with caution.

**Langstar** is a unified CLI for the LangChain ecosystem, providing ergonomic access to LangSmith, LangGraph Cloud, and other LangChain services.

## Features

- **Multiple Output Formats** - JSON for scripting, tables for human readability
- **Configuration Management** - Support for config files and environment variables
- **Automation-Friendly** - Designed for both interactive use and AI agent invocation

## Quick Start

### Installation

#### Quick Install (Recommended for Users)

Install the latest release with our installer script:

```bash
# Quick install (recommended)
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash
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
./install.sh --version 0.4.3

# Install to custom location
./install.sh --prefix ~/.local/bin

# See all options
./install.sh --help
```

The installer script:
- ✅ Downloads pre-built binaries (no compilation needed)
- ✅ Verifies SHA256 checksums
- ✅ Supports Linux (x86_64, aarch64) and macOS (Intel/Apple Silicon)
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

#### DevContainer Feature (For Development Containers)

The easiest way to use Langstar in development containers is via our official DevContainer feature:

```jsonc
// Add to .devcontainer/devcontainer.json
{
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

**Benefits:**
- ✅ Automatic installation in devcontainers, GitHub Codespaces, and devpod
- ✅ Supports x86_64 and ARM64 architectures
- ✅ Version pinning support (`"latest"` or specific version like `"v0.4.0"`)
- ✅ Tested across Ubuntu, Debian, and Alpine base images

For complete documentation and examples, see [docs/devcontainer-feature.md](./docs/devcontainer-feature.md).

> **Note:** We plan to submit this feature to the official [DevContainers index](https://containers.dev/features) for increased discoverability in VS Code and GitHub Codespaces. Track progress in [#285](https://github.com/codekiln/langstar/issues/285).

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

# Push a prompt with structured output (constrained schema)
langstar prompt push \
  -o owner -r my-prompt \
  -t "Extract data from: {input}" \
  --schema ./schemas/extraction.json

# Push with specific structured output method
langstar prompt push \
  -o owner -r my-prompt \
  -t "Analyze: {text}" \
  --schema ./schemas/analysis.json \
  --schema-method function_calling

# Pull a prompt (shows schema if structured)
langstar prompt pull owner/my-prompt
```

**Structured Output Prompts:**

Langstar supports creating prompts with JSON Schema constraints that ensure LLM outputs match a predefined structure. This enables reliable data extraction, API response formatting, and typed output handling.

```bash
# Create a JSON schema file
cat > invoice-schema.json << 'EOF'
{
  "type": "object",
  "properties": {
    "invoice_number": {"type": "string"},
    "date": {"type": "string", "format": "date"},
    "amount": {"type": "number"},
    "vendor": {"type": "string"}
  },
  "required": ["invoice_number", "amount"]
}
EOF

# Push structured prompt
langstar prompt push \
  -o team -r invoice-extractor \
  -t "Extract invoice data from: {document}" \
  --schema invoice-schema.json
```

For detailed examples and workflows, see [docs/examples/structured-output-prompts.md](./docs/examples/structured-output-prompts.md).

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

#### LangSmith Runs/Traces

Query and filter LangSmith runs (traces) to analyze LLM application execution.

```bash
# Query recent runs (root traces only)
langstar runs query --is-root --limit 10

# Query with JSON output
langstar runs query --limit 5 --output json

# Filter by run type (llm, chain, tool, retriever, embedding, prompt, parser)
langstar runs query --run-type llm --limit 10

# Filter by status
langstar runs query --status error --limit 20
langstar runs query --errors-only  # Shorthand for error runs

# Filter by tags
langstar runs query --tag production --tag gpt-4

# Filter by metadata
langstar runs query --meta environment=production --meta model=gpt-4

# Use raw filter expressions (LangSmith filter query language)
langstar runs query --filter 'eq(status, "error")'
langstar runs query --filter 'gt(total_tokens, 1000)'
langstar runs query --filter 'has(tags, "production")'

# Combine filters
langstar runs query --tag production --status error --run-type llm

# Time-based filtering
langstar runs query --since 2024-01-01T00:00:00Z --until 2024-01-31T23:59:59Z

# Sort order (asc or desc)
langstar runs query --order asc --limit 10

# Pretty-printed JSON output
langstar runs query --limit 5 --output json-pretty
```

**Run Types:**
- `llm` - LLM (Language Model) calls
- `chain` - Chain executions
- `tool` - Tool invocations
- `retriever` - Retriever operations
- `embedding` - Embedding operations
- `prompt` - Prompt template executions
- `parser` - Output parser executions

**Filter Query Language:**
The `--filter` option accepts LangSmith filter expressions:
- `eq(field, value)` - Equals
- `neq(field, value)` - Not equals
- `gt(field, value)` / `gte(field, value)` - Greater than (or equal)
- `lt(field, value)` / `lte(field, value)` - Less than (or equal)
- `has(array_field, value)` - Array contains
- `and(expr1, expr2)` / `or(expr1, expr2)` - Logical operators

#### Annotation Queues

Manage LangSmith annotation queues for human review and labeling workflows.

```bash
# Create an annotation queue
langstar queue create --name "Error Review" --description "Review production errors"

# List queues
langstar queue list
langstar queue list --name-contains "review"

# Get queue details
langstar queue get <queue-id>

# Add runs to a queue for review
langstar queue add-runs <queue-id> <run-id-1> <run-id-2>

# Add runs from a file (one UUID per line)
langstar queue add-runs <queue-id> --runs-file runs.txt

# List items in a queue
langstar queue items <queue-id> --limit 50

# Remove a run from queue
langstar queue remove-run <queue-id> <run-id>

# Delete a queue
langstar queue delete <queue-id> --force
```

**Queue Types:**
- `single` - Review runs individually (default)
- `pairwise` - Compare two runs side-by-side

For detailed usage and CI/CD examples, see [docs/queues.md](./docs/queues.md).

#### Dataset Management

Manage LangSmith datasets and examples for testing, evaluation, and fine-tuning workflows.

```bash
# Create a new dataset
langstar dataset create --name "my-qa-dataset" --data-type kv

# List all datasets
langstar dataset list
langstar dataset list --name-contains "test" --data-type chat

# Get dataset details
langstar dataset get <dataset-id>

# Update dataset metadata
langstar dataset update <dataset-id> --name "updated-name" --description "New description"

# Delete a dataset
langstar dataset delete <dataset-id> --yes

# Import examples from JSONL or CSV
langstar dataset import <dataset-id> --file examples.jsonl
langstar dataset import <dataset-id> --file data.csv --format csv

# List examples in a dataset
langstar dataset list-examples <dataset-id> --limit 50

# Export examples to file
langstar dataset export <dataset-id> --format jsonl --out backup.jsonl
langstar dataset export <dataset-id> --format csv --out data.csv

# JSON output
langstar dataset list --json
```

**Dataset Types:**
- `kv` - Key-value pairs (default) - generic input/output mapping
- `llm` - LLM completion format - prompt/completion pairs
- `chat` - Chat format - message-based conversations

**Import/Export Formats:**
- `jsonl` - JSON Lines format (recommended for programmatic access)
- `csv` - CSV format (convenient for spreadsheet tools)

For complete documentation including format specifications, SDK API reference, and common workflows, see [docs/datasets.md](./docs/datasets.md).

#### Evaluations

Run evaluations on datasets using heuristic or LLM-as-judge evaluators.

```bash
# Create evaluation with heuristic evaluator
langstar eval create \
  --name "exact-match-validation" \
  --dataset "my-test-dataset" \
  --evaluator exact-match

# Create evaluation with LLM judge (categorical scoring)
langstar eval create \
  --name "response-quality-judge" \
  --dataset "customer-support-dataset" \
  --evaluator llm-judge \
  --judge-model "claude-3-5-sonnet-20241022" \
  --judge-provider "anthropic" \
  --judge-prompt-file "./rubrics/quality.txt" \
  --score-type categorical \
  --score-choices "Poor,Fair,Good,Excellent" \
  --include-reasoning

# Create evaluation with LLM judge (continuous scoring)
langstar eval create \
  --name "relevance-score" \
  --dataset "qa-dataset" \
  --evaluator llm-judge \
  --judge-model "gpt-4" \
  --judge-provider "openai" \
  --judge-prompt-file "./rubrics/relevance.txt" \
  --score-type continuous \
  --score-min 0.0 \
  --score-max 1.0

# Run an evaluation
langstar eval run <eval-id>

# Preview evaluation (first 10 examples)
langstar eval run <eval-id> --preview 10

# Dry run (validate configuration only)
langstar eval run <eval-id> --dry-run

# List evaluations
langstar eval list
langstar eval list --name "quality"
langstar eval list --dataset <dataset-id>

# Get evaluation details
langstar eval get <eval-id>

# Export results
langstar eval export <eval-id> --format csv --output results.csv
langstar eval export <eval-id> --format jsonl --output results.jsonl
langstar eval export <eval-id> --format json --output results.json --include-metadata
```

**Evaluator Types:**

*Heuristic Evaluators (zero-cost, deterministic):*
- `exact-match` - Exact string equality check
- `contains` - Substring presence check
- `regex-match` - Regular expression pattern matching
- `json-valid` - JSON syntax validation
- `string-distance` - Levenshtein distance (fuzzy matching)

*LLM-as-Judge Evaluators (requires API calls):*
- `llm-judge` - Use an LLM to score outputs based on a rubric

**Environment Variables:**
- `LANGSMITH_API_KEY` - Required for all eval commands
- `ANTHROPIC_API_KEY` - Required for Anthropic judge models
- `OPENAI_API_KEY` - Required for OpenAI judge models

**Score Types for LLM Judge:**
- `categorical` - Predefined choices (e.g., Pass/Fail, Poor/Fair/Good/Excellent)
- `continuous` - Numeric range (e.g., 0.0-1.0, 1-10)

For detailed documentation including evaluator reference, judge prompt examples, and workflow patterns, see [docs/evals-implementation-plan.md](./docs/evals-implementation-plan.md).

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

Langstar takes the [OpenAPI specs for the various LangSmith APIs](reference/api-specs/LANGSMITH_API_OVERVIEW.md) and the [langsmith python SDKs](reference/repo/langchain-ai/langsmith-sdk/notes/README.md) as input, and uses them to implement a a thin wrapper over multiple LangSmith services.

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

This project follows a GitHub issue-driven development workflow.

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

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Resources

- [LangSmith Documentation](https://docs.smith.langchain.com/)
- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)
- [LangChain Documentation](https://python.langchain.com/)

## Developer Documentation

For coding conventions, best practices, and development guidelines, see the [Developer Documentation](./docs/dev/).

Key documentation:
- [GitHub Workflow](./docs/dev/github-workflow.md) - Issue-driven development process
- [Git SCM Conventions](./docs/dev/git-scm-conventions.md) - Commit message standards

## Setup

This project uses a devcontainer for consistent development environment. See [.devcontainer](./.devcontainer) for configuration details.

