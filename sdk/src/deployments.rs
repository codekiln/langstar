use crate::client::LangchainClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A secret environment variable for a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSecret {
    /// Name of the secret environment variable
    pub name: String,
    /// Value of the secret (will be redacted in responses)
    pub value: String,
}

/// Source type for LangGraph deployment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentSource {
    /// GitHub repository source
    Github,
    /// External Docker image source
    ExternalDocker,
    /// Unknown source type (for forward compatibility)
    #[serde(other)]
    #[default]
    Unknown,
}

/// Current status of a deployment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeploymentStatus {
    /// Deployment is awaiting database provisioning
    AwaitingDatabase,
    /// Deployment is ready and operational
    Ready,
    /// Deployment is not currently in use
    Unused,
    /// Deployment is awaiting deletion
    AwaitingDelete,
    /// Deployment status is unknown
    #[default]
    Unknown,
}

/// Type of deployment environment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentType {
    /// Free development deployment
    DevFree,
    /// Paid development deployment
    Dev,
    /// Production deployment with HA and autoscaling
    Prod,
}

/// A LangGraph deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Deployment {
    /// Unique identifier for the deployment
    pub id: String,
    /// User-assigned name for the deployment
    pub name: String,
    /// Source type (github or external_docker)
    pub source: DeploymentSource,
    /// Source configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_config: Option<serde_json::Value>,
    /// Source revision configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision_config: Option<serde_json::Value>,
    /// Environment variable secrets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<DeploymentSecret>>,
    /// When the deployment was created
    pub created_at: String,
    /// When the deployment was last updated
    pub updated_at: String,
    /// Current deployment status
    pub status: DeploymentStatus,
    /// ID of the latest revision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_revision_id: Option<String>,
    /// ID of the active (deployed) revision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_revision_id: Option<String>,
    /// Optional version specification for the image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_version: Option<String>,
}

impl Default for Deployment {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            source: DeploymentSource::Unknown,
            source_config: None,
            source_revision_config: None,
            secrets: None,
            created_at: String::new(),
            updated_at: String::new(),
            status: DeploymentStatus::Unknown,
            latest_revision_id: None,
            active_revision_id: None,
            image_version: None,
        }
    }
}

impl Deployment {
    /// Extract the custom URL from the deployment's source_config
    ///
    /// The Control Plane API returns deployments with a `custom_url` field
    /// nested inside the `source_config` JSON object. This method extracts
    /// that URL for use in assistant API calls.
    ///
    /// # Returns
    /// * `Some(String)` - The custom URL if present in source_config
    /// * `None` - If source_config is missing or doesn't contain custom_url
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::deployments::Deployment;
    /// # let deployment = Deployment::default();
    /// if let Some(url) = deployment.custom_url() {
    ///     println!("Deployment URL: {}", url);
    /// }
    /// ```
    pub fn custom_url(&self) -> Option<String> {
        self.source_config
            .as_ref()
            .and_then(|v| v.get("custom_url"))
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}

/// Response from listing deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentsList {
    /// List of deployments
    pub resources: Vec<Deployment>,
    /// Offset for pagination
    pub offset: i32,
}

/// Filters for querying deployments
#[derive(Debug, Clone, Default, Serialize)]
pub struct DeploymentFilters {
    /// Filter by name (substring match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,
    /// Filter by deployment status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DeploymentStatus>,
    /// Filter by deployment type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_type: Option<DeploymentType>,
    /// Filter by image version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_version: Option<String>,
}

/// Request to create a new deployment
#[derive(Debug, Clone, Serialize)]
pub struct CreateDeploymentRequest {
    /// Name of the deployment
    pub name: String,
    /// Source type (e.g., "github", "external_docker")
    pub source: String,
    /// Source configuration (repository URL, branch, etc.)
    pub source_config: serde_json::Value,
    /// Source revision configuration (commit hash, tag, etc.)
    pub source_revision_config: serde_json::Value,
    /// Deployment type (dev_free, dev, or prod)
    pub deployment_type: String,
    /// Environment variable secrets
    pub secrets: Vec<DeploymentSecret>,
    /// Optional environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_vars: Option<HashMap<String, String>>,
}

