//! Timezone and time utilities for CLI output and filtering.
//!
//! This module provides:
//! - Timezone parsing and formatting utilities for displaying timestamps
//! - Relative time duration parsing (e.g., "15m", "1h", "7d")
//! - Time presets for common filtering windows

use chrono::{DateTime, Duration, Local, Utc};
use chrono_tz::Tz;
use clap::ValueEnum;

// ═══════════════════════════════════════════════════════════════════════════
// Relative Time Parsing
// ═══════════════════════════════════════════════════════════════════════════

/// Parse a relative time string into a chrono Duration.
///
/// Supports formats like:
/// - `15m` - 15 minutes
/// - `1h` - 1 hour
/// - `7d` - 7 days
/// - `2w` - 2 weeks
///
/// # Arguments
///
/// * `s` - A string like "15m", "1h", "7d", "2w"
///
/// # Returns
///
/// The duration represented by the string, or an error message if invalid.
///
/// # Examples
///
/// ```rust,no_run
/// use langstar_cli::time::parse_relative_duration;
///
/// let duration = parse_relative_duration("15m").unwrap();
/// assert_eq!(duration.num_minutes(), 15);
///
/// let duration = parse_relative_duration("7d").unwrap();
/// assert_eq!(duration.num_days(), 7);
/// ```
pub fn parse_relative_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty duration string".to_string());
    }

    // Find the split point between number and unit
    let split_idx = s
        .find(|c: char| !c.is_ascii_digit())
        .ok_or_else(|| format!("Missing unit in duration '{}'. Use m, h, d, or w.", s))?;

    let (num_str, unit) = s.split_at(split_idx);

    let num: i64 = num_str
        .parse()
        .map_err(|_| format!("Invalid number in duration '{}'", s))?;

    if num <= 0 {
        return Err(format!("Duration must be positive, got '{}'", s));
    }

    match unit.to_lowercase().as_str() {
        "m" | "min" | "mins" | "minute" | "minutes" => Ok(Duration::minutes(num)),
        "h" | "hr" | "hrs" | "hour" | "hours" => Ok(Duration::hours(num)),
        "d" | "day" | "days" => Ok(Duration::days(num)),
        "w" | "wk" | "wks" | "week" | "weeks" => Ok(Duration::weeks(num)),
        _ => Err(format!(
            "Unknown unit '{}' in duration '{}'. Use m (minutes), h (hours), d (days), or w (weeks).",
            unit, s
        )),
    }
}

/// Valid unit strings for relative durations.
const VALID_DURATION_UNITS: &[&str] = &[
    "m", "min", "mins", "minute", "minutes", "h", "hr", "hrs", "hour", "hours", "d", "day", "days",
    "w", "wk", "wks", "week", "weeks",
];

/// Check if a string looks like a relative duration (e.g., "15m", "1h").
///
/// This is used to distinguish between ISO 8601 timestamps and relative durations.
/// A relative duration is a simple format like "15m", "1h", "7d", "2w" - it starts
/// with digits followed by exactly a valid unit string.
pub fn is_relative_duration(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // ISO 8601 timestamps contain '-', ':', or 'T' - relative durations don't
    if s.contains('-') || s.contains(':') || s.contains('T') {
        return false;
    }

    // Find the split point between number and unit
    let split_idx = match s.find(|c: char| !c.is_ascii_digit()) {
        Some(idx) => idx,
        None => return false, // No unit found (e.g., "123")
    };

    // Must start with at least one digit
    if split_idx == 0 {
        return false;
    }

    let (_num_str, unit) = s.split_at(split_idx);
    let unit_lower = unit.to_lowercase();

    // Check if unit exactly matches one of the valid units
    VALID_DURATION_UNITS.contains(&unit_lower.as_str())
}

// ═══════════════════════════════════════════════════════════════════════════
// Time Presets
// ═══════════════════════════════════════════════════════════════════════════

/// Predefined time presets for common filtering windows.
///
/// These match the LangSmith UI time filter options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TimePreset {
    /// Last 1 hour
    #[value(name = "1h")]
    OneHour,
    /// Last 3 hours
    #[value(name = "3h")]
    ThreeHours,
    /// Last 6 hours
    #[value(name = "6h")]
    SixHours,
    /// Last 12 hours
    #[value(name = "12h")]
    TwelveHours,
    /// Last 1 day
    #[value(name = "1d")]
    OneDay,
    /// Last 2 days
    #[value(name = "2d")]
    TwoDays,
    /// Last 7 days (default)
    #[value(name = "7d")]
    SevenDays,
    /// Last 14 days
    #[value(name = "14d")]
    FourteenDays,
}

impl TimePreset {
    /// Convert the preset to a chrono Duration.
    pub fn to_duration(self) -> Duration {
        match self {
            TimePreset::OneHour => Duration::hours(1),
            TimePreset::ThreeHours => Duration::hours(3),
            TimePreset::SixHours => Duration::hours(6),
            TimePreset::TwelveHours => Duration::hours(12),
            TimePreset::OneDay => Duration::days(1),
            TimePreset::TwoDays => Duration::days(2),
            TimePreset::SevenDays => Duration::days(7),
            TimePreset::FourteenDays => Duration::days(14),
        }
    }

    /// Get the default preset (7 days).
    pub fn default_preset() -> Self {
        TimePreset::SevenDays
    }

