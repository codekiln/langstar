# Design

## Context

`langstar prompt push` in `cli/src/commands/prompt.rs` builds one shape of prompt: a single human message, with an optional schema from [#402 structured output prompts](https://github.com/codekiln/langstar/issues/402). `langstar prompt pull` prints any manifest LangSmith returns, but `PromptClient` in `sdk/src/prompts.rs` never asks LangSmith for the model, so that manifest has none. `PromptClient` keeps the manifest as raw JSON and has a typed `StructuredPrompt` view that holds messages, the schema and the method.

`langstar assistant` in `cli/src/commands/assistant.rs` takes `--deployment` on every subcommand and sends requests to that deployment's Agent Server. `AssistantClient` in `sdk/src/assistants.rs` has no `context` or `version` field and its search cannot filter by graph ID.

Langstar reads one API key and one workspace at a time, from flags, environment variables or the config file. Langstar exits with status 1 when a command fails, and clap, the library that parses langstar's flags, exits with status 2 when it rejects a mistyped flag or a missing required one.

## Goals / Non-Goals

**Goals:**

- Carry a prompt's manifest and model, its commit description and tags, and its repo details, and an assistant's name, description, graph ID, configuration, context and metadata, through a file and back without loss.
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
  "commit": { "description": "...", "manifest": { "lc": 1, "type": "constructor", "id": ["langsmith", "playground", "PromptPlayground"], "kwargs": { "first": {}, "last": {} } }, "model_config": {}, "model_provider": "...", "tags": ["prod"] },
  "requires": { "secrets": ["OPENAI_API_KEY"], "model_provider": "ChatOpenAI" }
}
```

An assistant export has `kind: "assistant"`, a `source` with the deployment, assistant ID and version, an `assistant` object with `name`, `description`, `graph_id`, `config`, `context` and `metadata`, and `requires.prompts`.

The file holds no export time and langstar sorts its keys, so exporting an unchanged resource leaves Git with nothing to commit.

### Export asks LangSmith for the model

Export reads the commit with `include=model`, the form the Go SDK uses (`references/langsmith-go.md`). The Python SDK sends the older `include_model=true`, and the LangSmith OpenAPI spec accepts both. With the model included, the manifest wraps the prompt and the chat model with its settings in one constructor, and the response also carries `model_config` and `model_provider` beside the manifest (`references/langsmith-sdk.md`). No field of the create-commit request holds the model, so the model reaches the destination inside the manifest. [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) pushes that manifest to a second workspace, records whether LangSmith returns the same model, and records which outer `id` the live API returns.

### Import records what it created in a separate file

Import writes the destination's IDs to `<file stem>.destinations.json`, keyed by destination workspace ID for prompts and by deployment ID for assistants. The export file stays the same after any number of imports, and a fresh export never wipes out the record of earlier imports. A repeated import reads the record to find the copy it made. For a prompt, langstar finds the copy by its handle, and compares the commit hash in the record with the destination's latest commit to tell whether someone changed the prompt since the last import. For an assistant, whose ID differs on every deployment, the record is the only link from the file to its copy.

### Import changes an existing resource only with --update

A prompt that exists in the destination gets a new commit only with `--update`, and its parent is the destination's latest commit, so LangSmith keeps the earlier history. Import sends the file's commit description with each commit it creates. With `--update`, import also sets the destination repo's description, readme, tags and visibility to the file's values through `PATCH /api/v1/repos/{owner}/{repo}` when they differ. When only those repo fields differ and the manifest matches, import updates the repo and adds no commit. A commit tag in the file that the destination lacks, or that points at another commit there, also counts as a difference; with `--update`, import creates or moves the tag to the destination commit that matches the file, and adds no commit when the latest commit already matches. The tag endpoints take a commit's `id`, which differs from its hash. The commit read and the repo read return no `id`, so import looks the commit up in the commit list, `GET /api/v1/commits/{owner}/{repo}`, whose entries carry both the `id` and the hash.

With `--update`, import makes an existing assistant match the file and asks the Agent Server to remove what the file lacks. The Agent Server keeps the name, graph ID and description an update leaves out, and merges the metadata an update sends into the metadata the assistant already has, as `AssistantPatch` in `reference/openapi/langchain/agent-server/openapi.json` documents. So import sends the file's name, graph ID, configuration and context in full, an empty object for a configuration or context the file lacks, and a null value for each metadata key the destination has and the file lacks. `AssistantPatch` documents no way to clear a description, so [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) tries a null and an empty description against the live server, along with a null metadata value and a replaced configuration and context. Import then reads the assistant back, and any key or field the file lacks that is still there shows up as a difference, with exit status 3. The Agent Server saves each update as a new version of the assistant.

Without `--update`, import lists the differences and exits with status 3. `--dry-run` reports the action import would take and writes nothing.

### Compare reports differences by JSON path

Compare reads the destination, removes the fields the server sets (IDs, version numbers, timestamps, owner and like counts), and walks both JSON documents. Compare looks only at the `repo`, `commit` and `assistant` objects. The source environment's workspace, deployment and IDs sit in `source`, so they never show up as differences against a copy. Each difference names its JSON Pointer path and both values. For a prompt, the comparison covers the commit description, the commit tags and the commit each one points at, and the repo's description, readme, tags and visibility as well as the manifest. Import runs the same comparison after it writes, so a field LangSmith or the Agent Server drops shows up as a difference.

### Verify checks what the copy needs to run

Prompt verify looks up each name in `requires.secrets` among the workspace's secrets (`GET /api/v1/workspaces/current/secrets`, which `langstar secrets list` already calls). It also looks for a saved model configuration (`GET /api/v1/playground-settings`, which `langstar model-config list` calls) whose settings `id` ends in the class name that `requires.model_provider` holds, such as `ChatOpenAI`. Export reads that class name from the last element of the chat model constructor's `id` in the manifest, such as `["langchain", "chat_models", "openai", "ChatOpenAI"]`. LangSmith also returns a `model_provider` field beside the manifest, and the file keeps it as `commit.model_provider`. No SDK or spec shows what that field holds, so [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) records it. The prompt runs without a saved model configuration, because its manifest carries the model, so verify reports a missing one as a note and leaves the exit status alone; [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks for the report, and [Should prompt verify report a missing saved model configuration?](#7---should-prompt-verify-report-a-missing-saved-model-configuration) asks whether to keep it. Verify prints the name of each missing secret and no secret value. Langstar finds secret names by walking the manifest for LangChain secret references, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, the shape the Python SDK's `_manifest_has_secrets` matches.

Assistant verify checks the graph with `POST /assistants/search` filtered to the graph ID; every graph on a deployment has a default assistant, so an empty result means the deployment does not serve the graph. It checks each prompt in `requires.prompts`, such as `support-system:prod`, by asking LangSmith for the commit that tag points at (`GET /api/v1/commits/{owner}/{repo}/{tag}`, the request `langstar prompt pull --commit prod` already sends). A 404 means the workspace lacks the prompt or its tag.

### Import checks the destination's secrets, graph and prompts before it writes

Before it writes anything, import runs the dependency checks verify runs against the destination, because [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks import to check the destination's dependencies. It skips verify's comparison, since the destination may have no copy yet. Prompt import stops when the destination workspace lacks a secret in `requires.secrets`, and assistant import stops when the deployment does not serve the file's graph or the workspace, from `--workspace-id` or the configured workspace, lacks a prompt in `requires.prompts`. Each names what is missing, writes nothing and exits with status 3, and `--dry-run` reports the same failed checks. Import therefore writes a copy only when the destination has the secrets, graph and prompts that copy needs to run. A user therefore has to create the destination workspace's secrets before importing a prompt, and import the prompts before importing an assistant that refers to them. [Should a failed dependency check stop import before it writes?](#8---should-a-failed-dependency-check-stop-import-before-it-writes) asks whether import should write anyway and report the failed checks afterwards.

### The user names an assistant's prompt references

An assistant refers to a prompt through whatever configuration key its graph reads, so langstar cannot find the references by itself. `assistant export --prompt-ref <json-pointer>` names each one. The assistant's configuration should name each prompt by handle and tag, such as `support-system:prod`, because the same prompt gets a different commit hash in each workspace. Import copies the commit tags to the destination workspace, so when the graph on the destination deployment asks LangSmith for `support-system:prod`, it gets the same prompt.

### compare, verify and import exit with 3 when they find a difference

Status 0 means the resource matches and every check passed. Status 3 means langstar found a difference or a failed check. Status 1 means any other error. Status 2 stays clap's: `Cli::parse()` in `run` in `cli/src/main.rs` exits with 2 when a user mistypes a flag or leaves out a required one, and this change leaves that alone, so a script never reads a typo as a difference. `prompt import` declares `--workspace-id` optional, as the other `langstar prompt` subcommands do, and checks for it itself, so a missing flag exits with status 1 and an error that names the flag. `diff` uses 1 for differences and 2 for errors; langstar keeps 1 for errors so that a script testing for failure sees the same status from every command.

### A Python script checks that an imported structured prompt still returns structured data

A mise task runs a short Python script that pulls the imported prompt with `include_model=True`, invokes it, and checks that the result matches the prompt's schema. The script passes `secrets={"OPENAI_API_KEY": <value read from the environment>}` to [`pull_prompt`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L10228), because the LangSmith Python SDK reads no secret from the environment unless asked and raises when a manifest holds a secret reference and no `secrets` map arrives. The script exists because the LangSmith API has no documented way to run a prompt with its model: the one prompt endpoint in the OpenAPI spec, `POST /api/v1/prompts/invoke_prompt`, takes message templates and inputs with no model or schema, and no LangSmith SDK calls it. [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) tries that endpoint against the live API in case it does more than the spec says. The script needs a model provider API key, which codekiln has to create and add to fnox and CI before the test can run.

### Export and import use the same SDK calls as the planned `prompt create`, `get` and `update`

[#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728) splits prompt text, model and structured output into separate `create`, `get` and `update` commands for editing one field at a time. Export and import handle the whole prompt at once. Both use the same SDK calls, and once #728 lands, `prompt get --all` shows the same fields an export file holds.

## Risks / Trade-offs

- [LangSmith may store a pushed model differently from the source] → Import reads the prompt back and reports any difference, and [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) checks the round trip before the commands are built.
- [Pushing a manifest identical to the latest commit may return 409] → Import compares first and pushes nothing when the prompt matches.
- [langstar finds a deployment's Agent Server URL only in `source_config.custom_url` (`resolve_deployment_url` in `cli/src/deployment_utils.rs`), and a deployment built from GitHub has none, so every `langstar assistant` command, the new ones included, fails on such a deployment] → [export-import-prompts-and-assistants Task 2.2 Return the URL of a deployment built from GitHub](tasks.md#L13) adds the control plane's top-level `url` to the SDK's `Deployment`, which drops it today, and makes `resolve_deployment_url` use it before anyone builds the assistant export, compare, verify and import commands.
- [A tag in the destination may already point at another commit] → `--update` moves the tag, and `--dry-run` lists the move.

## Resolved Questions

None.

## Open Questions

### 1 - Should compare, verify and import exit with 2 when they find a difference?

> Context from the drafting agent, for question 1.
> Status 2 is taken: clap exits with 2 when a user mistypes a flag or leaves out a required one, so a difference reported as 2 would look like a typo to a script. The draft uses 0 for a match, 3 for a difference or failed check and 1 for every other error, and leaves clap's 2 alone. The other choice makes `run` in `cli/src/main.rs` exit with 1 on a usage error, which frees 2 for differences but changes the status every langstar command gives for a mistyped flag. `diff` and `git diff --exit-code` use 1 for differences, which would make a difference look like an error. Recommendation: 3 for a difference.

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
> The tests need a destination workspace and a destination deployment. The draft adds `LANGSTAR_TEST_DEST_WORKSPACE_ID`, a second workspace in the test organization so the existing API key works, and a second test deployment that the existing `TestDeploymentConfig` fixture builds from its own `langgraph.json`, because the control plane allows one deployment per repository and graph config file. A second deployment costs build time and hosting in every CI run that creates it. Recommendation: a second workspace in the same organization, which codekiln creates, and one destination deployment that the `TestDeploymentConfig` fixture creates on the first run and the tests reuse after that.

<ANSWER_HERE>

### 6 - Should prompt import refuse to run without --workspace-id even when LANGSMITH_WORKSPACE_ID is set?

> Context from the drafting agent, for question 6.
> [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks for an explicit destination workspace. Requiring the flag means a leftover environment variable can never send a prompt to the wrong workspace, at the cost of one more flag in scripts. Assistant import already requires `--deployment`. Recommendation: require the flag.

<ANSWER_HERE>

### 7 - Should prompt verify report a missing saved model configuration?

> Context from the drafting agent, for question 7.
> [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks import to report missing model configurations, so the draft has prompt verify look for a saved model configuration (`GET /api/v1/playground-settings`) whose settings `id` ends in the class name the export file holds in `requires.model_provider`, such as `ChatOpenAI`. Nothing fails at run time without one, because the prompt's manifest carries the model and its settings; the secret check is the one that finds a prompt that cannot authenticate. Reporting it as a failure would make verify exit with 3 for a prompt that runs. Recommendation: report a missing saved model configuration as a note that leaves the exit status alone.

<ANSWER_HERE>

### 8 - Should a failed dependency check stop import before it writes?

> Context from the drafting agent, for question 8.
> [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks import to check the destination's dependencies: the secrets a prompt's model reads, and an assistant's graph and referenced prompts. The draft runs those checks before import writes and writes nothing when one fails, so a user never has to find and delete a copy that cannot run. The cost is order: the destination workspace needs its secrets before a prompt import, and its prompts before an assistant import. The other choice writes the copy anyway and reports the failed checks with status 3 afterwards, which lets a user import in any order and fix the destination later. Recommendation: stop before writing.

<ANSWER_HERE>
