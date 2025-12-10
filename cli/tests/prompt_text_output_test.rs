use assert_cmd::Command;
use escargot::CargoBuild;
use predicates::prelude::*;

/// CLI Integration tests for text output format (`-f text`) with column selection
///
/// These tests verify that the langstar CLI correctly:
/// 1. Produces tab-separated output with `-f text`
/// 2. Supports `--columns` flag for field selection
/// 3. Supports `--show-columns` flag for column discovery
/// 4. Respects `LANGSTAR_OUTPUT_FORMAT` environment variable
/// 5. Produces actual TSV content (not just exit codes)
///
/// **Prerequisites:**
/// 1. Valid LANGSMITH_API_KEY environment variable
/// 2. Valid LANGSMITH_ORGANIZATION_ID environment variable (for scoped operations)
///
/// These tests run automatically in CI with configured secrets.
/// Run locally with: cargo test --test prompt_text_output_test
///
/// **Testing Philosophy:**
/// Following principles from #556 and milestone #14:
/// - Verify actual output content, not just exit codes
/// - Assert on tab separators, row counts, field values
/// - No "anemic tests" that only check success()
///
/// Reference: Issue #587, Parent #584

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

/// Helper function to get organization ID from environment
///
/// Integration tests MUST have LANGSMITH_ORGANIZATION_ID set. Panicking ensures
/// tests fail loudly instead of silently skipping (see issue #647).
fn get_org_id() -> String {
    std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests")
}

#[test]
fn test_prompt_list_text_output_basic() {
    // Test basic text output format with all columns
    let org_id = get_org_id();

    println!("Testing prompt list -f text with org ID: {}", org_id);

    let mut cmd = langstar_cmd();
    cmd.args([
        "prompt",
        "list",
        "-f",
        "text",
        "--limit",
        "5",
        "--organization-id",
        &org_id,
        "--public", // Public prompts to ensure we get results
    ]);

    // Run the command and capture output
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify actual TSV output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have at least one line of output (or "No results found.")
    assert!(!lines.is_empty(), "Expected output, got empty stdout");

    // If we have results, verify TSV format
    if lines[0] != "No results found." {
        // Each line should have tab-separated values
        for (i, line) in lines.iter().enumerate() {
            // Should contain tabs for field separation
            assert!(
                line.contains('\t'),
                "Line {} should contain tabs for TSV format: {}",
                i,
                line
            );

            // Count fields (should match PROMPT_COLUMNS length = 6)
            let field_count = line.split('\t').count();
            assert_eq!(
                field_count, 6,
                "Line {} should have 6 tab-separated fields, got {}: {}",
                i, field_count, line
            );
        }
    }

    println!("✓ CLI produced valid tab-separated text output");
}

#[test]
fn test_prompt_list_text_output_single_column() {
    // Test --columns flag with single field
    let org_id = get_org_id();

    println!("Testing prompt list -f text --columns handle");

    let mut cmd = langstar_cmd();
    cmd.args([
        "prompt",
        "list",
        "-f",
        "text",
        "--columns",
        "handle",
        "--limit",
        "5",
        "--organization-id",
        &org_id,
        "--public",
    ]);

    // Run the command and capture output
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have output
    assert!(!lines.is_empty(), "Expected output, got empty stdout");

    // If we have results, verify single-column format
    if lines[0] != "No results found." {
        for (i, line) in lines.iter().enumerate() {
            // Single column should have NO tabs
            assert!(
                !line.contains('\t'),
                "Line {} should not contain tabs (single column): {}",
                i,
                line
            );

            // Should not be empty
            assert!(!line.trim().is_empty(), "Line {} should not be empty", i);
        }
    }

    println!("✓ CLI produced valid single-column text output");
}

#[test]
fn test_prompt_list_text_output_multiple_columns() {
    // Test --columns flag with multiple fields
    let org_id = get_org_id();

    println!("Testing prompt list -f text --columns handle,likes,downloads");

    let mut cmd = langstar_cmd();
    cmd.args([
        "prompt",
        "list",
        "-f",
        "text",
        "--columns",
        "handle,likes,downloads",
        "--limit",
        "5",
        "--organization-id",
        &org_id,
        "--public",
    ]);

    // Run the command and capture output
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have output
    assert!(!lines.is_empty(), "Expected output, got empty stdout");

    // If we have results, verify three-column format
    if lines[0] != "No results found." {
        for (i, line) in lines.iter().enumerate() {
            // Should contain tabs
            assert!(
                line.contains('\t'),
                "Line {} should contain tabs for TSV format: {}",
                i,
                line
            );

            // Count fields (should be exactly 3)
            let field_count = line.split('\t').count();
            assert_eq!(
                field_count, 3,
                "Line {} should have 3 tab-separated fields, got {}: {}",
                i, field_count, line
            );
        }
    }

    println!("✓ CLI produced valid multi-column text output");
}

