//! Run types for LangSmith runs/traces API.
//!
//! This module provides types for querying and working with LangSmith runs.
//! Runs represent individual executions in a LangChain application trace.
//!
//! # API Reference
//!
//! - Endpoint: `POST /api/v1/runs/query`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//!
//! # Example
//!
//! ```no_run
//! use langstar_sdk::runs::{QueryRunsRequest, RunType};
//!
//! // Create a query request
//! let request = QueryRunsRequest {
//!     is_root: Some(true),
//!     run_type: Some(RunType::Llm),
//!     limit: Some(10),
//!     ..Default::default()
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Run type enum matching OpenAPI spec `RunTypeEnum`.
///
/// Represents the type of operation a run performed.
///
/// # OpenAPI Reference
///
/// Values: `["tool", "chain", "llm", "retriever", "embedding", "prompt", "parser"]`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    /// Tool execution run
    Tool,
    /// Chain execution run
    Chain,
    /// LLM (Language Model) call run
    Llm,
    /// Retriever execution run
    Retriever,
    /// Embedding operation run
    Embedding,
    /// Prompt template run
    Prompt,
    /// Output parser run
    Parser,
}

/// Run schema based on OpenAPI `RunSchema` spec.
///
/// Represents a single run/trace in LangSmith. Contains all 54 fields
/// from the OpenAPI specification.
///
/// # Required Fields (per OpenAPI spec)
///
/// - `id`, `name`, `run_type`, `trace_id`, `dotted_order`, `status`, `session_id`, `app_path`
///
/// # OpenAPI Reference
///
/// See `/workspace/reference/api-specs/run-schema.json`
#[derive(Debug, Clone, Deserialize)]
pub struct Run {
    // ═══════════════════════════════════════════════════════════════════════
    // Required fields (per OpenAPI spec)
    // ═══════════════════════════════════════════════════════════════════════
    /// Unique identifier for the run
    pub id: Uuid,

    /// Name of the run (typically the component name)
    pub name: String,

    /// Type of run (llm, chain, tool, etc.)
    pub run_type: RunType,

    /// ID of the root trace this run belongs to
    pub trace_id: Uuid,

    /// Dotted order string for hierarchical ordering within trace
    pub dotted_order: String,

    /// Current status of the run (e.g., "success", "error", "pending")
    pub status: String,

    /// Session/project ID this run belongs to
    pub session_id: Uuid,

    /// Application path identifier
    pub app_path: String,

    // ═══════════════════════════════════════════════════════════════════════
    // Timing fields (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// When the run started
    pub start_time: Option<DateTime<Utc>>,

    /// When the run ended
    pub end_time: Option<DateTime<Utc>>,

    /// When the first token was received (for streaming LLM calls)
    pub first_token_time: Option<DateTime<Utc>>,

    /// When the run was last queued
    pub last_queued_at: Option<DateTime<Utc>>,

    // ═══════════════════════════════════════════════════════════════════════
    // Content fields (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// Input data for the run
    pub inputs: Option<Value>,

    /// Output data from the run
    pub outputs: Option<Value>,

    /// Error message if the run failed
    pub error: Option<String>,

    /// Extra metadata for the run
    pub extra: Option<Value>,

    /// Events emitted during the run
    pub events: Option<Vec<Value>>,

    /// Serialized representation of the component
    pub serialized: Option<Value>,

    /// Preview of inputs (truncated string)
    pub inputs_preview: Option<String>,

    /// Preview of outputs (truncated string)
    pub outputs_preview: Option<String>,

    // ═══════════════════════════════════════════════════════════════════════
    // Hierarchy fields (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// ID of the parent run (if not root)
    pub parent_run_id: Option<Uuid>,

    /// IDs of all ancestor runs
    pub parent_run_ids: Option<Vec<Uuid>>,

    /// IDs of all child runs
    pub child_run_ids: Option<Vec<Uuid>>,

    /// IDs of direct child runs only
    pub direct_child_run_ids: Option<Vec<Uuid>>,

    // ═══════════════════════════════════════════════════════════════════════
    // Token fields (with defaults per OpenAPI)
    // ═══════════════════════════════════════════════════════════════════════
    /// Total tokens used (prompt + completion). Defaults to 0.
    #[serde(default)]
    pub total_tokens: i64,

    /// Tokens used in the prompt. Defaults to 0.
    #[serde(default)]
    pub prompt_tokens: i64,

    /// Tokens used in the completion. Defaults to 0.
    #[serde(default)]
    pub completion_tokens: i64,

    /// Detailed breakdown of prompt tokens
    pub prompt_token_details: Option<Value>,

