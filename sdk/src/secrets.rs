//! Workspace secrets types for LangSmith secrets management API.
//!
//! This module provides types for managing workspace secrets in LangSmith.
//! Workspace secrets store sensitive values (API keys, credentials, etc.) that
//! can be used in LangSmith deployments and evaluations.
//!
//! # Security Model
//!
//! - Secret values are NEVER returned by the API (list endpoint returns only keys)
//! - Upsert pattern: POST endpoint handles both create and update operations
//! - Delete via upsert: Setting `value: null` in upsert request deletes the secret
//! - Requires `workspaces:manage` permission for write operations
//!
//! # API Reference
//!
//! - Endpoint: `/api/v1/workspaces/current/secrets`
//! - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
//!
//! # Examples
//!
//! ## List secret keys
//!
//! ```no_run
//! # use langstar_sdk::{AuthConfig, LangchainClient};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let auth = AuthConfig::from_env()?;
//! let client = LangchainClient::new(auth)?;
//!
//! let keys = client.list_workspace_secrets().await?;
//! for key in keys {
//!     println!("Secret: {}", key.key);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Create or update secrets
//!
//! ```no_run
//! # use langstar_sdk::{AuthConfig, LangchainClient};
//! # use langstar_sdk::secrets::SecretUpsert;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let auth = AuthConfig::from_env()?;
//! let client = LangchainClient::new(auth)?;
//!
//! let secrets = vec![
//!     SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-..."),
//!     SecretUpsert::set("OPENAI_API_KEY", "sk-..."),
//! ];
//! client.upsert_workspace_secrets(secrets).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Delete a secret
//!
//! ```no_run
//! # use langstar_sdk::{AuthConfig, LangchainClient};
//! # use langstar_sdk::secrets::SecretUpsert;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let auth = AuthConfig::from_env()?;
//! let client = LangchainClient::new(auth)?;
//!
//! // Option 1: Using convenience method
//! client.delete_workspace_secret("OLD_API_KEY").await?;
//!
//! // Option 2: Using upsert with null value
//! let secrets = vec![SecretUpsert::delete("OLD_API_KEY")];
//! client.upsert_workspace_secrets(secrets).await?;
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Response Types
// ============================================================================

/// A workspace secret key (response from list endpoint).
///
/// Security note: Secret values are NEVER returned by the API. The list
/// endpoint returns only the key names, never the values.
///
/// # API Reference
///
/// Maps to list response from `GET /api/v1/workspaces/current/secrets`.
/// The API returns an array of objects with only the `key` field.
///
/// # Example
///
/// ```
/// use langstar_sdk::secrets::SecretKey;
/// use serde_json::json;
///
/// let json_data = json!({
///     "key": "ANTHROPIC_API_KEY"
/// });
///
/// let secret_key: SecretKey = serde_json::from_value(json_data).unwrap();
/// assert_eq!(secret_key.key, "ANTHROPIC_API_KEY");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretKey {
    /// The secret key name (e.g., "ANTHROPIC_API_KEY")
    ///
    /// This is the only field returned by the list endpoint.
    /// Secret values are never exposed through the API.
    pub key: String,
}

// ============================================================================
// Request Types
// ============================================================================

