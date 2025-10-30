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

/// LangSmith Workspace information
///
/// Workspaces are nested under organizations and provide narrower scoping
/// for resources like prompts, traces, and datasets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace ID (UUID)
    pub id: String,
    /// Display name of the workspace
    pub display_name: Option<String>,
    /// The organization this workspace belongs to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Workspace handle/slug
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

    /// List workspaces in an organization
    ///
    /// Returns all workspaces accessible within the current organization context.
    /// If an organization_id is configured on the client, it will be used to scope
    /// the request via the x-organization-id header.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?
    ///     .with_organization_id("org-id".to_string());
    /// let workspaces = client.get_workspaces().await?;
    /// for workspace in workspaces {
    ///     println!("Workspace: {} ({})", workspace.display_name.unwrap_or_default(), workspace.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        let path = "/api/v1/workspaces";
        let request = self.langsmith_get(path)?;
        let workspaces: Vec<Workspace> = self.execute(request).await?;
        Ok(workspaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_deserialization() {
        let json = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "display_name": "Test Org",
            "is_personal": false,
            "handle": "test-org"
        }"#;

        let org: Organization = serde_json::from_str(json).unwrap();
        assert_eq!(
            org.id,
            Some("12345678-1234-1234-1234-123456789012".to_string())
        );
        assert_eq!(org.display_name, Some("Test Org".to_string()));
        assert!(!org.is_personal);
        assert_eq!(org.handle, Some("test-org".to_string()));
    }

    #[test]
    fn test_workspace_deserialization() {
        let json = r#"{
            "id": "workspace-uuid-123",
            "display_name": "Test Workspace",
            "organization_id": "org-uuid-456",
            "handle": "test-workspace"
        }"#;

        let workspace: Workspace = serde_json::from_str(json).unwrap();
        assert_eq!(workspace.id, "workspace-uuid-123");
        assert_eq!(workspace.display_name, Some("Test Workspace".to_string()));
        assert_eq!(workspace.organization_id, Some("org-uuid-456".to_string()));
        assert_eq!(workspace.handle, Some("test-workspace".to_string()));
    }

    #[test]
    fn test_workspace_minimal_deserialization() {
        let json = r#"{
            "id": "workspace-123"
        }"#;

        let workspace: Workspace = serde_json::from_str(json).unwrap();
        assert_eq!(workspace.id, "workspace-123");
        assert!(workspace.display_name.is_none());
        assert!(workspace.organization_id.is_none());
        assert!(workspace.handle.is_none());
    }
}
