//! An empty config directory for the langstar binary under test

use std::path::PathBuf;

/// An empty directory for the tests to set as `LANGSTAR_CONFIG_DIR`, so the
/// langstar binary they run reads no config file from the developer's own
/// machine, on any platform. The variable also turns off langstar's fallback to
/// the pre-v2.2.0 macOS config file.
///
/// Tests take their credentials from environment variables, which still reach
/// the binary.
pub fn empty_config_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("empty-config-dir");
    std::fs::create_dir_all(&dir).expect("Failed to create the empty config dir for tests");
    dir
}
