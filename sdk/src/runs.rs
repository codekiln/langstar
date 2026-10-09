//! Run types for LangSmith runs/traces API.
//!
//! This module provides types for querying and working with LangSmith runs.
//! Runs represent individual executions in a LangChain application trace.
//!
//! # API Reference
//!
//! - Endpoint: `POST /api/v2/runs/query`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//! - Migration guide from `POST /api/v1/runs/query`:
//!   <https://docs.langchain.com/langsmith/smithdb-sdk-migration-query-runs>
//!
//! # Example
//!
//! ```no_run
//! use langstar_sdk::runs::{QueryRunsRequest, RunSelectField, RunType};
//! use uuid::Uuid;
//!
//! // Create a query request
//! let request = QueryRunsRequest {
//!     project_ids: Some(vec![Uuid::nil()]),
//!     is_root: Some(true),
//!     run_type: Some(RunType::Llm),
//!     page_size: Some(10),
//!     selects: Some(RunSelectField::ALL.to_vec()),
//!     ..Default::default()
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::serde_utils::deserialize_flexible_datetime_opt;

/// Run type enum matching OpenAPI spec `query.RunType`.
///
/// Represents the type of operation a run performed. Serializes as the
/// lowercase value the v1 endpoints use (`"llm"`) and also accepts the
/// uppercase value `POST /api/v2/runs/query` returns (`"LLM"`).
/// [`QueryRunsRequest`] sends the uppercase value.
///
/// # OpenAPI Reference
///
/// Values: `["tool", "chain", "llm", "retriever", "embedding", "prompt", "parser"]`
/// (`RunTypeEnum`); v2 uses the same values in uppercase (`query.RunType`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    /// Tool execution run
    #[serde(alias = "TOOL")]
    Tool,
    /// Chain execution run
    #[serde(alias = "CHAIN")]
    Chain,
    /// LLM (Language Model) call run
    #[serde(alias = "LLM")]
    Llm,
    /// Retriever execution run
    #[serde(alias = "RETRIEVER")]
    Retriever,
    /// Embedding operation run
    #[serde(alias = "EMBEDDING")]
    Embedding,
    /// Prompt template run
    #[serde(alias = "PROMPT")]
    Prompt,
    /// Output parser run
    #[serde(alias = "PARSER")]
    Parser,
}

/// Run schema based on OpenAPI `RunSchema` spec, as the v1 endpoints return it.
///
/// The annotation queue endpoints return this shape. Run queries return
/// [`QueriedRun`] instead.
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
/// See `reference/api-specs/langsmith/run-schema.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub start_time: Option<DateTime<Utc>>,

    /// When the run ended
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub end_time: Option<DateTime<Utc>>,

    /// When the first token was received (for streaming LLM calls)
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub first_token_time: Option<DateTime<Utc>>,

    /// When the run was last queued
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
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
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub trace_first_received_at: Option<DateTime<Utc>>,

    /// Minimum start time in the trace
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
    pub trace_min_start_time: Option<DateTime<Utc>>,

    /// Maximum start time in the trace
    #[serde(default, deserialize_with = "deserialize_flexible_datetime_opt")]
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

