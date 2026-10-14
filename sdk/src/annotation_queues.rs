//! Annotation queue types for LangSmith annotation queues API.
//!
//! This module provides types for managing annotation queues in LangSmith.
//! Annotation queues are used to organize runs for human review and annotation.
//!
//! # API Reference
//!
//! - Queue endpoints: `/api/v1/annotation-queues`
//! - Item endpoints: `/api/v1/platform/annotation-queues/{queue_id}/items`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//!
//! # Example
//!
//! ```no_run
//! use langstar_sdk::annotation_queues::{CreateAnnotationQueueRequest, QueueType};
//!
//! // Create a queue request
//! let request = CreateAnnotationQueueRequest {
//!     name: "Review Queue".to_string(),
//!     description: Some("Queue for reviewing production runs".to_string()),
//!     queue_type: Some(QueueType::Single),
//!     ..Default::default()
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::serde_utils::deserialize_flexible_datetime_opt;

/// Queue type enum for annotation queues.
///
/// # API Reference
///
/// OpenAPI enum values: `["single", "pairwise"]`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QueueType {
    /// Single-run annotation queue (review one run at a time)
    Single,
    /// Pairwise comparison queue (compare two runs side-by-side)
    Pairwise,
}

/// Annotation queue rubric item schema.
///
/// Defines a single rubric criterion for evaluating runs in the queue.
///
/// # API Reference
///
/// Maps to `AnnotationQueueRubricItemSchema` in OpenAPI spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueueRubricItem {
    /// Feedback key identifier
    pub feedback_key: String,

    /// Description of this rubric item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Descriptions for specific categorical values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_descriptions: Option<Value>,

    /// Descriptions for specific numerical scores
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_descriptions: Option<Value>,
}

/// Base annotation queue schema.
///
/// Represents the core metadata and configuration for an annotation queue.
///
/// # API Reference
///
/// Maps to `AnnotationQueueSchema` in OpenAPI spec.
/// Returned by `GET /annotation-queues` and `GET /annotation-queues/{id}`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueue {
    /// Unique identifier for the queue
    pub id: Uuid,

    /// Name of the queue
    pub name: String,

    /// Tenant ID (workspace) that owns this queue
    pub tenant_id: Uuid,

    /// Queue type: single or pairwise comparison
    pub queue_type: QueueType,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When the queue was created
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub created_at: Option<DateTime<Utc>>,

    /// When the queue was last updated
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub updated_at: Option<DateTime<Utc>>,

    /// Number of reviewers required per item (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_reviewers_per_item: Option<i32>,

    /// Whether to enable reservation system (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_reservations: Option<bool>,

    /// Reservation duration in minutes (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_minutes: Option<i32>,

    /// Source rule ID for auto-population
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_rule_id: Option<Uuid>,

    /// Run rule ID for filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_rule_id: Option<Uuid>,

    /// Default dataset ID for this queue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset: Option<Uuid>,

    /// Additional metadata (arbitrary JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Annotation queue with rubric details.
///
/// Extends the base queue with rubric instructions and items.
///
/// # API Reference
///
/// Maps to `AnnotationQueueSchemaWithRubric` in OpenAPI spec.
/// Returned by `GET /annotation-queues/{id}` and `POST /annotation-queues`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueueWithDetails {
    /// Base queue fields
    #[serde(flatten)]
    pub base: AnnotationQueue,

    /// Rubric instructions for annotators (markdown/text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_instructions: Option<String>,

    /// Structured rubric items defining evaluation criteria
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_items: Option<Vec<AnnotationQueueRubricItem>>,
}

