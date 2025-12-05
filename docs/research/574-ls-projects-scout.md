# ls-projects Feasibility Scout

**Issue**: #574
**Date**: 2025-12-05
**Status**: Complete

## Executive Summary

**Feasibility**: **Go**

Research support for adding `langstar project list` and other langstar project commands to enable CRUD operations on LangSmith tracing projects. Each LangSmith deployment generates its own tracing project.

**Key Finding**: Projects in LangSmith are actually called "Sessions" at the API level (endpoint: `/sessions`). The Python SDK provides comprehensive project management through the sessions API, and langstar already references projects indirectly via the `--project` flag in the runs query command.

## 1. Existing Langstar Code

### Current Project References

**CLI (`cli/src/commands/runs.rs:30-34`)**:
```rust
/// Project name or UUID to query runs from
///
/// Can be specified multiple times to query from multiple projects.
#[arg(short, long = "project", value_name = "PROJECT")]
pub projects: Vec<String>,
```

**SDK (`sdk/src/runs.rs:93-94`)**:
```rust
/// Session/project ID this run belongs to
pub session_id: Uuid,
```

**Observation**:
- Projects are currently only referenced as filtering criteria for runs
- No direct project management commands exist (no `langstar project list`, `create`, `delete`, etc.)
- The SDK uses `session_id` terminology matching the API
- No existing `projects.rs` module in the SDK

## 2. Python SDK Precedent

**Location**: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py`

### Available Methods (Lines 3375-3780)

1. **`list_projects()`** (line 3704)
   - Filters: project_ids, name, name_contains, reference_dataset_id, reference_free, include_stats, metadata
   - Returns: Iterator of `TracerSessionResult`
   - Uses pagination with max 100 per page

2. **`read_project()`** (line 3531)
   - Parameters: project_id OR project_name, include_stats
   - Returns: Single `TracerSessionResult`

3. **`create_project()`** (line 3408)
   - Parameters: project_name (required), description, metadata, upsert, reference_dataset_id
   - Returns: `TracerSession`

4. **`update_project()`** (line 3455)
   - Parameters: project_id (required), name, description, metadata, end_time
   - Note: name change only allowed if project has end_time (closed)
   - Returns: `TracerSession`

5. **`delete_project()`** (line 3780)
   - Parameters: project_name OR project_id

6. **`has_project()`** (line 3572)
   - Parameters: project_name, optional project_id
   - Returns: bool

7. **`list_shared_projects()`** (line 3375)
   - Not examined in detail

### Schema: TracerSession (schemas.py:729-781)

**Key Fields**:
```python
class TracerSession(BaseModel):
    id: UUID
    start_time: datetime
    end_time: Optional[datetime]
    description: Optional[str]
    name: Optional[str]
    extra: Optional[dict[str, Any]]  # Contains metadata and tags
    tenant_id: UUID
    reference_dataset_id: Optional[UUID]
```

**Important Comment** (line 732): *"Sessions are also referred to as 'Projects' in the UI."*

## 3. API Endpoints

**Base URL**: `https://api.smith.langchain.com`

### REST Endpoints (from Python SDK)

| Operation | Method | Endpoint | Query Params | Request Body |
|-----------|--------|----------|--------------|--------------|
| **List** | GET | `/sessions` | limit, id, name, name_contains, reference_dataset, reference_free, include_stats, dataset_version, metadata | N/A |
| **Read** | GET | `/sessions/{id}` or `/sessions?name={name}` | include_stats, limit=1 | N/A |
| **Create** | POST | `/sessions` | upsert (optional) | `{"name": str, "description": str, "extra": dict, "id": UUID, "reference_dataset_id": UUID}` |
| **Update** | PATCH | `/sessions/{id}` | N/A | `{"name": str, "description": str, "extra": dict, "end_time": ISO8601}` |
| **Delete** | DELETE | `/sessions/{id}` | N/A | N/A |

### Request/Response Shapes

**List Response**: Array of TracerSessionResult objects
**Read Response**: Single TracerSessionResult object
**Create Response**: TracerSession object
**Update Response**: TracerSession object
**Delete Response**: Success/error status

## 4. Complexity Assessment

**Complexity**: **Medium**

### Why Medium (not Low):

1. **Terminology Mapping**: Sessions vs Projects
   - API uses "sessions", UI/SDK uses "projects"
   - Need consistent naming in Rust SDK and CLI
   - Recommend: Use "projects" in public API, map to "sessions" internally

2. **Pagination Required**: List endpoint returns max 100 per page
   - Need to implement cursor-based pagination
   - Similar to existing pagination in runs.rs

3. **Multiple Filtering Options**: List supports 8+ filter parameters
   - Need thoughtful CLI design for usability
   - Can start with subset (name, name_contains, limit)

4. **Extra/Metadata Handling**:
   - The `extra` field contains nested metadata and tags
   - Need proper JSON serialization/deserialization

### Why Not High:

