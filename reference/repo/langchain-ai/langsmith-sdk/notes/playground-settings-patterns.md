# Playground Settings SDK Patterns Research

**Issue**: #471
**Parent**: #461
**Date**: 2025-11-30

## Executive Summary

This research documents SDK implementation patterns for the playground-settings API based on:

1. Analysis of Python SDK patterns for similar CRUD operations
2. Analysis of existing Langstar Rust SDK patterns (datasets, prompts)
3. OpenAPI spec from scout report #453

**Key Finding**: No Python SDK implementation exists for playground settings. Langstar will be the first SDK to implement this API, providing an opportunity to establish best practices.

## 1. Python SDK Analysis

### 1.1 Confirmation of No Implementation

As documented in scout report #453, the Python SDK (`langsmith-sdk`) does **not** provide methods for managing playground settings:

- No `list_playground_settings()` method
- No `create_playground_settings()` method
- No `update_playground_settings()` method
- No `delete_playground_settings()` method

### 1.2 General Python SDK Patterns (from WebFetch analysis)

**Method Naming Conventions:**

- Create: `create_*()` (e.g., `create_run()`, `create_dataset()`)
- Read single: `get_*()` (e.g., `get_dataset()`)
- Read multiple: `list_*()` or paginated helpers
- Update: `update_*()` (e.g., `update_run()`, `update_dataset()`)
- Delete: `delete_*()` (implied pattern)

**Pagination Patterns:**

- **Offset-based**: `_get_paginated_list(path, params)` - iterates until fewer items than limit
- **Cursor-based**: `_get_cursor_paginated_list(path, body, request_method, data_key)` - uses cursor tokens

**Error Handling:**

- Hierarchical custom exceptions: `LangSmithUserError`, `LangSmithConnectionError`, `LangSmithAPIError`
- HTTP-specific mappings: 401→AuthError, 404→NotFoundError, 429→RateLimitError
- Retry logic with exponential backoff

**Method Signature Patterns:**

- Required parameters: positional
- Optional parameters: keyword-only with `*,` separator
- Explicit type hints for returns
- Configuration via callable processors

## 2. Langstar Rust SDK Patterns

### 2.1 Dataset API Patterns

**File**: `sdk/src/client.rs:1037-1248`

#### Create Pattern

```rust
pub async fn create_dataset(
    &self,
    request: crate::datasets::DatasetCreate,
) -> Result<crate::datasets::Dataset> {
    let request_builder = self.langsmith_post("/api/v1/datasets")?.json(&request);
    self.execute(request_builder).await
}
```

**Characteristics:**

- Takes structured request type (e.g., `DatasetCreate`)
- Returns structured response type (e.g., `Dataset`)
- Uses `langsmith_post()` helper for authentication
- Single-line implementation delegating to `execute()`

#### List Pattern

```rust
pub async fn list_datasets(
    &self,
    params: crate::datasets::ListDatasetsParams,
) -> Result<Vec<crate::datasets::Dataset>> {
    let mut request = self.langsmith_get("/api/v1/datasets")?;

    // Add query parameters
    if let Some(ids) = &params.id {
        for id in ids {
            request = request.query(&[("id", id.to_string())]);
        }
    }
    if let Some(data_type) = &params.data_type {
        request = request.query(&[(
            "data_type",
            serde_json::to_string(data_type).unwrap().trim_matches('"'),
        )]);
    }
    // ... more optional parameters

    self.execute(request).await
}
```

**Characteristics:**

- Takes structured params type with all optional fields
- Returns `Vec<T>` of items
- Conditionally adds query parameters for non-None values
- Offset/limit pagination via query parameters
- Does NOT auto-paginate (returns single page)

#### Get (by ID) Pattern

```rust
pub async fn get_dataset(&self, dataset_id: uuid::Uuid) -> Result<crate::datasets::Dataset> {
    let path = format!("/api/v1/datasets/{}", dataset_id);
    let request = self.langsmith_get(&path)?;
    self.execute(request).await
}
```

**Characteristics:**

- Takes ID as parameter (UUID)
- Returns single item
- Simple path interpolation

#### Update Pattern

```rust
pub async fn update_dataset(
    &self,
    dataset_id: uuid::Uuid,
    request: crate::datasets::DatasetUpdate,
) -> Result<crate::datasets::Dataset> {
    let path = format!("/api/v1/datasets/{}", dataset_id);
    let request_builder = self.langsmith_patch(&path)?.json(&request);
    self.execute(request_builder).await
}
```

**Characteristics:**

- Takes ID + structured update request
- Returns updated item
- Uses `langsmith_patch()` for PATCH requests
- Update request type typically has all optional fields

#### Delete Pattern

