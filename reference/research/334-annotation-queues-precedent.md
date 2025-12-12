# Research Report: LangSmith Annotation Queue Implementation Precedent

**Issue**: #334 - ls-annotation-queues milestone
**Sub-Issue**: #335 - Research langsmith-sdk annotation queue precedent
**Date**: 2025-11-27
**Status**: Complete

---

## Executive Summary

This report analyzes the LangSmith Python SDK's annotation queue implementation to establish recommendations for implementing `langstar queue` commands in Rust. The Python SDK provides a well-designed reference with paginated listing, CRUD operations for queues, and run management within queues.

**Key Recommendations:**

1. Create `sdk/src/annotation_queues.rs` with `AnnotationQueue` and related types
2. Add annotation queue client methods to `sdk/src/client.rs` following existing patterns
3. Implement CLI commands under `langstar queue` subcommand group
4. Follow existing `runs.rs` patterns for serialization and pagination
5. Support queue creation, listing, deletion, and run management operations

---

## 1. Python SDK Analysis

### 1.1 SDK Methods Overview

**Location**: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py:6874`

The Python SDK provides the following annotation queue methods:

| Method                               | Return Type                  | Description                 |
| ------------------------------------ | ---------------------------- | --------------------------- |
| `list_annotation_queues()`           | `Iterator[AnnotationQueue]`  | List queues with pagination |
| `create_annotation_queue()`          | `AnnotationQueueWithDetails` | Create a new queue          |
| `read_annotation_queue()`            | `AnnotationQueueWithDetails` | Get queue by ID             |
| `update_annotation_queue()`          | `None`                       | Update queue metadata       |
| `delete_annotation_queue()`          | `None`                       | Delete a queue              |
| `add_runs_to_annotation_queue()`     | `None`                       | Add runs to a queue         |
| `delete_run_from_annotation_queue()` | `None`                       | Remove a run from queue     |
| `get_run_from_annotation_queue()`    | `RunWithAnnotationQueueInfo` | Get run at index            |

### 1.2 Method Signatures

#### list_annotation_queues

```python
def list_annotation_queues(
    self,
    *,
    queue_ids: Optional[list[ID_TYPE]] = None,
    name: Optional[str] = None,
    name_contains: Optional[str] = None,
    limit: Optional[int] = None,
) -> Iterator[ls_schemas.AnnotationQueue]:
```

**Request Parameters** (query string):

- `ids`: List of UUIDs to filter by
- `name`: Exact name match
- `name_contains`: Substring match on name
- `limit`: Max results per page (capped at 100)

**Endpoint**: `GET /annotation-queues`

#### create_annotation_queue

```python
def create_annotation_queue(
    self,
    *,
    name: str,
    description: Optional[str] = None,
    queue_id: Optional[ID_TYPE] = None,
    rubric_instructions: Optional[str] = None,
) -> ls_schemas.AnnotationQueueWithDetails:
```

**Request Body** (JSON):

```json
{
  "name": "My Queue",
  "description": "Optional description",
  "id": "optional-uuid-or-auto-generated",
  "rubric_instructions": "Optional rubric for annotators"
}
```

**Endpoint**: `POST /annotation-queues`

**Note**: If `queue_id` is not provided, the SDK generates a UUID client-side: `str(uuid.uuid4())`

#### read_annotation_queue

```python
def read_annotation_queue(self, queue_id: ID_TYPE) -> ls_schemas.AnnotationQueueWithDetails:
```

**Endpoint**: `GET /annotation-queues/{queue_id}`

Returns full queue details including `rubric_instructions`.

#### update_annotation_queue

```python
def update_annotation_queue(
    self,
    queue_id: ID_TYPE,
    *,
    name: str,
    description: Optional[str] = None,
    rubric_instructions: Optional[str] = None,
) -> None:
```

**Request Body** (JSON):

```json
{
  "name": "Required new name",
  "description": "Optional description",
  "rubric_instructions": "Optional rubric"
}
```

**Endpoint**: `PATCH /annotation-queues/{queue_id}`

**Note**: `name` is required for updates.

#### delete_annotation_queue

```python
def delete_annotation_queue(self, queue_id: ID_TYPE) -> None:
```

**Endpoint**: `DELETE /annotation-queues/{queue_id}`

#### add_runs_to_annotation_queue

```python
def add_runs_to_annotation_queue(
    self, queue_id: ID_TYPE, *, run_ids: list[ID_TYPE]
) -> None:
```

**Request Body** (JSON array of UUIDs):

```json
["uuid-1", "uuid-2", "uuid-3"]
```

**Endpoint**: `POST /annotation-queues/{queue_id}/runs`

**Key Insight**: The body is a JSON array of UUID strings, NOT an object with a `run_ids` field.

#### delete_run_from_annotation_queue

```python
def delete_run_from_annotation_queue(
    self, queue_id: ID_TYPE, *, run_id: ID_TYPE
) -> None:
```

**Endpoint**: `DELETE /annotation-queues/{queue_id}/runs/{run_id}`

#### get_run_from_annotation_queue

```python
def get_run_from_annotation_queue(
    self, queue_id: ID_TYPE, *, index: int
) -> ls_schemas.RunWithAnnotationQueueInfo:
```

**Endpoint**: `GET /annotation-queues/{queue_id}/run/{index}`

Returns a run with queue-specific metadata (added_at, last_reviewed_time).

---

## 2. Data Structures

### 2.1 AnnotationQueue Schema

**Location**: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py:845`