#[test]
fn test_prompt_list_show_columns() {
    // Test --show-columns flag for column discovery
    println!("Testing prompt list --show-columns");

    let mut cmd = langstar_cmd();
    cmd.args(["prompt", "list", "--show-columns"]);

    // Run the command and capture output
    let assert = cmd.assert();

    // Should succeed
    assert
        .success()
        .stdout(predicate::str::contains(
            "Available columns for prompt list:",
        ))
        .stdout(predicate::str::contains("handle"))
        .stdout(predicate::str::contains("likes"))
        .stdout(predicate::str::contains("downloads"))
        .stdout(predicate::str::contains("public"))
        .stdout(predicate::str::contains("description"))
        .stdout(predicate::str::contains("created_at"))
        .stdout(predicate::str::contains(
            "Usage: langstar prompt list -f text --columns",
        ));

    println!("✓ CLI showed available columns correctly");
}

#[test]
fn test_prompt_list_invalid_column() {
    // Test that invalid column names produce clear error messages
    let org_id = get_org_id();

    println!("Testing prompt list with invalid column name");

    let mut cmd = langstar_cmd();
    cmd.args([
        "prompt",
        "list",
        "-f",
        "text",
        "--columns",
        "invalid_column",
        "--organization-id",
        &org_id,
    ]);

    // Run the command and capture output
    let assert = cmd.assert();

    // Should fail with helpful error message
    assert
        .failure()
        .stderr(predicate::str::contains("Invalid column"))
        .stderr(predicate::str::contains("invalid_column"))
        .stderr(predicate::str::contains("Available columns:"));

    println!("✓ CLI rejected invalid column with helpful error");
}

#[test]
fn test_prompt_list_env_var_output_format() {
    // Test that LANGSTAR_OUTPUT_FORMAT environment variable works
    let org_id = get_org_id();

    println!("Testing prompt list with LANGSTAR_OUTPUT_FORMAT=text");

    let mut cmd = langstar_cmd();
    cmd.env("LANGSTAR_OUTPUT_FORMAT", "text");
    cmd.args([
        "prompt",
        "list",
        "--limit",
        "5",
        "--organization-id",
        &org_id,
        "--public",
    ]);

    // Run the command and capture output
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify TSV output (or "No results found.")
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "Expected output, got empty stdout");

    // If we have results, verify TSV format
    if lines[0] != "No results found." {
        // First line should have tabs
        assert!(
            lines[0].contains('\t'),
            "First line should contain tabs for TSV format"
        );
    }

    println!("✓ CLI respected LANGSTAR_OUTPUT_FORMAT environment variable");
}

#[test]
fn test_prompt_list_text_output_field_validation() {
    // Test that text output contains expected field types/formats
    let org_id = get_org_id();

    println!("Testing prompt list -f text field validation");

    let mut cmd = langstar_cmd();
    cmd.args([
        "prompt",
        "list",
        "-f",
        "text",
        "--columns",
        "handle,likes,downloads,public",
        "--limit",
        "5",
        "--organization-id",
        &org_id,
        "--public",
    ]);

    // Run the command and capture output
    let output = cmd.output().expect("Failed to execute command");

    // Should succeed
    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have output
    assert!(!lines.is_empty(), "Expected output, got empty stdout");

    // If we have results, verify field types
    if lines[0] != "No results found." {
        for (i, line) in lines.iter().enumerate() {
            let fields: Vec<&str> = line.split('\t').collect();

            // Should have 4 fields
            assert_eq!(
                fields.len(),
                4,
                "Line {} should have 4 fields, got {}: {}",
                i,
                fields.len(),
                line
            );

            // Field 0: handle - should not be empty
            assert!(
                !fields[0].trim().is_empty(),
                "Line {}: handle should not be empty",
                i
            );

            // Field 1: likes - should be a number
            assert!(
                fields[1].parse::<u64>().is_ok(),
                "Line {}: likes should be a number, got '{}'",
                i,
                fields[1]
            );

            // Field 2: downloads - should be a number
            assert!(
                fields[2].parse::<u64>().is_ok(),
                "Line {}: downloads should be a number, got '{}'",
                i,
                fields[2]
            );

            // Field 3: public - should be "true" or "false"
            assert!(
                fields[3] == "true" || fields[3] == "false",
                "Line {}: public should be 'true' or 'false', got '{}'",
                i,
                fields[3]
            );
        }
    }

    println!("✓ CLI produced valid field types in text output");
}