```rust
pub async fn delete_dataset(&self, dataset_id: uuid::Uuid) -> Result<()> {
    let path = format!("/api/v1/datasets/{}", dataset_id);
    let request = self.langsmith_delete(&path)?;
    self.execute_status_only_request(request).await
}
```

**Characteristics:**

- Takes only ID parameter
- Returns `Result<()>` (no content on success)
- Uses `execute_status_only_request()` helper

### 2.2 Prompts API Patterns

**File**: `sdk/src/prompts.rs:288-339`

#### List Pattern (with client-side filtering)

```rust
pub async fn list(
    &self,
    limit: Option<u32>,
    offset: Option<u32>,
    visibility: Option<Visibility>,
) -> Result<Vec<Prompt>> {
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);
    let visibility = visibility.unwrap_or(Visibility::Any);

    let path = format!("/api/v1/repos/?limit={}&offset={}", limit, offset);
    let request = self.client.langsmith_get(&path)?;

    // LangSmith API returns a paginated response with a "repos" field
    #[derive(Deserialize)]
    struct ListReposResponse {
        repos: Vec<Prompt>,
    }

    let response: ListReposResponse = self.client.execute(request).await?;

    // Filter by visibility if specified
    let filtered = match visibility {
        Visibility::Public => response.repos.into_iter().filter(|p| p.is_public).collect(),
        Visibility::Private => response.repos.into_iter().filter(|p| !p.is_public).collect(),
        Visibility::Any => response.repos,
    };

    Ok(filtered)
}
```

**Characteristics:**

- Parameters are `Option<T>` with defaults
- Uses inline response struct for deserialization
- Performs client-side filtering (visibility filter not supported by API)
- Embeds pagination params in path string

#### Get Pattern (by handle/name)

```rust
pub async fn get(&self, handle: &str) -> Result<Prompt> {
    let path = format!("/api/v1/repos/{}", handle);
    let request = self.client.langsmith_get(&path)?;

    // The API wraps the prompt in a "repo" field
    #[derive(Deserialize)]
    struct PromptResponse {
        repo: Prompt,
    }

    let response: PromptResponse = self.client.execute(request).await?;
    Ok(response.repo)
}
```

**Characteristics:**

- Uses string identifier (handle) instead of UUID
- Unwraps response from wrapper field
- Inline response struct

### 2.3 Error Handling Pattern

**File**: `sdk/src/error.rs`

```rust
pub type Result<T> = std::result::Result<T, LangstarError>;

#[derive(Error, Debug)]
pub enum LangstarError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("Invalid schema format: {0}")]
    InvalidSchemaError(String),

    #[error("Invalid structured output method: {0}. Valid methods: json_schema, function_calling")]
    InvalidMethodError(String),

    #[error("Error: {0}")]
    Other(String),
}
```

**Characteristics:**

- Uses `thiserror` for error definitions
- Custom `Result<T>` type alias
- Structured errors with context
- HTTP status codes preserved
- Automatic conversions via `#[from]`

## 3. Recommended Rust SDK Method Signatures

Based on the OpenAPI spec from scout #453 and existing Langstar patterns:

### 3.1 Type Definitions

```rust
// sdk/src/playground_settings.rs

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Playground settings response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundSettings {
    pub id: Uuid,
    pub settings: Value,  // LangChain serialized model config
    pub options: PlaygroundOptions,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Options for playground settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundOptions {
    pub requests_per_second: Option<u32>,
}

/// Request to create playground settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundSettingsCreate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub settings: Value,  // Required
    pub options: PlaygroundOptions,
}

/// Request to update playground settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaygroundSettingsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PlaygroundOptions>,
}

/// Parameters for listing playground settings
#[derive(Debug, Clone, Default)]
pub struct ListPlaygroundSettingsParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
```

### 3.2 Client Methods

