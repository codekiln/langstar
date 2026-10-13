//! HTTP-mocked tests for the annotation queue items methods, which call
//! `/api/v1/platform/annotation-queues/{queue_id}/items`.

use langstar_sdk::{
    AnnotationQueueItemInput, AnnotationQueueItemStatus, AnnotationQueueItemType, AuthConfig,
    LangchainClient, ListAnnotationQueueItemsParams,
};
use mockito::{Matcher, Server};
use serde_json::json;
use uuid::Uuid;

const QUEUE: &str = "12345678-1234-1234-1234-123456789012";

fn create_test_client(server_url: &str) -> LangchainClient {
    let auth = AuthConfig::new(Some("test_langsmith_key".to_string()), None, None);
    LangchainClient::with_base_urls(
        auth,
        server_url.to_string(),
        "https://api.langgraph.cloud".to_string(),
        "https://api.host.langchain.com".to_string(),
    )
    .expect("Failed to create test client")
}

fn list_item(id: Uuid, run_id: Uuid) -> serde_json::Value {
    json!({
        "id": id,
        "queue_id": QUEUE,
        "item_type": "RUN",
        "run_id": run_id,
        "added_at": "2026-01-01T00:00:00Z",
        "last_reviewed_time": null
    })
}

#[tokio::test]
async fn test_add_runs_sends_run_items_and_returns_created_items() {
    let mut server = Server::new_async().await;
    let run_id = Uuid::new_v4();
    let item_id = Uuid::new_v4();

    let mock = server
        .mock(
            "POST",
            format!("/api/v1/platform/annotation-queues/{QUEUE}/items").as_str(),
        )
        .match_body(Matcher::Json(json!({
            "items": [{"item_type": "RUN", "run_id": run_id}]
        })))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(json!({"items": [list_item(item_id, run_id)]}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());
    let items = client
        .add_runs_to_annotation_queue(Uuid::parse_str(QUEUE).unwrap(), vec![run_id])
        .await
        .expect("add failed");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, item_id);
    assert_eq!(items[0].run_id, Some(run_id));
    assert_eq!(items[0].item_type, Some(AnnotationQueueItemType::Run));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_add_items_splits_requests_at_100() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock(
            "POST",
            format!("/api/v1/platform/annotation-queues/{QUEUE}/items").as_str(),
        )
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(json!({"items": []}).to_string())
        .expect(2)
        .create_async()
        .await;

    let client = create_test_client(&server.url());
    let items = (0..101)
        .map(|_| AnnotationQueueItemInput {
            item_type: Some(AnnotationQueueItemType::Run),
            run_id: Some(Uuid::new_v4()),
            ..Default::default()
        })
        .collect();
    client
        .add_items_to_annotation_queue(Uuid::parse_str(QUEUE).unwrap(), items)
        .await
        .expect("add failed");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_items_sends_status_and_cursor() {
    let mut server = Server::new_async().await;
    let (item_id, run_id) = (Uuid::new_v4(), Uuid::new_v4());

    let mock = server
        .mock(
            "GET",
            format!("/api/v1/platform/annotation-queues/{QUEUE}/items").as_str(),
        )
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("status".into(), "archived".into()),
            Matcher::UrlEncoded("cursor".into(), "abc".into()),
            Matcher::UrlEncoded("page_size".into(), "5".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"items": [list_item(item_id, run_id)], "next_cursor": "def"}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());
    let mut params = ListAnnotationQueueItemsParams::new(AnnotationQueueItemStatus::Archived);
    params.page_size = Some(5);
    let page = client
        .list_annotation_queue_items(Uuid::parse_str(QUEUE).unwrap(), params, Some("abc"))
        .await
        .expect("list failed");

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].run_id, Some(run_id));
    assert_eq!(page.next_cursor.as_deref(), Some("def"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_count_items_returns_count() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock(
            "GET",
            format!("/api/v1/platform/annotation-queues/{QUEUE}/items/count").as_str(),
        )
        .match_query(Matcher::UrlEncoded(
            "status".into(),
            "needs_my_review".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"count": 7}).to_string())
        .create_async()
        .await;

    let client = create_test_client(&server.url());
    let count = client
        .count_annotation_queue_items(
            Uuid::parse_str(QUEUE).unwrap(),
            AnnotationQueueItemStatus::NeedsMyReview,
        )
        .await
        .expect("count failed");
    assert_eq!(count, 7);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_delete_run_finds_item_by_run_id_across_sections() {
    let mut server = Server::new_async().await;
    let (item_id, run_id) = (Uuid::new_v4(), Uuid::new_v4());
    let list_path = format!("/api/v1/platform/annotation-queues/{QUEUE}/items");

    // The run sits in the archived section; the other two sections hold another run.
    let mut list_mocks = Vec::new();
    for status in ["needs_my_review", "needs_others_review"] {
        list_mocks.push(
            server
                .mock("GET", list_path.as_str())
                .match_query(Matcher::UrlEncoded("status".into(), status.into()))
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(
                    json!({"items": [list_item(Uuid::new_v4(), Uuid::new_v4())]}).to_string(),
                )
                .create_async()
                .await,
        );
    }
    list_mocks.push(
        server
            .mock("GET", list_path.as_str())
            .match_query(Matcher::UrlEncoded("status".into(), "archived".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"items": [list_item(item_id, run_id)]}).to_string())
            .create_async()
            .await,
    );
    let delete = server
        .mock("POST", format!("{list_path}/delete").as_str())
        .match_body(Matcher::Json(json!({"item_ids": [item_id]})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create_async()
        .await;

    let client = create_test_client(&server.url());
    let removed = client
        .delete_run_from_annotation_queue(Uuid::parse_str(QUEUE).unwrap(), run_id)
        .await
        .expect("delete failed");

    assert!(removed);
    delete.assert_async().await;
    for mock in list_mocks {
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn test_delete_run_returns_false_when_run_not_in_queue() {
    let mut server = Server::new_async().await;
    let list_path = format!("/api/v1/platform/annotation-queues/{QUEUE}/items");
    let list = server
        .mock("GET", list_path.as_str())
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"items": []}).to_string())
        .expect(3)
        .create_async()
        .await;
    let delete = server
        .mock("POST", format!("{list_path}/delete").as_str())
        .expect(0)
        .create_async()
        .await;

    let client = create_test_client(&server.url());
    let removed = client
        .delete_run_from_annotation_queue(Uuid::parse_str(QUEUE).unwrap(), Uuid::new_v4())
        .await
        .expect("delete failed");

    assert!(!removed);
    list.assert_async().await;
    delete.assert_async().await;
}
