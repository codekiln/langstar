//! Dataset and Example types for LangSmith datasets API.
//!
//! This module provides types for managing datasets and examples in LangSmith.
//! Datasets are collections of input/output examples used for testing and evaluation.
//!
//! # API Reference
//!
//! - Dataset endpoints: `/api/v1/datasets`
//! - Example endpoints: `/api/v1/examples`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//!
//! # Example
//!
//! ```no_run
//! use langstar_sdk::datasets::{DatasetCreate, DataType};
//!
//! let request = DatasetCreate {
//!     name: "My Dataset".to_string(),
//!     description: Some("Test dataset for evaluation".to_string()),
//!     data_type: Some(DataType::Kv),
//!     ..Default::default()
//! };
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::serde_utils::{deserialize_flexible_datetime, deserialize_flexible_datetime_opt};

// ============================================================================
// Dataset Types
// ============================================================================

/// Data type enum for datasets.
///
/// Determines how inputs/outputs are structured and displayed.
///
/// # API Reference
///
/// OpenAPI enum: `["kv", "llm", "chat"]`
/// Default: `kv`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// Key-value pairs (default) - generic input/output mapping
    #[default]
    Kv,
    /// LLM completion format - prompt/completion pairs
    Llm,
    /// Chat format - message-based conversations
    Chat,
}

/// Dataset transformation type enum.
///
/// Defines the type of transformation to apply to dataset inputs/outputs.
///
/// # API Reference
///
/// OpenAPI enum values from `DatasetTransformationType`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DatasetTransformationType {
    /// Convert to OpenAI message format
    ConvertToOpenaiMessage,
    /// Convert to OpenAI tool format
    ConvertToOpenaiTool,
    /// Remove system messages from chat
    RemoveSystemMessages,
    /// Remove extra fields not in schema
    RemoveExtraFields,
    /// Extract tools from run data
    ExtractToolsFromRun,
}

/// Dataset transformation configuration.
///
/// Specifies a transformation to apply at a given path in the data.
///
/// # API Reference
///
/// Maps to `DatasetTransformation` in OpenAPI spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetTransformation {
    /// JSON path to the field to transform (as array of path segments)
    pub path: Vec<String>,

    /// Type of transformation to apply
    pub transformation_type: DatasetTransformationType,
}

/// Dataset schema (response type).
///
/// Represents a dataset as returned by the API.
///
/// # API Reference
///
/// Maps to `Dataset` and `DatasetSchemaForUpdate` in OpenAPI spec.
/// Returned by `GET /datasets`, `GET /datasets/{id}`, `POST /datasets`, `PATCH /datasets/{id}`
///
/// # Required Fields
///
/// - `id`, `name`, `tenant_id`
///
/// # Note
///
/// Fields `example_count`, `session_count`, and `modified_at` are optional because
/// PATCH responses (`DatasetSchemaForUpdate`) do not include these computed fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Unique identifier for the dataset
    pub id: Uuid,

    /// Name of the dataset
    pub name: String,

    /// Tenant ID (workspace) that owns this dataset
    pub tenant_id: Uuid,

    /// Number of examples in the dataset (not present in PATCH responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_count: Option<i64>,

    /// Number of sessions (projects) linked to this dataset (not present in PATCH responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_count: Option<i64>,

    /// When the dataset was last modified (not present in PATCH responses)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub modified_at: Option<DateTime<Utc>>,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When the dataset was created
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub created_at: Option<DateTime<Utc>>,

    /// Data type (kv, llm, chat)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,

    /// JSON Schema for input validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs_schema_definition: Option<Value>,

    /// JSON Schema for output validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs_schema_definition: Option<Value>,

    /// Whether the dataset is managed externally
    #[serde(skip_serializing_if = "Option::is_none")]
    pub externally_managed: Option<bool>,

    /// Transformations to apply to data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformations: Option<Vec<DatasetTransformation>>,

    /// When the last session started
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub last_session_start_time: Option<DateTime<Utc>>,

    /// Additional metadata (arbitrary JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Request to create a new dataset.
