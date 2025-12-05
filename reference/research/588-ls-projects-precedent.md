# Python langsmith-sdk Projects Precedent Research

**Issue**: #588
**Parent**: #586 (ls-projects milestone)
**Date**: 2025-12-05
**Status**: Complete

## Executive Summary

This document provides detailed analysis of the Python langsmith-sdk project methods to guide Rust SDK implementation. All methods operate on the `/sessions` API endpoint but expose "project" terminology to users.

**Key Findings**:
- 6 core methods: `list_projects`, `read_project`, `create_project`, `update_project`, `delete_project`, `has_project`
- Pagination: Offset-based, 100 items per page
- Error handling: Rich exception hierarchy with specific error types
- Parameter validation: Decorator-based mutual exclusivity checks

## 1. Method Inventory

### 1.1 list_projects()

**Location**: `client.py:3704-3778`

**Signature**:
```python
def list_projects(
    self,
    project_ids: Optional[list[ID_TYPE]] = None,
    name: Optional[str] = None,
    name_contains: Optional[str] = None,
    reference_dataset_id: Optional[ID_TYPE] = None,
    reference_dataset_name: Optional[str] = None,
    reference_free: Optional[bool] = None,
    include_stats: Optional[bool] = None,
    dataset_version: Optional[str] = None,
    limit: Optional[int] = None,
    metadata: Optional[dict[str, Any]] = None,
) -> Iterator[ls_schemas.TracerSessionResult]
```

**API Endpoint**: `GET /sessions`

**Query Parameters**:
- `limit`: Max 100 per page (hardcoded: `min(limit, 100) if limit else 100`)
- `offset`: Managed by `_get_paginated_list()` helper
- `id`: Project IDs filter (list)
- `name`: Exact name match
- `name_contains`: Substring search
- `reference_dataset`: Dataset UUID filter
- `reference_free`: Boolean filter for projects without datasets
- `include_stats`: Include aggregate statistics
- `dataset_version`: Dataset version filter
- `metadata`: JSON-encoded metadata filter

**Return Type**: Iterator yielding `TracerSessionResult` objects

**Special Logic**:
- Mutual exclusivity: `reference_dataset_id` XOR `reference_dataset_name`
- If `reference_dataset_name` provided, resolves to ID via `read_dataset()`
- Metadata dict is JSON-encoded before sending: `json.dumps(metadata)`
- Yields items until limit reached or no more results

**Pagination Implementation**: Uses `_get_paginated_list()` helper (see section 2)

### 1.2 read_project()

**Location**: `client.py:3531-3570`

**Signature**:
```python
@ls_utils.xor_args(("project_id", "project_name"))
def read_project(
    self,
    *,
    project_id: Optional[str] = None,
    project_name: Optional[str] = None,
    include_stats: bool = False,
) -> ls_schemas.TracerSessionResult
```

**API Endpoint**:
- By ID: `GET /sessions/{uuid}`
- By name: `GET /sessions?name={name}&limit=1`

**Query Parameters**:
- `include_stats`: Boolean (default: False)
- `limit`: Always 1 when searching by name

**Return Type**: Single `TracerSessionResult` object

**Error Handling**:
- Raises `LangSmithNotFoundError` if project not found
- Uses `@xor_args` decorator to enforce exactly one of `project_id` or `project_name`

**Special Logic**:
- Name-based lookup returns list, extracts first item
- Empty list triggers `LangSmithNotFoundError` with message: `"Project {project_name} not found"`
- ID-based lookup returns single object directly

### 1.3 create_project()

**Location**: `client.py:3408-3453`

**Signature**:
```python
def create_project(
    self,
    project_name: str,
    *,
    description: Optional[str] = None,
    metadata: Optional[dict] = None,
    upsert: bool = False,
    project_extra: Optional[dict] = None,
    reference_dataset_id: Optional[ID_TYPE] = None,
) -> ls_schemas.TracerSession
```

**API Endpoint**: `POST /sessions?upsert={bool}`

**Request Body**:
```python
{
    "name": str,              # Required
    "extra": dict,            # Contains metadata if provided
    "description": str,       # Optional
    "id": str,                # Auto-generated UUID
    "reference_dataset_id": UUID  # Optional
}
```

**Return Type**: `TracerSession` (base schema, not Result variant)

**Special Logic**:
- Auto-generates UUID for new project: `str(uuid.uuid4())`
- Metadata merged into `extra` field: `{"metadata": metadata, ...extra}`
- `upsert` parameter passed as query param, not in body
- Returns basic `TracerSession`, not `TracerSessionResult` (no stats)

