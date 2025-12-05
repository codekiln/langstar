# Sessions vs Projects Terminology Research - langsmith-mcp-server

**Research Date**: 2025-12-05
**Related Issue**: #574
**Related PR**: #575
**Purpose**: Examine terminology usage in Model Context Protocol (MCP) server for LangSmith

## Executive Summary

The langsmith-mcp-server, a production-ready server that provides AI model access to LangSmith capabilities, exclusively uses "project" terminology in all tool definitions, parameters, and documentation. This represents another independent validation that "project" is the canonical user-facing term across the LangChain ecosystem.

## Repository Overview

**Repository**: langchain-ai/langsmith-mcp-server
**Type**: MCP (Model Context Protocol) server implementation
**Purpose**: Enable AI models to interact programmatically with LangSmith
**Language**: Python (FastMCP framework)
**Target Users**: AI assistant integrations (Claude Desktop, Cursor, etc.)

## Architecture Context

The MCP server exposes LangSmith functionality as tools that AI models can call. Tool naming and parameters are critical because they form the "vocabulary" that AI models use to interact with LangSmith. The choice of "project" vs "session" directly impacts how AI models (and their users) conceptualize LangSmith organization.

## Key Findings

### 1. Tool Parameter Naming

**From**: `langsmith_mcp_server/services/tools/traces.py:15`

```python
def fetch_trace_tool(
    client: Client,
    project_name: str = None,  # ← "project_name" parameter
    trace_id: str = None
) -> Dict[str, Any]:
    """
    Fetch the trace content for a specific project or specify a trace ID.

    Note: Only one of the parameters (project_name or trace_id) is required.
    trace_id is preferred if both are provided.

    Args:
        client: LangSmith client instance
        project_name: The name of the project to fetch the last trace for
        trace_id: The ID of the trace to fetch (preferred parameter)
    """
```

**Observations**:
- MCP tool uses `project_name`, not `session_name`
- Docstring explicitly refers to "project"
- AI models calling this tool will use "project" vocabulary

**Impact**:
- When Claude or other AI assistants use this tool, they say "fetching trace from project X"
- Reinforces "project" in natural language interactions between humans and AI
- Creates consistent terminology across human-AI-LangSmith interactions

### 2. SDK Method Calls (Internal Implementation)

**From**: `langsmith_mcp_server/services/tools/traces.py:42-59`

```python
# Get the last run
runs = client.list_runs(
    project_name=project_name if project_name else None,  # ← SDK uses "project_name"
    id=[trace_id] if trace_id else None,
    select=[
        "inputs",
        "outputs",
        "run_type",
        "id",
        "error",
        "total_tokens",
        "total_cost",
        "feedback_stats",
        "app_path",
        "thread_id",
    ],
    is_root=True,
    limit=1,
)
```

**Observations**:
- MCP server calls `client.list_runs(project_name=...)`
- Consistent with Python SDK's public API
- No translation layer needed (MCP uses same terminology as SDK)

### 3. Tool Function Naming

**Project-Related Tools**:

```python
# From traces.py
def fetch_trace_tool(client, project_name, trace_id)
def get_thread_history(client, thread_id, project_name)  # Takes project_name parameter
def get_project_runs_stats(client, project_name, is_last_run)  # Named with "project"
```

**Observations**:
- Function names include "project" (e.g., `get_project_runs_stats`)
- Parameters consistently use `project_name`
- No `get_session_runs_stats` or similar

### 4. Documentation and Comments

**From**: `traces.py` docstrings

```python
def get_thread_history(thread_id: str, project_name: str) -> Dict[str, Any]:
    """
    Fetch the thread history for a specific thread ID.

    Args:
        thread_id: The thread ID to fetch history for
        project_name: The project name where the thread exists

    Returns:
        Dictionary containing thread history and metadata
    """
```

**Observations**:
- Documentation uses "project" in natural language
- Clear semantic: thread exists within a project
- No ambiguity or need to explain "session" terminology

### 5. Error Messages

**From**: `traces.py:64`

```python
if not runs or len(runs) == 0:
    return {"error": "No runs found for project_name: {}".format(project_name)}
```

**Observations**:
- Error messages refer to "project_name"
- Users see "project" when things go wrong
- Consistent messaging throughout error paths

## MCP Tool Interface Analysis

### Why Tool Names Matter for AI

MCP tools form the "API" that AI models use. The terminology in tool names and parameters influences:

1. **AI Model Reasoning**: Models use parameter names to understand concepts
2. **User Instructions**: Users tell AI "get the project stats" not "get the session stats"
3. **Documentation Generation**: Tools auto-generate usage docs with these names
4. **Mental Model Reinforcement**: Consistent terminology across human-AI interactions

### Hypothetical Comparison

**If MCP Used "Session"**:
```python
def fetch_trace_tool(client, session_name: str = None, trace_id: str = None):
    """Fetch the trace content for a specific session"""
    runs = client.list_runs(project_name=session_name, ...)  # Mismatch!
```

**Problems**:
- Mismatch between MCP parameter (`session_name`) and SDK parameter (`project_name`)
- Translation layer needed to convert terminology
- User says "session" but SDK docs say "project"
- Confusion when debugging or reading SDK docs

**Actual Implementation**:
```python
def fetch_trace_tool(client, project_name: str = None, trace_id: str = None):
    """Fetch the trace content for a specific project"""
    runs = client.list_runs(project_name=project_name, ...)  # Perfect match!
```