```rust
// sdk/src/client.rs

impl LangchainClient {
    /// List all playground settings for the current tenant
    pub async fn list_playground_settings(
        &self,
        params: crate::playground_settings::ListPlaygroundSettingsParams,
    ) -> Result<Vec<crate::playground_settings::PlaygroundSettings>> {
        let mut request = self.langsmith_get("/api/v1/playground-settings")?;

        if let Some(limit) = params.limit {
            request = request.query(&[("limit", limit)]);
        }
        if let Some(offset) = params.offset {
            request = request.query(&[("offset", offset)]);
        }

        self.execute(request).await
    }

    /// Get a single playground setting by ID
    pub async fn get_playground_settings(
        &self,
        id: uuid::Uuid,
    ) -> Result<crate::playground_settings::PlaygroundSettings> {
        let path = format!("/api/v1/playground-settings/{}", id);
        let request = self.langsmith_get(&path)?;
        self.execute(request).await
    }

    /// Create new playground settings
    pub async fn create_playground_settings(
        &self,
        request: crate::playground_settings::PlaygroundSettingsCreate,
    ) -> Result<crate::playground_settings::PlaygroundSettings> {
        let request_builder = self.langsmith_post("/api/v1/playground-settings")?.json(&request);
        self.execute(request_builder).await
    }

    /// Update existing playground settings
    pub async fn update_playground_settings(
        &self,
        id: uuid::Uuid,
        request: crate::playground_settings::PlaygroundSettingsUpdate,
    ) -> Result<crate::playground_settings::PlaygroundSettings> {
        let path = format!("/api/v1/playground-settings/{}", id);
        let request_builder = self.langsmith_patch(&path)?.json(&request);
        self.execute(request_builder).await
    }

    /// Delete playground settings
    pub async fn delete_playground_settings(&self, id: uuid::Uuid) -> Result<()> {
        let path = format!("/api/v1/playground-settings/{}", id);
        let request = self.langsmith_delete(&path)?;
        self.execute_status_only_request(request).await
    }
}
```

## 4. Pagination Strategy

### 4.1 Current Langstar Approach

Langstar SDK does **not** auto-paginate. Methods return a single page of results:

```rust
// Returns ONE page of up to `limit` items
pub async fn list_datasets(params: ListDatasetsParams) -> Result<Vec<Dataset>>
```

**Rationale:**

- Explicit control over pagination
- Predictable memory usage
- Allows caller to implement custom pagination logic

### 4.2 Playground Settings Pagination

The `/api/v1/playground-settings` endpoint uses offset/limit pagination:

- `limit`: Max items per page (default: unspecified in OpenAPI, recommend 20)
- `offset`: Starting position (default: 0)

**Recommendation**: Follow existing Langstar pattern - no auto-pagination.

**Example client usage:**

```rust
// Get first page
let page1 = client.list_playground_settings(ListPlaygroundSettingsParams {
    limit: Some(20),
    offset: Some(0),
}).await?;

// Get second page if needed
if page1.len() == 20 {
    let page2 = client.list_playground_settings(ListPlaygroundSettingsParams {
        limit: Some(20),
        offset: Some(20),
    }).await?;
}
```

## 5. Key Design Decisions

### 5.1 Settings Field as `Value`

The `settings` field is a dynamic JSON object with LangChain serialization format. Two options:

**Option A: Keep as `serde_json::Value` (Recommended)**

- Pros: Flexible, forward-compatible, minimal validation burden
- Cons: No compile-time type safety for provider-specific fields

**Option B: Typed enum with variants per provider**

- Pros: Type-safe, autocomplete, validation
- Cons: Brittle, requires updates when providers change, complex

**Recommendation**: Use `Value` for MVP. Add typed builders in later phase.

### 5.2 Error Handling

Reuse existing `LangstarError`:

- `ApiError` for HTTP errors (404, 400, etc.)
- `JsonError` for deserialization issues
- `ConfigError` for invalid settings format (if we add validation)

No new error variants needed for MVP.

### 5.3 Response Wrapper

The OpenAPI spec shows the list endpoint returns an array directly (not wrapped):

```json
GET /api/v1/playground-settings
Response: [ {...}, {...} ]
```

Therefore, no wrapper struct needed for list method.

## 6. Comparison with Python SDK

| Aspect          | Python SDK                    | Langstar Rust SDK               |
| --------------- | ----------------------------- | ------------------------------- |
| Pagination      | Auto-paginate with generators | Manual pagination via params    |
| Optional params | Keyword-only (`*,`)           | Struct with `Option<T>` fields  |
| Error handling  | Exception hierarchy           | `Result<T, LangstarError>` enum |
| Type safety     | Runtime via Pydantic          | Compile-time via structs        |
| Async           | Native async/await            | Tokio async/await               |

## 7. Implementation Phases (from Scout #453)

1. **Phase 1** (This phase): SDK research and design ✅
2. **Phase 2**: List playground settings
3. **Phase 3**: Get single playground setting by ID
4. **Phase 4**: Create playground setting
5. **Phase 5**: Update playground setting
6. **Phase 6**: Delete playground setting
7. **Phase 7**: CLI commands
8. **Phase 8**: Documentation and integration tests

## 8. References

- Scout report: `docs/research/453-ls-langsmith-model-providers-scout.md`
- Sample data: `reference/research/453-ls-langsmith-model-providers-playground-settings.json`
- OpenAPI spec: `reference/openapi/langchain/langsmith/openapi.json`
- Existing patterns:
  - Datasets: `sdk/src/client.rs:1037-1248`
  - Prompts: `sdk/src/prompts.rs:288-339`
  - Errors: `sdk/src/error.rs`
