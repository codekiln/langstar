//! `langstar config` commands when the config file sets an `output_format` langstar doesn't support
//!
//! These tests run the langstar binary with `LANGSTAR_CONFIG_DIR` set to a
//! temporary directory, so they read only the config file each test writes.
//!
//! Run with: `cargo test --test config_command_test`

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_commands_run_despite_unsupported_output_format() {
    let config_dir = TempDir::new().unwrap();
    let config_path = config_dir.path().join("config.toml");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, "output_format = \"yaml\"\n").unwrap();

    let langstar = || {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
        cmd.env("LANGSTAR_CONFIG_DIR", config_dir.path())
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

#[test]
fn test_config_dir_env_chooses_the_config_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), "output_format = \"json\"\n").unwrap();

    let show = Command::new(env!("CARGO_BIN_EXE_langstar"))
        .env("LANGSTAR_CONFIG_DIR", dir.path())
        .env_remove("LANGSTAR_OUTPUT_FORMAT")
        .args(["config", "show"])
        .output()
        .unwrap();

    assert!(show.status.success(), "{show:?}");
    let stdout = String::from_utf8_lossy(&show.stdout);
    assert!(stdout.contains("output_format: json"), "{stdout}");
    assert!(
        stdout.contains(&dir.path().join("config.toml").display().to_string()),
        "{stdout}"
    );
}

fn langstar_in(dir: &TempDir) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
    cmd.env("LANGSTAR_CONFIG_DIR", dir.path())
        .env_remove("LANGSTAR_OUTPUT_FORMAT")
        .env_remove("LANGSTAR_TIMEZONE");
    cmd
}

#[test]
fn test_config_validate_accepts_a_valid_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), "output_format = \"json\"\n").unwrap();

    let out = langstar_in(&dir)
        .args(["config", "validate"])
        .output()
        .unwrap();

    assert!(out.status.success(), "{out:?}");
}

#[test]
fn test_config_validate_accepts_a_missing_file() {
    let dir = TempDir::new().unwrap();

    let out = langstar_in(&dir)
        .args(["config", "validate"])
        .output()
        .unwrap();

    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("Config file does not exist"));
}

#[test]
fn test_config_validate_fails_on_an_unparsable_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, "output_format = \n").unwrap();

    let out = langstar_in(&dir)
        .args(["config", "validate"])
        .output()
        .unwrap();

    assert!(!out.status.success(), "{out:?}");
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(all.contains(&path.display().to_string()), "{all}");
    assert!(all.contains("Failed to parse config file"), "{all}");
}

#[test]
fn test_config_show_warns_about_an_unparsable_file() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), "output_format = \n").unwrap();

    let out = langstar_in(&dir).args(["config", "show"]).output().unwrap();

    assert!(out.status.success(), "{out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("Failed to parse config file"),
        "{out:?}"
    );
}

#[test]
fn test_config_set_fails_on_an_unparsable_file_and_leaves_it_unchanged() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, "output_format = \n").unwrap();

    let out = langstar_in(&dir)
        .args(["config", "output_format", "set", "json"])
        .output()
        .unwrap();

    assert!(!out.status.success(), "{out:?}");
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(all.contains("Failed to parse config file"), "{all}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "output_format = \n");
}
