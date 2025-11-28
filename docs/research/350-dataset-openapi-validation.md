# LangSmith Dataset API OpenAPI Validation Report

**Issue**: #350 - 346.3-openapi-validation Validate dataset design against LangSmith OpenAPI spec
**Epic**: #346 - ls-datasets - langstar dataset CLI support
**Research Reference**: #348, #349 - Research and Design phases
**Date**: 2025-11-28

## Executive Summary

This report validates the dataset API research findings from #348/349 against the live LangSmith OpenAPI specification. The validation confirms the research is largely accurate with a few corrections and discoveries that will improve the SDK implementation.

**Validation Status**: ✅ Research validated with minor corrections

---

## 1. OpenAPI Spec Update

| Property | Value |
|----------|-------|
| Source URL | `https://api.smith.langchain.com/openapi.json` |
| Fetch Date | 2025-11-28 |
| File Size | 639K (up from 635K on 2025-11-26) |
| Local Path | `reference/openapi/langchain/langsmith/openapi.json` |

### Extracted Fragments

| Fragment | Size | Purpose |
|----------|------|---------|
| `dataset-endpoints.json` | 61K | All `/api/v1/datasets/*` endpoints |
| `dataset-schemas.json` | 43K | All dataset-related schemas |
| `example-endpoints.json` | 30K | All `/api/v1/examples/*` endpoints |
| `example-schemas.json` | 29K | All example-related schemas |

---

## 2. Endpoint Validation

### 2.1 Dataset CRUD - ✅ Confirmed

| Method | Endpoint | Research | OpenAPI | Status |
|--------|----------|----------|---------|--------|
| GET | `/api/v1/datasets` | ✅ | ✅ | Confirmed |
| POST | `/api/v1/datasets` | ✅ | ✅ | Confirmed |
| GET | `/api/v1/datasets/{dataset_id}` | ✅ | ✅ | Confirmed |
| PATCH | `/api/v1/datasets/{dataset_id}` | ✅ | ✅ | Confirmed |
| DELETE | `/api/v1/datasets/{dataset_id}` | ✅ | ✅ | Confirmed |

### 2.2 Example CRUD - ✅ Confirmed

| Method | Endpoint | Research | OpenAPI | Status |
|--------|----------|----------|---------|--------|
| GET | `/api/v1/examples` | ✅ | ✅ | Confirmed |
| POST | `/api/v1/examples` | ✅ | ✅ | Confirmed |
| POST | `/api/v1/examples/bulk` | ✅ | ✅ | Confirmed |
| PATCH | `/api/v1/examples/bulk` | ✅ | ✅ | Confirmed |
| DELETE | `/api/v1/examples` | ✅ | ✅ | Confirmed |
| GET | `/api/v1/examples/{example_id}` | ✅ | ✅ | Confirmed |
| PATCH | `/api/v1/examples/{example_id}` | ✅ | ✅ | Confirmed |
| DELETE | `/api/v1/examples/{example_id}` | ✅ | ✅ | Confirmed |
| GET | `/api/v1/examples/count` | ✅ | ✅ | Confirmed |

### 2.3 Import/Export - ✅ Confirmed

All import/export endpoints documented in research are present in OpenAPI spec.

### 2.4 Versioning - ✅ Confirmed

All versioning endpoints documented in research are present in OpenAPI spec.

### 2.5 New Endpoints Discovered

