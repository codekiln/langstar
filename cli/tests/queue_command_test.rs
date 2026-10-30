//! CLI tests for `langstar queue` run commands (`add-runs`, `items`, `remove-run`).
//!
//! The commands under test call LangSmith's platform items API,
//! `/api/v1/platform/annotation-queues/{queue_id}/items`, so the lifecycle test
//! asserts on the items the CLI and SDK get back, not only on exit codes.
//!
//! **Prerequisites for Integration Tests:**
//!
//! - `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID` and `LANGSMITH_WORKSPACE_ID`
//!
//! Run with: `cargo test --test queue_command_test --features integration-tests`

#[path = "common/config_dir.rs"]
mod config_dir;

use assert_cmd::Command;
use chrono::Utc;
use langstar_sdk::{
    AnnotationQueueItemCountStatus, AnnotationQueueItemStatus, AnnotationQueueItemType, AuthConfig,
    LangchainClient, ListAnnotationQueueItemsParams, PatchAnnotationQueueItemRequest,
    ProjectCreate,
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use uuid::Uuid;

fn langstar_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_langstar"));
    cmd.env("LANGSTAR_CONFIG_DIR", config_dir::empty_config_dir());
    cmd
}

#[test]
fn test_queue_items_help_lists_status_flag() {
    let output = langstar_cmd()
        .args(["queue", "items", "--help"])
        .output()
        .expect("Failed to execute CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--status"),
        "help should list --status: {stdout}"
    );
}

#[test]
fn test_queue_items_rejects_unknown_status() {
    let output = langstar_cmd()
        .args([
            "queue",
            "items",
            "12345678-1234-1234-1234-123456789012",
            "--status",
            "bogus",
        ])
        .output()
        .expect("Failed to execute CLI");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("bogus"));
}

/// Deletes the test project, and the runs in it, when the test ends.
struct ProjectCleanup {
    client: LangchainClient,
    project_id: Uuid,
}

impl Drop for ProjectCleanup {
    fn drop(&mut self) {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        if let Err(e) = runtime.block_on(self.client.delete_project(self.project_id)) {
            eprintln!("[CLEANUP] Failed to delete test project: {}", e);
        }
    }
}

/// Deletes the test queue when the test ends, if the test did not already.
struct QueueCleanup {
    client: LangchainClient,
    queue_id: Uuid,
}

impl Drop for QueueCleanup {
    fn drop(&mut self) {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        // Ignore the result: the test deletes the queue itself on the success path.
        let _ = runtime.block_on(self.client.delete_annotation_queue(self.queue_id));
    }
}

/// Posts one root run to the ingest endpoint, `POST /api/v1/runs`, and returns its ID.
async fn ingest_run(client: &LangchainClient, project_name: &str, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let start = Utc::now() - chrono::Duration::minutes(5);
    let end = start + chrono::Duration::milliseconds(250);
    let dotted_order = format!("{}{}", start.format("%Y%m%dT%H%M%S%6fZ"), id);

    let body = json!({
        "id": id,
        "trace_id": id,
        "dotted_order": dotted_order,
        "name": name,
        "run_type": "chain",
        "session_name": project_name,
        "start_time": start.to_rfc3339(),
        "end_time": end.to_rfc3339(),
        "inputs": {"question": "seed"},
        "outputs": {"answer": "seed"},
    });

    let response = client
        .langsmith_post("/api/v1/runs")
        .expect("Failed to build ingest request")
        .json(&body)
        .send()
        .await
        .expect("Failed to send ingest request");
    assert!(
        response.status().is_success(),
        "Ingesting run '{}' failed with HTTP {}",
        name,
        response.status()
    );
    id
}