```python
class AnnotationQueue(BaseModel):
    """Represents an annotation queue."""

    id: UUID
    """The unique identifier of the annotation queue."""
    name: str
    """The name of the annotation queue."""
    description: Optional[str] = None
    """An optional description of the annotation queue."""
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    """The timestamp when the annotation queue was created."""
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    """The timestamp when the annotation queue was last updated."""
    tenant_id: UUID
    """The ID of the tenant associated with the annotation queue."""
```

### 2.2 AnnotationQueueWithDetails Schema

**Location**: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py:862`

```python
class AnnotationQueueWithDetails(AnnotationQueue):
    """Represents an annotation queue with details."""

    rubric_instructions: Optional[str] = None
    """The rubric instructions for the annotation queue."""
```

### 2.3 RunWithAnnotationQueueInfo Schema

**Location**: `reference/repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py:592`

```python
class RunWithAnnotationQueueInfo(RunBase):
    """Run schema with annotation queue info."""

    last_reviewed_time: Optional[datetime] = None
    """The last time this run was reviewed."""
    added_at: Optional[datetime] = None
    """The time this run was added to the queue."""
```

---

## 3. API Endpoints Summary

| Operation        | Method | Endpoint                                      | Request Body         | Response            |
| ---------------- | ------ | --------------------------------------------- | -------------------- | ------------------- |
| List queues      | GET    | `/annotation-queues`                          | -                    | Paginated list      |
| Create queue     | POST   | `/annotation-queues`                          | Queue create payload | Created queue       |
| Read queue       | GET    | `/annotation-queues/{queue_id}`               | -                    | Queue with details  |
| Update queue     | PATCH  | `/annotation-queues/{queue_id}`               | Queue update payload | -                   |
| Delete queue     | DELETE | `/annotation-queues/{queue_id}`               | -                    | -                   |
| Add runs         | POST   | `/annotation-queues/{queue_id}/runs`          | Array of run UUIDs   | -                   |
| Remove run       | DELETE | `/annotation-queues/{queue_id}/runs/{run_id}` | -                    | -                   |
| Get run at index | GET    | `/annotation-queues/{queue_id}/run/{index}`   | -                    | Run with queue info |

### 3.1 Pagination

List operations use offset-based pagination via `_get_paginated_list`:

- Query params: `limit`, `offset`
- Default limit: 100 (capped at 100)
- SDK yields items and fetches next page automatically

---

## 4. Recommendations for Rust Implementation

### 4.1 SDK Types (`sdk/src/annotation_queues.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Annotation queue schema.
///
/// # API Reference
///
/// Returned by `GET /annotation-queues` and `GET /annotation-queues/{id}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueue {
    /// Unique identifier for the queue
    pub id: Uuid,

    /// Name of the queue
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// When the queue was created
    pub created_at: DateTime<Utc>,

    /// When the queue was last updated
    pub updated_at: DateTime<Utc>,

    /// Tenant ID (workspace)
    pub tenant_id: Uuid,
}

