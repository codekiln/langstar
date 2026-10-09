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
use langstar_sdk::{LangchainClient, QueriedRun, QueryRunsRequest, RunSelectField, RunType};
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
    /// Project UUID to query runs from (required)
    ///
    /// Can be specified multiple times to query from multiple projects.
    /// The LangSmith runs API needs at least one project UUID.
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

    /// Filter runs after this time
    ///
    /// Accepts either:
    /// - Relative duration: 15m, 1h, 7d, 2w (minutes, hours, days, weeks)
    /// - ISO 8601 timestamp: 2024-01-01T00:00:00Z
    ///
    /// Examples:
    ///   --since 15m    # Last 15 minutes
    ///   --since 1h     # Last 1 hour
    ///   --since 7d     # Last 7 days
    ///   --since 2024-01-01T00:00:00Z  # Since specific time
    #[arg(long)]
    pub since: Option<String>,

    /// Filter runs before this time (ISO 8601 format only)
    ///
    /// Unlike --since, --until only accepts ISO 8601 timestamps.
    /// This is intentional: "until 1h ago" is semantically confusing.
    ///
    /// Example: --until 2024-01-31T23:59:59Z
    #[arg(long)]
    pub until: Option<String>,

    /// Use a preset time window
    ///
    /// Common time windows matching the LangSmith UI.
    /// Overridden by --since/--until if provided.
    ///
    /// Available presets: 1h, 3h, 6h, 12h, 1d, 2d, 7d, 14d
    #[arg(long, value_enum)]
    pub preset: Option<crate::time::TimePreset>,

    /// Widen the default 7-day time window to the last 400 days
    ///
    /// By default, runs query returns runs from the last 7 days.
    /// The LangSmith runs API rejects a window longer than 401 days;
    /// 400 days keeps a day of margin under that limit.
    #[arg(long)]
    pub no_time_filter: bool,

    /// Maximum number of runs to return (supports pagination)
    #[arg(short, long, default_value = "100")]
    pub limit: usize,

    /// Sort order for results
    ///
    /// The LangSmith runs API always returns the newest runs first, so
    /// `asc` fetches the newest --limit runs and prints them oldest first.
    #[arg(long, default_value = "desc", value_enum)]
    pub order: OrderArg,

    /// Output format for runs query
    ///
    /// Note: Uses `--output` to avoid conflict with global `-f/--format` flag
    #[arg(short = 'o', long = "output", default_value = "table", value_enum)]
    pub output: RunsOutputFormat,

    /// Fields to select (comma-separated)
    ///
    /// Limits the fields returned in the response. Without it, every field
    /// the API offers is returned. Table output always adds the fields its
    /// columns need.
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
    /// Ascending order (oldest first, among the newest --limit runs)
    Asc,
    /// Descending order (newest first)
    #[default]
    Desc,
}

/// Largest `page_size` the runs API accepts.
const MAX_PAGE_SIZE: u32 = 1000;

/// Days --no-time-filter asks for.
///
/// `POST /api/v2/runs/query` rejects a longer window with HTTP 400:
/// `time_range duration exceeds maximum of 401 days`. Neither the OpenAPI
/// spec nor LangChain's migration guide states this limit; a live request
/// returned that error. The CLI sends a start time and no end time, so the
/// API measures the window up to the moment the request arrives; 400 days
/// keeps a day of margin under the limit.
const NO_TIME_FILTER_DAYS: i64 = 400;

/// Fields the table columns read; added to --select for table output.
const TABLE_FIELDS: [RunSelectField; 7] = [
    RunSelectField::Id,
    RunSelectField::Name,
    RunSelectField::RunType,
    RunSelectField::Status,
    RunSelectField::TotalTokens,
    RunSelectField::StartTime,
    RunSelectField::EndTime,
];

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

