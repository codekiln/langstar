//! Integration tests for `langstar assistant export`
//!
//! Task 8.1, "Add langstar assistant export", of the
//! `export-import-prompts-and-assistants` OpenSpec change lists these tests in
//! `openspec/changes/export-import-prompts-and-assistants/tasks.md`. Each one creates its assistant through the SDK on the shared test
//! deployment from `common/fixtures.rs`, exports it through the CLI, and deletes
//! the assistant afterwards. The unit tests in `cli/src/commands/assistant.rs`
//! cover the credential check and `--not-a-secret`, which need no live server.
//!
//! They need `LANGSMITH_API_KEY` and `LANGSMITH_WORKSPACE_ID`, and run in the
//! Integration Tests CI job:
//!
//! cargo nextest run -p langstar --features integration-tests --test assistant_export_test

mod common;

use assert_cmd::Command;
use common::fixtures::TestDeployment;
use langstar_sdk::{Assistant, AuthConfig, CreateAssistantRequest, LangchainClient};
use serde_json::{Value, json};
use serial_test::serial;
use std::sync::OnceLock;
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

/// The graph the test deployment serves
const TEST_GRAPH: &str = "test_graph";

static TEST_DEPLOYMENT: OnceLock<TestDeployment> = OnceLock::new();

fn deployment() -> &'static TestDeployment {
    TEST_DEPLOYMENT.get_or_init(TestDeployment::create)
}

fn langstar_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
    cmd.env(
        "LANGSTAR_CONFIG_DIR",
        common::config_dir::empty_config_dir(),
    );
    cmd
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
}

/// A client for the test deployment's Agent Server.
///
/// A deployment built from GitHub can report READY while its `custom_url` is
/// still empty, for about a minute, so this waits for the URL. Once it has one,
/// the CLI's own lookup finds it too.
fn agent_server_client() -> LangchainClient {
    let deployment_id = deployment().id.clone();
    runtime().block_on(async {
        let auth = AuthConfig::from_env()
            .expect("LANGSMITH_API_KEY and LANGSMITH_WORKSPACE_ID must be set");
        let control_plane = LangchainClient::new(auth.clone()).expect("Failed to create client");
        let mut waited = Duration::ZERO;
        let url = loop {
            let found = control_plane
                .deployments()
                .get(&deployment_id)
                .await
                .expect("Failed to read the test deployment")
                .custom_url()
                .filter(|url| !url.is_empty());
            if let Some(url) = found {
                break url;
            }
            assert!(
                waited < Duration::from_secs(300),
                "the test deployment still has no custom_url after five minutes"
            );
            tokio::time::sleep(Duration::from_secs(10)).await;
            waited += Duration::from_secs(10);
        };
        LangchainClient::new(auth)
            .expect("Failed to create client")
            .with_langgraph_url(url)
    })
}

fn create_assistant(
    client: &LangchainClient,
    config: Option<Value>,
    context: Option<Value>,
    metadata: Value,
) -> Assistant {
    let name = format!("export-test-{}", &Uuid::new_v4().to_string()[..8]);
    runtime()
        .block_on(client.assistants().create(&CreateAssistantRequest {
            assistant_id: None,
            graph_id: TEST_GRAPH.to_string(),
            name,
            description: Some("Created by assistant_export_test".to_string()),
            config,
            context,
            metadata: Some(metadata),
            if_exists: None,
        }))
        .expect("the create should succeed")
}

fn read_assistant(client: &LangchainClient, assistant_id: &str) -> Assistant {
    runtime()
        .block_on(client.assistants().get(assistant_id))
        .expect("the read should succeed")
}

fn delete_assistant(client: &LangchainClient, assistant_id: &str) {
    if let Err(e) = runtime().block_on(client.assistants().delete(assistant_id)) {
        eprintln!("Warning: could not delete the test assistant {assistant_id}: {e:?}");
    }
}