The following endpoints were not in research but exist in OpenAPI:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/examples/validate` | POST | Validate single example |
| `/api/v1/examples/validate/bulk` | POST | Validate multiple examples |
| `/api/v1/datasets/{dataset_id}/splits` | GET/PUT | Manage dataset splits |
| `/api/v1/datasets/{dataset_id}/index` | POST | Create semantic index |
| `/api/v1/datasets/{dataset_id}/index/sync` | POST | Sync semantic index |
| `/api/v1/datasets/{dataset_id}/search` | POST | Semantic search |
| `/api/v1/datasets/{dataset_id}/generate` | POST | Generate synthetic examples |

---

## 3. Schema Validation

### 3.1 DataType Enum - ✅ Confirmed

```json
{
  "type": "string",
  "enum": ["kv", "llm", "chat"],
  "title": "DataType"
}
```

Research was correct: `kv`, `llm`, `chat` are the valid data types.

### 3.2 Dataset Schema - ⚠️ Corrections Required

**Research vs OpenAPI differences:**

| Field | Research | OpenAPI | Action |
|-------|----------|---------|--------|
| `inputs_schema` | `Option<serde_json::Value>` | `inputs_schema_definition` | **Rename field** |
| `outputs_schema` | `Option<serde_json::Value>` | `outputs_schema_definition` | **Rename field** |
| `tenant_id` | Not documented | `Uuid` (required) | **Add field** |
| `externally_managed` | Not documented | `Option<bool>` | **Add field** |
| `modified_at` | `Option<DateTime<Utc>>` | `DateTime<Utc>` (required) | **Make required** |
| `example_count` | `Option<i64>` | `i64` (required) | **Make required** |
| `session_count` | `Option<i64>` | `i64` (required) | **Make required** |

**Corrected Dataset struct:**

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Dataset {
    // Required fields
    pub id: Uuid,
    pub name: String,
    pub tenant_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub example_count: i64,
    pub session_count: i64,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs_schema_definition: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs_schema_definition: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub externally_managed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformations: Option<Vec<DatasetTransformation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_session_start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
```

### 3.3 DatasetCreate Schema - ⚠️ Corrections Required

| Field | Research | OpenAPI | Action |
|-------|----------|---------|--------|
| `id` | Not documented | `Option<Uuid>` | **Add field** (client-generated ID) |
| `extra` | Not documented | `Option<Value>` | **Add field** |
| `inputs_schema_definition` | Not in create | Present | **Add field** |
| `outputs_schema_definition` | Not in create | Present | **Add field** |
| `externally_managed` | Not in create | Present | **Add field** |
| `transformations` | Not in create | Present | **Add field** |
| `created_at` | Not in create | Present | **Add field** |

### 3.4 DatasetTransformation - ⚠️ Correction Required

Research showed `path: String`, but OpenAPI shows `path: Vec<String>`.

**Corrected struct:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetTransformation {
    pub path: Vec<String>,  // Was String, now Vec<String>
    pub transformation_type: DatasetTransformationType,
}
```

**DatasetTransformationType enum values (confirmed):**
- `convert_to_openai_message`
- `convert_to_openai_tool`
- `remove_system_messages`
- `remove_extra_fields`
- `extract_tools_from_run`

### 3.5 Example Schema - ⚠️ Corrections Required

| Field | Research | OpenAPI | Action |
|-------|----------|---------|--------|
| `name` | Not documented | `String` (required) | **Add field** |
| `inputs` | `Option<Value>` | `Value` (required) | **Make required** |
| `attachments` | `Option<HashMap<String, AttachmentInfo>>` | `attachment_urls: Option<Value>` | **Rename & simplify** |

**Example schema has required fields:**
- `id` (Uuid)
- `dataset_id` (Uuid)
- `inputs` (Object)
- `name` (String)

### 3.6 ExampleCreate Schema - ✅ Confirmed with additions

The ExampleCreate schema in OpenAPI includes additional fields:

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `dataset_id` | Uuid | - | **Required** |
| `inputs` | Option<Value> | - | Optional in create |
| `outputs` | Option<Value> | - | Optional |
| `metadata` | Option<Value> | - | Optional |
| `split` | String or String[] | `"base"` | Default is "base" |
| `id` | Option<Uuid> | - | Client-generated ID |
| `source_run_id` | Option<Uuid> | - | Link to source run |
| `use_source_run_io` | bool | `false` | Copy I/O from run |
| `use_source_run_attachments` | String[] | `[]` | Copy attachments |
| `use_legacy_message_format` | bool | `false` | LLM message format |
| `created_at` | Option<String> | - | Custom timestamp |

### 3.7 ExampleUpdate Schema - ✅ Confirmed with additions

New fields discovered:
- `attachments_operations: Option<AttachmentsOperations>` - Rename/retain attachments
- `overwrite: bool` (default: false) - Overwrite vs merge behavior

**AttachmentsOperations schema:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentsOperations {
    /// Mapping of old attachment names to new names
    pub rename: HashMap<String, String>,
    /// List of attachment names to keep
    pub retain: Vec<String>,
}
```

