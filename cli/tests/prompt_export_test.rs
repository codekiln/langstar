//! Integration tests for `langstar prompt export`
//!
//! Each test creates a private prompt in the workspace `LANGSMITH_WORKSPACE_ID`
//! names, pushes commits to it through the SDK, runs the langstar binary to export
//! it, and compares the file against the SDK's own read of the commit. Each test
//! deletes its prompt when it ends, whether it passed or not.
//!
//! The unit tests in `cli/src/commands/prompt.rs` cover the credential check's
//! refusals, `--not-a-secret` and the model class through a `mockito` server.
//!
//! **Prerequisites:** `LANGSMITH_API_KEY` and `LANGSMITH_WORKSPACE_ID`.
//!
//! Run with: cargo nextest run -p langstar --features integration-tests --test prompt_export_test
//!
//! The tasks these tests verify live in
//! `openspec/changes/export-import-prompts-and-assistants/tasks.md`, task 5.1.
#[path = "common/config_dir.rs"]
mod config_dir;

use assert_cmd::Command;
use langstar_sdk::auth::AuthConfig;
use langstar_sdk::client::LangchainClient;
use langstar_sdk::prompts::CommitRequest;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

/// The "-" owner, which names the workspace's own private prompts
const OWNER: &str = "-";

fn check_env_vars() {
    std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for prompt export tests");
}

/// A private prompt for one test, deleted when the test ends.
struct TestPrompt {
    repo: String,
    runtime: tokio::runtime::Runtime,
    client: LangchainClient,
}

impl TestPrompt {
    fn new(prefix: &str) -> Self {
        check_env_vars();
        let repo = format!(
            "langstar-export-{}-{}",
            prefix,
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        );
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let auth = AuthConfig::from_env().expect("Auth config error");
        let client = LangchainClient::new(auth).expect("Client creation error");
        runtime
            .block_on(client.prompts().create_repo(
                &repo,
                Some(format!("Test prompt for {}", prefix)),
                Some("Read me before you run it.".to_string()),
                false,
                Some(vec!["support".to_string(), "langstar".to_string()]),
            ))
            .unwrap_or_else(|e| panic!("Failed to create the test prompt {}: {}", repo, e));
        Self {
            repo,
            runtime,
            client,
        }
    }

    /// Push one commit and return its hash.
    fn push(&self, manifest: Value, description: &str) -> String {
        let parent = self
            .runtime
            .block_on(
                self.client
                    .prompts()
                    .get_commit(OWNER, &self.repo, "latest"),
            )
            .ok()
            .map(|commit| commit.commit_hash);
        let response = self
            .runtime
            .block_on(self.client.prompts().push(
                OWNER,
                &self.repo,
                &CommitRequest {
                    manifest,
                    description: Some(description.to_string()),
                    parent_commit: parent,
                    example_run_ids: None,
                },
            ))
            .unwrap_or_else(|e| panic!("Failed to push to {}: {}", self.repo, e));
        response.commit.commit_hash
    }

    /// Put a tag on the commit with this hash.
    fn tag(&self, tag_name: &str, commit_hash: &str) {
        let commits = self
            .runtime
            .block_on(
                self.client
                    .prompts()
                    .list_commits(OWNER, &self.repo, Some(100), None),
            )
            .expect("Failed to list commits");
        let commit_id = commits
            .commits
            .iter()
            .find(|commit| commit.commit_hash == commit_hash)
            .map(|commit| commit.id.clone())
            .expect("the pushed commit is in the commit list");
        self.runtime
            .block_on(
                self.client
                    .prompts()
                    .create_tag(OWNER, &self.repo, tag_name, &commit_id),
            )
            .unwrap_or_else(|e| panic!("Failed to tag {}: {}", self.repo, e));
    }