/// Request to create a new annotation queue.
///
/// # API Reference
///
/// Request body for `POST /annotation-queues`
/// Maps to `AnnotationQueueCreateSchema` in OpenAPI spec.
///
/// # Example
///
/// ```
/// use langstar_sdk::annotation_queues::{CreateAnnotationQueueRequest, QueueType};
///
/// let request = CreateAnnotationQueueRequest {
///     name: "Production Review".to_string(),
///     description: Some("Review production LLM outputs".to_string()),
///     queue_type: Some(QueueType::Single),
///     rubric_instructions: Some("Rate accuracy and helpfulness".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateAnnotationQueueRequest {
    /// Name of the queue (required)
    pub name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional client-provided ID (UUID generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Queue type (default: single)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue_type: Option<QueueType>,

    /// When the queue was created (usually auto-set by server)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub created_at: Option<DateTime<Utc>>,

    /// When the queue was last updated (usually auto-set by server)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub updated_at: Option<DateTime<Utc>>,

    /// Number of reviewers per item (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_reviewers_per_item: Option<i32>,

    /// Enable reservation system (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_reservations: Option<bool>,

    /// Reservation duration in minutes (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_minutes: Option<i32>,

    /// Default dataset for runs in this queue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset: Option<Uuid>,

    /// Structured rubric items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_items: Option<Vec<AnnotationQueueRubricItem>>,

    /// Rubric instructions (markdown/text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_instructions: Option<String>,

    /// Session/project IDs to auto-populate from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_ids: Option<Vec<Uuid>>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Request to update an annotation queue.
///
/// # API Reference
///
/// Request body for `PATCH /annotation-queues/{id}`
/// Maps to `AnnotationQueueUpdateSchema` in OpenAPI spec.
///
/// # Note
///
/// All fields are optional for partial updates.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateAnnotationQueueRequest {
    /// New name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// New description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// New default dataset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset: Option<Uuid>,

    /// New number of reviewers per item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_reviewers_per_item: Option<i32>,

    /// Enable/disable reservations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_reservations: Option<bool>,

    /// New reservation duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservation_minutes: Option<i32>,

    /// New rubric items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_items: Option<Vec<AnnotationQueueRubricItem>>,

    /// New rubric instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rubric_instructions: Option<String>,

    /// New metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Query parameters for listing annotation queues.
///
/// # API Reference
///
/// Query parameters for `GET /annotation-queues`
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

/// Kind of thing an annotation queue item points at.
///
/// Maps to `annotationqueues.AnnotationQueueItemType` in the OpenAPI spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AnnotationQueueItemType {
    /// A single run (trace)
    Run,
    /// A conversation thread
    Thread,
}

impl AnnotationQueueItemType {
    /// The wire value LangSmith expects in the `item_type` query parameter.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Run => "RUN",
            Self::Thread => "THREAD",
        }
    }
}

/// Review section of an annotation queue, as the list and count endpoints take it.
///
/// Maps to `annotationqueues.AnnotationQueueItemListStatus` in the OpenAPI spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationQueueItemStatus {
    /// Waiting for the calling reviewer
    NeedsMyReview,
    /// Waiting for other reviewers
    NeedsOthersReview,
    /// Fully reviewed
    Archived,
}

impl AnnotationQueueItemStatus {
    /// Every section, in the order a queue shows them.
    pub const ALL: [Self; 3] = [Self::NeedsMyReview, Self::NeedsOthersReview, Self::Archived];

    /// The wire value LangSmith expects in the `status` query parameter.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NeedsMyReview => "needs_my_review",
            Self::NeedsOthersReview => "needs_others_review",
            Self::Archived => "archived",
        }
    }
}

/// Bucket to count with `GET .../items/count`: one review section, or every item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationQueueItemCountStatus {
    /// Items in one review section
    Section(AnnotationQueueItemStatus),
    /// Every item in the queue
    All,
}

impl AnnotationQueueItemCountStatus {
    /// The wire value LangSmith expects in the `status` query parameter.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Section(status) => status.as_str(),
            Self::All => "all",
        }
    }
}

impl From<AnnotationQueueItemStatus> for AnnotationQueueItemCountStatus {
    fn from(status: AnnotationQueueItemStatus) -> Self {
        Self::Section(status)
    }
}