### 3.8 DatasetVersion Schema - ✅ Confirmed

Research was correct:
```rust
pub struct DatasetVersion {
    pub tags: Option<Vec<String>>,
    pub as_of: DateTime<Utc>,  // Required
}
```

### 3.9 DatasetDiffInfo Schema - ✅ Confirmed

Research was correct for the structure.

---

## 4. Query Parameter Validation

### 4.1 List Datasets Parameters - ⚠️ One correction

| Parameter | Research | OpenAPI | Status |
|-----------|----------|---------|--------|
| `id` | uuid[] | uuid[] | ✅ Confirmed |
| `data_type` | string | DataType or DataType[] | ⚠️ Can be array |
| `name` | string | string | ✅ Confirmed |
| `name_contains` | string | string | ✅ Confirmed |
| `metadata` | object | string (JSON) | ⚠️ Type is string |
| `offset` | int | int (default: 0) | ✅ Confirmed |
| `limit` | int | int (max: 100, default: 100) | ✅ Confirmed |
| `sort_by` | string | SortByDatasetColumn | ✅ Confirmed |
| `sort_by_desc` | bool | bool (default: true) | ✅ Confirmed |
| `tag_value_id` | uuid[] | uuid[] | ✅ Confirmed |
| `exclude_corrections_datasets` | Not documented | bool (default: false) | 🆕 New |

### 4.2 List Examples Parameters - ✅ Confirmed

| Parameter | Research | OpenAPI | Status |
|-----------|----------|---------|--------|
| `dataset` | uuid (required) | uuid | ⚠️ Optional in spec |
| `id` | uuid[] | uuid[] | ✅ Confirmed |
| `as_of` | datetime | datetime or "latest" | ✅ Confirmed |
| `metadata` | object | string (JSON) | ⚠️ Type is string |
| `full_text_contains` | string | string[] | ⚠️ Is array |
| `splits` | string[] | string[] | ✅ Confirmed |
| `filter` | string | string | ✅ Confirmed |
| `offset` | int | int (default: 0) | ✅ Confirmed |
| `limit` | int | int (max: 100, default: 100) | ✅ Confirmed |
| `order` | string | ExampleListOrder | ✅ Confirmed |
| `descending` | bool | bool | ✅ Confirmed |
| `select` | string[] | ExampleSelect[] | ✅ Confirmed |
| `random_seed` | int | number | ✅ Confirmed |

---

## 5. Summary of Required Changes

### 5.1 High Priority (Breaking)

1. **Dataset schema field renames:**
   - `inputs_schema` → `inputs_schema_definition`
   - `outputs_schema` → `outputs_schema_definition`

2. **Required field corrections:**
   - `Dataset.modified_at` - Make required
   - `Dataset.example_count` - Make required
   - `Dataset.session_count` - Make required
   - `Dataset.tenant_id` - Add as required
   - `Example.name` - Add as required
   - `Example.inputs` - Make required

3. **Type corrections:**
   - `DatasetTransformation.path` - `String` → `Vec<String>`
   - `full_text_contains` - `String` → `Vec<String>`
   - `data_type` filter - Can accept array

### 5.2 Medium Priority (Additions)

1. **New Dataset fields:**
   - `externally_managed: Option<bool>`
   - `tenant_id: Uuid`

2. **New DatasetCreate fields:**
   - `id: Option<Uuid>`
   - `extra: Option<Value>`
   - Schema definition fields

3. **New ExampleCreate fields:**
   - `use_legacy_message_format: bool`
   - Default split is `"base"`

4. **New ExampleUpdate fields:**
   - `attachments_operations: Option<AttachmentsOperations>`
   - `overwrite: bool`

### 5.3 Low Priority (New Features)

1. **New endpoints to potentially support:**
   - Example validation endpoints
   - Dataset splits management
   - Semantic indexing and search
   - Synthetic example generation

---

## 6. Implementation Guidance for Phase 4

