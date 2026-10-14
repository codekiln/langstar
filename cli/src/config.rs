use crate::error::{CliError, Result};
use langstar_sdk::AuthConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Once;

/// Configuration for the Langstar CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LangSmith API key (used for both LangSmith and LangGraph APIs)
    pub langsmith_api_key: Option<String>,
    /// Optional organization ID for scoping LangSmith operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Optional workspace ID for narrower scoping of LangSmith operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Default output format (json or table)
    #[serde(default = "default_output_format")]
    pub output_format: String,
    /// Timezone for displaying timestamps in CLI output
    ///
    /// Accepts IANA timezone names (e.g., "America/New_York", "Europe/London")
    /// or special values: "local" (system timezone), "UTC"
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Suppress warning when both organization_id and workspace_id are set
    #[serde(default)]
    pub hide_workspace_and_org_id_message: bool,
}

fn default_output_format() -> String {
    "table".to_string()
}

fn default_timezone() -> String {
    "local".to_string()
}

/// The config file langstar reads
#[derive(Debug, PartialEq)]
enum ConfigFile<'a> {
    /// The file at the current path
    Current(&'a Path),
    /// The file at the pre-v2.2.0 macOS path, used only when the current one is missing
    Old(&'a Path),
    /// Neither file exists
    Missing,
}

/// The warning langstar prints when it reads the config file at `old_path`,
/// telling the user how to move it to `config_path`
fn old_config_warning(old_path: &Path, config_path: &Path) -> String {
    format!(
        "Warning: Reading config from {}, where langstar kept it before v2.2.0. \
         Move it to {} with:\n  mkdir -p \"{}\" && mv \"{}\" \"{}\"",
        old_path.display(),
        config_path.display(),
        config_path.parent().unwrap_or(config_path).display(),
        old_path.display(),
        config_path.display()
    )
}

/// Returns whether a file exists at `path`, or an error that names the path when
/// langstar cannot check, as with a symlink that points at itself or a parent
/// directory langstar cannot search. A config file langstar cannot check is an
/// error here, where `Path::exists` would return `false` and langstar would run
/// as if the file were absent.
pub(crate) fn config_file_exists(path: &Path) -> Result<bool> {
    path.try_exists().map_err(|e| {
        CliError::Config(format!(
            "Failed to look up config file {}: {}",
            path.display(),
            e
        ))
    })
}

/// Pick the current config file when it exists, otherwise the old one when it
/// exists. Returns an error when looking up either file fails.
fn choose_config_file<'a>(
    config_path: &'a Path,
    old_path: Option<&'a Path>,
) -> Result<ConfigFile<'a>> {
    if config_file_exists(config_path)? {
        return Ok(ConfigFile::Current(config_path));
    }
    match old_path {
        Some(old_path) if config_file_exists(old_path)? => Ok(ConfigFile::Old(old_path)),
        _ => Ok(ConfigFile::Missing),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            langsmith_api_key: None,
            organization_id: None,
            workspace_id: None,
            output_format: default_output_format(),
            timezone: default_timezone(),
            hide_workspace_and_org_id_message: false,
        }
    }
}

