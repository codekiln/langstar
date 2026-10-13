# Design

## Context

`langstar prompt push` in `cli/src/commands/prompt.rs` builds one shape of prompt: a single human message, with an optional schema from [#402 structured output prompts](https://github.com/codekiln/langstar/issues/402). `langstar prompt pull` prints any manifest LangSmith returns, but `PromptClient` in `sdk/src/prompts.rs` never asks LangSmith for the model, so that manifest has none. `PromptClient` keeps the manifest as raw JSON and has a typed `StructuredPrompt` view that holds messages, the schema and the method.

`langstar assistant` in `cli/src/commands/assistant.rs` takes `--deployment` on every subcommand and sends requests to that deployment's Agent Server. `AssistantClient` in `sdk/src/assistants.rs` has no `context` or `version` field, its create request cannot set `assistant_id` or `if_exists`, and its search cannot filter by graph ID.

Langstar reads one API key and one workspace at a time, from flags, environment variables or the config file. Every error exits with status 1.

## Goals / Non-Goals

**Goals:**

- Carry every field of a prompt commit and an assistant through a file and back without loss.
- Make a repeated import of the same file safe: it finds its earlier copy and changes nothing until the user asks.
- Give users one way to check a copy: the same report shape and exit statuses for both resource types.

**Non-Goals:**

- Copy a prompt or assistant between environments in one command.
- Export or import every prompt in a workspace, or every assistant on a deployment, in one run.
- Redesign `prompt create`, `get` and `update`, which [#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728) covers.

## Decisions

### Export and import run at different times, each with its own environment's API key

Export runs with the source environment's credentials and writes a file. Import runs later with the destination's credentials and reads it. Each run uses langstar's existing settings, so a user moving a prompt to another organization sets a different `LANGSMITH_API_KEY` for the import. Langstar needs no second set of credentials and no profiles. The file sits in Git between the two runs, so a reviewer sees the prompt or assistant change in a pull request.

The LangSmith Go CLI moves a prompt through a file the same way, as `references/langsmith-cli.md` shows. Langstar's export asks for the model, and the Go CLI's pull does not. Langstar's import reads the file export writes, while the Go CLI's push sends whatever JSON it reads as the manifest, so its own pull output reaches LangSmith wrapped in an envelope. Langstar's import also requires `--workspace-id`, so the user names the destination on the command line and a leftover environment variable never chooses it.

### The export file keeps the manifest LangSmith returned

Export stores the manifest as raw JSON. The typed `StructuredPrompt` struct cannot hold a model binding, placeholders, several message types or partial variables, so routing through it would drop them. Langstar reads the manifest through typed views only to fill in `requires`.

One prompt export looks like this:

```json
{
  "kind": "prompt",
  "format_version": 1,
  "source": { "workspace_id": "<source-workspace-id>", "handle": "my-prompt", "commit_hash": "<hash>", "ref": "prod" },
  "repo": { "handle": "my-prompt", "description": "...", "readme": "...", "tags": ["support"], "is_public": false },
  "commit": { "description": "...", "manifest": { "lc": 1, "type": "constructor", "id": ["langsmith", "playground", "PromptPlayground"], "kwargs": { "first": {}, "last": {} } }, "model_config": {}, "model_provider": "ChatOpenAI", "tags": ["prod"] },
  "requires": { "secrets": ["OPENAI_API_KEY"], "model_provider": "ChatOpenAI" }
}
```

An assistant export has `kind: "assistant"`, a `source` with the deployment, assistant ID and version, an `assistant` object with `name`, `description`, `graph_id`, `config`, `context` and `metadata`, and `requires.prompts`.

The file holds no export time and langstar sorts its keys, so exporting an unchanged resource leaves Git with nothing to commit. Source IDs live only in `source`.

### Export asks LangSmith for the model

Export reads the commit with `include=model`, the form the Go SDK uses (`references/langsmith-go.md`). The Python SDK sends the older `include_model=true`, and the LangSmith OpenAPI spec accepts both. With the model included, the manifest wraps the prompt and the chat model with its settings in one constructor, and the response also carries `model_config` and `model_provider` beside the manifest (`references/langsmith-sdk.md`). No field of the create-commit request holds the model, so the model reaches the destination inside the manifest. [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) pushes that manifest to a second workspace, records whether LangSmith returns the same model, and records which outer `id` the live API returns.

### Import records what it created in a separate file

Import writes the destination's IDs to `<file stem>.destinations.json`, keyed by destination workspace ID for prompts and by deployment ID for assistants. The export file stays the same after any number of imports, and a fresh export never wipes out the record of earlier imports. A repeated import reads the record to find the copy it made. For a prompt, the handle already finds the copy, and the record tells langstar whether someone changed the destination since the last import. For an assistant, whose ID differs on every deployment, the record is the only link from the file to its copy.

### Import changes an existing resource only with --update

A prompt that exists in the destination gets a new commit only with `--update`, and its parent is the destination's latest commit, so LangSmith keeps the earlier history. Import sends the file's commit description with each commit it creates. With `--update`, import also sets the destination repo's description, readme, tags and visibility to the file's values through `PATCH /api/v1/repos/{owner}/{repo}` when they differ. When only those repo fields differ and the manifest matches, import updates the repo and adds no commit.

An assistant that exists gets updated only with `--update`, which makes the Agent Server save a new version. The Agent Server merges the metadata an update sends into the assistant's metadata (`AssistantPatch` in `reference/openapi/langchain/agent-server/openapi.json`), so sending the file's metadata alone would keep every key the destination has and the file lacks. Import sends each such key with a null value and then reads the assistant back. [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) checks against the live server whether a null value removes the key. If it does not, the read-back finds the leftover keys, and import reports them as differences and exits with status 2. Import also sends the file's graph ID, so an assistant whose graph changed at the source moves to that graph.

Without `--update`, import lists the differences and exits with status 2. `--dry-run` reports the action import would take and writes nothing.

### Compare reports differences by JSON path

Compare reads the destination, removes the fields the server sets (IDs, version numbers, timestamps, owner and like counts), and walks both JSON documents. Each difference names its JSON Pointer path and both values. For a prompt, the comparison covers the commit description and the repo's description, readme, tags and visibility as well as the manifest. Import runs the same comparison after it writes, so a field LangSmith or the Agent Server drops shows up as a difference.

### Verify checks what the copy needs to run

Prompt verify looks up each name in `requires.secrets` among the workspace's secrets (`GET /api/v1/workspaces/current/secrets`, which `langstar secrets list` already calls). It also looks for a saved model configuration (`GET /api/v1/playground-settings`, which `langstar model-config list` calls) whose settings `id` ends in the class name that `requires.model_provider` holds, such as `ChatOpenAI`. The prompt runs without a saved model configuration, because its manifest carries the model, so verify reports a missing one as a note and leaves the exit status alone; [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks for the report, and Open Question 7 asks whether to keep it. Verify reports names only. Langstar finds secret names by walking the manifest for LangChain secret references, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, the shape the Python SDK's `_manifest_has_secrets` matches.

Assistant verify checks the graph with `POST /assistants/search` filtered to the graph ID; every graph on a deployment has a default assistant, so an empty result means the deployment does not serve the graph. It checks each prompt in `requires.prompts`, such as `support-system:prod`, by asking LangSmith for the commit that tag points at (`GET /api/v1/commits/{owner}/{repo}/{tag}`, the request `langstar prompt pull --commit prod` already sends). A 404 means the workspace lacks the prompt or its tag.

### The user names an assistant's prompt references

An assistant refers to a prompt through whatever configuration key its graph reads, so langstar cannot find the references by itself. `assistant export --prompt-ref <json-pointer>` names each one. The assistant's configuration should name each prompt by handle and tag, such as `support-system:prod`, because the same prompt gets a different commit hash in each workspace. Import copies the commit tags to the destination workspace, so `support-system:prod` finds the prompt there too.

### compare, verify and import exit with 2 when they find a difference

Status 0 means the resource matches and every check passed. Status 2 means langstar found a difference or a failed check. Status 1 stays the status for every other error, as in every other langstar command. `diff` uses 1 for differences and 2 for errors; langstar keeps 1 for errors so that a script testing for failure sees the same status from every command.

### A Python script checks that an imported structured prompt still returns structured data

A mise task runs a short Python script that pulls the imported prompt with `include_model=True`, invokes it, and checks that the result matches the prompt's schema. The script exists because the LangSmith API has no documented way to run a prompt with its model: the one prompt endpoint in the OpenAPI spec, `POST /api/v1/prompts/invoke_prompt`, takes message templates and inputs with no model or schema, and no LangSmith SDK calls it. [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) tries that endpoint against the live API in case it does more than the spec says. The script needs a model provider API key, which codekiln has to create and add to fnox and CI before the test can run.

### Export and import use the same SDK calls as the planned `prompt create`, `get` and `update`

[#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728) splits prompt text, model and structured output into separate `create`, `get` and `update` commands for editing one field at a time. Export and import handle the whole prompt at once. Both use the same SDK calls, and once #728 lands, `prompt get --all` shows the same fields an export file holds.

## Risks / Trade-offs

- [LangSmith may store a pushed model differently from the source] → Import reads the prompt back and reports any difference, and [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) checks the round trip before the commands are built.
- [Pushing a manifest identical to the latest commit may return 409] → Import compares first and pushes nothing when the prompt matches.
- [langstar finds a deployment's Agent Server URL only in `source_config.custom_url` (`resolve_deployment_url` in `cli/src/deployment_utils.rs`), and a deployment built from GitHub has none, so every `langstar assistant` command, the new ones included, fails on such a deployment] → [export-import-prompts-and-assistants Task 2.3 Return the URL of a deployment built from GitHub](tasks.md#L14) makes `resolve_deployment_url` read the deployment's top-level `url`, which the control plane's `Deployment` schema carries, before anyone builds the assistant export, compare, verify and import commands.
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
> The LangSmith OpenAPI spec's only prompt-invoke endpoint, `POST /api/v1/prompts/invoke_prompt`, takes message templates and inputs with no model or schema, and no SDK calls it. [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) tries it live. The draft runs a Python script through a mise task, using the LangSmith Python SDK and LangChain, which is how most users run a prompt. It needs a model provider key, such as `OPENAI_API_KEY`, which codekiln would create and add to fnox and CI. Building the model call into langstar as `prompt verify --invoke` would need langstar to call each provider's API itself. Recommendation: the Python script, with an OpenAI key and a small model.

<ANSWER_HERE>

### 4 - Should the user name an assistant's prompt references with --prompt-ref?

> Context from the drafting agent, for question 4.
> A graph reads its prompt names from whatever configuration key its author chose, so langstar cannot find them reliably. Guessing from strings that look like prompt handles would miss some and flag others. Recommendation: `--prompt-ref <json-pointer>`, repeatable, recorded in the export file so import and verify reuse it.

<ANSWER_HERE>

### 5 - Which second workspace and second deployment should the tests use?

> Context from the drafting agent, for question 5.
> The tests need a destination workspace and a destination deployment. The draft adds `LANGSTAR_TEST_DEST_WORKSPACE_ID`, a second workspace in the test organization so the existing API key works, and a second test deployment built by the existing `TestDeploymentConfig` fixture under its own name prefix. A second deployment costs build time and hosting in every CI run that creates it. Recommendation: a second workspace in the same organization, which codekiln creates, and one destination deployment that the `TestDeploymentConfig` fixture creates on the first run and the tests reuse after that.

<ANSWER_HERE>

### 6 - Should prompt import refuse to run without --workspace-id even when LANGSMITH_WORKSPACE_ID is set?

> Context from the drafting agent, for question 6.
> Issue #807 asks for an explicit destination workspace. Requiring the flag means a leftover environment variable can never send a prompt to the wrong workspace, at the cost of one more flag in scripts. Assistant import already requires `--deployment`. Recommendation: require the flag.

<ANSWER_HERE>

### 7 - Should prompt verify report a missing saved model configuration?

> Context from the drafting agent, for question 7.
> Issue #807 asks import to report missing model configurations, so the draft has prompt verify look for a saved model configuration (`GET /api/v1/playground-settings`) whose settings `id` ends in the prompt's `model_provider` class name, such as `ChatOpenAI`. Nothing fails at run time without one, because the prompt's manifest carries the model and its settings; the secret check is the one that finds a prompt that cannot authenticate. Reporting it as a failure would make verify exit with 2 for a prompt that runs. Recommendation: report a missing saved model configuration as a note that leaves the exit status alone.

<ANSWER_HERE>
