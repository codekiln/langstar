# LangSmith Dataset API Research Report

**Issue**: #348 - Research langsmith-sdk dataset management precedent
**Epic**: #346 - ls-datasets - langstar dataset CLI support
**Date**: 2025-11-27

## Executive Summary

This document captures research findings on the LangSmith SDK dataset management capabilities to inform the design of langstar's Rust-based dataset CLI. The research covers API endpoints, SDK types, and implementation patterns from the official Python SDK.

## Data Sources

All API endpoint information in this document is sourced from the LangSmith OpenAPI specification:

| Source | Location | Description |
|--------|----------|-------------|
| LangSmith OpenAPI Spec | `reference/api-specs/langsmith-openapi.json` | Local copy of the LangSmith API specification |
| Remote URL | `https://api.smith.langchain.com/openapi.json` | Official LangSmith OpenAPI endpoint |

**Verification**: All jq queries in this document can be run against the local OpenAPI spec file:
```bash
cd /workspace && jq '<query>' reference/api-specs/langsmith-openapi.json
```

## 1. API Endpoints

### 1.1 Dataset CRUD Operations

| Method | Endpoint | Description | Citation (jq query) |
|--------|----------|-------------|---------------------|
| GET | `/api/v1/datasets` | List datasets with filtering | `.paths["/api/v1/datasets"].get.summary` |
| POST | `/api/v1/datasets` | Create a new dataset | `.paths["/api/v1/datasets"].post.summary` |
| GET | `/api/v1/datasets/{dataset_id}` | Get dataset by ID | `.paths["/api/v1/datasets/{dataset_id}"].get.summary` |
| PATCH | `/api/v1/datasets/{dataset_id}` | Update dataset | `.paths["/api/v1/datasets/{dataset_id}"].patch.summary` |
| DELETE | `/api/v1/datasets/{dataset_id}` | Delete dataset | `.paths["/api/v1/datasets/{dataset_id}"].delete.summary` |

#### List Datasets Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | uuid[] | Filter by specific IDs |
| `data_type` | string | Filter by data type (kv, llm, chat) |
| `name` | string | Exact name match |
| `name_contains` | string | Partial name match |
| `metadata` | object | Metadata filter |
| `offset` | int | Pagination offset |
| `limit` | int | Pagination limit |
| `sort_by` | string | Sort field |
| `sort_by_desc` | bool | Sort direction |
| `tag_value_id` | uuid[] | Filter by tags |

### 1.2 Example CRUD Operations

| Method | Endpoint | Description | Citation (jq query) |
|--------|----------|-------------|---------------------|
| GET | `/api/v1/examples` | List examples with filtering | `.paths["/api/v1/examples"].get.summary` |
| POST | `/api/v1/examples` | Create single example | `.paths["/api/v1/examples"].post.summary` |
| POST | `/api/v1/examples/bulk` | Create multiple examples | `.paths["/api/v1/examples/bulk"].post.summary` |
| GET | `/api/v1/examples/{example_id}` | Get example by ID | `.paths["/api/v1/examples/{example_id}"].get.summary` |
| PATCH | `/api/v1/examples/{example_id}` | Update example | `.paths["/api/v1/examples/{example_id}"].patch.summary` |
| DELETE | `/api/v1/examples/{example_id}` | Delete single example | `.paths["/api/v1/examples/{example_id}"].delete.summary` |
| DELETE | `/api/v1/examples` | Bulk delete examples | `.paths["/api/v1/examples"].delete.summary` |
| PATCH | `/api/v1/examples/bulk` | Bulk update examples | `.paths["/api/v1/examples/bulk"].patch.summary` |
| GET | `/api/v1/examples/count` | Count examples | `.paths["/api/v1/examples/count"].get.summary` |

#### List Examples Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `dataset` | uuid | Required - parent dataset ID |
| `id` | uuid[] | Filter by specific IDs |
| `as_of` | datetime | Point-in-time query |
| `metadata` | object | Metadata filter |
| `full_text_contains` | string | Full-text search |
| `splits` | string[] | Filter by splits |
| `filter` | string | Advanced filter expression |
| `offset` | int | Pagination offset |
| `limit` | int | Pagination limit |
| `order` | string | Sort field |
| `descending` | bool | Sort direction |
| `select` | string[] | Field selection |
| `random_seed` | int | Random sampling seed |