    /// Assert that every field of the export file matches the SDK's read of the
    /// commit with this hash.
    fn assert_file_matches_sdk_read(&self, file: &Value, commit_hash: &str) {
        let prompts = self.client.prompts();
        let details = self
            .runtime
            .block_on(prompts.get(&format!("{}/{}", OWNER, self.repo)))
            .expect("Failed to read the repo");
        let commit = self
            .runtime
            .block_on(prompts.get_commit_with_model(OWNER, &self.repo, commit_hash))
            .expect("Failed to read the commit");
        let commit_tags: BTreeSet<String> = self
            .runtime
            .block_on(prompts.list_tags(OWNER, &self.repo))
            .expect("Failed to list tags")
            .into_iter()
            .filter(|tag| tag.commit_hash == commit_hash)
            .map(|tag| tag.tag_name)
            .collect();
        let repo_tags: BTreeSet<String> = details.tags.iter().cloned().collect();

        assert_eq!(file["kind"], "prompt");
        assert_eq!(file["format_version"], 1);
        assert_eq!(
            file["source"],
            json!({
                "workspace_id": details.tenant_id,
                "handle": self.repo,
                "commit_hash": commit.commit_hash,
            })
        );
        assert_eq!(
            file["repo"],
            json!({
                "handle": details.repo_handle,
                "description": details.description,
                "readme": details.readme,
                "tags": repo_tags,
                "is_public": details.is_public,
            })
        );
        let mut expected_commit = json!({
            "description": commit.description,
            "manifest": commit.manifest,
            "tags": commit_tags,
        });
        if let Some(model_config) = &commit.model_config {
            expected_commit["model_config"] = model_config.clone();
        }
        if let Some(model_provider) = &commit.model_provider {
            expected_commit["model_provider"] = json!(model_provider);
        }
        assert_eq!(file["commit"], expected_commit);
    }
}

impl Drop for TestPrompt {
    fn drop(&mut self) {
        if let Err(e) = self
            .runtime
            .block_on(self.client.prompts().delete(&self.repo))
        {
            eprintln!(
                "Warning: could not delete the test prompt {}: {}",
                self.repo, e
            );
        }
    }
}

/// The langstar binary with no config file, so it reads its credentials from
/// the environment alone.
fn langstar() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
    cmd.env("LANGSTAR_CONFIG_DIR", config_dir::empty_config_dir());
    cmd
}

/// Run `langstar prompt export` with `args` after the handle, writing to
/// `output`, and return the file it wrote.
fn export_to_file(handle: &str, output: &Path, args: &[&str]) -> String {
    langstar()
        .args(["prompt", "export", handle, "--output"])
        .arg(output)
        .args(args)
        .assert()
        .success()
        .stdout("");
    std::fs::read_to_string(output).expect("export wrote the file")
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).expect("the export file is JSON")
}

fn message(class: &str, template: &str) -> Value {
    json!({
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "chat", class],
        "kwargs": {
            "prompt": {
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
                "kwargs": {
                    "input_variables": ["question"],
                    "template": template,
                    "template_format": "f-string",
                },
            },
        },
    })
}

fn structured_prompt(schema: Value, method: &str) -> Value {
    json!({
        "lc": 1,
        "type": "constructor",
        "id": ["langchain_core", "prompts", "structured", "StructuredPrompt"],
        "kwargs": {
            "input_variables": ["question"],
            "messages": [message("HumanMessagePromptTemplate", "Answer {question}")],
            "schema_": schema,
            "structured_output_kwargs": {"method": method},
        },
    })
}

fn chat_prompt(template: &str) -> Value {
    json!({
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "chat", "ChatPromptTemplate"],
        "kwargs": {
            "input_variables": ["question"],
            "messages": [message("HumanMessagePromptTemplate", template)],
        },
    })
}

/// Task 5.1: a `json_schema` structured prompt saved without a model exports
/// every field the SDK reads, an empty `requires.secrets` and a null
/// `requires.model_class`.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_json_schema_prompt_without_a_model() {
    let prompt = TestPrompt::new("json-schema");
    let schema = json!({
        "title": "Answer",
        "type": "object",
        "properties": {"answer": {"type": "string"}},
        "required": ["answer"],
    });
    let hash = prompt.push(
        structured_prompt(schema.clone(), "json_schema"),
        "json schema",
    );

    let dir = tempfile::tempdir().unwrap();
    let file = parse(&export_to_file(
        &prompt.repo,
        &dir.path().join("p.json"),
        &[],
    ));

    prompt.assert_file_matches_sdk_read(&file, &hash);
    assert_eq!(file["commit"]["manifest"]["kwargs"]["schema_"], schema);
    assert_eq!(
        file["requires"],
        json!({"secrets": [], "model_class": null})
    );
}