- ✅ All CRUD operations follow standard REST patterns
- ✅ Python SDK provides clear precedent for behavior
- ✅ No complex authentication beyond existing API key
- ✅ No streaming or real-time requirements
- ✅ Similar patterns already implemented in datasets.rs, annotation_queues.rs

### Technical Considerations

**Rust SDK (`sdk/src/projects.rs`)**:
- Define `Project` struct matching TracerSession schema
- Implement `list_projects()`, `get_project()`, `create_project()`, `update_project()`, `delete_project()`
- Handle pagination for list operation
- Use serde for JSON serialization of extra/metadata fields

**CLI (`cli/src/commands/projects.rs`)**:
- Subcommands: `list`, `get`, `create`, `update`, `delete`
- Table output for list (similar to runs, datasets)
- JSON/YAML output options
- Filter flags: --name, --name-contains, --limit, --include-stats

**Estimated Implementation Effort**:
- SDK module: ~200-300 lines (similar to datasets.rs)
- CLI module: ~250-350 lines (similar to datasets CLI commands)
- Tests: ~150-200 lines

## 5. Experiments

**Experiment Conducted**: Projects vs Sessions Terminology Disambiguation

**Location**: `reference/experiments/574-ls-projects/`

### Experiment Goal

Empirically verify the "projects" vs "sessions" terminology mapping through actual API calls, since:
- Python SDK uses "projects" in method names
- Python SDK uses "TracerSession" in schema classes
- API endpoints use "/sessions" paths
- LangSmith UI refers to them as "Projects"

### Experiment Method

Created Python script (`test_projects.py`) that:
1. Calls `list_projects(limit=3)` to retrieve projects
2. Calls `read_project(project_name=...)` to read by name
3. Inspects returned object types and field names
4. Documents actual API behavior

### Key Findings

**1. Object Types Confirmed**
```python
>>> type(project)
TracerSessionResult

>>> project.id
UUID('55de255e-0405-4737-82b3-75ce7aaf22f3')

>>> project.name
'pr-integration-test-1764940001'
```

**2. Terminology Mapping Verified**

| Layer | Terminology |
|-------|-------------|
| LangSmith UI URLs | `https://smith.langchain.com/o/{tenant}/projects/p/{id}` |
| Python SDK Methods | `list_projects()`, `read_project()`, `create_project()` |
| Python Schema Classes | `TracerSession`, `TracerSessionResult` |
| REST API Endpoints | `GET /sessions`, `POST /sessions`, etc. |

**3. Field Names Analysis**
- Only 1 field uses "session" terminology: `session_feedback_stats`
- All other fields are neutral: `id`, `name`, `description`, `tenant_id`, etc.
- No `session_id` field exists (project ID is just `id`)

**4. URL Structure**
```
https://smith.langchain.com/o/{tenant_id}/projects/p/{project_id}
                                            ^^^^^^^^
```
The UI explicitly uses "projects/p/" not "sessions/s/"

### Experiment Conclusion

**Recommendation for Rust SDK:**
- ✅ **Public API**: Use `Project` struct and `*_project()` methods
- ✅ **Internal mapping**: Map to `/sessions` REST endpoints
- ✅ **Field names**: Follow Python SDK (keep `session_feedback_stats`)
- ✅ **Rationale**: Consistency with Python SDK and user expectations

This matches Python SDK's design: user-facing "projects" terminology with internal "sessions" implementation.

## 6. Recommendation

**Decision**: **Go**

### Rationale

1. **Clear API Contract**: Python SDK provides comprehensive reference
2. **Proven Patterns**: Similar CRUD operations already implemented in langstar
3. **User Value**: Projects are fundamental to organizing traces/runs
4. **Medium Complexity**: Manageable within standard 8-phase process

### Proposed Implementation Order

**Phase 1**: SDK foundation
- Create `sdk/src/projects.rs` with Project struct and basic list/get methods
- Implement pagination

**Phase 2**: CLI read-only commands
- `langstar project list`
- `langstar project get`

**Phase 3**: CLI write commands
- `langstar project create`
- `langstar project update`
- `langstar project delete`

**Phase 4**: Advanced filters and metadata handling

### Technical Risks (Low)

1. **Sessions vs Projects terminology**: Mitigated by following Python SDK precedent
2. **Pagination complexity**: Mitigated by existing pagination patterns in runs.rs
3. **Metadata JSON handling**: Mitigated by serde ecosystem

### Next Steps

1. **Create milestone**: `ls-projects`
2. **Create Phase 0 epic issue**: Follow standard 8-phase template
3. **Begin Phase 1**: OpenAPI spec analysis and Rust type generation

## References

- LangSmith API Overview: `reference/api-specs/LANGSMITH_API_OVERVIEW.md`
- Python SDK Client: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:3375-3780`
- Python SDK Schemas: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py:729-788`
- **Python Experiment**: `reference/experiments/574-ls-projects/` - Empirical API testing
- Existing Langstar Datasets: `sdk/src/datasets.rs` (similar CRUD pattern)
- Existing Langstar Runs: `sdk/src/runs.rs` (pagination pattern)
- GitHub Issue: #574