### 1.3 Import/Export Operations

| Method | Endpoint | Description | Citation (jq query) |
|--------|----------|-------------|---------------------|
| POST | `/api/v1/datasets/upload` | Upload CSV to create dataset | `.paths["/api/v1/datasets/upload"].post.summary` |
| POST | `/api/v1/examples/upload/{dataset_id}` | Upload CSV to existing dataset | `.paths["/api/v1/examples/upload/{dataset_id}"].post.summary` |
| GET | `/api/v1/datasets/{dataset_id}/csv` | Export as CSV | `.paths["/api/v1/datasets/{dataset_id}/csv"].get.summary` |
| GET | `/api/v1/datasets/{dataset_id}/jsonl` | Export as JSONL | `.paths["/api/v1/datasets/{dataset_id}/jsonl"].get.summary` |
| GET | `/api/v1/datasets/{dataset_id}/openai` | Export for OpenAI Evals | `.paths["/api/v1/datasets/{dataset_id}/openai"].get.summary` |
| GET | `/api/v1/datasets/{dataset_id}/openai_ft` | Export for OpenAI fine-tuning | `.paths["/api/v1/datasets/{dataset_id}/openai_ft"].get.summary` |

### 1.4 Versioning Operations

| Method | Endpoint | Description | Citation (jq query) |
|--------|----------|-------------|---------------------|
| GET | `/api/v1/datasets/{dataset_id}/versions` | List versions | `.paths["/api/v1/datasets/{dataset_id}/versions"].get.summary` |
| GET | `/api/v1/datasets/{dataset_id}/version` | Get specific version | `.paths["/api/v1/datasets/{dataset_id}/version"].get.summary` |
| GET | `/api/v1/datasets/{dataset_id}/versions/diff` | Compare versions | `.paths["/api/v1/datasets/{dataset_id}/versions/diff"].get.summary` |
| PUT | `/api/v1/datasets/{dataset_id}/tags` | Tag a version | `.paths["/api/v1/datasets/{dataset_id}/tags"].put.summary` |

### 1.5 Sharing Operations

| Method | Endpoint | Description | Citation (jq query) |
|--------|----------|-------------|---------------------|
| GET | `/api/v1/datasets/{dataset_id}/share` | Get share info | `.paths["/api/v1/datasets/{dataset_id}/share"].get.summary` |
| PUT | `/api/v1/datasets/{dataset_id}/share` | Create/update share | `.paths["/api/v1/datasets/{dataset_id}/share"].put.summary` |

---

## 2. SDK Schema Types

### 2.1 DataType Enum

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Kv,    // Key-value pairs
    Llm,   // LLM completions
    Chat,  // Chat conversations
}
```

### 2.2 Dataset Schema

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// TODO: DatasetTransformation schema requires further research.
// This is a PRELIMINARY placeholder based on field naming patterns.
// Actual structure should be verified against live API responses before implementation.
/// Dataset transformation type - represents transformations applied to datasets.
/// **WARNING**: This is a placeholder definition. The exact schema must be verified
/// against actual LangSmith API responses before use in production code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetTransformation {
    pub path: String,
    pub transformation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub id: Uuid,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_session_start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformations: Option<Vec<DatasetTransformation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetCreate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
```

### 2.3 Example Schema

```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a value that can be either a single string or a vector of strings.
/// This is needed because the LangSmith API accepts both formats for the `split` field.
/// Requires custom serde deserialization to handle both `"train"` and `["train", "test"]` inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrVec {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Example {
    pub id: Uuid,
    pub dataset_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<HashMap<String, AttachmentInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleCreate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub dataset_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<StringOrVec>,  // Single string or list of strings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_source_run_io: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_source_run_attachments: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<StringOrVec>,
}
```

### 2.4 Version Schema

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub as_of: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetDiffInfo {
    pub examples_modified: Vec<Uuid>,
    pub examples_added: Vec<Uuid>,
    pub examples_removed: Vec<Uuid>,
}
```

### 2.5 Share Schema

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetShareSchema {
    pub dataset_id: Uuid,
    pub share_token: Uuid,
    pub url: String,
}
```

### 2.6 Attachment Types

```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub presigned_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Represents attachment data - either raw bytes or a file path reference.
#[derive(Debug, Clone)]
pub enum AttachmentData {
    /// In-memory binary data
    Bytes(Vec<u8>),
    /// Reference to a file on disk
    File(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub mime_type: String,
    pub data: AttachmentData,
}
```