impl Config {
    /// Load configuration from file and environment variables
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables
    /// 2. Config file (`config.toml` in the config directory, see `config_file_path`)
    /// 3. Default values
    ///
    /// A config file that langstar cannot look up, read or parse is ignored with a
    /// warning, so `config show` still runs and prints the file's path;
    /// `config validate` and `config <setting> set` fail on it instead.
    pub fn load() -> Result<Self> {
        // `load` returns an error when langstar cannot find the directory that holds
        // the config file, even if environment variables set every setting, so that
        // langstar tells the user instead of silently skipping a config file they
        // wrote. This happens when `LANGSTAR_CONFIG_DIR` is unset and the operating
        // system reports no home directory (macOS, Linux) or no config directory
        // (Windows).
        let config_path = Self::config_file_path()?;
        let old_path = Self::old_config_file_path();
        let mut config = Self::load_from_paths(&config_path, old_path.as_deref()).unwrap_or_else(|e| {
            // Warn on the first failed load only, so a process that loads the
            // config again does not repeat the warning
            static WARN_ONCE: Once = Once::new();
            WARN_ONCE.call_once(|| {
                eprintln!(
                    "Warning: {e}\n  langstar is using the default settings and any environment variables instead. Run 'langstar config validate' to see the error."
                )
            });
            Self::default()
        });

        // Override with environment variables
        if let Ok(key) = std::env::var("LANGSMITH_API_KEY") {
            config.langsmith_api_key = Some(key);
        }
        if let Ok(org_id) = std::env::var("LANGSMITH_ORGANIZATION_ID") {
            config.organization_id = Some(org_id);
        }
        if let Ok(workspace_id) = std::env::var("LANGSMITH_WORKSPACE_ID") {
            config.workspace_id = Some(workspace_id);
        }
        if let Ok(format) = std::env::var("LANGSTAR_OUTPUT_FORMAT") {
            config.output_format = format;
        }
        if let Ok(tz) = std::env::var("LANGSTAR_TIMEZONE") {
            config.timezone = tz;
        }
        if let Ok(hide) = std::env::var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE") {
            config.hide_workspace_and_org_id_message =
                hide == "1" || hide.eq_ignore_ascii_case("true");
        }

        // Log warning if both organization and workspace IDs are set
        if !config.hide_workspace_and_org_id_message
            && config.organization_id.is_some()
            && config.workspace_id.is_some()
        {
            use crate::commands::config::messages;
            eprintln!(
                "Warning: Both organization_id and workspace_id are set. Workspace ID takes precedence for narrower scoping."
            );
            eprintln!("  {}", messages::SUPPRESS_WORKSPACE_ORG_WARNING);
        }

