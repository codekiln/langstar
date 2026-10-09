//! `langstar config` commands when the config file sets an `output_format` langstar doesn't support
//!
//! These tests run the langstar binary with `HOME` set to a temporary
//! directory, so they read only the config file each test writes. On Windows
//! the config path doesn't come from `HOME`, so they run only on Unix.
//!
//! Run with: `cargo test --test config_command_test`

#![cfg(unix)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_commands_run_despite_unsupported_output_format() {
    let home = TempDir::new().unwrap();
    let config_path = home.path().join(".config/langstar/config.toml");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, "output_format = \"yaml\"\n").unwrap();

    let langstar = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
        cmd.env("HOME", home.path())
            .env_remove("LANGSTAR_OUTPUT_FORMAT");
        cmd
    };

    // `config show` reports the value the user needs to fix
    let show = langstar().args(["config", "show"]).output().unwrap();
    assert!(show.status.success(), "{show:?}");
    assert!(String::from_utf8_lossy(&show.stdout).contains("output_format: yaml"));

    // `config output_format set` fixes it
    let set = langstar()
        .args(["config", "output_format", "set", "table"])
        .output()
        .unwrap();
    assert!(set.status.success(), "{set:?}");
    assert!(
        fs::read_to_string(&config_path)
            .unwrap()
            .contains("output_format = \"table\"")
    );

    // Other commands still refuse an unsupported format
    fs::write(&config_path, "output_format = \"yaml\"\n").unwrap();
    let dataset_list = langstar().args(["dataset", "list"]).output().unwrap();
    assert!(!dataset_list.status.success());
    assert!(
        String::from_utf8_lossy(&dataset_list.stderr).contains("Invalid output format: yaml"),
        "{dataset_list:?}"
    );
}