/// A run returned by `POST /api/v2/runs/query`, matching OpenAPI `query.RunResponse`.
///
/// The API returns only the fields named in [`QueryRunsRequest::selects`]
/// (only `id` when `selects` is omitted), so every field except `id` is
/// optional.
///
/// # OpenAPI Reference
///
/// See `query.RunResponse` in `reference/openapi/langchain/langsmith/openapi.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueriedRun {
    /// Unique identifier for the run
    pub id: Uuid,

    /// Name of the run (typically the component name)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Type of run (LLM, CHAIN, TOOL, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_type: Option<RunType>,

    /// Completion status of the run: `SUCCESS`, `ERROR` or `PENDING`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Tracing project this run was logged to (v1 called this `session_id`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<Uuid>,

    /// ID of the root trace this run belongs to; equals `id` for a root run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Uuid>,

    /// Dotted order string for hierarchical ordering within trace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dotted_order: Option<String>,

    /// Whether this run is the root of its trace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_root: Option<bool>,

    /// Ancestor run IDs, from the trace root down to the direct parent
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run_ids: Option<Vec<Uuid>>,

    /// Conversation thread this run belongs to, if any
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// Application path identifier
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_path: Option<String>,

    /// When the run started
    #[serde(
        default,
        deserialize_with = "deserialize_flexible_datetime_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_time: Option<DateTime<Utc>>,

    /// When the run ended
    #[serde(
        default,
        deserialize_with = "deserialize_flexible_datetime_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub end_time: Option<DateTime<Utc>>,

    /// When the first token was received (for streaming LLM calls)
    #[serde(
        default,
        deserialize_with = "deserialize_flexible_datetime_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub first_token_time: Option<DateTime<Utc>>,

    /// When the run was last queued
    #[serde(
        default,
        deserialize_with = "deserialize_flexible_datetime_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub last_queued_at: Option<DateTime<Utc>>,

    /// Time between start and end, in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_seconds: Option<f64>,

    /// Input data for the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Value>,

    /// Preview of inputs (truncated string)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs_preview: Option<String>,

    /// Output data from the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Value>,

    /// Preview of outputs (truncated string)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs_preview: Option<String>,

    /// Error message if the run failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Preview of the error message (truncated string)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_preview: Option<String>,

    /// Extra data recorded with the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,

    /// Metadata recorded with the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Events emitted during the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<Value>>,

    /// Serialized manifest of the component
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<Value>,

    /// Attachment file names mapped to pre-signed download URLs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Value>,

    /// Tags attached to the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Total tokens used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i64>,

    /// Prompt tokens used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<i64>,

    /// Completion tokens used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<i64>,

    /// Breakdown of prompt tokens
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_token_details: Option<Value>,

    /// Breakdown of completion tokens
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_token_details: Option<Value>,

    /// Estimated total cost in USD
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_cost: Option<f64>,

    /// Estimated prompt cost in USD
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cost: Option<f64>,

    /// Estimated completion cost in USD
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_cost: Option<f64>,

    /// Breakdown of prompt cost
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_cost_details: Option<Value>,

    /// Breakdown of completion cost
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_cost_details: Option<Value>,

    /// Price model used to compute costs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_model_id: Option<Uuid>,

    /// Aggregated feedback scores, keyed by feedback key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback_stats: Option<Value>,

    /// Dataset example this run references
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_example_id: Option<Uuid>,

    /// Dataset this run references
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_dataset_id: Option<Uuid>,

    /// Whether the run has been added to a dataset (v1 called this `in_dataset`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_in_dataset: Option<bool>,

    /// Public share URL, if the run is shared
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_url: Option<String>,

    /// When the thread evaluation ran
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_evaluation_time: Option<String>,

    /// LangSmith user whose credential traced the run
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ls_user_id: Option<Uuid>,

    /// Metadata about this query result, e.g. `sem_filter_score`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_metadata: Option<Value>,
}

/// A run property that `POST /api/v2/runs/query` can return, matching
/// OpenAPI `query.RunSelectField`.
///
/// Serializes as the uppercase name the API expects (`"TOTAL_TOKENS"`).
/// [`RunSelectField::from_name`] accepts the snake_case field name instead.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
pub enum RunSelectField {
    Id,
    Name,
    RunType,
    Status,
    StartTime,
    EndTime,
    LatencySeconds,
    FirstTokenTime,
    Error,
    ErrorPreview,
    Extra,
    Metadata,
    Events,
    Inputs,
    InputsPreview,
    Outputs,
    OutputsPreview,
    Manifest,
    ParentRunIds,
    ProjectId,
    TraceId,
    ThreadId,
    DottedOrder,
    IsRoot,
    ReferenceExampleId,
    ReferenceDatasetId,
    TotalTokens,
    PromptTokens,
    CompletionTokens,
    TotalCost,
    PromptCost,
    CompletionCost,
    PromptTokenDetails,
    CompletionTokenDetails,
    PromptCostDetails,
    CompletionCostDetails,
    PriceModelId,
    Tags,
    AppPath,
    Attachments,
    ThreadEvaluationTime,
    IsInDataset,
    LastQueuedAt,
    ShareUrl,
    FeedbackStats,
    LsUserId,
}

