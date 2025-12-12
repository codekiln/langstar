# langsmith-mcp-server

## Repository Information

- **Repository**: [langchain-ai/langsmith-mcp-server](https://github.com/langchain-ai/langsmith-mcp-server)
- **Date Created**: 2025-11-28
- **Cloned to**: `/workspace/reference/repo/langchain-ai/langsmith-mcp-server/code`
- **MCP Docs**: https://modelcontextprotocol.io/

## Purpose

This repository provides a **production-ready reference implementation** of LangSmith API integration in Python. It's our reference for understanding:

- How to **implement** LangSmith API clients
- Proper error handling and validation patterns
- API parameter structures and filtering logic
- Tool/CLI interface design patterns

Use this MCP server to:

1. **Study implementation patterns** before writing Rust SDK equivalents
2. **Understand API behavior** through working Python code
3. **Identify edge cases** handled in production code
4. **Learn parameter validation** and error handling strategies

## What is This Repository?

**Model Context Protocol (MCP) Server** = A standardized way for AI assistants to access external tools and data.

This specific MCP server exposes LangSmith platform features as tools that can be called by AI models. While we're not building an MCP server ourselves, this codebase is valuable because:

- It's a **complete client implementation** of the LangSmith API
- It's **actively maintained** by the LangChain team
- It shows **production-quality** error handling and validation
- It has **comprehensive tool definitions** that map directly to SDK methods we need

## Repository Structure

```
langsmith-mcp-server/
├── langsmith_mcp_server/
│   ├── server.py              # FastMCP server setup (less relevant for us)
│   ├── langsmith_client.py    # ⭐ Thin wrapper around LangSmith SDK
│   └── services/
│       ├── tools/
│       │   ├── traces.py      # ⭐ Runs & traces implementation
│       │   ├── datasets.py    # ⭐ Dataset operations implementation
│       │   ├── prompts.py     # Prompt management
│       │   └── experiments.py # Evaluation experiments
│       ├── prompts/           # MCP prompt templates (less relevant)
│       └── resources/         # Documentation resources (less relevant)
├── tests/                     # ⭐ Test patterns and example usage
├── pyproject.toml            # ⭐ Dependency management patterns
└── Makefile                  # ⭐ Development workflow automation
```

**Key**: ⭐ = Most relevant for our SDK development

## Key Files by Milestone

### For `ls-runs-query` (Milestone 3)

**Must-read file:**

- `langsmith_mcp_server/services/tools/traces.py` - Complete traces/runs implementation

**Key tools defined:**

```python
fetch_runs(          # Primary run querying tool
    project_name: str,
    run_ids: List[str],
    filter_query: str,  # Query language expressions
    trace_filter: str,  # Trace-level constraints
    ...
)

list_projects(       # Project enumeration
    limit: int,
    include_details: bool,
    ...
)

fetch_trace(         # Individual trace retrieval
    project_name: str,
    trace_id: str,
    ...
)

get_project_runs_stats(  # Aggregate statistics
    project_name: str,
    is_last_run: bool,
    ...
)
```

**Implementation insights:**

- How to handle flexible filtering (see `filter_query` usage)
- Project name format variations ("owner/project" vs "project")
- Pagination patterns
- Error handling for missing projects/traces

### For `ls-datasets` (Milestone 5)

**Must-read file:**

- `langsmith_mcp_server/services/tools/datasets.py` - Complete dataset CRUD

**Key tools defined:**

```python
list_datasets(       # Dataset discovery
    dataset_ids: List[str],
    data_type: str,
    dataset_name: str,
    metadata: Dict,
    ...
)

read_dataset(        # Fetch single dataset
    dataset_id: str,
    dataset_name: str,
    ...
)

list_examples(       # Example enumeration
    dataset_id: str,
    dataset_name: str,
    limit: int,
    offset: int,
    ...
)

read_example(        # Fetch single example
    example_id: str,
    as_of: str,      # Version support!
    ...
)

# Note: create_dataset and update_examples are doc-only tools
# (They just return documentation, don't actually implement it)
```

**Implementation insights:**

- Flexible filtering by multiple criteria
- Pagination with limit/offset
- Versioning support via `as_of` parameter
- Dataset vs Examples separation
- Metadata querying patterns

### For `ls-annotation-queues` (Milestone 4)

**Current status**: Not explicitly implemented in MCP server yet (marked as "under development")

**Related patterns to study:**

- Project and workspace management patterns
- Feedback/annotation conceptual relationship
- How to structure queue-based operations

## Architecture Patterns to Learn

### 1. Client Wrapper Pattern (`langsmith_client.py`)

```python
class LangSmithClient:
    """Thin wrapper around LangSmith SDK"""

    def __init__(self, api_key: str, workspace_id: Optional[str] = None):
        self.client = Client(api_key=api_key)
        # Centralized client initialization
```

**Lesson**: Keep client wrapper thin, delegate to SDK. Good separation of concerns.

### 2. Service Layer Pattern (`services/tools/*.py`)

Each service module:

- Defines related tools together
- Uses `@mcp.tool()` decorators (analogous to CLI commands)
- Validates parameters before SDK calls
- Returns consistent error format

**Lesson**: Group related operations, validate early, consistent error handling.

### 3. Error Handling Pattern

```python
if not langsmith_client:
    return {"error": "LangSmith client not initialized"}

try:
    # SDK operation
    result = langsmith_client.client.list_datasets(...)
    return {"datasets": [dataset.dict() for dataset in result]}
except Exception as e:
    return {"error": str(e)}
```

**Lesson**: Graceful degradation, structured error responses, no uncaught exceptions.

### 4. Parameter Validation Pattern

```python
def list_examples(
    dataset_id: str = None,
    dataset_name: str = None,
    ...
):
    if not dataset_id and not dataset_name:
        return {"error": "Must provide dataset_id or dataset_name"}
```

**Lesson**: Validate mutually exclusive parameters, provide clear error messages.

## Development Patterns to Adopt

### Build System

**From `pyproject.toml`:**

- Modern Python packaging with `pdm-backend`
- Explicit dependencies with version constraints
- Development dependencies separated (`[tool.pdm.dev-dependencies]`)
- Entry points for CLI commands

**Translation to Rust:**

- `Cargo.toml` workspace structure
- Feature flags for optional dependencies
- Binary entry points in `[[bin]]` sections

### Testing

**From `tests/` and `pyproject.toml`:**

```toml
[tool.pytest.ini_options]
asyncio_mode = "auto"
addopts = "--disable-socket"  # Security: no network in tests
```

**Lesson**:

- Async test support (important for API clients)
- Disable network in unit tests (use mocks)
- Integration tests separate from unit tests

### Development Workflow

**From `Makefile`:**

```makefile
lint:    # Run linting
format:  # Auto-format code
test:    # Run test suite
```

**Lesson**: Standardize common operations, make it easy to run pre-commit checks.

## How to Use This Reference

### When Implementing SDK Features

1. **Locate the corresponding service module**
   - Runs/traces → `services/tools/traces.py`
   - Datasets → `services/tools/datasets.py`

2. **Study the tool implementation**
   - What parameters does it accept?
   - What validation is performed?
   - How are SDK methods called?
   - How are results formatted?

3. **Identify equivalent Rust patterns**
   - Python dicts → Rust structs
   - Optional parameters → `Option<T>`
   - List comprehensions → `.iter().map()`
   - Error dicts → `Result<T, E>`

4. **Implement in Rust with same ergonomics**
   - Match the parameter names
   - Preserve the filtering logic
   - Maintain the error handling strategy

### Example: Translating `list_datasets` to Rust

**Python (MCP Server):**

```python
def list_datasets(
    dataset_ids: List[str] = None,
    data_type: str = None,
    dataset_name: str = None,
    limit: int = 100,
) -> Dict[str, Any]:
    if not langsmith_client:
        return {"error": "Client not initialized"}

    try:
        datasets = langsmith_client.client.list_datasets(
            dataset_ids=dataset_ids,
            data_type=DataType(data_type) if data_type else None,
            dataset_name=dataset_name,
            limit=limit,
        )
        return {"datasets": [d.dict() for d in datasets]}
    except Exception as e:
        return {"error": str(e)}
```

**Rust (Our SDK - conceptual):**

```rust
pub struct ListDatasetsRequest {
    pub dataset_ids: Option<Vec<String>>,
    pub data_type: Option<DataType>,
    pub dataset_name: Option<String>,
    pub limit: Option<u32>,
}

impl Client {
    pub async fn list_datasets(
        &self,
        request: ListDatasetsRequest,
    ) -> Result<Vec<Dataset>, Error> {
        // Validation
        // HTTP request
        // Deserialize response
        // Return typed result
    }
}
```

## Quick Reference

**Comprehensive technical deep-dive**: `code/CLAUDE.md` - Excellent architectural overview!

**Main implementation**: `code/langsmith_mcp_server/` - Service layer tools

**Tests**: `code/tests/` - Example usage patterns

## Notes & Findings

### Finding: Documentation-Only Tools

Some tools (`create_dataset`, `update_examples`, `push_prompt`, `run_experiment`) are marked as "documentation-only":

```python
return {
    "message": "This is a documentation-only tool...",
    "refer_to": "https://docs.smith.langchain.com/..."
}
```

**Insight**: These operations may be complex enough that the MCP server punts to official docs. For our SDK, we'll need to fully implement these based on the OpenAPI spec.

### Finding: Workspace vs Organization

The MCP server accepts `workspace_id` in configuration but it's optional:

```python
def __init__(self, api_key: str, workspace_id: Optional[str] = None):
```

**Insight**: API keys can be scoped to single workspace (no workspace_id needed) or multiple workspaces (workspace_id required). Our SDK should support both patterns.

### Finding: Query Language Support

`fetch_runs` accepts `filter_query` parameter - a query language for complex filtering:

```python
filter_query: str  # e.g., 'eq(name, "my-run") and gt(latency, 100)'
```

**Insight**: LangSmith has a query DSL. We should document this and provide examples, even if we just pass strings through initially.

### Finding: Project Name Formats

The code handles multiple project name formats:

- `"owner/project"` - Fully qualified
- `"project"` - Implicit owner (current user/org)

**Insight**: Our SDK should accept both formats and handle them correctly.

## Additional Resources

- **FastMCP Framework**: The MCP server uses this - not relevant for us, but shows modern Python async patterns
- **LangSmith Python SDK**: The underlying SDK this wraps - our conceptual equivalent
- **MCP Specification**: https://modelcontextprotocol.io/ - Interesting for understanding tool-based interfaces
