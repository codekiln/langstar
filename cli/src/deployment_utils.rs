//! Shared utilities for deployment resolution

use crate::config::Config;
use crate::error::{CliError, Result};
use langstar_sdk::{AuthConfig, LangchainClient};

/// Resolve deployment name or ID to deployment URL
///
/// This function queries the Control Plane API to find a deployment matching
/// the provided name or UUID, then extracts its custom_url for Agent Server API calls.
///
/// # Arguments
/// * `config` - Reference to the CLI configuration containing API keys and workspace ID.
/// * `deployment_name_or_id` - The deployment name or UUID to resolve.
///
/// # Returns
/// * `Result<String>` - The custom URL of the resolved deployment if found.
///
/// # Errors
/// Returns a `CliError::Config` if:
/// - The deployment is not found.
/// - The deployment does not have a `custom_url` in its `source_config`.
pub async fn resolve_deployment_url(
    config: &Config,
    deployment_name_or_id: &str,
) -> Result<String> {
    // Create Control Plane client for deployment lookup
    let auth = AuthConfig::new(
        config.langsmith_api_key.clone(),
        None,
        None,
        config.workspace_id.clone(),
    );
    let client = LangchainClient::new(auth)?;

    // List deployments (limit 100 to catch most cases)
    let deployments_list = client.deployments().list(Some(100), Some(0), None).await?;

    // Find deployment by name or ID
    let deployment = deployments_list
        .resources
        .iter()
        .find(|d| d.name == deployment_name_or_id || d.id == deployment_name_or_id)
        .ok_or_else(|| {
            CliError::Config(format!(
                "Deployment '{}' not found. Run 'langstar deployment list' to see available deployments.",
                deployment_name_or_id
            ))
        })?;

    // Extract custom_url
    deployment.custom_url().ok_or_else(|| {
        CliError::Config(format!(
            "Deployment '{}' has no custom_url in source_config",
            deployment.name
        ))
    })
}
