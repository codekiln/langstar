# Sessions/Projects OpenAPI Validation Report

**Issue**: #592 (Phase 3 of #586)
**Date**: 2025-12-05
**Status**: Complete

## Executive Summary

**Validation Result**: Spec largely matches scout findings with minor discrepancies documented.

The OpenAPI specification for sessions (projects) endpoints confirms the scout findings from #574. The API maps "sessions" internally to "projects" in user-facing SDKs. All expected CRUD operations are available with well-documented schemas.

## 1. Endpoints Validation

### OpenAPI Spec vs Scout Findings

| Endpoint | Method | OpenAPI | Scout | Match |
|----------|--------|---------|-------|-------|
| `/api/v1/sessions` | GET | `read_tracer_sessions_api_v1_sessions_get` | List all sessions | ✅ |
| `/api/v1/sessions` | POST | `create_tracer_session_api_v1_sessions_post` | Create new session | ✅ |
| `/api/v1/sessions` | DELETE | `delete_tracer_sessions_api_v1_sessions_delete` | Batch delete | ⚠️ Not in scout |
| `/api/v1/sessions/{session_id}` | GET | `read_tracer_session_api_v1_sessions__session_id__get` | Get single session | ✅ |
| `/api/v1/sessions/{session_id}` | PATCH | `update_tracer_session_api_v1_sessions__session_id__patch` | Update session | ✅ |
| `/api/v1/sessions/{session_id}` | DELETE | `delete_tracer_session_api_v1_sessions__session_id__delete` | Delete single session | ✅ |

**Note**: The batch DELETE at `/api/v1/sessions` accepts `session_ids` as a required query parameter (array of UUIDs).

### Authentication

All endpoints support the same authentication methods:
- API Key
- Tenant ID
- Bearer Auth

### Additional Endpoints (Not in SDK Scope)

The OpenAPI spec includes additional session-related endpoints not in the initial SDK scope:
- `/sessions/{session_id}/dashboard` - Dashboard data
- `/sessions/{session_id}/insights/*` - Insights and clustering
- `/sessions/{session_id}/metadata` - Metadata operations
- `/sessions/{session_id}/views/*` - View management

These can be added in future phases as needed.

## 2. Schema Validation

### TracerSession (Response)

**OpenAPI Schema** (from `sessions-schemas.json`):

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | string (uuid) | ✅ Yes | Session/Project ID |
| `tenant_id` | string (uuid) | ✅ Yes | Organization ID |
| `name` | string | No | Project name |
| `description` | string \| null | No | Project description |
| `start_time` | string (date-time) | No | When project started |
| `end_time` | string (date-time) \| null | No | When project ended (closed) |
| `extra` | object \| null | No | Metadata/tags container |
| `default_dataset_id` | string (uuid) \| null | No | Default dataset reference |
| `reference_dataset_id` | string (uuid) \| null | No | Reference dataset |
| `trace_tier` | TraceTier \| null | No | `longlived` or `shortlived` |
| `run_count` | integer \| null | No | Total runs (stats) |
| `latency_p50` | number \| null | No | 50th percentile latency |
| `latency_p99` | number \| null | No | 99th percentile latency |
| `first_token_p50` | number \| null | No | Time to first token p50 |
| `first_token_p99` | number \| null | No | Time to first token p99 |
| `total_tokens` | integer \| null | No | Total token count |
| `prompt_tokens` | integer \| null | No | Input token count |
| `completion_tokens` | integer \| null | No | Output token count |
| `total_cost` | string \| null | No | Total cost (decimal string) |
| `prompt_cost` | string \| null | No | Prompt cost |
| `completion_cost` | string \| null | No | Completion cost |
| `last_run_start_time` | string (date-time) \| null | No | Last run timestamp |
| `last_run_start_time_live` | string (date-time) \| null | No | Live last run timestamp |
| `feedback_stats` | object \| null | No | Aggregated feedback stats |
| `session_feedback_stats` | object \| null | No | Session-level feedback |
| `run_facets` | array \| null | No | Run facet data |
| `error_rate` | number \| null | No | Error percentage |
| `streaming_rate` | number \| null | No | Streaming run percentage |
| `test_run_number` | integer \| null | No | Test run sequence number |

**Comparison with Python SDK TracerSession**: ✅ Fields align with Python SDK schema (`reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py:729-788`).