///
/// # API Reference
///
/// Request body for `POST /datasets`
///
/// # Example
///
/// ```
/// use langstar_sdk::datasets::{DatasetCreate, DataType};
///
/// let request = DatasetCreate {
///     name: "Evaluation Dataset".to_string(),
///     description: Some("Dataset for model evaluation".to_string()),
///     data_type: Some(DataType::Chat),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct DatasetCreate {
    /// Name of the dataset (required)
    pub name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional client-provided ID (UUID generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Data type (default: kv)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,

    /// JSON Schema for input validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs_schema_definition: Option<Value>,

    /// JSON Schema for output validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs_schema_definition: Option<Value>,

    /// Whether the dataset is managed externally
    #[serde(skip_serializing_if = "Option::is_none")]
    pub externally_managed: Option<bool>,

    /// Transformations to apply to data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformations: Option<Vec<DatasetTransformation>>,

    /// When the dataset was created (usually auto-set by server)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub created_at: Option<DateTime<Utc>>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

/// Request to update an existing dataset.
///
/// # API Reference
///
/// Request body for `PATCH /datasets/{id}`
///
/// # Note
///
/// All fields are optional for partial updates.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DatasetUpdate {
    /// New name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// New description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// New data type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,

    /// New input schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs_schema_definition: Option<Value>,

    /// New output schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs_schema_definition: Option<Value>,

    /// New transformations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformations: Option<Vec<DatasetTransformation>>,

    /// New metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Dataset version information.
///
/// Used for versioning and time-travel queries.
///
/// # API Reference
///
/// Maps to `DatasetVersion` in OpenAPI spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetVersion {
    /// Version tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Point in time for this version
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub as_of: DateTime<Utc>,
}

/// Dataset diff information.
///
/// Shows changes between dataset versions.
///
/// # API Reference
///
/// Maps to `DatasetDiffInfo` in OpenAPI spec.
#[derive(Debug, Clone, Deserialize)]
pub struct DatasetDiffInfo {
    /// IDs of added examples
    pub examples_added: Vec<Uuid>,

    /// IDs of modified examples
    pub examples_modified: Vec<Uuid>,

    /// IDs of removed examples
    pub examples_removed: Vec<Uuid>,
}

// ============================================================================
// Example Types
// ============================================================================

/// Example schema (response type).
///
/// Represents a single example in a dataset.
///
/// # API Reference
///
/// Maps to `Example` in OpenAPI spec.
/// Returned by `GET /examples`, `GET /examples/{id}`
///
/// # Required Fields
///
/// - `id`, `dataset_id`, `inputs`, `name`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Unique identifier for the example
    pub id: Uuid,

    /// Dataset this example belongs to
    pub dataset_id: Uuid,

    /// Input data (required, arbitrary JSON object)
    pub inputs: Value,

    /// Name/identifier for this example
    pub name: String,

    /// Expected output data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Value>,

    /// Source run ID (if created from a run)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<Uuid>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// When the example was created
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub created_at: Option<DateTime<Utc>>,

    /// When the example was last modified
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub modified_at: Option<DateTime<Utc>>,

    /// Attachment URLs (presigned URLs for binary data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_urls: Option<Value>,
}

/// Request to create a new example.
///
/// # API Reference
///
/// Request body for `POST /examples`
///
/// # Example
///
/// ```
/// use langstar_sdk::datasets::ExampleCreate;
/// use serde_json::json;
/// use uuid::Uuid;
///
/// let request = ExampleCreate {
///     dataset_id: Uuid::new_v4(),
///     inputs: Some(json!({"question": "What is 2+2?"})),
///     outputs: Some(json!({"answer": "4"})),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct ExampleCreate {
    /// Dataset to add the example to (required)
    pub dataset_id: Uuid,

    /// Input data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Value>,

    /// Expected output data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Value>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Split name(s) for this example (default: "base")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<ExampleSplit>,

    /// Optional client-provided ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Source run to copy data from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<Uuid>,

    /// Copy inputs/outputs from source run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_source_run_io: Option<bool>,

    /// Attachment names to copy from source run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_source_run_attachments: Option<Vec<String>>,

    /// Use legacy message format for LLM data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_legacy_message_format: Option<bool>,

    /// Custom creation timestamp
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub created_at: Option<DateTime<Utc>>,
}

/// Split specification for examples.
///
/// Can be a single split name or multiple split names.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExampleSplit {
    /// Single split name
    Single(String),
    /// Multiple split names
    Multiple(Vec<String>),
}

/// Operations on example attachments.
///
/// Used when updating examples to rename or retain attachments.
///
/// # API Reference
///
/// Maps to `AttachmentsOperations` in OpenAPI spec.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttachmentsOperations {
    /// Mapping of old attachment names to new names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<HashMap<String, String>>,

    /// List of attachment names to keep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retain: Option<Vec<String>>,
}