impl RunSelectField {
    /// Every field the API can return, in OpenAPI enum order.
    pub const ALL: [RunSelectField; 46] = [
        Self::Id,
        Self::Name,
        Self::RunType,
        Self::Status,
        Self::StartTime,
        Self::EndTime,
        Self::LatencySeconds,
        Self::FirstTokenTime,
        Self::Error,
        Self::ErrorPreview,
        Self::Extra,
        Self::Metadata,
        Self::Events,
        Self::Inputs,
        Self::InputsPreview,
        Self::Outputs,
        Self::OutputsPreview,
        Self::Manifest,
        Self::ParentRunIds,
        Self::ProjectId,
        Self::TraceId,
        Self::ThreadId,
        Self::DottedOrder,
        Self::IsRoot,
        Self::ReferenceExampleId,
        Self::ReferenceDatasetId,
        Self::TotalTokens,
        Self::PromptTokens,
        Self::CompletionTokens,
        Self::TotalCost,
        Self::PromptCost,
        Self::CompletionCost,
        Self::PromptTokenDetails,
        Self::CompletionTokenDetails,
        Self::PromptCostDetails,
        Self::CompletionCostDetails,
        Self::PriceModelId,
        Self::Tags,
        Self::AppPath,
        Self::Attachments,
        Self::ThreadEvaluationTime,
        Self::IsInDataset,
        Self::LastQueuedAt,
        Self::ShareUrl,
        Self::FeedbackStats,
        Self::LsUserId,
    ];

    /// Looks up a field by its name in either case: `total_tokens` or `TOTAL_TOKENS`.
    ///
    /// Returns `None` for a name the API does not offer.
    pub fn from_name(name: &str) -> Option<Self> {
        serde_json::from_value(Value::String(name.trim().to_ascii_uppercase())).ok()
    }
}

/// Request body for `POST /api/v2/runs/query`, matching OpenAPI
/// `query.QueryRunsRequestBody`.
///
/// Set exactly one of `project_ids` or `reference_dataset_id`. The API
/// always sorts results by `start_time`, newest first.
///
/// # Example
///
/// ```
/// use langstar_sdk::runs::{QueryRunsRequest, RunSelectField, RunType};
/// use uuid::Uuid;
///
/// let request = QueryRunsRequest {
///     project_ids: Some(vec![Uuid::nil()]),
///     is_root: Some(true),
///     run_type: Some(RunType::Llm),
///     page_size: Some(50),
///     selects: Some(vec![RunSelectField::Id, RunSelectField::Name]),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Default)]
pub struct QueryRunsRequest {
    /// Tracing projects to query. Mutually exclusive with `reference_dataset_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_ids: Option<Vec<Uuid>>,

    /// Dataset whose experiment projects to query. Mutually exclusive with `project_ids`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_dataset_id: Option<Uuid>,

    /// Limit results to these run IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<Uuid>>,

    /// Limit results to runs in this trace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Uuid>,

    /// Limit results to runs linked to these dataset examples
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_examples: Option<Vec<Uuid>>,

