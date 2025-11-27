//! CLI commands for querying and managing LangSmith runs.
//!
//! This module provides the `langstar runs query` command for querying
//! LangSmith runs/traces with filtering and pagination support.

use crate::config::Config;
use crate::error::Result;
use crate::output::{OutputFormat, OutputFormatter};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand, ValueEnum};
use futures_util::StreamExt;
use langstar_sdk::{LangchainClient, QueryRunsRequest, Run, RunDateOrder, RunType};
use serde::Serialize;
use tabled::Tabled;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════
// Commands
// ═══════════════════════════════════════════════════════════════════════════

/// Commands for interacting with LangSmith runs/traces
#[derive(Debug, Subcommand)]
pub enum RunsCommands {
    /// Query runs with filtering and pagination
    Query(QueryArgs),
}

/// Arguments for the `runs query` command
#[derive(Debug, Args)]
pub struct QueryArgs {
    /// Project name or UUID to query runs from
    ///
    /// Can be specified multiple times to query from multiple projects.
    #[arg(short, long = "project", value_name = "PROJECT")]
    pub projects: Vec<String>,

    /// Raw filter expression (LangSmith filter query language)
    ///
    /// Example: 'eq(status, "error")' or 'has(tags, "production")'
    #[arg(long)]
    pub filter: Option<String>,

    /// Filter for root run in trace
    ///
    /// Filter expression applied to the root run of each trace
    #[arg(long)]
    pub trace_filter: Option<String>,

    /// Filter for other runs in trace tree
    ///
    /// Filter expression applied to non-root runs in the trace
    #[arg(long)]
    pub tree_filter: Option<String>,

    /// Only return root runs (top-level traces)
    #[arg(long)]
    pub is_root: bool,

    /// Filter by tag (can be repeated)
    ///
    /// Adds a 'has(tags, "value")' condition to the filter
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,

    /// Filter by metadata key=value (can be repeated)
    ///
    /// Adds an 'eq(metadata["key"], "value")' condition to the filter.
    /// Format: KEY=VALUE (e.g., --meta environment=production)
    #[arg(long = "meta", value_name = "KEY=VALUE")]
    pub metadata: Vec<String>,

    /// Filter by run type
    #[arg(long, value_enum)]
    pub run_type: Option<RunTypeArg>,

    /// Filter by status
    ///
    /// Common values: "success", "error", "pending"
    #[arg(long)]
    pub status: Option<String>,

    /// Only show runs with errors
    #[arg(long)]
    pub errors_only: bool,

    /// Filter runs after this time (ISO 8601 format)
    ///
    /// Example: --since 2024-01-01T00:00:00Z
    #[arg(long)]
    pub since: Option<String>,

    /// Filter runs before this time (ISO 8601 format)
    ///
    /// Example: --until 2024-01-31T23:59:59Z
    #[arg(long)]
    pub until: Option<String>,

    /// Maximum number of runs to return (supports pagination)
    #[arg(short, long, default_value = "100")]
    pub limit: usize,

    /// Sort order for results
    #[arg(long, default_value = "desc", value_enum)]
    pub order: OrderArg,

    /// Output format for runs query
    ///
    /// Note: Uses `--output` to avoid conflict with global `-f/--format` flag
    #[arg(short = 'o', long = "output", default_value = "table", value_enum)]
    pub output: RunsOutputFormat,

    /// Fields to select (comma-separated)
    ///
    /// Limits the fields returned in the response.
    /// Example: --select id,name,status,total_tokens
    #[arg(long)]
    pub select: Option<String>,

    /// Organization ID for scoping (overrides config/env)
    #[arg(long)]
    pub organization_id: Option<String>,

    /// Workspace ID for narrower scoping (overrides config/env)
    #[arg(long)]
    pub workspace_id: Option<String>,
}