/// Annotation queue with full details.
///
/// # API Reference
///
/// Returned by `GET /annotation-queues/{id}` and `POST /annotation-queues`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueueWithDetails {
    #[serde(flatten)]
    pub base: AnnotationQueue,

    /// Rubric instructions for annotators
    pub rubric_instructions: Option<String>,
}

/// Request to create a new annotation queue.
#[derive(Debug, Clone, Serialize)]
pub struct CreateAnnotationQueueRequest {
    /// Name of the queue (required)
    pub name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional client-provided ID (UUID generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Optional rubric instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_instructions: Option<String>,
}

/// Request to update an annotation queue.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateAnnotationQueueRequest {
    /// New name (required for updates)
    pub name: String,

    /// Optional new description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional new rubric instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_instructions: Option<String>,
}

/// Query parameters for listing annotation queues.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListAnnotationQueuesParams {
    /// Filter by specific queue IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<Uuid>>,

    /// Filter by exact name match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Filter by name substring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,

    /// Max results per page (capped at 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Run with annotation queue metadata.
///
/// Extends the base Run with queue-specific fields.
#[derive(Debug, Clone, Deserialize)]
pub struct RunWithAnnotationQueueInfo {
    #[serde(flatten)]
    pub run: super::runs::Run,

    /// When the run was last reviewed
    pub last_reviewed_time: Option<DateTime<Utc>>,

    /// When the run was added to the queue
    pub added_at: Option<DateTime<Utc>>,
}
```

### 4.2 Client Methods (`sdk/src/client.rs`)

````rust
impl LangchainClient {
    // ═══════════════════════════════════════════════════════════════════════
    // Annotation Queue API
    // ═══════════════════════════════════════════════════════════════════════

    /// List annotation queues with optional filtering.
    ///
    /// # Arguments
    /// * `params` - Query parameters for filtering
    /// * `limit` - Optional client-side limit (pagination handled internally)
    ///
    /// # Example
    /// ```no_run
    /// let params = ListAnnotationQueuesParams {
    ///     name_contains: Some("review".to_string()),
    ///     ..Default::default()
    /// };
    /// let queues = client.list_annotation_queues(params, Some(10)).await?;
    /// ```
    pub async fn list_annotation_queues(
        &self,
        params: ListAnnotationQueuesParams,
        limit: Option<usize>,
    ) -> Result<Vec<AnnotationQueue>> {
        // Use _get_paginated_list pattern from Python SDK
        // GET /api/v1/annotation-queues with query params
    }

    /// Create a new annotation queue.
    ///
    /// # Arguments
    /// * `request` - Queue creation parameters
    ///
    /// # Returns
    /// The created queue with full details
    pub async fn create_annotation_queue(
        &self,
        request: CreateAnnotationQueueRequest,
    ) -> Result<AnnotationQueueWithDetails> {
        // POST /api/v1/annotation-queues
    }

    /// Get an annotation queue by ID.
    pub async fn read_annotation_queue(
        &self,
        queue_id: Uuid,
    ) -> Result<AnnotationQueueWithDetails> {
        // GET /api/v1/annotation-queues/{queue_id}
    }

    /// Update an annotation queue.
    pub async fn update_annotation_queue(
        &self,
        queue_id: Uuid,
        request: UpdateAnnotationQueueRequest,
    ) -> Result<()> {
        // PATCH /api/v1/annotation-queues/{queue_id}
    }

    /// Delete an annotation queue.
    pub async fn delete_annotation_queue(&self, queue_id: Uuid) -> Result<()> {
        // DELETE /api/v1/annotation-queues/{queue_id}
    }

    /// Add runs to an annotation queue.
    ///
    /// # Arguments
    /// * `queue_id` - The queue to add runs to
    /// * `run_ids` - List of run UUIDs to add
    pub async fn add_runs_to_annotation_queue(
        &self,
        queue_id: Uuid,
        run_ids: Vec<Uuid>,
    ) -> Result<()> {
        // POST /api/v1/annotation-queues/{queue_id}/runs
        // Body: JSON array of UUID strings
    }

    /// Remove a run from an annotation queue.
    pub async fn delete_run_from_annotation_queue(
        &self,
        queue_id: Uuid,
        run_id: Uuid,
    ) -> Result<()> {
        // DELETE /api/v1/annotation-queues/{queue_id}/runs/{run_id}
    }

    /// Get a run from an annotation queue at the specified index.
    pub async fn get_run_from_annotation_queue(
        &self,
        queue_id: Uuid,
        index: u32,
    ) -> Result<RunWithAnnotationQueueInfo> {
        // GET /api/v1/annotation-queues/{queue_id}/run/{index}
    }
}
````

