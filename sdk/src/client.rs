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
    /// # let auth = AuthConfig::new(Some("key".into()), None, None);
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

    /// Create a PATCH request to LangSmith API
    ///
    /// Per LangSmith documentation, both x-organization-id and X-Tenant-Id
    /// headers can be used together for workspace-scoped requests.
    pub fn langsmith_patch(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .patch(&url)
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

    /// Create a DELETE request to LangSmith API
    ///
    /// Per LangSmith documentation, both x-organization-id and X-Tenant-Id
    /// headers can be used together for workspace-scoped requests.
    pub fn langsmith_delete(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langsmith_base_url, path);

        let mut request = self
            .http_client
            .delete(&url)
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
        let api_key = self.auth.require_langsmith_key()?;
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
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langgraph_base_url, path);

        Ok(self
            .http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json"))
    }

    /// Create a PATCH request to LangGraph API
    pub fn langgraph_patch(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
        let url = format!("{}{}", self.langgraph_base_url, path);

        Ok(self
            .http_client
            .patch(&url)
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json"))
    }

    /// Create a DELETE request to LangGraph API
    pub fn langgraph_delete(&self, path: &str) -> Result<RequestBuilder> {
        let api_key = self.auth.require_langsmith_key()?;
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
    pub async fn execute_status_only_request(&self, request: RequestBuilder) -> Result<()> {
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

        // DEBUG: Log response details before parsing (for #509 investigation)
        // Check for LANGSTAR_DEBUG_HTTP environment variable
        if std::env::var("LANGSTAR_DEBUG_HTTP").is_ok() {
            eprintln!("=== LANGSTAR HTTP DEBUG ===");
            eprintln!("Status: {}", status);

            // Log headers
            eprintln!("Headers:");
            for (name, value) in response.headers() {
                eprintln!("  {}: {:?}", name, value);
            }

            // Get response body as text first to inspect it
            // NOTE: This consumes the response, so we parse from the captured text below
            // and return early (line 546) to avoid trying to use the consumed response
            let body_text = response.text().await?;
            eprintln!("Body length: {} bytes", body_text.len());

            // Write full response to file for detailed analysis
            // Use environment variable override or platform temp dir for cross-platform support
            let debug_path = std::env::var("LANGSTAR_DEBUG_FILE")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir().join("langstar_debug_response.json"));
            if let Ok(mut file) = std::fs::File::create(&debug_path) {
                use std::io::Write;
                let _ = file.write_all(body_text.as_bytes());
                eprintln!("Full response written to: {}", debug_path.display());
            }

            eprintln!(
                "Body preview (first 500 chars): {}",
                if body_text.len() > 500 {
                    &body_text[..500]
                } else {
                    &body_text
                }
            );
            eprintln!(
                "Body preview (last 100 chars): {}",
                if body_text.len() > 100 {
                    &body_text[body_text.len() - 100..]
                } else {
                    &body_text
                }
            );
            eprintln!("===========================");

            // Parse the body we already retrieved
            let data: T = serde_json::from_str(&body_text).map_err(|e| {
                eprintln!("!!! JSON PARSE ERROR !!!");
                eprintln!("Error: {}", e);
                eprintln!("Line: {}, Column: {}", e.line(), e.column());
                if let Some(pos) = body_text.char_indices().nth(e.column().saturating_sub(1)) {
                    let context_start = pos.0.saturating_sub(50);
                    let context_end = (pos.0 + 50).min(body_text.len());
                    eprintln!(
                        "Context around error: ...{}...",
                        &body_text[context_start..context_end]
                    );
                }
                eprintln!("!!! END ERROR !!!");
                e
            })?;
            return Ok(data);
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
        let request = self.langsmith_delete(&path)?;
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
        let request = self.langsmith_delete(&path)?;
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

    // ═══════════════════════════════════════════════════════════════════════
    // Datasets API Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// Create a new dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - Dataset creation parameters including name and optional configuration
    ///
    /// # Returns
    ///
    /// The created `Dataset` with full details.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, DatasetCreate, DataType};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let request = DatasetCreate {
    ///     name: "Evaluation Dataset".to_string(),
    ///     description: Some("Dataset for model evaluation".to_string()),
    ///     data_type: Some(DataType::Chat),
    ///     ..Default::default()
    /// };
    ///
    /// let dataset = client.create_dataset(request).await?;
    /// println!("Created dataset: {} ({})", dataset.name, dataset.id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/datasets`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn create_dataset(
        &self,
        request: crate::datasets::DatasetCreate,
    ) -> Result<crate::datasets::Dataset> {
        let request_builder = self.langsmith_post("/api/v1/datasets")?.json(&request);
        self.execute(request_builder).await
    }

    /// List datasets with optional filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters including filters (name, data_type) and pagination
    ///
    /// # Returns
    ///
    /// A vector of `Dataset` objects matching the query.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, ListDatasetsParams, DataType};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let params = ListDatasetsParams {
    ///     name_contains: Some("eval".to_string()),
    ///     data_type: Some(DataType::Chat),
    ///     limit: Some(50),
    ///     ..Default::default()
    /// };
    ///
    /// let datasets = client.list_datasets(params).await?;
    /// println!("Found {} datasets", datasets.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/datasets`
    /// - Max limit per request: 100 (per OpenAPI spec)
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn list_datasets(
        &self,
        params: crate::datasets::ListDatasetsParams,
    ) -> Result<Vec<crate::datasets::Dataset>> {
        let mut request = self.langsmith_get("/api/v1/datasets")?;

        // Add query parameters
        if let Some(ids) = &params.id {
            for id in ids {
                request = request.query(&[("id", id.to_string())]);
            }
        }
        if let Some(data_type) = &params.data_type {
            request = request.query(&[(
                "data_type",
                serde_json::to_string(data_type).unwrap().trim_matches('"'),
            )]);
        }
        if let Some(name) = &params.name {
            request = request.query(&[("name", name)]);
        }
        if let Some(name_contains) = &params.name_contains {
            request = request.query(&[("name_contains", name_contains)]);
        }
        if let Some(metadata) = &params.metadata {
            request = request.query(&[("metadata", metadata)]);
        }
        if let Some(offset) = params.offset {
            request = request.query(&[("offset", offset)]);
        }
        if let Some(limit) = params.limit {
            request = request.query(&[("limit", limit)]);
        }
        if let Some(sort_by) = &params.sort_by {
            request = request.query(&[(
                "sort_by",
                serde_json::to_string(sort_by).unwrap().trim_matches('"'),
            )]);
        }
        if let Some(sort_by_desc) = params.sort_by_desc {
            request = request.query(&[("sort_by_desc", sort_by_desc)]);
        }
        if let Some(tag_value_ids) = &params.tag_value_id {
            for id in tag_value_ids {
                request = request.query(&[("tag_value_id", id.to_string())]);
            }
        }
        if let Some(exclude) = params.exclude_corrections_datasets {
            request = request.query(&[("exclude_corrections_datasets", exclude)]);
        }

        self.execute(request).await
    }

    /// Get a dataset by ID.
    ///
    /// # Arguments
    ///
    /// * `dataset_id` - The UUID of the dataset
    ///
    /// # Returns
    ///
    /// The `Dataset` with full details.
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
    /// let dataset_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let dataset = client.get_dataset(dataset_id).await?;
    /// println!("Dataset: {} ({} examples)", dataset.name, dataset.example_count);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/datasets/{dataset_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn get_dataset(&self, dataset_id: uuid::Uuid) -> Result<crate::datasets::Dataset> {
        let path = format!("/api/v1/datasets/{}", dataset_id);
        let request = self.langsmith_get(&path)?;
        self.execute(request).await
    }

    /// Update a dataset.
    ///
    /// # Arguments
    ///
    /// * `dataset_id` - The UUID of the dataset to update
    /// * `request` - Update parameters (all fields are optional for partial updates)
    ///
    /// # Returns
    ///
    /// The updated `Dataset`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, DatasetUpdate};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let dataset_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let request = DatasetUpdate {
    ///     name: Some("Updated Name".to_string()),
    ///     description: Some("New description".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// let dataset = client.update_dataset(dataset_id, request).await?;
    /// println!("Updated dataset: {}", dataset.name);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `PATCH /api/v1/datasets/{dataset_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn update_dataset(
        &self,
        dataset_id: uuid::Uuid,
        request: crate::datasets::DatasetUpdate,
    ) -> Result<crate::datasets::Dataset> {
        let path = format!("/api/v1/datasets/{}", dataset_id);
        let request_builder = self.langsmith_patch(&path)?.json(&request);
        self.execute(request_builder).await
    }

    /// Delete a dataset.
    ///
    /// # Arguments
    ///
    /// * `dataset_id` - The UUID of the dataset to delete
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
    /// let dataset_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// client.delete_dataset(dataset_id).await?;
    /// println!("Dataset deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `DELETE /api/v1/datasets/{dataset_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn delete_dataset(&self, dataset_id: uuid::Uuid) -> Result<()> {
        let path = format!("/api/v1/datasets/{}", dataset_id);
        let request = self.langsmith_delete(&path)?;
        self.execute_status_only_request(request).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Examples API Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// Create a new example in a dataset.
    ///
    /// # Arguments
    ///
    /// * `request` - Example creation parameters including dataset_id and inputs/outputs
    ///
    /// # Returns
    ///
    /// The created `Example` with full details.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, ExampleCreate};
    /// # use serde_json::json;
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let request = ExampleCreate {
    ///     dataset_id: Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap(),
    ///     inputs: Some(json!({"question": "What is 2+2?"})),
    ///     outputs: Some(json!({"answer": "4"})),
    ///     ..Default::default()
    /// };
    ///
    /// let example = client.create_example(request).await?;
    /// println!("Created example: {}", example.id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/examples`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn create_example(
        &self,
        request: crate::datasets::ExampleCreate,
    ) -> Result<crate::datasets::Example> {
        let request_builder = self.langsmith_post("/api/v1/examples")?.json(&request);
        self.execute(request_builder).await
    }

    /// List examples with optional filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `params` - Query parameters including dataset filter and pagination
    ///
    /// # Returns
    ///
    /// A vector of `Example` objects matching the query.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, ListExamplesParams};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let params = ListExamplesParams {
    ///     dataset: Some(Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap()),
    ///     limit: Some(50),
    ///     ..Default::default()
    /// };
    ///
    /// let examples = client.list_examples(params).await?;
    /// println!("Found {} examples", examples.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/examples`
    /// - Max limit per request: 100 (per OpenAPI spec)
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn list_examples(
        &self,
        params: crate::datasets::ListExamplesParams,
    ) -> Result<Vec<crate::datasets::Example>> {
        let mut request = self.langsmith_get("/api/v1/examples")?;

        // Add query parameters
        if let Some(dataset) = params.dataset {
            request = request.query(&[("dataset", dataset.to_string())]);
        }
        if let Some(ids) = &params.id {
            for id in ids {
                request = request.query(&[("id", id.to_string())]);
            }
        }
        if let Some(as_of) = &params.as_of {
            request = request.query(&[("as_of", as_of)]);
        }
        if let Some(metadata) = &params.metadata {
            request = request.query(&[("metadata", metadata)]);
        }
        if let Some(full_text) = &params.full_text_contains {
            for term in full_text {
                request = request.query(&[("full_text_contains", term)]);
            }
        }
        if let Some(splits) = &params.splits {
            for split in splits {
                request = request.query(&[("splits", split)]);
            }
        }
        if let Some(filter) = &params.filter {
            request = request.query(&[("filter", filter)]);
        }
        if let Some(offset) = params.offset {
            request = request.query(&[("offset", offset)]);
        }
        if let Some(limit) = params.limit {
            request = request.query(&[("limit", limit)]);
        }
        if let Some(order) = &params.order {
            request = request.query(&[(
                "order",
                serde_json::to_string(order).unwrap().trim_matches('"'),
            )]);
        }
        if let Some(descending) = params.descending {
            request = request.query(&[("descending", descending)]);
        }
        if let Some(select) = &params.select {
            for field in select {
                request = request.query(&[(
                    "select",
                    serde_json::to_string(field).unwrap().trim_matches('"'),
                )]);
            }
        }
        if let Some(seed) = params.random_seed {
            request = request.query(&[("random_seed", seed)]);
        }

        self.execute(request).await
    }

    /// Get an example by ID.
    ///
    /// # Arguments
    ///
    /// * `example_id` - The UUID of the example
    ///
    /// # Returns
    ///
    /// The `Example` with full details.
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/examples/{example_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn get_example(&self, example_id: uuid::Uuid) -> Result<crate::datasets::Example> {
        let path = format!("/api/v1/examples/{}", example_id);
        let request = self.langsmith_get(&path)?;
        self.execute(request).await
    }

    /// Update an example.
    ///
    /// # Arguments
    ///
    /// * `example_id` - The UUID of the example to update
    /// * `request` - Update parameters (all fields are optional for partial updates)
    ///
    /// # Returns
    ///
    /// The updated `Example`.
    ///
    /// # API Reference
    ///
    /// - Endpoint: `PATCH /api/v1/examples/{example_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn update_example(
        &self,
        example_id: uuid::Uuid,
        request: crate::datasets::ExampleUpdate,
    ) -> Result<crate::datasets::Example> {
        let path = format!("/api/v1/examples/{}", example_id);
        let request_builder = self.langsmith_patch(&path)?.json(&request);
        self.execute(request_builder).await
    }

    /// Delete an example.
    ///
    /// # Arguments
    ///
    /// * `example_id` - The UUID of the example to delete
    ///
    /// # API Reference
    ///
    /// - Endpoint: `DELETE /api/v1/examples/{example_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn delete_example(&self, example_id: uuid::Uuid) -> Result<()> {
        let path = format!("/api/v1/examples/{}", example_id);
        let request = self.langsmith_delete(&path)?;
        self.execute_status_only_request(request).await
    }

    /// Bulk create examples in a dataset.
    ///
    /// # Arguments
    ///
    /// * `examples` - Vector of example creation requests
    ///
    /// # Returns
    ///
    /// A vector of created `Example` objects.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, ExampleCreate};
    /// # use serde_json::json;
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let dataset_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let examples = vec![
    ///     ExampleCreate {
    ///         dataset_id,
    ///         inputs: Some(json!({"q": "What is 2+2?"})),
    ///         outputs: Some(json!({"a": "4"})),
    ///         ..Default::default()
    ///     },
    ///     ExampleCreate {
    ///         dataset_id,
    ///         inputs: Some(json!({"q": "What is 3+3?"})),
    ///         outputs: Some(json!({"a": "6"})),
    ///         ..Default::default()
    ///     },
    /// ];
    ///
    /// let created = client.bulk_create_examples(examples).await?;
    /// println!("Created {} examples", created.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/examples/bulk`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn bulk_create_examples(
        &self,
        examples: Vec<crate::datasets::ExampleCreate>,
    ) -> Result<Vec<crate::datasets::Example>> {
        let request_builder = self
            .langsmith_post("/api/v1/examples/bulk")?
            .json(&examples);
        self.execute(request_builder).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Feedback (Evaluation) Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// Create feedback for a run (evaluation result).
    ///
    /// Feedback represents evaluation results, either from heuristic evaluators
    /// or LLM-as-judge evaluators. This is the primary way to record evaluation
    /// scores and assessments in LangSmith.
    ///
    /// # Arguments
    ///
    /// * `request` - Feedback creation parameters including the run_id, key (metric name),
    ///   score (numeric), value (categorical), and optional configuration
    ///
    /// # Returns
    ///
    /// The created `Feedback` with full details including server-assigned ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, FeedbackCreate, FeedbackConfig, FeedbackType};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let run_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let request = FeedbackCreate {
    ///     key: "accuracy".to_string(),
    ///     run_id: Some(run_id),
    ///     score: Some(0.95),
    ///     comment: Some("Output matches expected result".to_string()),
    ///     feedback_config: Some(FeedbackConfig {
    ///         feedback_type: FeedbackType::Continuous,
    ///         min: Some(0.0),
    ///         max: Some(1.0),
    ///         categories: None,
    ///     }),
    ///     ..Default::default()
    /// };
    ///
    /// let feedback = client.create_feedback(request).await?;
    /// println!("Created feedback: {} = {}", feedback.key, feedback.score.unwrap());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/feedback`
    /// - Feedback types: `continuous` (numeric score), `categorical` (enum value), `freeform` (text)
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn create_feedback(
        &self,
        request: crate::evaluations::FeedbackCreate,
    ) -> Result<crate::evaluations::Feedback> {
        let request_builder = self.langsmith_post("/api/v1/feedback")?.json(&request);
        self.execute(request_builder).await
    }

    /// List feedback entries with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `run_id` - Optional run ID to filter feedback for a specific run
    ///
    /// # Returns
    ///
    /// A vector of `Feedback` entries matching the query.
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
    /// // List all feedback for a specific run
    /// let run_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let feedback_list = client.list_feedback(Some(run_id)).await?;
    /// println!("Found {} feedback entries", feedback_list.len());
    ///
    /// // List all feedback (no filter)
    /// let all_feedback = client.list_feedback(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/feedback`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn list_feedback(
        &self,
        run_id: Option<uuid::Uuid>,
    ) -> Result<Vec<crate::evaluations::Feedback>> {
        let mut request = self.langsmith_get("/api/v1/feedback")?;

        if let Some(id) = run_id {
            request = request.query(&[("run", id.to_string())]);
        }

        self.execute(request).await
    }

    /// Get a specific feedback entry by ID.
    ///
    /// # Arguments
    ///
    /// * `feedback_id` - The UUID of the feedback entry
    ///
    /// # Returns
    ///
    /// The `Feedback` with full details.
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
    /// let feedback_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let feedback = client.get_feedback(feedback_id).await?;
    /// println!("Feedback: {} = {:?}", feedback.key, feedback.score);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/feedback/{feedback_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn get_feedback(
        &self,
        feedback_id: uuid::Uuid,
    ) -> Result<crate::evaluations::Feedback> {
        let path = format!("/api/v1/feedback/{}", feedback_id);
        let request = self.langsmith_get(&path)?;
        self.execute(request).await
    }

    /// Update an existing feedback entry.
    ///
    /// # Arguments
    ///
    /// * `feedback_id` - The UUID of the feedback to update
    /// * `request` - Update parameters (all fields are optional for partial updates)
    ///
    /// # Returns
    ///
    /// The updated `Feedback`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, FeedbackUpdate};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let feedback_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// let request = FeedbackUpdate {
    ///     score: Some(0.98),
    ///     comment: Some("Revised: excellent match".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// let feedback = client.update_feedback(feedback_id, request).await?;
    /// println!("Updated feedback score: {:?}", feedback.score);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `PATCH /api/v1/feedback/{feedback_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn update_feedback(
        &self,
        feedback_id: uuid::Uuid,
        request: crate::evaluations::FeedbackUpdate,
    ) -> Result<crate::evaluations::Feedback> {
        let path = format!("/api/v1/feedback/{}", feedback_id);
        let request_builder = self.langsmith_patch(&path)?.json(&request);
        self.execute(request_builder).await
    }

    /// Delete a feedback entry.
    ///
    /// # Arguments
    ///
    /// * `feedback_id` - The UUID of the feedback to delete
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
    /// let feedback_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// client.delete_feedback(feedback_id).await?;
    /// println!("Feedback deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `DELETE /api/v1/feedback/{feedback_id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn delete_feedback(&self, feedback_id: uuid::Uuid) -> Result<()> {
        let path = format!("/api/v1/feedback/{}", feedback_id);
        let request = self.langsmith_delete(&path)?;
        self.execute_status_only_request(request).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Playground Settings API Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// List playground settings (model configurations).
    ///
    /// Retrieves all saved model configurations for the current workspace.
    /// These settings store model provider configurations including API keys,
    /// model parameters, and rate limits used in the Prompt Hub playground.
    ///
    /// # Arguments
    ///
    /// * `params` - Optional pagination parameters (limit, offset)
    ///
    /// # Returns
    ///
    /// A vector of `PlaygroundSettingsResponse` objects.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, ListPlaygroundSettingsParams};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// // List all playground settings
    /// let settings = client.list_playground_settings(Default::default()).await?;
    /// for setting in settings {
    ///     println!("{}: {:?}", setting.id, setting.name);
    /// }
    ///
    /// // With pagination
    /// let params = ListPlaygroundSettingsParams {
    ///     limit: Some(10),
    ///     offset: Some(0),
    /// };
    /// let settings = client.list_playground_settings(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/playground-settings`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn list_playground_settings(
        &self,
        params: crate::playground_settings::ListPlaygroundSettingsParams,
    ) -> Result<Vec<crate::playground_settings::PlaygroundSettingsResponse>> {
        let mut request = self.langsmith_get("/api/v1/playground-settings")?;

        // Add query parameters
        if let Some(limit) = params.limit {
            request = request.query(&[("limit", limit)]);
        }
        if let Some(offset) = params.offset {
            request = request.query(&[("offset", offset)]);
        }

        self.execute(request).await
    }

    /// Create new playground settings (model configuration).
    ///
    /// Creates a new saved model configuration in the current workspace.
    ///
    /// # Arguments
    ///
    /// * `request` - The playground settings to create, including model configuration
    ///
    /// # Returns
    ///
    /// The created `PlaygroundSettingsResponse` with assigned ID and timestamps.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, PlaygroundSettingsCreateRequest, PlaygroundSavedOptions};
    /// # use serde_json::json;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let request = PlaygroundSettingsCreateRequest {
    ///     name: Some("Claude 3.5 Sonnet".to_string()),
    ///     description: Some("Production Claude configuration".to_string()),
    ///     settings: json!({
    ///         "lc": 1,
    ///         "type": "constructor",
    ///         "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
    ///         "kwargs": {
    ///             "model": "claude-3-5-sonnet-20241022",
    ///             "temperature": 0.0
    ///         }
    ///     }),
    ///     options: PlaygroundSavedOptions {
    ///         requests_per_second: Some(10),
    ///     },
    /// };
    ///
    /// let created = client.create_playground_settings(request).await?;
    /// println!("Created settings with ID: {}", created.id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/playground-settings`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn create_playground_settings(
        &self,
        request: crate::playground_settings::PlaygroundSettingsCreateRequest,
    ) -> Result<crate::playground_settings::PlaygroundSettingsResponse> {
        let request_builder = self
            .langsmith_post("/api/v1/playground-settings")?
            .json(&request);
        self.execute(request_builder).await
    }

    /// Update existing playground settings (model configuration).
    ///
    /// Updates an existing saved model configuration. All fields are optional;
    /// only provided fields will be updated.
    ///
    /// # Arguments
    ///
    /// * `settings_id` - The UUID of the playground settings to update
    /// * `request` - Update parameters (all fields are optional for partial updates)
    ///
    /// # Returns
    ///
    /// The updated `PlaygroundSettingsResponse`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient, PlaygroundSettingsUpdateRequest, PlaygroundSavedOptions};
    /// # use uuid::Uuid;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let settings_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    ///
    /// // Update only specific fields
    /// let request = PlaygroundSettingsUpdateRequest {
    ///     name: Some("Updated Name".to_string()),
    ///     options: Some(PlaygroundSavedOptions {
    ///         requests_per_second: Some(20),
    ///     }),
    ///     ..Default::default()
    /// };
    ///
    /// let updated = client.update_playground_settings(settings_id, request).await?;
    /// println!("Updated settings: {:?}", updated.name);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `PATCH /api/v1/playground-settings/{id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn update_playground_settings(
        &self,
        settings_id: uuid::Uuid,
        request: crate::playground_settings::PlaygroundSettingsUpdateRequest,
    ) -> Result<crate::playground_settings::PlaygroundSettingsResponse> {
        let path = format!("/api/v1/playground-settings/{}", settings_id);
        let request_builder = self.langsmith_patch(&path)?.json(&request);
        self.execute(request_builder).await
    }

    /// Delete playground settings (model configuration).
    ///
    /// Permanently removes a saved model configuration.
    ///
    /// # Arguments
    ///
    /// * `settings_id` - The UUID of the playground settings to delete
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
    /// let settings_id = Uuid::parse_str("12345678-1234-1234-1234-123456789012").unwrap();
    /// client.delete_playground_settings(settings_id).await?;
    /// println!("Playground settings deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `DELETE /api/v1/playground-settings/{id}`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn delete_playground_settings(&self, settings_id: uuid::Uuid) -> Result<()> {
        let path = format!("/api/v1/playground-settings/{}", settings_id);
        let request = self.langsmith_delete(&path)?;
        self.execute_status_only_request(request).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Workspace Secrets API Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// List all workspace secret keys.
    ///
    /// Returns the names of all secrets configured for the current workspace.
    ///
    /// # Security
    ///
    /// Returns ONLY keys, never values. Secret values are never exposed through
    /// the API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let keys = client.list_workspace_secrets().await?;
    /// for key in keys {
    ///     println!("Secret: {}", key.key);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `GET /api/v1/workspaces/current/secrets`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn list_workspace_secrets(&self) -> Result<Vec<crate::secrets::SecretKey>> {
        let request = self.langsmith_get("/api/v1/workspaces/current/secrets")?;
        self.execute(request).await
    }

    /// Create or update workspace secrets (upsert operation).
    ///
    /// This endpoint handles both creating new secrets and updating existing ones.
    /// If a secret key already exists, its value is updated. Otherwise, a new
    /// secret is created.
    ///
    /// # Permissions
    ///
    /// Requires API key with `workspaces:manage` permission.
    ///
    /// # Security
    ///
    /// - Secret values are encrypted at rest
    /// - Values are never returned by any API endpoint
    /// - Use `SecretUpsert::set()` to create/update secrets
    /// - Use `SecretUpsert::delete()` (value: null) to delete secrets
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # use langstar_sdk::secrets::SecretUpsert;
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// let secrets = vec![
    ///     SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-..."),
    ///     SecretUpsert::set("OPENAI_API_KEY", "sk-..."),
    /// ];
    /// client.upsert_workspace_secrets(secrets).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/workspaces/current/secrets`
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn upsert_workspace_secrets(
        &self,
        secrets: Vec<crate::secrets::SecretUpsert>,
    ) -> Result<()> {
        let request_builder = self
            .langsmith_post("/api/v1/workspaces/current/secrets")?
            .json(&secrets);
        self.execute_status_only_request(request_builder).await
    }

    /// Delete a workspace secret.
    ///
    /// Convenience method that calls `upsert_workspace_secrets` with a null value
    /// to delete the specified secret.
    ///
    /// # Permissions
    ///
    /// Requires API key with `workspaces:manage` permission.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use langstar_sdk::{AuthConfig, LangchainClient};
    /// # async fn example() -> langstar_sdk::Result<()> {
    /// let auth = AuthConfig::from_env()?;
    /// let client = LangchainClient::new(auth)?;
    ///
    /// client.delete_workspace_secret("OLD_API_KEY").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # API Reference
    ///
    /// - Endpoint: `POST /api/v1/workspaces/current/secrets` (with value: null)
    /// - OpenAPI spec: <https://api.smith.langchain.com/openapi.json>
    pub async fn delete_workspace_secret(&self, key: impl Into<String>) -> Result<()> {
        let secrets = vec![crate::secrets::SecretUpsert::delete(key)];
        self.upsert_workspace_secrets(secrets).await
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
        let auth = AuthConfig::new(None, None, None);
        let client = LangchainClient::new(auth).unwrap();

        // Should fail when trying to make authenticated requests
        assert!(client.langsmith_get("/test").is_err());
        assert!(client.langgraph_get("/test").is_err());
    }

    #[test]
    fn test_client_with_org_and_workspace() {
        let auth = AuthConfig::new(
            Some("test_key".to_string()),
            Some("org_123".to_string()),
            Some("workspace_456".to_string()),
        );
        let client = LangchainClient::new(auth).unwrap();
        assert_eq!(client.organization_id(), Some("org_123"));
        assert_eq!(client.workspace_id(), Some("workspace_456"));
    }

    #[test]
    fn test_client_builder_methods() {
        let auth = AuthConfig::new(Some("test_key".to_string()), None, None);
        let client = LangchainClient::new(auth)
            .unwrap()
            .with_organization_id("new_org".to_string())
            .with_workspace_id("new_workspace".to_string());

        assert_eq!(client.organization_id(), Some("new_org"));
        assert_eq!(client.workspace_id(), Some("new_workspace"));
    }
}
