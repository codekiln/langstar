//! Annotation queue types for LangSmith annotation queues API.
//!
//! This module provides types for managing annotation queues in LangSmith.
//! Annotation queues are used to organize runs for human review and annotation.
//!
//! # API Reference
//!
//! - Base endpoint: `/api/v1/annotation-queues`
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

use crate::serde_utils::{deserialize_flexible_datetime, deserialize_flexible_datetime_opt};

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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub created_at: DateTime<Utc>,

    /// When the queue was last updated
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub updated_at: DateTime<Utc>,

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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
/// Extends the base Run with queue-specific fields like when it was added
/// and when it was last reviewed.
///
/// # API Reference
///
/// Maps to `RunSchemaWithAnnotationQueueInfo` in OpenAPI spec.
/// Returned by `GET /annotation-queues/{queue_id}/run/{index}`
///
/// # Note
///
/// Uses snake_case field names to match existing Run struct pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunWithAnnotationQueueInfo {
    /// Base run fields
    #[serde(flatten)]
    pub run: super::runs::Run,

    /// Queue run ID (junction table ID)
    pub queue_run_id: Uuid,

    /// When the run was last reviewed
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub last_reviewed_time: Option<DateTime<Utc>>,

    /// When the run was added to the queue
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub added_at: Option<DateTime<Utc>>,

    /// Effective added time (for sorting/display)
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub effective_added_at: Option<DateTime<Utc>>,
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
            "tenantId": "87654321-4321-4321-4321-210987654321",
            "queueType": "single",
            "description": "Queue for reviews",
            "createdAt": "2024-01-01T12:00:00Z",
            "updatedAt": "2024-01-01T12:00:00Z",
            "numReviewersPerItem": 2,
            "enableReservations": true,
            "reservationMinutes": 5
        }"#;

        let queue: AnnotationQueue = serde_json::from_str(json).unwrap();
        assert_eq!(queue.name, "Review Queue");
        assert_eq!(queue.queue_type, QueueType::Single);
        assert_eq!(queue.num_reviewers_per_item, Some(2));
        assert_eq!(queue.enable_reservations, Some(true));
        assert_eq!(queue.reservation_minutes, Some(5));
    }

    #[test]
    fn test_annotation_queue_with_details_deserialization() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Review Queue",
            "tenantId": "87654321-4321-4321-4321-210987654321",
            "queueType": "pairwise",
            "createdAt": "2024-01-01T12:00:00Z",
            "updatedAt": "2024-01-01T12:00:00Z",
            "rubricInstructions": "Rate on accuracy and helpfulness",
            "rubricItems": [
                {
                    "feedbackKey": "accuracy",
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
        assert!(json.contains("\"queueType\":\"single\""));
        assert!(json.contains("\"rubricInstructions\":\"Test rubric\""));
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
        assert!(json.contains("\"nameContains\":\"review\""));
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
        assert!(json.contains("\"feedbackKey\":\"accuracy\""));
        assert!(json.contains("\"description\":\"Rate accuracy\""));
    }

    #[test]
    fn test_run_with_annotation_queue_info_deserialization() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "llm",
            "trace_id": "223e4567-e89b-12d3-a456-426614174001",
            "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
            "status": "success",
            "session_id": "323e4567-e89b-12d3-a456-426614174002",
            "app_path": "/chat",
            "queue_run_id": "423e4567-e89b-12d3-a456-426614174003",
            "last_reviewed_time": "2024-01-01T13:00:00Z",
            "added_at": "2024-01-01T12:00:00Z",
            "effective_added_at": "2024-01-01T12:00:00Z"
        }"#;

        let run: RunWithAnnotationQueueInfo = serde_json::from_str(json).unwrap();
        assert_eq!(run.run.name, "ChatOpenAI");
        assert!(run.last_reviewed_time.is_some());
        assert!(run.added_at.is_some());
        assert!(run.effective_added_at.is_some());
    }
}
