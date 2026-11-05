//! Integration tests of `langstar assistant compare` and `langstar assistant
//! verify` against the shared test deployment. Each test creates its assistant
//! through the SDK, exports it through the CLI, runs compare or verify, and
//! deletes the assistants and prompts it made. They are the integration and
//! CLI tests that tasks 9.1, "Add langstar assistant compare", and 9.2, "Add
//! langstar assistant verify", list in
//! `openspec/changes/export-import-prompts-and-assistants/tasks.md`. The unit
//! tests in `cli/src/commands/assistant.rs` cover the record lookup, the
//! prompt pointer check, the 422 answer and the prompt check's requests,
//! through a mock Agent Server.
//!
//! They need `LANGSMITH_API_KEY` and `LANGSMITH_WORKSPACE_ID`, and run in the
//! Integration Tests CI job:
//!
//! cargo nextest run -p langstar --features integration-tests --test assistant_compare_verify_test

mod common;

use assert_cmd::Command;
use common::fixtures::TestDeployment;
use langstar_sdk::{
    Assistant, AuthConfig, CommitRequest, CreateAssistantRequest, IfExists, LangchainClient,
    UpdateAssistantRequest,
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

fn create_assistant(client: &LangchainClient, config: Option<Value>) -> Assistant {
    let name = format!("compare-test-{}", &Uuid::new_v4().to_string()[..8]);
    runtime()
        .block_on(client.assistants().create(&CreateAssistantRequest {
            assistant_id: None,
            graph_id: TEST_GRAPH.to_string(),
            name,
            description: Some("Created by assistant_compare_verify_test".to_string()),
            config,
            context: None,
            metadata: Some(json!({"team": "support", "purpose": "compare_test"})),
            if_exists: Some(IfExists::Raise),
        }))
        .expect("the create should succeed")
}

fn delete_assistant(client: &LangchainClient, assistant_id: &str) {
    if let Err(e) = runtime().block_on(client.assistants().delete(assistant_id)) {
        eprintln!("Warning: could not delete the test assistant {assistant_id}: {e:?}");
    }
}

/// Change the assistant's `team` metadata value on the deployment.
fn change_team(client: &LangchainClient, assistant_id: &str) {
    runtime()
        .block_on(client.assistants().update(
            assistant_id,
            &UpdateAssistantRequest {
                name: None,
                graph_id: None,
                description: None,
                config: None,
                context: None,
                metadata: Some(json!({"team": "billing", "purpose": "compare_test"})),
            },
        ))
        .expect("the update should succeed");
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

/// What one run of `langstar -f json assistant <command>` printed.
struct Run {
    status: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    /// The JSON report on stdout.
    fn report(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|_| {
            panic!(
                "stdout should hold one JSON report: {}\nstderr: {}",
                self.stdout, self.stderr
            )
        })
    }
}

/// Run `langstar -f json assistant <command> <file> --deployment <test deployment>`.
fn run(command: &str, file: &Path, extra: &[&str]) -> Run {
    let mut args = vec![
        "-f",
        "json",
        "assistant",
        command,
        file.to_str().unwrap(),
        "--deployment",
        deployment().name.as_str(),
    ];
    args.extend_from_slice(extra);
    let output = langstar_cmd()
        .args(&args)
        .output()
        .expect("Failed to run langstar");
    Run {
        status: output.status.code().expect("langstar exits"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("the file should exist"))
        .expect("the file should hold JSON")
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, value.to_string()).unwrap();
}

/// Whether the report lists a difference at `path`.
fn reports_path(report: &Value, path: &str) -> bool {
    report["differences"]
        .as_array()
        .expect("the report lists its differences")
        .iter()
        .any(|difference| difference["path"] == path)
}

/// Create a prompt repo in the workspace `client` reaches, with one commit,
/// and return the commit's ID.
fn create_prompt(client: &LangchainClient, repo: &str) -> String {
    runtime().block_on(async {
        client
            .prompts()
            .create_repo(repo, None, None, false, None)
            .await
            .expect("the repo create should succeed");
        let pushed = client
            .prompts()
            .push(
                "-",
                repo,
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
        pushed.commit.id.expect("the push returns the commit's ID")
    })
}

fn delete_prompt(client: &LangchainClient, repo: &str) {
    if let Err(e) = runtime().block_on(client.prompts().delete(repo)) {
        eprintln!("Warning: could not delete the test prompt {repo}: {e:?}");
    }
}

/// Task 9.1, "Add langstar assistant compare": compare with `--assistant-id`
/// lists the metadata value someone changed on the deployment and exits with
/// status 3.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_compare_reports_changed_metadata() {
    let client = agent_server_client();
    let source = create_assistant(&client, None);
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    change_team(&client, &source.assistant_id);

    let compared = run("compare", &file, &["--assistant-id", &source.assistant_id]);
    delete_assistant(&client, &source.assistant_id);

    let report = compared.report();
    assert_eq!(compared.status, 3, "{report}");
    assert!(
        reports_path(&report, "/assistant/metadata/team"),
        "{report}"
    );
    assert_eq!(report["status"], "failed");
    assert_eq!(report["checks"], json!([]));
}

/// Task 9.1, "Add langstar assistant compare": compare with no destination
/// record entry and no `--assistant-id` exits with status 1 and names the
/// deployment.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_compare_without_a_record_entry_names_the_deployment() {
    let client = agent_server_client();
    let source = create_assistant(&client, None);
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    delete_assistant(&client, &source.assistant_id);

    let compared = run("compare", &file, &[]);

    assert_eq!(compared.status, 1, "{}", compared.stderr);
    assert!(
        compared.stderr.contains(deployment().name.as_str()),
        "{}",
        compared.stderr
    );
    assert!(
        compared.stderr.contains("--assistant-id"),
        "{}",
        compared.stderr
    );
}

