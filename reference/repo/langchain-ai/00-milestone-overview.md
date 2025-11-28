# LangSmith Reference Resources - Milestone Overview

This document maps project milestones to relevant resources from the LangSmith reference repositories.

## Reference Repositories

### langsmith-cookbook
**Location**: `reference/repo/langchain-ai/langsmith-cookbook/`
**Repository**: https://github.com/langchain-ai/langsmith-cookbook
**Description**: Collection of practical tutorials and examples for LangSmith platform features

### langsmith-mcp-server
**Location**: `reference/repo/langchain-ai/langsmith-mcp-server/`
**Repository**: https://github.com/langchain-ai/langsmith-mcp-server
**Description**: Model Context Protocol (MCP) server for LangSmith integration

---

## Milestone Mapping

### 1. devcontainer-feature

**Status**: Active development
**Goal**: Create a devcontainer feature for langstar CLI installation

#### Relevant Resources

**MCP Server Integration:**
- **Tool**: `langsmith-mcp-server/` - Full MCP server implementation for LangSmith
- **Architecture**: Study `langsmith_mcp_server/server.py` for FastMCP patterns
- **Configuration**: Review `smithery.yml` for deployment patterns
- **Dockerfile**: Alpine-based containerization in `Dockerfile` (useful for devcontainer setup)

**Development Environment Patterns:**
- **Setup Scripts**: `langsmith-mcp-server/Makefile` - development workflow automation
- **Dependencies**: `pyproject.toml` - modern Python packaging with uv/pdm-backend
- **Testing**: `tests/` directory structure for pytest integration

#### Why These Matter
The MCP server demonstrates production-ready containerization and development environment setup that can inform our devcontainer feature design.

---

### 2. new-release-ci

**Status**: Active development
**Goal**: Implement automated release CI/CD workflows

#### Relevant Resources

**CI/CD Patterns:**
- **GitHub Actions**: `langsmith-mcp-server/.github/workflows/` - CI automation examples
- **Testing Automation**: `Makefile` commands for lint, format, test workflows
- **Release Management**: Check for release workflow patterns

**Package Management:**
- **PyPI Publishing**: `pyproject.toml` configuration for package distribution
- **Version Management**: Python packaging standards for semantic versioning

#### Why These Matter
Both repositories demonstrate modern CI/CD practices that can inform our release automation strategy.

---

### 3. ls-runs-query

**Status**: Planning/Development
**Goal**: Implement SDK and CLI for querying LangSmith runs (traces)

#### Relevant Resources

**Cookbook Examples:**
- **Tracing Basics**: `tracing-examples/traceable/tracing_without_langchain.ipynb` - Core tracing concepts
- **REST API**: `tracing-examples/rest/rest.ipynb` - REST API patterns for runs/traces
- **Nested Runs**: `tracing-examples/nesting-tools/nest_runs_within_tools.ipynb` - Understanding run hierarchies

**MCP Server Implementation:**
- **Traces Module**: `langsmith_mcp_server/services/tools/traces.py` - Production trace querying
- **Tools Available**:
  - `fetch_runs()` - Flexible run fetching with filters and query language
  - `list_projects()` - Project enumeration for run context
  - `get_project_runs_stats()` - Aggregate statistics for runs
  - `fetch_trace()` - Individual trace retrieval by ID

**Key Code References:**
- API client wrapper: `langsmith_mcp_server/langsmith_client.py`
- Trace data structures and filtering logic in services/tools/traces.py

#### Why These Matter
The MCP server provides a production-ready implementation of trace/run querying that directly maps to our SDK requirements. The cookbook examples demonstrate real-world usage patterns.

---

### 4. ls-annotation-queues

**Status**: Planning/Development
**Goal**: Implement SDK and CLI for LangSmith annotation queue operations

#### Relevant Resources

**Cookbook Examples:**
- **Feedback Systems**: `feedback-examples/` directory
  - `algorithmic-feedback/algorithmic_feedback.ipynb` - Automated feedback pipelines
  - `realtime-algorithmic-feedback/realtime_feedback.ipynb` - Real-time feedback collection
  - `streamlit/` - Interactive feedback capture in applications
- **Evaluation Patterns**: `testing-examples/` - Evaluation workflows that feed annotation queues

**MCP Server Capabilities:**
While annotation queues aren't explicitly listed in current MCP tools, related functionality includes:
- **Feedback Systems**: Integration patterns for user and automated feedback
- **Project Management**: `list_projects()` for organizing annotation work
- **Data Flow**: Understanding how feedback connects to evaluation workflows

**Related Concepts:**
- Annotation queues are used for human review and labeling of LLM outputs
- Connected to feedback collection and evaluation pipelines
- See cookbook feedback examples for end-to-end workflows

#### Why These Matter
Understanding feedback collection patterns helps design annotation queue APIs. The cookbook shows how annotation fits into the evaluation lifecycle.

---

### 5. ls-datasets

**Status**: Active development (types implemented, client methods in progress)
**Goal**: Implement SDK and CLI for LangSmith dataset operations

#### Relevant Resources

