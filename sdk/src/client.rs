use crate::auth::AuthConfig;
use crate::error::{LangstarError, Result};
use reqwest::{Client as HttpClient, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Base URLs for LangChain services
pub const LANGSMITH_API_BASE: &str = "https://api.smith.langchain.com";
pub const LANGGRAPH_API_BASE: &str = "https://api.langgraph.cloud";

/// HTTP client for interacting with LangChain APIs
#[derive(Clone)]
pub struct LangchainClient {
    http_client: HttpClient,
    auth: AuthConfig,
    langsmith_base_url: String,
    langgraph_base_url: String,
    /// Optional organization ID for API requests (used in X-Organization-Id header)
    organization_id: Option<String>,
}

impl LangchainClient {
    /// Create a new client with the given authentication configuration
    pub fn new(auth: AuthConfig) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            http_client,
            auth,
            langsmith_base_url: LANGSMITH_API_BASE.to_string(),
            langgraph_base_url: LANGGRAPH_API_BASE.to_string(),
            organization_id: None,
        })
    }

    /// Set the organization ID for API requests
    ///
    /// Some write operations may require an organization ID to be specified.
    /// This adds the X-Organization-Id header to subsequent requests.
    pub fn with_organization_id(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// Get the current organization ID if set
    pub fn organization_id(&self) -> Option<&str> {
        self.organization_id.as_deref()
    }

    /// Create a new client with custom base URLs (useful for testing)
    pub fn with_base_urls(
        auth: AuthConfig,
        langsmith_base_url: String,
        langgraph_base_url: String,
    ) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            http_client,
            auth,
            langsmith_base_url,
            langgraph_base_url,
            organization_id: None,
        })
    }

    /// Create a GET request to LangSmith API
    pub fn langsmith_get(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .get(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
        }

        Ok(request)
    }

    /// Create a POST request to LangSmith API
    pub fn langsmith_post(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
        }

        Ok(request)
    }

    /// Create a PUT request to LangSmith API
    pub fn langsmith_put(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .put(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
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
        let auth = AuthConfig::new(Some("test_key".to_string()), Some("test_key".to_string()));
        let client = LangchainClient::new(auth);
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_missing_auth() {
        let auth = AuthConfig::new(None, None);
        let client = LangchainClient::new(auth).unwrap();

        // Should fail when trying to make authenticated requests
        assert!(client.langsmith_get("/test").is_err());
        assert!(client.langgraph_get("/test").is_err());
    }
}
