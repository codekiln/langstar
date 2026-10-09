//! An empty home directory for the langstar binary under test

use std::path::PathBuf;

/// An empty directory for the tests to set as `HOME`, so the langstar binary
/// they run reads no config file from the developer's own home directory. On
/// macOS that includes the pre-v2.2.0 `~/Library/Application Support/langstar/config.toml`,
/// which langstar still reads when `~/.config/langstar/config.toml` is missing.
///
/// Tests take their credentials from environment variables, which still reach
/// the binary.
///
/// On Windows this does not isolate the tests: langstar takes the config path
/// from `dirs::config_dir()`, which calls `SHGetKnownFolderPath` (dirs-sys) and
/// ignores both `HOME` and `APPDATA`. CI doesn't build Windows. Issue #792
/// (<https://github.com/codekiln/langstar/issues/792>) proposes
/// `LANGSTAR_CONFIG_DIR` as the fix.
pub fn empty_home() -> PathBuf {
    let home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("empty-home");
    std::fs::create_dir_all(&home).expect("Failed to create the empty HOME for tests");
    home
}
