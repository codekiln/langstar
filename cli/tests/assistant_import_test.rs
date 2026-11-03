//! Integration tests that export an assistant from the shared test deployment
//! and import the file back onto the same deployment as a new assistant. Each
//! test creates its source assistant through the SDK, exports it through the
//! CLI, and deletes the assistants it made. They are the tests that task 10.1,
//! "Add langstar assistant import", lists in
//! `openspec/changes/export-import-prompts-and-assistants/tasks.md`. Until task
//! 2.4 adds a destination deployment, the test deployment is the destination
//! too, as tasks.md says. The unit tests in
//! `cli/src/commands/assistant.rs` cover the settings field import sends, the
//! pending record entry and the read-back, through a mock Agent Server.
//!
//! They need `LANGSMITH_API_KEY` and `LANGSMITH_WORKSPACE_ID`, and run in the
//! Integration Tests CI job:
//!
//! cargo nextest run -p langstar --features integration-tests --test assistant_import_test

mod common;

use assert_cmd::Command;
use common::fixtures::TestDeployment;
use langstar_sdk::{
    Assistant, AuthConfig, CommitRequest, CreateAssistantRequest, IfExists, LangchainClient,
};
use serde_json::{Value, json};
use serial_test::serial;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

/// The graph the test deployment serves
const TEST_GRAPH: &str = "test_graph";

/// Where the test assistants keep their prompt name
const SYSTEM_PROMPT: &str = "/config/configurable/system_prompt";

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

/// A client for the test workspace's LangSmith API.
fn langsmith_client() -> LangchainClient {
    let auth =
        AuthConfig::from_env().expect("LANGSMITH_API_KEY and LANGSMITH_WORKSPACE_ID must be set");
    LangchainClient::new(auth).expect("Failed to create client")
}

/// A client for the test deployment's Agent Server.
///
/// A deployment built from GitHub can report READY while its `custom_url` is
/// still empty, for about a minute, so this waits for the URL. Once it has one,
/// the CLI's own lookup finds it too.
fn agent_server_client() -> LangchainClient {
    let deployment_id = deployment().id.clone();
    let control_plane = langsmith_client();
    runtime().block_on(async {
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
        control_plane.clone().with_langgraph_url(url)
    })
}

fn create_assistant(
    client: &LangchainClient,
    assistant_id: Option<String>,
    config: Option<Value>,
    context: Option<Value>,
) -> Assistant {
    let name = format!("import-test-{}", &Uuid::new_v4().to_string()[..8]);
    runtime()
        .block_on(client.assistants().create(&CreateAssistantRequest {
            assistant_id,
            graph_id: TEST_GRAPH.to_string(),
            name,
            description: Some("Created by assistant_import_test".to_string()),
            config,
            context,
            metadata: Some(json!({"team": "support", "purpose": "import_test"})),
            if_exists: Some(IfExists::Raise),
        }))
        .expect("the create should succeed")
}

fn read_assistant(client: &LangchainClient, assistant_id: &str) -> Option<Assistant> {
    runtime()
        .block_on(client.assistants().get(assistant_id))
        .ok()
}

fn delete_assistant(client: &LangchainClient, assistant_id: &str) {
    if let Err(e) = runtime().block_on(client.assistants().delete(assistant_id)) {
        eprintln!("Warning: could not delete the test assistant {assistant_id}: {e:?}");
    }
}

/// Export `assistant_id` to `support-bot.json` in `directory` and return its path.
fn export(directory: &Path, assistant_id: &str, extra: &[&str]) -> PathBuf {
    let file = directory.join("support-bot.json");
    let mut args = vec![
        "assistant",
        "export",
        assistant_id,
        "--deployment",
        deployment().name.as_str(),
        "--output",
        file.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    let output = langstar_cmd()
        .args(&args)
        .output()
        .expect("Failed to run langstar");
    assert!(
        output.status.success(),
        "export should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    file
}

/// Run `langstar -f json assistant import` and return its exit status and report.
fn import(file: &Path, extra: &[&str]) -> (i32, Value) {
    let mut args = vec![
        "-f",
        "json",
        "assistant",
        "import",
        file.to_str().unwrap(),
        "--deployment",
        deployment().name.as_str(),
    ];
    args.extend_from_slice(extra);
    let output = langstar_cmd()
        .args(&args)
        .output()
        .expect("Failed to run langstar");
    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "stdout should hold the JSON report: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().expect("langstar exits"), report)
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("the file should exist"))
        .expect("the file should hold JSON")
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, value.to_string()).unwrap();
}

