# Research Report: LangSmith Runs Query Implementation Precedent

**Issue**: #298 - ls-runs-query milestone
**Sub-Issue**: #299 - Research langsmith-sdk runs query precedent
**Date**: 2025-11-26
**Status**: Complete

---

## Executive Summary

This report analyzes the LangSmith Python SDK's `list_runs` implementation to establish recommendations for implementing `langstar runs query` in Rust. The Python SDK provides a well-designed reference with cursor-based pagination, flexible filtering, and field selection capabilities.

**Key Recommendations:**
1. Use POST `/runs/query` endpoint (not GET)
2. Implement cursor-based pagination with streaming iterator
3. Support the LangSmith filter query language (function-style syntax)
4. Provide both raw filter strings and convenience flags
5. Follow existing `LangchainClient` patterns in `sdk/src/client.rs`

---

## 1. Python SDK Analysis

### 1.1 `list_runs` Method Signature

**Location**: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:2754`

```python
def list_runs(
    self,
    *,
    project_id: Optional[Union[ID_TYPE, Sequence[ID_TYPE]]] = None,
    project_name: Optional[Union[str, Sequence[str]]] = None,
    run_type: Optional[str] = None,
    trace_id: Optional[ID_TYPE] = None,
    reference_example_id: Optional[ID_TYPE] = None,
    query: Optional[str] = None,           # Natural language (experimental)
    filter: Optional[str] = None,          # Filter query language
    trace_filter: Optional[str] = None,    # Filter for ROOT run in trace
    tree_filter: Optional[str] = None,     # Filter for OTHER runs in trace
    is_root: Optional[bool] = None,
    parent_run_id: Optional[ID_TYPE] = None,
    start_time: Optional[datetime.datetime] = None,
    error: Optional[bool] = None,
    run_ids: Optional[Sequence[ID_TYPE]] = None,
    select: Optional[Sequence[str]] = None,
    limit: Optional[int] = None,
    **kwargs: Any,
) -> Iterator[ls_schemas.Run]:
```

### 1.2 API Endpoint

**Endpoint**: `POST /runs/query`
**Request Body** (JSON):
```json
{
  "session": ["<project-id-1>", "<project-id-2>"],
  "run_type": "llm",
  "filter": "eq(status, \"failed\")",
  "trace_filter": "...",
  "tree_filter": "...",
  "is_root": true,
  "parent_run": "<uuid>",
  "start_time": "2025-01-01T00:00:00Z",
  "error": true,
  "id": ["<run-id-1>", "<run-id-2>"],
  "trace": "<trace-id>",
  "select": ["id", "name", "status", "inputs", "outputs"]
}
```

**Key Insight**: Uses `session` (not `project`) for project IDs in the request body.

### 1.3 Cursor-Based Pagination

**Location**: `client.py:1139`

```python
def _get_cursor_paginated_list(
    self,
    path: str,
    *,
    body: Optional[dict] = None,
    request_method: Literal["GET", "POST"] = "POST",
    data_key: str = "runs",
) -> Iterator[dict]:
    params_ = body.copy() if body else {}
    while True:
        response = self.request_with_retries(request_method, path, ...)
        response_body = response.json()
        if not response_body or not response_body.get(data_key):
            break
        yield from response_body[data_key]
        cursors = response_body.get("cursors")
        if not cursors or not cursors.get("next"):
            break
        params_["cursor"] = cursors["next"]