impl CreateDeploymentRequest {
    /// Create a new deployment request
    ///
    /// # Arguments
    /// * `name` - Name for the deployment
    /// * `source` - Source type (e.g., "github")
    /// * `source_config` - Configuration for the source (repo URL, branch, etc.)
    /// * `deployment_type` - Type of deployment (dev_free, dev, or prod)
    pub fn new(
        name: String,
        source: String,
        source_config: serde_json::Value,
        deployment_type: String,
    ) -> Self {
        use serde_json::json;
        Self {
            name,
            source,
            source_config,
            source_revision_config: json!({}), // Empty object for default/auto revision
            deployment_type,
            secrets: Vec::new(), // Empty secrets list by default
            env_vars: None,
        }
    }

    /// Add environment variables to the deployment
    pub fn with_env_vars(mut self, env_vars: HashMap<String, String>) -> Self {
        self.env_vars = Some(env_vars);
        self
    }

    /// Add secrets to the deployment
    pub fn with_secrets(mut self, secrets: Vec<DeploymentSecret>) -> Self {
        self.secrets = secrets;
        self
    }

    /// Set the source revision configuration
    pub fn with_source_revision_config(
        mut self,
        source_revision_config: serde_json::Value,
    ) -> Self {
        self.source_revision_config = source_revision_config;
        self
    }
}

/// Client for interacting with LangGraph Control Plane Deployments API
pub struct DeploymentClient<'a> {
    client: &'a LangchainClient,
}

impl<'a> DeploymentClient<'a> {
    /// Create a new DeploymentClient
    pub fn new(client: &'a LangchainClient) -> Self {
        Self { client }
    }

    /// List all deployments
    ///
    /// # Arguments
    /// * `limit` - Maximum number of deployments to return (default: 20, max: 100)
    /// * `offset` - Number of deployments to skip (default: 0)
    /// * `filters` - Optional filters to apply
    pub async fn list(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        filters: Option<DeploymentFilters>,
    ) -> Result<DeploymentsList> {
        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);

        let mut query_params = vec![format!("limit={}", limit), format!("offset={}", offset)];

        // Add optional filters
        if let Some(filters) = filters {
            if let Some(name) = filters.name_contains {
                query_params.push(format!("name_contains={}", urlencoding::encode(&name)));
            }
            if let Some(status) = filters.status {
                let status_str = match status {
                    DeploymentStatus::AwaitingDatabase => "AWAITING_DATABASE",
                    DeploymentStatus::Ready => "READY",
                    DeploymentStatus::Unused => "UNUSED",
                    DeploymentStatus::AwaitingDelete => "AWAITING_DELETE",
                    DeploymentStatus::Unknown => "UNKNOWN",
                };
                query_params.push(format!("status={}", status_str));
            }
            if let Some(deployment_type) = filters.deployment_type {
                let type_str = match deployment_type {
                    DeploymentType::DevFree => "dev_free",
                    DeploymentType::Dev => "dev",
                    DeploymentType::Prod => "prod",
                };
                query_params.push(format!("deployment_type={}", type_str));
            }
            if let Some(version) = filters.image_version {
                query_params.push(format!("image_version={}", urlencoding::encode(&version)));
            }
        }