/// Task 5.1: a `function_calling` structured prompt keeps its nested schema and
/// its method unchanged.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_function_calling_prompt_with_a_nested_schema() {
    let prompt = TestPrompt::new("function-calling");
    let schema = json!({
        "title": "Ticket",
        "type": "object",
        "properties": {
            "summary": {"type": "string"},
            "steps": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "action": {"type": "string"},
                        "owner": {
                            "type": "object",
                            "properties": {"name": {"type": "string"}},
                            "required": ["name"],
                        },
                    },
                    "required": ["action"],
                },
            },
        },
        "required": ["summary", "steps"],
    });
    let hash = prompt.push(
        structured_prompt(schema.clone(), "function_calling"),
        "function calling",
    );

    let dir = tempfile::tempdir().unwrap();
    let file = parse(&export_to_file(
        &prompt.repo,
        &dir.path().join("p.json"),
        &[],
    ));

    prompt.assert_file_matches_sdk_read(&file, &hash);
    let kwargs = &file["commit"]["manifest"]["kwargs"];
    assert_eq!(kwargs["schema_"], schema);
    assert_eq!(
        kwargs["structured_output_kwargs"]["method"],
        "function_calling"
    );
}

/// Task 5.1: a prompt with several messages and a placeholder keeps every
/// message.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_prompt_with_several_messages_and_a_placeholder() {
    let prompt = TestPrompt::new("messages");
    let manifest = json!({
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "chat", "ChatPromptTemplate"],
        "kwargs": {
            "input_variables": ["history", "question"],
            "messages": [
                message("SystemMessagePromptTemplate", "You answer {question} briefly."),
                {
                    "lc": 1,
                    "type": "constructor",
                    "id": ["langchain", "prompts", "chat", "MessagesPlaceholder"],
                    "kwargs": {"variable_name": "history", "optional": true},
                },
                message("HumanMessagePromptTemplate", "{question}"),
            ],
        },
    });
    let hash = prompt.push(manifest, "several messages");

    let dir = tempfile::tempdir().unwrap();
    let file = parse(&export_to_file(
        &prompt.repo,
        &dir.path().join("p.json"),
        &[],
    ));

    prompt.assert_file_matches_sdk_read(&file, &hash);
    let messages = file["commit"]["manifest"]["kwargs"]["messages"]
        .as_array()
        .expect("the manifest holds the messages");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[1]["kwargs"]["variable_name"], "history");
    assert_eq!(
        file["requires"],
        json!({"secrets": [], "model_class": null})
    );
}

/// Task 5.1: a prompt with a model and metadata lists the model's secret and
/// class, and holds no secret value.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_prompt_with_a_model_and_metadata() {
    let prompt = TestPrompt::new("model");
    let mut first = chat_prompt("Answer {question}");
    first["kwargs"]["metadata"] = json!({"team": "support", "lc_hub_repo": "export-test"});
    let manifest = json!({
        "lc": 1,
        "type": "constructor",
        "id": ["langsmith", "playground", "PromptPlayground"],
        "kwargs": {
            "first": first,
            "last": {
                "lc": 1,
                "type": "constructor",
                "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                "kwargs": {
                    "model": "gpt-4o-mini",
                    "temperature": 0.2,
                    "max_tokens": 1024,
                    "openai_api_key": {"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]},
                },
            },
        },
    });
    let hash = prompt.push(manifest, "with a model");

    let dir = tempfile::tempdir().unwrap();
    let text = export_to_file(&prompt.repo, &dir.path().join("p.json"), &[]);
    let file = parse(&text);

    prompt.assert_file_matches_sdk_read(&file, &hash);
    assert_eq!(
        file["requires"],
        json!({"secrets": ["OPENAI_API_KEY"], "model_class": "ChatOpenAI"})
    );
    assert_eq!(
        file["commit"]["manifest"]["kwargs"]["first"]["kwargs"]["metadata"]["team"],
        "support"
    );
    if let Ok(value) = std::env::var("OPENAI_API_KEY")
        && !value.is_empty()
    {
        assert!(!text.contains(&value), "the file holds the secret's value");
    }
}

