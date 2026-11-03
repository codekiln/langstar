# Tasks

## 1. Call the platform items API

- [x] 1.1 Have the annotation-queue calls in `sdk/src/client.rs` use `/api/v1/platform/annotation-queues/{queue_id}/items`, with the item types in `sdk/src/annotation_queues.rs`, and remove `get_run_from_annotation_queue` and `RunWithAnnotationQueueInfo`
- [x] 1.2 Have `langstar queue items` print items and take `--status`
- [x] 1.3 Have `langstar queue remove-run` find the run's item by run ID and exit with an error when the queue holds no item for the run

## 2. Tests

- [x] 2.1 Mocked request-shape and paging tests in `sdk/tests/annotation_queue_items_test.rs`
- [x] 2.2 `test_queue_items_lifecycle`, `test_queue_items_help_lists_status_flag` and `test_queue_items_rejects_unknown_status` in `cli/tests/queue_command_test.rs`

## 3. Docs

- [x] 3.1 Update the `items` and `remove-run` sections of `docs/queues.md`

## 4. Archive

- [ ] 4.1 After this change merges, run `openspec validate queue-items-api --strict`, then `openspec archive queue-items-api`, which copies these requirements into `openspec/specs/queue/spec.md`