### TracerSessionCreate (Request)

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | No | Project name |
| `description` | string \| null | No | Optional description |
| `start_time` | string (date-time) | No | Auto-set if not provided |
| `end_time` | string (date-time) \| null | No | For closed projects |
| `extra` | object \| null | No | Metadata/tags |
| `default_dataset_id` | string (uuid) \| null | No | Default dataset |
| `reference_dataset_id` | string (uuid) \| null | No | Reference dataset |
| `trace_tier` | TraceTier \| null | No | Trace retention tier |
| `id` | string (uuid) \| null | No | Client-specified ID |

**Key Finding**: No fields are strictly required. The API will auto-generate ID and use defaults.

### TracerSessionUpdate (Request)

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string \| null | No | Update name |
| `description` | string \| null | No | Update description |
| `default_dataset_id` | string (uuid) \| null | No | Update default dataset |
| `end_time` | string (date-time) \| null | No | Close the project |
| `extra` | object \| null | No | Update metadata |
| `trace_tier` | TraceTier \| null | No | Update trace tier |

**Key Finding**: All fields optional - supports partial updates via PATCH.

### TracerSessionWithoutVirtualFields (Create/Update Response)

Simplified response returned from POST and PATCH operations:

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | string (uuid) | ✅ Yes | Session ID |
| `tenant_id` | string (uuid) | ✅ Yes | Organization ID |
| `name` | string | No | Project name |
| `description` | string \| null | No | Description |
| `start_time` | string (date-time) | No | Start time |
| `end_time` | string (date-time) \| null | No | End time |
| `extra` | object \| null | No | Metadata |
| `default_dataset_id` | string (uuid) \| null | No | Default dataset |
| `reference_dataset_id` | string (uuid) \| null | No | Reference dataset |
| `trace_tier` | TraceTier \| null | No | Trace tier |
| `last_run_start_time_live` | string (date-time) \| null | No | Last run time |

**Key Finding**: Excludes computed/stats fields like `run_count`, `latency_p*`, `*_cost`, etc.

### SessionSortableColumns (Enum)

```
name, start_time, last_run_start_time, latency_p50, latency_p99, error_rate, feedback, runs_count
```

### TraceTier (Enum)

```
longlived, shortlived
```

## 3. Query Parameters (List Endpoint)

| Parameter | Type | Default | Notes |
|-----------|------|---------|-------|
| `reference_free` | boolean \| null | - | Filter by no reference dataset |
| `reference_dataset` | array[uuid] \| null | - | Filter by reference datasets |
| `id` | array[uuid] \| null | - | Filter by specific IDs |
| `name` | string \| null | - | Exact name match |
| `name_contains` | string \| null | - | Partial name match |
| `dataset_version` | string \| null | - | Filter by dataset version |
| `sort_by` | SessionSortableColumns | `start_time` | Sort field |
| `sort_by_desc` | boolean | `true` | Descending order |
| `metadata` | string \| null | - | Metadata filter (JSON) |
| `sort_by_feedback_key` | string \| null | - | Sort by feedback key |
| `offset` | integer | `0` | Pagination offset |
| `limit` | integer | `100` | Max 100 |
| `tag_value_id` | array[uuid] \| null | - | Filter by tags |
| `facets` | boolean | `false` | Include facet data |
| `filter` | string \| null | - | Generic filter string |
| `include_stats` | boolean | `false` | Include statistics |
| `use_approx_stats` | boolean | `false` | Use approximate stats |
| `stats_start_time` | string (date-time) \| null | - | Stats time window |

## 4. Discrepancies

### Clarifications from OpenAPI Spec

1. **Schema Naming**: Scout referenced `TracerSessionResult` (Python SDK naming), but OpenAPI uses:
   - `TracerSession` - Full response with stats
   - `TracerSessionWithoutVirtualFields` - Simplified response from mutations

   **Rust SDK Recommendation**: Use `Project` for user-facing type, internally map to these.

2. **Batch Delete**: OpenAPI spec reveals a batch delete endpoint at `DELETE /sessions` that wasn't emphasized in scout. This accepts `session_ids` query param.

3. **Create Request**: Scout implied `name` was required for creation, but OpenAPI shows no required fields. The API will create a project without a name (though this may be unusual in practice).

