# Implementation Plan: LangSmith Runs Query CLI

**Issue**: #298 - ls-runs-query milestone
**Research Phase**: #299 (completed) - Python SDK analysis
**Validation Phase**: #303 (completed) - OpenAPI spec validation
**Date**: 2025-11-26

## Executive Summary

This plan provides the implementation roadmap for `langstar runs query`, enabling users to query and filter LangSmith runs/traces from the CLI. The implementation is based on:

1. **Python SDK precedent analysis** (#299) - Established patterns from `langsmith-sdk`
2. **OpenAPI spec validation** (#303) - Verified API contracts against official spec
3. **Existing Langstar patterns** - Follows conventions from `sdk/src/client.rs` and `cli/src/commands/`

**Key Decisions:**

- Use `POST /api/v1/runs/query` endpoint (validated against OpenAPI)
- Implement cursor-based pagination with streaming support
- Support LangSmith filter query language (function-style syntax)
- Provide both raw `--filter` and convenience flags (`--tag`, `--meta`, etc.)

---

## Research Sources

1. **Research report** (#299): `/workspace/reference/research/298-ls-runs-query-precedent.md`
2. **OpenAPI validation** (#303): `/workspace/reference/research/298-openapi-validation.md`
3. **OpenAPI artifacts**:
   - `/workspace/reference/openapi/langchain/langsmith/openapi.json` - Full spec (635KB)
   - `/workspace/reference/api-specs/langsmith/runs-query-request-schema.json` - Request schema
   - `/workspace/reference/api-specs/langsmith/runs-query-response-schema.json` - Response schema
   - `/workspace/reference/api-specs/langsmith/run-schema.json` - Run object schema (54 fields)

---

## Implementation Phases

### Phase 1: SDK Foundation (MVP)

**Goal**: Implement `LangchainClient::query_runs()` method with core functionality.

**Deliverables**:

1. `sdk/src/runs.rs` - Run types and query implementation
2. Tests using httpmock for mocked API responses

**Tasks**:

#### 1.1 Create Run Types

**File**: `sdk/src/runs.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Run type enum matching OpenAPI spec
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    Tool,
    Chain,
    Llm,
    Retriever,
    Embedding,
    Prompt,
    Parser,
}

/// Run schema based on OpenAPI spec (required fields are non-optional)
/// Reference: /workspace/reference/api-specs/langsmith/run-schema.json
#[derive(Debug, Clone, Deserialize)]
pub struct Run {
    // Required fields (per OpenAPI spec)
    pub id: Uuid,
    pub name: String,
    pub run_type: RunType,
    pub trace_id: Uuid,
    pub dotted_order: String,
    pub status: String,
    pub session_id: Uuid,
    pub app_path: String,

    // Optional timing fields
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub first_token_time: Option<DateTime<Utc>>,

    // Optional content fields
    pub inputs: Option<Value>,
    pub outputs: Option<Value>,
    pub error: Option<String>,
    pub extra: Option<Value>,
    pub events: Option<Value>,
    pub serialized: Option<Value>,

    // Optional hierarchy fields
    pub parent_run_id: Option<Uuid>,
    pub parent_run_ids: Option<Vec<Uuid>>,
    pub child_run_ids: Option<Vec<Uuid>>,
    pub direct_child_run_ids: Option<Vec<Uuid>>,

    // Token/cost fields (with defaults per OpenAPI)
    #[serde(default)]
    pub total_tokens: i64,
    #[serde(default)]
    pub prompt_tokens: i64,
    #[serde(default)]
    pub completion_tokens: i64,
    pub total_cost: Option<String>,
    pub prompt_cost: Option<String>,
    pub completion_cost: Option<String>,

    // Metadata fields
    pub tags: Option<Vec<String>>,
    pub feedback_stats: Option<Value>,
    pub reference_example_id: Option<Uuid>,
    pub reference_dataset_id: Option<Uuid>,

    // Additional fields (full API completeness)
    pub execution_order: Option<i32>,
    pub in_dataset: Option<bool>,
    pub share_token: Option<String>,
    pub trace_tier: Option<String>,
    pub trace_upgrade: Option<bool>,
    pub thread_id: Option<Uuid>,
    pub ttl_seconds: Option<i64>,

    // Preview fields
    pub inputs_preview: Option<String>,
    pub outputs_preview: Option<String>,
}
```

#### 1.2 Create Query Request/Response Types

```rust
/// Request body for POST /api/v1/runs/query
/// Reference: /workspace/reference/api-specs/langsmith/runs-query-request-schema.json
#[derive(Debug, Clone, Serialize, Default)]
pub struct QueryRunsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<Vec<Uuid>>,  // Project IDs

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<Uuid>>,  // Run IDs

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_type: Option<RunType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_filter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_filter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_root: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,  // max 100 per OpenAPI

    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,  // "asc" or "desc" (default: "desc")
}

/// Response from POST /api/v1/runs/query
/// Reference: /workspace/reference/api-specs/langsmith/runs-query-response-schema.json
#[derive(Debug, Clone, Deserialize)]
pub struct QueryRunsResponse {
    pub runs: Vec<Run>,
    pub cursors: Cursors,
    pub search_cursors: Option<Value>,
    pub parsed_query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cursors {
    pub next: Option<String>,
    pub prev: Option<String>,
}
```

#### 1.3 Implement Client Method

Add to `sdk/src/client.rs`:

```rust
impl LangchainClient {
    /// Query runs from LangSmith with filtering and pagination
    ///
    /// Uses POST /api/v1/runs/query endpoint.
    /// Supports the LangSmith filter query language.
    pub async fn query_runs(
        &self,
        request: QueryRunsRequest,
    ) -> Result<QueryRunsResponse> {
        let url = format!("{}/api/v1/runs/query", self.langsmith_base_url);

        let response = self
            .langsmith_request_builder(&url)
            .post(&url)
            .json(&request)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Query runs with automatic pagination
    ///
    /// Returns an iterator that automatically fetches next pages.
    pub fn query_runs_paginated(
        &self,
        mut request: QueryRunsRequest,
        limit: Option<usize>,
    ) -> impl Stream<Item = Result<Run>> + '_ {
        async_stream::try_stream! {
            let mut total_yielded = 0usize;
            let page_limit = limit.unwrap_or(usize::MAX);

            loop {
                let response = self.query_runs(request.clone()).await?;

                for run in response.runs {
                    if total_yielded >= page_limit {
                        return;
                    }
                    total_yielded += 1;
                    yield run;
                }

                match response.cursors.next {
                    Some(next) if total_yielded < page_limit => {
                        request.cursor = Some(next);
                    }
                    _ => break,
                }
            }
        }
    }
}
```

#### 1.4 Export from lib.rs

Add to `sdk/src/lib.rs`:

```rust
pub mod runs;
pub use runs::{Run, RunType, QueryRunsRequest, QueryRunsResponse, Cursors};
```

---

### Phase 2: CLI Command

**Goal**: Implement `langstar runs query` CLI command.

**Deliverables**:

1. `cli/src/commands/runs.rs` - Runs subcommand
2. Integration tests

**Tasks**:

#### 2.1 Create Runs Command

**File**: `cli/src/commands/runs.rs`

```rust
use clap::{Args, Subcommand};
use langstar_sdk::{QueryRunsRequest, RunType};
use uuid::Uuid;

#[derive(Debug, Subcommand)]
pub enum RunsCommand {
    /// Query runs with filtering
    Query(QueryArgs),
}

#[derive(Debug, Args)]
pub struct QueryArgs {
    /// Project name or ID to query runs from
    #[arg(short, long)]
    pub project: Option<String>,

    /// Raw filter expression (LangSmith filter query language)
    /// Example: 'eq(status, "failed")'
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Filter for root run in trace
    #[arg(long)]
    pub trace_filter: Option<String>,

    /// Filter for other runs in trace tree
    #[arg(long)]
    pub tree_filter: Option<String>,

    /// Only return root runs
    #[arg(long)]
    pub is_root: bool,

    /// Filter by tag (can be repeated)
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,

    /// Filter by metadata key=value (can be repeated)
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    pub metadata: Vec<String>,

    /// Filter by run type
    #[arg(long)]
    pub run_type: Option<String>,

    /// Filter by status
    #[arg(long)]
    pub status: Option<String>,

    /// Only show runs with errors
    #[arg(long)]
    pub errors_only: bool,

    /// Filter runs after this time (ISO 8601)
    #[arg(long)]
    pub since: Option<String>,

    /// Filter runs before this time (ISO 8601)
    #[arg(long)]
    pub until: Option<String>,

    /// Maximum number of runs to return
    #[arg(short, long, default_value = "100")]
    pub limit: usize,

    /// Output format
    #[arg(long, default_value = "table")]
    pub format: OutputFormat,

    /// Fields to select (comma-separated)
    #[arg(long)]
    pub select: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    JsonPretty,
}
```

#### 2.2 Implement Filter Builder

**File**: `cli/src/commands/runs/filter_builder.rs`

```rust
/// Build LangSmith filter expressions from CLI flags
pub struct FilterBuilder {
    conditions: Vec<String>,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self { conditions: vec![] }
    }

    /// Add tag filter: has(tags, "value")
    pub fn tag(mut self, tag: &str) -> Self {
        self.conditions.push(format!("has(tags, \"{}\")", escape(tag)));
        self
    }

    /// Add metadata filter: eq(metadata["key"], "value")
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.conditions.push(format!(
            "eq(metadata[\"{}\"], \"{}\")",
            escape(key),
            escape(value)
        ));
        self
    }

    /// Add status filter: eq(status, "value")
    pub fn status(mut self, status: &str) -> Self {
        self.conditions.push(format!("eq(status, \"{}\")", escape(status)));
        self
    }

    /// Add error filter: eq(error, true)
    pub fn has_error(mut self) -> Self {
        self.conditions.push("eq(error, true)".to_string());
        self
    }

    /// Add raw filter expression
    pub fn raw(mut self, filter: &str) -> Self {
        self.conditions.push(filter.to_string());
        self
    }

    /// Build final filter string
    pub fn build(self) -> Option<String> {
        if self.conditions.is_empty() {
            None
        } else {
            Some(self.conditions.join(" and "))
        }
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
```

#### 2.3 Add to CLI Main

Update `cli/src/commands/mod.rs`:

```rust
pub mod runs;
pub use runs::RunsCommand;
```

Update `cli/src/main.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Query and manage LangSmith runs
    Runs {
        #[command(subcommand)]
        command: RunsCommand,
    },
}
```

---

### Phase 3: Testing & Documentation

**Goal**: Comprehensive tests and documentation.

**Tasks**:

#### 3.1 SDK Unit Tests

**File**: `sdk/tests/runs_test.rs`

- Test `Run` deserialization from OpenAPI-compliant JSON
- Test `QueryRunsRequest` serialization
- Test pagination handling
- Test filter builder

#### 3.2 SDK Integration Tests (mocked)

**File**: `sdk/tests/runs_integration_test.rs`

- Mock `/api/v1/runs/query` endpoint
- Test cursor-based pagination
- Test error handling

#### 3.3 CLI Integration Tests

**File**: `cli/tests/runs_command_test.rs`

- Test CLI argument parsing
- Test filter building from convenience flags
- Test output formatting

#### 3.4 Documentation

- Update CLI help text
- Add examples to README
- Document filter query language syntax

---

## Phased Sub-Issues

Based on this plan, create the following GitHub sub-issues:

### 298.3-sdk-runs-types

**Title**: `298.3-sdk-runs-types Implement Run types and QueryRunsRequest in SDK`
**Scope**: Phase 1.1-1.2 (Run types, request/response types)
**Deliverable**: `sdk/src/runs.rs` with types only

### 298.4-sdk-runs-client

**Title**: `298.4-sdk-runs-client Implement query_runs client method`
**Scope**: Phase 1.3-1.4 (client method, pagination)
**Depends on**: 298.3
**Deliverable**: `query_runs()` and `query_runs_paginated()` methods

### 298.5-cli-runs-command

**Title**: `298.5-cli-runs-command Implement langstar runs query CLI command`
**Scope**: Phase 2 (CLI command, filter builder)
**Depends on**: 298.4
**Deliverable**: `langstar runs query` working command

### 298.6-runs-testing

**Title**: `298.6-runs-testing Add comprehensive tests for runs query`
**Scope**: Phase 3 (testing)
**Depends on**: 298.5
**Deliverable**: Unit, integration, and CLI tests

---

## Key Implementation Notes

### Filter Query Language

The filter language is **not documented in the OpenAPI spec**. Use the research report (#299) as the authoritative source:

**Supported Operators** (function-style syntax):

- `eq(field, value)` - Equals
- `neq(field, value)` - Not equals
- `gt(field, value)` - Greater than
- `gte(field, value)` - Greater than or equal
- `lt(field, value)` - Less than
- `lte(field, value)` - Less than or equal
- `has(array_field, value)` - Array contains
- `search(field, value)` - Full-text search
- `and(expr1, expr2)` - Logical AND
- `or(expr1, expr2)` - Logical OR

**Common Filterable Fields**:

- `status` - Run status ("success", "error", "pending")
- `error` - Boolean error flag
- `tags` - Array of tags
- `metadata` - Object with custom metadata
- `name` - Run name
- `run_type` - Run type enum

### Required vs Optional Fields

**Use OpenAPI spec requirements** (not Python SDK):

- Required: `id`, `name`, `run_type`, `trace_id`, `dotted_order`, `status`, `session_id`, `app_path`
- All other fields are optional with appropriate `Option<T>` wrappers

### Token Fields

Per OpenAPI spec, token fields have `default: 0`:

```rust
#[serde(default)]
pub total_tokens: i64,
```

### Pagination

- Max `limit` per request: 100 (per OpenAPI spec)
- Use cursor-based pagination for requests > 100
- Response includes `cursors.next` for next page

---

## Dependencies

### New Crate Dependencies (sdk/Cargo.toml)

```toml
[dependencies]
async-stream = "0.3"  # For streaming pagination
```

### Existing Dependencies (already available)

- `chrono` - DateTime handling
- `uuid` - UUID types
- `serde` / `serde_json` - Serialization
- `reqwest` - HTTP client
- `tokio` - Async runtime

---

## Risk Assessment

| Risk                               | Likelihood | Impact | Mitigation                           |
| ---------------------------------- | ---------- | ------ | ------------------------------------ |
| Filter syntax mismatch             | Low        | Medium | Test against live API before release |
| Pagination edge cases              | Medium     | Low    | Comprehensive integration tests      |
| Field type mismatches              | Low        | High   | Use OpenAPI spec as source of truth  |
| Performance with large result sets | Medium     | Medium | Implement streaming pagination       |

---

## Success Criteria

1. `langstar runs query --project <name> --filter 'eq(status, "error")'` returns filtered runs
2. Convenience flags (`--tag`, `--meta`) produce correct filter expressions
3. Pagination works for results > 100 runs
4. JSON and table output formats work correctly
5. Unit tests pass with > 80% coverage
6. Integration tests verify API contract compliance

---

## References

- [OpenAPI Validation Report](../reference/research/298-openapi-validation.md)
- [Python SDK Research Report](../reference/research/298-ls-runs-query-precedent.md)
- [LangSmith Trace Query Syntax](https://docs.langchain.com/langsmith/trace-query-syntax)
- [OpenAPI Spec Artifacts](../reference/api-specs/)