**Error Handling**:
- Uses `raise_for_status_with_text()` to include response body in exceptions

### 1.4 update_project()

**Location**: `client.py:3455-3502`

**Signature**:
```python
def update_project(
    self,
    project_id: ID_TYPE,
    *,
    name: Optional[str] = None,
    description: Optional[str] = None,
    metadata: Optional[dict] = None,
    project_extra: Optional[dict] = None,
    end_time: Optional[datetime.datetime] = None,
) -> ls_schemas.TracerSession
```

**API Endpoint**: `PATCH /sessions/{uuid}`

**Request Body**:
```python
{
    "name": Optional[str],
    "extra": Optional[dict],
    "description": Optional[str],
    "end_time": Optional[str]  # ISO8601 format
}
```

**Return Type**: `TracerSession`

**Special Logic**:
- Name change only allowed if project has `end_time` (closed project)
- `end_time` converted to ISO8601: `end_time.isoformat()`
- Metadata merged into `extra` like in create
- Uses `_as_uuid()` helper to validate project_id

**Important Constraint** (from docstring line 3471-3472):
> "The new name to give the project. This is only valid if the project has been assigned an end_time, meaning it has been completed/closed."

### 1.5 delete_project()

**Location**: `client.py:3780-3806`

**Signature**:
```python
@ls_utils.xor_args(("project_name", "project_id"))
def delete_project(
    self,
    *,
    project_name: Optional[str] = None,
    project_id: Optional[str] = None
) -> None
```

**API Endpoint**: `DELETE /sessions/{uuid}`

**Return Type**: `None`

**Special Logic**:
- If `project_name` provided, resolves to ID via `read_project()`
- Uses `@xor_args` decorator for mutual exclusivity
- No response body expected (successful delete returns nothing)

**Error Handling**:
- `read_project()` will raise `LangSmithNotFoundError` if name doesn't exist
- DELETE endpoint uses `raise_for_status_with_text()` for error responses

### 1.6 has_project()

**Location**: `client.py:3572-3590`

**Signature**:
```python
def has_project(
    self,
    project_name: str,
    *,
    project_id: Optional[str] = None
) -> bool
```

**Implementation**: Convenience wrapper around `read_project()`

```python
try:
    self.read_project(project_name=project_name)
except ls_utils.LangSmithNotFoundError:
    return False
return True
```

**Note**: `project_id` parameter is accepted but unused in the implementation

## 2. Pagination Strategy

**Implementation**: `_get_paginated_list()` helper (`client.py:1136-1166`)

### 2.1 Pagination Method

**Type**: Offset-based pagination (not cursor-based as initially expected)

**Mechanism**:
```python
def _get_paginated_list(
    self, path: str, *, params: Optional[dict] = None
) -> Iterator[dict]:
    params_ = params.copy() if params else {}
    offset = params_.get("offset", 0)
    params_["limit"] = params_.get("limit", 100)
    while True:
        params_["offset"] = offset
        response = self.request_with_retries("GET", path, params=params_)
        items = response.json()
        if not items:
            break
        yield from items
        if len(items) < params_["limit"]:
            break
        offset += len(items)
```

### 2.2 Key Characteristics

1. **Default page size**: 100 items
2. **Max page size**: 100 (enforced in `list_projects()`)
3. **Termination conditions**:
   - Empty response (`if not items`)
   - Partial page (`if len(items) < params_["limit"]`)
4. **Offset calculation**: `offset += len(items)` (not `offset += limit`)
   - Handles cases where API returns fewer items than requested

### 2.3 Rust Implementation Implications

**Recommended approach**:
```rust
pub struct ProjectListParams {
    pub limit: Option<u32>,  // Max 100, default 100
    pub offset: Option<u32>, // For pagination continuation
    // ... other filters
}

pub async fn list_projects(
    &self,
    params: &ProjectListParams
) -> Result<Vec<Project>> {
    let mut all_projects = Vec::new();
    let mut offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100).min(100);

    loop {
        let mut query = vec![
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];
        // Add other params...

        let response: Vec<Project> = self.get("/sessions", &query).await?;

        if response.is_empty() {
            break;
        }

        let count = response.len();
        all_projects.extend(response);

        if count < limit as usize {
            break;
        }

        offset += count as u32;
    }

    Ok(all_projects)
}
```