### 4.3 CLI Commands (`cli/src/commands/queue.rs`)

```rust
use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser)]
pub struct QueueArgs {
    #[command(subcommand)]
    pub command: QueueCommand,
}

#[derive(Subcommand)]
pub enum QueueCommand {
    /// Create a new annotation queue
    Create {
        /// Name of the queue
        #[arg(short, long)]
        name: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,

        /// Optional rubric instructions
        #[arg(long)]
        rubric: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List annotation queues
    List {
        /// Filter by name substring
        #[arg(long)]
        name_contains: Option<String>,

        /// Filter by exact name
        #[arg(long)]
        name: Option<String>,

        /// Maximum number of queues to return
        #[arg(long, default_value = "100")]
        limit: u32,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Get details of a specific queue
    Get {
        /// Queue ID
        queue_id: Uuid,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Delete an annotation queue
    Delete {
        /// Queue ID to delete
        queue_id: Uuid,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Add runs to an annotation queue
    AddRuns {
        /// Queue ID
        queue_id: Uuid,

        /// Run IDs to add
        #[arg(required = true)]
        run_ids: Vec<Uuid>,
    },

    /// Remove a run from an annotation queue
    RemoveRun {
        /// Queue ID
        queue_id: Uuid,

        /// Run ID to remove
        run_id: Uuid,
    },

    /// List runs in an annotation queue
    ///
    /// NOTE: The SDK only provides `get_run_from_annotation_queue(index)`.
    /// This command would need to fetch runs sequentially by index until
    /// exhausted, or use an undocumented endpoint if one exists.
    /// See Open Questions section 8.1.
    Items {
        /// Queue ID
        queue_id: Uuid,

        /// Maximum items to return
        #[arg(long, default_value = "100")]
        limit: u32,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
```

### 4.4 Example CLI UX

```bash
# Create a queue
$ langstar queue create --name "CI Review" --description "Auto triage from CI"
Created queue:
  ID: 12345678-1234-1234-1234-123456789012
  Name: CI Review
  Created: 2025-11-27T10:00:00Z

# List queues
$ langstar queue list --name-contains "review"
ID                                    NAME         DESCRIPTION      CREATED
12345678-1234-1234-1234-123456789012  CI Review    Auto triage...   2025-11-27

# List as JSON
$ langstar queue list --json
[{"id": "12345678-...", "name": "CI Review", ...}]

# Get queue details
$ langstar queue get 12345678-1234-1234-1234-123456789012

# Add runs to queue
$ langstar queue add-runs 12345678-... abcdef01-... abcdef02-...
Added 2 runs to queue CI Review

# Remove run from queue
$ langstar queue remove-run 12345678-... abcdef01-...
Removed run abcdef01-... from queue CI Review

# Delete queue
$ langstar queue delete 12345678-... --force
Deleted queue 12345678-...
```

---

## 5. Implementation Phases