/// Task 10.1, "Add langstar assistant import": import creates the assistant
/// with every field of the file, records its ID, and a second import finds it
/// and creates no other.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_import_creates_the_assistant_and_finds_it_again() {
    let client = agent_server_client();
    let source = create_assistant(
        &client,
        None,
        Some(json!({"recursion_limit": 7})),
        Some(json!({"tone": "warm", "max_turns": 4})),
    );
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    delete_assistant(&client, &source.assistant_id);

    let (first_status, first) = import(&file, &[]);
    let created_id = first["destination_id"].as_str().map(str::to_string);
    let (second_status, second) = import(&file, &[]);
    let created = created_id
        .as_deref()
        .and_then(|id| read_assistant(&client, id));
    if let Some(id) = &created_id {
        delete_assistant(&client, id);
    }

    assert_eq!(first_status, 0, "{first}");
    assert_eq!(first["action"], "create");
    let created = created.expect("the deployment holds the new assistant");
    let written = read_json(&file)["assistant"].clone();
    assert_eq!(json!(created.name), written["name"]);
    assert_eq!(json!(created.description), written["description"]);
    assert_eq!(created.graph_id, TEST_GRAPH);
    assert_eq!(json!(created.config), written["config"]);
    assert_eq!(json!(created.context), written["context"]);
    assert_eq!(json!(created.metadata), written["metadata"]);

    let record = read_json(&directory.path().join("support-bot.destinations.json"));
    assert_eq!(
        record["destinations"][deployment().id.as_str()],
        json!({"assistant_id": created_id, "state": "created"})
    );

    assert_eq!(second_status, 0, "{second}");
    assert_eq!(second["action"], "unchanged");
    assert_eq!(second["destination_id"], first["destination_id"]);
}

/// Task 10.1, "Add langstar assistant import": `--record <path>` writes the
/// destination record to that path and no record beside the export file.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_import_writes_the_record_where_record_names() {
    let client = agent_server_client();
    let source = create_assistant(&client, None, None, Some(json!({"tone": "warm"})));
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    delete_assistant(&client, &source.assistant_id);
    let record = directory.path().join("prod.json");

    let (status, report) = import(&file, &["--record", record.to_str().unwrap()]);
    if let Some(id) = report["destination_id"].as_str() {
        delete_assistant(&client, id);
    }

    assert_eq!(status, 0, "{report}");
    assert_eq!(
        read_json(&record)["destinations"][deployment().id.as_str()]["assistant_id"],
        report["destination_id"]
    );
    assert!(
        !directory
            .path()
            .join("support-bot.destinations.json")
            .exists()
    );
}

/// Task 10.1, "Add langstar assistant import": a file whose graph the
/// deployment does not serve creates no assistant and exits with status 3.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_import_of_a_graph_the_deployment_lacks_creates_nothing() {
    let client = agent_server_client();
    let source = create_assistant(&client, None, None, Some(json!({"tone": "warm"})));
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    delete_assistant(&client, &source.assistant_id);
    let mut edited = read_json(&file);
    edited["assistant"]["graph_id"] = json!("graph_this_deployment_lacks");
    write_json(&file, &edited);

    let (status, report) = import(&file, &[]);

    assert_eq!(status, 3, "{report}");
    assert_eq!(report["checks"][0]["name"], "graph");
    assert_eq!(report["checks"][0]["outcome"], "failed");
    assert!(report.get("action").is_none(), "{report}");
    assert!(
        !directory
            .path()
            .join("support-bot.destinations.json")
            .exists()
    );
}

/// Task 10.1, "Add langstar assistant import": a file naming a prompt the
/// workspace lacks creates no assistant and exits with status 3, and one naming
/// a prompt the workspace holds passes the prompt check.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_import_checks_the_prompts_the_assistant_names() {
    let langsmith = langsmith_client();
    let repo = format!("import-test-{}", &Uuid::new_v4().to_string()[..8]);
    runtime().block_on(async {
        langsmith
            .prompts()
            .create_repo(&repo, None, None, false, None)
            .await
            .expect("the repo create should succeed");
        langsmith
            .prompts()
            .push(
                "-",
                &repo,
                &CommitRequest {
                    manifest: json!({
                        "lc": 1,
                        "type": "constructor",
                        "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
                        "kwargs": {
                            "input_variables": ["question"],
                            "template": "Answer this question: {question}",
                            "template_format": "f-string"
                        }
                    }),
                    description: None,
                    parent_commit: None,
                    example_run_ids: None,
                },
            )
            .await
            .expect("the push should succeed");
    });

    let client = agent_server_client();
    let found = create_assistant(
        &client,
        None,
        Some(json!({"configurable": {"system_prompt": repo}})),
        None,
    );
    let missing_name = format!("{repo}-missing:prod");
    let missing = create_assistant(
        &client,
        None,
        Some(json!({"configurable": {"system_prompt": missing_name}})),
        None,
    );
    let found_directory = TempDir::new().unwrap();
    let missing_directory = TempDir::new().unwrap();
    let found_file = export(
        found_directory.path(),
        &found.assistant_id,
        &["--prompt-ref", SYSTEM_PROMPT],
    );
    let missing_file = export(
        missing_directory.path(),
        &missing.assistant_id,
        &["--prompt-ref", SYSTEM_PROMPT],
    );
    delete_assistant(&client, &found.assistant_id);
    delete_assistant(&client, &missing.assistant_id);

    let (found_status, found_report) = import(&found_file, &[]);
    let (missing_status, missing_report) = import(&missing_file, &[]);
    if let Some(id) = found_report["destination_id"].as_str() {
        delete_assistant(&client, id);
    }
    if let Some(id) = missing_report["destination_id"].as_str() {
        delete_assistant(&client, id);
    }
    if let Err(e) = runtime().block_on(langsmith.prompts().delete(&repo)) {
        eprintln!("Warning: could not delete the test prompt {repo}: {e:?}");
    }

    assert_eq!(found_status, 0, "{found_report}");
    assert_eq!(found_report["checks"][1]["name"], "prompt");
    assert_eq!(found_report["checks"][1]["outcome"], "passed");

    assert_eq!(missing_status, 3, "{missing_report}");
    assert_eq!(missing_report["checks"][1]["outcome"], "failed");
    assert!(
        missing_report["checks"][1]["detail"]
            .as_str()
            .unwrap()
            .contains(&missing_name)
    );
    assert!(missing_report.get("action").is_none(), "{missing_report}");
}

