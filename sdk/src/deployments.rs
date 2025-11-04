use crate::client::LangchainClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Source type for LangGraph deployment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentSource {
    /// GitHub repository source
    Github,
    /// External Docker image source
    ExternalDocker,
}

/// Current status of a deployment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
    pub secrets: Option<Vec<String>>,
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
}