/// Runs a CLI command that must succeed and returns its stdout.
fn run_ok(args: &[&str]) -> String {
    let output = langstar_cmd()
        .args(args)
        .output()
        .expect("Failed to execute CLI");
    assert!(
        output.status.success(),
        "langstar {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// `queue items --json` as a list of items.
fn queue_items_json(queue_id: &str) -> Vec<Value> {
    let stdout = run_ok(&["queue", "items", queue_id, "--json"]);
    serde_json::from_str(&stdout).expect("queue items --json should print a JSON array")
}

fn run_ids(items: &[Value]) -> Vec<&str> {
    let mut ids: Vec<&str> = items
        .iter()
        .map(|i| i["run_id"].as_str().expect("item should carry a run_id"))
        .collect();
    ids.sort();
    ids
}

/// Pattern: CREATE queue → ADD runs (CLI) → READ items (CLI, SDK) → UPDATE → REMOVE → DELETE
#[test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
fn test_queue_items_lifecycle() {
    let _api_key = std::env::var("LANGSMITH_API_KEY")
        .expect("LANGSMITH_API_KEY must be set for integration tests");
    let _org_id = std::env::var("LANGSMITH_ORGANIZATION_ID")
        .expect("LANGSMITH_ORGANIZATION_ID must be set for integration tests");
    let _workspace_id = std::env::var("LANGSMITH_WORKSPACE_ID")
        .expect("LANGSMITH_WORKSPACE_ID must be set for integration tests");

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    let auth = AuthConfig::from_env().expect("Auth config required");
    let client = LangchainClient::new(auth).expect("SDK client required");
    let suffix = &Uuid::new_v4().to_string()[..12];

    // ── CREATE: a project with two runs, and a queue ──
    let project_name = format!("test-queue-items-{suffix}");
    let project = runtime
        .block_on(client.create_project(ProjectCreate {
            name: Some(project_name.clone()),
            description: Some("Test project for queue items lifecycle".to_string()),
            ..Default::default()
        }))
        .expect("Failed to create test project");
    let _project_cleanup = ProjectCleanup {
        client: client.clone(),
        project_id: project.id,
    };
    let (run_a, run_b) = runtime.block_on(async {
        (
            ingest_run(&client, &project_name, "queue-run-a").await,
            ingest_run(&client, &project_name, "queue-run-b").await,
        )
    });

    let queue_name = format!("test-queue-{suffix}");
    let created: Value = serde_json::from_str(&run_ok(&[
        "queue",
        "create",
        "--name",
        &queue_name,
        "--json",
    ]))
    .expect("queue create --json should print JSON");
    let queue_id = created["id"]
        .as_str()
        .expect("queue should have an id")
        .to_string();
    let queue_uuid = Uuid::parse_str(&queue_id).unwrap();
    let _queue_cleanup = QueueCleanup {
        client: client.clone(),
        queue_id: queue_uuid,
    };
    println!("[CREATE] Created test queue and 2 runs");

    // ── Wait until both runs are queryable: the items API looks runs up by ID ──
    let project_id = project.id.to_string();
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        let stdout = run_ok(&[
            "runs",
            "query",
            "-p",
            &project_id,
            "--since",
            "1h",
            "-o",
            "json",
        ]);
        let runs: Vec<Value> = serde_json::from_str(&stdout).expect("runs query should print JSON");
        if runs.len() >= 2 {
            break;
        }
        assert!(
            Instant::now() <= deadline,
            "only {} of 2 runs visible after 180s",
            runs.len()
        );
        std::thread::sleep(Duration::from_secs(5));
    }

    // ── ADD (CLI) ──
    let (a, b) = (run_a.to_string(), run_b.to_string());
    let stdout = run_ok(&["queue", "add-runs", &queue_id, &a, &b]);
    assert!(stdout.contains("Added 2 runs"), "add-runs output: {stdout}");

    // ── READ (CLI): both runs show up as RUN items, wait for them to be visible ──
    let deadline = Instant::now() + Duration::from_secs(120);
    let items = loop {
        let items = queue_items_json(&queue_id);
        if items.len() >= 2 {
            break items;
        }
        assert!(
            Instant::now() <= deadline,
            "only {} of 2 items visible after 120s",
            items.len()
        );
        std::thread::sleep(Duration::from_secs(3));
    };
    let mut expected = vec![a.as_str(), b.as_str()];
    expected.sort();
    assert_eq!(run_ids(&items), expected);
    for item in &items {
        assert_eq!(item["item_type"], "RUN");
        assert!(
            ["needs_my_review", "needs_others_review", "archived"]
                .contains(&item["status"].as_str().expect("item should carry a status")),
            "status should name a review section: {item}"
        );
        assert_eq!(item["queue_id"].as_str(), Some(queue_id.as_str()));
        assert!(item["id"].is_string(), "item should carry a queue item id");
    }
    println!("[READ] queue items returned {} items", items.len());

    // ── READ (SDK): count and placement agree with the list ──
    let first_item_id = items[0]["id"].as_str().unwrap().to_string();
    let counted = runtime
        .block_on(
            client
                .count_annotation_queue_items(queue_uuid, AnnotationQueueItemStatus::NeedsMyReview),
        )
        .expect("count failed");
    let others =
        runtime
            .block_on(client.count_annotation_queue_items(
                queue_uuid,
                AnnotationQueueItemStatus::NeedsOthersReview,
            ))
            .expect("count failed");
    assert_eq!(counted + others, 2, "count should match the two items");
    let total = runtime
        .block_on(
            client.count_annotation_queue_items(queue_uuid, AnnotationQueueItemCountStatus::All),
        )
        .expect("count all failed");
    assert_eq!(total, 2, "count with status all should cover every item");

    let page = runtime
        .block_on(client.list_annotation_queue_items(
            queue_uuid,
            ListAnnotationQueueItemsParams::new(AnnotationQueueItemStatus::NeedsMyReview),
            None,
        ))
        .expect("list failed");
    assert_eq!(page.items.len() as u64, counted);

    let item_uuid = Uuid::parse_str(&first_item_id).unwrap();
    let placement = runtime
        .block_on(client.get_annotation_queue_item_placement(queue_uuid, item_uuid))
        .expect("placement failed");
    assert!(
        placement.section.is_some(),
        "placement should name a section"
    );
    assert_eq!(placement.item_type, Some(AnnotationQueueItemType::Run));

    // ── UPDATE (SDK): mark one item reviewed ──
    let reviewed_at = Utc::now();
    let patched = runtime
        .block_on(client.update_annotation_queue_item(
            queue_uuid,
            item_uuid,
            PatchAnnotationQueueItemRequest {
                last_reviewed_time: Some(reviewed_at),
                ..Default::default()
            },
        ))
        .expect("patch failed");
    assert_eq!(patched.id, item_uuid);
    assert!(patched.last_reviewed_time.is_some());

    // ── REMOVE (CLI): one run, then the same run again ──
    let stdout = run_ok(&["queue", "remove-run", &queue_id, &a]);
    assert!(
        stdout.contains("Removed run"),
        "remove-run output: {stdout}"
    );
    let items = queue_items_json(&queue_id);
    assert_eq!(run_ids(&items), vec![b.as_str()]);

    let output = langstar_cmd()
        .args(["queue", "remove-run", &queue_id, &a])
        .output()
        .expect("Failed to execute CLI");
    assert!(
        !output.status.success(),
        "removing a missing run should fail"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not in queue"));

    // ── DELETE ──
    run_ok(&["queue", "delete", &queue_id, "--force"]);
    let output = langstar_cmd()
        .args(["queue", "get", &queue_id])
        .output()
        .expect("Failed to execute CLI");
    assert!(
        !output.status.success(),
        "queue should be gone after delete"
    );
}
