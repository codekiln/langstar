//! CLI tests for delete confirmation on `dataset delete` (#702) and `queue delete` (#703),
//! plus `project delete` and `assistant delete`, which share the same prompt.
//!
//! assert_cmd pipes stdin, so these exercise the non-TTY path: without the skip flag the
//! command must fail with a clear message, before any API request is made. The interactive
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
    // Dummy credentials: the non-TTY check must fire before any request is sent.
    cmd.env("LANGSMITH_API_KEY", "dummy-key-for-tests")
        .env_remove("LANGSMITH_ORGANIZATION_ID")
        .env_remove("LANGSMITH_WORKSPACE_ID");
    cmd
}

fn assert_refuses_without_tty(args: &[&str], flag: &str) {
    langstar_cmd()
        .args(args)
        .write_stdin("y\n")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a terminal"))
        .stderr(predicate::str::contains(flag))
        .stdout(predicate::str::contains("Deleted").not());
}

#[test]
fn dataset_delete_without_yes_on_non_tty_errors() {
    assert_refuses_without_tty(&["dataset", "delete", ID], "--yes");
}

#[test]
fn queue_delete_without_force_on_non_tty_errors() {
    assert_refuses_without_tty(&["queue", "delete", ID], "--force");
}

#[test]
fn project_delete_by_name_without_force_on_non_tty_errors_before_lookup() {
    assert_refuses_without_tty(&["project", "delete", "some-project-name"], "--force");
}

#[test]
fn assistant_delete_without_force_on_non_tty_errors_before_lookup() {
    assert_refuses_without_tty(
        &["assistant", "delete", ID, "--deployment", "some-deployment"],
        "--force",
    );
}
