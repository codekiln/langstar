//! Shared utilities for deployment resolution

use crate::config::Config;
use crate::error::{CliError, Result};
use langstar_sdk::organization::{CHOOSE_WORKSPACE_HELP, describe_workspaces};
use langstar_sdk::{AuthConfig, Deployment, LangchainClient, LangstarError, Workspace};

/// A deployment found on the Control Plane API
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDeployment {
    /// The deployment's Agent Server URL (`source_config.custom_url`)
    pub url: String,
    /// The workspace that holds the deployment, found by searching each workspace the
    /// key reaches. Set only when no workspace ID was given and the deployment lookup
    /// needed one; `None` otherwise.
    pub workspace_id: Option<String>,
}

impl ResolvedDeployment {
    /// A client for the deployment's Agent Server, with the workspace the lookup found
    /// when none was chosen.
    pub fn client(&self, config: &Config) -> Result<LangchainClient> {
        let mut client =
            LangchainClient::new(config.to_auth_config())?.with_langgraph_url(self.url.clone());
        if let (None, Some(workspace_id)) = (&config.workspace_id, &self.workspace_id) {
            client = client.with_workspace_id(workspace_id.clone());
        }
        Ok(client)
    }
}

/// Resolve a deployment name or ID to its Agent Server URL and workspace
///
/// This function queries the Control Plane API to find a deployment matching
/// the provided name or UUID, then extracts its custom_url for Agent Server API calls.
///
/// When no workspace was chosen and the API key reaches several, the Control Plane
/// answers 403. The lookup then searches each workspace the key reaches. When exactly
/// one holds the deployment, it uses that workspace and says so on stderr.
///
/// # Errors
/// Returns a `CliError::Config` if:
/// - The deployment is not found.
/// - Several workspaces hold a deployment with that name or ID.
/// - The deployment does not have a `custom_url` in its `source_config`.
pub async fn resolve_deployment(
    config: &Config,
    deployment_name_or_id: &str,
) -> Result<ResolvedDeployment> {
    // Create Control Plane client for deployment lookup
    let auth = AuthConfig::new(
        config.langsmith_api_key.clone(),
        None,
        config.workspace_id.clone(),
    );
    let client = LangchainClient::new(auth)?;
    resolve_deployment_with(&client, deployment_name_or_id).await
}

/// [`resolve_deployment`] with the Control Plane client passed in, so tests can point it
/// at a mock server
async fn resolve_deployment_with(
    client: &LangchainClient,
    deployment_name_or_id: &str,
) -> Result<ResolvedDeployment> {
    let workspace_chosen = client.workspace_id().is_some();
    let (deployment, workspace_id) = match find_deployment(client, deployment_name_or_id).await {
        Ok(deployment) => {
            let deployment = deployment.ok_or_else(|| {
                CliError::Config(format!(
                    "Deployment '{}' not found. Run 'langstar deployment list' to see available deployments.",
                    deployment_name_or_id
                ))
            })?;
            // Set when the key reaches one workspace and the client picked it after a 403.
            let workspace_id = if workspace_chosen {
                None
            } else {
                client.resolved_workspace_id().map(str::to_string)
            };
            (deployment, workspace_id)
        }
        Err(LangstarError::WorkspaceRequired { reachable }) => {
            let (deployment, workspace) =
                find_deployment_in_workspaces(client, deployment_name_or_id, &reachable).await?;
            eprintln!(
                "Using workspace '{}' ({}), which holds deployment '{}'. Pass --workspace-id to choose a workspace yourself.",
                workspace_name(&workspace),
                workspace.id,
                deployment.name
            );
            let workspace_id = deployment
                .tenant_id
                .clone()
                .unwrap_or_else(|| workspace.id.clone());
            (deployment, Some(workspace_id))
        }
        Err(e) => return Err(e.into()),
    };

    // Extract custom_url
    let url = deployment.custom_url().ok_or_else(|| {
        CliError::Config(format!(
            "Deployment '{}' has no custom_url in source_config",
            deployment.name
        ))
    })?;
    Ok(ResolvedDeployment { url, workspace_id })
}

