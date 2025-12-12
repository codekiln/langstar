# OpenAPI Validation Report: LangSmith Annotation Queues

**Issue**: #341 - 334.1.1-openapi-validation
**Parent**: #335 (334.1-research-annotation-queues-precedent)
**Date**: 2025-11-27
**Status**: Complete

---

## Executive Summary

Validation of research report #335 against the LangSmith OpenAPI specification at `reference/api-specs/langsmith-openapi.json` using jq queries. This report documents confirmations, corrections, and additional API capabilities discovered.

**Key Findings:**

1. ✅ All 8 documented endpoints confirmed with correct HTTP methods
2. ✅ `add_runs_to_annotation_queue` body format confirmed as JSON array (not object)
3. ⚠️ **Correction**: `GET /annotation-queues/{queue_id}/runs` EXISTS - answers open question 8.1
4. ⚠️ **Correction**: List queues returns `total_runs` count - answers queue statistics question
5. ⚡ **Discovery**: 13 additional endpoints not documented in Python SDK
6. ⚡ **Discovery**: 10 additional schema fields for queue creation/response

---

## 1. Endpoint Validation

### 1.1 Documented Endpoints (All Confirmed ✅)

| Endpoint                                            | Method | Research Report  | OpenAPI Spec                                 | Status    |
| --------------------------------------------------- | ------ | ---------------- | -------------------------------------------- | --------- |
| `/annotation-queues`                                | GET    | List queues      | ✅ Returns `AnnotationQueueSchemaWithSize[]` | Confirmed |
| `/annotation-queues`                                | POST   | Create queue     | ✅ Accepts `AnnotationQueueCreateSchema`     | Confirmed |
| `/annotation-queues/{queue_id}`                     | GET    | Read queue       | ✅                                           | Confirmed |
| `/annotation-queues/{queue_id}`                     | PATCH  | Update queue     | ✅                                           | Confirmed |
| `/annotation-queues/{queue_id}`                     | DELETE | Delete queue     | ✅                                           | Confirmed |
| `/annotation-queues/{queue_id}/runs`                | POST   | Add runs         | ✅ Body: `array[uuid]`                       | Confirmed |
| `/annotation-queues/{queue_id}/runs/{queue_run_id}` | DELETE | Remove run       | ✅                                           | Confirmed |
| `/annotation-queues/{queue_id}/run/{index}`         | GET    | Get run at index | ✅                                           | Confirmed |

### 1.2 Critical Validation: Add Runs Request Body

**Research report claimed**: Body is JSON array of UUIDs, not object with `run_ids` field.

**OpenAPI spec confirms** (jq query):

```bash
jq '.paths["/api/v1/annotation-queues/{queue_id}/runs"].post.requestBody.content["application/json"].schema' reference/api-specs/langsmith-openapi.json
```

**Result**:

```json
{
  "type": "array",
  "items": {
    "type": "string",
    "format": "uuid"
  },
  "title": "Run Ids"
}
```

✅ **Confirmed**: Body is `["uuid-1", "uuid-2"]`, NOT `{"run_ids": [...]}`

### 1.3 Correction: List Runs in Queue EXISTS

**Research report Open Question 8.1**: "Is there an endpoint to list all runs in a queue?"

**OpenAPI spec shows** `GET /annotation-queues/{queue_id}/runs` exists:

```bash
jq '.paths["/api/v1/annotation-queues/{queue_id}/runs"].get.parameters' reference/api-specs/langsmith-openapi.json
```

**Parameters**:

- `offset`: integer (default: 0)
- `limit`: integer (min: 1, max: 100, default: 100)
- `archived`: boolean (optional)

⚠️ **Correction**: The Python SDK doesn't expose this endpoint, but the REST API supports paginated listing of runs in a queue.

### 1.4 Correction: List Queues Returns Size

**Research report**: Didn't mention queue size in list response.

**OpenAPI spec shows** list endpoint returns `AnnotationQueueSchemaWithSize[]`:

```bash
jq '.paths["/api/v1/annotation-queues"].get.responses["200"].content["application/json"].schema' reference/api-specs/langsmith-openapi.json
```

**Result**: Array of `AnnotationQueueSchemaWithSize` which includes `total_runs: integer`.

