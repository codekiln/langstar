use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer};

/// Deserialize DateTime<Utc> that accepts both formats:
/// - RFC 3339 with timezone: "2025-12-02T16:28:50.113929Z" or "...+00:00"
/// - ISO 8601 without timezone: "2025-12-02T16:28:50.113929" (assumes UTC)
pub fn deserialize_flexible_datetime<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;

    // Try RFC 3339 format with timezone first
    if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Fall back: parse as naive datetime and assume UTC
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
        .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
        .map_err(serde::de::Error::custom)
}

/// Deserialize Option<DateTime<Utc>> with flexible format handling
pub fn deserialize_flexible_datetime_opt<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(d)?;

    match s {
        None => Ok(None),
        Some(s) => {
            // Try RFC 3339 with timezone
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }

            // Fall back: assume UTC
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|naive| Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)))
                .map_err(serde::de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_flexible_datetime")]
        timestamp: DateTime<Utc>,
    }

    #[derive(Debug, Deserialize)]
    struct TestStructOpt {
        #[serde(
            skip_serializing_if = "Option::is_none",
            deserialize_with = "deserialize_flexible_datetime_opt"
        )]
        timestamp: Option<DateTime<Utc>>,
    }

    #[test]
    fn test_deserialize_datetime_with_timezone_z() {
        let json = r#"{"timestamp": "2025-09-15T18:00:09.568206Z"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Should parse timestamp with Z timezone");
    }

    #[test]
    fn test_deserialize_datetime_with_timezone_offset() {
        let json = r#"{"timestamp": "2025-09-15T18:00:09.568206+00:00"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Should parse timestamp with +00:00 timezone"
        );
    }

    #[test]
    fn test_deserialize_datetime_without_timezone() {
        let json = r#"{"timestamp": "2025-12-02T01:29:20.134633"}"#;
        let result: Result<TestStruct, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Should parse timestamp without timezone, assuming UTC"
        );
    }

    #[test]
    fn test_deserialize_datetime_opt_null() {
        let json = r#"{"timestamp": null}"#;
        let result: Result<TestStructOpt, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Should parse null timestamp");
        assert!(result.unwrap().timestamp.is_none());
    }

    #[test]
    fn test_deserialize_datetime_opt_with_timezone() {
        let json = r#"{"timestamp": "2025-09-15T18:00:09.568206+00:00"}"#;
        let result: Result<TestStructOpt, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Should parse optional timestamp with timezone"
        );
        assert!(result.unwrap().timestamp.is_some());
    }

    #[test]
    fn test_deserialize_datetime_opt_without_timezone() {
        let json = r#"{"timestamp": "2025-12-02T01:29:20.134633"}"#;
        let result: Result<TestStructOpt, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Should parse optional timestamp without timezone"
        );
        assert!(result.unwrap().timestamp.is_some());
    }

    #[test]
    fn test_datetime_values_match() {
        // Verify that timestamps with and without timezone parse to the same value
        let json_with_tz = r#"{"timestamp": "2025-09-15T18:00:09.568206+00:00"}"#;
        let json_without_tz = r#"{"timestamp": "2025-09-15T18:00:09.568206"}"#;

        let with_tz: TestStruct = serde_json::from_str(json_with_tz).unwrap();
        let without_tz: TestStruct = serde_json::from_str(json_without_tz).unwrap();

        assert_eq!(
            with_tz.timestamp, without_tz.timestamp,
            "Timestamps should be equal regardless of timezone suffix"
        );
    }
}
