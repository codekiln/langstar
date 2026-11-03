# List and remove queue items through LangSmith's platform items API

Issue: [#765 Move annotation-queue run calls to the platform items API](https://github.com/codekiln/langstar/issues/765)

## Why

LangSmith deprecated the annotation-queue run endpoints that `langstar queue add-runs`, `queue remove-run` and `queue items` call. The platform items endpoints that replace them return queue items rather than runs, so `queue items` can no longer show a run's name or status, and `queue remove-run` has to find the run's queue item before it can delete it.

## What Changes

- `langstar queue items` lists queue items: each item's short ID, short run ID, type, review section and the date it was added. A new `--status` option lists one review section: `needs-my-review`, `needs-others-review` or `archived`.
- `langstar queue remove-run` exits with an error that says the run is not in the queue when the queue holds no item for it.
- **BREAKING** for `langstar-sdk` callers: `get_run_from_annotation_queue` and `RunWithAnnotationQueueInfo` are removed, `add_runs_to_annotation_queue` and `delete_run_from_annotation_queue` return the created items and a `bool`, and the queue types read and write snake_case JSON.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `queue`: the requirement to list a queue's runs becomes a requirement to list its items, with `--status`, and removing a run that is not in the queue becomes an error.

## Impact

- `sdk/src/client.rs` and `sdk/src/annotation_queues.rs`: the queue calls use `/api/v1/platform/annotation-queues/{queue_id}/items`.
- `cli/src/commands/queue.rs`: `items` gains `--status` and prints items; `remove-run` looks the run up by run ID.
- `cli/tests/queue_command_test.rs` and `sdk/tests/annotation_queue_items_test.rs`: tests for each scenario.
- `docs/queues.md`: the `items` and `remove-run` sections.