### Phase 1: SDK Types

- [ ] Create `sdk/src/annotation_queues.rs` with type definitions
- [ ] Add `pub mod annotation_queues;` to `sdk/src/lib.rs`
- [ ] Add unit tests for serialization/deserialization

### Phase 2: Client Methods

- [ ] Add annotation queue methods to `sdk/src/client.rs`
- [ ] Implement pagination for `list_annotation_queues`
- [ ] Add integration tests with httpmock

### Phase 3: CLI Commands

- [ ] Create `cli/src/commands/queue.rs` with subcommands
- [ ] Register `queue` subcommand in CLI main
- [ ] Implement table and JSON output formats
- [ ] Add CLI tests with assert_cmd

### Phase 4: Documentation

- [ ] Add doc comments with examples
- [ ] Update CLI help text
- [ ] Create docs/queues.md with usage examples

---

## 6. Differences from Original Issue Spec

The original issue #334 spec proposed some endpoints that don't match the actual SDK:

| Original Spec                          | Actual API                      | Notes                              |
| -------------------------------------- | ------------------------------- | ---------------------------------- |
| `POST .../items` with `run_ids` object | `POST .../runs` with array body | Body is JSON array, not object     |
| `GET .../items`                        | `GET .../run/{index}`           | Gets single run by index, not list |
| `DELETE .../items/{item_id}`           | `DELETE .../runs/{run_id}`      | Uses run_id, not item_id           |

**Recommendation**: Follow the actual SDK implementation rather than the original spec.

---

## 7. Testing Strategy

### Unit Tests

- Serialization/deserialization of all types
- Request body generation
- Query parameter serialization

### Integration Tests (httpmock)

- Mock all annotation queue endpoints
- Test pagination for list operations
- Test error handling (404, 400, etc.)

### CLI Tests (assert_cmd)

- Test all subcommands with valid inputs
- Test JSON output format
- Test error messages

---

## 8. Open Questions

1. **Listing runs in a queue**: The SDK only provides `get_run_from_annotation_queue(index)`. Is there an endpoint to list all runs in a queue?
   - **Recommendation**: Check if there's an undocumented endpoint, or implement sequential fetching by index.

2. **Bulk import from file**: The original spec mentioned `--runs-file` for bulk imports.
   - **Recommendation**: Implement as CLI feature reading UUIDs from file and calling `add_runs_to_annotation_queue` in batches.

3. **Queue size/count**: No endpoint for getting queue statistics.
   - **Recommendation**: Could be useful CLI feature if API supports it.

---

## 9. References

- [LangSmith SDK Python client.py](../repo/langchain-ai/langsmith-sdk/code/python/langsmith/client.py) - Lines 6874-7085
- [LangSmith SDK schemas.py](../repo/langchain-ai/langsmith-sdk/code/python/langsmith/schemas.py) - Lines 592-599, 845-867
- [Existing langstar runs.rs](../../sdk/src/runs.rs) - Pattern to follow
- [Existing langstar client.rs](../../sdk/src/client.rs) - Client patterns
- [Prior art: runs query research](./298-ls-runs-query-precedent.md) - Similar research format

---

## Appendix: HTTP Request Examples

### Create Queue

```http
POST /api/v1/annotation-queues
Content-Type: application/json
X-Api-Key: <api-key>

{
  "name": "My Queue",
  "description": "Description",
  "id": "optional-uuid",
  "rubric_instructions": "Instructions for annotators"
}
```

### List Queues

```http
GET /api/v1/annotation-queues?name_contains=review&limit=50
X-Api-Key: <api-key>
```

### Add Runs to Queue

```http
POST /api/v1/annotation-queues/12345678-1234-1234-1234-123456789012/runs
Content-Type: application/json
X-Api-Key: <api-key>

["run-uuid-1", "run-uuid-2", "run-uuid-3"]
```

### Get Run at Index

```http
GET /api/v1/annotation-queues/12345678-1234-1234-1234-123456789012/run/0
X-Api-Key: <api-key>
```
