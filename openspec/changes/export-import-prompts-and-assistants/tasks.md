# Tasks

Each numbered group is one pull request, in the order listed.

## 1. Check the API round trip before building on it

- [ ] 1.1 Extract the commit get, commit create, repo tags, workspace secrets and playground settings endpoints from `reference/openapi/langchain/langsmith/openapi.json` into `reference/api-specs/langsmith/` and add their jq queries to `FRAGMENTS.md`; verify by rerunning each query and getting the same file
- [ ] 1.2 Write `reference/research/807-openapi-validation.md`, recording from the live API: the manifest returned with `include=model` for a structured prompt with a model, whether pushing that manifest to a second workspace returns the same model, what pushing an identical manifest returns, what the Agent Server returns for `context`, and what `POST /api/v1/prompts/invoke_prompt` returns for a prompt with a model; verify that every claim in the report names the request that produced it

## 2. Test environments

- [ ] 2.1 Add the destination workspace and the model provider key to `fnox.toml`, the CI workflow and `docs/dev/environment-variables.md`, with placeholders only; verify with `mise run check` and a CI run that reads both. This task waits on codekiln to create a second workspace in the test organization and an OpenAI API key, and to store both in 1Password and the repository's CI secrets. The import tests need the workspace as a place to import into. The key lets the check in 7.3 run each imported structured prompt once against a small OpenAI model, which bills that account on every CI run; it is the one test that shows a copy still returns structured data, where every other test compares text.
- [ ] 2.2 Add a destination deployment fixture to `sdk/src/test_utils.rs` and `cli/tests/common/fixtures.rs` that returns a deployment with a reachable URL; verify with an integration test that lists assistants on it
- [ ] 2.3 Make `resolve_deployment_url` in `cli/src/deployment_utils.rs` return the URL of a deployment built from GitHub, which has no `source_config.custom_url`; verify by removing `#[ignore]` from `test_assistant_create_basic` in `cli/tests/assistant_command_test.rs` and running it against the test deployment

## 3. SDK reads and writes the fields export and import need

- [ ] 3.1 Add a commit read that sends `include=model` and returns `model_config` and `model_provider` to `sdk/src/prompts.rs`; verify with an httpmock test that asserts the query string and the parsed fields, and an integration test that reads a prompt with a model
- [ ] 3.2 Add listing and creating prompt tags to `sdk/src/prompts.rs`; verify with httpmock tests and an integration test that creates a tag and reads it back through the SDK
- [ ] 3.3 Add `context` and `version` to `Assistant`, `context` to `CreateAssistantRequest` and `UpdateAssistantRequest`, `assistant_id` and `if_exists` to `CreateAssistantRequest`, and a `graph_id` filter to `AssistantSearchRequest`, in `sdk/src/assistants.rs`; verify with httpmock tests of the request bodies and an integration test that creates an assistant with context, reads it back with its version, and finds it by graph ID

## 4. Code the prompt and assistant commands share: the export file, the comparison and the report

- [ ] 4.1 Add the export file types, with sorted keys and no export time, and the destination record file, to a new module in `cli/src/`; verify with unit tests that serialize the same value twice to identical text and that updating one destination leaves the others unchanged
- [ ] 4.2 Add the JSON comparison that drops server-set fields and reports differences by JSON Pointer; verify with unit tests on nested objects, arrays, missing keys and ignored fields
- [ ] 4.3 Add the shared result report and the exit statuses 0, 2 and 1, printing JSON to stdout and progress to stderr; verify with unit tests of the report and a CLI test that checks the exit status
- [ ] 4.4 Add the secret reference walk that lists secret names from a manifest; verify with unit tests on the manifest fixtures from `references/langsmith-sdk.md`

## 5. prompt export

- [ ] 5.1 Add `langstar prompt export` with `--commit`, `--output` and the scoping flags; verify with integration tests that create prompts through the SDK (a `json_schema` prompt, a `function_calling` prompt with a nested schema, a prompt with several messages and a placeholder, and a prompt with a model and metadata), export each through the CLI, and assert every field of the file against the SDK's read of the commit
- [ ] 5.2 Document `prompt export` in `docs/usage/` and the README; verify that each documented command runs as written

## 6. prompt compare and verify

- [ ] 6.1 Add `langstar prompt compare`; verify with integration tests that export a prompt, change its model temperature through the SDK, and assert the reported path, both values and exit status 2
- [ ] 6.2 Add `langstar prompt verify` with the secret check and the saved model configuration note, matching `requires.model_provider` against the class name at the end of each configuration's settings `id`; verify with integration tests against a workspace that lacks a listed secret and one that has it, and a test that a missing saved model configuration is reported and leaves the exit status at 0
- [ ] 6.3 Document compare and verify; verify that each documented command runs as written

## 7. prompt import

- [ ] 7.1 Add `langstar prompt import` with required `--workspace-id`, creating the repo, the commit and its tags, then reading the commit back; verify with integration tests that export from the test workspace, import into the destination workspace, and compare the full source and destination commits through the SDK with the model included, for every prompt shape from 5.1
- [ ] 7.2 Add `--update`, `--dry-run`, `--record` and the destination record; verify with integration tests that a repeated import creates no commit, that a changed destination is refused without `--update`, that `--update` adds one commit with the right parent, and that `--dry-run` writes nothing
- [ ] 7.3 Add the structured output check as a mise task that pulls the imported prompt with `include_model=True`, invokes it and validates the result against the schema; verify by running it from the import integration test for both methods
- [ ] 7.4 Document prompt import, the workspace scope of prompts and setting up destination credentials; verify that each documented command runs as written

## 8. assistant export

- [ ] 8.1 Add `langstar assistant export` with `--output` and `--prompt-ref`; verify with integration tests that create an assistant with configuration, context and metadata through the SDK and assert every field of the exported file, and an error test for a pointer that names nothing
- [ ] 8.2 Document assistant export; verify that each documented command runs as written

## 9. assistant compare and verify

- [ ] 9.1 Add `langstar assistant compare`; verify with an integration test that changes metadata through the SDK and asserts the reported path and exit status 2
- [ ] 9.2 Add `langstar assistant verify` with the graph check and the prompt check, which reads `GET /api/v1/commits/{owner}/{repo}/{tag}`; verify with integration tests for a missing graph ID and a missing prompt tag
- [ ] 9.3 Document assistant compare and verify; verify that each documented command runs as written

## 10. assistant import

- [ ] 10.1 Add `langstar assistant import` with creation, read-back and the destination record; verify with an integration test that imports into the destination deployment and checks name, graph ID, configuration, context, metadata and prompt references through the Agent Server API
- [ ] 10.2 Add `--update`, `--assistant-id` and `--dry-run`; verify with integration tests that a repeated import creates no assistant, that `--update` changes the recorded one, and that a deleted recorded assistant exits with status 1
- [ ] 10.3 Add an end-to-end test that imports prompts into the destination workspace and then an assistant that refers to them onto the destination deployment, and runs `assistant verify`; verify that it passes and that it fails after deleting one prompt tag
- [ ] 10.4 Document assistant import, the deployment scope of assistants, and how export and import relate to the `create`, `get` and `update` commands of [#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728); verify that each documented command runs as written

## Workflow follow-up

- Run `openspec validate export-import-prompts-and-assistants --strict`, then archive the change in the pull request for group 10.