This validation report feeds directly into **Phase 4: SDK Types** (issue to be created as `346.4-sdk-dataset-types`).

### 6.1 Target File

Create `sdk/src/datasets.rs` following the pattern from `sdk/src/annotation_queues.rs`.

### 6.2 Structs to Implement (Priority Order)

| Struct | Source | Notes |
|--------|--------|-------|
| `DataType` | Section 3.1 | Enum: `kv`, `llm`, `chat` |
| `Dataset` | Section 3.2 | Use corrected struct (7 required, 8 optional fields) |
| `DatasetCreate` | Section 3.3 | Only `name` required |
| `DatasetUpdate` | OpenAPI `DatasetUpdate` | All fields optional |
| `DatasetTransformation` | Section 3.4 | `path: Vec<String>` (not String) |
| `DatasetTransformationType` | Section 3.4 | 5-value enum |
| `Example` | Section 3.5 | 4 required fields including `name` |
| `ExampleCreate` | Section 3.6 | Only `dataset_id` required |
| `ExampleUpdate` | Section 3.7 | Includes `AttachmentsOperations` |
| `DatasetVersion` | Section 3.8 | `as_of` required |
| `DatasetDiffInfo` | Research doc | 3 Vec<Uuid> fields |
| `AttachmentsOperations` | Section 3.7 | For attachment rename/retain |

### 6.3 Key Implementation Notes

1. **Use `#[serde(rename_all = "snake_case")]`** - API uses snake_case, not camelCase
2. **Separate request/response types** - `Dataset` (response) vs `DatasetCreate` (request)
3. **Default values to implement:**
   - `DatasetCreate.data_type` defaults to `kv`
   - `ExampleCreate.split` defaults to `"base"`
   - `ExampleUpdate.overwrite` defaults to `false`

### 6.4 Reference Files

| File | Purpose |
|------|---------|
| `reference/api-specs/langsmith/dataset-schemas.json` | Full Dataset schemas from OpenAPI |
| `reference/api-specs/langsmith/example-schemas.json` | Full Example schemas from OpenAPI |
| `sdk/src/annotation_queues.rs` | Pattern to follow |
| `docs/research/346-dataset-api-research.md` | Original research with SDK method signatures |

---

## 7. jq Validation Commands

All validation was performed using jq queries against the OpenAPI spec:

```bash
# Refresh OpenAPI spec
curl -s -o reference/openapi/langchain/langsmith/openapi.json \
  https://api.smith.langchain.com/openapi.json

# List all dataset endpoints
jq '.paths | keys | map(select(contains("dataset")))' openapi.json

# Get Dataset schema
jq '.components.schemas.Dataset' openapi.json

# Get DataType enum
jq '.components.schemas.DataType' openapi.json

# Get list datasets parameters
jq '.["/api/v1/datasets"].get.parameters' dataset-endpoints.json

# Get ExampleCreate from endpoint
jq '.["/api/v1/examples"].post.requestBody.content["application/json"].schema' \
  example-endpoints.json
```

---

## 8. Recommendations for Implementation

1. **Use snake_case in Rust, camelCase via serde:**
   - All API fields use snake_case in the spec
   - Research incorrectly suggested camelCase
   - Use `#[serde(rename_all = "snake_case")]` is not needed (already snake_case)

2. **Handle optional vs required carefully:**
   - Response schemas have more required fields than create schemas
   - Create separate `Dataset` (response) and `DatasetCreate` (request) types

3. **Default values:**
   - `data_type` defaults to `"kv"`
   - `split` defaults to `"base"`
   - `limit` defaults to `100` with max `100`
   - `sort_by_desc` defaults to `true`

4. **Pagination:**
   - Research correctly identified offset-based pagination
   - Max limit is 100 per request (confirmed)

---

## 9. References

- OpenAPI Spec: `reference/openapi/langchain/langsmith/openapi.json`
- Research Report: `docs/research/346-dataset-api-research.md`
- Extracted Fragments: `reference/api-specs/langsmith/dataset-*.json`, `example-*.json`
- jq Queries: `reference/api-specs/langsmith/FRAGMENTS.md`