    /// Filter by run type
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_run_type_uppercase"
    )]
    pub run_type: Option<RunType>,

    /// Filter expression using LangSmith filter query language.
    ///
    /// Example: `eq(status, "error")` or `has(tags, "production")`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    /// Filter applied to the root run of each trace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_filter: Option<String>,

    /// Filter matching any run in each trace's tree
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_filter: Option<String>,

    /// Only root runs (`true`) or only non-root runs (`false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_root: Option<bool>,

    /// Only errored runs (`true`) or only runs without error (`false`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_error: Option<bool>,

    /// Lower bound for run `start_time`. The API defaults to 1 day ago and
    /// rejects a window longer than 401 days.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_start_time: Option<DateTime<Utc>>,

    /// Upper bound for run `start_time`. The API defaults to now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_start_time: Option<DateTime<Utc>>,

    /// Runs per page (1-1000). The API defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,

    /// Opaque cursor from a previous response's `next_cursor`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Fields to return on each run. The API returns only `id` when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selects: Option<Vec<RunSelectField>>,
}

/// Serializes a run type the way `POST /api/v2/runs/query` expects it: `"LLM"`.
fn serialize_run_type_uppercase<S: serde::Serializer>(
    run_type: &Option<RunType>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    match run_type {
        Some(rt) => match serde_json::to_value(rt) {
            Ok(Value::String(name)) => serializer.serialize_str(&name.to_ascii_uppercase()),
            _ => Err(serde::ser::Error::custom(
                "run type did not serialize as a string",
            )),
        },
        None => serializer.serialize_none(),
    }
}

/// Response from `POST /api/v2/runs/query`, matching OpenAPI
/// `query.QueryRunsResponseBody`.
#[derive(Debug, Clone, Deserialize)]
pub struct QueryRunsResponse {
    /// One page of runs, sorted by `start_time` descending
    #[serde(default)]
    pub items: Vec<QueriedRun>,

