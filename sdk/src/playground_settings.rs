//! Playground settings types for LangSmith model configuration API.
//!
//! This module provides types for managing model configurations (playground settings)
//! in LangSmith. Playground settings store model provider configurations including
//! API keys, model parameters, and rate limits used in the Prompt Hub playground.
//!
//! # API Reference
//!
//! - Endpoint: `/api/v1/playground-settings`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//!
//! # Example
//!
//! ```no_run
//! use langstar_sdk::playground_settings::{PlaygroundSettingsCreateRequest, PlaygroundSavedOptions};
//! use serde_json::json;
//!
//! let request = PlaygroundSettingsCreateRequest {
//!     name: Some("Claude 3.5 Sonnet Config".to_string()),
//!     description: Some("Production configuration for Claude".to_string()),
//!     settings: json!({
//!         "lc": 1,
//!         "type": "constructor",
//!         "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
//!         "kwargs": {
//!             "model": "claude-3-5-sonnet-20241022",
//!             "temperature": 0.0
//!         }
//!     }),
//!     options: PlaygroundSavedOptions {
//!         requests_per_second: Some(10),
//!     },
//! };
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// ============================================================================
// Response Types
// ============================================================================

/// Playground settings response from the API.
///
/// Represents a saved model configuration as returned by the LangSmith API.
/// The `settings` field contains LangChain-serialized model configuration
/// in LC-JSON format.
///
/// # API Reference
///
/// Maps to `PlaygroundSettingsResponse` in OpenAPI spec.
/// Returned by `GET /playground-settings`, `GET /playground-settings/{id}`,
/// `POST /playground-settings`, `PATCH /playground-settings/{id}`
///
/// # Example
///
/// ```
/// use langstar_sdk::playground_settings::PlaygroundSettingsResponse;
/// use serde_json::json;
///
/// let json_data = json!({
///     "id": "550e8400-e29b-41d4-a716-446655440000",
///     "settings": {
///         "lc": 1,
///         "type": "constructor",
///         "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
///         "kwargs": {"model": "claude-3-5-sonnet-20241022"}
///     },
///     "options": {"requests_per_second": 10},
///     "name": "My Config",
///     "description": null,
///     "created_at": "2024-01-01T00:00:00Z",
///     "updated_at": "2024-01-01T00:00:00Z"
/// });
///
/// let settings: PlaygroundSettingsResponse = serde_json::from_value(json_data).unwrap();
/// assert_eq!(settings.name, Some("My Config".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaygroundSettingsResponse {
    /// Unique identifier for the playground settings
    pub id: Uuid,

    /// LangChain-serialized model configuration (LC-JSON format).
    ///
    /// This is a dynamic JSON object containing the model provider configuration.
    /// The structure follows LangChain's serialization format with `lc`, `type`,
    /// `id`, and `kwargs` fields.
    pub settings: Value,

    /// Optional saved options including rate limiting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PlaygroundSavedOptions>,

    /// Optional human-readable name for the configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional description of the configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When the settings were created
    pub created_at: DateTime<Utc>,

    /// When the settings were last updated
    pub updated_at: DateTime<Utc>,
}

/// Saved options for playground settings.
///
/// Contains configuration options that are not part of the model configuration
/// itself but affect how it is used.
///
/// # Example
///
/// ```
/// use langstar_sdk::playground_settings::PlaygroundSavedOptions;
///
/// let options = PlaygroundSavedOptions {
///     requests_per_second: Some(10),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlaygroundSavedOptions {
    /// Rate limit: maximum requests per second.
    ///
    /// Used to prevent hitting provider rate limits during batch operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_second: Option<i32>,
}

// ============================================================================
// Request Types
// ============================================================================

/// Request to create new playground settings.
///
/// # API Reference
///
/// Request body for `POST /playground-settings`
///
/// # Example
///
/// ```
/// use langstar_sdk::playground_settings::{PlaygroundSettingsCreateRequest, PlaygroundSavedOptions};
/// use serde_json::json;
///
/// let request = PlaygroundSettingsCreateRequest {
///     name: Some("OpenAI GPT-4".to_string()),
///     description: Some("GPT-4 configuration for evaluation".to_string()),
///     settings: json!({
///         "lc": 1,
///         "type": "constructor",
///         "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
///         "kwargs": {
///             "model": "gpt-4-turbo",
///             "temperature": 0.0
///         }
///     }),
///     options: PlaygroundSavedOptions {
///         requests_per_second: Some(5),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundSettingsCreateRequest {
    /// Optional human-readable name for the configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional description of the configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// LangChain-serialized model configuration (LC-JSON format) - required
    pub settings: Value,

    /// Options including rate limiting
    pub options: PlaygroundSavedOptions,
}

