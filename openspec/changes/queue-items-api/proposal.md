# List and remove queue items through LangSmith's platform items API

Issue: [#765 Move annotation-queue run calls to the platform items API](https://github.com/codekiln/langstar/issues/765)

## Why

LangSmith deprecated the endpoints that `langstar queue add-runs`, `queue remove-run` and `queue items` call, and those commands stop working when LangSmith removes them. Once this change lands, `queue add-runs`, `queue remove-run` and `queue items` call LangSmith's queue items endpoints, so people who review runs in an annotation queue can keep adding, listing and removing runs from langstar. The new endpoints return each run's ID without its name or status, so `queue items` shows less about each run than it did.

## What Changes

- `langstar queue items` lists queue items: each item's short ID, short run ID, type, review section and the date it was added. A new `--status` option lists one review section: `needs-my-review`, `needs-others-review` or `archived`.
- `langstar queue remove-run` exits with an error that says the run is not in the queue when the queue holds no item for it.
- **BREAKING** for `langstar-sdk` callers: `get_run_from_annotation_queue` and `RunWithAnnotationQueueInfo` are removed, `add_runs_to_annotation_queue` and `delete_run_from_annotation_queue` return the created items and a `bool`, and the queue types read and write snake_case JSON.

## Capabilities

### Modified Capabilities

- [The queue specification](https://github.com/codekiln/langstar/blob/main/openspec/specs/queue/spec.md): `langstar queue items` lists queue items and takes `--status`, and `langstar queue remove-run` exits with an error when the run is not in the queue.

## Impact

- `sdk/src/client.rs` and `sdk/src/annotation_queues.rs`: the queue calls use `/api/v1/platform/annotation-queues/{queue_id}/items`.
- `cli/src/commands/queue.rs`: `items` gains `--status` and prints items; `remove-run` looks the run up by run ID.
- `cli/tests/queue_command_test.rs` and `sdk/tests/annotation_queue_items_test.rs`: tests for each scenario.
- `docs/queues.md`: the `items` and `remove-run` sections.