## 3. Error Handling Patterns

### 3.1 Exception Hierarchy

**Location**: `utils.py:41-78`

**Base exception**:
```python
class LangSmithError(Exception):
    """An error occurred while communicating with the LangSmith API."""
```

**Specific exceptions** (all inherit from `LangSmithError`):

| Exception | Status Code | Usage |
|-----------|-------------|-------|
| `LangSmithAPIError` | 500+ | Internal server errors |
| `LangSmithUserError` | 400-499 | Client-side errors (invalid params) |
| `LangSmithNotFoundError` | 404 | Resource not found |
| `LangSmithConflictError` | 409 | Resource already exists |
| `LangSmithAuthError` | 401, 403 | Authentication/authorization failures |
| `LangSmithRateLimitError` | 429 | Rate limit exceeded |
| `LangSmithConnectionError` | - | Network connection failures |
| `LangSmithRequestTimeout` | 408 | Request timeout |

### 3.2 Error Raising Pattern

**Helper function**: `raise_for_status_with_text()` (`utils.py:154-167`)

```python
def raise_for_status_with_text(response: Union[requests.Response, httpx.Response]) -> None:
    try:
        response.raise_for_status()
    except requests.HTTPError as e:
        raise requests.HTTPError(str(e), response.text) from e
    except httpx.HTTPStatusError as e:
        raise httpx.HTTPStatusError(
            f"{str(e)}: {response.text}",
            request=response.request,
            response=response,
        ) from e
```

**Usage pattern in project methods**:
```python
response = self.request_with_retries("POST", endpoint, ...)
ls_utils.raise_for_status_with_text(response)  # Throws on HTTP error
return ls_schemas.TracerSession(**response.json(), ...)
```

### 3.3 Retry Logic

**Location**: `request_with_retries()` (`client.py:1070-1134`)

**Retry configuration**:
- Default attempts: 3 (configurable via `retry_config`)
- Rate limit backoff: Exponential (`retry_after * 2^idx + random`)
- Rate limit header: Uses `retry-after` from response, defaults to 30s
- General backoff: `2^idx + (random * 0.5)`

**Retryable exceptions**:
- `LangSmithRateLimitError` (429 status)
- `LangSmithConnectionError`
- `requests.ConnectionError`
- `requests.Timeout`

### 3.4 Rust Error Handling Recommendations

```rust
// Error type hierarchy
#[derive(Debug, thiserror::Error)]
pub enum LangSmithError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid parameters: {0}")]
    UserError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

// Helper for status code to error mapping
fn handle_response_status(status: StatusCode, body: &str) -> Result<()> {
    match status {
        StatusCode::OK | StatusCode::CREATED => Ok(()),
        StatusCode::NOT_FOUND => Err(LangSmithError::NotFound(body.to_string())),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN =>
            Err(LangSmithError::AuthError(body.to_string())),
        StatusCode::TOO_MANY_REQUESTS => Err(LangSmithError::RateLimit),
        StatusCode::BAD_REQUEST => Err(LangSmithError::UserError(body.to_string())),
        _ if status.is_server_error() => Err(LangSmithError::ApiError(body.to_string())),
        _ => Err(LangSmithError::HttpError(/* ... */)),
    }
}
```

## 4. Schema Types

### 4.1 TracerSession (Base)

**Location**: `schemas.py:729-781`

**Fields**:
```python
class TracerSession(BaseModel):
    id: UUID
    start_time: datetime  # Default: now(UTC)
    end_time: Optional[datetime] = None
    description: Optional[str] = None
    name: Optional[str] = None
    extra: Optional[dict[str, Any]] = None  # Contains metadata, tags
    tenant_id: UUID
    reference_dataset_id: Optional[UUID] = None
```

**Properties**:
- `url`: Computed property returning UI URL: `f"{host}/o/{tenant_id}/projects/p/{id}"`
- `metadata`: Extracts `extra["metadata"]` if present, else `{}`
- `tags`: Extracts `extra["tags"]` if present, else `[]`

**Important comment** (line 732):
> "Sessions are also referred to as 'Projects' in the UI."

### 4.2 TracerSessionResult (Extended)

**Location**: `schemas.py:783-821`