/// Task 9.2, "Add langstar assistant verify": verify with `--assistant-id`
/// against the assistant it exported finds no difference, passes the graph
/// check and exits with status 0.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_verify_of_an_unchanged_assistant_passes() {
    let client = agent_server_client();
    let source = create_assistant(&client, Some(json!({"recursion_limit": 7})));
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);

    let verified = run("verify", &file, &["--assistant-id", &source.assistant_id]);
    delete_assistant(&client, &source.assistant_id);

    let report = verified.report();
    assert_eq!(verified.status, 0, "{report}");
    assert_eq!(report["differences"], json!([]));
    assert_eq!(report["checks"][0]["name"], "graph");
    assert_eq!(report["checks"][0]["outcome"], "passed");
}

/// Task 9.2, "Add langstar assistant verify": verify lists the metadata value
/// someone changed on the deployment and exits with status 3.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_verify_reports_changed_metadata() {
    let client = agent_server_client();
    let source = create_assistant(&client, None);
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    change_team(&client, &source.assistant_id);

    let verified = run("verify", &file, &["--assistant-id", &source.assistant_id]);
    delete_assistant(&client, &source.assistant_id);

    let report = verified.report();
    assert_eq!(verified.status, 3, "{report}");
    assert!(
        reports_path(&report, "/assistant/metadata/team"),
        "{report}"
    );
    assert_eq!(report["checks"][0]["outcome"], "passed");
}

/// Task 9.2, "Add langstar assistant verify": verify of a file whose graph ID
/// the deployment does not serve names the graph ID and exits with status 3.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_verify_reports_a_graph_the_deployment_lacks() {
    let client = agent_server_client();
    let source = create_assistant(&client, None);
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);
    let mut edited = read_json(&file);
    edited["assistant"]["graph_id"] = json!("graph_this_deployment_lacks");
    write_json(&file, &edited);

    let verified = run("verify", &file, &["--assistant-id", &source.assistant_id]);
    delete_assistant(&client, &source.assistant_id);

    let report = verified.report();
    assert_eq!(verified.status, 3, "{report}");
    assert_eq!(report["checks"][0]["name"], "graph");
    assert_eq!(report["checks"][0]["outcome"], "failed");
    assert!(
        report["checks"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("graph_this_deployment_lacks"),
        "{report}"
    );
}

