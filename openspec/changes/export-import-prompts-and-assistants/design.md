# Design

## Context

`langstar prompt push` in `cli/src/commands/prompt.rs` builds one shape of prompt: a single human message, with an optional schema from [#402 structured output prompts](https://github.com/codekiln/langstar/issues/402). `langstar prompt pull` prints any manifest LangSmith returns, but `PromptClient` in `sdk/src/prompts.rs` never asks LangSmith for the model, so that manifest has none. `PromptClient` keeps the manifest as raw JSON and has a typed `StructuredPrompt` view that holds messages, the schema and the method.

Langstar's assistant SDK lacks what an export needs: `Assistant` in `sdk/src/assistants.rs` has no `context` or `version` field, and the search in `AssistantClient` cannot filter by graph ID. The assistant commands themselves already work against the test deployment: `test_assistant_description_field` in `cli/tests/assistant_command_test.rs` creates and reads an assistant there in the Integration Tests CI job.

Langstar reads one API key and one workspace at a time, from flags, environment variables or the config file. Langstar exits with status 1 when a command fails, and clap, the library that parses langstar's flags, exits with status 2 when it rejects a mistyped flag or a missing required one.

## Goals / Non-Goals

**Goals:**

- Recreate a prompt or an assistant from its export file with nothing lost. A prompt keeps its manifest and model, its commit description and tags, and its repo details. An assistant keeps its name, description, graph ID, configuration, context and metadata.
- Make a repeated import of the same file safe: it finds its earlier copy and changes nothing until the user asks.
- Give users one way to check a copy: the same report shape and exit statuses for both resource types.

**Non-Goals:**

- Copy a prompt or assistant between environments in one command.
- Export or import every prompt in a workspace, or every assistant on a deployment, in one run.
- Redesign `prompt create`, `get` and `update`, which [#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728) covers.

## Decisions

### Export and import run at different times, each with its own environment's API key

Export runs with the source environment's credentials and writes a file. Import runs later with the destination's credentials and reads it. Each run uses langstar's existing settings, so a user moving a prompt to another organization sets a different `LANGSMITH_API_KEY` for the import. Langstar needs no second set of credentials and no profiles. The file sits in Git between the two runs, so a reviewer sees the prompt or assistant change in a pull request.

The LangSmith Go CLI moves a prompt through a file the same way, as `references/langsmith-cli.md` shows. Langstar's export asks for the model, and the Go CLI's pull does not. Langstar's import reads the file export writes, while the Go CLI's push sends whatever JSON it reads as the manifest, so its own pull output reaches LangSmith wrapped in an envelope. Langstar's import also requires `--workspace-id`, so langstar imports into the workspace the user typed on the command line, even when `LANGSMITH_WORKSPACE_ID` names another.

### The export file keeps the manifest LangSmith returned

Export stores the manifest as raw JSON. The typed `StructuredPrompt` struct cannot hold a model binding, placeholders, several message types or partial variables, so routing through it would drop them. Langstar reads the manifest through typed views only to fill in `requires`.

One prompt export looks like this:

```json
{
  "kind": "prompt",
  "format_version": 1,
  "source": { "workspace_id": "<source-workspace-id>", "handle": "my-prompt", "commit_hash": "<hash>" },
  "repo": { "handle": "my-prompt", "description": "...", "readme": "...", "tags": ["support"], "is_public": false },
  "commit": { "description": "...", "manifest": { "lc": 1, "type": "constructor", "id": ["langsmith", "playground", "PromptPlayground"], "kwargs": { "first": {}, "last": {} } }, "model_config": {}, "model_provider": "...", "tags": ["prod"] },
  "requires": { "secrets": ["OPENAI_API_KEY"], "model_class": "ChatOpenAI" }
}
```

An assistant export has `kind: "assistant"`, a `source` with the deployment, assistant ID and version, an `assistant` object with `name`, `description`, `graph_id`, `config`, `context` and `metadata`, and `requires.prompts`.

The file holds no export time, and langstar sorts its keys, the commit's tags, the repo's tags and the secret names under `requires.secrets`, so exporting an unchanged resource leaves Git with nothing to commit.

Without `--output`, export prints the file to stdout and nothing else, so `langstar prompt export my-prompt > my-prompt.json` writes a file import can read. Export writes its progress messages to stderr in every output format. [`OutputFormatter::print_message` in cli/src/output.rs:173](https://github.com/codekiln/langstar/blob/e4ae746d51f1f3f9ccf3fc97787148c3b4df2bce/cli/src/output.rs#L173), which `info` calls, prints to stdout in table mode, so export does not use it for those messages.

Import, compare and verify read a file only when its `format_version` is 1 and its `kind` matches the command: `prompt` for the prompt commands and `assistant` for the assistant commands. For any other file the command writes nothing, exits with status 1 and names the field and the value it found. When a user runs `prompt import` on an assistant export, or on a file a newer langstar wrote, langstar stops with that error before it changes anything in the destination.

### Export asks LangSmith for the model

Export reads the commit with `include=model`, the form the Go SDK uses (`references/langsmith-go.md`). The Python SDK sends the older `include_model=true`, and the LangSmith OpenAPI spec accepts both. With the model included, the manifest wraps the prompt and the chat model with its settings in one constructor, and the response also carries `model_config` and `model_provider` beside the manifest (`references/langsmith-sdk.md`). No field of the create-commit request holds the model, so the model reaches the destination inside the manifest, and export keeps `model_config` and `model_provider` in the file only for the reader. The LangSmith Python SDK's [`PromptCommit`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/schemas.py#L1054) documents `model_provider` as the model's class name, such as `ChatOpenAI`. In [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8), a developer pushes that manifest to a second workspace and records whether LangSmith returns the same manifest, `model_config` and `model_provider`, and which outer `id` the live API returns.

### Import records what it created in a separate file

Import writes the IDs of what it created to `<file stem>.destinations.json`, keyed by destination workspace ID for prompts and by deployment ID for assistants. The export file stays the same after any number of imports, and a fresh export never wipes out the record of earlier imports. Import replaces the record by writing the whole new record to a temporary file in the same directory and renaming it over the old one, so a langstar that stops in the middle of a write leaves the earlier record whole, pending assistant IDs included.

A repeated prompt import finds the copy by its handle and uses the record to tell whether someone changed it. Import writes the new commit hash to the record before it reads the commit back, so a read-back that finds a difference still leaves the hash in the record. The next import compares the destination's latest commit with that hash, and reports a change only when someone else pushed a commit after langstar's. Langstar can stop after it pushes a commit and before it writes that commit's hash to the record. The next import finds the destination's latest commit matching the file, writes its hash to the record and reports the prompt unchanged.

An assistant's ID differs on every deployment, so the record is the only way a repeated import finds the assistant it made. Import chooses the new assistant's ID itself and writes it to the record, marked pending, before it sends the create request with that `assistant_id` and `if_exists` set to `do_nothing`, which the Agent Server's `AssistantCreate` accepts. After the create succeeds, import marks the entry created. If langstar stops before then, the next import finds the pending entry and sends the same create again, and the Agent Server returns the assistant the first import made. Assistant compare and verify find the assistant through the same record, and take `--record <path>` as import does, so a record kept at another path still works.

### Import changes an existing resource only with --update

A prompt that exists in the destination gets a new commit only with `--update`, and its parent is the destination's latest commit, so LangSmith keeps the earlier history. Import sends the file's commit description with each commit it creates. With `--update`, import also sets the destination repo's description, readme, tags and visibility to the file's values through `PATCH /api/v1/repos/{owner}/{repo}` when they differ. When only those repo fields differ and the manifest matches, import updates the repo and adds no commit.

Import treats the destination commit's tags as one more field to match. A tag in the file that the destination lacks, or that points at another commit there, is a difference, and so is a tag on the destination's matching commit that the file lacks. With `--update`, import creates or moves each of the file's tags to that commit and deletes each tag on it that the file lacks, through `DELETE /api/v1/repos/{owner}/{repo}/tags/{tag_name}`, so the read-back finds the same tags as the file. It adds no commit when the latest commit already matches. A tag on an older destination commit is not part of the comparison, so import leaves it alone. The tag endpoints take a commit's `id`, which differs from its hash. The commit read and the repo read return no `id`, so import looks the commit up in the commit list, `GET /api/v1/commits/{owner}/{repo}`, whose entries carry both the `id` and the hash.

An import that stops after it creates the repo and before it pushes the commit leaves a prompt with no commit. The next import finds the prompt by its handle and, through the commit list, finds no commit. It then pushes the file's manifest as the first commit and creates the file's tags, as it does for a new prompt, without `--update`. The repo details it created hold the file's values already, and any that someone changed in between still need `--update`.

With `--update`, import makes an existing assistant match the file and asks the Agent Server to remove what the file lacks. The Agent Server keeps the name, graph ID and description an update leaves out, and merges the metadata an update sends into the metadata the assistant already has, as `AssistantPatch` in `reference/openapi/langchain/agent-server/openapi.json` documents. So import sends the file's name, graph ID, configuration and context in full, an empty object for a configuration or context the file lacks, and a null value for each metadata key the destination has and the file lacks. `AssistantPatch` documents no way to clear a description, so in [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) a developer sends a null and an empty description to the live server, along with a null metadata value and a replaced configuration and context, and records what the server keeps. Import then reads the assistant back, and any key or field the file lacks that is still there shows up as a difference, with exit status 3. The Agent Server saves each update as a new version of the assistant.

Without `--update`, import lists the differences and exits with status 3. `--dry-run` reports the action import would take and writes nothing.

### Compare reports differences by JSON path

Compare reads the prompt or assistant from the destination, puts it in the shape of the export file's `repo`, `commit` and `assistant` objects, and lists every value that differs from the file. Compare checks every key in those objects, including the chat model's settings and the `id` arrays inside a manifest that name the prompt and model classes, such as `ChatOpenAI`. It skips `commit.model_config` and `commit.model_provider`, because LangSmith fills both in itself when import pushes a commit, and a destination workspace can fill them in differently from the source. Export leaves the values the server sets, such as IDs, version numbers, timestamps, owner and like counts, out of those objects, so they never show up as differences. The source environment's workspace, deployment and IDs sit in `source`, so they never show up as differences against a copy. Each difference names its JSON Pointer path and both values, except at a chat model setting whose name ends in `api_key`: there the report prints `<redacted>` in place of a string on either side, so a key someone typed into the destination prompt's model never reaches a terminal or a CI log. For a prompt, the comparison covers the commit description, the commit tags and the commit each one points at, any tag on the compared commit that the file lacks, and the repo's description, readme, tags and visibility as well as the manifest. Import runs the same comparison after it writes, so a field LangSmith or the Agent Server drops shows up as a difference.

### Verify checks what the copy needs to run

Prompt verify checks that the workspace holds every secret the prompt's model reads. Export lists those names in `requires.secrets` by walking the manifest for LangChain secret references, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, the shape the Python SDK's `_manifest_has_secrets` matches. Verify looks each name up among the workspace's secrets (`GET /api/v1/workspaces/current/secrets`, which `langstar secrets list` already calls) and prints the name of each missing secret and no secret value.

Prompt verify also notes when the workspace has no saved model configuration for the prompt's model, and leaves the exit status alone, because the prompt runs with the model its manifest carries. Export takes the model's class name, such as `ChatOpenAI`, from the last element of the chat model constructor's `id` in the manifest, such as `["langchain", "chat_models", "openai", "ChatOpenAI"]`, and stores it in `requires.model_class`. Verify looks for a saved model configuration (`GET /api/v1/playground-settings`, which `langstar model-config list` calls) whose settings `id` ends in that class name. A prompt saved without a model has no chat model constructor in its manifest, so export writes `requires.model_class` as null, and verify and import skip this check for it and still run the secret check. [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks for the note, and [Should prompt verify report a missing saved model configuration?](#7---should-prompt-verify-report-a-missing-saved-model-configuration) asks whether to keep it.

Import and verify work out `requires` again from the file's manifest: the secret names, and the model class or null. When either differs from what the file holds, they write nothing, name the field and exit with status 1. This catches a manifest someone edited in Git without updating `requires`.

Assistant verify checks the graph with `POST /assistants/search` filtered to the graph ID; every graph on a deployment has a default assistant, so an empty result means the deployment does not serve the graph. It checks each prompt in `requires.prompts`, such as `support-system:prod`, by asking LangSmith for the commit that tag points at (`GET /api/v1/commits/{owner}/{repo}/{tag}`, the request `langstar prompt pull --commit prod` already sends). A 404 means the workspace lacks the prompt or its tag.

### Import checks the destination's secrets, graph and prompts before it writes

Before it writes anything, import checks that the destination has the secrets, the graph and the prompts the copy needs. These are the checks `verify` makes, and import runs them because [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks import to check the destination's dependencies. Prompt import stops when the destination workspace lacks a secret in `requires.secrets`, and assistant import stops when the deployment does not serve the file's graph or the workspace, from `--workspace-id` or the configured workspace, lacks a prompt in `requires.prompts`. Each names what is missing, writes nothing and exits with status 3, and `--dry-run` reports the same failed checks. A user therefore creates the destination workspace's secrets before importing a prompt, and imports the prompts before importing an assistant that refers to them. [Should a failed dependency check stop import before it writes?](#8---should-a-failed-dependency-check-stop-import-before-it-writes) asks whether import should write anyway and report the failed checks afterwards.

### The user names an assistant's prompt references

An assistant refers to a prompt through whatever configuration key its graph reads, so langstar cannot find the references by itself. `assistant export --prompt-ref <json-pointer>` names each one. The assistant's configuration should name each prompt by handle and tag, such as `support-system:prod`, because the same prompt gets a different commit hash in each workspace. Import copies the commit tags to the destination workspace, so when the graph on the destination deployment asks LangSmith for `support-system:prod`, it gets the same prompt. Compare, verify and import look up each pointer from `requires.prompts` in the file's `assistant` object, and exit with status 1 when the value there differs from the prompt name recorded beside the pointer. This catches a configuration someone edited in Git without updating `requires.prompts`.

### compare, verify and import exit with 3 when they find a difference

Status 0 means the resource matches and every check passed. Status 3 means langstar found a difference or a failed check. Status 1 means any other error. Status 2 stays clap's: `Cli::parse()` in `run` in `cli/src/main.rs` exits with 2 when a user mistypes a flag or leaves out a required one, and this change leaves that alone, so a script never reads a typo as a difference. `prompt import` declares `--workspace-id` optional, as the other `langstar prompt` subcommands do, and checks for the flag itself. When a user leaves the flag out, langstar exits with status 1 and an error that names `--workspace-id`, where clap would have exited with 2. `diff` uses 1 for differences and 2 for errors; langstar keeps 1 for errors so that a script testing for failure sees the same status from every command.

### A Python script checks that an imported structured prompt still returns structured data

The LangSmith API has no documented way to run a prompt with its model, so a short Python script runs each imported structured prompt with LangChain and checks that the result matches the prompt's schema. The one prompt endpoint in the OpenAPI spec, `POST /api/v1/prompts/invoke_prompt`, takes message templates and inputs with no model or schema, and no LangSmith SDK calls it; in [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8), a developer calls it on the live API in case it does more than the spec says. A mise task runs the script with `uv run --script`, which installs the `langsmith` and `langchain-openai` packages the script lists, on a developer's machine and in the Integration Tests CI job. The script reads the destination workspace: it builds its LangSmith `Client` with `workspace_id` set to `LANGSTAR_TEST_DEST_WORKSPACE_ID`, because CI sets `LANGSMITH_WORKSPACE_ID` to the source workspace, which holds a prompt with the same handle. The script pulls the imported prompt with `include_model=True` and passes `secrets={"OPENAI_API_KEY": <value read from the environment>}` to [`pull_prompt`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L10228), because the LangSmith Python SDK reads no secret from the environment unless asked, and raises when a manifest holds a secret reference and no `secrets` map arrives. The script needs an OpenAI API key. codekiln creates it and stores it in 1Password, in the repository's CI secrets and as the destination workspace's `OPENAI_API_KEY` secret, and in [export-import-prompts-and-assistants Task 2.1 Add the destination workspace and the model provider key](tasks.md#L16) a developer adds it to `fnox.toml` and the CI workflow.

### Export and import use the same SDK calls as the planned `prompt create`, `get` and `update`

[#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728) plans `prompt create`, `get` and `update` commands that edit a prompt's text, model or structured output one field at a time. Export and import read and write the whole prompt at once, through the same `PromptClient` calls in `sdk/src/prompts.rs` that those commands will use, so a field the SDK reads for export is there for `prompt get` too.

## Risks / Trade-offs

- [LangSmith may store a pushed model differently from the source] → Import reads the prompt back and reports any difference, and in [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) a developer checks the round trip against the live API before anyone builds the commands.
- [Pushing a manifest identical to the latest commit may return 409] → Import compares first and pushes nothing when the prompt matches.
- [The SDK changes for export and import add public fields to `Prompt`, `CommitManifestResponse`, `CommitData`, `CommitRequest`, `Assistant` and the assistant request structs, so a crate that builds one of those structs with a struct literal or destructures it with a pattern that names every field stops compiling] → Each pull request that adds a field updates every struct literal of that struct in this repository. [Should the SDK field additions count as a breaking change?](#9---should-the-sdk-field-additions-count-as-a-breaking-change) asks codekiln whether those pull requests also carry a `BREAKING CHANGE:` footer, which `docs/dev/git-scm-conventions.md` says makes the release a major version.
- [A prompt's manifest may hold a credential written as a literal model setting, rather than as a secret reference] → Export refuses with status 1 and writes nothing when a chat model setting whose name ends in `api_key` holds a string, and names that setting's path without its value. A credential under any other name, such as a header, still reaches the file, because langstar cannot tell it from any other string. The prompt export docs in [export-import-prompts-and-assistants Task 5.2 Document prompt export](tasks.md#L56) tell users to keep credentials in the workspace's secrets and to read an export file before committing it.
- [An assistant's configuration, context or metadata may hold a credential, which export copies into a file meant for Git] → Export writes those objects as the Agent Server returns them, since a credential there looks like any other string. The assistant export docs in [export-import-prompts-and-assistants Task 8.2 Document assistant export](tasks.md#L111) tell users to keep credentials in the deployment's environment variables or the workspace's secrets, and to read an export file before committing it.
- [A tag in the destination may already point at another commit] → `--update` moves the tag, and `--dry-run` lists the move.
- [`--update` deletes a tag on the matching destination commit that the file lacks, and a graph on a deployment may read that tag] → `--dry-run` lists each delete, and `assistant verify` reports a prompt tag an assistant refers to that the workspace no longer has.
- [The control plane may refuse a second deployment from this repository with 409] → In [export-import-prompts-and-assistants Task 2.3 Register the test graph under a second graph ID](tasks.md#L18), a developer deploys the new `langgraph.json` once, before anyone builds the fixture. If the control plane answers with 409, the developer brings [Which second workspace and second deployment should the tests use?](#5---which-second-workspace-and-second-deployment-should-the-tests-use) back to codekiln.

## Resolved Questions

None.

## Open Questions

### 1 - Should compare, verify and import exit with 3 when they find a difference?

> Context from the drafting agent, for question 1.
> Status 2 is taken: clap exits with 2 when a user mistypes a flag or leaves out a required one, so a difference reported as 2 would look like a typo to a script. The draft uses 0 for a match, 3 for a difference or failed check and 1 for every other error, and leaves clap's 2 alone. The other choice makes `run` in `cli/src/main.rs` exit with 1 on a usage error, which frees 2 for differences but changes the status every langstar command gives for a mistyped flag. `diff` and `git diff --exit-code` use 1 for differences, which would make a difference look like an error. Recommendation: 3 for a difference.

<ANSWER_HERE>

### 2 - Should import record destination IDs in a separate file next to the export?

> Context from the drafting agent, for question 2.
> The draft writes `<file stem>.destinations.json` and leaves the export unchanged. Keeping the record inside the export file would make one file per resource, but export would then have to keep the record when it overwrites the file. Recommendation: the separate file.

<ANSWER_HERE>

### 3 - How should the end-to-end test show that an imported structured prompt still returns structured data?

> Context from the drafting agent, for question 3.
> The LangSmith OpenAPI spec's only prompt-invoke endpoint, `POST /api/v1/prompts/invoke_prompt`, takes message templates and inputs with no model or schema, and no SDK calls it. In [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8), a developer calls it on the live API. The draft runs a Python script through a mise task with `uv run --script`. The script pulls the prompt from the destination workspace with the LangSmith Python SDK's `pull_prompt`, runs it with LangChain, and names both packages as inline dependencies. [export-import-prompts-and-assistants Task 7.3 Add the structured output check](tasks.md#L99) adds `uv` to `mise.toml` and removes `install: false` from the Integration Tests CI job, so that job runs `mise install`. It needs a model provider key, such as `OPENAI_API_KEY`, which codekiln would create and store in 1Password, the CI secrets and the destination workspace's secrets. Building the model call into langstar as `prompt verify --invoke` would need langstar to call each provider's API itself. Recommendation: the Python script, with an OpenAI key and a small model.

<ANSWER_HERE>

### 4 - Should the user name an assistant's prompt references with --prompt-ref?

> Context from the drafting agent, for question 4.
> A graph reads its prompt names from whatever configuration key its author chose, so langstar cannot find them reliably. Guessing from strings that look like prompt handles would miss some and flag others. Recommendation: `--prompt-ref <json-pointer>`, repeatable, recorded in the export file so import and verify reuse it.

<ANSWER_HERE>

### 5 - Which second workspace and second deployment should the tests use?

> Context from the drafting agent, for question 5.
> The import tests need a workspace and a deployment to import into. The draft has codekiln create a second workspace in the test organization, which the existing API key can reach, and the tests read its ID from `LANGSTAR_TEST_DEST_WORKSPACE_ID`. The destination deployment fixture from [export-import-prompts-and-assistants Task 2.4 Add a destination deployment fixture](tasks.md#L19) deploys the test graph a second time from a `langgraph.json` of its own, which registers the graph as both `test_graph` and `test_graph_alt` so the update tests can move an assistant from one graph to the other. Nobody knows yet whether the control plane accepts that deployment. It answers a second deployment of the same agent environment with 409 "A deployment already exists for this agent environment", and no document says whether a second `langgraph.json` in the same repository makes a new agent environment. In [export-import-prompts-and-assistants Task 2.3 Register the test graph under a second graph ID](tasks.md#L18), a developer deploys the new `langgraph.json` once to find out. If the control plane answers with the 409, the developer stops before task 2.4 and brings this question back to codekiln with what the control plane returned. `.github/workflows/cleanup-test-deployments.yml` deletes each test deployment once it is four hours old and the fixture rebuilds it on the next CI run, so the first CI run after each cleanup waits for two deployments to build instead of one. Recommendation: a second workspace in the same organization, which codekiln creates, and one destination deployment that the fixture from task 2.4 rebuilds whenever the cleanup workflow has deleted it.

<ANSWER_HERE>

### 6 - Should prompt import refuse to run without --workspace-id even when LANGSMITH_WORKSPACE_ID is set?

> Context from the drafting agent, for question 6.
> [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks for an explicit destination workspace. Requiring the flag means langstar imports into the workspace the user typed on the command line, even when `LANGSMITH_WORKSPACE_ID` names another, at the cost of one more flag in scripts. Assistant import already requires `--deployment`. Recommendation: require the flag.

<ANSWER_HERE>

### 7 - Should prompt verify report a missing saved model configuration?

> Context from the drafting agent, for question 7.
> [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks import to report missing model configurations, so the draft has prompt verify look for a saved model configuration (`GET /api/v1/playground-settings`) whose settings `id` ends in the class name the export file holds in `requires.model_class`, such as `ChatOpenAI`. Nothing fails at run time without one, because the prompt's manifest carries the model and its settings; the secret check is the one that finds a prompt that cannot authenticate. Reporting it as a failure would make verify exit with 3 for a prompt that runs. Recommendation: report a missing saved model configuration as a note that leaves the exit status alone.

<ANSWER_HERE>

### 8 - Should a failed dependency check stop import before it writes?

> Context from the drafting agent, for question 8.
> [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks import to check the destination's dependencies: the secrets a prompt's model reads, and an assistant's graph and referenced prompts. The draft runs those checks before import writes and writes nothing when one fails, so a user never has to find and delete a copy that cannot run. The cost is order: the destination workspace needs its secrets before a prompt import, and its prompts before an assistant import. The other choice writes the copy anyway and reports the failed checks with status 3 afterwards, which lets a user import in any order and fix the destination later. Recommendation: stop before writing.

<ANSWER_HERE>

### 9 - Should the SDK field additions count as a breaking change?

> Context from the drafting agent, for question 9.
> A new public field breaks only a crate that builds the struct with a struct literal or destructures it with a pattern that names every field; nothing in this repository destructures these structs that way. The only crate known to use the SDK is langstar's own CLI, and each pull request that adds a field updates the CLI's struct literals. `sdk/Cargo.toml` leaves the crate publishable, though, so an outside crate could break, and `docs/dev/git-scm-conventions.md` gives every API break a `BREAKING CHANGE:` footer, which makes the release a major version. The structs that gain fields are `Prompt` (which `PromptClient::get` returns), `CommitManifestResponse` (which `PromptClient::get_commit` returns), `CommitData`, `CommitRequest`, `Assistant` and the assistant request structs. Marking them `#[non_exhaustive]` would stop every crate except the SDK itself from building them with struct literals, even with `..Default::default()`. langstar's own CLI builds `Assistant`, `CommitRequest`, `CreateAssistantRequest` and `UpdateAssistantRequest` with struct literals today, and the SDK's integration tests build all of them except `Assistant` the same way, so the SDK would need a constructor for each of those four structs before the CLI and those tests compile again. After that one break, a new field would break nothing. [🚨 BREAKING CHANGE(sdk): move annotation-queue run calls to the platform items API · Pull Request #815](https://github.com/codekiln/langstar/pull/815) removes SDK functions and changes return types, marks itself breaking, and leaves that choice to codekiln too, so one answer to this question can set the policy for both pull requests. Recommendation: treat the additions as `feat` changes with no `BREAKING CHANGE:` footer, and name the new fields in the release notes.

<ANSWER_HERE>