**Benefits**:
- One-to-one mapping between MCP and SDK terminology
- No translation needed
- Users can reference SDK docs directly
- Consistent vocabulary across entire toolchain

## Thread vs Project Semantics

**Interesting Distinction in MCP**:

```python
def get_thread_history(thread_id: str, project_name: str):
    """Fetch the thread history for a specific thread ID within a project"""
```

**Hierarchical Relationship**:
- Threads exist **within** projects
- Projects **contain** threads (conversations)
- Natural semantic hierarchy

**Reinforces "Project" as Container**:
- Projects are the organizational unit
- Threads are a feature within projects
- Aligns with "project = collection of traces" definition

**Note**: Thread uses `thread_id` (not `session_id` in parameter names), though metadata may use `session_id` for backwards compatibility. This shows LangChain moving towards clearer terminology even for threads.

## Production Integration Context

### Where This MCP Server Is Used

**Integration Examples** (from smithery.yml and README):
- **Claude Desktop**: Official Anthropic AI assistant
- **Cursor**: AI-powered code editor
- **Custom AI Agents**: Any MCP-compatible AI system

**User Interaction Pattern**:
```
User: "Show me the stats for my chatbot project"
      ↓
AI Model: Calls get_project_runs_stats(project_name="chatbot")
      ↓
MCP Server: Fetches from LangSmith using project_name parameter
      ↓
AI Model: "Here are the stats for your chatbot project..."
```

**Terminology Flow**:
User → "project" → AI Model → "project" → MCP → "project" → SDK → "project" → API

**Consistency**: User's vocabulary matches the entire toolchain

## No "Session" Terminology Found

**Comprehensive Search Results**:
- ❌ No `session_name` parameters in any tools
- ❌ No `get_session_*` function names
- ❌ No "session" in tool docstrings (except `thread_id`/`session_id` metadata)
- ❌ No session-related error messages

**Only "Project" References**:
- ✅ `project_name` parameters throughout
- ✅ `get_project_runs_stats` function name
- ✅ "project" in all docstrings and comments
- ✅ "project" in error messages

## Architectural Significance

### MCP as Validation

The MCP server provides unique validation because:

1. **Independent Implementation**: Different codebase from SDK
2. **Different Use Case**: AI-to-API bridge, not direct Python usage
3. **Different Developers**: Could have made different terminology choices
4. **Same Result**: Still chose "project" terminology

**Implication**: "Project" wasn't just a Python SDK decision - it's an ecosystem-wide standard.

### Tool Design Philosophy

**From MCP Tool Signatures**:
```python
fetch_trace_tool(project_name: str, trace_id: str)
get_thread_history(thread_id: str, project_name: str)
get_project_runs_stats(project_name: str, is_last_run: str)
```

**Design Principles Evident**:
1. **Consistency**: All tools use `project_name`
2. **Clarity**: Parameter names match concepts
3. **User-Centric**: Names chosen for user understanding, not API accuracy
4. **SDK Alignment**: Perfect match with Python SDK parameters

## Implications for Rust SDK

### Rust SDK as CLI Backend

The langstar CLI will be similar to the MCP server:
- **Layer**: Sits between users and LangSmith API
- **Purpose**: Provide user-friendly interface to LangSmith
- **Users**: Developers and automation scripts

**Parallel to MCP**:
- MCP: AI models → MCP tools → LangSmith SDK → LangSmith API
- Langstar: Users → CLI commands → Rust SDK → LangSmith API

**Lesson from MCP**:
- MCP chose "project" for tool parameters
- Langstar should choose "project" for CLI flags and SDK methods
- Same reasoning applies: user-facing terminology should be user-friendly

### Consistency Across Interfaces

**Human Interfaces**:
- Python SDK: `client.list_projects()`
- JS SDK: `client.listProjects()`
- MCP Tools: `get_project_runs_stats(project_name=...)`
- Langstar CLI: `langstar project list` ← Should match

**Benefit**: Users can switch between interfaces without relearning terminology

## Conclusion

The langsmith-mcp-server provides strong additional evidence for "project" terminology:

1. **Production System**: Real-world MCP server uses "project" in all tools
2. **AI Integration**: Tool parameters that AI models use say "project_name"
3. **Independent Validation**: Different codebase, same terminology choice
4. **Consistent Implementation**: Perfect alignment with Python SDK parameters
5. **User Experience**: Natural language flow from users through AI to API all uses "project"

**Recommendation for Rust SDK**: The MCP server demonstrates that "project" is the correct choice even in novel interfaces (AI tools). The Rust SDK should confidently use "project" terminology for CLI commands, SDK methods, and documentation.

**Key Insight**: The MCP server could have used "session" to match the API more literally, but deliberately chose "project" for user comprehension. The Rust SDK should follow this same philosophy.

## Files Examined

- `langsmith_mcp_server/services/tools/traces.py` - Core trace fetching tools
- `langsmith_mcp_server/services/tools/experiments.py` - Experiment management
- `langsmith_mcp_server/services/tools/datasets.py` - Dataset operations
- `langsmith_mcp_server/services/register_tools.py` - Tool registration system
- `langsmith_mcp_server/server.py` - MCP server main entry point

All files consistently use "project" terminology in user-facing tool interfaces.

## Additional Context: MCP Server Architecture

For comprehensive details on the MCP server architecture and tool design, see the extensive CLAUDE.md file in the langsmith-mcp-server repository notes directory (co-located with this file).