⚠️ **Correction**: List queues already returns run counts - no need for separate size endpoint for basic use cases.

---

## 2. Additional Endpoints Discovered

### 2.1 Standard Queue Endpoints (Not in Python SDK)

| Endpoint                                              | Method | Description                   | Priority |
| ----------------------------------------------------- | ------ | ----------------------------- | -------- |
| `/annotation-queues/{queue_id}/runs`                  | GET    | **List runs with pagination** | High     |
| `/annotation-queues/{queue_id}/size`                  | GET    | Get queue size                | Medium   |
| `/annotation-queues/{queue_id}/total_size`            | GET    | Get total size                | Low      |
| `/annotation-queues/{queue_id}/total_archived`        | GET    | Get archived count            | Low      |
| `/annotation-queues/{queue_id}/export`                | POST   | Export queue runs             | Medium   |
| `/annotation-queues/{queue_id}/runs/delete`           | POST   | Bulk delete runs              | Medium   |
| `/annotation-queues/populate`                         | POST   | Populate queue                | Medium   |
| `/annotation-queues/status/{annotation_queue_run_id}` | POST   | Update run status             | Low      |
| `/annotation-queues/{run_id}/queues`                  | GET    | Get queues for a run          | Low      |

### 2.2 Pairwise Queue Endpoints (Entirely Undocumented)

The OpenAPI spec reveals a complete "pairwise" queue type for A/B comparison:

| Endpoint                                                | Method     | Description                 |
| ------------------------------------------------------- | ---------- | --------------------------- |
| `/annotation-queues/pairwise`                           | GET, POST  | List/create pairwise queues |
| `/annotation-queues/pairwise/{queue_id}`                | GET, PATCH | Read/update pairwise queue  |
| `/annotation-queues/pairwise/{queue_id}/entries`        | GET        | List comparison entries     |
| `/annotation-queues/pairwise/{queue_id}/entries/delete` | POST       | Bulk delete entries         |
| `/annotation-queues/pairwise/{queue_id}/populate`       | POST       | Populate pairwise queue     |
| `/annotation-queues/pairwise/populate`                  | POST       | Populate any pairwise queue |
| `/annotation-queues/pairwise/{queue_id}/size`           | GET        | Get pairwise queue size     |
| `/annotation-queues/pairwise/status/{queue_entry_id}`   | POST       | Update entry status         |

---

## 3. Schema Corrections and Additions

### 3.1 AnnotationQueueCreateSchema

**Research report** documented:

- `name` (required)
- `description` (optional)
- `id` (optional)
- `rubric_instructions` (optional)

**OpenAPI spec reveals additional fields**:

```bash
jq '.components.schemas.AnnotationQueueCreateSchema.properties | keys' reference/api-specs/langsmith-openapi.json
```

| Field                    | Type    | Default | Description                    |
| ------------------------ | ------- | ------- | ------------------------------ |
| `num_reviewers_per_item` | integer | 1       | Number of reviewers per item   |
| `enable_reservations`    | boolean | true    | Enable item reservations       |
| `reservation_minutes`    | integer | 1       | Reservation timeout            |
| `default_dataset`        | uuid    | null    | Linked dataset for corrections |
| `rubric_items`           | array   | null    | Structured rubric items        |
| `session_ids`            | uuid[]  | null    | Linked project sessions        |
| `metadata`               | object  | null    | Arbitrary metadata             |

### 3.2 AnnotationQueueSchema (Response)

**Additional fields in queue responses**:

| Field            | Type    | Required | Description                       |
| ---------------- | ------- | -------- | --------------------------------- |
| `queue_type`     | enum    | Yes      | `"single"` or `"pairwise"`        |
| `source_rule_id` | uuid    | No       | Automation source rule            |
| `run_rule_id`    | uuid    | No       | Automation run rule               |
| `total_runs`     | integer | Yes*     | Queue size (*in WithSize variant) |

### 3.3 AnnotationQueueRubricItemSchema

**New structured rubric type** (not just `rubric_instructions` string):

```json
{
  "feedback_key": "string (required)",
  "description": "string | null",
  "value_descriptions": {"key": "description"} | null,
  "score_descriptions": {"score": "description"} | null
}
```

---

## 4. jq Queries Used

### List all annotation-queue endpoints:

```bash
jq '[.paths | to_entries[] | select(.key | contains("annotation-queue")) | {path: .key, methods: (.value | keys)}]' reference/api-specs/langsmith-openapi.json
```

### Extract all annotation queue schemas:

```bash
jq '.components.schemas | to_entries | map(select(.key | test("AnnotationQueue|QueueRun"; "i"))) | from_entries' reference/api-specs/langsmith-openapi.json
```

### Get specific endpoint details:

```bash
jq '.paths["/api/v1/annotation-queues/{queue_id}/runs"]' reference/api-specs/langsmith-openapi.json
```

### Get schema by name:

```bash
jq '.components.schemas.AnnotationQueueCreateSchema' reference/api-specs/langsmith-openapi.json
```

---

## 5. Recommendations for Implementation Issues

### 5.1 Update #337 (SDK Types)

Add these fields to type definitions:

```rust
// In CreateAnnotationQueueRequest
pub num_reviewers_per_item: Option<u32>,
pub enable_reservations: Option<bool>,
pub reservation_minutes: Option<u32>,
pub default_dataset: Option<Uuid>,
pub rubric_items: Option<Vec<RubricItem>>,
pub session_ids: Option<Vec<Uuid>>,
pub metadata: Option<serde_json::Value>,

// In AnnotationQueue response
pub queue_type: QueueType,  // enum { Single, Pairwise }
pub source_rule_id: Option<Uuid>,
pub run_rule_id: Option<Uuid>,
pub total_runs: Option<u32>,  // Present in list response

// New type
pub struct RubricItem {
    pub feedback_key: String,
    pub description: Option<String>,
    pub value_descriptions: Option<HashMap<String, String>>,
    pub score_descriptions: Option<HashMap<String, String>>,
}
```

### 5.2 Update #338 (SDK Client Methods)

Add these methods:

```rust
// HIGH PRIORITY - answers open question
pub async fn list_runs_in_annotation_queue(
    &self,
    queue_id: Uuid,
    offset: Option<u32>,
    limit: Option<u32>,
    archived: Option<bool>,
) -> Result<Vec<RunWithAnnotationQueueInfo>>

// MEDIUM PRIORITY
pub async fn get_annotation_queue_size(&self, queue_id: Uuid) -> Result<AnnotationQueueSize>
pub async fn bulk_delete_runs_from_queue(&self, queue_id: Uuid, run_ids: Vec<Uuid>) -> Result<()>
pub async fn export_annotation_queue(&self, queue_id: Uuid, ...) -> Result<...>
```

### 5.3 Update #339 (CLI Commands)

Change `queue items` from sequential index fetching to proper pagination:

```rust
// In cli/src/commands/queue.rs - QueueCommand enum variant
#[derive(Subcommand)]
pub enum QueueCommand {
    // ... other variants ...

    /// List runs in an annotation queue (uses paginated API)
    Items {
        /// Queue ID
        queue_id: Uuid,

        /// Maximum items to return
        #[arg(long, default_value = "100")]
        limit: u32,

        /// Starting offset for pagination
        #[arg(long, default_value = "0")]
        offset: u32,

        /// Show only archived runs
        #[arg(long)]
        archived: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
```

### 5.4 Future Work (Out of Scope)

- Pairwise queue support (entire feature)
- Automation rule integration (`source_rule_id`, `run_rule_id`)
- Export functionality

---

## 6. Saved Artifacts

The following files were saved to `reference/api-specs/`:

- `annotation-queue-endpoints.json` - All 21 endpoints with methods
- `annotation-queue-schemas.json` - All 22 related schemas

---

## 7. Summary

| Category                    | Count | Details                                          |
| --------------------------- | ----- | ------------------------------------------------ |
| Endpoints Validated         | 8     | All confirmed correct                            |
| Critical Findings Confirmed | 1     | Array body format                                |
| Corrections to Research     | 2     | List runs endpoint exists, size in list response |
| Additional Endpoints        | 13    | Standard queues (9) + Pairwise (8, overlapping)  |
| Additional Schema Fields    | 10+   | Reviewer settings, rubric items, metadata        |
| New Queue Type Discovered   | 1     | Pairwise (A/B comparison)                        |

**Recommendation**: Update implementation issues #337, #338, #339 with findings before starting implementation.