```

**Response Shape**:
```json
{
  "runs": [...],
  "cursors": {
    "next": "<cursor-token>"
  }
}
```

---

## 2. Filter Query Language

### 2.1 Syntax Overview

The filter uses a **function-style syntax** (NOT SQL-like):

| Operator | Usage | Example |
|----------|-------|---------|
| `eq` | Equals | `eq(status, "failed")` |
| `neq` | Not equals | `neq(error, null)` |
| `gt` | Greater than | `gt(latency, 10)` |
| `gte` | Greater or equal | `gte(start_time, "2025-01-01T00:00:00Z")` |
| `lt` | Less than | `lt(total_tokens, 1000)` |
| `lte` | Less or equal | `lte(latency, 5)` |
| `has` | Array contains | `has(tags, "production")` |
| `search` | Substring match | `search(name, "agent")` |
| `and` | Logical AND | `and(eq(status, "failed"), gt(latency, 10))` |
| `or` | Logical OR | `or(has(tags, "experimental"), has(tags, "beta"))` |

### 2.2 Filterable Fields

From the SDK docs and Run schema:

| Field | Type | Notes |
|-------|------|-------|
| `id` | UUID | Run ID |
| `name` | string | Run name |
| `run_type` | string | "llm", "chain", "tool", "retriever" |
| `status` | string | "success", "error", "pending" |
| `start_time` | datetime | ISO 8601 format |
| `end_time` | datetime | ISO 8601 format |
| `latency` | float | Duration in seconds |
| `error` | string/null | Error message if failed |
| `total_tokens` | int | Total token count |
| `prompt_tokens` | int | Input token count |
| `completion_tokens` | int | Output token count |
| `tags` | array[string] | User-defined tags |
| `metadata.*` | any | Custom metadata fields |
| `feedback_key` | string | Feedback key name |
| `feedback_score` | float | Feedback score value |

### 2.3 Complex Filter Examples

From the Python SDK docstrings:

```python
# Failed runs with high latency and token usage
filter='and(eq(run_type, "chain"), gt(latency, 10), gt(total_tokens, 5000))'

# Runs with specific feedback on root trace
filter='eq(name, "extractor")'
trace_filter='and(eq(feedback_key, "user_score"), eq(feedback_score, 1))'

# Runs after timestamp with error OR bad feedback
filter='and(gt(start_time, "2023-07-15T12:34:56Z"), or(neq(error, null), and(eq(feedback_key, "Correctness"), eq(feedback_score, 0.0))))'

# Runs with experimental or beta tags and slow latency
filter='and(or(has(tags, "experimental"), has(tags, "beta")), gt(latency, 2))'
```

---

## 3. Run Schema

### 3.1 Core Fields

**Location**: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py:333`

```python
class RunBase(BaseModel):
    id: UUID
    name: str
    start_time: datetime
    run_type: str  # "tool", "chain", "llm", "retriever", "embedding", "prompt", "parser"
    end_time: Optional[datetime] = None
    extra: Optional[dict] = None  # Contains metadata
    error: Optional[str] = None
    events: Optional[list[dict]] = None
    inputs: dict
    outputs: Optional[dict] = None
    reference_example_id: Optional[UUID] = None
    parent_run_id: Optional[UUID] = None
    tags: Optional[list[str]] = None
    attachments: dict

class Run(RunBase):
    session_id: Optional[UUID] = None  # Project ID
    feedback_stats: Optional[dict[str, Any]] = None
    app_path: Optional[str] = None
    status: Optional[str] = None
    prompt_tokens: Optional[int] = None
    completion_tokens: Optional[int] = None
    total_tokens: Optional[int] = None
    first_token_time: Optional[datetime] = None
    total_cost: Optional[Decimal] = None
    prompt_cost: Optional[Decimal] = None
    completion_cost: Optional[Decimal] = None
    parent_run_ids: Optional[list[UUID]] = None
    trace_id: UUID
    dotted_order: str  # Trace ordering string
```

### 3.2 Default Select Fields

The SDK uses a default `select` list for performance:

```python
default_select = [
    "app_path", "completion_cost", "completion_tokens", "dotted_order",
    "end_time", "error", "events", "extra", "feedback_stats",
    "first_token_time", "id", "inputs", "name", "outputs",
    "parent_run_id", "parent_run_ids", "prompt_cost", "prompt_tokens",
    "reference_example_id", "run_type", "session_id", "start_time",
    "status", "tags", "total_cost", "total_tokens", "trace_id",
]
```

---

## 4. Recommendations for Rust Implementation

### 4.1 SDK Layer (`sdk/src/runs.rs`)

#### Struct Definitions