**Additional fields** (all optional):
```python
class TracerSessionResult(TracerSession):
    run_count: Optional[int]
    latency_p50: Optional[timedelta]
    latency_p99: Optional[timedelta]
    total_tokens: Optional[int]
    prompt_tokens: Optional[int]
    completion_tokens: Optional[int]
    last_run_start_time: Optional[datetime]
    feedback_stats: Optional[dict[str, Any]]
    session_feedback_stats: Optional[dict[str, Any]]  # NOTE: "session" terminology
    run_facets: Optional[list[dict[str, Any]]]
    total_cost: Optional[Decimal]
    prompt_cost: Optional[Decimal]
    completion_cost: Optional[Decimal]
    first_token_p50: Optional[timedelta]
    first_token_p99: Optional[timedelta]
    error_rate: Optional[float]
```

**Usage**:
- Returned by `list_projects()` and `read_project()`
- Includes aggregate statistics when `include_stats=True`

**Rust mapping**:
```rust
// Base project type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
    pub tenant_id: Uuid,
    pub reference_dataset_id: Option<Uuid>,
}

// Extended project with statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResult {
    #[serde(flatten)]
    pub project: Project,

    pub run_count: Option<u32>,
    pub latency_p50: Option<Duration>,
    pub latency_p99: Option<Duration>,
    pub total_tokens: Option<u64>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub last_run_start_time: Option<DateTime<Utc>>,
    pub feedback_stats: Option<HashMap<String, serde_json::Value>>,
    pub session_feedback_stats: Option<HashMap<String, serde_json::Value>>,
    // ... other stats fields
}
```

## 5. Parameter Validation

### 5.1 XOR Decorator Pattern

**Location**: `utils.py:128-151`

**Implementation**:
```python
def xor_args(*arg_groups: tuple[str, ...]) -> Callable:
    """Validate specified keyword args are mutually exclusive."""
    def decorator(func: Callable) -> Callable:
        @functools.wraps(func)
        def wrapper(*args: Any, **kwargs: Any) -> Any:
            counts = [
                sum(1 for arg in arg_group if kwargs.get(arg) is not None)
                for arg_group in arg_groups
            ]
            invalid_groups = [i for i, count in enumerate(counts) if count != 1]
            if invalid_groups:
                invalid_group_names = [", ".join(arg_groups[i]) for i in invalid_groups]
                raise ValueError(
                    "Exactly one argument in each of the following"
                    " groups must be defined:"
                    f" {', '.join(invalid_group_names)}"
                )
            return func(*args, **kwargs)
        return wrapper
    return decorator
```

**Usage examples**:
```python
@ls_utils.xor_args(("project_id", "project_name"))
def read_project(self, *, project_id=None, project_name=None): ...

@ls_utils.xor_args(("project_name", "project_id"))
def delete_project(self, *, project_name=None, project_id=None): ...
```

### 5.2 UUID Validation

**Helper**: `_as_uuid()` (`client.py:357-364`)

```python
def _as_uuid(value: ID_TYPE, var: Optional[str] = None) -> uuid.UUID:
    try:
        return uuid.UUID(value) if not isinstance(value, uuid.UUID) else value
    except ValueError as e:
        var = var or "value"
        raise ls_utils.LangSmithUserError(
            f"{var} must be a valid UUID or UUID string. Got {value}"
        ) from e
```

**Rust equivalent**:
```rust
fn as_uuid(value: &str, param_name: &str) -> Result<Uuid, LangSmithError> {
    Uuid::parse_str(value)
        .map_err(|e| LangSmithError::UserError(
            format!("{} must be a valid UUID. Got: {}", param_name, value)
        ))
}
```

## 6. Rust Implementation Recommendations

### 6.1 Module Structure

```
sdk/src/projects.rs
├── Types
│   ├── Project (base schema)
│   ├── ProjectResult (with stats)
│   ├── CreateProjectRequest
│   ├── UpdateProjectRequest
│   └── ListProjectsParams
├── Methods
│   ├── list_projects()
│   ├── get_project()      // Note: "get" not "read" for Rust conventions
│   ├── create_project()
│   ├── update_project()
│   ├── delete_project()
│   └── has_project()
└── Helpers
    ├── paginate_projects()
    └── resolve_project_id()
```

### 6.2 API Design Patterns

**Prefer builder pattern for complex params**:
```rust
let projects = client
    .list_projects()
    .name_contains("test")
    .include_stats(true)
    .limit(50)
    .execute()
    .await?;
```

**Or use struct params**:
```rust
let params = ListProjectsParams {
    name_contains: Some("test".to_string()),
    include_stats: Some(true),
    limit: Some(50),
    ..Default::default()
};
let projects = client.list_projects(&params).await?;
```