        let path = format!("/v2/deployments?{}", query_params.join("&"));
        let request = self.client.control_plane_get(&path)?;
        let response: DeploymentsList = self.client.execute(request).await?;
        Ok(response)
    }

    /// Get a single deployment by ID
    ///
    /// # Arguments
    /// * `deployment_id` - UUID of the deployment to retrieve
    pub async fn get(&self, deployment_id: &str) -> Result<Deployment> {
        let path = format!("/v2/deployments/{}", deployment_id);
        let request = self.client.control_plane_get(&path)?;
        let response: Deployment = self.client.execute(request).await?;
        Ok(response)
    }

    /// Create a new deployment
    ///
    /// # Arguments
    /// * `request` - The deployment creation request with name, source, config, and type
    ///
    /// # Returns
    /// The created `Deployment` object with ID and status
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{LangchainClient, AuthConfig};
    /// # use langstar_sdk::deployments::CreateDeploymentRequest;
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = AuthConfig::new(None, Some("key".into()), None, Some("ws_id".into()));
    /// # let client = LangchainClient::new(auth)?;
    /// let source_config = json!({
    ///     "repo_url": "https://github.com/owner/repo",
    ///     "branch": "main"
    /// });
    ///
    /// let request = CreateDeploymentRequest::new(
    ///     "my-deployment".to_string(),
    ///     "github".to_string(),
    ///     source_config,
    ///     "dev_free".to_string(),
    /// );
    ///
    /// let deployment = client.deployments().create(request).await?;
    /// println!("Created deployment: {}", deployment.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, request: CreateDeploymentRequest) -> Result<Deployment> {
        let path = "/v2/deployments";
        let http_request = self.client.control_plane_post(path)?.json(&request);
        let response: Deployment = self.client.execute(http_request).await?;
        Ok(response)
    }

    /// Delete a deployment by ID
    ///
    /// # Arguments
    /// * `deployment_id` - UUID of the deployment to delete
    ///
    /// # Returns
    /// `Ok(())` if the deployment was successfully deleted
    ///
    /// # Errors
    /// Returns an error if:
    /// - The deployment does not exist (404)
    /// - The user lacks permission to delete the deployment
    /// - The API request fails
    ///
    /// # Example
    /// ```no_run
    /// # use langstar_sdk::{LangchainClient, AuthConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let auth = AuthConfig::new(None, Some("key".into()), None, Some("ws_id".into()));
    /// # let client = LangchainClient::new(auth)?;
    /// client.deployments().delete("abc-123e4567-e89b-12d3").await?;
    /// println!("Deployment deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, deployment_id: &str) -> Result<()> {
        let path = format!("/v2/deployments/{}", deployment_id);
        let request = self.client.control_plane_delete(&path)?;

        // Execute request and ignore response body (DELETE typically returns empty or status)
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
    /// Get a DeploymentClient for interacting with LangGraph deployments
    pub fn deployments(&self) -> DeploymentClient<'_> {
        DeploymentClient::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_source_serialization() {
        let github = DeploymentSource::Github;
        let json = serde_json::to_string(&github).unwrap();
        assert_eq!(json, "\"github\"");

        let docker = DeploymentSource::ExternalDocker;
        let json = serde_json::to_string(&docker).unwrap();
        assert_eq!(json, "\"external_docker\"");
    }

    #[test]
    fn test_deployment_status_serialization() {
        let ready = DeploymentStatus::Ready;
        let json = serde_json::to_string(&ready).unwrap();
        assert_eq!(json, "\"READY\"");

        let awaiting = DeploymentStatus::AwaitingDatabase;
        let json = serde_json::to_string(&awaiting).unwrap();
        assert_eq!(json, "\"AWAITING_DATABASE\"");
    }

    #[test]
    fn test_deployment_type_serialization() {
        let dev_free = DeploymentType::DevFree;
        let json = serde_json::to_string(&dev_free).unwrap();
        assert_eq!(json, "\"dev_free\"");

        let prod = DeploymentType::Prod;
        let json = serde_json::to_string(&prod).unwrap();
        assert_eq!(json, "\"prod\"");
    }

    #[test]
    fn test_deployment_deserialization() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "my-deployment",
            "source": "github",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "status": "READY"
        }"#;

        let deployment: Deployment = serde_json::from_str(json).unwrap();
        assert_eq!(deployment.name, "my-deployment");
        assert_eq!(deployment.source, DeploymentSource::Github);
        assert_eq!(deployment.status, DeploymentStatus::Ready);
    }

    #[test]
    fn test_deployments_list_deserialization() {
        let json = r#"{
            "resources": [
                {
                    "id": "123e4567-e89b-12d3-a456-426614174000",
                    "name": "deployment-1",
                    "source": "github",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-02T00:00:00Z",
                    "status": "READY"
                }
            ],
            "offset": 0
        }"#;

        let list: DeploymentsList = serde_json::from_str(json).unwrap();
        assert_eq!(list.resources.len(), 1);
        assert_eq!(list.offset, 0);
        assert_eq!(list.resources[0].name, "deployment-1");
    }

    #[test]
    fn test_deployment_custom_url_extraction() {
        // Test with custom_url present
        let json_with_url = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "my-deployment",
            "source": "github",
            "source_config": {
                "custom_url": "https://my-deployment.us.langgraph.app",
                "integration_id": "d23cce11-20c1-424c-b2b2-4322c4ff4d90",
                "deployment_type": "dev"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "status": "READY"
        }"#;

        let deployment: Deployment = serde_json::from_str(json_with_url).unwrap();
        let url = deployment.custom_url();
        assert_eq!(
            url,
            Some("https://my-deployment.us.langgraph.app".to_string())
        );

        // Test without source_config
        let json_without_config = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "my-deployment",
            "source": "github",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "status": "READY"
        }"#;

        let deployment: Deployment = serde_json::from_str(json_without_config).unwrap();
        assert_eq!(deployment.custom_url(), None);

        // Test with source_config but no custom_url
        let json_without_url = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "my-deployment",
            "source": "github",
            "source_config": {
                "integration_id": "d23cce11-20c1-424c-b2b2-4322c4ff4d90"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-02T00:00:00Z",
            "status": "READY"
        }"#;

        let deployment: Deployment = serde_json::from_str(json_without_url).unwrap();
        assert_eq!(deployment.custom_url(), None);
    }

    #[test]
    fn test_create_deployment_request_serialization() {
        use serde_json::json;

        let source_config = json!({
            "repo_url": "https://github.com/owner/repo",
            "branch": "main"
        });

        let request = CreateDeploymentRequest::new(
            "test-deployment".to_string(),
            "github".to_string(),
            source_config,
            "dev_free".to_string(),
        );

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["name"], "test-deployment");
        assert_eq!(json["source"], "github");
        assert_eq!(json["deployment_type"], "dev_free");
        assert_eq!(
            json["source_config"]["repo_url"],
            "https://github.com/owner/repo"
        );
        assert_eq!(json["source_config"]["branch"], "main");
        assert!(json["env_vars"].is_null()); // Should be omitted when None
    }

    #[test]
    fn test_create_deployment_request_with_env_vars() {
        use serde_json::json;

        let source_config = json!({
            "repo_url": "https://github.com/owner/repo",
            "branch": "main"
        });

        let mut env_vars = HashMap::new();
        env_vars.insert("API_KEY".to_string(), "secret123".to_string());
        env_vars.insert("DEBUG".to_string(), "true".to_string());

        let request = CreateDeploymentRequest::new(
            "test-deployment".to_string(),
            "github".to_string(),
            source_config,
            "dev_free".to_string(),
        )
        .with_env_vars(env_vars);

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["name"], "test-deployment");
        assert!(json["env_vars"].is_object());
        assert_eq!(json["env_vars"]["API_KEY"], "secret123");
        assert_eq!(json["env_vars"]["DEBUG"], "true");
    }

    #[test]
    fn test_create_deployment_request_builder_pattern() {
        use serde_json::json;

        let source_config = json!({
            "repo_url": "https://github.com/owner/repo",
            "branch": "main"
        });

        // Test builder pattern
        let request = CreateDeploymentRequest::new(
            "test-deployment".to_string(),
            "github".to_string(),
            source_config,
            "prod".to_string(),
        );

        assert_eq!(request.name, "test-deployment");
        assert_eq!(request.source, "github");
        assert_eq!(request.deployment_type, "prod");
        assert!(request.env_vars.is_none());

        // Add env vars using builder
        let mut env_vars = HashMap::new();
        env_vars.insert("KEY".to_string(), "value".to_string());

        let request_with_env = request.with_env_vars(env_vars);
        assert!(request_with_env.env_vars.is_some());
    }
}