```rust
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub name: String,
    pub run_type: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub error: Option<String>,
    pub inputs: serde_json::Value,
    pub outputs: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub trace_id: Uuid,
    pub session_id: Option<Uuid>,  // Project ID
    pub parent_run_id: Option<Uuid>,
    pub parent_run_ids: Option<Vec<Uuid>>,
    pub total_tokens: Option<i64>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_cost: Option<Decimal>,
    pub feedback_stats: Option<serde_json::Value>,
    pub extra: Option<serde_json::Value>,
    // ... other fields as needed
}

#[derive(Debug, Default, Serialize)]
pub struct ListRunsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<Vec<Uuid>>,  // Project IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_root: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    // Limit handled client-side
}

#[derive(Debug, Deserialize)]
pub struct ListRunsResponse {
    pub runs: Vec<Run>,
    pub cursors: Option<Cursors>,
}

#[derive(Debug, Deserialize)]
pub struct Cursors {
    pub next: Option<String>,
}
```

#### Client Method

```rust
impl LangchainClient {
    /// List runs with cursor-based pagination
    pub async fn list_runs(
        &self,
        request: ListRunsRequest,
        limit: Option<usize>,
    ) -> Result<impl Stream<Item = Result<Run>>> {
        // Use async_stream or futures::stream
        // Yield runs, handle pagination internally
    }

    /// Single page fetch (for internal use)
    async fn fetch_runs_page(
        &self,
        request: &ListRunsRequest,
    ) -> Result<ListRunsResponse> {
        let response = self
            .langsmith_post("/runs/query")?
            .json(request)
            .send()
            .await?;
        // Handle errors, parse response
    }
}
```

### 4.2 Filter Builder Module (`sdk/src/filter_builder.rs`)

Provide type-safe filter construction:

```rust
pub enum FilterOp {
    Eq(String, FilterValue),
    Neq(String, FilterValue),
    Gt(String, FilterValue),
    Gte(String, FilterValue),
    Lt(String, FilterValue),
    Lte(String, FilterValue),
    Has(String, String),
    Search(String, String),
    And(Vec<FilterOp>),
    Or(Vec<FilterOp>),
}

pub enum FilterValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl FilterValue {
    /// Format the value for use in a filter expression
    pub fn format(&self) -> String {
        match self {
            FilterValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            FilterValue::Number(n) => n.to_string(),
            FilterValue::Bool(b) => b.to_string(),
            FilterValue::Null => "null".to_string(),
        }
    }
}

impl FilterOp {
    /// Convert the filter operation to a filter expression string
    pub fn to_filter_string(&self) -> String {
        match self {
            FilterOp::Eq(field, val) => format!("eq({}, {})", field, val.format()),
            FilterOp::And(ops) => {
                let parts: Vec<_> = ops.iter().map(|o| o.to_filter_string()).collect();
                format!("and({})", parts.join(", "))
            }
            // ... other operators
        }
    }
}

// Convenience constructors
pub fn eq(field: &str, value: impl Into<FilterValue>) -> FilterOp {
    FilterOp::Eq(field.to_string(), value.into())
}

pub fn failed_runs() -> FilterOp {
    eq("status", "error")
}

pub fn by_tag(tag: &str) -> FilterOp {
    FilterOp::Has("tags".to_string(), tag.to_string())
}
```

### 4.3 CLI Layer (`cli/src/commands/runs.rs`)

```rust
#[derive(Parser)]
pub struct RunsQueryArgs {
    /// Project name or ID
    #[arg(short, long)]
    project: Option<String>,

    /// Raw filter expression
    #[arg(short = 'q', long)]
    filter: Option<String>,

    /// Filter by tag (can be repeated)
    #[arg(long)]
    tag: Vec<String>,

    /// Filter by metadata key=value
    #[arg(long)]
    meta: Vec<String>,

    /// Filter by run type
    #[arg(long)]
    run_type: Option<String>,

    /// Only show root runs
    #[arg(long)]
    is_root: bool,

    /// Only show failed runs
    #[arg(long)]
    failed: bool,

    /// Filter runs since this time (ISO 8601 or relative like "24h")
    #[arg(long)]
    since: Option<String>,

    /// Tree filter expression
    #[arg(long)]
    tree_filter: Option<String>,

    /// Trace filter expression
    #[arg(long)]
    trace_filter: Option<String>,

    /// Maximum runs to return
    #[arg(long, default_value = "100")]
    limit: usize,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Select specific fields
    #[arg(long)]
    select: Vec<String>,
}
```

**Filter Building Logic**:

```rust
fn build_filter(args: &RunsQueryArgs) -> Result<Option<String>> {
    let mut parts = Vec::new();

    // Add convenience flags as filter parts
    if args.failed {
        parts.push("eq(status, \"error\")".to_string());
    }
    for tag in &args.tag {
        parts.push(format!("has(tags, \"{}\")", escape_string(tag)));
    }
    for meta in &args.meta {
        if let Some((key, value)) = meta.split_once('=') {
            parts.push(format!("eq(metadata.{}, \"{}\")", key, escape_string(value)));
        }
    }
    if let Some(since) = &args.since {
        let timestamp = parse_time_spec(since)?;
        parts.push(format!("gte(start_time, \"{}\")", timestamp.to_rfc3339()));
    }

    // Append raw filter if provided
    if let Some(raw) = &args.filter {
        parts.push(raw.clone());
    }

    if parts.is_empty() {
        Ok(None)
    } else if parts.len() == 1 {
        Ok(Some(parts.remove(0)))
    } else {
        Ok(Some(format!("and({})", parts.join(", "))))
    }
}
```

### 4.4 Testing Strategy

1. **Unit Tests** (`sdk/src/runs.rs`):
   - Filter builder produces correct query strings
   - Request serialization matches expected JSON
   - Run deserialization handles all fields

2. **Integration Tests** (with httpmock):
   - Mock `/runs/query` endpoint
   - Test pagination across multiple pages
   - Test error handling for API failures

3. **CLI Tests** (with assert_cmd):
   - Flag parsing works correctly
   - Convenience flags translate to filters
   - JSON output is valid
   - Table output is formatted correctly

---

## 5. Implementation Phases

### Phase 1: SDK Foundation (MVP)
- [ ] Add `Run` and related structs to `sdk/src/runs.rs`
- [ ] Implement `list_runs` with basic filtering
- [ ] Implement cursor-based pagination
- [ ] Add unit tests with httpmock

### Phase 2: Filter Builder
- [ ] Create `sdk/src/filter_builder.rs` module
- [ ] Implement type-safe filter construction
- [ ] Add pre-built common filters
- [ ] Unit test filter string generation

### Phase 3: CLI Integration
- [ ] Add `runs` subcommand group to CLI
- [ ] Implement `runs query` command
- [ ] Add convenience flags (--tag, --failed, --since)
- [ ] Implement JSON and table output formats
- [ ] Add CLI tests

### Phase 4: Advanced Features
- [ ] Support `trace_filter` and `tree_filter`
- [ ] Add `--select` field filtering
- [ ] Implement `--output <file>` export
- [ ] Add `runs get <id>` for single run lookup

---

## 6. Open Questions

1. **Project name resolution**: Should we resolve project names to IDs client-side (extra API call) or let the API handle it?
   - **Recommendation**: Let API handle it if supported, otherwise resolve client-side with caching.

2. **Streaming vs collecting**: Should `list_runs` return `Stream<Item=Run>` or `Vec<Run>`?
   - **Recommendation**: Return `Stream` for memory efficiency, provide convenience method to collect.

3. **Default select fields**: Should we use the same defaults as Python SDK?
   - **Recommendation**: Yes, for consistency and performance.

4. **Error field names**: The Python SDK uses `session` for project IDs - verify this matches current API.
   - **Action**: Test against live API before implementing.

---

## 7. References

- [LangSmith SDK Python client.py](../repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py) - Line 2754
- [LangSmith SDK schemas.py](../repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py) - Run schema
- [LangSmith Trace Query Syntax](https://docs.langchain.com/langsmith/trace-query-syntax) - Filter language docs
- [Existing langstar client.rs](../../sdk/src/client.rs) - Patterns to follow

---

## Appendix: Filter Query Grammar (Informal)

```
filter     ::= operation
operation  ::= comparator | logical
comparator ::= ("eq" | "neq" | "gt" | "gte" | "lt" | "lte") "(" field "," value ")"
             | "has" "(" field "," string ")"
             | "search" "(" field "," string ")"
logical    ::= ("and" | "or") "(" operation ("," operation)* ")"
field      ::= identifier ("." identifier)*
value      ::= string | number | "null" | "true" | "false"
string     ::= '"' chars '"'
```