/// Request to update existing playground settings.
///
/// All fields are optional - only provided fields will be updated.
///
/// # API Reference
///
/// Request body for `PATCH /playground-settings/{id}`
///
/// # Example
///
/// ```
/// use langstar_sdk::playground_settings::{PlaygroundSettingsUpdateRequest, PlaygroundSavedOptions};
///
/// // Update only the name and rate limit
/// let request = PlaygroundSettingsUpdateRequest {
///     name: Some("Updated Config Name".to_string()),
///     description: None,
///     settings: None,
///     options: Some(PlaygroundSavedOptions {
///         requests_per_second: Some(20),
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaygroundSettingsUpdateRequest {
    /// New name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// New description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// New settings (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Value>,

    /// New options (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PlaygroundSavedOptions>,
}

// ============================================================================
// List Parameters
// ============================================================================

/// Parameters for listing playground settings.
///
/// # Example
///
/// ```
/// use langstar_sdk::playground_settings::ListPlaygroundSettingsParams;
///
/// let params = ListPlaygroundSettingsParams {
///     limit: Some(50),
///     offset: Some(0),
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ListPlaygroundSettingsParams {
    /// Maximum number of items to return (default: 20)
    pub limit: Option<u32>,

    /// Number of items to skip (default: 0)
    pub offset: Option<u32>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========================================================================
    // PlaygroundSettingsResponse Tests
    // ========================================================================

    #[test]
    fn test_playground_settings_response_deserialization() {
        let json_data = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "settings": {
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
                "kwargs": {
                    "model": "claude-3-5-sonnet-20241022",
                    "temperature": 0.0
                }
            },
            "options": {
                "requests_per_second": 10
            },
            "name": "Claude Config",
            "description": "Production Claude configuration",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z"
        });

        let settings: PlaygroundSettingsResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(
            settings.id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(settings.name, Some("Claude Config".to_string()));
        assert_eq!(
            settings.description,
            Some("Production Claude configuration".to_string())
        );
        assert_eq!(
            settings.options.as_ref().unwrap().requests_per_second,
            Some(10)
        );

        // Verify settings structure
        assert_eq!(settings.settings["lc"], 1);
        assert_eq!(settings.settings["type"], "constructor");
        assert_eq!(
            settings.settings["kwargs"]["model"],
            "claude-3-5-sonnet-20241022"
        );
    }

    #[test]
    fn test_playground_settings_response_minimal() {
        // Test with only required fields
        let json_data = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "settings": {"key": "value"},
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z"
        });

        let settings: PlaygroundSettingsResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(
            settings.id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(settings.name, None);
        assert_eq!(settings.description, None);
        assert_eq!(settings.options, None);
    }

    #[test]
    fn test_playground_settings_response_round_trip() {
        let original = PlaygroundSettingsResponse {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            settings: json!({
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                "kwargs": {"model": "gpt-4"}
            }),
            options: Some(PlaygroundSavedOptions {
                requests_per_second: Some(5),
            }),
            name: Some("Test Config".to_string()),
            description: Some("Test description".to_string()),
            created_at: DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: PlaygroundSettingsResponse = serde_json::from_str(&json_str).unwrap();

        assert_eq!(original, deserialized);
    }

    // ========================================================================
    // PlaygroundSavedOptions Tests
    // ========================================================================

    #[test]
    fn test_playground_saved_options_serialization() {
        let options = PlaygroundSavedOptions {
            requests_per_second: Some(10),
        };

        let json = serde_json::to_value(&options).unwrap();
        assert_eq!(json["requests_per_second"], 10);
    }

    #[test]
    fn test_playground_saved_options_none() {
        let options = PlaygroundSavedOptions {
            requests_per_second: None,
        };

        let json = serde_json::to_value(&options).unwrap();
        // With skip_serializing_if, the field should be absent
        assert!(
            !json
                .as_object()
                .unwrap()
                .contains_key("requests_per_second")
        );
    }

    #[test]
    fn test_playground_saved_options_default() {
        let options = PlaygroundSavedOptions::default();
        assert_eq!(options.requests_per_second, None);
    }

    // ========================================================================
    // PlaygroundSettingsCreateRequest Tests
    // ========================================================================

    #[test]
    fn test_create_request_serialization() {
        let request = PlaygroundSettingsCreateRequest {
            name: Some("My Config".to_string()),
            description: Some("Description".to_string()),
            settings: json!({
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
                "kwargs": {"model": "claude-3-5-sonnet-20241022"}
            }),
            options: PlaygroundSavedOptions {
                requests_per_second: Some(10),
            },
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["name"], "My Config");
        assert_eq!(json["description"], "Description");
        assert_eq!(json["settings"]["lc"], 1);
        assert_eq!(json["options"]["requests_per_second"], 10);
    }

    #[test]
    fn test_create_request_minimal() {
        let request = PlaygroundSettingsCreateRequest {
            name: None,
            description: None,
            settings: json!({"key": "value"}),
            options: PlaygroundSavedOptions::default(),
        };

        let json = serde_json::to_value(&request).unwrap();

        // Optional fields with None should be absent
        assert!(!json.as_object().unwrap().contains_key("name"));
        assert!(!json.as_object().unwrap().contains_key("description"));

        // Required fields should be present
        assert!(json.as_object().unwrap().contains_key("settings"));
        assert!(json.as_object().unwrap().contains_key("options"));
    }

    // ========================================================================
    // PlaygroundSettingsUpdateRequest Tests
    // ========================================================================

    #[test]
    fn test_update_request_partial() {
        let request = PlaygroundSettingsUpdateRequest {
            name: Some("New Name".to_string()),
            description: None,
            settings: None,
            options: None,
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["name"], "New Name");
        // Other fields should be absent (skip_serializing_if)
        assert!(!json.as_object().unwrap().contains_key("description"));
        assert!(!json.as_object().unwrap().contains_key("settings"));
        assert!(!json.as_object().unwrap().contains_key("options"));
    }

    #[test]
    fn test_update_request_full() {
        let request = PlaygroundSettingsUpdateRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated Description".to_string()),
            settings: Some(json!({"updated": true})),
            options: Some(PlaygroundSavedOptions {
                requests_per_second: Some(20),
            }),
        };

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["name"], "Updated Name");
        assert_eq!(json["description"], "Updated Description");
        assert_eq!(json["settings"]["updated"], true);
        assert_eq!(json["options"]["requests_per_second"], 20);
    }

    #[test]
    fn test_update_request_default() {
        let request = PlaygroundSettingsUpdateRequest::default();

        let json = serde_json::to_value(&request).unwrap();

        // All fields should be absent for default
        assert!(json.as_object().unwrap().is_empty());
    }

    // ========================================================================
    // ListPlaygroundSettingsParams Tests
    // ========================================================================

    #[test]
    fn test_list_params_default() {
        let params = ListPlaygroundSettingsParams::default();
        assert_eq!(params.limit, None);
        assert_eq!(params.offset, None);
    }

    #[test]
    fn test_list_params_with_values() {
        let params = ListPlaygroundSettingsParams {
            limit: Some(50),
            offset: Some(100),
        };
        assert_eq!(params.limit, Some(50));
        assert_eq!(params.offset, Some(100));
    }

    // ========================================================================
    // Real API Response Format Tests
    // ========================================================================

    #[test]
    fn test_anthropic_config_format() {
        // Test with real Anthropic configuration format from scout research
        let json_data = json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "settings": {
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
                "kwargs": {
                    "anthropic_api_key": {
                        "lc": 1,
                        "type": "secret",
                        "id": ["ANTHROPIC_API_KEY"]
                    },
                    "model": "claude-3-5-sonnet-20241022",
                    "temperature": 0.0,
                    "max_tokens": 4096
                }
            },
            "options": {
                "requests_per_second": 10
            },
            "name": "Claude 3.5 Sonnet",
            "description": "Production Anthropic configuration",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z"
        });

        let settings: PlaygroundSettingsResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(settings.name, Some("Claude 3.5 Sonnet".to_string()));
        assert_eq!(
            settings.settings["kwargs"]["model"],
            "claude-3-5-sonnet-20241022"
        );
        assert_eq!(settings.settings["id"][3], "ChatAnthropic");
    }

    #[test]
    fn test_openai_config_format() {
        // Test with OpenAI configuration format
        let json_data = json!({
            "id": "660e8400-e29b-41d4-a716-446655440000",
            "settings": {
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                "kwargs": {
                    "openai_api_key": {
                        "lc": 1,
                        "type": "secret",
                        "id": ["OPENAI_API_KEY"]
                    },
                    "model": "gpt-4-turbo",
                    "temperature": 0.7
                }
            },
            "options": {
                "requests_per_second": 5
            },
            "name": "GPT-4 Turbo",
            "description": null,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z"
        });

        let settings: PlaygroundSettingsResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(settings.name, Some("GPT-4 Turbo".to_string()));
        assert_eq!(settings.description, None);
        assert_eq!(settings.settings["kwargs"]["model"], "gpt-4-turbo");
        assert_eq!(settings.settings["id"][3], "ChatOpenAI");
    }

    #[test]
    fn test_bedrock_config_format() {
        // Test with AWS Bedrock configuration format
        let json_data = json!({
            "id": "770e8400-e29b-41d4-a716-446655440000",
            "settings": {
                "lc": 1,
                "type": "constructor",
                "id": ["langchain_aws", "chat_models", "bedrock_converse", "ChatBedrockConverse"],
                "kwargs": {
                    "model": "anthropic.claude-3-5-sonnet-20241022-v2:0",
                    "region_name": "us-east-1"
                }
            },
            "options": {
                "requests_per_second": 3
            },
            "name": "Bedrock Claude",
            "description": "AWS Bedrock Claude configuration",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z"
        });

        let settings: PlaygroundSettingsResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(settings.name, Some("Bedrock Claude".to_string()));
        assert_eq!(settings.settings["id"][0], "langchain_aws");
        assert_eq!(settings.settings["kwargs"]["region_name"], "us-east-1");
    }
}