/// One item to add to an annotation queue.
///
/// Maps to `annotationqueues.AnnotationQueueItemInput`. A `RUN` item needs `run_id`;
/// `project_id` and `start_time` locate the run directly, as langsmith-sdk's `RunKey` does.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AnnotationQueueItemInput {
    /// What the item points at
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<AnnotationQueueItemType>,

    /// Run to review (for `RUN` items)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,

    /// Tracing project the run belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,

    /// When the run started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,

    /// Thread to review (for `THREAD` items)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// Suggested example the item was created from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_proposed_example_id: Option<Uuid>,
}

/// An item as the add and patch endpoints return it.
///
/// Maps to `annotationqueues.AnnotationQueueItem`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueueItem {
    /// Queue item ID (not the run ID)
    pub id: Uuid,

    /// Queue the item belongs to
    #[serde(default)]
    pub queue_id: Option<Uuid>,

    /// What the item points at
    #[serde(default)]
    pub item_type: Option<AnnotationQueueItemType>,

    /// Run under review, for `RUN` items
    #[serde(default, deserialize_with = "deserialize_nonempty_uuid_opt")]
    pub run_id: Option<Uuid>,

    /// Tracing project of the run
    #[serde(default, deserialize_with = "deserialize_nonempty_uuid_opt")]
    pub project_id: Option<Uuid>,

    /// Thread under review, for `THREAD` items
    #[serde(default)]
    pub thread_id: Option<String>,

    /// Suggested example the item was created from
    #[serde(default, deserialize_with = "deserialize_nonempty_uuid_opt")]
    pub source_proposed_example_id: Option<Uuid>,

    /// When the run started
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub start_time: Option<DateTime<Utc>>,

    /// When the item was added to the queue
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub added_at: Option<DateTime<Utc>>,

    /// When the item was last reviewed; null until reviewed
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub last_reviewed_time: Option<DateTime<Utc>>,
}

/// An item as the list endpoint returns it, with reviewer state.
///
/// Maps to `annotationqueues.AnnotationQueueListItem`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueueListItem {
    /// Queue item ID (not the run ID)
    pub id: Uuid,

    /// Queue the item belongs to
    #[serde(default)]
    pub queue_id: Option<Uuid>,

    /// What the item points at
    #[serde(default)]
    pub item_type: Option<AnnotationQueueItemType>,

    /// Run under review, for `RUN` items
    #[serde(default, deserialize_with = "deserialize_nonempty_uuid_opt")]
    pub run_id: Option<Uuid>,

    /// Tracing project of the run
    #[serde(default, deserialize_with = "deserialize_nonempty_uuid_opt")]
    pub project_id: Option<Uuid>,

    /// Thread under review, for `THREAD` items
    #[serde(default)]
    pub thread_id: Option<String>,

    /// Suggested example the item was created from
    #[serde(default, deserialize_with = "deserialize_nonempty_uuid_opt")]
    pub source_proposed_example_id: Option<Uuid>,

    /// When the run started
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub start_time: Option<DateTime<Utc>>,

    /// When the item was added to the queue
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub added_at: Option<DateTime<Utc>>,

    /// Added time used for ordering
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub effective_added_at: Option<DateTime<Utc>>,

    /// When the item was last reviewed; null until reviewed
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub last_reviewed_time: Option<DateTime<Utc>>,

    /// Identities currently reviewing the item
    #[serde(default)]
    pub reserved_by: Option<Vec<String>>,

    /// Identities that finished reviewing the item
    #[serde(default)]
    pub completed_by: Option<Vec<String>>,
}

/// One page of queue items.
///
/// Maps to `annotationqueues.ListAnnotationQueueItemsResponse`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListAnnotationQueueItemsResponse {
    /// Items on this page
    #[serde(default)]
    pub items: Vec<AnnotationQueueListItem>,

    /// Cursor for the next page, if there is one
    #[serde(default)]
    pub next_cursor: Option<String>,

    /// Cursor for the previous page, if there is one
    #[serde(default)]
    pub previous_cursor: Option<String>,
}