/// List every deployment in the client's workspace and return the one with this name or ID
async fn find_deployment(
    client: &LangchainClient,
    deployment_name_or_id: &str,
) -> langstar_sdk::Result<Option<Deployment>> {
    // List all deployments with pagination
    let page_size = 100;
    let mut offset = 0;
    let mut all_deployments = Vec::new();

    loop {
        let deployments_list = client
            .deployments()
            .list(Some(page_size), Some(offset), None)
            .await?;
        let count = deployments_list.resources.len();
        all_deployments.extend(deployments_list.resources);

        if count < page_size as usize {
            break;
        }
        offset += page_size;
    }

    Ok(all_deployments
        .into_iter()
        .find(|d| d.name == deployment_name_or_id || d.id == deployment_name_or_id))
}

/// Search each workspace for the deployment; succeed only when exactly one holds it
async fn find_deployment_in_workspaces(
    client: &LangchainClient,
    deployment_name_or_id: &str,
    workspaces: &[Workspace],
) -> Result<(Deployment, Workspace)> {
    let mut found = Vec::new();
    for workspace in workspaces {
        let in_workspace = client.clone().with_workspace_id(workspace.id.clone());
        if let Some(deployment) = find_deployment(&in_workspace, deployment_name_or_id).await? {
            found.push((deployment, workspace.clone()));
        }
    }

    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(CliError::Config(format!(
            "Deployment '{}' not found in any workspace the API key reaches.\n\
             Workspaces this key reaches:\n{}\n{}",
            deployment_name_or_id,
            describe_workspaces(workspaces),
            CHOOSE_WORKSPACE_HELP
        ))),
        _ => {
            let holding: Vec<Workspace> = found.into_iter().map(|(_, w)| w).collect();
            Err(CliError::Config(format!(
                "More than one workspace the API key reaches holds a deployment named '{}'.\n\
                 Workspaces that hold it:\n{}\n{}",
                deployment_name_or_id,
                describe_workspaces(&holding),
                CHOOSE_WORKSPACE_HELP
            )))
        }
    }
}