        Ok(config)
    }

    /// Load the config file at `config_path`, or the file at `old_path` when
    /// only the old one exists, warning that it should be moved.
    ///
    /// Takes both paths as arguments so tests on any platform can point it at
    /// temporary directories.
    fn load_from_paths(config_path: &Path, old_path: Option<&Path>) -> Result<Self> {
        let path = match choose_config_file(config_path, old_path)? {
            ConfigFile::Current(path) => path,
            ConfigFile::Old(path) => {
                // Warn on the first load only, so a process that loads the
                // config again does not repeat the warning
                static WARN_ONCE: Once = Once::new();
                WARN_ONCE.call_once(|| eprintln!("{}", old_config_warning(path, config_path)));
                path
            }
            ConfigFile::Missing => return Ok(Self::default()),
        };

        // The warning leaves out the line of the file that failed to parse and the
        // value the message quotes, in backticks or double quotes (for example
        // "invalid type: integer `5`" or `invalid type: string "abc"`), because
        // either can hold the API key and every command that loads the settings
        // prints this warning. `config validate` and `config <setting> set` print
        // the line the parse error points at.
        Self::parse_config_file(path, false)
    }

    /// Read and parse the config file at `path`, returning any read or parse error.
    ///
    /// `config validate` calls this function to show the user the full parse
    /// error, including the line of the file that the parse error points at.
    /// `Config::load` reaches `parse_config_file` through `load_from_paths`, which
    /// passes `quote_source` as false so that the warning leaves that line out;
    /// `load` then runs on the default config.
    pub fn read_file(path: &Path) -> Result<Self> {
        Self::parse_config_file(path, true)
    }

    /// Reads and parses the config file at `path`. When `quote_source` is true, a
    /// parse error carries toml's full message, including the quoted line of the
    /// file. When it is false, the error gives the line and column and the part of
    /// toml's message before the backtick or double quote that opens the value, so
    /// the error leaves out the file's text and the value.
    fn parse_config_file(path: &Path, quote_source: bool) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            CliError::Config(format!(
                "Failed to read config file {}: {}",
                path.display(),
                e
            ))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            if quote_source {
                return CliError::Config(format!(
                    "Failed to parse config file {}: {}",
                    path.display(),
                    e
                ));
            }
            let position = e
                .span()
                .map(|span| {
                    let before = &content[..span.start.min(content.len())];
                    let line = before.matches('\n').count() + 1;
                    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
                    format!(" at line {line}, column {column}")
                })
                .unwrap_or_default();
            CliError::Config(format!(
                "Failed to parse config file {}{}: {}",
                path.display(),
                position,
                message_without_value(e.message())
            ))
        })?;

        Ok(config)
    }

    /// The config file `load` reads: the current one, or on macOS the one from
    /// before v2.2.0 when only that one exists. `None` when neither exists, and
    /// an error that names the file when looking one up fails.
    pub fn file_to_read() -> Result<Option<PathBuf>> {
        let config_path = Self::config_file_path()?;
        let old_path = Self::old_config_file_path();
        Ok(
            match choose_config_file(&config_path, old_path.as_deref())? {
                ConfigFile::Current(path) | ConfigFile::Old(path) => Some(path.to_path_buf()),
                ConfigFile::Missing => None,
            },
        )
    }

    /// Where langstar kept the config file on macOS before v2.2.0:
    /// `~/Library/Application Support/langstar/config.toml`. `None` when
    /// `LANGSTAR_CONFIG_DIR` is set, because that variable names the only place
    /// langstar looks.
    #[cfg(target_os = "macos")]
    pub fn old_config_file_path() -> Option<PathBuf> {
        if config_dir_from_env().is_some() {
            return None;
        }
        dirs::config_dir().map(|dir| dir.join("langstar").join("config.toml"))
    }

    /// langstar kept its config file in the same place on Linux and Windows,
    /// so they have no old path.
    #[cfg(not(target_os = "macos"))]
    pub fn old_config_file_path() -> Option<PathBuf> {
        None
    }

    /// Get the path to the config file
    ///
    /// Uses `$LANGSTAR_CONFIG_DIR/config.toml` when that variable is set and not
    /// empty. Otherwise uses ~/.config/langstar/config.toml on macOS and Linux
    /// for consistency, and the platform-specific AppData location on Windows.
    pub fn config_file_path() -> Result<PathBuf> {
        if let Some(dir) = config_dir_from_env() {
            return Ok(config_file_in(&dir));
        }
        Self::config_file_under(Self::default_base_dir())
    }

    /// The directory langstar's default config location sits under: the config
    /// directory on Windows, the home directory on macOS and Linux. `None` when the
    /// operating system reports none.
    #[cfg(target_os = "windows")]
    fn default_base_dir() -> Option<PathBuf> {
        dirs::config_dir()
    }

    #[cfg(not(target_os = "windows"))]
    fn default_base_dir() -> Option<PathBuf> {
        dirs::home_dir()
    }

    /// Returns the config file's path under `base_dir`:
    /// `<config dir>\langstar\config.toml` on Windows and
    /// `~/.config/langstar/config.toml` on macOS and Linux. `default_base_dir`
    /// supplies `base_dir`.
    ///
    /// Takes `base_dir` as an argument so a unit test can pass `None`, which
    /// `dirs` cannot be made to report: `dirs::home_dir` falls back to the passwd
    /// entry when `HOME` is unset.
    #[cfg(target_os = "windows")]
    fn config_file_under(base_dir: Option<PathBuf>) -> Result<PathBuf> {
        let config_dir = base_dir
            .ok_or_else(|| CliError::Config("langstar cannot find your config directory, so it cannot tell where the config file belongs. Set LANGSTAR_CONFIG_DIR to the directory that holds config.toml.".to_string()))?;
        Ok(config_dir.join("langstar").join("config.toml"))
    }

    #[cfg(not(target_os = "windows"))]
    fn config_file_under(base_dir: Option<PathBuf>) -> Result<PathBuf> {
        let home_dir = base_dir
            .ok_or_else(|| CliError::Config("langstar cannot find your home directory, so it cannot tell where the config file belongs. Set HOME, or set LANGSTAR_CONFIG_DIR to the directory that holds config.toml.".to_string()))?;
        Ok(home_dir
            .join(".config")
            .join("langstar")
            .join("config.toml"))
    }

    /// Save the current configuration to file
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Convert to AuthConfig for the SDK
    pub fn to_auth_config(&self) -> AuthConfig {
        AuthConfig::new(
            self.langsmith_api_key.clone(),
            self.organization_id.clone(),
            self.workspace_id.clone(),
        )
    }
}