    /// Detailed breakdown of completion tokens
    pub completion_token_details: Option<Value>,

    // ═══════════════════════════════════════════════════════════════════════
    // Cost fields (optional, string format for decimal precision)
    // ═══════════════════════════════════════════════════════════════════════
    /// Total cost as a decimal string
    pub total_cost: Option<String>,

    /// Prompt cost as a decimal string
    pub prompt_cost: Option<String>,

    /// Completion cost as a decimal string
    pub completion_cost: Option<String>,

    /// Detailed breakdown of prompt costs
    pub prompt_cost_details: Option<Value>,

    /// Detailed breakdown of completion costs
    pub completion_cost_details: Option<Value>,

    /// Price model ID used for cost calculation
    pub price_model_id: Option<Uuid>,

    // ═══════════════════════════════════════════════════════════════════════
    // Metadata fields (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// Tags associated with the run
    pub tags: Option<Vec<String>>,

    /// Aggregated feedback statistics
    pub feedback_stats: Option<Value>,

    /// Reference example ID (for evaluation runs)
    pub reference_example_id: Option<Uuid>,

    /// Reference dataset ID (for evaluation runs)
    pub reference_dataset_id: Option<Uuid>,

    // ═══════════════════════════════════════════════════════════════════════
    // Execution fields (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// Execution order within the trace. Defaults to 1.
    #[serde(default = "default_execution_order")]
    pub execution_order: i32,

    /// Whether this run is included in a dataset
    pub in_dataset: Option<bool>,

    // ═══════════════════════════════════════════════════════════════════════
    // Sharing and trace metadata (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// Share token for public sharing
    pub share_token: Option<Uuid>,

    /// Trace tier classification
    pub trace_tier: Option<String>,

    /// Whether trace upgrade is enabled. Defaults to false.
    #[serde(default)]
    pub trace_upgrade: bool,

    /// When the trace was first received
    pub trace_first_received_at: Option<DateTime<Utc>>,

    /// Minimum start time in the trace
    pub trace_min_start_time: Option<DateTime<Utc>>,

    /// Maximum start time in the trace
    pub trace_max_start_time: Option<DateTime<Utc>>,

    /// Thread ID for conversation tracking
    pub thread_id: Option<String>,

    /// Time-to-live in seconds
    pub ttl_seconds: Option<i64>,

    // ═══════════════════════════════════════════════════════════════════════
    // S3 storage fields (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// S3 URLs for inputs
    pub inputs_s3_urls: Option<Value>,

    /// S3 URLs for outputs
    pub outputs_s3_urls: Option<Value>,

    /// General S3 URLs
    pub s3_urls: Option<Value>,

    // ═══════════════════════════════════════════════════════════════════════
    // Manifest fields (optional)
    // ═══════════════════════════════════════════════════════════════════════
    /// Manifest ID for deployment tracking
    pub manifest_id: Option<Uuid>,

    /// Manifest S3 ID
    pub manifest_s3_id: Option<Uuid>,
}

/// Default execution order value (1) per OpenAPI spec.
fn default_execution_order() -> i32 {
    1
}

/// Sort order for run queries.
///
/// # OpenAPI Reference
///
/// Maps to `RunDateOrder` enum with values `["asc", "desc"]`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunDateOrder {
    /// Ascending order (oldest first)
    Asc,
    /// Descending order (newest first, default)
    #[default]
    Desc,
}

/// Request body for `POST /api/v1/runs/query`.
///
/// All fields are optional, allowing flexible querying.
///
/// # OpenAPI Reference
///
/// See `/workspace/reference/api-specs/runs-query-request-schema.json`
///
/// # Example
///
/// ```
/// use langstar_sdk::runs::{QueryRunsRequest, RunType};
///
/// let request = QueryRunsRequest {
///     is_root: Some(true),
///     run_type: Some(RunType::Llm),
///     limit: Some(50),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct QueryRunsRequest {
    /// Filter by session/project IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<Vec<Uuid>>,

    /// Filter by specific run IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<Uuid>>,

    /// Filter by trace ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Uuid>,

    /// Filter by parent run ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run: Option<Uuid>,

    /// Filter by run type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_type: Option<RunType>,

    /// Filter by reference example IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_example: Option<Vec<Uuid>>,

    /// Filter by execution order (must be 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_order: Option<i32>,

    /// Filter runs starting after this time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,

    /// Filter runs ending before this time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,

    /// Filter by error status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<bool>,

    /// Natural language query (experimental)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// Filter expression using LangSmith filter query language.
    ///
    /// Example: `eq(status, "error")` or `has(tags, "production")`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    /// Filter for root run in trace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_filter: Option<String>,

    /// Filter for other runs in trace tree
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_filter: Option<String>,

    /// Only return root runs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_root: Option<bool>,

    /// Data source type filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_source_type: Option<String>,

    /// Skip pagination and return all results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_pagination: Option<bool>,

    /// Alternative search filter syntax
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_filter: Option<String>,

    /// Enable experimental search features. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_experimental_search: Option<bool>,

    /// Cursor for pagination (from previous response)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Maximum number of runs to return (1-100, default: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Fields to select in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<Vec<String>>,

    /// Sort order for results. Defaults to descending.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<RunDateOrder>,

    /// Skip returning previous cursor. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_prev_cursor: Option<bool>,
}

