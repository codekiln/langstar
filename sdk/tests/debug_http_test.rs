/// Checks that an API key in a response reaches neither stderr nor a file when
/// `LANGSTAR_DEBUG_HTTP` is set.
///
/// `LANGSTAR_DEBUG_HTTP` used to make every successful SDK call print each response
/// to stderr and save it to a file, API keys included. This test fails if any code
/// brings that output back.
///
/// The test starts a second copy of its own test binary with `LANGSTAR_DEBUG_HTTP`,
/// `LANGSTAR_DEBUG_FILE`, `TMPDIR`, `TMP` and `TEMP` set, so it can read what that
/// copy prints to stderr.
use langstar_sdk::playground_settings::ListPlaygroundSettingsParams;
use langstar_sdk::{AuthConfig, LangchainClient};
use mockito::Server;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

const TEST_NAME: &str = "debug_http_writes_no_api_key";
const CHILD_MARKER: &str = "LANGSTAR_TEST_DEBUG_HTTP_CHILD";
const CHILD_DONE: &str = "debug-http-child-done";

/// Builds an `lsv2_pt_`-shaped key from pieces, so this file never holds a whole key for GitHub secret scanning to flag
fn fake_api_key() -> String {
    format!("lsv2_pt_{}_{}", "0123456789abcdef".repeat(2), "abcdef0123")
}

#[test]
fn debug_http_writes_no_api_key() {
    if std::env::var_os(CHILD_MARKER).is_some() {
        run_child();
        return;
    }

    let scratch = std::env::temp_dir().join(format!("langstar-debug-http-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let debug_file = scratch.join("debug.json");

    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST_NAME, "--nocapture", "--test-threads=1"])
        .env(CHILD_MARKER, "1")
        .env("LANGSTAR_DEBUG_HTTP", "1")
        .env("LANGSTAR_DEBUG_FILE", &debug_file)
        .env("TMPDIR", &scratch)
        .env("TMP", &scratch)
        .env("TEMP", &scratch)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let leftover: Vec<PathBuf> = std::fs::read_dir(&scratch)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    std::fs::remove_dir_all(&scratch).unwrap();

    assert!(output.status.success(), "child failed:\n{stdout}\n{stderr}");
    assert!(
        stdout.contains(CHILD_DONE),
        "child did not run the request:\n{stdout}"
    );
    assert!(
        !stderr.contains("lsv2_pt_"),
        "stderr holds the key:\n{stderr}"
    );
    assert!(
        !stdout.contains("lsv2_pt_"),
        "stdout holds the key:\n{stdout}"
    );
    assert!(leftover.is_empty(), "debug output written: {leftover:?}");
}

fn run_child() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/playground-settings")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                json!([{
                    "id": "550e8400-e29b-41d4-a716-446655440001",
                    "settings": {
                        "lc": 1,
                        "type": "constructor",
                        "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
                        "kwargs": { "model": "gpt-4o", "api_key": fake_api_key() }
                    },
                    "name": "Holds a key",
                    "created_at": "2024-01-15T10:30:00Z",
                    "updated_at": "2024-01-15T10:30:00Z"
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let auth = AuthConfig::new(Some("test-api-key".to_string()), None, None);
        let client = LangchainClient::with_base_urls(
            auth,
            server.url(),
            "http://mock-langgraph".to_string(),
            "http://mock-control-plane".to_string(),
        )
        .unwrap()
        .with_organization_id("test-org-id".to_string());

        let settings = client
            .list_playground_settings(ListPlaygroundSettingsParams::default())
            .await
            .unwrap();

        mock.assert_async().await;
        assert_eq!(settings.len(), 1);
    });
    println!("{CHILD_DONE}");
}
