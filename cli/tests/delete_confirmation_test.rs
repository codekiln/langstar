//! Tests that `dataset delete`, `queue delete`, `project delete`, `assistant delete`,
//! `deployment delete`, `model-config delete` and `secrets delete` refuse to run when no terminal is attached and the skip flag is missing. codekiln filed
//! [#702 Fix dataset delete confirmation to actually wait for user input](https://github.com/codekiln/langstar/issues/702)
//! and [#703 Fix queue delete confirmation to actually wait for user input](https://github.com/codekiln/langstar/issues/703)
//! for the dataset and queue prompts.
//!
//! assert_cmd feeds the command's input through a pipe, so these tests run the command with no
//! terminal attached: without the skip flag the command must fail with a clear message before
//! it sends any API request. The interactive
//! answers (`n`, `yes`, and a bare `y` that is refused) are covered by unit tests in
//! `cli/src/confirm.rs`.

use assert_cmd::Command;
use escargot::CargoBuild;
use predicates::prelude::*;

const ID: &str = "00000000-0000-0000-0000-000000000001";

fn langstar_cmd() -> Command {
    let bin = CargoBuild::new()
        .bin("langstar")
        .run()
        .expect("Failed to build langstar binary")
        .path()
        .to_owned();
    let mut cmd = Command::new(bin);
    // Dummy credentials: the no-terminal check must fire before any request is sent.
    cmd.env("LANGSMITH_API_KEY", "dummy-key-for-tests")
        .env_remove("LANGSMITH_ORGANIZATION_ID")
        .env_remove("LANGSMITH_WORKSPACE_ID");
    cmd
}

fn assert_refuses_without_terminal(args: &[&str], flag: &str) {
    langstar_cmd()
        .args(args)
        .write_stdin("y\n")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a terminal"))
        .stderr(predicate::str::contains(flag))
        .stdout(
            predicate::str::contains("Deleted")
                .or(predicate::str::contains("deleted"))
                .not(),
        );
}

#[test]
fn dataset_delete_without_yes_on_no_terminal_errors() {
    assert_refuses_without_terminal(&["dataset", "delete", ID], "--yes");
}

#[test]
fn queue_delete_without_force_on_no_terminal_errors() {
    assert_refuses_without_terminal(&["queue", "delete", ID], "--force");
}

#[test]
fn project_delete_by_name_without_force_on_no_terminal_errors_before_lookup() {
    assert_refuses_without_terminal(&["project", "delete", "some-project-name"], "--force");
}

#[test]
fn assistant_delete_without_force_on_no_terminal_errors_before_lookup() {
    assert_refuses_without_terminal(
        &["assistant", "delete", ID, "--deployment", "some-deployment"],
        "--force",
    );
}

#[test]
fn deployment_delete_without_yes_with_no_terminal_errors() {
    assert_refuses_without_terminal(&["deployment", "delete", ID], "--yes");
}

#[test]
fn model_config_delete_without_yes_with_no_terminal_errors() {
    assert_refuses_without_terminal(&["model-config", "delete", ID], "--yes");
}

#[test]
fn secrets_delete_without_yes_with_no_terminal_errors() {
    assert_refuses_without_terminal(&["secrets", "delete", "SOME_KEY"], "--yes");
}