/// Task 5.1: `--commit prod` exports the earlier commit the `prod` tag points at.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_the_commit_a_tag_points_at() {
    let prompt = TestPrompt::new("tagged");
    let earlier_manifest = chat_prompt("First version: {question}");
    let earlier = prompt.push(earlier_manifest.clone(), "first");
    prompt.tag("prod", &earlier);
    let later = prompt.push(chat_prompt("Second version: {question}"), "second");
    assert_ne!(earlier, later);

    let dir = tempfile::tempdir().unwrap();
    let file = parse(&export_to_file(
        &prompt.repo,
        &dir.path().join("p.json"),
        &["--commit", "prod"],
    ));

    assert_eq!(file["source"]["commit_hash"], earlier);
    prompt.assert_file_matches_sdk_read(&file, &earlier);
    assert_eq!(
        file["commit"]["manifest"]["kwargs"]["messages"][0]["kwargs"]["prompt"]["kwargs"]["template"],
        "First version: {question}"
    );
    assert_eq!(file["commit"]["tags"], json!(["prod"]));
}

/// Task 5.1: a handle the workspace has no prompt for exits with status 1,
/// names the handle and writes no file.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_a_handle_with_no_prompt() {
    check_env_vars();
    let handle = format!(
        "langstar-export-missing-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    );
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("p.json");

    let assert = langstar()
        .args(["prompt", "export", &handle, "--output"])
        .arg(&output)
        .assert()
        .code(1)
        .stdout("");
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).into_owned();
    assert!(stderr.contains(&handle), "{stderr}");
    assert!(!output.exists(), "export wrote {}", output.display());
}

/// Task 5.1: `my-prompt` and `-/my-prompt`, without `--`, write the same file.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_private_prompt_with_and_without_the_dash() {
    let prompt = TestPrompt::new("private");
    prompt.push(chat_prompt("Answer {question}"), "private");

    let dir = tempfile::tempdir().unwrap();
    let short = export_to_file(&prompt.repo, &dir.path().join("short.json"), &[]);
    let dashed = export_to_file(
        &format!("-/{}", prompt.repo),
        &dir.path().join("dashed.json"),
        &[],
    );
    assert_eq!(short, dashed);
}

/// Task 5.1: without `--output`, stdout holds one JSON document and the
/// progress message goes to stderr.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_to_stdout_holds_only_the_file() {
    let prompt = TestPrompt::new("stdout");
    let hash = prompt.push(chat_prompt("Answer {question}"), "stdout");

    let assert = langstar()
        .args(["prompt", "export", &prompt.repo])
        .assert()
        .success();
    let output = assert.get_output();
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();

    let file = parse(&stdout);
    assert_eq!(file["source"]["commit_hash"], hash);
    prompt.assert_file_matches_sdk_read(&file, &hash);
    assert!(
        stderr.contains("Reading"),
        "no progress message on stderr: {stderr}"
    );
}

/// Task 5.1: one commit with two tags exports to the same file twice.
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_export_a_commit_with_two_tags_twice() {
    let prompt = TestPrompt::new("two-tags");
    let hash = prompt.push(chat_prompt("Answer {question}"), "two tags");
    prompt.tag("staging", &hash);
    prompt.tag("prod", &hash);

    let dir = tempfile::tempdir().unwrap();
    let first = export_to_file(&prompt.repo, &dir.path().join("first.json"), &[]);
    let second = export_to_file(&prompt.repo, &dir.path().join("second.json"), &[]);
    assert_eq!(first, second);
    assert_eq!(parse(&first)["commit"]["tags"], json!(["prod", "staging"]));
}
