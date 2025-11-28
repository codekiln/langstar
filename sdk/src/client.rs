use crate::auth::AuthConfig;
use crate::error::{LangstarError, Result};
use crate::runs::{QueryRunsRequest, QueryRunsResponse, Run};
use futures_core::Stream;
use reqwest::{Client as HttpClient, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
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

    /// Override the LangGraph base URL for deployment-specific operations
    ///
    /// This method allows you to set a custom LangGraph deployment URL
    /// instead of using the default `https://api.langgraph.cloud`.
    /// This is useful when targeting a specific LangGraph deployment
    /// that has a custom URL (e.g., from Control Plane API's `custom_url` field).
    ///
    /// # Arguments
    /// * `url` - The custom deployment URL (e.g., "https://my-deployment.us.langgraph.app")
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{LangchainClient, AuthConfig};
    /// # let auth = AuthConfig::new(None, Some("key".into()), None, None);
    /// let client = LangchainClient::new(auth).unwrap()
    ///     .with_langgraph_url("https://my-deployment.us.langgraph.app".to_string());
    /// ```
    pub fn with_langgraph_url(mut self, url: String) -> Self {
        self.langgraph_base_url = url;
        self
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

    /// Create a POST request to Control Plane API
    ///
    /// The Control Plane API uses the same authentication as LangSmith:
    /// X-Api-Key (LangSmith API key) and X-Tenant-Id (workspace ID) headers.
    pub fn control_plane_post(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.control_plane_base_url, path);

        let mut request = self
            .http_client
            .post(&url)
            .header("X-Api-Key", api_key)
            .header("Content-Type", "application/json");

        // Add workspace ID header if set (required for Control Plane API)
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        Ok(request)
    }

    /// Create a PATCH request to Control Plane API
    ///
    /// The Control Plane API uses the same authentication as LangSmith:
    /// X-Api-Key (LangSmith API key) and X-Tenant-Id (workspace ID) headers.
    pub fn control_plane_patch(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.control_plane_base_url, path);

        let mut request = self
            .http_client
            .patch(&url)
            .header("X-Api-Key", api_key)
            .header("Content-Type", "application/json");

        // Add workspace ID header if set (required for Control Plane API)
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        Ok(request)
    }

    /// Create a DELETE request to Control Plane API
    ///
    /// The Control Plane API uses the same authentication as LangSmith:
    /// X-Api-Key (LangSmith API key) and X-Tenant-Id (workspace ID) headers.
    pub fn control_plane_delete(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.control_plane_base_url, path);

        let mut request = self
            .http_client
            .delete(&url)
            .header("X-Api-Key", api_key)
            .header("Content-Type", "application/json");

        // Add workspace ID header if set (required for Control Plane API)
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        Ok(request)
    }

    /// Create a GET request to LangGraph API
    ///
    /// ## Deployment-Level Resources
    ///
    /// **Important:** Unlike `langsmith_get()`, this method does NOT add organization
    /// or workspace scoping headers (`x-organization-id`, `X-Tenant-Id`).
    ///
    /// LangGraph assistants are deployment-level resources. The API key used in the
    /// request is tied to a specific deployment, and all operations are automatically
    /// scoped to that deployment. No additional scoping is needed or supported.
    ///
    /// ### Why No Scoping Headers?
    ///
    /// LangGraph and LangSmith have different resource models:
    /// - **LangSmith**: Hierarchical (Organization → Workspace → Prompts)
    /// - **LangGraph**: Flat (API Key → Deployment → Assistants)
    ///
    /// This is the intended design, not a limitation. Access control for LangGraph
    /// resources is managed entirely at the API key/deployment level.
    ///
    /// For more details, see the [LangGraph Cloud documentation](https://langchain-ai.github.io/langgraph/cloud/).
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
    ///
    /// ## Deployment-Level Resources
    ///
    /// **Important:** Unlike `langsmith_post()`, this method does NOT add organization
    /// or workspace scoping headers (`x-organization-id`, `X-Tenant-Id`).
    ///
    /// LangGraph assistants are deployment-level resources. The API key used in the
    /// request is tied to a specific deployment, and all operations are automatically
    /// scoped to that deployment. No additional scoping is needed or supported.
    ///
    /// ### Why No Scoping Headers?
    ///
    /// LangGraph and LangSmith have different resource models:
    /// - **LangSmith**: Hierarchical (Organization → Workspace → Prompts)
    /// - **LangGraph**: Flat (API Key → Deployment → Assistants)
    ///
    /// This is the intended design, not a limitation. Access control for LangGraph
    /// resources is managed entirely at the API key/deployment level.
    ///
    /// For more details, see the [LangGraph Cloud documentation](https://langchain-ai.github.io/langgraph/cloud/).
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

    /// Execute a request that returns no response body (status-only response).
    ///
    /// This helper is used for API endpoints that only return a status code
    /// without a response body (e.g., DELETE operations, some PATCH/PUT operations).
    ///
    /// # Arguments
    /// * `request` - The configured RequestBuilder to execute
    ///
    /// # Returns
    /// * `Ok(())` if the request succeeds (2xx status)
    /// * `Err(LangstarError::ApiError)` if the request fails
    async fn execute_status_only_request(&self, request: RequestBuilder) -> Result<()> {
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

        Ok(())
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

    // ═══════════════════════════════════════════════════════════════════════
    // Annotation Queues API Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// List annotation queues with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters including filters (name, name_contains, ids)
    ///
    /// # Returns
    ///
    /// A vector of `AnnotationQueue` objects matching the query.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, ListAnnotationQueuesParams};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let params = ListAnnotationQueuesParams {
    ///     name_contains: Some("review".to_string()),
    ///     limit: Some(50),
    ///     ..Default::default()
    /// };
    ///
    /// let queues = client.list_annotation_queues(params).await?;
    /// println!("Found {} queues", queues.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/annotation-queues`
    /// - Max limit per request: 100 (per OpenAPI spec)
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn list_annotation_queues(
        &self,
        params: crate::annotation_queues::ListAnnotationQueuesParams,
    ) -> Result<Vec<crate::annotation_queues::AnnotationQueue>> {
        let request = self.langsmith_get("/api/v1/annotation-queues")?;

        // Add query parameters
        let request = if let Some(ids) = params.ids {
            request.query(&[(
                "ids",
                ids.iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            )])
        } else {
            request
        };

        let request = if let Some(name) = params.name {
            request.query(&[("name", name)])
        } else {
            request
        };

        let request = if let Some(name_contains) = params.name_contains {
            request.query(&[("name_contains", name_contains)])
        } else {
            request
        };

        let request = if let Some(limit) = params.limit {
            request.query(&[("limit", limit)])
        } else {
            request
        };

        self.execute(request).await
    }

    /// Create a new annotation queue.
    ///
    /// # Arguments
    ///
    /// * `request` - Queue creation parameters including name, description, and configuration
    ///
    /// # Returns
    ///
    /// The created queue with full details including rubric information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, CreateAnnotationQueueRequest, QueueType};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let request = CreateAnnotationQueueRequest {
    ///     name: "Production Review".to_string(),
    ///     description: Some("Review production LLM outputs".to_string()),
    ///     queue_type: Some(QueueType::Single),
    ///     rubric_instructions: Some("Rate accuracy and helpfulness".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// let queue = client.create_annotation_queue(request).await?;
    /// println!("Created queue: {}", queue.base.name);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/annotation-queues`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn create_annotation_queue(
        &self,
        request: crate::annotation_queues::CreateAnnotationQueueRequest,
    ) -> Result<crate::annotation_queues::AnnotationQueueWithDetails> {
        let request_builder = self
            .langsmith_post("/api/v1/annotation-queues")?
            .json(&request);
        self.execute(request_builder).await
    }

    /// Get an annotation queue by ID.
    ///
    /// # Arguments
    ///
    /// * `queue_id` - The UUID of the annotation queue
    ///
    /// # Returns
    ///
    /// The queue with full details including rubric information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let queue_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let queue = client.read_annotation_queue(queue_id).await?;
    /// println!("Queue: {} ({})", queue.base.name, queue.base.id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/annotation-queues/{queue_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn read_annotation_queue(
        &self,
        queue_id: uuid::Uuid,
    ) -> Result<crate::annotation_queues::AnnotationQueueWithDetails> {
        let path = format!("/api/v1/annotation-queues/{}", queue_id);
        let request = self.langsmith_get(&path)?;
        self.execute(request).await
    }

    /// Update an annotation queue.
    ///
    /// # Arguments
    ///
    /// * `queue_id` - The UUID of the annotation queue to update
    /// * `request` - Update parameters (all fields are optional for partial updates)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, UpdateAnnotationQueueRequest};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let queue_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let request = UpdateAnnotationQueueRequest {
    ///     name: Some("Updated Queue Name".to_string()),
    ///     description: Some("New description".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// client.update_annotation_queue(queue_id, request).await?;
    /// println!("Queue updated successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `PATCH /api/v1/annotation-queues/{queue_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn update_annotation_queue(
        &self,
        queue_id: uuid::Uuid,
        request: crate::annotation_queues::UpdateAnnotationQueueRequest,
    ) -> Result<()> {
        let path = format!("/api/v1/annotation-queues/{}", queue_id);
        let request_builder = self.langsmith_put(&path)?.json(&request);
        self.execute_status_only_request(request_builder).await
    }

    /// Delete an annotation queue.
    ///
    /// # Arguments
    ///
    /// * `queue_id` - The UUID of the annotation queue to delete
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let queue_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// client.delete_annotation_queue(queue_id).await?;
    /// println!("Queue deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `DELETE /api/v1/annotation-queues/{queue_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn delete_annotation_queue(&self, queue_id: uuid::Uuid) -> Result<()> {
        let path = format!("/api/v1/annotation-queues/{}", queue_id);

        // Create DELETE request manually since we don't have a langsmith_delete helper
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .delete(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
        }

        // Add workspace ID header if set
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        self.execute_status_only_request(request).await
    }

    /// Add runs to an annotation queue.
    ///
    /// # Arguments
    ///
    /// * `queue_id` - The UUID of the annotation queue
    /// * `run_ids` - Vector of run UUIDs to add to the queue
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let queue_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let run_ids = vec![
    ///     Uuid::parse_str("abcdef01-1234-1234-1234-123456789012").unwrap(),
    ///     Uuid::parse_str("abcdef02-1234-1234-1234-123456789012").unwrap(),
    /// ];
    ///
    /// client.add_runs_to_annotation_queue(queue_id, run_ids).await?;
    /// println!("Runs added successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/annotation-queues/{queue_id}/runs`
    /// - Request body: JSON array of UUID strings
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn add_runs_to_annotation_queue(
        &self,
        queue_id: uuid::Uuid,
        run_ids: Vec<uuid::Uuid>,
    ) -> Result<()> {
        let path = format!("/api/v1/annotation-queues/{}/runs", queue_id);

        // Convert UUIDs to strings for JSON serialization
        let run_id_strings: Vec<String> = run_ids.iter().map(|id| id.to_string()).collect();

        let request_builder = self.langsmith_post(&path)?.json(&run_id_strings);
        self.execute_status_only_request(request_builder).await
    }

    /// Remove a run from an annotation queue.
    ///
    /// # Arguments
    ///
    /// * `queue_id` - The UUID of the annotation queue
    /// * `run_id` - The UUID of the run to remove
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let queue_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let run_id = Uuid::parse_str("abcdef01-1234-1234-1234-123456789012").unwrap();
    ///
    /// client.delete_run_from_annotation_queue(queue_id, run_id).await?;
    /// println!("Run removed successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `DELETE /api/v1/annotation-queues/{queue_id}/runs/{run_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn delete_run_from_annotation_queue(
        &self,
        queue_id: uuid::Uuid,
        run_id: uuid::Uuid,
    ) -> Result<()> {
        let path = format!("/api/v1/annotation-queues/{}/runs/{}", queue_id, run_id);

        // Create DELETE request manually
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .delete(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json");

        // Add organization ID header if set
        if let Some(org_id) = &self.organization_id {
            request = request.header("x-organization-id", org_id);
        }

        // Add workspace ID header if set
        if let Some(ws_id) = &self.workspace_id {
            request = request.header("X-Tenant-Id", ws_id);
        }

        self.execute_status_only_request(request).await
    }

    /// Get a run from an annotation queue at the specified index.
    ///
    /// # Arguments
    ///
    /// * `queue_id` - The UUID of the annotation queue
    /// * `index` - Zero-based index of the run in the queue
    ///
    /// # Returns
    ///
    /// A `RunWithAnnotationQueueInfo` containing the run data and queue-specific metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let queue_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let run = client.get_run_from_annotation_queue(queue_id, 0).await?;
    /// println!("Run: {} (added at: {:?})", run.run.name, run.added_at);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/annotation-queues/{queue_id}/run/{index}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn get_run_from_annotation_queue(
        &self,
        queue_id: uuid::Uuid,
        index: u32,
    ) -> Result<crate::annotation_queues::RunWithAnnotationQueueInfo> {
        let path = format!("/api/v1/annotation-queues/{}/run/{}", queue_id, index);
        let request = self.langsmith_get(&path)?;
        self.execute(request).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Runs API Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// Query runs from LangSmith with filtering and pagination.
    ///
    /// Uses `POST /api/v1/runs/query` endpoint with cursor-based pagination.
    /// Supports the LangSmith filter query language for complex filtering.
    ///
    /// # Arguments
    ///
    /// * `request` - Query parameters including filters, pagination, and field selection
    ///
    /// # Returns
    ///
    /// A `QueryRunsResponse` containing the matching runs and pagination cursors.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, QueryRunsRequest, RunType};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let request = QueryRunsRequest {
    ///     is_root: Some(true),
    ///     run_type: Some(RunType::Llm),
    ///     limit: Some(50),
    ///     ..Default::default()
    /// };
    ///
    /// let response = client.query_runs(request).await?;
    /// println!("Found {} runs", response.runs.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/runs/query`
    /// - Max limit per request: 100 (per OpenAPI spec)
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn query_runs(&self, request: QueryRunsRequest) -> Result<QueryRunsResponse> {
        let request_builder = self.langsmith_post("/api/v1/runs/query")?.json(&request);

        self.execute(request_builder).await
    }

    /// Query runs with automatic pagination, returning a stream of runs.
    ///
    /// This method handles cursor-based pagination automatically, fetching
    /// additional pages as needed until the total limit is reached or no
    /// more results are available.
    ///
    /// # Arguments
    ///
    /// * `request` - Base query parameters (cursor field will be managed automatically)
    /// * `total_limit` - Optional maximum number of runs to return across all pages.
    ///   If `None`, fetches all matching runs.
    ///
    /// # Returns
    ///
    /// A `Stream` of `Result<Run>` that yields runs one at a time.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, QueryRunsRequest, RunType};
    /// # use futures_core::Stream;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// use tokio_stream::StreamExt;
    ///
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let request = QueryRunsRequest {
    ///     is_root: Some(true),
    ///     run_type: Some(RunType::Llm),
    ///     ..Default::default()
    /// };
    ///
    /// // Fetch up to 500 runs with automatic pagination
    /// let mut stream = client.query_runs_paginated(request, Some(500));
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok(run) => println!("Run: {} ({})", run.name, run.status),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Notes
    ///
    /// - Each page fetches up to 100 runs (API maximum)
    /// - The stream continues until either:
    ///   - `total_limit` runs have been yielded
    ///   - No more pages are available (no `next` cursor)
    ///   - An error occurs
    /// - Errors are yielded as `Err` items, allowing partial results
    pub fn query_runs_paginated(
        &self,
        mut request: QueryRunsRequest,
        total_limit: Option<usize>,
    ) -> Pin<Box<dyn Stream<Item = Result<Run>> + Send + '_>> {
        let limit = total_limit.unwrap_or(usize::MAX);

        Box::pin(async_stream::try_stream! {
            let mut total_yielded = 0usize;

            loop {
                let response = self.query_runs(request.clone()).await?;

                for run in response.runs {
                    if total_yielded >= limit {
                        return;
                    }
                    total_yielded += 1;
                    yield run;
                }

                match response.cursors.next {
                    Some(next) if total_yielded < limit => {
                        request.cursor = Some(next);
                    }
                    _ => break,
                }
            }
        })
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
