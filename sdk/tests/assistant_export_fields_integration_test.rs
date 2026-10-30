//! Integration tests for the assistant fields export and import read and write
//!
//! These tests cover the assistant's `context` and `version`, the search by graph ID,
//! and the graph an update moves an assistant to. They run against the test deployment
//! the fixture in `sdk/src/test_utils.rs` keeps alive, and need `LANGSMITH_API_KEY` and
//! `LANGSMITH_WORKSPACE_ID`.
//!
//! Run with: cargo test --features integration-tests,test-utils -p langstar-sdk --test assistant_export_fields_integration_test
//!
//! The SDK Integration Tests job in `.github/workflows/ci.yml` runs them on every pull
//! request, and `mise run check` runs them locally.

use langstar_sdk::test_utils::TestDeploymentConfig;
use langstar_sdk::{
    Assistant, AssistantClient, AuthConfig, CreateAssistantRequest, LangchainClient,
    UpdateAssistantRequest,
};
use serde_json::json;
use uuid::Uuid;

/// The graph the test deployment serves, as `tests/fixtures/test-graph-deployment/langgraph.json`
/// registers it
const TEST_GRAPH: &str = "test_graph";

/// The second graph ID the destination deployment serves.
///
/// Task 2.3 registers the test graph under this ID as well, and task 2.4 deploys that
/// file as the destination deployment, so only the test that waits for task 2.4 uses it.
const TEST_GRAPH_ALT: &str = "test_graph_alt";

/// The name prefix task 2.4 gives the destination deployment
const DESTINATION_DEPLOYMENT_PREFIX: &str = "dest-integration-test";

/// The graph config file task 2.3 adds
const DESTINATION_CONFIG_PATH: &str = "tests/fixtures/test-graph-deployment-alt/langgraph.json";

fn test_assistant_name(prefix: &str) -> String {
    format!("{}-{}", prefix, &Uuid::new_v4().to_string()[..8])
}

/// A client pointing at one deployment's Agent Server
async fn deployment_client(config: &TestDeploymentConfig) -> LangchainClient {
    let auth =
        AuthConfig::from_env().expect("LANGSMITH_API_KEY and LANGSMITH_WORKSPACE_ID must be set");
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required");
    let client = LangchainClient::new(auth.clone()).expect("Failed to create client");

    let (deployment_id, _revision_id, _name) =
        langstar_sdk::test_utils::get_or_create_deployment(&client, config)
            .await
            .expect("Failed to get or create the test deployment");

    let deployment = client
        .deployments()
        .get(&deployment_id)
        .await
        .expect("Failed to read the test deployment");
    let custom_url = deployment
        .custom_url()
        .expect("The test deployment has no custom_url");

    LangchainClient::new(auth)
        .expect("Failed to create client")
        .with_langgraph_url(custom_url)
}

async fn delete_assistant(assistants: &AssistantClient<'_>, assistant: &Assistant) {
    if let Err(e) = assistants.delete(&assistant.assistant_id).await {
        eprintln!(
            "Warning: could not delete the test assistant {}: {:?}",
            assistant.assistant_id, e
        );
    }
}

/// Task 3.3: an assistant keeps the context a create sends, comes back with its version,
/// and the search finds it by graph ID.
#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
async fn test_assistant_keeps_its_context_and_is_found_by_graph_id() {
    let client = deployment_client(&TestDeploymentConfig::default()).await;
    let assistants = client.assistants();

    let name = test_assistant_name("context-test");
    let created = assistants
        .create(&CreateAssistantRequest {
            assistant_id: None,
            graph_id: TEST_GRAPH.to_string(),
            name: name.clone(),
            description: None,
            config: None,
            context: Some(json!({"tone": "warm"})),
            metadata: Some(json!({"purpose": "export_fields_test"})),
            if_exists: None,
        })
        .await
        .expect("the create should succeed");

    let read_back = assistants
        .get(&created.assistant_id)
        .await
        .expect("the read should succeed");
    assert_eq!(
        read_back.context,
        Some(json!({"tone": "warm"})),
        "the assistant keeps the context the create sent"
    );
    assert_eq!(
        read_back.version,
        Some(1),
        "a newly created assistant is at version 1"
    );

    let by_graph = assistants
        .search_by_graph_id(TEST_GRAPH, Some(100))
        .await
        .expect("the search by graph ID should succeed");
    assert!(
        by_graph
            .iter()
            .any(|a| a.assistant_id == created.assistant_id),
        "the search by graph ID finds the assistant it just created"
    );
    assert!(
        by_graph.iter().all(|a| a.graph_id == TEST_GRAPH),
        "the search by graph ID returns only that graph's assistants"
    );

    delete_assistant(&assistants, &created).await;
}

/// Task 3.3: an update moves an assistant from one graph to another.
///
/// The existing test deployment serves only `test_graph`, so this test waits for the
/// destination deployment that task 2.4 adds and the second graph ID that task 2.3
/// registers. Task 2.4 removes the `#[ignore]` and takes the deployment from the fixture
/// it adds to `sdk/src/test_utils.rs` in place of the config built here.
#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
#[ignore = "needs task 2.4"]
async fn test_update_moves_an_assistant_to_another_graph() {
    let config = TestDeploymentConfig {
        name_prefix: Some(DESTINATION_DEPLOYMENT_PREFIX.to_string()),
        config_path: DESTINATION_CONFIG_PATH.to_string(),
        ..Default::default()
    };
    let client = deployment_client(&config).await;
    let assistants = client.assistants();

    let name = test_assistant_name("graph-move-test");
    let created = assistants
        .create(&CreateAssistantRequest {
            assistant_id: None,
            graph_id: TEST_GRAPH.to_string(),
            name: name.clone(),
            description: None,
            config: None,
            context: None,
            metadata: Some(json!({"purpose": "export_fields_test"})),
            if_exists: None,
        })
        .await
        .expect("the create should succeed");
    assert_eq!(created.graph_id, TEST_GRAPH);

    let moved = assistants
        .update(
            &created.assistant_id,
            &UpdateAssistantRequest {
                name: None,
                graph_id: Some(TEST_GRAPH_ALT.to_string()),
                description: None,
                config: None,
                context: None,
                metadata: None,
            },
        )
        .await
        .expect("the update should succeed");
    assert_eq!(
        moved.graph_id, TEST_GRAPH_ALT,
        "the update moves the assistant to the second graph"
    );

    let read_back = assistants
        .get(&created.assistant_id)
        .await
        .expect("the read should succeed");
    assert_eq!(read_back.graph_id, TEST_GRAPH_ALT);

    delete_assistant(&assistants, &created).await;
}