    /// Cursor for the next page; `None` on the last page
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_type_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&RunType::Llm).unwrap(), "\"llm\"");
        assert_eq!(serde_json::to_string(&RunType::Chain).unwrap(), "\"chain\"");
        assert_eq!(serde_json::to_string(&RunType::Tool).unwrap(), "\"tool\"");
    }

    #[test]
    fn test_query_runs_request_sends_every_run_type_uppercase() {
        let all = [
            RunType::Tool,
            RunType::Chain,
            RunType::Llm,
            RunType::Retriever,
            RunType::Embedding,
            RunType::Prompt,
            RunType::Parser,
        ];
        let sent: Vec<Value> = all
            .iter()
            .map(|rt| {
                let request = QueryRunsRequest {
                    run_type: Some(*rt),
                    ..Default::default()
                };
                serde_json::to_value(&request).unwrap()["run_type"].clone()
            })
            .collect();
        let spec: Value = serde_json::from_str(include_str!(
            "../../reference/openapi/langchain/langsmith/openapi.json"
        ))
        .unwrap();
        let expected = spec["components"]["schemas"]["query.RunType"]["enum"].clone();
        assert_eq!(Value::Array(sent), expected);
    }

    #[test]
    fn test_run_type_deserializes_either_case() {
        let upper: RunType = serde_json::from_str("\"RETRIEVER\"").unwrap();
        assert_eq!(upper, RunType::Retriever);
        let lower: RunType = serde_json::from_str("\"parser\"").unwrap();
        assert_eq!(lower, RunType::Parser);
    }

    #[test]
    fn test_all_run_types() {
        let types = [
            ("\"TOOL\"", RunType::Tool),
            ("\"CHAIN\"", RunType::Chain),
            ("\"LLM\"", RunType::Llm),
            ("\"RETRIEVER\"", RunType::Retriever),
            ("\"EMBEDDING\"", RunType::Embedding),
            ("\"PROMPT\"", RunType::Prompt),
            ("\"PARSER\"", RunType::Parser),
        ];

        for (json, expected) in types {
            let run_type: RunType = serde_json::from_str(json).unwrap();
            assert_eq!(run_type, expected);
        }
    }

    #[test]
    fn test_query_runs_request_serialization() {
        let project = Uuid::parse_str("323e4567-e89b-12d3-a456-426614174002").unwrap();
        let request = QueryRunsRequest {
            project_ids: Some(vec![project]),
            is_root: Some(true),
            run_type: Some(RunType::Llm),
            page_size: Some(50),
            selects: Some(vec![RunSelectField::Id, RunSelectField::TotalTokens]),
            ..Default::default()
        };

        let json: Value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "project_ids": ["323e4567-e89b-12d3-a456-426614174002"],
                "is_root": true,
                "run_type": "LLM",
                "page_size": 50,
                "selects": ["ID", "TOTAL_TOKENS"]
            })
        );
    }

    #[test]
    fn test_query_runs_request_omits_none_fields() {
        let request = QueryRunsRequest::default();
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_run_select_field_all_matches_openapi_enum() {
        let spec: Value = serde_json::from_str(include_str!(
            "../../reference/openapi/langchain/langsmith/openapi.json"
        ))
        .unwrap();
        let expected = &spec["components"]["schemas"]["query.RunSelectField"]["enum"];
        assert_eq!(
            &serde_json::to_value(RunSelectField::ALL.as_slice()).unwrap(),
            expected
        );
    }

    #[test]
    fn test_run_select_field_from_name() {
        assert_eq!(
            RunSelectField::from_name("total_tokens"),
            Some(RunSelectField::TotalTokens)
        );
        assert_eq!(RunSelectField::from_name(" ID "), Some(RunSelectField::Id));
        assert_eq!(RunSelectField::from_name("session_id"), None);
    }

    #[test]
    fn test_run_deserialization_id_only() {
        // With no `selects`, the API returns only `id`.
        let run: QueriedRun =
            serde_json::from_str(r#"{"id": "123e4567-e89b-12d3-a456-426614174000"}"#).unwrap();
        assert!(run.name.is_none());
        assert!(run.status.is_none());
        assert!(run.total_tokens.is_none());
    }

    #[test]
    fn test_run_deserialization_v2_fields() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "LLM",
            "status": "SUCCESS",
            "project_id": "323e4567-e89b-12d3-a456-426614174002",
            "trace_id": "123e4567-e89b-12d3-a456-426614174000",
            "parent_run_ids": [],
            "start_time": "2024-01-01T12:00:00.000Z",
            "end_time": "2024-01-01T12:00:05.000Z",
            "total_tokens": 150,
            "total_cost": 0.0015,
            "tags": ["production"],
            "is_in_dataset": false
        }"#;

        let run: QueriedRun = serde_json::from_str(json).unwrap();
        assert_eq!(run.name.as_deref(), Some("ChatOpenAI"));
        assert_eq!(run.run_type, Some(RunType::Llm));
        assert_eq!(run.status.as_deref(), Some("SUCCESS"));
        assert_eq!(
            run.project_id.unwrap().to_string(),
            "323e4567-e89b-12d3-a456-426614174002"
        );
        assert_eq!(run.total_tokens, Some(150));
        assert_eq!(run.total_cost, Some(0.0015));
        assert!(run.start_time.is_some() && run.end_time.is_some());
        assert_eq!(run.is_in_dataset, Some(false));
    }

    #[test]
    fn test_run_serialization_omits_unselected_fields() {
        let run: QueriedRun =
            serde_json::from_str(r#"{"id": "123e4567-e89b-12d3-a456-426614174000", "name": "x"}"#)
                .unwrap();
        let json: Value = serde_json::to_value(&run).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"id": "123e4567-e89b-12d3-a456-426614174000", "name": "x"})
        );
    }

    #[test]
    fn test_query_runs_response_deserialization() {
        let json = r#"{
            "items": [{"id": "123e4567-e89b-12d3-a456-426614174000", "name": "ChatOpenAI"}],
            "next_cursor": "cursor_abc123"
        }"#;

        let response: QueryRunsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].name.as_deref(), Some("ChatOpenAI"));
        assert_eq!(response.next_cursor.as_deref(), Some("cursor_abc123"));
    }

    #[test]
    fn test_query_runs_response_last_page() {
        let response: QueryRunsResponse =
            serde_json::from_str(r#"{"items": [], "next_cursor": null}"#).unwrap();
        assert!(response.items.is_empty());
        assert!(response.next_cursor.is_none());
    }
}
