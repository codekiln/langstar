//! CLI tests for `langstar runs` commands.
//!
//! These tests verify:
//! - CLI argument parsing and validation
//! - Help text completeness
//! - Output format options
//! - Error handling for invalid inputs
//! - Filter flag combinations
//! - Integration with LangSmith API (when credentials available)
//!
//! **Test Categories:**
//!
//! 1. **Unit tests** (no API access): Verify CLI parsing, help text, error handling
//! 2. **Integration tests** (requires API): Verify actual runs query behavior
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY` environment variable
//! - `LANGSMITH_ORGANIZATION_ID` environment variable
//! - `LANGSMITH_WORKSPACE_ID` environment variable
//!
//! Run with: `cargo test --test runs_command_test`

#[path = "common/home.rs"]
mod home;

use assert_cmd::Command;
use chrono::Utc;
use futures_util::StreamExt;
use langstar_sdk::{AuthConfig, LangchainClient, ProjectCreate, QueryRunsRequest, RunSelectField};
use predicates::prelude::*;
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Helper function to get a CLI command builder
fn langstar_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
    cmd.env("HOME", home::empty_home());
    cmd
}

// ═══════════════════════════════════════════════════════════════════════════
// Help and Documentation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Query and manage LangSmith runs"))
        .stdout(predicate::str::contains("query"));
}

#[test]
fn test_runs_query_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--help"]);

    cmd.assert()
        .success()
        // Command description
        .stdout(predicate::str::contains("Query runs with filtering"))
        // Filter options
        .stdout(predicate::str::contains("--filter"))
        .stdout(predicate::str::contains("--tag"))
        .stdout(predicate::str::contains("--meta"))
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--errors-only"))
        // Project filter
        .stdout(predicate::str::contains("--project"))
        // Time filters
        .stdout(predicate::str::contains("--since"))
        .stdout(predicate::str::contains("--until"))
        // Type filters
        .stdout(predicate::str::contains("--run-type"))
        .stdout(predicate::str::contains("--is-root"))
        // Output options
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--order"))
        // Scoping options
        .stdout(predicate::str::contains("--organization-id"))
        .stdout(predicate::str::contains("--workspace-id"));
}

#[test]
fn test_runs_query_help_shows_run_types() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--help"]);

    cmd.assert()
        .success()
        // Run type values should be shown
        .stdout(predicate::str::contains("llm"))
        .stdout(predicate::str::contains("chain"))
        .stdout(predicate::str::contains("tool"));
}

#[test]
fn test_runs_query_help_shows_output_formats() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--help"]);

    cmd.assert()
        .success()
        // Output format options
        .stdout(predicate::str::contains("table"))
        .stdout(predicate::str::contains("json"))
        .stdout(predicate::str::contains("json-pretty"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Argument Validation Tests (No API Access Required)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_invalid_run_type() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--run-type", "invalid-type"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_runs_query_invalid_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "xml"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_runs_query_invalid_order() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--order", "random"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_runs_query_invalid_limit_not_number() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--limit", "abc"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Filter Flag Combinations (Parsing Tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_accepts_multiple_tags() {
    // This test verifies the CLI accepts multiple --tag flags
    // It will fail at runtime without API key, but the parsing should succeed
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--tag",
        "production",
        "--tag",
        "gpt-4",
        "--limit",
        "1",
    ]);

    // Without API key, this will fail but not due to parsing
    let output = cmd.output().expect("Failed to execute command");

    // Should not be a clap parsing error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept multiple --tag flags"
    );
    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept multiple --tag flags"
    );
}

#[test]
fn test_runs_query_accepts_multiple_meta() {
    // Verify CLI accepts multiple --meta flags
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--meta",
        "environment=prod",
        "--meta",
        "model=gpt-4",
        "--limit",
        "1",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept multiple --meta flags"
    );
}

#[test]
fn test_runs_query_accepts_multiple_projects() {
    // Verify CLI accepts multiple --project flags
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--project",
        "00000000-0000-0000-0000-000000000001",
        "--project",
        "00000000-0000-0000-0000-000000000002",
        "--limit",
        "1",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept multiple --project flags"
    );
}

#[test]
fn test_runs_query_accepts_combined_filters() {
    // Verify CLI accepts a combination of filter flags
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "--tag",
        "production",
        "--status",
        "error",
        "--run-type",
        "llm",
        "--is-root",
        "--errors-only",
        "--filter",
        "gt(total_tokens, 100)",
        "--limit",
        "1",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("unexpected argument"),
        "CLI should accept combined filter flags"
    );
    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept combined filter flags"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Output Format Tests (Parsing Only)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_accepts_table_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "table", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept --output table"
    );
}

#[test]
fn test_runs_query_accepts_json_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "json", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept --output json"
    );
}

#[test]
fn test_runs_query_accepts_json_pretty_format() {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "--output", "json-pretty", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("invalid value"),
        "CLI should accept --output json-pretty"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// All Run Types Accepted Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_accepts_all_run_types() {
    let run_types = [
        "tool",
        "chain",
        "llm",
        "retriever",
        "embedding",
        "prompt",
        "parser",
    ];

    for run_type in run_types {
        let mut cmd = langstar_cmd();
        cmd.args(["runs", "query", "--run-type", run_type, "--limit", "1"]);

        let output = cmd.output().expect("Failed to execute command");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !stderr.contains("invalid value"),
            "CLI should accept --run-type {}",
            run_type
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Error Handling Tests (No API Access)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_runs_query_without_project_errors() {
    // The runs API cannot query across all projects, so the CLI stops before
    // sending a request and says what to pass.
    let mut cmd = langstar_cmd();
    cmd.env("LANGSMITH_API_KEY", "lsv2_dummy_key_for_test");
    cmd.args(["runs", "query", "--limit", "1"]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "should exit non-zero");
    assert!(
        stderr.contains("needs at least one project UUID") && stderr.contains("--project <UUID>"),
        "should ask for --project: {}",
        stderr
    );
}

#[test]
fn test_runs_query_unknown_select_field_errors() {
    let mut cmd = langstar_cmd();
    cmd.env("LANGSMITH_API_KEY", "lsv2_dummy_key_for_test");
    cmd.args([
        "runs",
        "query",
        "--project",
        "00000000-0000-0000-0000-000000000001",
        "--select",
        "id,session_id",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "should exit non-zero");
    assert!(
        stderr.contains("Unknown --select field 'session_id'"),
        "should name the bad field: {}",
        stderr
    );
}

#[test]
fn test_runs_query_without_api_key() {
    let mut cmd = langstar_cmd();
    cmd.env_remove("LANGSMITH_API_KEY");
    cmd.args([
        "runs",
        "query",
        "--project",
        "00000000-0000-0000-0000-000000000001",
        "--limit",
        "1",
    ]);

    let output = cmd.output().expect("Failed to execute command");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "should exit non-zero");
    assert!(
        stderr.contains("LANGSMITH_API_KEY") || stderr.contains("API key"),
        "Should mention the missing API key: {}",
        stderr
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Integration Tests (Require API Access)
// ═══════════════════════════════════════════════════════════════════════════

/// Deletes the test project, and the runs in it, when the test ends.
struct ProjectCleanup {
    client: LangchainClient,
    project_id: Uuid,
}

impl Drop for ProjectCleanup {
    fn drop(&mut self) {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        if let Err(e) = runtime.block_on(self.client.delete_project(self.project_id)) {
            eprintln!("[CLEANUP] Failed to delete test project: {}", e);
        }
    }
}

/// One run to ingest: name, run type, minutes before now it started, error.
struct SeedRun {
    name: &'static str,
    run_type: &'static str,
    minutes_ago: i64,
    error: Option<&'static str>,
}

const SEED_RUNS: [SeedRun; 3] = [
    SeedRun {
        name: "seed-oldest-chain",
        run_type: "chain",
        minutes_ago: 30,
        error: None,
    },
    SeedRun {
        name: "seed-middle-llm",
        run_type: "llm",
        minutes_ago: 20,
        error: None,
    },
    SeedRun {
        name: "seed-newest-failed",
        run_type: "chain",
        minutes_ago: 10,
        error: Some("seeded failure"),
    },
];

/// Posts one root run to the ingest endpoint, `POST /api/v1/runs`.
async fn ingest_run(client: &LangchainClient, project_name: &str, seed: &SeedRun) {
    let id = Uuid::new_v4();
    let start = Utc::now() - chrono::Duration::minutes(seed.minutes_ago);
    let end = start + chrono::Duration::milliseconds(250);
    let dotted_order = format!("{}{}", start.format("%Y%m%dT%H%M%S%6fZ"), id);

    let body = json!({
        "id": id,
        "trace_id": id,
        "dotted_order": dotted_order,
        "name": seed.name,
        "run_type": seed.run_type,
        "session_name": project_name,
        "start_time": start.to_rfc3339(),
        "end_time": end.to_rfc3339(),
        "inputs": {"question": "seed"},
        "outputs": {"answer": "seed"},
        "error": seed.error,
    });

    let response = client
        .langsmith_post("/api/v1/runs")
        .expect("Failed to build ingest request")
        .json(&body)
        .send()
        .await
        .expect("Failed to send ingest request");
    assert!(
        response.status().is_success(),
        "Ingesting run '{}' failed with HTTP {}",
        seed.name,
        response.status()
    );
}

/// Runs `langstar runs query -p <project> <extra args>` and parses its JSON output.
fn query_runs_json(project_id: &str, extra: &[&str]) -> Vec<Value> {
    let mut cmd = langstar_cmd();
    cmd.args(["runs", "query", "-p", project_id, "--since", "1h"]);
    cmd.args(extra);
    let output = cmd.output().expect("Failed to execute CLI");
    assert!(
        output.status.success(),
        "runs query {:?} failed: {}",
        extra,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("Output should be a JSON array")
}

fn names(runs: &[Value]) -> Vec<&str> {
    runs.iter()
        .map(|r| r["name"].as_str().expect("run should carry a name"))
        .collect()
}

/// Seeds runs into a new project, then checks `runs query` against them.
///
/// Pattern: CREATE (SDK) → INGEST runs → READ (CLI) → VERIFY → DELETE
#[test]
fn test_runs_query_lifecycle() {
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let auth = AuthConfig::from_env().expect("Auth config required");
    let client = LangchainClient::new(auth).expect("SDK client required");

    // ── CREATE ──
    let project_name = format!("test-runs-query-{}", &Uuid::new_v4().to_string()[..12]);
    let project = runtime
        .block_on(client.create_project(ProjectCreate {
            name: Some(project_name.clone()),
            description: Some("Test project for runs query lifecycle".to_string()),
            ..Default::default()
        }))
        .expect("Failed to create test project");
    let _cleanup = ProjectCleanup {
        client: client.clone(),
        project_id: project.id,
    };
    let project_id = project.id.to_string();
    println!("[CREATE] Created test project");

    // ── INGEST ──
    runtime.block_on(async {
        for seed in &SEED_RUNS {
            ingest_run(&client, &project_name, seed).await;
        }
    });
    println!("[INGEST] Posted {} runs", SEED_RUNS.len());

    // ── READ: wait until every seeded run is queryable ──
    let deadline = Instant::now() + Duration::from_secs(180);
    let all = loop {
        let runs = query_runs_json(&project_id, &["-o", "json"]);
        if runs.len() >= SEED_RUNS.len() || Instant::now() > deadline {
            break runs;
        }
        std::thread::sleep(Duration::from_secs(5));
    };
    println!("[READ] Query returned {} runs", all.len());

    // ── VERIFY: default order is newest first, with v2 field names ──
    assert_eq!(
        names(&all),
        vec!["seed-newest-failed", "seed-middle-llm", "seed-oldest-chain"],
        "default --order desc should list newest first"
    );
    for run in &all {
        assert_eq!(run["project_id"].as_str(), Some(project_id.as_str()));
        assert!(run.get("session_id").is_none(), "v2 has no session_id");
        assert!(run["start_time"].is_string() && run["end_time"].is_string());
    }
    assert_eq!(all[1]["run_type"], "llm");
    assert_eq!(
        all[0]["status"].as_str().map(str::to_lowercase).as_deref(),
        Some("error")
    );
    assert_eq!(
        all[1]["status"].as_str().map(str::to_lowercase).as_deref(),
        Some("success")
    );
    println!("[VERIFY] JSON output carries the v2 fields");

    // --order asc: oldest first
    let asc = query_runs_json(&project_id, &["-o", "json", "--order", "asc"]);
    assert_eq!(
        names(&asc),
        vec!["seed-oldest-chain", "seed-middle-llm", "seed-newest-failed"]
    );

    // --limit with --order asc: the newest N, printed oldest first
    let asc2 = query_runs_json(&project_id, &["-o", "json", "--order", "asc", "-l", "2"]);
    assert_eq!(names(&asc2), vec!["seed-middle-llm", "seed-newest-failed"]);

    // --limit 1: the CLI sets page_size to 1 and stops after the first run
    let one = query_runs_json(&project_id, &["-o", "json", "-l", "1"]);
    assert_eq!(names(&one), vec!["seed-newest-failed"]);

    // Paging: the CLI only pages past 1000 runs, so ask the SDK for one run
    // per page and check that it follows next_cursor through all three.
    let paged: Vec<String> = runtime.block_on(async {
        let request = QueryRunsRequest {
            project_ids: Some(vec![project.id]),
            min_start_time: Some(Utc::now() - chrono::Duration::hours(1)),
            page_size: Some(1),
            selects: Some(vec![RunSelectField::Name]),
            ..Default::default()
        };
        let mut stream = client.query_runs_paginated(request, None);
        let mut names = Vec::new();
        while let Some(run) = stream.next().await {
            names.push(run.expect("page request failed").name.unwrap_or_default());
        }
        names
    });
    assert_eq!(
        paged,
        vec!["seed-newest-failed", "seed-middle-llm", "seed-oldest-chain"],
        "one run per page should still reach every run"
    );

    // --run-type
    let llm = query_runs_json(&project_id, &["-o", "json", "--run-type", "llm"]);
    assert_eq!(names(&llm), vec!["seed-middle-llm"]);

    // --errors-only sends has_error: true; --status adds eq(status, ...) to the filter
    let errors = query_runs_json(&project_id, &["-o", "json", "--errors-only"]);
    assert_eq!(names(&errors), vec!["seed-newest-failed"]);
    let success = query_runs_json(&project_id, &["-o", "json", "--status", "success"]);
    assert_eq!(
        names(&success),
        vec!["seed-middle-llm", "seed-oldest-chain"]
    );

    // --select returns only the named fields in JSON
    let selected = query_runs_json(&project_id, &["-o", "json", "--select", "id,name"]);
    assert_eq!(selected.len(), SEED_RUNS.len());
    for run in &selected {
        let mut keys: Vec<&str> = run
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort();
        assert_eq!(keys, vec!["id", "name"]);
    }

    // --select without id: the runs come back without one
    let no_id = query_runs_json(&project_id, &["-o", "json", "--select", "name"]);
    assert_eq!(
        names(&no_id),
        vec!["seed-newest-failed", "seed-middle-llm", "seed-oldest-chain"]
    );
    for run in &no_id {
        assert!(run.get("id").is_none(), "id was not selected: {}", run);
    }
    println!("[VERIFY] Order, limit, run type, status and select flags work");

    // Table output, even with a narrow --select, fills its columns
    let mut cmd = langstar_cmd();
    cmd.args([
        "runs",
        "query",
        "-p",
        &project_id,
        "--since",
        "1h",
        "--select",
        "id",
    ]);
    let output = cmd.output().expect("Failed to execute CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("seed-middle-llm"), "table: {}", stdout);
    assert!(
        stdout.contains("llm") && stdout.contains("success"),
        "table: {}",
        stdout
    );
    assert!(stdout.contains("Found 3 runs"), "table: {}", stdout);
    println!("[VERIFY] Table output lists the seeded runs");
}