/// Task 10.1, "Add langstar assistant import": a file holding its settings in
/// one field imports onto a deployment that answers with both, reports no
/// difference, and a second import creates no other assistant.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_import_of_settings_in_one_field_matches_on_every_import() {
    let client = agent_server_client();
    let source = create_assistant(&client, None, None, Some(json!({"tone": "warm"})));
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    delete_assistant(&client, &source.assistant_id);
    let exported = read_json(&file);

    let mut config_only = exported.clone();
    config_only["assistant"]["config"]["configurable"] = json!({"tone": "warm"});
    config_only["assistant"]
        .as_object_mut()
        .unwrap()
        .remove("context");
    let mut context_only = exported;
    context_only["assistant"]["config"]
        .as_object_mut()
        .unwrap()
        .remove("configurable");
    context_only["assistant"]["context"] = json!({"tone": "warm"});

    for (field, edited) in [("config", config_only), ("context", context_only)] {
        let directory = TempDir::new().unwrap();
        let file = directory.path().join("support-bot.json");
        write_json(&file, &edited);

        let (first_status, first) = import(&file, &[]);
        let (second_status, second) = import(&file, &[]);
        if let Some(id) = first["destination_id"].as_str() {
            delete_assistant(&client, id);
        }

        assert_eq!(first_status, 0, "settings in {field}: {first}");
        assert_eq!(second_status, 0, "settings in {field}: {second}");
        assert_eq!(second["action"], "unchanged");
        assert_eq!(second["destination_id"], first["destination_id"]);
    }
}

/// Task 10.1, "Add langstar assistant import": a record that holds an ID as
/// pending, for an assistant an earlier import created, makes import send the
/// same create, so the deployment still holds one assistant with that ID.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_import_finishes_a_pending_create() {
    let client = agent_server_client();
    let source = create_assistant(&client, None, None, Some(json!({"tone": "warm"})));
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    delete_assistant(&client, &source.assistant_id);

    let written = read_json(&file)["assistant"].clone();
    let chosen = Uuid::new_v4().to_string();
    runtime()
        .block_on(client.assistants().create(&CreateAssistantRequest {
            assistant_id: Some(chosen.clone()),
            graph_id: TEST_GRAPH.to_string(),
            name: written["name"].as_str().unwrap().to_string(),
            description: written["description"].as_str().map(str::to_string),
            config: Some(written["config"].clone()),
            context: None,
            metadata: Some(written["metadata"].clone()),
            if_exists: Some(IfExists::Raise),
        }))
        .expect("the create should succeed");
    let record = directory.path().join("support-bot.destinations.json");
    write_json(
        &record,
        &json!({
            "kind": "assistant",
            "format_version": 1,
            "destinations": {
                deployment().id.as_str(): {"assistant_id": chosen, "state": "pending"}
            }
        }),
    );

    let (status, report) = import(&file, &[]);
    let named = runtime()
        .block_on(
            client
                .assistants()
                .search(written["name"].as_str().unwrap(), Some(10)),
        )
        .unwrap_or_default();
    delete_assistant(&client, &chosen);
    for other in named.iter().filter(|a| a.assistant_id != chosen) {
        delete_assistant(&client, &other.assistant_id);
    }

    assert_eq!(status, 0, "{report}");
    assert_eq!(report["destination_id"], json!(chosen));
    assert_eq!(
        named.len(),
        1,
        "the deployment holds one assistant from the file"
    );
    assert_eq!(
        read_json(&record)["destinations"][deployment().id.as_str()],
        json!({"assistant_id": chosen, "state": "created"})
    );
}