/// Request to update an existing example.
///
/// # API Reference
///
/// Request body for `PATCH /examples/{id}`
///
/// # Note
///
/// All fields are optional for partial updates.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ExampleUpdate {
    /// Move to different dataset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<Uuid>,

    /// New input data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Value>,

    /// New output data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Value>,

    /// New metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// New split assignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split: Option<ExampleSplit>,

    /// Attachment operations (rename/retain)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments_operations: Option<AttachmentsOperations>,

    /// Overwrite existing data (false = merge)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
}

/// Request to update multiple examples in bulk.
///
/// # API Reference
///
/// Request body for `PATCH /examples/bulk`
#[derive(Debug, Clone, Serialize)]
pub struct ExampleBulkUpdate {
    /// Example ID to update
    pub id: Uuid,

    /// Update payload
    #[serde(flatten)]
    pub update: ExampleUpdate,
}

// ============================================================================
// Query Parameter Types
// ============================================================================

/// Sort column for listing datasets.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortByDatasetColumn {
    /// Sort by name
    Name,
    /// Sort by creation time
    CreatedAt,
    /// Sort by last modified time
    ModifiedAt,
    /// Sort by example count
    ExampleCount,
}

/// Order for listing examples.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExampleListOrder {
    /// Order by creation time
    CreatedAt,
    /// Order by last modified time
    ModifiedAt,
}

/// Field selection for examples.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExampleSelect {
    /// Select ID
    Id,
    /// Select inputs
    Inputs,
    /// Select outputs
    Outputs,
    /// Select metadata
    Metadata,
    /// Select creation time
    CreatedAt,
    /// Select modification time
    ModifiedAt,
    /// Select dataset ID
    DatasetId,
    /// Select source run ID
    SourceRunId,
    /// Select name
    Name,
}

/// Query parameters for listing datasets.
///
/// # API Reference
///
/// Query parameters for `GET /datasets`
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListDatasetsParams {
    /// Filter by specific dataset IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<Uuid>>,

    /// Filter by data type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,

    /// Filter by exact name match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Filter by name substring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,

    /// Filter by metadata (JSON string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,

    /// Pagination offset (default: 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,

    /// Max results per page (max: 100, default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// Sort field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortByDatasetColumn>,

    /// Sort descending (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by_desc: Option<bool>,

    /// Filter by tag value IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_value_id: Option<Vec<Uuid>>,

    /// Exclude corrections datasets (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_corrections_datasets: Option<bool>,
}

/// Query parameters for listing examples.
///
/// # API Reference
///
/// Query parameters for `GET /examples`
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListExamplesParams {
    /// Parent dataset ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<Uuid>,

    /// Filter by specific example IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<Uuid>>,

    /// Point-in-time query (datetime or "latest")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,

    /// Filter by metadata (JSON string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,

    /// Full-text search (array of terms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_contains: Option<Vec<String>>,

    /// Filter by splits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub splits: Option<Vec<String>>,

    /// Advanced filter expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    /// Pagination offset (default: 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,

    /// Max results per page (max: 100, default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,

    /// Sort field
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<ExampleListOrder>,

    /// Sort descending
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descending: Option<bool>,

    /// Field selection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<Vec<ExampleSelect>>,

    /// Random sampling seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub random_seed: Option<f64>,
}

/// Dataset share information.
///
/// # API Reference
///
/// Returned by `GET/PUT /datasets/{id}/share`
#[derive(Debug, Clone, Deserialize)]
pub struct DatasetShareSchema {
    /// Dataset ID
    pub dataset_id: Uuid,
    /// Share token
    pub share_token: Uuid,
    /// Public URL for the shared dataset
    pub url: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_serialization() {
        let kv = DataType::Kv;
        let json = serde_json::to_string(&kv).unwrap();
        assert_eq!(json, "\"kv\"");

        let llm = DataType::Llm;
        let json = serde_json::to_string(&llm).unwrap();
        assert_eq!(json, "\"llm\"");

        let chat = DataType::Chat;
        let json = serde_json::to_string(&chat).unwrap();
        assert_eq!(json, "\"chat\"");
    }

    #[test]
    fn test_data_type_deserialization() {
        let kv: DataType = serde_json::from_str("\"kv\"").unwrap();
        assert_eq!(kv, DataType::Kv);

        let llm: DataType = serde_json::from_str("\"llm\"").unwrap();
        assert_eq!(llm, DataType::Llm);

        let chat: DataType = serde_json::from_str("\"chat\"").unwrap();
        assert_eq!(chat, DataType::Chat);
    }

