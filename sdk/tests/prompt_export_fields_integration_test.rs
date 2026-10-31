//! Integration tests for the prompt fields export and import read and write
//!
//! These tests cover the commit read that asks for the model, the commit list and the
//! commit tags, the commit description, and the repo update, against the live LangSmith
//! API. They need `LANGSMITH_API_KEY`, and the workspace `LANGSMITH_WORKSPACE_ID` names.
//!
//! Run with: cargo test --features integration-tests -p langstar-sdk --test prompt_export_fields_integration_test
//!
//! The SDK Integration Tests job in `.github/workflows/ci.yml` runs them on every pull
//! request, and `mise run check` runs them locally.
//!
//! The tasks each test names live in
//! `openspec/changes/export-import-prompts-and-assistants/tasks.md`.

use langstar_sdk::prompts::{CommitRequest, UpdateRepoRequest};
use langstar_sdk::{AuthConfig, LangchainClient};
use serde_json::json;

/// The "-" owner, which names the authenticated user's own private prompts
const TEST_OWNER: &str = "-";

/// A public prompt saved with a model, pinned to one commit so its manifest never changes.
///
/// No prompt in the test workspace carries a model, as
/// `reference/research/807-openapi-validation.md` records. That report read this public
/// prompt instead, which a person saved with a model in the web interface, so these
/// constants name it and the commit the report read.
const MODEL_PROMPT_OWNER: &str = "langchain-ai";
const MODEL_PROMPT_REPO: &str = "chain-of-density";
const MODEL_PROMPT_COMMIT: &str =
    "8c0ba491ce9b01f707e56db935d7eeee7aa6ff29c04258b458928cf7c8ebe35b";

async fn create_integration_test_client() -> LangchainClient {
    let auth = AuthConfig::from_env().expect("LANGSMITH_API_KEY must be set");
    auth.require_langsmith_key()
        .expect("LANGSMITH_API_KEY is required");

    let mut client = LangchainClient::new(auth).expect("Failed to create client");

    match client.get_current_organization().await {
        Ok(org) => {
            if let Some(org_id) = org.id {
                client = client.with_organization_id(org_id);
            }
        }
        Err(e) => eprintln!("Warning: could not fetch the organization: {:?}", e),
    }

    client
}

/// A repo name no other test run uses
fn test_repo_name(suffix: &str) -> String {
    format!(
        "langstar-export-fields-{}-{}",
        suffix,
        &uuid::Uuid::new_v4().to_string()[..8]
    )
}

/// A small prompt manifest with one template variable
fn manifest(template: &str) -> serde_json::Value {
    json!({
        "lc": 1,
        "type": "constructor",
        "id": ["langchain", "prompts", "prompt", "PromptTemplate"],
        "kwargs": {
            "input_variables": ["question"],
            "template": template,
            "template_format": "f-string"
        }
    })
}

async fn create_test_repo(client: &LangchainClient, repo: &str) {
    client
        .prompts()
        .create_repo(
            repo,
            Some("Test prompt for the export fields".to_string()),
            None,
            false,
            None,
        )
        .await
        .unwrap_or_else(|e| panic!("Failed to create the test repo {}: {:?}", repo, e));
}

async fn delete_test_repo(client: &LangchainClient, repo: &str) {
    if let Err(e) = client.prompts().delete(repo).await {
        eprintln!("Warning: could not delete the test repo {}: {:?}", repo, e);
    }
}

/// Task 3.1, "Add the commit read that asks for the model": the commit read with
/// `include=model` returns the model inside the manifest.
///
/// The chat model constructor sits at `manifest.kwargs.last.kwargs.bound`, behind a
/// `RunnableBinding`, which is where `prompt export` reads the model class from.
#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
async fn test_commit_read_with_model_returns_the_bound_chat_model() {
    let client = create_integration_test_client().await;

    let with_model = client
        .prompts()
        .get_commit_with_model(MODEL_PROMPT_OWNER, MODEL_PROMPT_REPO, MODEL_PROMPT_COMMIT)
        .await
        .expect("the commit read with the model should succeed");

    assert_eq!(
        with_model.model_provider.as_deref(),
        Some("ChatOpenAI"),
        "the pinned commit is saved with a ChatOpenAI model"
    );
    assert!(
        with_model.model_config.is_some(),
        "a commit with a model carries model_config beside the manifest"
    );

    let last = &with_model.manifest["kwargs"]["last"];
    let class_of = |node: &serde_json::Value| {
        node["id"]
            .as_array()
            .and_then(|id| id.last())
            .and_then(|name| name.as_str())
            .map(str::to_string)
    };
    assert_eq!(
        class_of(last).as_deref(),
        Some("RunnableBinding"),
        "a RunnableBinding wraps the chat model, so kwargs.last is not the constructor"
    );
    assert_eq!(
        class_of(&last["kwargs"]["bound"]).as_deref(),
        Some("ChatOpenAI"),
        "the chat model constructor sits at kwargs.last.kwargs.bound"
    );
    assert_eq!(
        with_model.model_provider.as_deref(),
        class_of(&last["kwargs"]["bound"]).as_deref(),
        "model_provider names the bound constructor's class"
    );

    // Without the parameter the manifest holds the prompt alone.
    let without_model = client
        .prompts()
        .get_commit(MODEL_PROMPT_OWNER, MODEL_PROMPT_REPO, MODEL_PROMPT_COMMIT)
        .await
        .expect("the plain commit read should succeed");
    assert_eq!(
        class_of(&without_model.manifest).as_deref(),
        Some("ChatPromptTemplate"),
        "without include=model the manifest is the prompt on its own"
    );
}

