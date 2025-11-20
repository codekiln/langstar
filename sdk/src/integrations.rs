use crate::client::LangchainClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// A GitHub integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIntegration {
    /// Unique identifier for the integration
    pub id: String,
    /// Name of the integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    /// Repository owner (e.g., "codekiln")
    pub owner: String,
    /// Repository name (e.g., "langstar")
    pub name: String,
}

/// Client for interacting with GitHub integrations
pub struct IntegrationClient<'a> {
    client: &'a LangchainClient,
}

impl<'a> IntegrationClient<'a> {
    /// Create a new IntegrationClient
    pub fn new(client: &'a LangchainClient) -> Self {
        Self { client }
    }

    /// List all GitHub integrations for the workspace
    pub async fn list_github_integrations(&self) -> Result<Vec<GitHubIntegration>> {
        let path = "/v1/integrations/github/install";
        let request = self.client.control_plane_get(path)?;
        let response: Vec<GitHubIntegration> = self.client.execute(request).await?;
        Ok(response)
    }

    /// List repositories for a specific GitHub integration
    ///
    /// # Arguments
    /// * `integration_id` - The UUID of the GitHub integration
    pub async fn list_github_repositories(
        &self,
        integration_id: &str,
    ) -> Result<Vec<GitHubRepository>> {
        let path = format!("/v1/integrations/github/{}/repos", integration_id);
        let request = self.client.control_plane_get(&path)?;
        let response: Vec<GitHubRepository> = self.client.execute(request).await?;
        Ok(response)
    }

    /// Find the integration ID for a specific GitHub repository
    ///
    /// This method searches all GitHub integrations to find which one
    /// has access to the specified repository.
    ///
    /// # Arguments
    /// * `owner` - Repository owner (e.g., "codekiln")
    /// * `repo` - Repository name (e.g., "langstar")
    ///
    /// # Returns
    /// * `Ok(String)` - The integration ID that has access to this repository
    /// * `Err(...)` - If no integration is found or if there's an API error
    pub async fn find_integration_for_repo(&self, owner: &str, repo: &str) -> Result<String> {
        // Get all GitHub integrations
        let integrations = self.list_github_integrations().await?;

        // Check each integration for the target repository
        for integration in integrations {
            let integration_id = integration.id.clone();

            // List repositories for this integration
            match self.list_github_repositories(&integration_id).await {
                Ok(repos) => {
                    // Search for target repository
                    for r in repos {
                        if r.owner == owner && r.name == repo {
                            return Ok(integration_id);
                        }
                    }
                }
                Err(_) => {
                    // Skip integrations that fail to list repos
                    continue;
                }
            }
        }

        Err(crate::error::LangstarError::ApiError {
            status: 404,
            message: format!("No integration found with access to {}/{}", owner, repo),
        })
    }
}

impl LangchainClient {
    /// Get an IntegrationClient for interacting with GitHub integrations
    pub fn integrations(&self) -> IntegrationClient<'_> {
        IntegrationClient::new(self)
    }
}
