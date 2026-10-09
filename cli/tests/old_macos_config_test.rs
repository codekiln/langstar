//! The langstar binary on macOS with a config file from before v2.2.0
//!
//! Before v2.2.0 langstar kept its config file at
//! `~/Library/Application Support/langstar/config.toml`. These tests run the
//! binary with `HOME` set to a temporary directory that holds that file, and
//! check what the user sees. The unit tests in `cli/src/config.rs` cover the
//! choice of file on every platform.
//!
//! Run with: `cargo test --test old_macos_config_test`

#![cfg(target_os = "macos")]

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const WARNING: &str = "Warning: Reading config from";

/// A temporary home directory for the langstar binary
struct Home {
    dir: TempDir,
}

impl Home {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn old_config(&self) -> PathBuf {
        self.dir
            .path()
            .join("Library/Application Support/langstar/config.toml")
    }

    fn config(&self) -> PathBuf {
        self.dir.path().join(".config/langstar/config.toml")
    }

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// langstar with this home and without the output format from the environment
    fn langstar(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
        cmd.env("HOME", self.dir.path())
            .env_remove("LANGSTAR_OUTPUT_FORMAT");
        cmd
    }
}

#[test]
fn test_config_show_reads_old_file_and_warns_once() {
    let home = Home::new();
    Home::write(&home.old_config(), "output_format = \"json\"\n");

    // `config show` loads the config twice in one run
    let output = home.langstar().args(["config", "show"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("output_format: json"), "{stdout}");
    assert!(stdout.contains("Reading instead:"), "{stdout}");
    assert_eq!(stderr.matches(WARNING).count(), 1, "{stderr}");
    assert!(stderr.contains("mv \""), "{stderr}");
}

#[test]
fn test_config_show_reads_new_file_without_warning_when_both_exist() {
    let home = Home::new();
    Home::write(&home.old_config(), "output_format = \"json\"\n");
    Home::write(&home.config(), "output_format = \"table\"\n");

    let output = home.langstar().args(["config", "show"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("output_format: table"), "{stdout}");
    assert!(!stdout.contains("Reading instead:"), "{stdout}");
    assert!(!stderr.contains(WARNING), "{stderr}");
}

#[test]
fn test_config_validate_fails_on_malformed_old_file() {
    let home = Home::new();
    Home::write(&home.old_config(), "output_format = \"unterminated\n");

    let output = home
        .langstar()
        .args(["config", "validate"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("validation FAILED"), "{stdout}");
}