/// Request to create or update workspace secrets (upsert operation).
///
/// This type is used for both creating new secrets and updating existing ones.
/// The API uses a single POST endpoint that handles both operations based on
/// whether the key already exists.
///
/// # Delete Semantics
///
/// Setting `value` to `None` (which serializes as `null` in JSON) deletes
/// the secret. This is the API's delete mechanism - there is no separate
/// DELETE endpoint.
///
/// # API Reference
///
/// Request body for `POST /api/v1/workspaces/current/secrets`.
/// The endpoint accepts an array of these objects.
///
/// # Examples
///
/// ## Set a secret value
///
/// ```
/// use langstar_sdk::secrets::SecretUpsert;
///
/// let secret = SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-...");
/// assert_eq!(secret.key, "ANTHROPIC_API_KEY");
/// assert!(secret.value.is_some());
/// ```
///
/// ## Delete a secret
///
/// ```
/// use langstar_sdk::secrets::SecretUpsert;
///
/// let secret = SecretUpsert::delete("OLD_API_KEY");
/// assert_eq!(secret.key, "OLD_API_KEY");
/// assert!(secret.value.is_none());
/// ```
///
/// ## Batch upsert
///
/// ```
/// use langstar_sdk::secrets::SecretUpsert;
/// use serde_json;
///
/// let secrets = vec![
///     SecretUpsert::set("API_KEY_1", "value1"),
///     SecretUpsert::set("API_KEY_2", "value2"),
///     SecretUpsert::delete("OLD_KEY"),
/// ];
///
/// let json = serde_json::to_string(&secrets).unwrap();
/// assert!(json.contains("API_KEY_1"));
/// assert!(json.contains("value1"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretUpsert {
    /// The secret key name
    ///
    /// This identifies which secret to create, update, or delete.
    /// Key names are case-sensitive.
    pub key: String,

    /// The secret value, or None to delete
    ///
    /// - `Some(value)`: Creates or updates the secret with the given value
    /// - `None`: Deletes the secret (serializes as `null` in JSON)
    ///
    /// The `skip_serializing_if` attribute ensures that when `value` is `None`,
    /// it's serialized as `{"key": "...", "value": null}` rather than omitting
    /// the field entirely.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl SecretUpsert {
    /// Create a new secret upsert request to set a value.
    ///
    /// This is a convenience constructor for creating or updating a secret.
    ///
    /// # Example
    ///
    /// ```
    /// use langstar_sdk::secrets::SecretUpsert;
    ///
    /// let secret = SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-...");
    /// ```
    pub fn set(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: Some(value.into()),
        }
    }

    /// Create a new secret upsert request to delete a secret.
    ///
    /// This is a convenience constructor for deleting a secret.
    /// Internally, this creates an upsert request with `value: None`,
    /// which the API interprets as a delete operation.
    ///
    /// # Example
    ///
    /// ```
    /// use langstar_sdk::secrets::SecretUpsert;
    ///
    /// let secret = SecretUpsert::delete("OLD_API_KEY");
    /// ```
    pub fn delete(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========================================================================
    // SecretKey Tests
    // ========================================================================

    #[test]
    fn test_secret_key_deserialize() {
        let json_data = json!({
            "key": "ANTHROPIC_API_KEY"
        });

        let secret_key: SecretKey = serde_json::from_value(json_data).unwrap();
        assert_eq!(secret_key.key, "ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_secret_key_serialize() {
        let secret_key = SecretKey {
            key: "ANTHROPIC_API_KEY".to_string(),
        };

        let json = serde_json::to_value(&secret_key).unwrap();
        assert_eq!(json["key"], "ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_secret_key_list_deserialize() {
        let json_data = json!([
            {"key": "ANTHROPIC_API_KEY"},
            {"key": "OPENAI_API_KEY"},
            {"key": "DATABASE_URL"}
        ]);

        let keys: Vec<SecretKey> = serde_json::from_value(json_data).unwrap();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0].key, "ANTHROPIC_API_KEY");
        assert_eq!(keys[1].key, "OPENAI_API_KEY");
        assert_eq!(keys[2].key, "DATABASE_URL");
    }

    // ========================================================================
    // SecretUpsert Tests
    // ========================================================================

    #[test]
    fn test_secret_upsert_set_constructor() {
        let secret = SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-test123");
        assert_eq!(secret.key, "ANTHROPIC_API_KEY");
        assert_eq!(secret.value, Some("sk-ant-test123".to_string()));
    }

    #[test]
    fn test_secret_upsert_delete_constructor() {
        let secret = SecretUpsert::delete("OLD_API_KEY");
        assert_eq!(secret.key, "OLD_API_KEY");
        assert_eq!(secret.value, None);
    }

    #[test]
    fn test_secret_upsert_set_serialize() {
        let secret = SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-test123");
        let json = serde_json::to_value(&secret).unwrap();

        assert_eq!(json["key"], "ANTHROPIC_API_KEY");
        assert_eq!(json["value"], "sk-ant-test123");
    }

    #[test]
    fn test_secret_upsert_delete_serialize() {
        let secret = SecretUpsert::delete("OLD_API_KEY");
        let json = serde_json::to_value(&secret).unwrap();

        assert_eq!(json["key"], "OLD_API_KEY");
        // When value is None, skip_serializing_if should omit the field
        assert!(!json.as_object().unwrap().contains_key("value"));
    }

    #[test]
    fn test_secret_upsert_set_deserialize() {
        let json_data = json!({
            "key": "ANTHROPIC_API_KEY",
            "value": "sk-ant-test123"
        });

        let secret: SecretUpsert = serde_json::from_value(json_data).unwrap();
        assert_eq!(secret.key, "ANTHROPIC_API_KEY");
        assert_eq!(secret.value, Some("sk-ant-test123".to_string()));
    }

    #[test]
    fn test_secret_upsert_delete_deserialize() {
        let json_data = json!({
            "key": "OLD_API_KEY",
            "value": null
        });

        let secret: SecretUpsert = serde_json::from_value(json_data).unwrap();
        assert_eq!(secret.key, "OLD_API_KEY");
        assert_eq!(secret.value, None);
    }

    #[test]
    fn test_secret_upsert_batch_serialize() {
        let secrets = vec![
            SecretUpsert::set("API_KEY_1", "value1"),
            SecretUpsert::set("API_KEY_2", "value2"),
            SecretUpsert::delete("OLD_KEY"),
        ];

        let json = serde_json::to_value(&secrets).unwrap();
        let array = json.as_array().unwrap();

        assert_eq!(array.len(), 3);
        assert_eq!(array[0]["key"], "API_KEY_1");
        assert_eq!(array[0]["value"], "value1");
        assert_eq!(array[1]["key"], "API_KEY_2");
        assert_eq!(array[1]["value"], "value2");
        assert_eq!(array[2]["key"], "OLD_KEY");
        assert!(!array[2].as_object().unwrap().contains_key("value"));
    }

    #[test]
    fn test_secret_upsert_batch_deserialize() {
        let json_data = json!([
            {"key": "API_KEY_1", "value": "value1"},
            {"key": "API_KEY_2", "value": "value2"},
            {"key": "OLD_KEY", "value": null}
        ]);

        let secrets: Vec<SecretUpsert> = serde_json::from_value(json_data).unwrap();
        assert_eq!(secrets.len(), 3);
        assert_eq!(secrets[0].key, "API_KEY_1");
        assert_eq!(secrets[0].value, Some("value1".to_string()));
        assert_eq!(secrets[1].key, "API_KEY_2");
        assert_eq!(secrets[1].value, Some("value2".to_string()));
        assert_eq!(secrets[2].key, "OLD_KEY");
        assert_eq!(secrets[2].value, None);
    }

    // ========================================================================
    // Roundtrip Tests
    // ========================================================================

    #[test]
    fn test_secret_key_roundtrip() {
        let original = SecretKey {
            key: "TEST_KEY".to_string(),
        };

        let json = serde_json::to_value(&original).unwrap();
        let deserialized: SecretKey = serde_json::from_value(json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_secret_upsert_set_roundtrip() {
        let original = SecretUpsert::set("TEST_KEY", "test_value");

        let json = serde_json::to_value(&original).unwrap();
        let deserialized: SecretUpsert = serde_json::from_value(json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_secret_upsert_delete_roundtrip() {
        let original = SecretUpsert::delete("TEST_KEY");

        let json = serde_json::to_value(&original).unwrap();
        let deserialized: SecretUpsert = serde_json::from_value(json).unwrap();

        assert_eq!(original, deserialized);
    }
}
