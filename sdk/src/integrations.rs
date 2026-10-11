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
    /// * `Err(NoGitHubIntegrationForRepo { .. })` - Every integration listed its
    ///   repositories, and none includes this one
    /// * `Err(...)` - Listing the integrations failed, or listing an
    ///   integration's repositories failed and no other integration matched
    pub async fn find_integration_for_repo(&self, owner: &str, repo: &str) -> Result<String> {
        let integrations = self.list_github_integrations().await?;

        // An integration whose repositories fail to list is skipped, so another
        // integration can still match. Its error is kept and returned when none
        // does, because a 401, 403 or 500 there is not proof of "no access".
        let mut listing_error = None;
        for integration in integrations {
            match self.list_github_repositories(&integration.id).await {
                Ok(repos) => {
                    // GitHub owner and repository names are case-insensitive.
                    if repos.iter().any(|r| {
                        r.owner.eq_ignore_ascii_case(owner) && r.name.eq_ignore_ascii_case(repo)
                    }) {
                        return Ok(integration.id);
                    }
                }
                Err(e) => listing_error = Some(e),
            }
        }

        Err(listing_error.unwrap_or_else(|| {
            crate::error::LangstarError::NoGitHubIntegrationForRepo {
                owner: owner.to_string(),
                repo: repo.to_string(),
            }
        }))
    }
}

impl LangchainClient {
    /// Get an IntegrationClient for interacting with GitHub integrations
    pub fn integrations(&self) -> IntegrationClient<'_> {
        IntegrationClient::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::AuthConfig;
    use crate::client::LangchainClient;
    use crate::error::LangstarError;

    fn mock_client(server: &mockito::ServerGuard) -> LangchainClient {
        LangchainClient::with_base_urls(
            AuthConfig::new(
                Some("test_key".to_string()),
                None,
                Some("test_workspace".to_string()),
            ),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_integration_for_repo_returns_listing_error_without_match() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/v1/integrations/github/install")
            .with_body(r#"[{"id":"forbidden","installation_id":1,"name":"a"}]"#)
            .create_async()
            .await;
        server
            .mock("GET", "/v1/integrations/github/forbidden/repos")
            .with_status(403)
            .with_body(r#"{"detail":"Forbidden"}"#)
            .create_async()
            .await;
        let client = mock_client(&server);

        let err = client
            .integrations()
            .find_integration_for_repo("codekiln", "langstar")
            .await
            .unwrap_err();

        assert!(
            matches!(err, LangstarError::ApiError { status: 403, .. }),
            "expected the 403 from the repository listing, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_find_integration_for_repo_matches_after_a_listing_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/v1/integrations/github/install")
            .with_body(
                r#"[{"id":"broken","installation_id":1,"name":"a"},
                    {"id":"works","installation_id":2,"name":"b"}]"#,
            )
            .create_async()
            .await;
        server
            .mock("GET", "/v1/integrations/github/broken/repos")
            .with_status(500)
            .create_async()
            .await;
        server
            .mock("GET", "/v1/integrations/github/works/repos")
            .with_body(r#"[{"owner":"codekiln","name":"langstar"}]"#)
            .create_async()
            .await;
        let client = mock_client(&server);

        let id = client
            .integrations()
            .find_integration_for_repo("codekiln", "langstar")
            .await
            .unwrap();

        assert_eq!(id, "works");
    }

    #[tokio::test]
    async fn test_find_integration_for_repo_ignores_case() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/v1/integrations/github/install")
            .with_body(r#"[{"id":"works","installation_id":1,"name":"a"}]"#)
            .create_async()
            .await;
        server
            .mock("GET", "/v1/integrations/github/works/repos")
            .with_body(r#"[{"owner":"codekiln","name":"langstar"}]"#)
            .create_async()
            .await;
        let client = mock_client(&server);

        let id = client
            .integrations()
            .find_integration_for_repo("CodeKiln", "LangStar")
            .await
            .unwrap();

        assert_eq!(id, "works");
    }

    #[tokio::test]
    async fn test_find_integration_for_repo_reports_no_match() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/v1/integrations/github/install")
            .with_body(r#"[{"id":"other","installation_id":1,"name":"a"}]"#)
            .create_async()
            .await;
        server
            .mock("GET", "/v1/integrations/github/other/repos")
            .with_body(r#"[{"owner":"someone","name":"elsewhere"}]"#)
            .create_async()
            .await;
        let client = mock_client(&server);

        let err = client
            .integrations()
            .find_integration_for_repo("codekiln", "langstar")
            .await
            .unwrap_err();

        assert!(
            matches!(
                &err,
                LangstarError::NoGitHubIntegrationForRepo { owner, repo }
                    if owner == "codekiln" && repo == "langstar"
            ),
            "expected NoGitHubIntegrationForRepo, got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_find_integration_for_repo_returns_listing_404() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/v1/integrations/github/install")
            .with_body(r#"[{"id":"deleted","installation_id":1,"name":"a"}]"#)
            .create_async()
            .await;
        server
            .mock("GET", "/v1/integrations/github/deleted/repos")
            .with_status(404)
            .with_body(r#"{"detail":"Not Found"}"#)
            .create_async()
            .await;
        let client = mock_client(&server);

        let err = client
            .integrations()
            .find_integration_for_repo("codekiln", "langstar")
            .await
            .unwrap_err();

        // A listing 404 (for example, a deleted integration) is an API error,
        // not "no integration has access".
        assert!(
            matches!(err, LangstarError::ApiError { status: 404, .. }),
            "expected the listing 404, got {:?}",
            err
        );
    }
}