/// Response from `POST /api/v1/runs/query`.
///
/// # OpenAPI Reference
///
/// See `/workspace/reference/api-specs/runs-query-response-schema.json`
#[derive(Debug, Clone, Deserialize)]
pub struct QueryRunsResponse {
    /// List of runs matching the query
    pub runs: Vec<Run>,

    /// Pagination cursors
    pub cursors: Cursors,

    /// Cursors for experimental search (if enabled)
    pub search_cursors: Option<Value>,

    /// How the query was parsed (for debugging)
    pub parsed_query: Option<String>,
}

/// Pagination cursors for runs query response.
#[derive(Debug, Clone, Deserialize)]
pub struct Cursors {
    /// Cursor for the next page of results
    pub next: Option<String>,

    /// Cursor for the previous page of results
    pub prev: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_type_serialization() {
        let llm = RunType::Llm;
        let json = serde_json::to_string(&llm).unwrap();
        assert_eq!(json, "\"llm\"");

        let chain = RunType::Chain;
        let json = serde_json::to_string(&chain).unwrap();
        assert_eq!(json, "\"chain\"");

        let tool = RunType::Tool;
        let json = serde_json::to_string(&tool).unwrap();
        assert_eq!(json, "\"tool\"");
    }

    #[test]
    fn test_run_type_deserialization() {
        let llm: RunType = serde_json::from_str("\"llm\"").unwrap();
        assert_eq!(llm, RunType::Llm);

        let retriever: RunType = serde_json::from_str("\"retriever\"").unwrap();
        assert_eq!(retriever, RunType::Retriever);

        let parser: RunType = serde_json::from_str("\"parser\"").unwrap();
        assert_eq!(parser, RunType::Parser);
    }

    #[test]
    fn test_run_date_order_serialization() {
        let asc = RunDateOrder::Asc;
        let json = serde_json::to_string(&asc).unwrap();
        assert_eq!(json, "\"asc\"");

        let desc = RunDateOrder::Desc;
        let json = serde_json::to_string(&desc).unwrap();
        assert_eq!(json, "\"desc\"");
    }