    #[test]
    fn test_transformation_type_serialization() {
        let t = DatasetTransformationType::ConvertToOpenaiMessage;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"convert_to_openai_message\"");

        let t = DatasetTransformationType::RemoveSystemMessages;
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"remove_system_messages\"");
    }

    #[test]
    fn test_dataset_deserialization() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Test Dataset",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "example_count": 100,
            "session_count": 5,
            "modified_at": "2024-01-01T12:00:00Z",
            "description": "A test dataset",
            "data_type": "chat"
        }"#;

        let dataset: Dataset = serde_json::from_str(json).unwrap();
        assert_eq!(dataset.name, "Test Dataset");
        assert_eq!(dataset.example_count, Some(100));
        assert_eq!(dataset.session_count, Some(5));
        assert_eq!(dataset.data_type, Some(DataType::Chat));
        assert_eq!(dataset.description, Some("A test dataset".to_string()));
    }

    #[test]
    fn test_dataset_create_serialization() {
        let request = DatasetCreate {
            name: "New Dataset".to_string(),
            description: Some("Test description".to_string()),
            data_type: Some(DataType::Llm),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"New Dataset\""));
        assert!(json.contains("\"description\":\"Test description\""));
        assert!(json.contains("\"data_type\":\"llm\""));
    }

    #[test]
    fn test_dataset_create_omits_none_fields() {
        let request = DatasetCreate {
            name: "Minimal Dataset".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Minimal Dataset\""));
        assert!(!json.contains("description"));
        assert!(!json.contains("data_type"));
    }

    #[test]
    fn test_example_deserialization() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "dataset_id": "87654321-4321-4321-4321-210987654321",
            "inputs": {"question": "What is 2+2?"},
            "name": "math-example-1",
            "outputs": {"answer": "4"},
            "created_at": "2024-01-01T12:00:00Z"
        }"#;

        let example: Example = serde_json::from_str(json).unwrap();
        assert_eq!(example.name, "math-example-1");
        assert_eq!(example.inputs["question"], "What is 2+2?");
        assert_eq!(example.outputs.as_ref().unwrap()["answer"], "4");
    }

    #[test]
    fn test_example_create_serialization() {
        let request = ExampleCreate {
            dataset_id: Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap(),
            inputs: Some(serde_json::json!({"input": "test"})),
            outputs: Some(serde_json::json!({"output": "result"})),
            split: Some(ExampleSplit::Single("train".to_string())),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"dataset_id\":\"12345678-1234-1234-1234-123456789012\""));
        assert!(json.contains("\"split\":\"train\""));
    }

    #[test]
    fn test_example_split_serialization() {
        let single = ExampleSplit::Single("base".to_string());
        let json = serde_json::to_string(&single).unwrap();
        assert_eq!(json, "\"base\"");

        let multiple = ExampleSplit::Multiple(vec!["train".to_string(), "test".to_string()]);
        let json = serde_json::to_string(&multiple).unwrap();
        assert_eq!(json, "[\"train\",\"test\"]");
    }

    #[test]
    fn test_attachments_operations_serialization() {
        let ops = AttachmentsOperations {
            rename: Some(HashMap::from([(
                "old.txt".to_string(),
                "new.txt".to_string(),
            )])),
            retain: Some(vec!["keep.txt".to_string()]),
        };

        let json = serde_json::to_string(&ops).unwrap();
        assert!(json.contains("\"rename\""));
        assert!(json.contains("\"retain\""));
    }

    #[test]
    fn test_dataset_transformation_serialization() {
        let transformation = DatasetTransformation {
            path: vec!["inputs".to_string(), "messages".to_string()],
            transformation_type: DatasetTransformationType::ConvertToOpenaiMessage,
        };

        let json = serde_json::to_string(&transformation).unwrap();
        assert!(json.contains("\"path\":[\"inputs\",\"messages\"]"));
        assert!(json.contains("\"transformation_type\":\"convert_to_openai_message\""));
    }

    #[test]
    fn test_dataset_version_serialization() {
        let version = DatasetVersion {
            tags: Some(vec!["v1.0".to_string()]),
            as_of: DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let json = serde_json::to_string(&version).unwrap();
        assert!(json.contains("\"tags\":[\"v1.0\"]"));
        assert!(json.contains("\"as_of\":"));
    }
}
