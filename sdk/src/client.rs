use crate::auth::AuthConfig;
use crate::error::{LangstarError, Result};
use reqwest::{Client as HttpClient, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Base URLs for LangChain services
pub const LANGSMITH_API_BASE: &str = "https://api.smith.langchain.com";
pub const LANGGRAPH_API_BASE: &str = "https://api.langgraph.cloud";
pub const CONTROL_PLANE_API_BASE: &str = "https://api.host.langchain.com";

/// HTTP client for interacting with LangChain APIs
#[derive(Clone)]
pub struct LangchainClient {
    http_client: HttpClient,
    auth: AuthConfig,
    langsmith_base_url: String,
    langgraph_base_url: String,
    control_plane_base_url: String,
    /// Optional organization ID for API requests (used in x-organization-id header)
    organization_id: Option<String>,
    /// Optional workspace ID for narrower scoping (used in X-Tenant-Id header)
    workspace_id: Option<String>,
}

impl LangchainClient {
    /// Create a new client with the given authentication configuration
    ///
    /// The client will use organization_id and workspace_id from the AuthConfig
    /// to automatically add the appropriate scoping headers to API requests.
    pub fn new(auth: AuthConfig) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let organization_id = auth.organization_id.clone();
        let workspace_id = auth.workspace_id.clone();

        Ok(Self {
            http_client,
            auth,
            langsmith_base_url: LANGSMITH_API_BASE.to_string(),
            langgraph_base_url: LANGGRAPH_API_BASE.to_string(),
            control_plane_base_url: CONTROL_PLANE_API_BASE.to_string(),
            organization_id,
            workspace_id,
        })
    }

    /// Set the organization ID for API requests
    ///
    /// Some write operations may require an organization ID to be specified.
    /// This adds the x-organization-id header to subsequent requests.
    pub fn with_organization_id(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// Get the current organization ID if set
    pub fn organization_id(&self) -> Option<&str> {
        self.organization_id.as_deref()
    }

    /// Set the workspace ID for API requests
    ///
    /// Workspace ID provides narrower scoping than organization ID.
    /// This adds the X-Tenant-Id header to subsequent requests.
    /// Per LangSmith documentation, both x-organization-id and X-Tenant-Id
    /// can be used together for workspace-scoped requests.
    pub fn with_workspace_id(mut self, workspace_id: String) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    /// Get the current workspace ID if set
    pub fn workspace_id(&self) -> Option<&str> {
        self.workspace_id.as_deref()
    }

    /// Create a new client with custom base URLs (useful for testing)
    pub fn with_base_urls(
        auth: AuthConfig,
        langsmith_base_url: String,
        langgraph_base_url: String,
        control_plane_base_url: String,
    ) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let organization_id = auth.organization_id.clone();
        let workspace_id = auth.workspace_id.clone();

        Ok(Self {
            http_client,
            auth,
            langsmith_base_url,
            langgraph_base_url,
            control_plane_base_url,
            organization_id,
            workspace_id,
        })
    }

    /// Create a GET request to LangSmith API
    ///
    /// Per LangSmith documentation, both x-organization-id and X-Tenant-Id
    /// headers can be used together for workspace-scoped requests.
    pub fn langsmith_get(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .get(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set (should be present on all requests per docs)
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
        }

        // Add workspace ID header if set (for workspace-scoped requests)
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        Ok(request)
    }

    /// Create a POST request to LangSmith API
    ///
    /// Per LangSmith documentation, both x-organization-id and X-Tenant-Id
    /// headers can be used together for workspace-scoped requests.
    pub fn langsmith_post(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set (should be present on all requests per docs)
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
        }

        // Add workspace ID header if set (for workspace-scoped requests)
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        Ok(request)
    }

    /// Create a PUT request to LangSmith API
    ///
    /// Per LangSmith documentation, both x-organization-id and X-Tenant-Id
    /// headers can be used together for workspace-scoped requests.
    pub fn langsmith_put(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .put(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set (should be present on all requests per docs)
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
        }

        // Add workspace ID header if set (for workspace-scoped requests)
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        Ok(request)
    }

    /// Create a GET request to Control Plane API
    ///
    /// The Control Plane API uses the same authentication as LangSmith:
    /// X-Api-Key (LangSmith API key) and X-Tenant-Id (workspace ID) headers.
    pub fn control_plane_get(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.control_plane_base_url, path);

        let mut request = self
            .http_client
            .get(&url)
            .header("X-Api-Key", api_key)
            .header("Content-Type", "application/json");

        // Add workspace ID header if set (required for Control Plane API)
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        Ok(request)
    }

    /// Create a GET request to LangGraph API
    pub fn langgraph_get(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langgraph_key()?;
        let url = format!("{}{}", self.langgraph_base_url, path);

        Ok(self
            .http_client
            .get(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json"))
    }

    /// Create a POST request to LangGraph API
    pub fn langgraph_post(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langgraph_key()?;
        let url = format!("{}{}", self.langgraph_base_url, path);

        Ok(self
            .http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json"))
    }

    /// Create a PATCH request to LangGraph API
    pub fn langgraph_patch(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langgraph_key()?;
        let url = format!("{}{}", self.langgraph_base_url, path);

        Ok(self
            .http_client
            .patch(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json"))
    }

    /// Create a DELETE request to LangGraph API
    pub fn langgraph_delete(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langgraph_key()?;
        let url = format!("{}{}", self.langgraph_base_url, path);

        Ok(self
            .http_client
            .delete(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json"))
    }

    /// Execute a request and parse the response
    pub async fn execute<T: for<'de> Deserialize<'de>>(
        &self,
        request: RequestBuilder,
    ) -> Result<T> {
        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LangstarError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    /// Get the underlying HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }
}

/// Generic response wrapper for paginated API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let auth = AuthConfig::new(
            Some("test_key".to_string()),
            Some("test_key".to_string()),
            None,
            None,
        );
        let client = LangchainClient::new(auth);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_missing_auth() {
        let auth = AuthConfig::new(None, None, None, None);
        let client = LangchainClient::new(auth).unwrap();

        // Should fail when trying to make authenticated requests
        assert!(client.langsmith_get("/test").is_err());
        assert!(client.langgraph_get("/test").is_err());
    }

    #[test]
    fn test_client_with_org_and_workspace() {
        let auth = AuthConfig::new(
            Some("test_key".to_string()),
            None,
            Some("org_123".to_string()),
            Some("workspace_456".to_string()),
        );
        let client = LangchainClient::new(auth).unwrap();
        assert_eq!(client.organization_id(), Some("org_123"));
        assert_eq!(client.workspace_id(), Some("workspace_456"));
    }

    #[test]
    fn test_client_builder_methods() {
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None, None);
        let client = LangchainClient::new(auth)
            .unwrap()
            .with_organization_id("new_org".to_string())
            .with_workspace_id("new_workspace".to_string());

        assert_eq!(client.organization_id(), Some("new_org"));
        assert_eq!(client.workspace_id(), Some("new_workspace"));
    }
}