impl RunRow {
    /// Create a RunRow from a Run, formatting time in the specified timezone.
    ///
    /// # Arguments
    ///
    /// * `run` - The run to convert
    /// * `tz` - The timezone to use for formatting the time column
    fn from_run_with_timezone(run: &QueriedRun, tz: &crate::time::ConfiguredTimezone) -> Self {
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

        // Format time in configured timezone (use start_time or "-")
        let time = run
            .start_time
            .map(|t| tz.format_datetime(t, "%Y-%m-%d %H:%M:%S"))
            .unwrap_or_else(|| "-".to_string());

        // Truncate name if too long (unicode-safe)
        let name = run.name.as_deref().unwrap_or("-");
        let name = if name.chars().count() > 30 {
            format!("{}...", name.chars().take(27).collect::<String>())
        } else {
            name.to_string()
        };

        // Format tokens
        let tokens = match run.total_tokens {
            Some(tokens) if tokens > 0 => tokens.to_string(),
            _ => "-".to_string(),
        };

        Self {
            id: run
                .id
                .map(|id| id.to_string().chars().take(8).collect::<String>()) // Short UUID
                .unwrap_or_else(|| "-".to_string()),
            name,
            run_type: run
                .run_type
                .map(|rt| format!("{:?}", rt).to_lowercase())
                .unwrap_or_else(|| "-".to_string()),
            status: run
                .status
                .as_deref()
                .map(str::to_lowercase)
                .unwrap_or_else(|| "-".to_string()),
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

    /// Resolve time filters with precedence rules.
    ///
    /// Precedence (highest to lowest):
    /// 1. --since/--until with ISO 8601 timestamps
    /// 2. --since with relative duration (e.g., "15m", "1h", "7d")
    /// 3. --preset
    /// 4. Default (7 days)
    ///
    /// --no-time-filter overrides all of these with the last
    /// [`NO_TIME_FILTER_DAYS`] days.
    ///
    /// Returns (start_time, end_time, description).
    fn resolve_time_filters(
        args: &QueryArgs,
        formatter: &OutputFormatter,
    ) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>, String) {
        let now = Utc::now();

        // The API searches only the last day when no start time is sent, so
        // --no-time-filter sends a start time NO_TIME_FILTER_DAYS ago.
        if args.no_time_filter {
            return (
                Some(now - chrono::Duration::days(NO_TIME_FILTER_DAYS)),
                None,
                format!("last {} days (--no-time-filter)", NO_TIME_FILTER_DAYS),
            );
        }

        // Parse --until first (it's always ISO 8601 if present)
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

        // Try to parse --since (relative duration or ISO 8601)
        if let Some(since_str) = &args.since {
            // Check if it looks like a relative duration
            if crate::time::is_relative_duration(since_str) {
                match crate::time::parse_relative_duration(since_str) {
                    Ok(duration) => {
                        let start = now - duration;
                        return (
                            Some(start),
                            end_time,
                            format!("last {} (relative)", since_str),
                        );
                    }
                    Err(e) => {
                        formatter.warning(&format!("Invalid --since duration: {}", e));
                        // Fall through to try ISO 8601
                    }
                }
            }

            // Try ISO 8601 format
            match DateTime::parse_from_rfc3339(since_str) {
                Ok(dt) => {
                    return (
                        Some(dt.with_timezone(&Utc)),
                        end_time,
                        format!("since {} (ISO 8601)", since_str),
                    );
                }
                Err(e) => {
                    formatter.warning(&format!(
                        "Invalid --since format: {}. Expected relative duration (e.g., 15m, 1h, 7d) or ISO 8601 (e.g., 2024-01-01T00:00:00Z)",
                        e
                    ));
                    // Fall through to preset or default
                }
            }
        }

        // Try --preset
        if let Some(preset) = args.preset {
            let duration = preset.to_duration();
            let start = now - duration;
            return (
                Some(start),
                end_time,
                format!("{} (preset)", preset.description()),
            );
        }

        // Default: last 7 days
        let default_duration = crate::time::TimePreset::default_preset().to_duration();
        let start = now - default_duration;
        (Some(start), end_time, "last 7 days (default)".to_string())
    }

    /// Fields to request from the API.
    ///
    /// Without --select, every field the API offers, so JSON output carries
    /// the whole run. With --select, the named fields, plus the ones the
    /// table columns read when the output is a table.
    fn resolve_selects(
        args: &QueryArgs,
        formatter: &OutputFormatter,
    ) -> Result<Vec<RunSelectField>> {
        let Some(select) = &args.select else {
            return Ok(RunSelectField::ALL.to_vec());
        };

        let mut fields = Vec::new();
        for name in select.split(',').map(str::trim).filter(|n| !n.is_empty()) {
            match RunSelectField::from_name(name) {
                Some(field) if !fields.contains(&field) => fields.push(field),
                Some(_) => {}
                None => {
                    return Err(crate::error::CliError::Other(anyhow::anyhow!(
                        "Unknown --select field '{}'. Valid fields: {}",
                        name,
                        RunSelectField::ALL
                            .iter()
                            .map(|f| {
                                serde_json::to_value(f)
                                    .ok()
                                    .and_then(|v| v.as_str().map(str::to_lowercase))
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }
            }
        }

        if args.output == RunsOutputFormat::Table {
            for field in TABLE_FIELDS {
                if !fields.contains(&field) {
                    fields.push(field);
                }
            }
        } else if fields.is_empty() {
            // The API returns only `id` when `selects` is left out, but the
            // CLI always sends `selects`, so it names the ID itself.
            formatter.warning("--select named no fields; returning run IDs only");
            fields.push(RunSelectField::Id);
        }

        Ok(fields)
    }

    /// Put runs in the requested order. The API always returns the newest
    /// runs first, so ascending order reverses them.
    fn order_runs(mut runs: Vec<QueriedRun>, order: OrderArg) -> Vec<QueriedRun> {
        if matches!(order, OrderArg::Asc) {
            runs.reverse();
        }
        runs
    }

    /// Runs to request per page: --limit, capped at the API maximum.
    fn page_size(limit: usize) -> u32 {
        limit.clamp(1, MAX_PAGE_SIZE as usize) as u32
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

        // Add raw filter (if provided)
        if let Some(raw_filter) = &args.filter {
            filter_builder = filter_builder.raw(raw_filter);
        }

        let combined_filter = filter_builder.build();

        // Parse project IDs/names (warn if not valid UUIDs)
        let project_ids: Vec<Uuid> = args
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

        if project_ids.is_empty() {
            return Err(crate::error::CliError::Other(anyhow::anyhow!(
                "runs query needs at least one project UUID: pass --project <UUID>. \
                 The LangSmith runs API cannot query across all projects."
            )));
        }

        // Parse time filters with precedence:
        // --since/--until (explicit) > --since (relative) > --preset > default (7d)
        //
        // If --no-time-filter is set, skip all time filtering.
        let (start_time, end_time, time_filter_source) =
            Self::resolve_time_filters(args, &formatter);

        let selects = Self::resolve_selects(args, &formatter)?;

        // Show query info (only for table output to keep JSON clean)
        if args.output == RunsOutputFormat::Table {
            formatter.info(&format!(
                "Querying runs from projects: {}",
                args.projects.join(", ")
            ));

            // Show time filter info
            formatter.info(&format!("Time filter: {}", time_filter_source));

            if let Some(filter) = &combined_filter {
                formatter.info(&format!("Filter: {}", filter));
            }

            formatter.info(&format!("Limit: {}", args.limit));
        }

        // Build the request (combined_filter is moved, not cloned)
        let request = QueryRunsRequest {
            project_ids: Some(project_ids),
            filter: combined_filter,
            trace_filter: args.trace_filter.clone(),
            tree_filter: args.tree_filter.clone(),
            is_root: if args.is_root { Some(true) } else { None },
            run_type: args.run_type.map(|rt| rt.into()),
            // The v2 filter language cannot compare `error`, so --errors-only
            // uses the request's has_error field.
            has_error: args.errors_only.then_some(true),
            min_start_time: start_time,
            max_start_time: end_time,
            selects: Some(selects),
            page_size: Some(Self::page_size(args.limit)),
            ..Default::default()
        };

        // Execute query with pagination
        let mut stream = client.query_runs_paginated(request, Some(args.limit));
        let mut runs: Vec<QueriedRun> = Vec::new();

        while let Some(result) = stream.next().await {
            runs.push(result?);
        }

        let runs = Self::order_runs(runs, args.order);

        // Output results
        match args.output {
            RunsOutputFormat::Table => {
                // Parse timezone from config string on each invocation.
                // Note: Caching in Config would require a non-serializable field and add complexity.
                // The parsing overhead is negligible (single string comparison + map lookup).
                let timezone = match crate::time::ConfiguredTimezone::parse(&config.timezone) {
                    Ok(tz) => tz,
                    Err(e) => {
                        formatter.warning(&format!("{}. Using UTC.", e));
                        crate::time::ConfiguredTimezone::Utc
                    }
                };
                let rows: Vec<RunRow> = runs
                    .iter()
                    .map(|r| RunRow::from_run_with_timezone(r, &timezone))
                    .collect();
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

    fn run_started_at(start: &str) -> QueriedRun {
        serde_json::from_value(serde_json::json!({
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "start_time": start
        }))
        .unwrap()
    }

    #[test]
    fn test_order_runs_desc_keeps_api_order() {
        let newest_first = vec![
            run_started_at("2024-01-03T00:00:00Z"),
            run_started_at("2024-01-02T00:00:00Z"),
            run_started_at("2024-01-01T00:00:00Z"),
        ];
        let ordered = RunsCommands::order_runs(newest_first, OrderArg::Desc);
        let days: Vec<u32> = ordered
            .iter()
            .map(|r| chrono::Datelike::day(&r.start_time.unwrap()))
            .collect();
        assert_eq!(days, vec![3, 2, 1]);
    }

    #[test]
    fn test_order_runs_asc_puts_oldest_first() {
        let newest_first = vec![
            run_started_at("2024-01-03T00:00:00Z"),
            run_started_at("2024-01-02T00:00:00Z"),
            run_started_at("2024-01-01T00:00:00Z"),
        ];
        let ordered = RunsCommands::order_runs(newest_first, OrderArg::Asc);
        let days: Vec<u32> = ordered
            .iter()
            .map(|r| chrono::Datelike::day(&r.start_time.unwrap()))
            .collect();
        assert_eq!(days, vec![1, 2, 3]);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // RunRow conversion tests
    // ═══════════════════════════════════════════════════════════════════════

    fn create_test_run(name: &str, total_tokens: i64) -> QueriedRun {
        serde_json::from_value(serde_json::json!({
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": name,
            "run_type": "LLM",
            "status": "SUCCESS",
            "project_id": "323e4567-e89b-12d3-a456-426614174002",
            "total_tokens": total_tokens
        }))
        .unwrap()
    }

    fn create_test_run_with_timing(start: &str, end: &str) -> QueriedRun {
        serde_json::from_value(serde_json::json!({
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "ChatOpenAI",
            "run_type": "LLM",
            "status": "SUCCESS",
            "start_time": start,
            "end_time": end
        }))
        .unwrap()
    }

    #[test]
    fn test_run_row_name_truncation_short() {
        let run = create_test_run("ShortName", 0);
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(row.name, "ShortName");
    }

    #[test]
    fn test_run_row_name_truncation_long() {
        let run = create_test_run("ThisIsAVeryLongNameThatShouldBeTruncated", 0);
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(row.name, "ThisIsAVeryLongNameThatShou...");
        assert_eq!(row.name.chars().count(), 30);
    }

    #[test]
    fn test_run_row_name_truncation_unicode() {
        // Test with emoji (multi-byte characters)
        let run = create_test_run("🚀🎉✨💡🔥⭐🌟🎯💫🌈🎊🎁", 0);
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        // Should not panic and should truncate properly
        assert!(row.name.chars().count() <= 30);
    }

    #[test]
    fn test_run_row_duration_milliseconds() {
        // 500ms duration
        let run =
            create_test_run_with_timing("2024-01-01T12:00:00.000Z", "2024-01-01T12:00:00.500Z");
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(row.duration, "500ms");
    }

    #[test]
    fn test_run_row_duration_seconds() {
        // 5 second duration
        let run = create_test_run_with_timing("2024-01-01T12:00:00Z", "2024-01-01T12:00:05Z");
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(row.duration, "5.00s");
    }

    #[test]
    fn test_run_row_tokens_display() {
        let run = create_test_run("Test", 150);
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(row.tokens, "150");
    }

    #[test]
    fn test_run_row_tokens_display_zero() {
        let run = create_test_run("Test", 0);
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(row.tokens, "-");
    }

    #[test]
    fn test_run_row_lowercases_v2_type_and_status() {
        let run = create_test_run("Test", 0);
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(row.run_type, "llm");
        assert_eq!(row.status, "success");
    }

    #[test]
    fn test_run_row_shows_dash_for_unselected_fields() {
        let run: QueriedRun =
            serde_json::from_str(r#"{"id": "123e4567-e89b-12d3-a456-426614174000"}"#).unwrap();
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        assert_eq!(
            (
                row.name.as_str(),
                row.run_type.as_str(),
                row.status.as_str(),
                row.tokens.as_str(),
                row.duration.as_str(),
                row.time.as_str()
            ),
            ("-", "-", "-", "-", "-", "-")
        );
    }

    #[test]
    fn test_run_row_uuid_truncation() {
        let run = create_test_run("Test", 0);
        let row = RunRow::from_run_with_timezone(&run, &crate::time::ConfiguredTimezone::Utc);
        // Should be first 8 chars of UUID
        assert_eq!(row.id, "123e4567");
        assert_eq!(row.id.len(), 8);
    }

    #[test]
    fn test_run_row_timezone_formatting_produces_different_outputs() {
        // Create a run with a known start time
        let run = create_test_run_with_timing("2024-06-15T14:30:00Z", "2024-06-15T14:31:00Z");

        // Format with UTC - should show 14:30
        let utc_tz = crate::time::ConfiguredTimezone::Utc;
        let row_utc = RunRow::from_run_with_timezone(&run, &utc_tz);

        // Format with America/New_York - should show 10:30 (EDT in June)
        let ny_tz = crate::time::ConfiguredTimezone::parse("America/New_York").unwrap();
        let row_ny = RunRow::from_run_with_timezone(&run, &ny_tz);

        // Format with Asia/Tokyo - should show 23:30 (JST = UTC+9)
        let tokyo_tz = crate::time::ConfiguredTimezone::parse("Asia/Tokyo").unwrap();
        let row_tokyo = RunRow::from_run_with_timezone(&run, &tokyo_tz);

        // Verify different timezones produce different time strings
        assert_ne!(row_utc.time, row_ny.time, "UTC and NY should differ");
        assert_ne!(row_utc.time, row_tokyo.time, "UTC and Tokyo should differ");
        assert_ne!(row_ny.time, row_tokyo.time, "NY and Tokyo should differ");

        // Verify the actual times are correct
        assert!(row_utc.time.contains("14:30"), "UTC should show 14:30");
        assert!(row_ny.time.contains("10:30"), "NY should show 10:30");
        assert!(row_tokyo.time.contains("23:30"), "Tokyo should show 23:30");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Pagination limit tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_page_size_follows_limit_below_max() {
        assert_eq!(RunsCommands::page_size(1), 1);
        assert_eq!(RunsCommands::page_size(100), 100);
        assert_eq!(RunsCommands::page_size(999), 999);
    }

    #[test]
    fn test_page_size_clamps_to_api_range() {
        // The SDK's query_runs_paginated fetches further pages past 1000.
        assert_eq!(RunsCommands::page_size(1000), 1000);
        assert_eq!(RunsCommands::page_size(5000), 1000);
        assert_eq!(RunsCommands::page_size(0), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // resolve_time_filters tests
    // ═══════════════════════════════════════════════════════════════════════

    /// Helper to create a minimal QueryArgs for testing
    fn create_test_query_args() -> QueryArgs {
        QueryArgs {
            projects: vec![],
            filter: None,
            trace_filter: None,
            tree_filter: None,
            is_root: false,
            tags: vec![],
            metadata: vec![],
            run_type: None,
            status: None,
            errors_only: false,
            since: None,
            until: None,
            preset: None,
            no_time_filter: false,
            limit: 100,
            order: OrderArg::Desc,
            output: RunsOutputFormat::Table,
            select: None,
            organization_id: None,
            workspace_id: None,
        }
    }

    #[test]
    fn test_resolve_time_filters_default_7_days() {
        let args = create_test_query_args();
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let (start, end, desc) = RunsCommands::resolve_time_filters(&args, &formatter);

        assert!(start.is_some(), "Default should have start time");
        assert!(end.is_none(), "Default should have no end time");
        assert_eq!(desc, "last 7 days (default)");

        // Verify it's approximately 7 days ago
        let now = chrono::Utc::now();
        let diff = now - start.unwrap();
        assert!(diff.num_days() >= 6 && diff.num_days() <= 7);
    }

    #[test]
    fn test_resolve_time_filters_no_time_filter() {
        let mut args = create_test_query_args();
        args.no_time_filter = true;
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let (start, end, desc) = RunsCommands::resolve_time_filters(&args, &formatter);

        // --no-time-filter asks for NO_TIME_FILTER_DAYS (400) days.
        let days = (chrono::Utc::now() - start.expect("--no-time-filter sends a start")).num_days();
        assert!(
            (399..=400).contains(&days),
            "expected ~400 days, got {}",
            days
        );
        assert!(end.is_none(), "--no-time-filter should have no end");
        assert_eq!(desc, "last 400 days (--no-time-filter)");
    }

    #[test]
    fn test_resolve_time_filters_preset() {
        let mut args = create_test_query_args();
        args.preset = Some(crate::time::TimePreset::ThreeHours);
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let (start, end, desc) = RunsCommands::resolve_time_filters(&args, &formatter);

        assert!(start.is_some(), "Preset should have start time");
        assert!(end.is_none(), "Preset should have no end time");
        assert_eq!(desc, "Last 3 hours (preset)");

        // Verify it's approximately 3 hours ago
        let now = chrono::Utc::now();
        let diff = now - start.unwrap();
        assert!(diff.num_hours() >= 2 && diff.num_hours() <= 3);
    }

    #[test]
    fn test_resolve_time_filters_since_relative() {
        let mut args = create_test_query_args();
        args.since = Some("2h".to_string());
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let (start, end, desc) = RunsCommands::resolve_time_filters(&args, &formatter);

        assert!(start.is_some(), "Relative --since should have start time");
        assert!(end.is_none(), "Relative --since should have no end time");
        assert_eq!(desc, "last 2h (relative)");

        // Verify it's approximately 2 hours ago
        let now = chrono::Utc::now();
        let diff = now - start.unwrap();
        assert!(diff.num_hours() >= 1 && diff.num_hours() <= 2);
    }

    #[test]
    fn test_resolve_time_filters_since_iso8601() {
        let mut args = create_test_query_args();
        args.since = Some("2024-06-15T14:30:00Z".to_string());
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let (start, end, desc) = RunsCommands::resolve_time_filters(&args, &formatter);

        assert!(start.is_some(), "ISO 8601 --since should have start time");
        assert!(end.is_none(), "ISO 8601 --since should have no end time");
        assert!(desc.contains("ISO 8601"));
    }

    #[test]
    fn test_resolve_time_filters_until_iso8601() {
        let mut args = create_test_query_args();
        args.until = Some("2024-06-15T14:30:00Z".to_string());
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let (start, end, desc) = RunsCommands::resolve_time_filters(&args, &formatter);

        // Default start still applies, but end should be set
        assert!(start.is_some(), "Should still have default start time");
        assert!(end.is_some(), "--until should set end time");
        assert_eq!(desc, "last 7 days (default)");
    }

    #[test]
    fn test_resolve_time_filters_precedence_since_over_preset() {
        let mut args = create_test_query_args();
        args.since = Some("1h".to_string());
        args.preset = Some(crate::time::TimePreset::SevenDays);
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let (start, _end, desc) = RunsCommands::resolve_time_filters(&args, &formatter);

        // --since should take precedence over --preset
        assert!(start.is_some());
        assert_eq!(desc, "last 1h (relative)");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // resolve_selects tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_resolve_selects_default_requests_every_field() {
        let args = create_test_query_args();
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let selects = RunsCommands::resolve_selects(&args, &formatter).unwrap();
        assert_eq!(selects, RunSelectField::ALL.to_vec());
    }

    #[test]
    fn test_resolve_selects_json_keeps_only_named_fields() {
        let mut args = create_test_query_args();
        args.output = RunsOutputFormat::Json;
        args.select = Some("id, total_tokens,ID".to_string());
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Json);
        let selects = RunsCommands::resolve_selects(&args, &formatter).unwrap();
        assert_eq!(
            selects,
            vec![RunSelectField::Id, RunSelectField::TotalTokens]
        );
    }

    #[test]
    fn test_resolve_selects_table_adds_column_fields() {
        let mut args = create_test_query_args();
        args.select = Some("tags".to_string());
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let selects = RunsCommands::resolve_selects(&args, &formatter).unwrap();
        assert_eq!(selects[0], RunSelectField::Tags);
        for field in TABLE_FIELDS {
            assert!(selects.contains(&field), "table needs {:?}", field);
        }
        assert_eq!(selects.len(), 1 + TABLE_FIELDS.len());
    }

    #[test]
    fn test_resolve_selects_empty_json_asks_for_id() {
        let mut args = create_test_query_args();
        args.output = RunsOutputFormat::Json;
        args.select = Some(" , ".to_string());
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Json);
        let selects = RunsCommands::resolve_selects(&args, &formatter).unwrap();
        assert_eq!(selects, vec![RunSelectField::Id]);
    }

    #[test]
    fn test_resolve_selects_rejects_unknown_field() {
        let mut args = create_test_query_args();
        args.select = Some("id,session_id".to_string());
        let formatter = crate::output::OutputFormatter::new(crate::output::OutputFormat::Table);
        let err = RunsCommands::resolve_selects(&args, &formatter)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("Unknown --select field 'session_id'"),
            "{}",
            err
        );
        assert!(err.contains("project_id"), "{}", err);
    }
}