/// Output format for runs query
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum RunsOutputFormat {
    /// Human-readable table format
    #[default]
    Table,
    /// Compact JSON format
    Json,
    /// Pretty-printed JSON format
    JsonPretty,
}

/// Run type argument for CLI (maps to SDK RunType)
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RunTypeArg {
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

impl From<RunTypeArg> for RunType {
    fn from(arg: RunTypeArg) -> Self {
        match arg {
            RunTypeArg::Tool => RunType::Tool,
            RunTypeArg::Chain => RunType::Chain,
            RunTypeArg::Llm => RunType::Llm,
            RunTypeArg::Retriever => RunType::Retriever,
            RunTypeArg::Embedding => RunType::Embedding,
            RunTypeArg::Prompt => RunType::Prompt,
            RunTypeArg::Parser => RunType::Parser,
        }
    }
}

/// Sort order argument for CLI
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OrderArg {
    /// Ascending order (oldest first)
    Asc,
    /// Descending order (newest first)
    #[default]
    Desc,
}

impl From<OrderArg> for RunDateOrder {
    fn from(arg: OrderArg) -> Self {
        match arg {
            OrderArg::Asc => RunDateOrder::Asc,
            OrderArg::Desc => RunDateOrder::Desc,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Filter Builder
// ═══════════════════════════════════════════════════════════════════════════

/// Builder for constructing LangSmith filter expressions from CLI flags.
///
/// Combines multiple filter conditions using the LangSmith filter query language.
///
/// # Example
///
/// ```ignore
/// use langstar_cli::commands::runs::FilterBuilder;
///
/// let filter = FilterBuilder::new()
///     .tag("production")
///     .status("error")
///     .metadata("environment", "staging")
///     .build();
///
/// assert_eq!(
///     filter,
///     Some(r#"has(tags, "production") and eq(status, "error") and eq(metadata["environment"], "staging")"#.to_string())
/// );
/// ```
#[derive(Debug, Default)]
pub struct FilterBuilder {
    conditions: Vec<String>,
}

impl FilterBuilder {
    /// Create a new filter builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag filter: `has(tags, "value")`
    pub fn tag(mut self, tag: &str) -> Self {
        self.conditions
            .push(format!("has(tags, \"{}\")", escape_string(tag)));
        self
    }

    /// Add a metadata filter: `eq(metadata["key"], "value")`
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.conditions.push(format!(
            "eq(metadata[\"{}\"], \"{}\")",
            escape_string(key),
            escape_string(value)
        ));
        self
    }

    /// Add a status filter: `eq(status, "value")`
    pub fn status(mut self, status: &str) -> Self {
        self.conditions
            .push(format!("eq(status, \"{}\")", escape_string(status)));
        self
    }

    /// Add an error filter: `eq(error, true)`
    ///
    /// Named to match the CLI flag `--errors-only`.
    pub fn errors_only(mut self) -> Self {
        self.conditions.push("eq(error, true)".to_string());
        self
    }

    /// Add a raw filter expression
    pub fn raw(mut self, filter: &str) -> Self {
        if !filter.is_empty() {
            self.conditions.push(filter.to_string());
        }
        self
    }

    /// Build the final filter string
    ///
    /// Returns `None` if no conditions were added.
    /// Joins multiple conditions with ` and `.
    pub fn build(self) -> Option<String> {
        if self.conditions.is_empty() {
            None
        } else {
            Some(self.conditions.join(" and "))
        }
    }
}

/// Escape special characters in filter string values
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ═══════════════════════════════════════════════════════════════════════════
// Table Display
// ═══════════════════════════════════════════════════════════════════════════

/// Simplified run info for table display
#[derive(Debug, Tabled, Serialize)]
struct RunRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    run_type: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Tokens")]
    tokens: String,
    #[tabled(rename = "Duration")]
    duration: String,
    #[tabled(rename = "Time")]
    time: String,
}

impl From<&Run> for RunRow {
    fn from(run: &Run) -> Self {
        // Calculate duration if we have both start and end times
        let duration = match (&run.start_time, &run.end_time) {
            (Some(start), Some(end)) => {
                let duration = *end - *start;
                let millis = duration.num_milliseconds();
                if millis < 1000 {
                    format!("{}ms", millis)
                } else {
                    format!("{:.2}s", millis as f64 / 1000.0)
                }
            }
            _ => "-".to_string(),
        };

        // Format time (use start_time or "-")
        let time = run
            .start_time
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "-".to_string());

        // Truncate name if too long (unicode-safe)
        let name = if run.name.chars().count() > 30 {
            format!("{}...", run.name.chars().take(27).collect::<String>())
        } else {
            run.name.clone()
        };

        // Format tokens
        let tokens = if run.total_tokens > 0 {
            run.total_tokens.to_string()
        } else {
            "-".to_string()
        };

        Self {
            id: run.id.to_string().chars().take(8).collect::<String>(), // Short UUID
            name,
            run_type: format!("{:?}", run.run_type).to_lowercase(),
            status: run.status.clone(),
            tokens,
            duration,
            time,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Command Implementation
// ═══════════════════════════════════════════════════════════════════════════

impl RunsCommands {
    /// Apply organization and workspace ID overrides to the client
    fn apply_scoping(
        client: LangchainClient,
        flag_org_id: &Option<String>,
        flag_workspace_id: &Option<String>,
        formatter: &OutputFormatter,
    ) -> LangchainClient {
        let mut client = client;

        // Warn if both organization and workspace IDs are specified
        if flag_org_id.is_some() && flag_workspace_id.is_some() {
            formatter.warning(
                "Both organization ID and workspace ID are specified. \
                The workspace ID will be used within the specified organization. \
                If this is not intended, please specify only one.",
            );
        }

        // Apply organization ID if provided via flag (overrides config/env)
        if let Some(org_id) = flag_org_id {
            client = client.with_organization_id(org_id.clone());
        }

        // Apply workspace ID if provided via flag (overrides config/env)
        if let Some(workspace_id) = flag_workspace_id {
            client = client.with_workspace_id(workspace_id.clone());
        }

        client
    }

    /// Execute the runs command
    pub async fn execute(&self, config: &Config, format: OutputFormat) -> Result<()> {
        match self {
            RunsCommands::Query(args) => Self::execute_query(args, config, format).await,
        }
    }

    /// Execute the query subcommand
    async fn execute_query(args: &QueryArgs, config: &Config, _format: OutputFormat) -> Result<()> {
        // Create output formatter based on runs-specific format (needed for apply_scoping warnings)
        let formatter = match args.output {
            RunsOutputFormat::Table => OutputFormatter::new(OutputFormat::Table),
            RunsOutputFormat::Json | RunsOutputFormat::JsonPretty => {
                OutputFormatter::new(OutputFormat::Json)
            }
        };

        let auth = config.to_auth_config();
        let client = LangchainClient::new(auth)?;
        let client = Self::apply_scoping(
            client,
            &args.organization_id,
            &args.workspace_id,
            &formatter,
        );

        // Build filter from convenience flags
        let mut filter_builder = FilterBuilder::new();

        // Add tag filters
        for tag in &args.tags {
            filter_builder = filter_builder.tag(tag);
        }

        // Add metadata filters (parse KEY=VALUE format)
        for meta in &args.metadata {
            if let Some((key, value)) = meta.split_once('=') {
                filter_builder = filter_builder.metadata(key, value);
            } else {
                formatter.warning(&format!(
                    "Invalid metadata format '{}', expected KEY=VALUE",
                    meta
                ));
            }
        }

        // Add status filter
        if let Some(status) = &args.status {
            filter_builder = filter_builder.status(status);
        }

        // Add error filter
        if args.errors_only {
            filter_builder = filter_builder.errors_only();
        }

        // Add raw filter (if provided)
        if let Some(raw_filter) = &args.filter {
            filter_builder = filter_builder.raw(raw_filter);
        }

        let combined_filter = filter_builder.build();

        // Parse project IDs/names (warn if not valid UUIDs)
        let session_ids: Vec<Uuid> = args
            .projects
            .iter()
            .filter_map(|p| match Uuid::parse_str(p) {
                Ok(uuid) => Some(uuid),
                Err(_) => {
                    formatter.warning(&format!("Project '{}' is not a valid UUID, ignoring", p));
                    None
                }
            })
            .collect();

        // Parse time filters (warn on invalid formats)
        let start_time: Option<DateTime<Utc>> = args.since.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    formatter.warning(&format!(
                        "Invalid --since format: {}. Expected ISO 8601 format (e.g., 2024-01-01T00:00:00Z)",
                        e
                    ));
                })
                .ok()
        });

        let end_time: Option<DateTime<Utc>> = args.until.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| {
                    formatter.warning(&format!(
                        "Invalid --until format: {}. Expected ISO 8601 format (e.g., 2024-01-01T00:00:00Z)",
                        e
                    ));
                })
                .ok()
        });

        // Parse select fields
        let select: Option<Vec<String>> = args
            .select
            .as_ref()
            .map(|s| s.split(',').map(|f| f.trim().to_string()).collect());

        // Show query info (only for table output to keep JSON clean)
        if args.output == RunsOutputFormat::Table {
            if !args.projects.is_empty() {
                formatter.info(&format!(
                    "Querying runs from projects: {}",
                    args.projects.join(", ")
                ));
            } else {
                formatter.info("Querying runs from all projects...");
            }

            if let Some(filter) = &combined_filter {
                formatter.info(&format!("Filter: {}", filter));
            }

            formatter.info(&format!("Limit: {}", args.limit));
        }

        // Build the request (combined_filter is moved, not cloned)
        let request = QueryRunsRequest {
            session: if session_ids.is_empty() {
                None
            } else {
                Some(session_ids)
            },
            filter: combined_filter,
            trace_filter: args.trace_filter.clone(),
            tree_filter: args.tree_filter.clone(),
            is_root: if args.is_root { Some(true) } else { None },
            run_type: args.run_type.map(|rt| rt.into()),
            // Note: errors_only is handled via filter_builder.errors_only(), not the error field
            start_time,
            end_time,
            select,
            order: Some(args.order.into()),
            limit: Some(100.min(args.limit as u32)), // API max is 100 per page
            ..Default::default()
        };

        // Execute query with pagination
        let mut stream = client.query_runs_paginated(request, Some(args.limit));
        let mut runs: Vec<Run> = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(run) => runs.push(run),
                Err(e) => {
                    formatter.error(&format!("Error fetching runs: {}", e));
                    break;
                }
            }
        }

        // Output results
        match args.output {
            RunsOutputFormat::Table => {
                let rows: Vec<RunRow> = runs.iter().map(RunRow::from).collect();
                formatter.print_table(&rows)?;
                println!("\nFound {} runs", runs.len());
            }
            RunsOutputFormat::Json => {
                println!("{}", serde_json::to_string(&runs)?);
            }
            RunsOutputFormat::JsonPretty => {
                println!("{}", serde_json::to_string_pretty(&runs)?);
            }
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder_empty() {
        let filter = FilterBuilder::new().build();
        assert!(filter.is_none());
    }

    #[test]
    fn test_filter_builder_single_tag() {
        let filter = FilterBuilder::new().tag("production").build();
        assert_eq!(filter, Some(r#"has(tags, "production")"#.to_string()));
    }

    #[test]
    fn test_filter_builder_multiple_tags() {
        let filter = FilterBuilder::new().tag("production").tag("gpt-4").build();
        assert_eq!(
            filter,
            Some(r#"has(tags, "production") and has(tags, "gpt-4")"#.to_string())
        );
    }

    #[test]
    fn test_filter_builder_metadata() {
        let filter = FilterBuilder::new()
            .metadata("environment", "staging")
            .build();
        assert_eq!(
            filter,
            Some(r#"eq(metadata["environment"], "staging")"#.to_string())
        );
    }

    #[test]
    fn test_filter_builder_status() {
        let filter = FilterBuilder::new().status("error").build();
        assert_eq!(filter, Some(r#"eq(status, "error")"#.to_string()));
    }

    #[test]
    fn test_filter_builder_errors_only() {
        let filter = FilterBuilder::new().errors_only().build();
        assert_eq!(filter, Some("eq(error, true)".to_string()));
    }

    #[test]
    fn test_filter_builder_combined() {
        let filter = FilterBuilder::new()
            .tag("production")
            .status("error")
            .metadata("model", "gpt-4")
            .build();
        assert_eq!(
            filter,
            Some(
                r#"has(tags, "production") and eq(status, "error") and eq(metadata["model"], "gpt-4")"#
                    .to_string()
            )
        );
    }

    #[test]
    fn test_filter_builder_with_raw() {
        let filter = FilterBuilder::new()
            .tag("production")
            .raw("gt(total_tokens, 100)")
            .build();
        assert_eq!(
            filter,
            Some(r#"has(tags, "production") and gt(total_tokens, 100)"#.to_string())
        );
    }

    #[test]
    fn test_filter_builder_raw_only() {
        let filter = FilterBuilder::new().raw("eq(name, \"ChatOpenAI\")").build();
        assert_eq!(filter, Some(r#"eq(name, "ChatOpenAI")"#.to_string()));
    }

    #[test]
    fn test_filter_builder_empty_raw() {
        let filter = FilterBuilder::new().raw("").build();
        assert!(filter.is_none());
    }

    #[test]
    fn test_escape_string_basic() {
        assert_eq!(escape_string("hello"), "hello");
    }

    #[test]
    fn test_escape_string_with_quotes() {
        assert_eq!(escape_string(r#"hello "world""#), r#"hello \"world\""#);
    }

    #[test]
    fn test_escape_string_with_backslashes() {
        assert_eq!(escape_string(r"path\to\file"), r"path\\to\\file");
    }

    #[test]
    fn test_run_type_conversion() {
        assert!(matches!(RunType::from(RunTypeArg::Tool), RunType::Tool));
        assert!(matches!(RunType::from(RunTypeArg::Chain), RunType::Chain));
        assert!(matches!(RunType::from(RunTypeArg::Llm), RunType::Llm));
        assert!(matches!(
            RunType::from(RunTypeArg::Retriever),
            RunType::Retriever
        ));
        assert!(matches!(
            RunType::from(RunTypeArg::Embedding),
            RunType::Embedding
        ));
        assert!(matches!(RunType::from(RunTypeArg::Prompt), RunType::Prompt));
        assert!(matches!(RunType::from(RunTypeArg::Parser), RunType::Parser));
    }

    #[test]
    fn test_order_arg_conversion() {
        assert!(matches!(
            RunDateOrder::from(OrderArg::Asc),
            RunDateOrder::Asc
        ));
        assert!(matches!(
            RunDateOrder::from(OrderArg::Desc),
            RunDateOrder::Desc
        ));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // RunRow conversion tests
    // ═══════════════════════════════════════════════════════════════════════

    fn create_test_run(name: &str, total_tokens: i64) -> Run {
        let json = format!(
            r#"{{
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "name": "{}",
                "run_type": "llm",
                "trace_id": "223e4567-e89b-12d3-a456-426614174001",
                "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
                "status": "success",
                "session_id": "323e4567-e89b-12d3-a456-426614174002",
                "app_path": "/chat",
                "total_tokens": {}
            }}"#,
            name, total_tokens
        );
        serde_json::from_str(&json).unwrap()
    }

    fn create_test_run_with_timing(start: &str, end: &str) -> Run {
        let json = format!(
            r#"{{
                "id": "123e4567-e89b-12d3-a456-426614174000",
                "name": "ChatOpenAI",
                "run_type": "llm",
                "trace_id": "223e4567-e89b-12d3-a456-426614174001",
                "dotted_order": "20240101T000000000000Z123e4567-e89b-12d3-a456-426614174000",
                "status": "success",
                "session_id": "323e4567-e89b-12d3-a456-426614174002",
                "app_path": "/chat",
                "start_time": "{}",
                "end_time": "{}"
            }}"#,
            start, end
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn test_run_row_name_truncation_short() {
        let run = create_test_run("ShortName", 0);
        let row = RunRow::from(&run);
        assert_eq!(row.name, "ShortName");
    }

    #[test]
    fn test_run_row_name_truncation_long() {
        let run = create_test_run("ThisIsAVeryLongNameThatShouldBeTruncated", 0);
        let row = RunRow::from(&run);
        assert_eq!(row.name, "ThisIsAVeryLongNameThatShou...");
        assert_eq!(row.name.chars().count(), 30);
    }

    #[test]
    fn test_run_row_name_truncation_unicode() {
        // Test with emoji (multi-byte characters)
        let run = create_test_run("🚀🎉✨💡🔥⭐🌟🎯💫🌈🎊🎁", 0);
        let row = RunRow::from(&run);
        // Should not panic and should truncate properly
        assert!(row.name.chars().count() <= 30);
    }

    #[test]
    fn test_run_row_duration_milliseconds() {
        // 500ms duration
        let run =
            create_test_run_with_timing("2024-01-01T12:00:00.000Z", "2024-01-01T12:00:00.500Z");
        let row = RunRow::from(&run);
        assert_eq!(row.duration, "500ms");
    }

    #[test]
    fn test_run_row_duration_seconds() {
        // 5 second duration
        let run = create_test_run_with_timing("2024-01-01T12:00:00Z", "2024-01-01T12:00:05Z");
        let row = RunRow::from(&run);
        assert_eq!(row.duration, "5.00s");
    }

    #[test]
    fn test_run_row_tokens_display() {
        let run = create_test_run("Test", 150);
        let row = RunRow::from(&run);
        assert_eq!(row.tokens, "150");
    }

    #[test]
    fn test_run_row_tokens_display_zero() {
        let run = create_test_run("Test", 0);
        let row = RunRow::from(&run);
        assert_eq!(row.tokens, "-");
    }

    #[test]
    fn test_run_row_uuid_truncation() {
        let run = create_test_run("Test", 0);
        let row = RunRow::from(&run);
        // Should be first 8 chars of UUID
        assert_eq!(row.id, "123e4567");
        assert_eq!(row.id.len(), 8);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Pagination limit tests
    // ═══════════════════════════════════════════════════════════════════════

    /// Helper to calculate the per-page limit (mirrors the logic in execute_query)
    fn calculate_per_page_limit(args_limit: usize) -> u32 {
        100.min(args_limit as u32)
    }

    #[test]
    fn test_per_page_limit_below_max() {
        // When user requests less than 100, use their limit
        assert_eq!(calculate_per_page_limit(50), 50);
        assert_eq!(calculate_per_page_limit(1), 1);
        assert_eq!(calculate_per_page_limit(99), 99);
    }

    #[test]
    fn test_per_page_limit_at_max() {
        // When user requests exactly 100, use 100
        assert_eq!(calculate_per_page_limit(100), 100);
    }

    #[test]
    fn test_per_page_limit_above_max() {
        // When user requests more than 100, clamp to API max of 100
        // The SDK's query_runs_paginated handles fetching additional pages
        assert_eq!(calculate_per_page_limit(101), 100);
        assert_eq!(calculate_per_page_limit(500), 100);
        assert_eq!(calculate_per_page_limit(1000), 100);
    }
}
