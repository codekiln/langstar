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
//! use langstar_sdk::{LangchainClient, AuthConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create auth config and client (scoped to deployment by API key)
//!     let auth = AuthConfig::new(Some("your-api-key".into()), None, None);
//!     let client = LangchainClient::new(auth)?;
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
    /// Name of the assistant, or an empty string when the API leaves it out.
    /// The Agent Server API doesn't require a name.
    #[serde(default)]
    pub name: String,
    /// Description of the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Configuration for the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Static context for the assistant, the field LangGraph 0.6 added beside
    /// `config.configurable`. The Agent Server fills one from the other, so both
    /// usually hold the same settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Metadata for the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Which version of the assistant this is. The Agent Server saves a new
    /// version on every update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    /// When the assistant was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// When the assistant was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// What the Agent Server does when a create names an assistant that already exists
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IfExists {
    /// Answer with an error, which is what the Agent Server does by default
    Raise,
    /// Answer with the assistant that is already there
    DoNothing,
}

/// Request to create a new assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssistantRequest {
    /// The ID to give the new assistant. The Agent Server picks a random one when
    /// this is `None`, so a caller that wants to send the same create twice and get
    /// one assistant chooses the ID here and sets `if_exists` to
    /// [`IfExists::DoNothing`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_id: Option<String>,
    /// Graph ID to base the assistant on
    pub graph_id: String,
    /// Name for the assistant
    pub name: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Static context for the assistant. A newer Agent Server refuses a create that
    /// sends both this and `config.configurable`, so send one of them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// What to do when `assistant_id` names an assistant that is already there.
    /// The Agent Server raises an error when this is `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_exists: Option<IfExists>,
}