4. **Response Types Differ**: POST/PATCH return `TracerSessionWithoutVirtualFields` (no stats), while GET returns `TracerSession` (with stats).

### Minor Observations

1. **No Dedicated GET-by-name**: To lookup by name, use GET `/sessions?name={exact_name}&limit=1`.

2. **Cost as String**: Token costs are returned as strings (not numbers) to preserve decimal precision.

3. **Stats are Optional**: Most statistical fields only populated when `include_stats=true`.

## 5. SDK Implementation Notes

### Rust Type Mapping

| OpenAPI Type | Rust Type |
|--------------|-----------|
| `string (uuid)` | `uuid::Uuid` |
| `string (date-time)` | `chrono::DateTime<Utc>` |
| `string \| null` | `Option<String>` |
| `integer \| null` | `Option<i64>` |
| `number \| null` | `Option<f64>` |
| `object (additionalProperties)` | `serde_json::Value` |

### Recommended Struct Definitions

```rust
/// Project/TracerSession response from list and get operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub tenant_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub extra: Option<Value>,
    pub default_dataset_id: Option<Uuid>,
    pub reference_dataset_id: Option<Uuid>,
    pub trace_tier: Option<TraceTier>,
    // Stats fields (populated when include_stats=true)
    pub run_count: Option<i64>,
    pub latency_p50: Option<f64>,
    pub latency_p99: Option<f64>,
    pub first_token_p50: Option<f64>,
    pub first_token_p99: Option<f64>,
    pub total_tokens: Option<i64>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_cost: Option<String>,
    pub prompt_cost: Option<String>,
    pub completion_cost: Option<String>,
    pub last_run_start_time: Option<DateTime<Utc>>,
    pub last_run_start_time_live: Option<DateTime<Utc>>,
    pub feedback_stats: Option<Value>,
    pub session_feedback_stats: Option<Value>,
    pub run_facets: Option<Vec<Value>>,
    pub error_rate: Option<f64>,
    pub streaming_rate: Option<f64>,
    pub test_run_number: Option<i64>,
}

/// TraceTier enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceTier {
    Longlived,
    Shortlived,
}

/// Create request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectCreate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_dataset_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_tier: Option<TraceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset_id: Option<Uuid>,
}

/// Update request (all fields optional)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_tier: Option<TraceTier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset_id: Option<Uuid>,
}

/// List query parameters
#[derive(Debug, Clone, Default)]
pub struct ProjectListParams {
    pub name: Option<String>,
    pub name_contains: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub include_stats: Option<bool>,
    pub sort_by: Option<SessionSortableColumn>,
    pub sort_by_desc: Option<bool>,
    // ... additional params as needed
}

/// Sortable columns enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionSortableColumn {
    Name,
    StartTime,
    LastRunStartTime,
    LatencyP50,
    LatencyP99,
    ErrorRate,
    Feedback,
    RunsCount,
}
```

## 6. Extracted Fragments

The following OpenAPI fragments were extracted and added to `reference/api-specs/langsmith/`:

| File | Size | Content |
|------|------|---------|
| `sessions-endpoints.json` | 15K | Core CRUD endpoint definitions |
| `sessions-schemas.json` | 12K | TracerSession, Create, Update, related types |

**jq extraction commands** added to `reference/api-specs/langsmith/FRAGMENTS.md`.

## 7. Conclusion

The sessions/projects API is well-documented and ready for SDK implementation:

1. **Complete CRUD support**: All operations available including batch delete
2. **Consistent terminology mapping**: API "sessions" → SDK "projects"
3. **Rich filtering**: 15+ query parameters for list operation
4. **Optional stats**: Can include/exclude statistics for performance
5. **Partial updates**: PATCH supports updating individual fields

### Recommended Next Steps

1. Proceed with SDK Phase (Phase 4: SDK Types)
2. Use extracted schemas as reference for type definitions
3. Implement both single and batch delete operations
4. Consider `include_stats` as optional parameter for list/get

## References

- Scout Report: `docs/research/574-ls-projects-scout.md`
- Design Decisions: Issue #586 (Phase 2)
- Python SDK: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:3375-3780`
- OpenAPI Spec: `reference/openapi/langchain/langsmith/openapi.json`
- Extracted Fragments: `reference/api-specs/langsmith/sessions-*.json`
