# Design

## Context

`langstar prompt push` in `cli/src/commands/prompt.rs` builds one shape of prompt: a single human message, with an optional schema from [#402 structured output prompts](https://github.com/codekiln/langstar/issues/402). `langstar prompt pull` prints any manifest LangSmith returns, but `PromptClient` in `sdk/src/prompts.rs` never asks LangSmith for the model, so that manifest has none. `PromptClient` keeps the manifest as raw JSON and has a typed `StructuredPrompt` view that holds messages, the schema and the method.

Langstar's assistant SDK lacks what an export needs: `Assistant` in `sdk/src/assistants.rs` has no `context` or `version` field, and the search in `AssistantClient` cannot filter by graph ID. The assistant commands themselves already work against the test deployment: `test_assistant_description_field` in `cli/tests/assistant_command_test.rs` creates and reads an assistant there in the Integration Tests CI job.

Langstar reads one API key and one workspace at a time, from flags, environment variables or the config file. Langstar exits with status 1 when a command fails, and clap, the library that parses langstar's flags, exits with status 2 when it rejects a mistyped flag or a missing required one.

## Goals / Non-Goals

**Goals:**

- Recreate a prompt or an assistant from its export file with nothing lost. A prompt keeps its manifest and model, its commit description and tags, and its repo details. An assistant keeps its name, description, graph ID, configuration, context and metadata.
- Make a repeated import of the same file safe: a second import of one file finds the copy the first one made and changes nothing until the user asks.
- Give users one way to check a copy: the same report shape and exit statuses for both resource types.

**Non-Goals:**

- Copy a prompt or assistant between environments in one command.
- Export or import every prompt in a workspace, or every assistant on a deployment, in one run.
- Import onto an Agent Server the control plane does not list, such as a local `langgraph dev` server. The assistant commands find a deployment with `--deployment <name-or-id>`, which [`resolve_deployment_url` in cli/src/deployment_utils.rs:23](https://github.com/codekiln/langstar/blob/7850530475f49eb6374652855d6f3a04741899d3/cli/src/deployment_utils.rs#L23) looks up in the control plane's deployment list, and the destination record keys an assistant by that deployment's ID.
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

### Export refuses to write a file that holds a credential

Before it writes anything, export runs a credential check over every string it would put in the `repo`, `commit` and `assistant` objects, because a secret pushed to Git stays in the repository's history after someone deletes the file. For a prompt, the check reads the manifest with each model setting, header, `model_kwargs` entry, message template and `metadata` value in it, `commit.model_config`, the commit description and the repo's description and readme. For an assistant it covers the name, description, configuration, context and metadata. The check skips a LangChain secret reference, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, because it holds only a secret's name, and skips the empty string.

The check flags a string when the name of its key or the shape of its value marks it as a credential.

- **By key name:** the check flags a string whose key, or the key of the array it sits in, holds one of the words `secret`, `token`, `password`, `passwd`, `pwd`, `credential`, `credentials`, `apikey`, `authorization` or `cookie`, or holds `key` or `keys` right after `api`, `access`, `private`, `secret`, `signing` or `account`. It splits a key name into words at `_`, `-`, `.`, spaces and each change from a lower-case to an upper-case letter, and ignores case. It flags `openai_api_key`, `OPENAI_API_KEY`, `x-api-key`, `Authorization`, `access_token`, `client_secret`, `db_password`, `privateKey` and `api_keys`, and passes `max_tokens` and `cache_key`.
- **By value shape**, under any key: the check flags a string that holds a bearer token (`Bearer ` followed by at least 16 token characters) or a `Basic ` value followed by at least 16 base64 characters; a PEM private key (`-----BEGIN` and `PRIVATE KEY-----`); a URL with a password in it, `scheme://user:password@host` or `scheme://:password@host`, which covers database and queue connection strings; a key-value connection string with a non-empty `Password=`, `Pwd=`, `AccountKey=`, `SharedAccessKey=` or `SharedAccessSignature=`; a JSON Web Token, three base64url parts that start with `eyJ`; or a token with a provider's published prefix, which are `sk-` (OpenAI and Anthropic), `lsv2_` (LangSmith), `ghp_`, `gho_`, `ghs_` and `github_pat_` (GitHub), `xoxb-`, `xoxp-` and `xoxa-` (Slack), `AKIA` and `ASIA` followed by 16 upper-case letters or digits (AWS), `AIza` (Google) and `hf_` (Hugging Face), each followed by enough characters to rule out an ordinary word.

When the check flags any string, export writes no file and nothing to stdout, lists the JSON Pointer path of each flagged string with the rule that flagged it, such as `key name` or `URL with a password`, leaves the string and every part of it out of the message, and exits with status 1. Export refuses rather than writing `<redacted>` in place of the credential. A redacted file would import a prompt or assistant that cannot authenticate, and a reviewer reading the file in a pull request would not see that the source holds a credential. The refusal asks the user to move the credential into the workspace's secrets, which a prompt's model reads through a secret reference, or into the deployment's environment variables, which a graph reads.

Some flagged strings are harmless, such as a message template that shows `Bearer <your-token>` followed by an example token. A user who has read such a string names its path with `--not-a-secret <json-pointer>`, once for each path, and export writes the string and prints a warning to stderr naming the path. When a pointer names a string the check lets through, export writes nothing, names the pointer and exits with status 1, so a script that keeps naming a path after the string there has changed stops at the export. Every export runs the check.

A credential under an ordinary key name with an ordinary-looking value, such as a random string under a key called `x`, passes the check and reaches the file. The export docs tell users to keep credentials in secrets and environment variables, and to read an export file before committing it.

### Export asks LangSmith for the model

Export reads the commit with `include=model`, the form the Go SDK uses (`references/langsmith-go.md`). The Python SDK sends the older `include_model=true`, and the LangSmith OpenAPI spec accepts both. With the model included, the manifest wraps the prompt and the chat model with its settings in one constructor, and the response also carries `model_config` and `model_provider` beside the manifest (`references/langsmith-sdk.md`). No field of the create-commit request holds the model, so the model reaches the destination inside the manifest, and export keeps `model_config` and `model_provider` in the file only for the reader. The LangSmith Python SDK's [`PromptCommit`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/schemas.py#L1054) documents `model_provider` as the model's class name, such as `ChatOpenAI`. In [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8), a developer pushes that manifest to a second workspace and records whether LangSmith returns the same manifest, `model_config` and `model_provider`, and which outer `id` the live API returns.

### Import records the IDs it created in a separate file next to the export

Import writes the IDs of what it created to `<file stem>.destinations.json`, beside the export file, keyed by destination workspace ID for prompts and by deployment ID for assistants. A separate file keeps the two apart: export never has to carry an earlier record forward when it overwrites its file, the file a reviewer reads in Git changes only when the source changes, and a fresh export never wipes out the record of earlier imports. Import replaces the record by writing the whole new record to a temporary file in the same directory and renaming it over the old one, so a langstar that stops in the middle of a write leaves the earlier record whole, pending assistant IDs included.

Langstar supports one import of an export file at a time, because a person runs import by hand from a Git checkout. When two imports of the same file run at once, each reads the record before the other writes it, and the import that writes last drops the other's entries. Two first imports of one assistant into the same deployment can also create two assistants. [#828 make concurrent prompt and assistant imports into one deployment safe](https://github.com/codekiln/langstar/issues/828) covers making concurrent imports safe.

A repeated prompt import finds the copy by its handle and uses the record to tell whether someone changed it. Import writes the new commit hash to the record before it reads the commit back, so a read-back that finds a difference still leaves the hash in the record. The next import compares the destination's latest commit with that hash, and reports a change only when someone else pushed a commit after langstar's. Langstar can stop after it pushes a commit and before it writes that commit's hash to the record. The next import finds the destination's latest commit matching the file, writes its hash to the record and reports the prompt unchanged.

An assistant's ID differs on every deployment, so the record is the only way a repeated import finds the assistant it made. Import chooses the new assistant's ID itself and writes it to the record, marked pending, before it sends the create request with that `assistant_id` and `if_exists` set to `do_nothing`, which the Agent Server's `AssistantCreate` accepts. After the create succeeds, import marks the entry created. If langstar stops before then, the next import finds the pending entry and sends the same create again, and the Agent Server returns the assistant the first import made. Assistant compare and verify find the assistant through the same record, and take `--record <path>` as import does, so a record kept at another path still works.

### Import changes an existing resource only with --update

A prompt that exists in the destination gets a new commit only with `--update`, and its parent is the destination's latest commit, so LangSmith keeps the earlier history. Import sends the file's commit description with each commit it creates. With `--update`, import also sets the destination repo's description, readme, tags and visibility to the file's values through `PATCH /api/v1/repos/{owner}/{repo}` when they differ. When only those repo fields differ and the manifest matches, import updates the repo and adds no commit.

Import treats the destination commit's tags as one more field to match. A tag in the file that the destination lacks, or that points at another commit there, is a difference, and so is a tag on the destination's matching commit that the file lacks. With `--update`, import creates or moves each of the file's tags to that commit and deletes each tag on it that the file lacks, through `DELETE /api/v1/repos/{owner}/{repo}/tags/{tag_name}`, so the read-back finds the same tags as the file. It adds no commit when the latest commit already matches. A tag on an older destination commit is not part of the comparison, so import leaves it alone. The tag endpoints take a commit's `id`, which differs from its hash. The commit read and the repo read return no `id`, so import looks the commit up in the commit list, `GET /api/v1/commits/{owner}/{repo}`, whose entries carry both the `id` and the hash.

An import that stops after it creates the repo and before it pushes the commit leaves a prompt with no commit. The next import finds the prompt by its handle and, through the commit list, finds no commit. It then pushes the file's manifest as the first commit and creates the file's tags, as it does for a new prompt, without `--update`. The repo details it created hold the file's values already, and any that someone changed in between still need `--update`: without it, import pushes the commit and tags, leaves those repo details alone, lists them as differences and exits with status 3.

With `--update`, import makes an existing assistant match the file and asks the Agent Server to remove what the file lacks. The Agent Server keeps the name, graph ID and description an update leaves out, and merges the metadata an update sends into the metadata the assistant already has, as `AssistantPatch` in `reference/openapi/langchain/agent-server/openapi.json` documents. So import sends the file's name, graph ID and settings in full, and a null value for each metadata key the destination has and the file lacks. [Import sends an assistant's settings in one field](#import-sends-an-assistants-settings-in-one-field) says which of the two settings fields import sends and which it leaves alone. `AssistantPatch` documents no way to clear a description, so in [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) a developer sends a null and an empty description to the live server, along with a null metadata value and a replaced configuration and context, and records what the server keeps. Import then reads the assistant back, and any key or field the file lacks that is still there shows up as a difference, with exit status 3. The Agent Server saves each update as a new version of the assistant.

Without `--update`, import lists the differences and exits with status 3. `--dry-run` reports the action import would take and writes nothing.

### Import sends an assistant's settings in one field

An assistant keeps its settings in `config.configurable`, the older field, and in `context`, which arrived with LangGraph 0.6. When langstar reads an assistant, the Agent Server returns the same settings in both, so an export file holds them twice. A newer Agent Server refuses to create an assistant that sets both, with status 400 and `Cannot specify both configurable and context. Prefer setting context alone.` An import that sent the file's fields as they stand would fail on that server.

So import sends one field:

- When the file has both and they hold the same values, import sends `config` with `configurable` and leaves `context` out. The server fills `context` from `configurable` itself, so the read-back matches the file and compare finds no difference. A trial run that copied assistants from one deployment to a newer one sent `config` this way, and every copy matched its file.
- When the file has only one of them, import sends that one.
- When the file has both and they differ, import cannot tell which one the user meant. It writes nothing, names `/assistant/config/configurable` and `/assistant/context` without their values, and exits with status 1, the way it rejects a file of another kind.

Compare reads an assistant's settings once, because the two fields hold one value. It takes the file's settings from whichever field the file holds and the destination's from whichever the server returned, and reports a difference only when the values differ. A settings field one side carries and the other lacks is no difference, since the server fills the second field from the first. So a file that holds settings only in `config.configurable` imports into a server that answers with both, and compare reports nothing.

`--update` sends the settings in one field the same way, and leaves the other field alone. An empty `context` sent beside a `config.configurable` would either be refused by a newer server or wipe out the settings the server had just filled in from `configurable`. Import clears `context` only for a file that holds settings in neither field. In [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8), a developer records whether an update that sends both is refused as a create is, and whether an update that sends `configurable` alone replaces `context` too.

Export still writes both fields as the server returned them, so the file shows what the source held.

### Compare reports differences by JSON path

Compare reads the prompt or assistant from the destination, puts it in the shape of the export file's `repo`, `commit` and `assistant` objects, and lists every value that differs from the file. Each difference names its JSON Pointer path and both values. Compare checks every key in those objects, including the chat model's settings and the `id` arrays inside a manifest that name the prompt and model classes, such as `ChatOpenAI`. For a prompt, it also covers the commit description, the commit tags and the commit each one points at, any tag on the compared commit that the file lacks, and the repo's description, readme, tags and visibility. Import runs the same comparison after it writes, so a field LangSmith or the Agent Server drops shows up as a difference.

Compare skips `commit.model_config` and `commit.model_provider`, because LangSmith fills both in itself when import pushes a commit, and a destination workspace can fill them in differently from the source. Export leaves the values the server sets out of those objects altogether — IDs, version numbers, timestamps, owner and like counts — and keeps the source environment's workspace, deployment and IDs in `source`, so none of them reads as a difference against a copy.

The report goes to a terminal or a CI log, so it hides credentials before it prints. It prints `<redacted>` in place of every string export's credential check flags, by either rule, whether the difference sits at that string or at an object holding it. A credential someone pasted into the destination prompt, the destination assistant or the file therefore stays off the screen while the user still sees where each difference is. A LangChain secret reference, such as `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`, prints unchanged, because it holds the secret's name and never its value.

### `prompt verify` and `assistant verify` answer whether the copy can run

A user who has just imported a prompt or an assistant wants one question answered: will it run here? A create request that came back 200 does not answer it. The copy runs only if the destination workspace holds the API keys the prompt's model reads, the destination deployment serves the graph the assistant names, and the destination workspace holds the prompts the assistant refers to. Verify asks the destination about each of those and names what is missing, so the user fixes it before a learner or a test hits the copy.

Prompt verify starts with the workspace's secrets. A prompt's model needs an API key, and the manifest names the key rather than carrying its value, as `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`; that is the shape the Python SDK's `_manifest_has_secrets` matches. Export walks the manifest for those references and writes each name into `requires.secrets`. Verify reads the workspace's secrets (`GET /api/v1/workspaces/current/secrets`, the request `langstar secrets list` already sends) and prints the name of every one the workspace lacks. It prints names, never values.

Prompt verify then looks for a saved model configuration for the prompt's model, and reports a missing one as a note. The prompt still runs without one, because its manifest carries the model and its settings. Export reads the model's class name, such as `ChatOpenAI`, from the last element of the chat model constructor's `id` in the manifest, such as `["langchain", "chat_models", "openai", "ChatOpenAI"]`, and writes it to `requires.model_class`. Verify then looks for a saved model configuration whose settings `id` ends in that class name (`GET /api/v1/playground-settings`, the request `langstar model-config list` sends). A prompt someone saved without a model has no chat model constructor, so export writes `requires.model_class` as null, and verify and import skip this check and still check the secrets. [Prompt verify reports a missing saved model configuration as a note, not a failure](#prompt-verify-reports-a-missing-saved-model-configuration-as-a-note-not-a-failure) gives the reason.

Import and verify both read the file's manifest and work out `requires` from it again, then compare that with the `requires` the file carries. When the two disagree, they write nothing, name the field and exit with status 1. This catches a manifest someone edited in Git and left `requires` behind.

Assistant verify asks the destination deployment whether it serves the graph the assistant names. The Agent Server has no call that lists a deployment's graphs, so verify searches the deployment's assistants for that graph ID (`POST /assistants/search` with `graph_id`). Every graph a deployment serves has a default assistant, so a deployment that serves the graph answers with at least that one, and an empty answer means the deployment does not serve it. Verify then names the graph ID as missing and the user picks a deployment that serves it.

The Agent Server may refuse the search instead of answering it with nothing: its OpenAPI spec lists the deployment's graph IDs as the allowed values of `graph_id`, so an unknown graph ID may come back as a 422. Verify reads a 422 whose error names `graph_id` the same way it reads an empty answer, as a graph this deployment does not serve, and exits with status 1 on every other error. In [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8), a developer runs the search against the live server and records which of the two answers it gives.

Assistant verify also checks each prompt the assistant refers to. `requires.prompts` holds names such as `support-system:prod`, and verify asks LangSmith for the commit each tag points at (`GET /api/v1/commits/{owner}/{repo}/{tag}`, the request `langstar prompt pull --commit prod` already sends). A name has the form `[<owner>/]<handle>[:<tag>]`: with no owner, verify uses `-`, which LangSmith reads as the current workspace, and with no tag it asks for `latest`. A 404 means the workspace lacks that prompt or that tag, and verify prints the name.

### Import runs verify's checks before it writes anything

Import runs the checks `verify` runs, and it runs them before it creates anything. Prompt import stops when the destination workspace lacks a secret from `requires.secrets`. Assistant import stops when the deployment does not serve the file's graph, or when the workspace — the one from `--workspace-id`, or the configured one — lacks a prompt from `requires.prompts`. Either one names what is missing, writes nothing and exits with status 3, and `--dry-run` reports the same failures.

Checking first saves the user a cleanup. An import that creates a prompt whose API key the destination workspace lacks leaves behind a copy that cannot run, and the user has to find that copy and delete it by hand. Running the checks first costs only the order of the steps. [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks import to check the destination's dependencies.

A user therefore creates the destination workspace's secrets before importing a prompt, and imports the prompts before importing an assistant that refers to them.

### Users name an assistant's prompt references with `--prompt-ref`

An assistant refers to a prompt through whatever configuration key its graph reads, so langstar cannot find those references by itself, and guessing from strings that look like prompt handles would miss some and flag others. The user names each one with `assistant export --prompt-ref <json-pointer>`. The assistant's configuration should name each prompt by handle and tag, such as `support-system:prod`, because the same prompt gets a different commit hash in each workspace. Import copies the commit tags to the destination workspace, so when the graph on the destination deployment asks LangSmith for `support-system:prod`, it gets the same prompt. Compare, verify and import look up each pointer from `requires.prompts` in the file's `assistant` object, and exit with status 1 when the value there differs from the prompt name recorded beside the pointer. This catches a configuration someone edited in Git without updating `requires.prompts`.

### compare, verify and import exit with 3 when they find a difference

Status 3 is new. Langstar exits today with 0 when a command succeeds, with 1 from `main` in `cli/src/main.rs` when a command returns an error, and with clap's 2, so 3 is the first status free to carry a meaning of its own. Status 0 means the resource matches and every check passed. Status 3 means langstar found a difference or a failed check. Status 1 means any other error.

Status 2 stays clap's: `Cli::parse()` in `run` in `cli/src/main.rs` exits with 2 when a user mistypes a flag or leaves out a required one, and this change leaves that alone, so a script never reads a typo as a difference. `prompt import` declares `--workspace-id` optional, as the other `langstar prompt` subcommands do, and checks for the flag itself, so a user who leaves it out gets status 1 and an error naming `--workspace-id` where clap would have exited with 2. `diff` uses 1 for differences and 2 for errors; langstar keeps 1 for errors, so a script testing for failure sees the same status from every command.

### An optional Python script checks that an imported structured prompt still returns structured data

The LangSmith API has no documented way to run a prompt with its model, so a short Python script runs each imported structured prompt with LangChain and checks that the result matches the prompt's schema. The one prompt endpoint in the OpenAPI spec, `POST /api/v1/prompts/invoke_prompt`, takes message templates and inputs with no model and no schema, and no LangSmith SDK calls it. In [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8), a developer calls it on the live API in case it does more than the spec says.

A mise task runs the script with `uv run --locked --script` against a committed script lockfile, which installs the `langsmith` and `langchain-openai` packages the script lists, on a developer's machine and in the Integration Tests CI job. The script reads the workspace import wrote to: the import test passes it the workspace ID it gave `prompt import --workspace-id` and the handle it imported under, and the script builds its LangSmith `Client` with `workspace_id` set to that ID rather than reading `LANGSMITH_WORKSPACE_ID`. That is the existing test workspace today, and the destination workspace once [export-import-prompts-and-assistants Task 2.1 Add the destination workspace and the model provider key](tasks.md#L16) adds one.

The script pulls the imported prompt with `include_model=True` and passes `secrets={"OPENAI_API_KEY": <value read from the environment>}` to [`pull_prompt`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L10228), because the LangSmith Python SDK reads no secret from the environment unless asked, and raises when a manifest holds a secret reference and no `secrets` map arrives. Running the prompt bills an OpenAI API key on every run, so the mise task skips the check and says so when `OPENAI_API_KEY` is unset. Every other test runs without a paid key, and a developer with a key of their own runs this one by hand.

### Export and import use the same SDK calls as the planned `prompt create`, `get` and `update`

[#728 Implement CRUD command structure (create, get, update) for prompt commands](https://github.com/codekiln/langstar/issues/728) plans `prompt create`, `get` and `update` commands that edit a prompt's text, model or structured output one field at a time. Export and import read and write the whole prompt at once, through the same `PromptClient` calls in `sdk/src/prompts.rs` that those commands will use, so a field the SDK reads for export is there for `prompt get` too.

### Developers commit the new SDK fields as `✨ feat(sdk)`, so the next release is a minor version

Developers commit the SDK's new fields and calls as `✨ feat(sdk)`, with no `BREAKING CHANGE:` footer and no `!` after the type, and the Prepare Release workflow sets the version in `Cargo.toml`. The new fields go on `Prompt`, `CommitManifestResponse`, `CommitData`, `CommitRequest`, `Assistant`, `CreateAssistantRequest`, `UpdateAssistantRequest` and `AssistantSearchRequest`, in [export-import-prompts-and-assistants Task group 3 SDK reads and writes the fields export and import need](tasks.md#L21). The repository's release docs ask for this:

- [docs/dev/git-scm-conventions.md:26](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/docs/dev/git-scm-conventions.md#L26) gives `✨ feat` a minor version bump, and [docs/dev/git-scm-conventions.md:28](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/docs/dev/git-scm-conventions.md#L28) keeps the major bump for `🚨 BREAKING CHANGE`. [docs/dev/ci-cd.md:209](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/docs/dev/ci-cd.md#L209) also gives a major bump to a commit with a `BREAKING CHANGE:` footer or a `!` before the colon.
- [docs/dev/procedures.md:116](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/docs/dev/procedures.md#L116) names changing function signatures and removing fields as breaking SDK changes. Adding a field or a call is neither.
- The Prepare Release workflow, [.github/workflows/prepare-release.yml:62](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/.github/workflows/prepare-release.yml#L62), picks the bump by running the bump-release skill's `analyze_commits.py` over the commits since the last tag, and writes the new version itself. git-cliff writes the changelog from the same commit messages, so nobody writes release notes or a version by hand.
- [release.toml:5](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/release.toml#L5) sets `publish = false`, which keeps `langstar-sdk` off crates.io, so the only crate known to build these structs is the CLI in this repository.
- [4a57ccb ✨ feat(assistant): add description field support from Agent Server API (#726)](https://github.com/codekiln/langstar/commit/4a57ccb6802e988a1210ba8a8b1434a9bf9a803e) added `description` to `Assistant`, `CreateAssistantRequest` and `UpdateAssistantRequest` with no footer, and langstar released it in v2.2.0, a minor version after v2.1.2.

Developers leave the structs as they are, without `#[non_exhaustive]`, because adding that attribute would stop the CLI and the SDK's integration tests from building `Assistant`, `CommitRequest`, `CreateAssistantRequest` and `UpdateAssistantRequest` with struct literals, which [docs/dev/procedures.md:116](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/docs/dev/procedures.md#L116) counts as a breaking change.

### Prompt import requires `--workspace-id`, even when `LANGSMITH_WORKSPACE_ID` is set

Import writes, and `LANGSMITH_WORKSPACE_ID` is easy to leave pointing at the wrong workspace: `fnox.toml` and CI set it to the source workspace, so a fallback would put the copy back where it came from without a word. Requiring the flag puts the destination on the command line, where the user and the shell history can see it, as assistant import does with `--deployment`.

### Prompt verify reports a missing saved model configuration as a note, not a failure

The prompt's manifest carries the model and its settings, so the prompt runs without a saved model configuration, and the secret check is the one that finds a prompt that cannot authenticate. Failing on the note would make verify exit with 3 for a prompt that runs.

### Developers add the second workspace and deployment before they archive the change

The integration tests import each prompt into the existing test workspace under a fresh handle, and each assistant onto the existing test deployment as a new assistant. That covers import, compare, verify and the destination record, but not a move between workspaces or deployments. [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807) asks for that move: assistant tests that create an assistant on a separate deployment, and an assistant whose prompts a developer imported into a separate workspace. When the source and the destination are one workspace and one deployment, an import that ignores `--workspace-id`, or that writes to the deployment it read from, passes every test anyway, because it writes where the test looks. The tests that need a second environment carry an `#[ignore]` naming the task that adds it: the assistant graph moves to `test_graph_alt` in [export-import-prompts-and-assistants Task 3.3 Add the assistant fields to the SDK](tasks.md#L29) and [export-import-prompts-and-assistants Task 10.2 Add assistant import --update](tasks.md#L158) need the destination deployment from [export-import-prompts-and-assistants Task 2.4 Add a destination deployment fixture](tasks.md#L19), and the test in [export-import-prompts-and-assistants Task 9.2 Add assistant verify](tasks.md#L138) of a prompt that only the destination workspace holds needs the workspace from [export-import-prompts-and-assistants Task 2.1 Add the destination workspace and the model provider key](tasks.md#L16). Developers merge [export-import-prompts-and-assistants Task 2.1 Add the destination workspace and the model provider key](tasks.md#L16), [export-import-prompts-and-assistants Task 2.3 Register the test graph under a second graph ID](tasks.md#L18) and [export-import-prompts-and-assistants Task 2.4 Add a destination deployment fixture](tasks.md#L19) before anyone archives the change. The pull request for each one removes the `#[ignore]` from the tests that waited for it, and the pull request that archives the change links a CI run in which those tests passed. If the control plane answers [export-import-prompts-and-assistants Task 2.3 Register the test graph under a second graph ID](tasks.md#L18) with a 409, this repository cannot hold a second deployment, and the developer reports that answer to codekiln, who decides what the assistant tests run against.

Trying export and compare on a real set of assistants and prompts, with langstar's current commands and the SDK, showed that one workspace and one test deployment are enough for the rest of the work. Only the assistants needed to move; the prompts stayed in one shared workspace, and the assistants name them without an owner or a commit. Every failure the trial found shows up in an export, or in an import to any deployment. The trial needed no live model call either: comparing each structured prompt's schema and model settings finds a bad copy, and a live call tests the model, not the copy.

## Risks / Trade-offs

- [LangSmith may store a pushed model differently from the source] → Import reads the prompt back and reports any difference, and in [export-import-prompts-and-assistants Task 1.2 Record what the live API returns](tasks.md#L8) a developer checks the round trip against the live API before anyone builds the commands.
- [Pushing a manifest identical to the latest commit may return 409] → Import compares first and pushes nothing when the prompt matches.
- [The SDK changes for export and import add public fields to `Prompt`, `CommitManifestResponse`, `CommitData`, `CommitRequest`, `Assistant` and the assistant request structs, so a crate that builds one of those structs with a struct literal stops compiling] → `release.toml` keeps `langstar-sdk` off crates.io, and the developer who adds a field updates every struct literal of that struct in the same pull request, as [docs/dev/procedures.md:116](https://github.com/codekiln/langstar/blob/ab158b14160cf7803a79384f88756911bcd21857/docs/dev/procedures.md#L116) asks. [Developers commit the new SDK fields as `✨ feat(sdk)`, so the next release is a minor version](#developers-commit-the-new-sdk-fields-as--featsdk-so-the-next-release-is-a-minor-version) gives the release policy.
- [A prompt or an assistant may hold a credential, such as an API key, a token, a password, an authorization header, a connection string with a password or a private key, which export would copy into a file meant for Git] → Export runs the credential check from [Export refuses to write a file that holds a credential](#export-refuses-to-write-a-file-that-holds-a-credential) and, when it flags a string, writes nothing, names each path and exits with status 1. Only `--not-a-secret <json-pointer>` lets a flagged string through, one path at a time.
- [A credential under an ordinary key name with an ordinary-looking value, such as a random string under a key called `x`, passes the credential check] → The export docs in [export-import-prompts-and-assistants Task 5.2 Document prompt export](tasks.md#L67) and [export-import-prompts-and-assistants Task 8.2 Document assistant export](tasks.md#L125) list what the check catches and what it cannot, tell users to keep credentials in the workspace's secrets or the deployment's environment variables, and tell them to read an export file before committing it.
- [The check flags a string that holds no secret, such as an example token in a message template] → Export names the path and the rule, and the user passes `--not-a-secret` with that path after reading the string.
- [A tag in the destination may already point at another commit] → `--update` moves the tag, and `--dry-run` lists the move.
- [`--update` deletes a tag on the matching destination commit that the file lacks, and a graph on a deployment may read that tag] → `--dry-run` lists each delete, and `assistant verify` reports a prompt tag an assistant refers to that the workspace no longer has.
- [The control plane may refuse a second deployment from this repository with 409] → In [export-import-prompts-and-assistants Task 2.3 Register the test graph under a second graph ID](tasks.md#L18), a developer deploys the new `langgraph.json` once, before anyone builds the fixture. If the control plane answers with 409, the developer records the answer in this design.

## Resolved Questions

None.

## Open Questions

None.