    #[test]
    fn test_query_runs_request_serialization() {
        let request = QueryRunsRequest {
            is_root: Some(true),
            run_type: Some(RunType::Llm),
            limit: Some(50),
            ..Default::default()
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"is_root\":true"));
        assert!(json.contains("\"run_type\":\"llm\""));
        assert!(json.contains("\"limit\":50"));
    }

    #[test]
    fn test_query_runs_request_omits_none_fields() {
        let request = QueryRunsRequest::default();
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_run_deserialization_minimal() {
        // Minimal required fields only
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "llm",
            "trace_id": "223e4567-e89b-12d3-a456-426614174001",
            "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
            "status": "success",
            "session_id": "323e4567-e89b-12d3-a456-426614174002",
            "app_path": "/chat"
        }"#;

        let run: Run = serde_json::from_str(json).unwrap();
        assert_eq!(run.name, "ChatOpenAI");
        assert_eq!(run.run_type, RunType::Llm);
        assert_eq!(run.status, "success");
        assert_eq!(run.total_tokens, 0); // default
        assert_eq!(run.prompt_tokens, 0); // default
        assert_eq!(run.completion_tokens, 0); // default
        assert_eq!(run.execution_order, 1); // default
        assert!(!run.trace_upgrade); // default false
    }

    #[test]
    fn test_run_deserialization_with_tokens() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "llm",
            "trace_id": "223e4567-e89b-12d3-a456-426614174001",
            "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
            "status": "success",
            "session_id": "323e4567-e89b-12d3-a456-426614174002",
            "app_path": "/chat",
            "total_tokens": 150,
            "prompt_tokens": 100,
            "completion_tokens": 50,
            "total_cost": "0.0015"
        }"#;

        let run: Run = serde_json::from_str(json).unwrap();
        assert_eq!(run.total_tokens, 150);
        assert_eq!(run.prompt_tokens, 100);
        assert_eq!(run.completion_tokens, 50);
        assert_eq!(run.total_cost, Some("0.0015".to_string()));
    }

    #[test]
    fn test_run_deserialization_with_timing() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "llm",
            "trace_id": "223e4567-e89b-12d3-a456-426614174001",
            "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
            "status": "success",
            "session_id": "323e4567-e89b-12d3-a456-426614174002",
            "app_path": "/chat",
            "start_time": "2024-01-01T12:00:00Z",
            "end_time": "2024-01-01T12:00:05Z",
            "first_token_time": "2024-01-01T12:00:01Z"
        }"#;

        let run: Run = serde_json::from_str(json).unwrap();
        assert!(run.start_time.is_some());
        assert!(run.end_time.is_some());
        assert!(run.first_token_time.is_some());
    }

    #[test]
    fn test_run_deserialization_with_hierarchy() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "llm",
            "trace_id": "223e4567-e89b-12d3-a456-426614174001",
            "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
            "status": "success",
            "session_id": "323e4567-e89b-12d3-a456-426614174002",
            "app_path": "/chat",
            "parent_run_id": "423e4567-e89b-12d3-a456-426614174003",
            "parent_run_ids": ["423e4567-e89b-12d3-a456-426614174003"],
            "child_run_ids": ["523e4567-e89b-12d3-a456-426614174004", "623e4567-e89b-12d3-a456-426614174005"]
        }"#;

        let run: Run = serde_json::from_str(json).unwrap();
        assert!(run.parent_run_id.is_some());
        assert_eq!(run.parent_run_ids.as_ref().unwrap().len(), 1);
        assert_eq!(run.child_run_ids.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_run_deserialization_with_tags_and_metadata() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "llm",
            "trace_id": "223e4567-e89b-12d3-a456-426614174001",
            "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
            "status": "success",
            "session_id": "323e4567-e89b-12d3-a456-426614174002",
            "app_path": "/chat",
            "tags": ["production", "gpt-4"],
            "extra": {"model": "gpt-4", "temperature": 0.7}
        }"#;

        let run: Run = serde_json::from_str(json).unwrap();
        let tags = run.tags.unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"production".to_string()));
        assert!(tags.contains(&"gpt-4".to_string()));
        assert!(run.extra.is_some());
    }

    #[test]
    fn test_query_runs_response_deserialization() {
        let json = r#"{
            "runs": [{
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "name": "ChatOpenAI",
                "run_type": "llm",
                "trace_id": "223e4567-e89b-12d3-a456-426614174001",
                "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
                "status": "success",
                "session_id": "323e4567-e89b-12d3-a456-426614174002",
                "app_path": "/chat"
            }],
            "cursors": {
                "next": "cursor_abc123",
                "prev": null
            },
            "parsed_query": "status = success"
        }"#;

        let response: QueryRunsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.runs.len(), 1);
        assert_eq!(response.runs[0].name, "ChatOpenAI");
        assert_eq!(response.cursors.next, Some("cursor_abc123".to_string()));
        assert!(response.cursors.prev.is_none());
        assert_eq!(response.parsed_query, Some("status = success".to_string()));
    }

    #[test]
    fn test_cursors_deserialization() {
        let json = r#"{
            "next": "abc123",
            "prev": "xyz789"
        }"#;

        let cursors: Cursors = serde_json::from_str(json).unwrap();
        assert_eq!(cursors.next, Some("abc123".to_string()));
        assert_eq!(cursors.prev, Some("xyz789".to_string()));
    }

    #[test]
    fn test_cursors_deserialization_nulls() {
        let json = r#"{
            "next": null,
            "prev": null
        }"#;

        let cursors: Cursors = serde_json::from_str(json).unwrap();
        assert!(cursors.next.is_none());
        assert!(cursors.prev.is_none());
    }

    #[test]
    fn test_all_run_types() {
        let types = [
            ("\"tool\"", RunType::Tool),
            ("\"chain\"", RunType::Chain),
            ("\"llm\"", RunType::Llm),
            ("\"retriever\"", RunType::Retriever),
            ("\"embedding\"", RunType::Embedding),
            ("\"prompt\"", RunType::Prompt),
            ("\"parser\"", RunType::Parser),
        ];

        for (json, expected) in types {
            let run_type: RunType = serde_json::from_str(json).unwrap();
            assert_eq!(run_type, expected);
        }
    }
}