fn export_args<'a>(assistant_id: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "assistant",
        "export",
        assistant_id,
        "--deployment",
        deployment().name.as_str(),
    ];
    args.extend_from_slice(extra);
    args
}

fn read_json(path: &std::path::Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("export should write the file"))
        .expect("the file should hold JSON")
}

/// With `LANGSTAR_DEBUG_HTTP` set, the SDK writes each response body to a
/// debug file and prints the start of it to stderr. Export refuses before it
/// sends any request, so neither holds the assistant. This test sends no
/// request, so it runs without credentials.
#[test]
fn test_export_refuses_while_http_debugging_is_on() {
    let directory = TempDir::new().unwrap();
    let debug_file = directory.path().join("debug.json");
    let file = directory.path().join("support-bot.json");
    let output = langstar_cmd()
        .env("LANGSTAR_DEBUG_HTTP", "1")
        .env("LANGSTAR_DEBUG_FILE", &debug_file)
        .env("LANGSMITH_API_KEY", "placeholder-key")
        .args([
            "assistant",
            "export",
            "3f1c2b8e-0d4a-4c6e-9b7a-2e5f8d9c1a00",
            "--deployment",
            "my-agent",
            "--output",
            file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run langstar");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("LANGSTAR_DEBUG_HTTP"), "{stderr}");
    assert!(output.stdout.is_empty());
    assert!(!debug_file.exists(), "no request reached the debug file");
    assert!(!file.exists(), "export should write no file");
}

/// Task 8.1, "Add langstar assistant export": export writes every field of an assistant with configuration,
/// context and metadata.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_writes_every_field() {
    let client = agent_server_client();
    let created = create_assistant(
        &client,
        Some(json!({"recursion_limit": 7})),
        Some(json!({"tone": "warm", "max_turns": 4})),
        json!({"team": "support", "purpose": "export_test"}),
    );
    let source = read_assistant(&client, &created.assistant_id);

    let directory = TempDir::new().unwrap();
    let file = directory.path().join("support-bot.json");
    let file_arg = file.to_str().unwrap();
    let output = langstar_cmd()
        .args(export_args(&created.assistant_id, &["--output", file_arg]))
        .output()
        .expect("Failed to run langstar");
    delete_assistant(&client, &created.assistant_id);

    assert!(
        output.status.success(),
        "export should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "with --output, stdout stays empty"
    );

    let written = read_json(&file);
    assert_eq!(written["kind"], "assistant");
    assert_eq!(written["format_version"], 1);
    assert_eq!(
        written["source"],
        json!({
            "deployment_id": deployment().id,
            "assistant_id": source.assistant_id,
            "version": source.version,
        })
    );
    let assistant = &written["assistant"];
    assert_eq!(assistant["name"], json!(source.name));
    assert_eq!(assistant["description"], json!(source.description));
    assert_eq!(assistant["graph_id"], TEST_GRAPH);
    assert_eq!(assistant["config"], json!(source.config));
    assert_eq!(assistant["context"], json!(source.context));
    assert_eq!(assistant["metadata"], json!(source.metadata));
    assert_eq!(assistant["config"]["recursion_limit"], 7);
    assert_eq!(assistant["context"]["tone"], "warm");
    assert_eq!(assistant["metadata"]["purpose"], "export_test");
    assert_eq!(written["requires"], json!({"prompts": {}}));
}

/// Task 8.1, "Add langstar assistant export": `--prompt-ref` records the pointer and the prompt name it holds.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_records_a_prompt_reference() {
    let client = agent_server_client();
    let created = create_assistant(
        &client,
        Some(json!({"configurable": {"system_prompt": "support-system:prod"}})),
        None,
        json!({"purpose": "export_test"}),
    );

    let directory = TempDir::new().unwrap();
    let file = directory.path().join("support-bot.json");
    let file_arg = file.to_str().unwrap();
    let output = langstar_cmd()
        .args(export_args(
            &created.assistant_id,
            &[
                "--output",
                file_arg,
                "--prompt-ref",
                "/config/configurable/system_prompt",
            ],
        ))
        .output()
        .expect("Failed to run langstar");
    delete_assistant(&client, &created.assistant_id);

    assert!(
        output.status.success(),
        "export should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        read_json(&file)["requires"]["prompts"],
        json!({"/config/configurable/system_prompt": "support-system:prod"})
    );
}

