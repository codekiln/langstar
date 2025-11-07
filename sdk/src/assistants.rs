//! LangGraph Assistants API
//!
//! This module provides access to the LangGraph Cloud Assistants API.
//!
//! ## Deployment-Level Resources
//!
//! **Important:** Assistants are deployment-level resources in LangGraph Cloud.
//! Unlike LangSmith prompts, they are NOT scoped by organization or workspace.
//!
//! ### What This Means
//!
//! - Each assistant belongs to a specific LangGraph deployment
//! - Your API key determines which deployment you're accessing
//! - No additional scoping headers are needed or supported
//! - Access control is handled entirely at the API key/deployment level
//!
//! ### Comparison with LangSmith Prompts
//!
//! | Aspect | LangSmith Prompts | LangGraph Assistants |
//! |--------|-------------------|----------------------|
//! | Scoping | Organization/Workspace | Deployment |
//! | Headers | `x-organization-id`, `X-Tenant-Id` | `x-api-key` only |
//! | Multi-tenancy | Yes | No |
//! | Access model | Hierarchical | Flat |
//!
//! ## Configuration
//!
//! Assistants require only an API key (same as prompts):
//!
//! ```bash
//! export LANGSMITH_API_KEY="<your-api-key>"
//! ```
//!
//! No organization or workspace configuration is needed for assistants.
//! Note: LangGraph Cloud is part of LangSmith, so the same API key is used for both services.
//!
//! ## Usage Example
//!
//! ```no_run
//! use langstar_sdk::LangchainClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client (scoped to deployment by API key)
//!     let client = LangchainClient::new()?;
//!
//!     // List assistants in this deployment
//!     let assistants = client.assistants().list(Some(10), None).await?;
//!
//!     for assistant in assistants {
//!         println!("{}: {}", assistant.assistant_id, assistant.name);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## API Reference
//!
//! For detailed API documentation, see:
//! - [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)
//! - [Assistants API Reference](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref/)

use crate::client::LangchainClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// A LangGraph assistant (configured instance of a graph)
///
/// Assistants are deployment-level resources, automatically scoped to your API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistant {
    /// Unique identifier for the assistant
    pub assistant_id: String,
    /// Graph ID this assistant is based on
    pub graph_id: String,
    /// Name of the assistant
    pub name: String,
    /// Configuration for the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Metadata for the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// When the assistant was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// When the assistant was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Request to create a new assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequest {
    /// Graph ID to base the assistant on
    pub graph_id: String,
    /// Name for the assistant
    pub name: String,
    /// Optional configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Request to update an existing assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssistantRequest {
    /// Updated name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Updated configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Updated metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Request to search for assistants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantSearchRequest {
    /// Search query (searches assistant names). Empty string lists all assistants.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Maximum number of results (default: 20)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Number of results to skip (default: 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Client for interacting with LangGraph Assistants API
pub struct AssistantClient<'a> {
    client: &'a LangchainClient,
}

impl<'a> AssistantClient<'a> {
    /// Create a new AssistantClient
    pub fn new(client: &'a LangchainClient) -> Self {
        Self { client }
    }

    /// List all assistants
    ///
    /// # Arguments
    /// * `limit` - Maximum number of assistants to return (default: 20)
    /// * `offset` - Number of assistants to skip (default: 0)
    ///
    /// # Note
    /// This method uses the POST /assistants/search endpoint with an empty query,
    /// which is the correct way to list all assistants in the LangGraph API.
    pub async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Assistant>> {
        let request_body = AssistantSearchRequest {
            query: None, // Empty query lists all assistants
            limit,
            offset,
        };

        let path = "/assistants/search";
        let request = self.client.langgraph_post(path)?.json(&request_body);

        // LangGraph API returns a raw array of assistants
        let response: Vec<Assistant> = self.client.execute(request).await?;
        Ok(response)
    }

    /// Search for assistants by name
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results (default: 20)
    pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<Vec<Assistant>> {
        let request_body = AssistantSearchRequest {
            query: Some(query.to_string()),
            limit,
            offset: None,
        };

        let path = "/assistants/search";
        let request = self.client.langgraph_post(path)?.json(&request_body);

        // LangGraph API returns a raw array of assistants
        let response: Vec<Assistant> = self.client.execute(request).await?;
        Ok(response)
    }

    /// Get a specific assistant by ID
    ///
    /// # Arguments
    /// * `assistant_id` - The assistant ID
    pub async fn get(&self, assistant_id: &str) -> Result<Assistant> {
        let path = format!("/assistants/{}", assistant_id);
        let request = self.client.langgraph_get(&path)?;

        let assistant: Assistant = self.client.execute(request).await?;
        Ok(assistant)
    }

    /// Create a new assistant
    ///
    /// # Arguments
    /// * `request` - The create assistant request
    pub async fn create(&self, request: &CreateAssistantRequest) -> Result<Assistant> {
        let path = "/assistants";
        let req = self.client.langgraph_post(path)?.json(request);

        let assistant: Assistant = self.client.execute(req).await?;
        Ok(assistant)
    }

    /// Update an existing assistant
    ///
    /// # Arguments
    /// * `assistant_id` - The assistant ID to update
    /// * `request` - The update assistant request
    pub async fn update(
        &self,
        assistant_id: &str,
        request: &UpdateAssistantRequest,
    ) -> Result<Assistant> {
        let path = format!("/assistants/{}", assistant_id);
        let req = self.client.langgraph_patch(&path)?.json(request);

        let assistant: Assistant = self.client.execute(req).await?;
        Ok(assistant)
    }

    /// Delete an assistant
    ///
    /// # Arguments
    /// * `assistant_id` - The assistant ID to delete
    pub async fn delete(&self, assistant_id: &str) -> Result<()> {
        let path = format!("/assistants/{}", assistant_id);
        let request = self.client.langgraph_delete(&path)?;

        // DELETE typically returns 204 No Content, so we need to handle empty response
        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::error::LangstarError::ApiError {
                status: status.as_u16(),
                message: error_text,
            });
        }

        Ok(())
    }
}

impl LangchainClient {
    /// Get an AssistantClient for interacting with assistants
    pub fn assistants(&self) -> AssistantClient<'_> {
        AssistantClient::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthConfig;

    #[test]
    fn test_assistant_client_creation() {
        let auth = AuthConfig::new(None, Some("test".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let _assistant_client = client.assistants();
    }

    #[test]
    fn test_assistant_serialization() {
        let assistant = Assistant {
            assistant_id: "test-id".to_string(),
            graph_id: "graph-123".to_string(),
            name: "Test Assistant".to_string(),
            config: Some(serde_json::json!({"key": "value"})),
            metadata: None,
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            updated_at: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&assistant).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Assistant"));
    }

    #[test]
    fn test_create_request_serialization() {
        let request = CreateAssistantRequest {
            graph_id: "graph-123".to_string(),
            name: "My Assistant".to_string(),
            config: Some(serde_json::json!({"temperature": 0.7})),
            metadata: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("graph-123"));
        assert!(json.contains("My Assistant"));
    }

    #[test]
    fn test_search_request_serialization() {
        // Test with query
        let request = AssistantSearchRequest {
            query: Some("test".to_string()),
            limit: Some(10),
            offset: Some(5),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("\"limit\":10"));
        assert!(json.contains("\"offset\":5"));

        // Test without query (for list all)
        let request = AssistantSearchRequest {
            query: None,
            limit: Some(20),
            offset: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("query")); // Should be omitted when None
        assert!(json.contains("\"limit\":20"));
    }
}