/// Task 9.2, "Add langstar assistant verify": verify of a file that names a
/// prompt tag the workspace lacks names the prompt and exits with status 3.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_verify_reports_a_prompt_tag_the_workspace_lacks() {
    let langsmith = langsmith_client();
    let repo = format!("verify-test-{}", &Uuid::new_v4().to_string()[..8]);
    create_prompt(&langsmith, &repo);
    let missing_tag = format!("{repo}:prod");

    let client = agent_server_client();
    let source = create_assistant(
        &client,
        Some(json!({"configurable": {"system_prompt": missing_tag}})),
    );
    let directory = TempDir::new().unwrap();
    let file = export(
        directory.path(),
        &source.assistant_id,
        &["--prompt-ref", SYSTEM_PROMPT],
    );

    let verified = run("verify", &file, &["--assistant-id", &source.assistant_id]);
    delete_assistant(&client, &source.assistant_id);
    delete_prompt(&langsmith, &repo);

    let report = verified.report();
    assert_eq!(verified.status, 3, "{report}");
    assert_eq!(report["checks"][1]["name"], "prompt");
    assert_eq!(report["checks"][1]["outcome"], "failed");
    assert!(
        report["checks"][1]["detail"]
            .as_str()
            .unwrap()
            .contains(&missing_tag),
        "{report}"
    );
    assert_eq!(report["differences"], json!([]));
}

/// Task 9.2, "Add langstar assistant verify": verify with `--workspace-id` set
/// to the destination workspace finds a prompt tag that only that workspace
/// holds, and exits with status 0. Until task 2.1 adds the destination
/// workspace, the destination workspace is the test workspace, so the task 2.1
/// pull request removes the `#[ignore]` here.
#[test]
#[serial]
#[ignore = "needs task 2.1"]
fn test_verify_reads_prompts_from_the_workspace_that_workspace_id_names() {
    let destination_workspace = std::env::var("LANGSMITH_DESTINATION_WORKSPACE_ID")
        .expect("LANGSMITH_DESTINATION_WORKSPACE_ID must be set");
    let destination = langsmith_client().with_workspace_id(destination_workspace.clone());
    let repo = format!("verify-test-{}", &Uuid::new_v4().to_string()[..8]);
    let commit_id = create_prompt(&destination, &repo);
    runtime()
        .block_on(
            destination
                .prompts()
                .create_tag("-", &repo, "prod", &commit_id),
        )
        .expect("the tag create should succeed");
    let name = format!("{repo}:prod");

    let client = agent_server_client();
    let source = create_assistant(
        &client,
        Some(json!({"configurable": {"system_prompt": name}})),
    );
    let directory = TempDir::new().unwrap();
    let file = export(
        directory.path(),
        &source.assistant_id,
        &["--prompt-ref", SYSTEM_PROMPT],
    );

    let verified = run(
        "verify",
        &file,
        &[
            "--assistant-id",
            &source.assistant_id,
            "--workspace-id",
            &destination_workspace,
        ],
    );
    delete_assistant(&client, &source.assistant_id);
    delete_prompt(&destination, &repo);

    let report = verified.report();
    assert_eq!(verified.status, 0, "{report}");
    assert_eq!(report["checks"][1]["name"], "prompt");
    assert_eq!(report["checks"][1]["outcome"], "passed");
    assert!(
        report["checks"][1]["detail"]
            .as_str()
            .unwrap()
            .contains(&name),
        "{report}"
    );
}

/// Task 9.2, "Add langstar assistant verify": `langstar -f json assistant
/// verify` prints one JSON object with `status`, `differences` and `checks` to
/// stdout, and its progress to stderr.
#[test]
#[serial]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_verify_prints_one_json_object() {
    let client = agent_server_client();
    let source = create_assistant(&client, None);
    let directory = TempDir::new().unwrap();
    let file = export(directory.path(), &source.assistant_id, &[]);

    let verified = run("verify", &file, &["--assistant-id", &source.assistant_id]);
    delete_assistant(&client, &source.assistant_id);

    let report = verified.report();
    let members = report.as_object().expect("stdout holds one JSON object");
    for key in ["status", "differences", "checks"] {
        assert!(members.contains_key(key), "{key} missing from {report}");
    }
    assert!(
        !members.contains_key("action"),
        "verify reports no action: {report}"
    );
    assert!(
        verified.stderr.contains("Comparing"),
        "progress goes to stderr: {}",
        verified.stderr
    );
}
