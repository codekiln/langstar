//! Project types for LangSmith projects (sessions) API.
//!
//! This module provides types for managing projects in LangSmith.
//! Projects (called "sessions" at the API level) are containers for organizing
//! traces and runs from your applications.
//!
//! # API Reference
//!
//! - Project endpoints: `/api/v1/sessions`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//!
//! # Terminology Note
//!
//! The LangSmith API uses "sessions" terminology in endpoints and schemas,
//! but projects are referred to as "projects" in:
//! - The LangSmith UI
//! - Python/JS SDKs (e.g., `list_projects()`, `create_project()`)
//! - Environment variables (`LANGSMITH_PROJECT`)
//! - Documentation and examples
//!
//! This SDK follows the Python SDK convention of using "project" in public APIs
//! while mapping to the underlying "sessions" REST endpoints.
//!
//! # Example
//!
//! ```no_run
//! use langstar_sdk::projects::ProjectCreate;
//!
//! let request = ProjectCreate {
//!     name: Some("my-application".to_string()),
//!     description: Some("Production traces".to_string()),
//!     ..Default::default()
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::serde_utils::deserialize_flexible_datetime_opt;

// ============================================================================
// Project Types
// ============================================================================

/// Trace tier for project storage configuration.
///
/// Determines data retention and storage characteristics.
///
/// # API Reference
///
/// OpenAPI enum: `["longlived", "shortlived"]`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TraceTier {
    /// Long-lived traces with extended retention
    Longlived,
    /// Short-lived traces with shorter retention
    Shortlived,
}

/// Project schema (response type).
///
/// Represents a project as returned by the API.
/// Projects are containers for organizing traces and runs.
///
/// # API Reference
///
/// Maps to `TracerSession` in OpenAPI spec.
/// Returned by `GET /sessions`, `GET /sessions/{id}`, `POST /sessions`
///
/// # Required Fields
///
/// Only `id` and `tenant_id` are required by the API.
///
/// # Virtual/Computed Fields
///
/// Many fields are computed by the server and only present in GET responses:
/// - `run_count`, `latency_p50`, `latency_p99` (statistics)
/// - `total_tokens`, `total_cost` (aggregated metrics)
/// - `last_run_start_time` (derived from runs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier for the project
    pub id: Uuid,

    /// Tenant ID (workspace) that owns this project
    pub tenant_id: Uuid,

    /// Project name (optional but commonly set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When the project started (first trace)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub start_time: Option<DateTime<Utc>>,

    /// When the project ended (closed/archived)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub end_time: Option<DateTime<Utc>>,

    /// Additional metadata (tags, custom fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,

    /// Reference dataset for evaluations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_dataset_id: Option<Uuid>,

    /// Default dataset for this project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset_id: Option<Uuid>,

    /// Trace storage tier configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_tier: Option<TraceTier>,

    /// Number of runs in this project (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_count: Option<i64>,

    /// Median latency (p50) in milliseconds (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_p50: Option<f64>,

    /// 99th percentile latency in milliseconds (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_p99: Option<f64>,

    /// Median time to first token in milliseconds (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_token_p50: Option<f64>,

    /// 99th percentile time to first token in milliseconds (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_token_p99: Option<f64>,

    /// Total token count across all runs (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i64>,

    /// Total prompt tokens (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<i64>,

    /// Total completion tokens (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<i64>,

    /// Total cost as string (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost: Option<String>,

    /// Prompt cost as string (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cost: Option<String>,

    /// Completion cost as string (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_cost: Option<String>,

    /// When the last run started (computed field)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub last_run_start_time: Option<DateTime<Utc>>,

    /// When the last live run started (computed field)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub last_run_start_time_live: Option<DateTime<Utc>>,

    /// Feedback statistics (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_stats: Option<Value>,

    /// Session-level feedback statistics (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_feedback_stats: Option<Value>,

    /// Run facets for filtering (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_facets: Option<Vec<Value>>,

    /// Error rate (0.0 to 1.0) (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_rate: Option<f64>,

    /// Streaming rate (0.0 to 1.0) (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming_rate: Option<f64>,

    /// Test run number (computed field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_run_number: Option<i64>,
}

