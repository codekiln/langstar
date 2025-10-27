use crate::client::LangchainClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// LangSmith Organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Organization ID (UUID)
    pub id: Option<String>,
    /// Display name of the organization
    pub display_name: Option<String>,
    /// Whether this is a personal organization
    pub is_personal: bool,
    /// Organization handle/slug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
}

impl LangchainClient {
    /// Get information about the current organization
    ///
    /// This is useful for obtaining the organization ID that may be required
    /// for certain write operations.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    /// let org = client.get_current_organization().await?;
    /// println!("Organization: {:?}", org.display_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_current_organization(&self) -> Result<Organization> {
        let path = "/api/v1/orgs/current";
        let request = self.langsmith_get(path)?;
        let org: Organization = self.execute(request).await?;
        Ok(org)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthConfig;

    #[test]
    fn test_organization_deserialization() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "display_name": "Test Org",
            "is_personal": false,
            "handle": "test-org"
        }"#;

        let org: Organization = serde_json::from_str(json).unwrap();
        assert_eq!(org.id, Some("12345678-1234-1234-1234-123456789012".to_string()));
        assert_eq!(org.display_name, Some("Test Org".to_string()));
        assert_eq!(org.is_personal, false);
        assert_eq!(org.handle, Some("test-org".to_string()));
    }
}