---

## 3. Python SDK Client Methods

### 3.1 Dataset Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `create_dataset` | name, description?, data_type?, metadata? | Dataset | Create new dataset |
| `read_dataset` | dataset_name?, dataset_id? | Dataset | Get dataset by name or ID |
| `list_datasets` | data_type?, dataset_name?, dataset_name_contains?, metadata?, limit?, offset? | Iterator[Dataset] | List datasets with filters |
| `delete_dataset` | dataset_name?, dataset_id? | None | Delete dataset |
| `has_dataset` | dataset_name?, dataset_id? | bool | Check if dataset exists |
| `clone_public_dataset` | token, dataset_name? | Dataset | Clone shared dataset |
| `share_dataset` | dataset_id?, dataset_name? | DatasetShareSchema | Make dataset shareable |
| `unshare_dataset` | dataset_id?, dataset_name? | None | Remove share access |

### 3.2 Example Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `create_example` | inputs, outputs?, dataset_name?, dataset_id?, created_at?, example_id?, metadata?, split? | Example | Create single example |
| `create_examples` | inputs: list, outputs?: list, dataset_name?, dataset_id?, ... | list[Example] | Create multiple examples |
| `read_example` | example_id | Example | Get example by ID |
| `list_examples` | dataset_name?, dataset_id?, as_of?, splits?, metadata?, limit?, offset?, filter? | Iterator[Example] | List examples with filters |
| `update_example` | example_id, inputs?, outputs?, metadata?, split? | dict | Update example |
| `update_examples` | updates: list[ExampleUpdate] | dict | Bulk update examples |
| `delete_example` | example_id | None | Delete single example |
| `similar_examples` | inputs, dataset_id?, dataset_name?, limit? | list[Example] | Find similar examples |
| `create_example_from_run` | run, dataset_name?, dataset_id?, ... | Example | Create from traced run |

### 3.3 Import/Export Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `upload_csv` | csv_file, input_keys, output_keys, name?, description?, data_type? | Dataset | Import CSV |
| `upload_dataframe` | df, name, input_keys, output_keys, description?, data_type? | Dataset | Import DataFrame |

### 3.4 Version Methods

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `read_dataset_version` | dataset_id?, dataset_name?, as_of?, tag? | DatasetVersion | Get version info |
| `list_dataset_versions` | dataset_id?, dataset_name?, limit?, offset? | Iterator[DatasetVersion] | List all versions |
| `diff_dataset_versions` | dataset_id?, dataset_name?, from_version, to_version | DatasetDiffInfo | Compare versions |
| `update_dataset_tag` | dataset_id?, dataset_name?, as_of, tag | None | Tag a version |

---

## 4. Pagination Patterns

### 4.1 Offset-Based Pagination

The LangSmith API uses offset/limit pagination:

```python
# Python SDK pattern
for example in client.list_examples(dataset_id=dataset_id, limit=100):
    process(example)
```

### 4.2 Recommended Rust Implementation

```rust
pub struct PaginationParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: Option<i64>,
    pub offset: i64,
    pub limit: i64,
}

// Iterator-based pagination
impl LangSmithClient {
    pub fn list_examples_iter(
        &self,
        dataset_id: Uuid,
        params: ListExamplesParams,
    ) -> impl Stream<Item = Result<Example, DatasetError>> {
        // Async stream implementation
    }
}
```

---

## 5. Error Handling Patterns

### 5.1 Common Error Responses

| Status | Meaning | Handling |
|--------|---------|----------|
| 400 | Bad Request | Validation error - check parameters |
| 401 | Unauthorized | Invalid or missing API key |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Dataset/example doesn't exist |
| 409 | Conflict | Duplicate name or concurrent update |
| 422 | Unprocessable | Schema validation failed |
| 429 | Rate Limited | Implement backoff/retry |
| 500 | Server Error | Retry with exponential backoff |

### 5.2 Recommended Error Types

```rust
pub enum DatasetError {
    NotFound { id: String },
    AlreadyExists { name: String },
    ValidationError { message: String, details: Option<Value> },
    PermissionDenied { message: String },
    RateLimited { retry_after: Option<Duration> },
    ApiError { status: u16, message: String },
}
```

---

## 6. Implementation Recommendations