/// Request to create a new project.
///
/// # API Reference
///
/// Request body for `POST /sessions`
///
/// # Example
///
/// ```
/// use langstar_sdk::projects::ProjectCreate;
///
/// let request = ProjectCreate {
///     name: Some("my-app-production".to_string()),
///     description: Some("Production environment traces".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct ProjectCreate {
    /// Project name (recommended)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional client-provided ID (UUID generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,

    /// Project start time (usually auto-set by server)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub start_time: Option<DateTime<Utc>>,

    /// Project end time (for creating closed projects)
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub end_time: Option<DateTime<Utc>>,

    /// Additional metadata (tags, custom fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,

    /// Reference dataset for evaluations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_dataset_id: Option<Uuid>,

    /// Default dataset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset_id: Option<Uuid>,

    /// Trace storage tier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_tier: Option<TraceTier>,
}

/// Request to update an existing project.
///
/// # API Reference
///
/// Request body for `PATCH /sessions/{id}`
///
/// # Note
///
/// All fields are optional for partial updates.
/// According to Python SDK: name changes only allowed if project has end_time (is closed).
#[derive(Debug, Clone, Serialize, Default)]
pub struct ProjectUpdate {
    /// New name (only if project is closed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// New description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Close the project by setting end_time
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_flexible_datetime_opt"
    )]
    pub end_time: Option<DateTime<Utc>>,

    /// Update metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,

    /// Update default dataset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_dataset_id: Option<Uuid>,

    /// Update trace tier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_tier: Option<TraceTier>,
}

// ============================================================================
// Query Parameter Types
// ============================================================================

/// Sort column for listing projects.
///
/// # API Reference
///
/// OpenAPI enum: `SessionSortableColumns`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSortColumn {
    /// Sort by name
    Name,
    /// Sort by start time
    StartTime,
    /// Sort by last run start time
    LastRunStartTime,
    /// Sort by median latency
    LatencyP50,
    /// Sort by 99th percentile latency
    LatencyP99,
    /// Sort by error rate
    ErrorRate,
    /// Sort by feedback
    Feedback,
    /// Sort by run count
    RunsCount,
}