/// Task 8.1, "Add langstar assistant export": two exports of one assistant version write identical files.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_twice_writes_identical_files() {
    let client = agent_server_client();
    let created = create_assistant(
        &client,
        Some(json!({"configurable": {"b": "2", "a": "1"}})),
        None,
        json!({"zeta": 1, "alpha": 2}),
    );

    let directory = TempDir::new().unwrap();
    let mut texts = Vec::new();
    for name in ["first.json", "second.json"] {
        let file = directory.path().join(name);
        let output = langstar_cmd()
            .args(export_args(
                &created.assistant_id,
                &["--output", file.to_str().unwrap()],
            ))
            .output()
            .expect("Failed to run langstar");
        assert!(
            output.status.success(),
            "export should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        texts.push(std::fs::read_to_string(&file).unwrap());
    }
    delete_assistant(&client, &created.assistant_id);

    assert_eq!(texts[0], texts[1]);
}

/// Task 8.1, "Add langstar assistant export": an assistant ID the deployment lacks exits with status 1, names
/// the ID and writes no file.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_of_a_missing_assistant_names_the_id() {
    let _client = agent_server_client();
    let missing = Uuid::new_v4().to_string();

    let directory = TempDir::new().unwrap();
    let file = directory.path().join("support-bot.json");
    let output = langstar_cmd()
        .args(export_args(&missing, &["--output", file.to_str().unwrap()]))
        .output()
        .expect("Failed to run langstar");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&missing),
        "stderr should name the ID: {stderr}"
    );
    assert!(!file.exists(), "export should write no file");
}

/// Task 8.1, "Add langstar assistant export": export refuses a graph's default assistant.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_refuses_a_graphs_default_assistant() {
    let client = agent_server_client();
    // The test deployment holds many assistants the other tests created, so a
    // search by graph ID alone can return a page without the default one. The
    // Agent Server's search filters by metadata as well, and the default
    // assistant carries `created_by: system`.
    let default = runtime()
        .block_on(async {
            let request = client.langgraph_post("/assistants/search")?.json(&json!({
                "graph_id": TEST_GRAPH,
                "metadata": {"created_by": "system"},
                "limit": 10,
            }));
            client.execute::<Vec<Assistant>>(request).await
        })
        .expect("the search for the default assistant should succeed")
        .into_iter()
        .next()
        .expect("the deployment has a default assistant for test_graph");

    let directory = TempDir::new().unwrap();
    let file = directory.path().join("default.json");
    let output = langstar_cmd()
        .args(export_args(
            &default.assistant_id,
            &["--output", file.to_str().unwrap()],
        ))
        .output()
        .expect("Failed to run langstar");

    assert_eq!(output.status.code(), Some(1));
    assert!(!file.exists(), "export should write no file");
    assert!(output.stdout.is_empty());
}

/// Task 8.1, "Add langstar assistant export": without `--output`, stdout holds only the file and progress goes
/// to stderr.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_to_stdout_prints_only_the_file() {
    let client = agent_server_client();
    let created = create_assistant(&client, None, None, json!({"purpose": "export_test"}));

    let output = langstar_cmd()
        .args(export_args(&created.assistant_id, &[]))
        .output()
        .expect("Failed to run langstar");
    delete_assistant(&client, &created.assistant_id);

    assert!(
        output.status.success(),
        "export should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let written: Value =
        serde_json::from_slice(&output.stdout).expect("stdout should hold one JSON document");
    assert_eq!(
        written["source"]["assistant_id"],
        json!(created.assistant_id)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Reading assistant"),
        "progress goes to stderr"
    );
}