fn workspace_name(workspace: &Workspace) -> &str {
    workspace
        .display_name
        .as_deref()
        .unwrap_or("unnamed workspace")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;

    fn mock_client(server: &mockito::ServerGuard, workspace_id: Option<&str>) -> LangchainClient {
        LangchainClient::with_base_urls(
            AuthConfig::new(
                Some("test_key".to_string()),
                None,
                workspace_id.map(str::to_string),
            ),
            server.url(),
            server.url(),
            server.url(),
        )
        .unwrap()
    }

    fn deployments_body(deployments: &[(&str, &str)]) -> String {
        let resources: Vec<serde_json::Value> = deployments
            .iter()
            .map(|(name, tenant_id)| {
                serde_json::json!({
                    "id": format!("{}-id", name),
                    "name": name,
                    "tenant_id": tenant_id,
                    "source_config": {"custom_url": format!("https://{}.example", name)},
                })
            })
            .collect();
        serde_json::json!({"resources": resources, "offset": 0}).to_string()
    }

    const TWO_WORKSPACES: &str = r#"[{"id":"ws-a","display_name":"Workspace A"},
                                     {"id":"ws-b","display_name":"Workspace B"}]"#;

    async fn mock_deployments(
        server: &mut mockito::ServerGuard,
        workspace_header: Matcher,
        body: String,
    ) -> mockito::Mock {
        server
            .mock("GET", Matcher::Regex("^/v2/deployments".to_string()))
            .match_header("x-tenant-id", workspace_header)
            .with_body(body)
            .create_async()
            .await
    }

    #[tokio::test]
    async fn test_several_workspaces_uses_the_one_that_holds_the_deployment() {
        let mut server = mockito::Server::new_async().await;
        let no_header = server
            .mock("GET", Matcher::Regex("^/v2/deployments".to_string()))
            .match_header("x-tenant-id", Matcher::Missing)
            .with_status(403)
            .with_body(r#"{"detail":"Forbidden"}"#)
            .expect(1)
            .create_async()
            .await;
        let lookup = server
            .mock("GET", "/api/v1/workspaces")
            .match_header("x-tenant-id", Matcher::Missing)
            .with_body(TWO_WORKSPACES)
            .expect(1)
            .create_async()
            .await;
        let in_a = mock_deployments(&mut server, "ws-a".into(), deployments_body(&[])).await;
        let in_b = mock_deployments(
            &mut server,
            "ws-b".into(),
            deployments_body(&[("my-deployment", "ws-b")]),
        )
        .await;

        let resolved = resolve_deployment_with(&mock_client(&server, None), "my-deployment")
            .await
            .unwrap();

        assert_eq!(
            resolved,
            ResolvedDeployment {
                url: "https://my-deployment.example".to_string(),
                workspace_id: Some("ws-b".to_string()),
            }
        );
        no_header.assert_async().await;
        lookup.assert_async().await;
        in_a.assert_async().await;
        in_b.assert_async().await;
    }

    #[tokio::test]
    async fn test_several_workspaces_holding_the_deployment_is_an_error() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", Matcher::Regex("^/v2/deployments".to_string()))
            .match_header("x-tenant-id", Matcher::Missing)
            .with_status(403)
            .create_async()
            .await;
        server
            .mock("GET", "/api/v1/workspaces")
            .with_body(TWO_WORKSPACES)
            .create_async()
            .await;
        for ws in ["ws-a", "ws-b"] {
            mock_deployments(
                &mut server,
                ws.into(),
                deployments_body(&[("my-deployment", ws)]),
            )
            .await;
        }

        let err = resolve_deployment_with(&mock_client(&server, None), "my-deployment")
            .await
            .unwrap_err()
            .to_string();

        assert!(err.contains("More than one workspace"), "{}", err);
        assert!(err.contains("Workspace A (ws-a)"), "{}", err);
        assert!(err.contains("Workspace B (ws-b)"), "{}", err);
        assert!(err.contains("--workspace-id"), "{}", err);
    }

    #[tokio::test]
    async fn test_several_workspaces_none_holding_the_deployment_lists_them() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", Matcher::Regex("^/v2/deployments".to_string()))
            .match_header("x-tenant-id", Matcher::Missing)
            .with_status(403)
            .create_async()
            .await;
        server
            .mock("GET", "/api/v1/workspaces")
            .with_body(TWO_WORKSPACES)
            .create_async()
            .await;
        for ws in ["ws-a", "ws-b"] {
            mock_deployments(&mut server, ws.into(), deployments_body(&[])).await;
        }

        let err = resolve_deployment_with(&mock_client(&server, None), "missing")
            .await
            .unwrap_err()
            .to_string();

        assert!(err.contains("not found in any workspace"), "{}", err);
        assert!(err.contains("Workspace A (ws-a)"), "{}", err);
        assert!(err.contains("LANGSMITH_WORKSPACE_ID"), "{}", err);
    }

    #[tokio::test]
    async fn test_chosen_workspace_is_not_returned_as_found() {
        let mut server = mockito::Server::new_async().await;
        let lookup = server
            .mock("GET", "/api/v1/workspaces")
            .expect(0)
            .create_async()
            .await;
        mock_deployments(
            &mut server,
            "ws-a".into(),
            deployments_body(&[("my-deployment", "ws-a")]),
        )
        .await;

        let resolved =
            resolve_deployment_with(&mock_client(&server, Some("ws-a")), "my-deployment")
                .await
                .unwrap();

        assert_eq!(resolved.workspace_id, None);
        lookup.assert_async().await;
    }
}