/// Query parameters for listing projects.
///
/// # API Reference
///
/// Query parameters for `GET /sessions`
///
/// # Example
///
/// ```
/// use langstar_sdk::projects::{ListProjectsParams, ProjectSortColumn};
///
/// let params = ListProjectsParams {
///     name_contains: Some("production".to_string()),
///     limit: Some(50),
///     sort_by: Some(ProjectSortColumn::LastRunStartTime),
///     sort_by_desc: Some(true),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListProjectsParams {
    /// Filter by specific project IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<Uuid>>,

    /// Filter by exact name match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Filter by name substring
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,

    /// Filter by reference dataset ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_dataset_id: Option<Vec<Uuid>>,

    /// Only return projects without a reference dataset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_free: Option<bool>,

    /// Include computed statistics (run counts, latencies, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_stats: Option<bool>,

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
    pub sort_by: Option<ProjectSortColumn>,

    /// Sort descending (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by_desc: Option<bool>,

    /// Filter by tag value IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_value_id: Option<Vec<Uuid>>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_tier_serialization() {
        let longlived = TraceTier::Longlived;
        let json = serde_json::to_string(&longlived).unwrap();
        assert_eq!(json, "\"longlived\"");

        let shortlived = TraceTier::Shortlived;
        let json = serde_json::to_string(&shortlived).unwrap();
        assert_eq!(json, "\"shortlived\"");
    }

    #[test]
    fn test_trace_tier_deserialization() {
        let longlived: TraceTier = serde_json::from_str("\"longlived\"").unwrap();
        assert_eq!(longlived, TraceTier::Longlived);

        let shortlived: TraceTier = serde_json::from_str("\"shortlived\"").unwrap();
        assert_eq!(shortlived, TraceTier::Shortlived);
    }

    #[test]
    fn test_project_deserialization_minimal() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "tenant_id": "87654321-4321-4321-4321-210987654321"
        }"#;

        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(
            project.id,
            Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap()
        );
        assert_eq!(
            project.tenant_id,
            Uuid::parse_str("87654321-4321-4321-4321-210987654321").unwrap()
        );
        assert_eq!(project.name, None);
        assert_eq!(project.description, None);
    }

    #[test]
    fn test_project_deserialization_full() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "name": "my-application",
            "description": "Production traces",
            "start_time": "2024-01-01T12:00:00Z",
            "end_time": null,
            "extra": {"environment": "production"},
            "reference_dataset_id": "11111111-1111-1111-1111-111111111111",
            "trace_tier": "longlived",
            "run_count": 1500,
            "latency_p50": 125.5,
            "latency_p99": 500.0,
            "total_tokens": 100000,
            "total_cost": "5.50",
            "error_rate": 0.02,
            "last_run_start_time": "2024-01-15T10:30:00Z"
        }"#;

        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.name, Some("my-application".to_string()));
        assert_eq!(project.description, Some("Production traces".to_string()));
        assert_eq!(project.run_count, Some(1500));
        assert_eq!(project.latency_p50, Some(125.5));
        assert_eq!(project.latency_p99, Some(500.0));
        assert_eq!(project.total_tokens, Some(100000));
        assert_eq!(project.total_cost, Some("5.50".to_string()));
        assert_eq!(project.error_rate, Some(0.02));
        assert_eq!(project.trace_tier, Some(TraceTier::Longlived));
    }

    #[test]
    fn test_project_create_serialization() {
        let request = ProjectCreate {
            name: Some("test-project".to_string()),
            description: Some("Test description".to_string()),
            trace_tier: Some(TraceTier::Shortlived),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"test-project\""));
        assert!(json.contains("\"description\":\"Test description\""));
        assert!(json.contains("\"trace_tier\":\"shortlived\""));
    }

    #[test]
    fn test_project_create_omits_none_fields() {
        let request = ProjectCreate {
            name: Some("minimal-project".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"minimal-project\""));
        assert!(!json.contains("description"));
        assert!(!json.contains("trace_tier"));
        assert!(!json.contains("extra"));
    }

    #[test]
    fn test_project_update_serialization() {
        let request = ProjectUpdate {
            description: Some("Updated description".to_string()),
            extra: Some(serde_json::json!({"tag": "v2"})),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"description\":\"Updated description\""));
        assert!(json.contains("\"extra\""));
        assert!(!json.contains("name")); // name not set, should be omitted
    }

    #[test]
    fn test_project_sort_column_serialization() {
        let col = ProjectSortColumn::LastRunStartTime;
        let json = serde_json::to_string(&col).unwrap();
        assert_eq!(json, "\"last_run_start_time\"");

        let col = ProjectSortColumn::LatencyP99;
        let json = serde_json::to_string(&col).unwrap();
        assert_eq!(json, "\"latency_p99\"");
    }

    #[test]
    fn test_list_projects_params_serialization() {
        let params = ListProjectsParams {
            name_contains: Some("test".to_string()),
            limit: Some(50),
            sort_by: Some(ProjectSortColumn::StartTime),
            sort_by_desc: Some(false),
            include_stats: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"name_contains\":\"test\""));
        assert!(json.contains("\"limit\":50"));
        assert!(json.contains("\"sort_by\":\"start_time\""));
        assert!(json.contains("\"sort_by_desc\":false"));
        assert!(json.contains("\"include_stats\":true"));
    }

    #[test]
    fn test_list_projects_params_omits_none_fields() {
        let params = ListProjectsParams {
            name_contains: Some("prod".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"name_contains\":\"prod\""));
        assert!(!json.contains("limit"));
        assert!(!json.contains("sort_by"));
        assert!(!json.contains("include_stats"));
    }

    #[test]
    fn test_project_with_metadata() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "tenant_id": "87654321-4321-4321-4321-210987654321",
            "name": "prod-app",
            "extra": {
                "environment": "production",
                "version": "2.1.0",
                "tags": ["critical", "monitored"]
            }
        }"#;

        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.name, Some("prod-app".to_string()));
        assert!(project.extra.is_some());

        let extra = project.extra.unwrap();
        assert_eq!(extra["environment"], "production");
        assert_eq!(extra["version"], "2.1.0");
    }
}