/// Query parameters for `GET /api/v1/platform/annotation-queues/{queue_id}/items`, which
/// lists the items of one review section of an annotation queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListAnnotationQueueItemsParams {
    /// Review section to list; the API requires one
    pub status: AnnotationQueueItemStatus,

    /// Items per page (the API defaults to 20)
    pub page_size: Option<u32>,

    /// Only items of this type
    pub item_type: Option<AnnotationQueueItemType>,
}

impl ListAnnotationQueueItemsParams {
    /// Params for one review section, with the API's defaults for everything else.
    pub fn new(status: AnnotationQueueItemStatus) -> Self {
        Self {
            status,
            page_size: None,
            item_type: None,
        }
    }
}

/// Body for `PATCH /api/v1/platform/annotation-queues/{queue_id}/items/{item_id}`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PatchAnnotationQueueItemRequest {
    /// Move the item to this added time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_at: Option<DateTime<Utc>>,

    /// Set the last reviewed time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_reviewed_time: Option<DateTime<Utc>>,
}

/// Where an item sits in its queue.
///
/// Maps to `annotationqueues.AnnotationQueueItemPlacementResponse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationQueueItemPlacement {
    /// Section the item is in
    #[serde(default)]
    pub section: Option<AnnotationQueueItemStatus>,

    /// Zero-based position within the section
    #[serde(default)]
    pub position: Option<u64>,

    /// Cursor that lists the page holding the item
    #[serde(default)]
    pub cursor: Option<String>,

    /// What the item points at
    #[serde(default)]
    pub item_type: Option<AnnotationQueueItemType>,
}

#[derive(Deserialize)]
pub(crate) struct AnnotationQueueItemCountResponse {
    #[serde(default)]
    pub(crate) count: u64,
}