### 6.1 SDK Layer (langstar-sdk crate)

**Types to implement:**
1. `Dataset`, `DatasetCreate`, `DatasetUpdate`
2. `Example`, `ExampleCreate`, `ExampleUpdate`
3. `DataType` enum
4. `DatasetVersion`, `DatasetDiffInfo`
5. `PaginatedResponse<T>`
6. `DatasetError` enum

**Client methods:**
1. `create_dataset(&self, request: DatasetCreate) -> Result<Dataset>`
2. `get_dataset(&self, id: Uuid) -> Result<Dataset>`
3. `get_dataset_by_name(&self, name: &str) -> Result<Dataset>`
4. `list_datasets(&self, params: ListDatasetsParams) -> Result<PaginatedResponse<Dataset>>`
5. `delete_dataset(&self, id: Uuid) -> Result<()>`
6. `create_example(&self, request: ExampleCreate) -> Result<Example>`
7. `create_examples(&self, requests: Vec<ExampleCreate>) -> Result<Vec<Example>>`
8. `list_examples(&self, dataset_id: Uuid, params: ListExamplesParams) -> Result<PaginatedResponse<Example>>`
9. `update_example(&self, id: Uuid, request: ExampleUpdate) -> Result<Example>`
10. `update_examples(&self, updates: Vec<(Uuid, ExampleUpdate)>) -> Result<Vec<Example>>`
11. `delete_example(&self, id: Uuid) -> Result<()>`
12. `delete_examples(&self, ids: Vec<Uuid>) -> Result<()>`

### 6.2 CLI Layer (langstar CLI)

**Recommended command structure:**
```
langstar dataset create --name <name> --type <type> [--description <desc>] [--metadata k=v]
langstar dataset list [--filter <name>] [--type <type>] [--limit <n>] [--format table|json]
langstar dataset get <id|name> [--format table|json]
langstar dataset delete <id|name> [--yes]
langstar dataset import <id|name> --file <path> [--format jsonl|csv]
langstar dataset export <id|name> --format <jsonl|csv> --out <path>
langstar dataset examples <id|name> [--filter <expr>] [--limit <n>]
```

### 6.3 Import/Export Format Support

**Priority formats:**
1. JSONL (primary - native to LangSmith)
2. CSV (secondary - good for spreadsheet users)

**JSONL format:**
```jsonl
{"inputs": {"question": "What is 2+2?"}, "outputs": {"answer": "4"}, "metadata": {"source": "math"}}
{"inputs": {"question": "What is 3+3?"}, "outputs": {"answer": "6"}, "metadata": {"source": "math"}}
```

### 6.4 Consistency with Existing Langstar Patterns

Based on the annotation queue implementation, the dataset implementation should:
1. Use the same HTTP client and authentication pattern
2. Follow the same module structure (`types.rs`, `client.rs`)
3. Use consistent error handling with `thiserror`
4. Use `serde` with `#[serde(rename_all = "camelCase")]`
5. Support both sync and async operations where needed

---

## 7. Key Findings Summary

1. **API is comprehensive** - LangSmith provides full CRUD for datasets and examples with good filtering/pagination support.

2. **Versioning is point-in-time** - Versions are snapshots referenced by timestamp, not explicit version numbers.

3. **Export formats available** - Native JSONL and CSV export endpoints eliminate need for client-side conversion.

4. **Bulk operations supported** - API supports batch create/update/delete for examples.

5. **Attachments supported** - Examples can include file attachments with presigned URLs for upload/download.

6. **Splits supported** - Examples can be tagged with splits (train/test/dev) for ML workflows.

7. **No annotation queue integration in API** - Linking datasets to annotation queues will need custom logic.

---

## 8. Open Questions

1. **Rate limits**: What are the specific rate limits for dataset/example operations?
2. **Max batch size**: What's the maximum number of examples in bulk operations?
3. **File size limits**: What are the limits for CSV uploads and attachments?
4. **Version retention**: How long are dataset versions retained?

---

## 9. References

- [LangSmith API OpenAPI Spec](https://api.smith.langchain.com/openapi.json)
- [LangSmith SDK Python Client](https://langsmith-sdk.readthedocs.io/en/latest/)
- [LangSmith Documentation](https://docs.smith.langchain.com/)
- [langsmith-sdk GitHub](https://github.com/langchain-ai/langsmith-sdk)
