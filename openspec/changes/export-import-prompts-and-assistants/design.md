# Design

## Context

`langstar prompt push` and `pull` in `cli/src/commands/prompt.rs` handle one shape of prompt: a single human message, with an optional schema from [#402 structured output prompts](https://github.com/codekiln/langstar/issues/402). `PromptClient` in `sdk/src/prompts.rs` keeps the manifest as raw JSON and has a typed `StructuredPrompt` view that holds messages, the schema and the method. No call asks LangSmith for the model, so neither command sees one.

`langstar assistant` in `cli/src/commands/assistant.rs` takes `--deployment` on every subcommand and sends requests to that deployment's Agent Server. `AssistantClient` in `sdk/src/assistants.rs` has no `context` field, and its create request cannot set `assistant_id` or `if_exists`.

Langstar reads one API key and one workspace at a time, from flags, environment variables or the config file. Every error exits with status 1.

## Goals / Non-Goals

**Goals:**

- Carry every field of a prompt commit and an assistant through a file and back without loss.
- Make a repeated import of the same file safe: it finds its earlier copy and changes nothing until the user asks.
- Give users one way to check a copy: the same report shape and exit statuses for both resource types.

**Non-Goals:**

- Copy a prompt or assistant between environments in one command.
- Export or import every prompt in a workspace, or every assistant on a deployment, in one run.
- Redesign `prompt create`, `get` and `update`, which [#728 Redesign prompt commands around create/get/update](https://github.com/codekiln/langstar/issues/728) covers.

## Decisions

### A file carries each resource between environments

Export runs with the source environment's credentials and writes a file. Import runs later with the destination's credentials and reads it. Each run uses langstar's existing settings, so a user moving a prompt to another organization sets a different `LANGSMITH_API_KEY` for the import. Langstar needs no second set of credentials and no profiles. The file sits in Git between the two runs, so a reviewer sees the prompt or assistant change in a pull request.

The LangSmith Go CLI follows the same file workflow, as `references/langsmith-cli.md` shows. Langstar differs from it on purpose. Langstar's export asks for the model and the Go CLI's pull does not. Langstar's import reads the file export writes, while the Go CLI's push wants the bare manifest and rejects its own pull output. Langstar's import also requires an explicit `--workspace-id`, so the destination is named on the command line and never comes from a leftover environment variable.

### The export file keeps the manifest LangSmith returned

Export stores the manifest as raw JSON. The typed `StructuredPrompt` struct cannot hold a model binding, placeholders, several message types or partial variables, so routing through it would drop them. Langstar reads the manifest through typed views only to fill in `requires`.

One prompt export looks like this:

```json
{
  "kind": "prompt",
  "format_version": 1,
  "source": { "workspace_id": "<source-workspace-id>", "handle": "my-prompt", "commit_hash": "<hash>", "ref": "prod" },
  "repo": { "handle": "my-prompt", "description": "...", "readme": "...", "tags": ["support"], "is_public": false },
  "commit": { "manifest": { "lc": 1, "type": "constructor", "id": ["langchain", "schema", "runnable", "RunnableSequence"], "kwargs": { "first": {}, "last": {} } }, "model_config": {}, "model_provider": "ChatOpenAI", "tags": ["prod"] },
  "requires": { "secrets": ["OPENAI_API_KEY"], "model_provider": "ChatOpenAI" }
}
```

An assistant export has `kind: "assistant"`, a `source` with the deployment, assistant ID and version, an `assistant` object with `name`, `description`, `graph_id`, `config`, `context` and `metadata`, and `requires.prompts`.

The file holds no export time and langstar sorts its keys, so exporting an unchanged resource leaves Git with nothing to commit. Source IDs live only in `source`.

### Export asks LangSmith for the model

Export reads the commit with `include=model`, the form the Go SDK uses (`references/langsmith-go.md`). The Python SDK sends the older `include_model=true`, and the LangSmith OpenAPI spec accepts both. With the model included, the manifest becomes a `RunnableSequence` whose last step binds the chat model and its settings, and the response also carries `model_config` and `model_provider` beside the manifest (`references/langsmith-sdk.md`). Creating a commit accepts only `manifest`, `parent_commit` and `description`, so the model reaches the destination inside the manifest. The OpenAPI validation task checks that the destination returns the same model after the push.

### Import records what it created in a separate file

Import writes the destination's IDs to `<file stem>.destinations.json`, keyed by destination workspace ID for prompts and by deployment ID for assistants. The export file stays the same after any number of imports, and a fresh export never wipes out the record of earlier imports. A repeated import reads the record to find the copy it made. For a prompt, the handle already finds the copy, and the record tells langstar whether someone changed the destination since the last import. For an assistant, whose ID differs on every deployment, the record is the only link from the file to its copy.

### Import changes an existing resource only with --update

A prompt that exists in the destination gets a new commit only with `--update`, and its parent is the destination's latest commit, so LangSmith keeps the earlier history. An assistant that exists gets updated only with `--update`, which makes the Agent Server save a new version. Without `--update`, import lists the differences and exits with status 2. `--dry-run` reports the action import would take and writes nothing.

### Compare reports differences by JSON path

Compare reads the destination, removes the fields the server sets (IDs, version numbers, timestamps, owner and like counts), and walks both JSON documents. Each difference names its JSON Pointer path and both values. Import runs the same comparison after it writes, so a field LangSmith or the Agent Server drops shows up as a difference.

### Verify checks what the copy needs to run

Prompt verify looks up each name in `requires.secrets` among the workspace's secrets (`GET /api/v1/workspaces/current/secrets`, which `langstar secrets list` already calls) and looks for a model configuration whose provider matches `requires.model_provider` (`GET /api/v1/playground-settings`, used by `langstar model-config list`). It reports names only. Langstar finds secret names by walking the manifest for LangChain secret references, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, the shape the Python SDK's `_manifest_has_secrets` matches.

Assistant verify checks the graph with `POST /assistants/search` filtered to the graph ID; every graph on a deployment has a default assistant, so an empty result means the deployment does not serve the graph. It checks each prompt in `requires.prompts` with the same lookup `langstar prompt get` uses.

### The user names an assistant's prompt references

An assistant refers to a prompt through whatever configuration key its graph reads, so langstar cannot find the references by itself. `assistant export --prompt-ref <json-pointer>` names each one. Prompts should be referred to by handle and tag, such as `support-system:prod`, because a commit hash differs between workspaces. Import carries commit tags across for this reason.

### Exit statuses

Status 0 means the resource matches and every check passed. Status 2 means langstar found a difference or a failed check. Status 1 stays the status for every other error, as in every other langstar command. `diff` uses 1 for differences and 2 for errors; langstar keeps 1 for errors so that a script testing for failure sees the same status from every command.

### Structured output check

The LangSmith OpenAPI spec has one endpoint that invokes a prompt, `POST /api/v1/prompts/invoke_prompt`. Its body, `InvokePromptPayload`, holds only message templates, `template_format` and `inputs`, with no field for a model or an output schema, and the spec gives no shape for its response. No LangSmith SDK calls it. So the API offers no documented way to show that a structured prompt still returns structured data; the OpenAPI validation task tries the endpoint against the live API to settle this. The end-to-end test does this outside langstar: a mise task runs a short Python script that pulls the imported prompt with `include_model=True`, invokes it, and checks that the result matches the schema. The script needs a model provider API key, which codekiln has to create and add to fnox and CI before that test can run.

### Relationship to #728

[#728](https://github.com/codekiln/langstar/issues/728) splits prompt text, model and structured output into separate `create`, `get` and `update` commands for editing one field at a time. Export and import handle the whole prompt at once. Both use the same SDK calls, and once #728 lands, `prompt get --all` shows the same fields an export file holds.

## Risks / Trade-offs

- [LangSmith may store a pushed model differently from the source] → Import reads the prompt back and reports any difference, and the OpenAPI validation task checks the round trip before the commands are built.
- [Pushing a manifest identical to the latest commit may return 409] → Import compares first and pushes nothing when the prompt matches.
- [Most assistant integration tests are ignored today, because GitHub-built test deployments have no URL] → The test fixture task makes a deployment langstar can reach before the assistant tests are written.
- [A tag in the destination may already point at another commit] → `--update` moves the tag, and `--dry-run` lists the move.

## Resolved Questions

None.

## Open Questions

### 1 - Should compare, verify and import exit with 2 when they find a difference?

> Context from the drafting agent, for question 1.
> The draft uses 0 for a match, 2 for a difference or failed check, and 1 for every other error, as in every other langstar command. `diff` and `git diff --exit-code` use 1 for differences, which would make a difference look like an error to scripts written against langstar. Recommendation: keep 0, 2 and 1.

<ANSWER_HERE>

### 2 - Should import record destination IDs in a separate file next to the export?

> Context from the drafting agent, for question 2.
> The draft writes `<file stem>.destinations.json` and leaves the export unchanged. Keeping the record inside the export file would make one file per resource, but export would then have to keep the record when it overwrites the file. Recommendation: the separate file.

<ANSWER_HERE>

### 3 - How should the end-to-end test show that an imported structured prompt still returns structured data?

> Context from the drafting agent, for question 3.
> The LangSmith OpenAPI spec's only prompt-invoke endpoint, `POST /api/v1/prompts/invoke_prompt`, takes message templates and inputs with no model or schema, and no SDK calls it. The validation task tries it live. The draft runs a Python script through a mise task, using the LangSmith Python SDK and LangChain, which is how most users run a prompt. It needs a model provider key, such as `OPENAI_API_KEY`, which codekiln would create and add to fnox and CI. Building the model call into langstar as `prompt verify --invoke` would need langstar to call each provider's API itself. Recommendation: the Python script, with an OpenAI key and a small model.

<ANSWER_HERE>

### 4 - Should the user name an assistant's prompt references with --prompt-ref?

> Context from the drafting agent, for question 4.
> A graph reads its prompt names from whatever configuration key its author chose, so langstar cannot find them reliably. Guessing from strings that look like prompt handles would miss some and flag others. Recommendation: `--prompt-ref <json-pointer>`, repeatable, recorded in the export file so import and verify reuse it.

<ANSWER_HERE>

### 5 - Which second workspace and second deployment should the tests use?

> Context from the drafting agent, for question 5.
> The tests need a destination workspace and a destination deployment. The draft adds `LANGSTAR_TEST_DEST_WORKSPACE_ID`, a second workspace in the test organization so the existing API key works, and a second test deployment built by the existing `TestDeploymentConfig` fixture under its own name prefix. A second deployment costs build time and hosting in every CI run that creates it. Recommendation: a second workspace in the same organization, and one long-lived destination deployment that the tests reuse.

<ANSWER_HERE>

### 6 - Should prompt import refuse to run without --workspace-id even when LANGSMITH_WORKSPACE_ID is set?

> Context from the drafting agent, for question 6.
> Issue #807 asks for an explicit destination workspace. Requiring the flag means a leftover environment variable can never send a prompt to the wrong workspace, at the cost of one more flag in scripts. Assistant import already requires `--deployment`. Recommendation: require the flag.

<ANSWER_HERE>