/// The items endpoints send `""` for an id that does not apply to the item type.
fn deserialize_nonempty_uuid_opt<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    match value.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => Uuid::parse_str(s)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_type_serialization() {
        let single = QueueType::Single;
        let json = serde_json::to_string(&single).unwrap();
        assert_eq!(json, "\"single\"");

        let pairwise = QueueType::Pairwise;
        let json = serde_json::to_string(&pairwise).unwrap();
        assert_eq!(json, "\"pairwise\"");
    }

    #[test]
    fn test_queue_type_deserialization() {
        let single: QueueType = serde_json::from_str("\"single\"").unwrap();
        assert_eq!(single, QueueType::Single);

        let pairwise: QueueType = serde_json::from_str("\"pairwise\"").unwrap();
        assert_eq!(pairwise, QueueType::Pairwise);
    }

    #[test]
    fn test_annotation_queue_deserialization() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Review Queue",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "queue_type": "single",
            "description": "Queue for reviews",
            "created_at": "2024-01-01T12:00:00Z",
            "updated_at": "2024-01-01T12:00:00Z",
            "num_reviewers_per_item": 2,
            "enable_reservations": true,
            "reservation_minutes": 5
        }"#;

        let queue: AnnotationQueue = serde_json::from_str(json).unwrap();
        assert_eq!(queue.name, "Review Queue");
        assert_eq!(queue.queue_type, QueueType::Single);
        assert!(queue.created_at.is_some());
        assert!(queue.updated_at.is_some());
        assert_eq!(queue.num_reviewers_per_item, Some(2));
        assert_eq!(queue.enable_reservations, Some(true));
        assert_eq!(queue.reservation_minutes, Some(5));
    }

    #[test]
    fn test_annotation_queue_deserialization_without_timestamps() {
        // Test that queues can be deserialized even without created_at/updated_at
        // This was causing the bug in issue #624
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Minimal Queue",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "queue_type": "single"
        }"#;

        let queue: AnnotationQueue = serde_json::from_str(json).unwrap();
        assert_eq!(queue.name, "Minimal Queue");
        assert_eq!(queue.queue_type, QueueType::Single);
        assert!(queue.created_at.is_none());
        assert!(queue.updated_at.is_none());
    }

    #[test]
    fn test_annotation_queue_with_details_deserialization() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Review Queue",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "queue_type": "pairwise",
            "created_at": "2024-01-01T12:00:00Z",
            "updated_at": "2024-01-01T12:00:00Z",
            "rubric_instructions": "Rate on accuracy and helpfulness",
            "rubric_items": [
                {
                    "feedback_key": "accuracy",
                    "description": "How accurate is the response?"
                }
            ]
        }"#;

        let queue: AnnotationQueueWithDetails = serde_json::from_str(json).unwrap();
        assert_eq!(queue.base.name, "Review Queue");
        assert_eq!(queue.base.queue_type, QueueType::Pairwise);
        assert_eq!(
            queue.rubric_instructions,
            Some("Rate on accuracy and helpfulness".to_string())
        );
        assert!(queue.rubric_items.is_some());
        assert_eq!(queue.rubric_items.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_create_annotation_queue_request_serialization() {
        let request = CreateAnnotationQueueRequest {
            name: "Test Queue".to_string(),
            description: Some("Test description".to_string()),
            queue_type: Some(QueueType::Single),
            rubric_instructions: Some("Test rubric".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Test Queue\""));
        assert!(json.contains("\"description\":\"Test description\""));
        assert!(json.contains("\"queue_type\":\"single\""));
        assert!(json.contains("\"rubric_instructions\":\"Test rubric\""));
    }

    #[test]
    fn test_create_annotation_queue_request_omits_none_fields() {
        let request = CreateAnnotationQueueRequest {
            name: "Minimal Queue".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Minimal Queue\""));
        assert!(!json.contains("description"));
        assert!(!json.contains("rubricInstructions"));
    }

    #[test]
    fn test_update_annotation_queue_request_serialization() {
        let request = UpdateAnnotationQueueRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
        assert!(json.contains("\"description\":\"Updated description\""));
    }

    #[test]
    fn test_list_annotation_queues_params_serialization() {
        let params = ListAnnotationQueuesParams {
            name_contains: Some("review".to_string()),
            limit: Some(50),
            ..Default::default()
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"name_contains\":\"review\""));
        assert!(json.contains("\"limit\":50"));
    }

    #[test]
    fn test_rubric_item_serialization() {
        let item = AnnotationQueueRubricItem {
            feedback_key: "accuracy".to_string(),
            description: Some("Rate accuracy".to_string()),
            value_descriptions: None,
            score_descriptions: None,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"feedback_key\":\"accuracy\""));
        assert!(json.contains("\"description\":\"Rate accuracy\""));
    }

    #[test]
    fn test_queue_item_deserialization_blank_ids() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "queue_id": "223e4567-e89b-12d3-a456-426614174001",
            "item_type": "THREAD",
            "run_id": "",
            "thread_id": "thread-1",
            "added_at": "2024-01-01T12:00:00Z",
            "last_reviewed_time": null
        }"#;

        let item: AnnotationQueueItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.item_type, Some(AnnotationQueueItemType::Thread));
        assert!(item.run_id.is_none());
        assert!(item.added_at.is_some());
        assert!(item.last_reviewed_time.is_none());
    }

    #[test]
    fn test_queue_item_input_serialization() {
        let run_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let input = AnnotationQueueItemInput {
            item_type: Some(AnnotationQueueItemType::Run),
            run_id: Some(run_id),
            ..Default::default()
        };
        let value = serde_json::to_value(&input).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"item_type": "RUN", "run_id": run_id.to_string()})
        );
    }

    #[test]
    fn test_item_status_wire_values() {
        for status in AnnotationQueueItemStatus::ALL {
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{}\"", status.as_str())
            );
        }
    }
}