/// The environment variable that names the directory holding `config.toml`
pub const CONFIG_DIR_ENV: &str = "LANGSTAR_CONFIG_DIR";

/// The directory `LANGSTAR_CONFIG_DIR` names, or `None` when it is unset or empty
fn config_dir_from_env() -> Option<PathBuf> {
    config_dir_from_value(std::env::var_os(CONFIG_DIR_ENV))
}

fn config_dir_from_value(value: Option<std::ffi::OsString>) -> Option<PathBuf> {
    value.filter(|v| !v.is_empty()).map(PathBuf::from)
}

fn config_file_in(dir: &Path) -> PathBuf {
    dir.join("config.toml")
}

/// Returns toml's error message without the value toml rejected, which can be the
/// API key. toml writes that value between backticks, or between double quotes for
/// a string, so the function stops at the first of either on the first line and
/// drops the later lines, which quote tokens too. When the cut leaves a trailing
/// `expected`, which names nothing without the token, the function drops that as
/// well, and returns `invalid TOML` when nothing is left.
fn message_without_value(message: &str) -> &str {
    let first_line = message.lines().next().unwrap_or("");
    let end = first_line.find(['`', '"']).unwrap_or(first_line.len());
    let cut = first_line[..end].trim_end_matches([' ', ':', ',']);
    let cut = cut
        .strip_suffix("expected")
        .unwrap_or(cut)
        .trim_end_matches([' ', ':', ',']);
    if cut.is_empty() { "invalid TOML" } else { cut }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_env_chooses_directory() {
        let dir = config_dir_from_value(Some("/tmp/ls-config".into())).unwrap();
        assert_eq!(
            config_file_in(&dir),
            Path::new("/tmp/ls-config").join("config.toml")
        );
    }

    #[test]
    fn test_config_dir_env_unset_or_empty_is_ignored() {
        assert_eq!(config_dir_from_value(None), None);
        assert_eq!(config_dir_from_value(Some("".into())), None);
    }

    #[test]
    fn test_config_file_path_fails_without_a_home_or_config_directory() {
        let err = Config::config_file_under(None).unwrap_err().to_string();
        let expected = if cfg!(target_os = "windows") {
            "langstar cannot find your config directory"
        } else {
            "langstar cannot find your home directory"
        };
        assert!(err.contains(expected), "{err}");
    }

    #[test]
    fn test_config_file_path_sits_under_the_home_or_config_directory() {
        let path = Config::config_file_under(Some("/base".into())).unwrap();
        assert!(path.starts_with("/base"), "{path:?}");
        assert!(
            path.ends_with(Path::new("langstar").join("config.toml")),
            "{path:?}"
        );
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.output_format, "table");
        assert_eq!(config.timezone, "local");
        assert!(config.langsmith_api_key.is_none());
    }

    /// A temporary home with the current and the pre-v2.2.0 macOS config paths
    /// inside it; each test writes the files it needs.
    struct ConfigPaths {
        _dir: tempfile::TempDir,
        current: PathBuf,
        old: PathBuf,
    }

    impl ConfigPaths {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let current = dir.path().join(".config/langstar/config.toml");
            let old = dir
                .path()
                .join("Library/Application Support/langstar/config.toml");
            Self {
                _dir: dir,
                current,
                old,
            }
        }

        fn write(path: &Path, api_key: &str) {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, format!("langsmith_api_key = \"{api_key}\"\n")).unwrap();
        }

        fn load(&self) -> Config {
            Config::load_from_paths(&self.current, Some(&self.old)).unwrap()
        }
    }

    #[test]
    fn test_load_reads_current_file_when_only_it_exists() {
        let paths = ConfigPaths::new();
        ConfigPaths::write(&paths.current, "current-key");

        assert_eq!(
            choose_config_file(&paths.current, Some(&paths.old)).unwrap(),
            ConfigFile::Current(&paths.current)
        );
        assert_eq!(
            paths.load().langsmith_api_key.as_deref(),
            Some("current-key")
        );
    }

    #[test]
    fn test_load_reads_old_file_when_only_it_exists() {
        let paths = ConfigPaths::new();
        ConfigPaths::write(&paths.old, "old-key");

        assert_eq!(
            choose_config_file(&paths.current, Some(&paths.old)).unwrap(),
            ConfigFile::Old(&paths.old)
        );
        assert_eq!(paths.load().langsmith_api_key.as_deref(), Some("old-key"));
    }

    #[test]
    fn test_load_prefers_current_file_when_both_exist() {
        let paths = ConfigPaths::new();
        ConfigPaths::write(&paths.current, "current-key");
        ConfigPaths::write(&paths.old, "old-key");

        assert_eq!(
            choose_config_file(&paths.current, Some(&paths.old)).unwrap(),
            ConfigFile::Current(&paths.current)
        );
        assert_eq!(
            paths.load().langsmith_api_key.as_deref(),
            Some("current-key")
        );
    }

    #[test]
    fn test_load_uses_defaults_when_neither_file_exists() {
        let paths = ConfigPaths::new();

        assert_eq!(
            choose_config_file(&paths.current, Some(&paths.old)).unwrap(),
            ConfigFile::Missing
        );
        let config = paths.load();
        assert!(config.langsmith_api_key.is_none());
        assert_eq!(config.output_format, "table");
    }

    #[test]
    fn test_load_ignores_old_file_without_an_old_path() {
        // On Linux and Windows, `load` passes no old path, so
        // langstar never reads the old macOS file.
        let paths = ConfigPaths::new();
        ConfigPaths::write(&paths.old, "old-key");

        assert_eq!(
            choose_config_file(&paths.current, None).unwrap(),
            ConfigFile::Missing
        );
        let config = Config::load_from_paths(&paths.current, None).unwrap();
        assert!(config.langsmith_api_key.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_load_names_a_config_file_it_cannot_look_up() {
        // A symlink to itself makes the metadata lookup fail, where
        // `Path::exists` would say the file is missing
        let paths = ConfigPaths::new();
        std::fs::create_dir_all(paths.current.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&paths.current, &paths.current).unwrap();

        let message = choose_config_file(&paths.current, Some(&paths.old))
            .unwrap_err()
            .to_string();
        assert!(
            message.contains("Failed to look up config file"),
            "{message}"
        );
        assert!(
            message.contains(&paths.current.display().to_string()),
            "{message}"
        );
        assert!(Config::load_from_paths(&paths.current, Some(&paths.old)).is_err());
    }

    #[test]
    fn test_old_config_warning_names_both_paths_and_the_move_command() {
        let old = Path::new("/home/user/Library/Application Support/langstar/config.toml");
        let current = Path::new("/home/user/.config/langstar/config.toml");

        assert_eq!(
            old_config_warning(old, current),
            "Warning: Reading config from /home/user/Library/Application Support/langstar/config.toml, \
             where langstar kept it before v2.2.0. Move it to /home/user/.config/langstar/config.toml with:\n  \
             mkdir -p \"/home/user/.config/langstar\" && \
             mv \"/home/user/Library/Application Support/langstar/config.toml\" \"/home/user/.config/langstar/config.toml\""
        );
    }

    #[test]
    fn test_read_file_reports_a_parse_error() {
        let paths = ConfigPaths::new();
        std::fs::create_dir_all(paths.old.parent().unwrap()).unwrap();
        std::fs::write(&paths.old, "langsmith_api_key = \"unterminated\n").unwrap();

        let error = Config::read_file(&paths.old).unwrap_err().to_string();
        assert!(error.contains("Failed to parse config file"), "{error}");
    }

    #[test]
    fn test_config_ignores_removed_github_integration_id() {
        // Config files written before the key was removed still load; the
        // integration ID now comes from --integration-id or the Control Plane API.
        let config: Config =
            toml::from_str("langsmith_api_key = \"key\"\ngithub_integration_id = \"old-id\"\n")
                .unwrap();
        assert_eq!(config.langsmith_api_key.as_deref(), Some("key"));
        assert!(
            !toml::to_string(&config)
                .unwrap()
                .contains("github_integration_id")
        );
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            langsmith_api_key: Some("test_key".to_string()),
            organization_id: Some("test_org_id".to_string()),
            workspace_id: None,
            output_format: "json".to_string(),
            timezone: "America/New_York".to_string(),
            hide_workspace_and_org_id_message: false,
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("langsmith_api_key"));
        assert!(toml.contains("test_key"));
        assert!(toml.contains("organization_id"));
        assert!(toml.contains("test_org_id"));
        assert!(toml.contains("json"));
        assert!(toml.contains("America/New_York"));
    }

    #[test]
    fn test_config_with_workspace() {
        let config = Config {
            langsmith_api_key: Some("test_key".to_string()),
            organization_id: None,
            workspace_id: Some("test_workspace_id".to_string()),
            output_format: "table".to_string(),
            timezone: "local".to_string(),
            hide_workspace_and_org_id_message: false,
        };

        let auth = config.to_auth_config();
        assert!(auth.organization_id.is_none());
        assert_eq!(auth.workspace_id, Some("test_workspace_id".to_string()));
    }

    #[test]
    fn test_config_to_auth_config_with_both() {
        let config = Config {
            langsmith_api_key: Some("key".to_string()),
            organization_id: Some("org_123".to_string()),
            workspace_id: Some("workspace_456".to_string()),
            output_format: "table".to_string(),
            timezone: "UTC".to_string(),
            hide_workspace_and_org_id_message: false,
        };

        let auth = config.to_auth_config();
        assert_eq!(auth.organization_id, Some("org_123".to_string()));
        assert_eq!(auth.workspace_id, Some("workspace_456".to_string()));
    }

    #[test]
    fn test_hide_warning_flag() {
        let mut config = Config::default();
        assert!(!config.hide_workspace_and_org_id_message);

        config.hide_workspace_and_org_id_message = true;
        assert!(config.hide_workspace_and_org_id_message);
    }

    #[test]
    fn test_config_serde_with_hide_warning() {
        let toml = r#"
            output_format = "json"
            hide_workspace_and_org_id_message = true
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.hide_workspace_and_org_id_message);
    }

    #[test]
    fn test_env_var_hide_warning_parsing() {
        unsafe {
            // Test parsing "1"
            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "1");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            // Test parsing "true" (case-insensitive)
            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "true");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "TRUE");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "True");
            let config = Config::load().unwrap();
            assert!(config.hide_workspace_and_org_id_message);

            // Test that other values are treated as false
            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "0");
            let config = Config::load().unwrap();
            assert!(!config.hide_workspace_and_org_id_message);

            std::env::set_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE", "false");
            let config = Config::load().unwrap();
            assert!(!config.hide_workspace_and_org_id_message);

            // Cleanup
            std::env::remove_var("LANGSTAR_HIDE_WORKSPACE_AND_ORG_ID_MESSAGE");
        }
    }
}
