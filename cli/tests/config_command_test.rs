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
    let path = dir.path().join("config.toml");
    fs::write(&path, "output_format = \n").unwrap();

    let out = langstar_in(&dir).args(["config", "show"]).output().unwrap();

    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("output_format: table"), "{stdout}");
    assert!(stdout.contains(&path.display().to_string()), "{stdout}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    assert!(stderr.contains("Failed to parse config file"), "{stderr}");
}

#[test]
fn test_config_load_warns_about_an_unreadable_config_file() {
    let dir = TempDir::new().unwrap();
    // A directory where the file should be cannot be read as a file
    let path = dir.path().join("config.toml");
    fs::create_dir(&path).unwrap();

    let out = langstar_in(&dir).arg("version").output().unwrap();

    assert!(out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Failed to read config file"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    #[cfg(unix)]
    assert!(stderr.contains("Is a directory"), "{stderr}");
}

#[test]
fn test_config_load_warning_leaves_out_the_line_of_the_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, "langsmith_api_key = lsv2_pt_SECRET123\n").unwrap();

    let out = langstar_in(&dir).arg("version").output().unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    assert!(stderr.contains("line 1, column"), "{stderr}");
    assert!(stderr.contains("langstar config validate"), "{stderr}");
    assert!(!stderr.contains("SECRET123"), "{stderr}");
}

#[test]
fn test_config_load_warning_leaves_out_a_value_of_the_wrong_type() {
    for (line, value) in [
        ("langsmith_api_key = 12345678", "12345678"),
        ("timezone = 5", "`5`"),
        ("lsv2_pt_SECRET123", "SECRET123"),
        (
            "hide_workspace_and_org_id_message = \"lsv2_pt_SECRET123\"",
            "SECRET123",
        ),
    ] {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("config.toml"), format!("{line}\n")).unwrap();

        let out = langstar_in(&dir).arg("version").output().unwrap();

        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("Failed to parse config file"), "{stderr}");
        assert!(stderr.contains("line 1, column"), "{stderr}");
        assert!(!stderr.contains(value), "{stderr}");
        assert!(!stderr.contains("\nexpected"), "{stderr}");
        let first_line = stderr.lines().next().unwrap();
        assert!(!first_line.ends_with("expected"), "{stderr}");
    }
}

#[test]
fn test_config_validate_prints_the_line_the_parse_error_points_at() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("config.toml"), "output_format = \n").unwrap();

    let out = langstar_in(&dir)
        .args(["config", "validate"])
        .output()
        .unwrap();

    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success(), "{out:?}");
    assert!(all.contains("1 | output_format ="), "{all}");
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

#[test]
fn test_config_set_replaces_a_value_of_the_wrong_type() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, "output_format = 5\n").unwrap();

    let out = langstar_in(&dir)
        .args(["config", "output_format", "set", "json"])
        .output()
        .unwrap();

    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        fs::read_to_string(&path).unwrap().trim(),
        "output_format = \"json\""
    );
}

#[test]
fn test_config_set_fails_while_another_setting_holds_a_value_of_the_wrong_type() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, "timezone = 5\n").unwrap();

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
    assert_eq!(fs::read_to_string(&path).unwrap(), "timezone = 5\n");
}

#[test]
fn test_config_set_names_an_unreadable_config_file() {
    let dir = TempDir::new().unwrap();
    // A directory where the file should be cannot be read as a file
    let path = dir.path().join("config.toml");
    fs::create_dir(&path).unwrap();

    let out = langstar_in(&dir)
        .args(["config", "timezone", "set", "UTC"])
        .output()
        .unwrap();

    assert!(!out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Failed to read config file"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[cfg(unix)]
fn dir_with_config_symlink_loop() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    // A symlink to itself makes the metadata lookup fail with a "too many
    // levels of symbolic links" error, where `Path::exists` says false
    let path = dir.path().join("config.toml");
    std::os::unix::fs::symlink(&path, &path).unwrap();
    (dir, path)
}

#[cfg(unix)]
#[test]
fn test_config_load_warns_about_a_config_file_it_cannot_look_up() {
    let (dir, path) = dir_with_config_symlink_loop();

    let out = langstar_in(&dir).arg("version").output().unwrap();

    assert!(out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Failed to look up config file"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[cfg(unix)]
#[test]
fn test_config_validate_names_a_config_file_it_cannot_look_up() {
    let (dir, path) = dir_with_config_symlink_loop();

    let out = langstar_in(&dir)
        .args(["config", "validate"])
        .output()
        .unwrap();

    assert!(!out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Failed to look up config file"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[cfg(unix)]
#[test]
fn test_config_set_names_a_config_file_it_cannot_look_up() {
    let (dir, path) = dir_with_config_symlink_loop();

    let out = langstar_in(&dir)
        .args(["config", "timezone", "set", "UTC"])
        .output()
        .unwrap();

    assert!(!out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Failed to look up config file"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[cfg(unix)]
#[test]
fn test_config_show_does_not_say_a_config_file_it_cannot_look_up_is_missing() {
    let (dir, _path) = dir_with_config_symlink_loop();

    let out = langstar_in(&dir).args(["config", "show"]).output().unwrap();

    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("File exists: unknown"), "{stdout}");
    assert!(!stdout.contains("File exists: false"), "{stdout}");
}