    /// Get a human-readable description of the preset.
    pub fn description(self) -> &'static str {
        match self {
            TimePreset::OneHour => "Last 1 hour",
            TimePreset::ThreeHours => "Last 3 hours",
            TimePreset::SixHours => "Last 6 hours",
            TimePreset::TwelveHours => "Last 12 hours",
            TimePreset::OneDay => "Last 1 day",
            TimePreset::TwoDays => "Last 2 days",
            TimePreset::SevenDays => "Last 7 days",
            TimePreset::FourteenDays => "Last 14 days",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Timezone Configuration
// ═══════════════════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════════════
    // Relative Duration Parsing Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_parse_relative_duration_minutes() {
        assert_eq!(parse_relative_duration("15m").unwrap().num_minutes(), 15);
        assert_eq!(parse_relative_duration("1m").unwrap().num_minutes(), 1);
        assert_eq!(parse_relative_duration("60m").unwrap().num_minutes(), 60);
        assert_eq!(parse_relative_duration("5min").unwrap().num_minutes(), 5);
        assert_eq!(parse_relative_duration("5mins").unwrap().num_minutes(), 5);
        assert_eq!(parse_relative_duration("5minute").unwrap().num_minutes(), 5);
        assert_eq!(
            parse_relative_duration("5minutes").unwrap().num_minutes(),
            5
        );
    }

    #[test]
    fn test_parse_relative_duration_hours() {
        assert_eq!(parse_relative_duration("1h").unwrap().num_hours(), 1);
        assert_eq!(parse_relative_duration("24h").unwrap().num_hours(), 24);
        assert_eq!(parse_relative_duration("2hr").unwrap().num_hours(), 2);
        assert_eq!(parse_relative_duration("2hrs").unwrap().num_hours(), 2);
        assert_eq!(parse_relative_duration("2hour").unwrap().num_hours(), 2);
        assert_eq!(parse_relative_duration("2hours").unwrap().num_hours(), 2);
    }

    #[test]
    fn test_parse_relative_duration_days() {
        assert_eq!(parse_relative_duration("7d").unwrap().num_days(), 7);
        assert_eq!(parse_relative_duration("1d").unwrap().num_days(), 1);
        assert_eq!(parse_relative_duration("30d").unwrap().num_days(), 30);
        assert_eq!(parse_relative_duration("1day").unwrap().num_days(), 1);
        assert_eq!(parse_relative_duration("2days").unwrap().num_days(), 2);
    }

    #[test]
    fn test_parse_relative_duration_weeks() {
        assert_eq!(parse_relative_duration("1w").unwrap().num_weeks(), 1);
        assert_eq!(parse_relative_duration("2w").unwrap().num_weeks(), 2);
        assert_eq!(parse_relative_duration("1wk").unwrap().num_weeks(), 1);
        assert_eq!(parse_relative_duration("1wks").unwrap().num_weeks(), 1);
        assert_eq!(parse_relative_duration("1week").unwrap().num_weeks(), 1);
        assert_eq!(parse_relative_duration("2weeks").unwrap().num_weeks(), 2);
    }

    #[test]
    fn test_parse_relative_duration_case_insensitive() {
        assert_eq!(parse_relative_duration("1H").unwrap().num_hours(), 1);
        assert_eq!(parse_relative_duration("1D").unwrap().num_days(), 1);
        assert_eq!(parse_relative_duration("1W").unwrap().num_weeks(), 1);
        assert_eq!(parse_relative_duration("1M").unwrap().num_minutes(), 1);
    }

    #[test]
    fn test_parse_relative_duration_whitespace() {
        assert_eq!(parse_relative_duration(" 1h ").unwrap().num_hours(), 1);
        assert_eq!(parse_relative_duration("  7d  ").unwrap().num_days(), 7);
    }

    #[test]
    fn test_parse_relative_duration_invalid() {
        assert!(parse_relative_duration("").is_err());
        assert!(parse_relative_duration("h").is_err()); // No number
        assert!(parse_relative_duration("123").is_err()); // No unit
        assert!(parse_relative_duration("0d").is_err()); // Zero not allowed
        assert!(parse_relative_duration("-1d").is_err()); // Negative not allowed
        assert!(parse_relative_duration("1x").is_err()); // Invalid unit
        assert!(parse_relative_duration("abc").is_err()); // Not a number
    }

    #[test]
    fn test_is_relative_duration() {
        // Valid relative durations
        assert!(is_relative_duration("15m"));
        assert!(is_relative_duration("1h"));
        assert!(is_relative_duration("7d"));
        assert!(is_relative_duration("2w"));
        assert!(is_relative_duration("  1h  ")); // With whitespace

        // Not relative durations (ISO 8601 or other formats)
        assert!(!is_relative_duration("2024-01-01T00:00:00Z"));
        assert!(!is_relative_duration("2024-01-01"));
        assert!(!is_relative_duration(""));
        assert!(!is_relative_duration("abc"));
        assert!(!is_relative_duration("123")); // No unit
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TimePreset Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_time_preset_to_duration() {
        assert_eq!(TimePreset::OneHour.to_duration().num_hours(), 1);
        assert_eq!(TimePreset::ThreeHours.to_duration().num_hours(), 3);
        assert_eq!(TimePreset::SixHours.to_duration().num_hours(), 6);
        assert_eq!(TimePreset::TwelveHours.to_duration().num_hours(), 12);
        assert_eq!(TimePreset::OneDay.to_duration().num_days(), 1);
        assert_eq!(TimePreset::TwoDays.to_duration().num_days(), 2);
        assert_eq!(TimePreset::SevenDays.to_duration().num_days(), 7);
        assert_eq!(TimePreset::FourteenDays.to_duration().num_days(), 14);
    }

    #[test]
    fn test_time_preset_default() {
        assert_eq!(TimePreset::default_preset(), TimePreset::SevenDays);
    }

    #[test]
    fn test_time_preset_description() {
        assert_eq!(TimePreset::OneHour.description(), "Last 1 hour");
        assert_eq!(TimePreset::SevenDays.description(), "Last 7 days");
        assert_eq!(TimePreset::FourteenDays.description(), "Last 14 days");
    }
}