/// Request to update an existing assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssistantRequest {
    /// Updated name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The graph the assistant should run from now on. The assistant keeps its
    /// current graph when this is `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<String>,
    /// Updated description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Updated configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Updated static context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    /// Updated metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Request to search for assistants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantSearchRequest {
    /// Keep only assistants whose name contains this text, ignoring case.
    /// `None` lists all assistants.
    ///
    /// Sent as `name`, the field `POST /assistants/search` filters on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Keep only assistants that run this graph. `None` lists every graph's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<String>,
    /// Maximum number of results. When `None`, the API returns up to 10.
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
    /// * `limit` - Maximum number of assistants to return. When `None`, the API returns up to 10.
    /// * `offset` - Number of assistants to skip (default: 0)
    ///
    /// # Note
    /// This method uses the POST /assistants/search endpoint with no name filter,
    /// which is the correct way to list all assistants in the LangGraph API.
    pub async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Assistant>> {
        let request_body = AssistantSearchRequest {
            name: None, // No name filter lists all assistants
            graph_id: None,
            limit,
            offset,
        };

        let path = "/assistants/search";
        let request = self.client.langgraph_post(path)?.json(&request_body);

        // LangGraph API returns a raw array of assistants
        let response: Vec<Assistant> = self.client.execute_read(request).await?;
        Ok(response)
    }

    /// Search for assistants whose name contains `query`, ignoring case
    ///
    /// # Arguments
    /// * `query` - Text to look for in assistant names
    /// * `limit` - Maximum number of results. When `None`, the API returns up to 10.
    pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<Vec<Assistant>> {
        let request_body = AssistantSearchRequest {
            name: Some(query.to_string()),
            graph_id: None,
            limit,
            offset: None,
        };

        let path = "/assistants/search";
        let request = self.client.langgraph_post(path)?.json(&request_body);

        // LangGraph API returns a raw array of assistants
        let response: Vec<Assistant> = self.client.execute_read(request).await?;
        Ok(response)
    }

    /// List the assistants that run one graph
    ///
    /// # Arguments
    /// * `graph_id` - The graph ID, as `langgraph.json` registers it
    /// * `limit` - Maximum number of results. When `None`, the API returns up to 10.
    pub async fn search_by_graph_id(
        &self,
        graph_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Assistant>> {
        let request_body = AssistantSearchRequest {
            name: None,
            graph_id: Some(graph_id.to_string()),
            limit,
            offset: None,
        };

        let path = "/assistants/search";
        let request = self.client.langgraph_post(path)?.json(&request_body);

        let response: Vec<Assistant> = self.client.execute_read(request).await?;
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

        // DELETE typically returns 204 No Content, so there is no body to parse
        self.client.execute_status_only_request(request).await
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
        let auth = AuthConfig::new(Some("test".to_string()), None, None);
        let client = LangchainClient::new(auth).unwrap();
        let _assistant_client = client.assistants();
    }

    #[test]
    fn test_assistant_serialization() {
        let assistant = Assistant {
            assistant_id: "test-id".to_string(),
            graph_id: "graph-123".to_string(),
            name: "Test Assistant".to_string(),
            description: Some("Test description".to_string()),
            config: Some(serde_json::json!({"key": "value"})),
            context: None,
            metadata: None,
            version: Some(1),
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
            assistant_id: None,
            graph_id: "graph-123".to_string(),
            name: "My Assistant".to_string(),
            description: Some("My description".to_string()),
            config: Some(serde_json::json!({"temperature": 0.7})),
            context: None,
            metadata: None,
            if_exists: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("graph-123"));
        assert!(json.contains("My Assistant"));
    }

    #[test]
    fn test_assistant_without_name_deserializes() {
        let assistant: Assistant = serde_json::from_value(serde_json::json!({
            "assistant_id": "a1",
            "graph_id": "agent",
            "config": {},
            "metadata": {},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }))
        .expect("an assistant without a name should still deserialize");
        assert_eq!(assistant.name, "");
    }

    #[test]
    fn test_search_request_serialization() {
        // The API filters on `name` and ignores `query`.
        let request = AssistantSearchRequest {
            name: Some("test".to_string()),
            graph_id: None,
            limit: Some(10),
            offset: Some(5),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(!json.contains("query"));
        assert!(json.contains("\"limit\":10"));
        assert!(json.contains("\"offset\":5"));

        // Without a name filter (for list all)
        let request = AssistantSearchRequest {
            name: None,
            graph_id: None,
            limit: Some(20),
            offset: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("name")); // Should be omitted when None
        assert!(json.contains("\"limit\":20"));
    }

    #[tokio::test]
    async fn test_list_and_search_retry_a_502() {
        let mut server = mockito::Server::new_async().await;
        let bad = server
            .mock("POST", "/assistants/search")
            .with_status(502)
            .expect(2)
            .create_async()
            .await;
        let good = server
            .mock("POST", "/assistants/search")
            .with_status(200)
            .with_body("[]")
            .expect(2)
            .create_async()
            .await;
        let client = LangchainClient::with_base_urls(
            AuthConfig::new(Some("test".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap()
        .with_retry_policy(crate::client::RetryPolicy {
            max_retries: 2,
            base_delay: std::time::Duration::from_millis(1),
            max_delay: std::time::Duration::from_millis(5),
        });

        assert!(
            client
                .assistants()
                .list(None, None)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            client
                .assistants()
                .search("x", None)
                .await
                .unwrap()
                .is_empty()
        );
        bad.assert_async().await;
        good.assert_async().await;
    }
}

#[cfg(test)]
mod export_field_tests {
    use super::*;
    use crate::auth::AuthConfig;
    use serde_json::json;

    fn mock_client(server: &mockito::ServerGuard) -> LangchainClient {
        LangchainClient::with_base_urls(
            AuthConfig::new(Some("test_key".to_string()), None, None),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap()
    }

    fn assistant_body() -> serde_json::Value {
        json!({
            "assistant_id": "11111111-2222-3333-4444-555555555555",
            "graph_id": "test_graph",
            "name": "support",
            "config": {"configurable": {"tone": "warm"}},
            "context": {"tone": "warm"},
            "metadata": {"purpose": "export"},
            "version": 3,
            "created_at": "2026-10-30T00:00:00Z",
            "updated_at": "2026-10-30T00:00:00Z"
        })
    }

    #[tokio::test]
    async fn test_create_sends_the_chosen_id_the_context_and_if_exists() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/assistants")
            .match_body(mockito::Matcher::Json(json!({
                "assistant_id": "11111111-2222-3333-4444-555555555555",
                "graph_id": "test_graph",
                "name": "support",
                "context": {"tone": "warm"},
                "if_exists": "do_nothing"
            })))
            .with_body(assistant_body().to_string())
            .create_async()
            .await;

        let assistant = mock_client(&server)
            .assistants()
            .create(&CreateAssistantRequest {
                assistant_id: Some("11111111-2222-3333-4444-555555555555".to_string()),
                graph_id: "test_graph".to_string(),
                name: "support".to_string(),
                description: None,
                config: None,
                context: Some(json!({"tone": "warm"})),
                metadata: None,
                if_exists: Some(IfExists::DoNothing),
            })
            .await
            .expect("the create should succeed");

        mock.assert_async().await;
        assert_eq!(assistant.context, Some(json!({"tone": "warm"})));
        assert_eq!(assistant.version, Some(3));
    }

    #[tokio::test]
    async fn test_update_sends_the_new_graph_id_and_context() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("PATCH", "/assistants/11111111-2222-3333-4444-555555555555")
            .match_body(mockito::Matcher::Json(json!({
                "graph_id": "test_graph_alt",
                "context": {"tone": "brisk"}
            })))
            .with_body(
                json!({
                    "assistant_id": "11111111-2222-3333-4444-555555555555",
                    "graph_id": "test_graph_alt",
                    "name": "support",
                    "config": {"configurable": {"tone": "brisk"}},
                    "context": {"tone": "brisk"},
                    "metadata": {},
                    "version": 4,
                    "created_at": "2026-10-30T00:00:00Z",
                    "updated_at": "2026-10-30T00:05:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let assistant = mock_client(&server)
            .assistants()
            .update(
                "11111111-2222-3333-4444-555555555555",
                &UpdateAssistantRequest {
                    name: None,
                    graph_id: Some("test_graph_alt".to_string()),
                    description: None,
                    config: None,
                    context: Some(json!({"tone": "brisk"})),
                    metadata: None,
                },
            )
            .await
            .expect("the update should succeed");

        mock.assert_async().await;
        assert_eq!(assistant.graph_id, "test_graph_alt");
        assert_eq!(assistant.version, Some(4));
    }

    #[tokio::test]
    async fn test_search_by_graph_id_filters_on_the_graph() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/assistants/search")
            .match_body(mockito::Matcher::Json(json!({
                "graph_id": "test_graph",
                "limit": 5
            })))
            .with_body(json!([assistant_body()]).to_string())
            .create_async()
            .await;

        let assistants = mock_client(&server)
            .assistants()
            .search_by_graph_id("test_graph", Some(5))
            .await
            .expect("the search should succeed");

        mock.assert_async().await;
        assert_eq!(assistants.len(), 1);
        assert_eq!(assistants[0].graph_id, "test_graph");
    }

    #[tokio::test]
    async fn test_an_assistant_without_context_or_version_still_deserializes() {
        let assistant: Assistant = serde_json::from_value(json!({
            "assistant_id": "a1",
            "graph_id": "agent",
            "config": {},
            "metadata": {},
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }))
        .expect("an assistant from an older server should still deserialize");
        assert!(assistant.context.is_none());
        assert!(assistant.version.is_none());
    }
}