**Cookbook Examples:**
- **Dataset Creation**: `testing-examples/` - Creating datasets for evaluation
- **Using Fixed Sources**: `testing-examples/using-fixed-sources/using_fixed_sources.ipynb` - Dataset-driven testing
- **Dynamic Data**: `testing-examples/dynamic-data/testing_dynamic_data.ipynb` - Datasets with changing data
- **Backtesting**: `testing-examples/backtesting/backtesting.ipynb` - Converting production runs to datasets
- **Export/Import**: `testing-examples/export-test-to-csv/export-test-to-csv.ipynb` - Dataset export patterns

**MCP Server Implementation:**
- **Datasets Module**: `langsmith_mcp_server/services/tools/datasets.py` - Full dataset CRUD
- **Tools Available**:
  - `list_datasets()` - Flexible dataset discovery with filtering
  - `read_dataset()` - Individual dataset retrieval
  - `list_examples()` - Dataset example enumeration with pagination
  - `read_example()` - Individual example access with versioning
  - `create_dataset()` - Documentation for dataset creation (doc-only tool)
  - `update_examples()` - Documentation for example updates (doc-only tool)

**Key Features to Study:**
- **Filtering**: By ID, name, type, metadata, creation date
- **Pagination**: Handling large datasets efficiently
- **Versioning**: `as_of` parameter for historical access
- **Data Types**: Chat datasets, key-value pairs, custom formats
- **Examples Management**: CRUD operations on dataset examples

**Reference Implementation:**
```python
# From langsmith_mcp_server/services/tools/datasets.py
@mcp.tool()
def list_datasets(
    dataset_ids: List[str] = None,
    data_type: str = None,
    dataset_name: str = None,
    ...
) -> Dict[str, Any]
```

#### Why These Matter
The MCP server provides a complete reference implementation for dataset operations. The cookbook examples demonstrate real-world usage patterns and data flows for testing and evaluation.

---

## Cross-Cutting Resources

### API Specifications
- **OpenAPI Spec**: Mentioned in cookbook (LangSmith SDK openapi.yaml)
- **REST API Examples**: `tracing-examples/rest/rest.ipynb` demonstrates direct API usage
- **See also**: `reference/api-specs/LANGSMITH_API_OVERVIEW.md` in this project

### SDK Architecture Patterns
**From MCP Server:**
- **Client Wrapper**: `langsmith_client.py` - thin abstraction over LangSmith Python SDK
- **Service Layer**: Modular organization (tools/, prompts/, resources/)
- **Error Handling**: Graceful degradation, validation, timeout management
- **Type Safety**: Full type hints and mypy checking

### Testing & Quality
**From MCP Server:**
- **Test Structure**: `tests/` directory organization
- **Pytest Configuration**: `pyproject.toml` test settings
- **Socket Restrictions**: Security-focused testing
- **MCP Inspector**: Interactive testing tool (`mcp dev server.py`)

**From Cookbook:**
- **Example Notebooks**: Extensive testing and evaluation examples
- **Real-world Patterns**: Production-proven testing strategies

---

## Getting Started

### For New Contributors

1. **Explore Cookbook Examples**
   ```bash
   cd reference/repo/langchain-ai/langsmith-cookbook/code/
   # Browse tracing-examples/, testing-examples/, feedback-examples/
   ```

2. **Study MCP Server Implementation**
   ```bash
   cd reference/repo/langchain-ai/langsmith-mcp-server/code/
   # Read CLAUDE.md for architectural deep-dive
   # Review langsmith_mcp_server/ source code
   ```

3. **Map to Current Milestone**
   - Use this document to find relevant examples for your work
   - Cross-reference with project issues and API specs
   - Study both cookbook usage patterns AND MCP server implementation

### For SDK Development

When implementing SDK features:
1. **Check cookbook** for usage patterns and examples
2. **Review MCP server** for implementation reference
3. **Consult OpenAPI specs** for authoritative API contracts
4. **Study both repos together** - cookbook shows "how to use", MCP shows "how to implement"

### For CLI Development

When building CLI commands:
1. **Study MCP tool definitions** for parameter patterns
2. **Review cookbook examples** for user workflows
3. **Consider output formats** shown in both repos
4. **Align with SDK** being developed in parallel

---

## Notes Structure

Each reference repo has its own notes subdirectory:

```
reference/repo/langchain-ai/
├── 00-milestone-overview.md (this file)
├── langsmith-cookbook/
│   ├── notes/
│   │   └── README.md
│   └── code/ (gitignored clone)
└── langsmith-mcp-server/
    ├── notes/
    │   └── README.md
    └── code/ (gitignored clone)
```

**Notes Directory Usage:**
- Add research findings and insights to notes/
- Keep code/ as read-only reference (can be re-cloned anytime)
- Create additional markdown files as needed
- Commit notes with your branch work

---

## Maintenance

This overview should be updated:
- When new milestones are created
- When discovering particularly relevant examples in the reference repos
- When reference repos are updated with new features
- As we complete milestones and learn what resources were most valuable

---

Last Updated: 2025-11-28