/// Tasks 3.1 and 3.4, "Add the repo details and the commit description": a commit
/// carries its description, and a repo update changes the repo details the export file
/// holds.
#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
async fn test_commit_description_and_repo_details_round_trip() {
    let client = create_integration_test_client().await;
    let repo = test_repo_name("repo-details");
    create_test_repo(&client, &repo).await;

    let description = "Ask the question plainly";
    client
        .prompts()
        .push(
            TEST_OWNER,
            &repo,
            &CommitRequest {
                manifest: manifest("Answer this question: {question}"),
                description: Some(description.to_string()),
                parent_commit: None,
                example_run_ids: None,
            },
        )
        .await
        .expect("the push should succeed");

    let commit = client
        .prompts()
        .get_commit_with_model(TEST_OWNER, &repo, "latest")
        .await
        .expect("the commit read should succeed");
    assert_eq!(
        commit.description.as_deref(),
        Some(description),
        "the commit read returns the description the push sent"
    );
    assert!(
        commit.model_config.is_none() && commit.model_provider.is_none(),
        "a prompt saved without a model has neither field"
    );

    let updated = client
        .prompts()
        .update_repo(
            TEST_OWNER,
            &repo,
            &UpdateRepoRequest {
                description: Some("Answers support questions".to_string()),
                readme: Some("# Support\n\nWhat this prompt does.".to_string()),
                tags: Some(vec!["support".to_string(), "triage".to_string()]),
                is_public: Some(false),
            },
        )
        .await
        .expect("the repo update should succeed");
    assert_eq!(
        updated.description.as_deref(),
        Some("Answers support questions")
    );

    let read_back = client
        .prompts()
        .get(&format!("{}/{}", TEST_OWNER, repo))
        .await
        .expect("the prompt read should succeed");
    assert_eq!(
        read_back.description.as_deref(),
        Some("Answers support questions")
    );
    assert_eq!(
        read_back.readme.as_deref(),
        Some("# Support\n\nWhat this prompt does.")
    );
    let mut tags = read_back.tags.clone();
    tags.sort();
    assert_eq!(tags, vec!["support".to_string(), "triage".to_string()]);
    assert!(!read_back.is_public, "the prompt stays private");
    assert!(
        read_back.tenant_id.is_some(),
        "the repo read names the workspace that holds the prompt"
    );

    delete_test_repo(&client, &repo).await;
}

/// Task 3.2, "Add the commit list and the commit tag calls": the commit list gives each
/// commit's ID, and the tag calls move a tag from one commit to another and delete it.
#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
async fn test_commit_tags_move_between_commits_and_delete() {
    let client = create_integration_test_client().await;
    let repo = test_repo_name("tags");
    create_test_repo(&client, &repo).await;

    let first = client
        .prompts()
        .push(
            TEST_OWNER,
            &repo,
            &CommitRequest {
                manifest: manifest("First: {question}"),
                description: Some("The first commit".to_string()),
                parent_commit: None,
                example_run_ids: None,
            },
        )
        .await
        .expect("the first push should succeed");
    let first_commit_id = first
        .commit
        .id
        .clone()
        .expect("the push response names the new commit's ID");

    let second = client
        .prompts()
        .push(
            TEST_OWNER,
            &repo,
            &CommitRequest {
                manifest: manifest("Second: {question}"),
                description: Some("The second commit".to_string()),
                parent_commit: Some(first.commit.commit_hash.clone()),
                example_run_ids: None,
            },
        )
        .await
        .expect("the second push should succeed");

    client
        .prompts()
        .create_tag(TEST_OWNER, &repo, "prod", &first_commit_id)
        .await
        .expect("the tag create should succeed");

    // The tag endpoints take a commit's ID, and the commit read returns only its hash,
    // so the commit list is where a caller finds the ID of a commit it knows by hash.
    let commits = client
        .prompts()
        .list_commits(TEST_OWNER, &repo, Some(10), None)
        .await
        .expect("the commit list should succeed");
    let second_commit_id = commits
        .commits
        .iter()
        .find(|c| c.commit_hash == second.commit.commit_hash)
        .map(|c| c.id.clone())
        .expect("the commit list holds the second commit");

    let moved = client
        .prompts()
        .move_tag(TEST_OWNER, &repo, "prod", &second_commit_id)
        .await
        .expect("the tag move should succeed");
    assert_eq!(moved.commit_id, second_commit_id);

    let tags = client
        .prompts()
        .list_tags(TEST_OWNER, &repo)
        .await
        .expect("the tag list should succeed");
    let prod = tags
        .iter()
        .find(|t| t.tag_name == "prod")
        .expect("the tag list holds the prod tag");
    assert_eq!(
        prod.commit_hash, second.commit.commit_hash,
        "the prod tag now points at the second commit"
    );

    client
        .prompts()
        .delete_tag(TEST_OWNER, &repo, "prod")
        .await
        .expect("the tag delete should succeed");

    let tags_after = client
        .prompts()
        .list_tags(TEST_OWNER, &repo)
        .await
        .expect("the tag list should succeed");
    assert!(
        !tags_after.iter().any(|t| t.tag_name == "prod"),
        "the tag list no longer holds the deleted tag"
    );

    delete_test_repo(&client, &repo).await;
}
