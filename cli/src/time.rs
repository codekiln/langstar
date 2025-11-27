//! Timezone utilities for displaying timestamps in CLI output.
//!
//! This module provides timezone parsing and formatting utilities that allow
//! users to configure how timestamps are displayed in CLI output.

use chrono::{DateTime, Local, Utc};
use chrono_tz::Tz;

/// Parsed timezone configuration for displaying timestamps.
///
/// Supports three modes:
/// - `Local`: Use the system's local timezone
/// - `Utc`: Use UTC (no conversion)
/// - `Named`: Use a specific IANA timezone (e.g., "America/New_York")
#[derive(Debug, Clone, Default)]
pub enum ConfiguredTimezone {
    /// Use system local timezone
    #[default]
    Local,
    /// Use UTC
    Utc,
    /// Use a specific IANA timezone
    Named(Tz),
}

impl ConfiguredTimezone {
    /// Parse a timezone string from configuration.
    ///
    /// Accepts:
    /// - "local" or "system" → uses system timezone
    /// - "utc" or "gmt" → uses UTC
    /// - IANA timezone names → e.g., "America/New_York", "Europe/London"
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use langstar_cli::time::ConfiguredTimezone;
    ///
    /// let tz = ConfiguredTimezone::parse("America/New_York").unwrap();
    /// let tz = ConfiguredTimezone::parse("local").unwrap();
    /// let tz = ConfiguredTimezone::parse("UTC").unwrap();
    /// ```
    pub fn parse(tz_str: &str) -> Result<Self, String> {
        match tz_str.to_lowercase().as_str() {
            "local" | "system" => Ok(Self::Local),
            "utc" | "gmt" => Ok(Self::Utc),
            _ => tz_str
                .parse::<Tz>()
                .map(Self::Named)
                .map_err(|_| {
                    format!(
                        "Invalid timezone: '{}'. Use IANA names like 'America/New_York' or 'Europe/London', \
                        or special values 'local' or 'UTC'.",
                        tz_str
                    )
                })
        }
    }

    /// Format a UTC datetime in this timezone.
    ///
    /// # Arguments
    ///
    /// * `dt` - A UTC datetime to format
    /// * `fmt` - A strftime format string (e.g., "%Y-%m-%d %H:%M:%S")
    ///
    /// # Returns
    ///
    /// The formatted datetime string in the configured timezone.
    pub fn format_datetime(&self, dt: DateTime<Utc>, fmt: &str) -> String {
        match self {
            Self::Local => dt.with_timezone(&Local).format(fmt).to_string(),
            Self::Utc => dt.format(fmt).to_string(),
            Self::Named(tz) => dt.with_timezone(tz).format(fmt).to_string(),
        }
    }

    /// Get a human-readable description of this timezone.
    ///
    /// Useful for displaying in `langstar config` output.
    pub fn description(&self) -> String {
        match self {
            Self::Local => "local (system timezone)".to_string(),
            Self::Utc => "UTC".to_string(),
            Self::Named(tz) => tz.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local() {
        assert!(matches!(
            ConfiguredTimezone::parse("local").unwrap(),
            ConfiguredTimezone::Local
        ));
        assert!(matches!(
            ConfiguredTimezone::parse("LOCAL").unwrap(),
            ConfiguredTimezone::Local
        ));
        assert!(matches!(
            ConfiguredTimezone::parse("system").unwrap(),
            ConfiguredTimezone::Local
        ));
    }

    #[test]
    fn test_parse_utc() {
        assert!(matches!(
            ConfiguredTimezone::parse("utc").unwrap(),
            ConfiguredTimezone::Utc
        ));
        assert!(matches!(
            ConfiguredTimezone::parse("UTC").unwrap(),
            ConfiguredTimezone::Utc
        ));
        assert!(matches!(
            ConfiguredTimezone::parse("gmt").unwrap(),
            ConfiguredTimezone::Utc
        ));
        assert!(matches!(
            ConfiguredTimezone::parse("GMT").unwrap(),
            ConfiguredTimezone::Utc
        ));
    }

    #[test]
    fn test_parse_iana_timezone() {
        let tz = ConfiguredTimezone::parse("America/New_York").unwrap();
        assert!(matches!(tz, ConfiguredTimezone::Named(_)));

        let tz = ConfiguredTimezone::parse("Europe/London").unwrap();
        assert!(matches!(tz, ConfiguredTimezone::Named(_)));

        let tz = ConfiguredTimezone::parse("Asia/Tokyo").unwrap();
        assert!(matches!(tz, ConfiguredTimezone::Named(_)));
    }

    #[test]
    fn test_parse_invalid_timezone() {
        let result = ConfiguredTimezone::parse("Invalid/Timezone");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid timezone"));

        let result = ConfiguredTimezone::parse("not-a-timezone");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_datetime_utc() {
        let tz = ConfiguredTimezone::Utc;
        let dt = DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = tz.format_datetime(dt, "%Y-%m-%d %H:%M:%S");
        assert_eq!(formatted, "2024-06-15 14:30:00");
    }

    #[test]
    fn test_format_datetime_named_timezone() {
        let tz = ConfiguredTimezone::parse("America/New_York").unwrap();
        let dt = DateTime::parse_from_rfc3339("2024-06-15T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let formatted = tz.format_datetime(dt, "%Y-%m-%d %H:%M:%S");
        // 14:30 UTC = 10:30 EDT (America/New_York in June)
        assert_eq!(formatted, "2024-06-15 10:30:00");
    }

    #[test]
    fn test_description() {
        assert_eq!(
            ConfiguredTimezone::Local.description(),
            "local (system timezone)"
        );
        assert_eq!(ConfiguredTimezone::Utc.description(), "UTC");
        assert_eq!(
            ConfiguredTimezone::parse("America/New_York")
                .unwrap()
                .description(),
            "America/New_York"
        );
    }

    #[test]
    fn test_default() {
        let tz = ConfiguredTimezone::default();
        assert!(matches!(tz, ConfiguredTimezone::Local));
    }
}