**Enforce mutual exclusivity at type level**:
```rust
pub enum ProjectIdentifier {
    Id(Uuid),
    Name(String),
}

pub async fn get_project(
    &self,
    identifier: ProjectIdentifier,
    include_stats: bool,
) -> Result<ProjectResult>
```

### 6.3 Pagination Handling

**Option 1: Return Vec (auto-paginate)**:
```rust
pub async fn list_projects(&self, params: &ListProjectsParams) -> Result<Vec<ProjectResult>>
```

**Option 2: Return Stream (lazy pagination)**:
```rust
pub fn list_projects(&self, params: ListProjectsParams) -> impl Stream<Item = Result<ProjectResult>>
```

**Option 3: Manual pagination**:
```rust
pub async fn list_projects_page(
    &self,
    params: &ListProjectsParams
) -> Result<(Vec<ProjectResult>, Option<u32>)>  // (items, next_offset)
```

**Recommendation**: Start with Option 1 (auto-paginate) for simplicity, add Option 2 later if needed.

### 6.4 Error Handling

**Use thiserror for ergonomics**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LangSmithError {
    #[error("project not found: {0}")]
    ProjectNotFound(String),

    #[error("invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
}
```

**Implement From for common conversions**:
```rust
impl From<uuid::Error> for LangSmithError {
    fn from(e: uuid::Error) -> Self {
        LangSmithError::InvalidUuid(e.to_string())
    }
}
```

### 6.5 Metadata Handling

**Use serde_json::Value for flexibility**:
```rust
use serde_json::Value;

#[derive(Serialize)]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

// Helper to build extra field with metadata
pub fn with_metadata(metadata: HashMap<String, Value>) -> Value {
    json!({ "metadata": metadata })
}
```

### 6.6 Testing Strategy

**Unit tests** (no network):
- UUID validation
- Parameter validation
- Request serialization
- Response deserialization

**Integration tests** (real API):
- CRUD operations against test project
- Pagination with >100 items
- Error handling (404, 400, etc.)
- Filter combinations

**Test fixture pattern**:
```rust
#[tokio::test]
async fn test_project_crud() {
    let client = test_client();

    // Create
    let project = client.create_project("test-project", None).await.unwrap();

    // Read
    let fetched = client.get_project(ProjectIdentifier::Id(project.id)).await.unwrap();
    assert_eq!(fetched.name, Some("test-project".to_string()));

    // Update
    let updated = client.update_project(project.id, Some("new-name"), None).await.unwrap();

    // Delete
    client.delete_project(ProjectIdentifier::Id(project.id)).await.unwrap();

    // Verify deleted
    let result = client.get_project(ProjectIdentifier::Id(project.id)).await;
    assert!(matches!(result, Err(LangSmithError::ProjectNotFound(_))));
}
```

## 7. Notable Absences

### 7.1 No rename_project() Method

The scout report mentioned `rename_project()` as a potential method, but **it does not exist** in the Python SDK. Renaming is done via `update_project()` with the constraint that the project must be closed (`end_time` set).

### 7.2 list_shared_projects()

**Location**: `client.py:3375-3406`

Not included in core CRUD operations. This is a separate method for accessing shared projects via a dataset share token. Recommend implementing in later phase if needed.

## 8. References

- Python SDK client.py: Lines 3375-3806 (project methods)
- Python SDK schemas.py: Lines 729-821 (TracerSession types)
- Python SDK utils.py: Lines 41-78 (error classes), 128-151 (xor_args), 154-167 (raise_for_status)
- Scout report: `docs/research/574-ls-projects-scout.md`
- Experiments: `reference/experiments/574-ls-projects/`
- API endpoint: `https://api.smith.langchain.com/sessions`

## 9. Next Steps

**Phase 2: OpenAPI Schema Analysis** (#587)
- Extract TracerSession schema from OpenAPI spec
- Validate field types match Python SDK
- Identify any additional fields or constraints

**Phase 3: Rust SDK Implementation**
- Implement `Project` and `ProjectResult` types
- Implement 6 core methods
- Add pagination helper
- Add error types

**Phase 4: CLI Implementation**
- `langstar project list` with table output
- `langstar project get` with JSON/YAML output
- `langstar project create/update/delete`

**Phase 5: Integration Testing**
- Test against real LangSmith API
- Validate all CRUD operations
- Test pagination with large project counts
- Test error scenarios
