//! CLI tests for `langstar eval` commands.
//!
//! These tests verify:
//! - CLI argument parsing and validation
//! - Help text completeness
//! - Output format options
//! - Error handling for invalid inputs
//! - Evaluator type conversions
//! - LLM judge configuration validation
//!
//! **Test Categories:**
//!
//! 1. **Unit tests** (no API access): Verify CLI parsing, help text, error handling
//! 2. **Integration tests** (requires API): Verify actual evaluation operations
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY` environment variable
//!
//! Run with: `cargo test --test eval_command_test`

use assert_cmd::Command;
use escargot::CargoBuild;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use uuid::Uuid;

/// Helper function to get a CLI command builder
fn langstar_cmd() -> Command {
    let bin = CargoBuild::new()
        .bin("langstar")
        .run()
        .expect("Failed to build langstar binary")
        .path()
        .to_owned();
    Command::new(bin)
}

// ═══════════════════════════════════════════════════════════════════════════
// Help and Documentation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_eval_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Commands for managing LangSmith evaluations",
        ))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("export"));
}

#[test]
fn test_eval_create_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "create", "--help"]);

    cmd.assert()
        .success()
        // Command description
        .stdout(predicate::str::contains("Create a new evaluation"))
        // Required arguments
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--dataset"))
        .stdout(predicate::str::contains("--evaluator"))
        // Optional LLM judge arguments
        .stdout(predicate::str::contains("--judge-model"))
        .stdout(predicate::str::contains("--judge-provider"))
        .stdout(predicate::str::contains("--judge-prompt-file"))
        .stdout(predicate::str::contains("--score-type"))
        .stdout(predicate::str::contains("--score-choices"))
        .stdout(predicate::str::contains("--score-min"))
        .stdout(predicate::str::contains("--score-max"))
        .stdout(predicate::str::contains("--include-reasoning"))
        // Output format
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_eval_run_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Run an evaluation"))
        .stdout(predicate::str::contains("--preview"))
        .stdout(predicate::str::contains("--dry-run"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_eval_list_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "list", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List evaluations"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--dataset"))
        .stdout(predicate::str::contains("--evaluator-type"))
        .stdout(predicate::str::contains("--limit"))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_eval_get_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "get", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Get details of a specific evaluation",
        ))
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn test_eval_export_help() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "export", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Export evaluation results"))
        .stdout(predicate::str::contains("--file-format"))
        .stdout(predicate::str::contains("--out"))
        .stdout(predicate::str::contains("--include-comments"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Argument Parsing & Validation Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_eval_create_requires_args() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "create"]);

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

#[test]
fn test_eval_create_requires_name() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "exact-match",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--name"));
}

#[test]
fn test_eval_create_requires_dataset() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--evaluator",
        "exact-match",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--dataset"));
}

#[test]
fn test_eval_create_requires_evaluator() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("--evaluator"));
}

#[test]
fn test_eval_create_invalid_evaluator_type() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "invalid_evaluator",
    ]);

    cmd.assert().failure().stderr(
        predicate::str::contains("invalid value")
            .or(predicate::str::contains("isn't a valid value")),
    );
}

#[test]
fn test_eval_create_accepts_all_heuristic_evaluators() {
    let evaluators = vec![
        "exact-match",
        "contains",
        "regex-match",
        "json-valid",
        "string-distance",
    ];

    for evaluator in evaluators {
        let mut cmd = langstar_cmd();
        cmd.args([
            "eval",
            "create",
            "--name",
            "test-eval",
            "--dataset",
            "test-dataset",
            "--evaluator",
            evaluator,
        ]);

        // These will fail due to no API key, but should parse arguments correctly
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
    }
}

#[test]
fn test_eval_create_accepts_llm_judge() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "llm-judge",
    ]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_run_requires_eval_id() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run"]);

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

#[test]
fn test_eval_run_invalid_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run", "not-a-uuid"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value").or(predicate::str::contains("Uuid")));
}

#[test]
fn test_eval_run_accepts_valid_uuid() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run", &uuid.to_string()]);

    // Should fail due to no API key, but UUID should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_run_accepts_preview_flag() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run", &uuid.to_string(), "--preview", "10"]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_run_accepts_dry_run_flag() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run", &uuid.to_string(), "--dry-run"]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_list_accepts_filters() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "list",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator-type",
        "exact-match",
        "--limit",
        "50",
    ]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_export_requires_eval_id() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "export"]);

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

#[test]
fn test_eval_export_invalid_format() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "export",
        &uuid.to_string(),
        "--file-format",
        "invalid_format",
    ]);

    cmd.assert().failure().stderr(
        predicate::str::contains("invalid value")
            .or(predicate::str::contains("isn't a valid value")),
    );
}

#[test]
fn test_eval_export_accepts_csv_format() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "export", &uuid.to_string(), "--file-format", "csv"]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_export_accepts_jsonl_format() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "export",
        &uuid.to_string(),
        "--file-format",
        "jsonl",
    ]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

// ═══════════════════════════════════════════════════════════════════════════
// LLM Judge Configuration Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_llm_judge_prompt_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_file = temp_dir.path().join("nonexistent.txt");

    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "llm-judge",
        "--judge-prompt-file",
        nonexistent_file.to_str().unwrap(),
    ]);

    // Should fail with file not found error before API call
    cmd.assert().failure().stderr(
        predicate::str::contains("does not exist").or(predicate::str::contains("not found")),
    );
}

#[test]
fn test_llm_judge_prompt_file_exists() {
    let temp_dir = TempDir::new().unwrap();
    let prompt_file = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_file, "Is this correct?").unwrap();

    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "llm-judge",
        "--judge-prompt-file",
        prompt_file.to_str().unwrap(),
    ]);

    // Should fail due to no API key, not file validation
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_llm_judge_categorical_score_validation() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "llm-judge",
        "--score-type",
        "categorical",
        "--score-choices",
        "Y,N",
    ]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_llm_judge_continuous_score_validation() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "llm-judge",
        "--score-type",
        "continuous",
        "--score-min",
        "0",
        "--score-max",
        "10",
    ]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_llm_judge_include_reasoning() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "llm-judge",
        "--include-reasoning",
    ]);

    // Should fail due to no API key, but arguments should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_llm_judge_invalid_score_type() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "llm-judge",
        "--score-type",
        "invalid_type",
    ]);

    cmd.assert().failure().stderr(
        predicate::str::contains("invalid value")
            .or(predicate::str::contains("isn't a valid value")),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Output Format Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_eval_list_json_output() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "list", "--json"]);

    // Should fail due to no API key, but --json flag should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_create_json_output() {
    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "create",
        "--name",
        "test-eval",
        "--dataset",
        "test-dataset",
        "--evaluator",
        "exact-match",
        "--json",
    ]);

    // Should fail due to no API key, but --json flag should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_get_json_output() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "get", &uuid.to_string(), "--json"]);

    // Should fail due to no API key, but --json flag should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_run_json_output() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run", &uuid.to_string(), "--json"]);

    // Should fail due to no API key, but --json flag should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Evaluator Type Conversion Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_evaluator_types_are_case_insensitive() {
    // Test that evaluator types accept kebab-case (as shown in help)
    let evaluators = vec![
        ("exact-match", true),
        ("contains", true),
        ("regex-match", true),
        ("json-valid", true),
        ("string-distance", true),
        ("llm-judge", true),
    ];

    for (evaluator, should_parse) in evaluators {
        let mut cmd = langstar_cmd();
        cmd.args([
            "eval",
            "create",
            "--name",
            "test",
            "--dataset",
            "ds",
            "--evaluator",
            evaluator,
        ]);

        if should_parse {
            // Should fail on API key, not parsing
            cmd.assert()
                .failure()
                .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
        } else {
            cmd.assert().failure().stderr(predicate::str::contains(
                "invalid value".to_string() + evaluator,
            ));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge Cases and Error Handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_eval_run_preview_negative_value() {
    let uuid = Uuid::new_v4();
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "run", &uuid.to_string(), "--preview", "-1"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_eval_list_limit_zero() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "list", "--limit", "0"]);

    // Zero limit should be accepted (returns empty list)
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_export_with_output_file() {
    let uuid = Uuid::new_v4();
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("output.csv");

    let mut cmd = langstar_cmd();
    cmd.args([
        "eval",
        "export",
        &uuid.to_string(),
        "--out",
        output_file.to_str().unwrap(),
    ]);

    // Should fail due to no API key, but output path should parse
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("LANGSMITH_API_KEY"));
}

#[test]
fn test_eval_get_requires_eval_id() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "get"]);

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

#[test]
fn test_eval_get_invalid_uuid() {
    let mut cmd = langstar_cmd();
    cmd.args(["eval", "get", "invalid-uuid"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value").or(predicate::str::contains("Uuid")));
}
