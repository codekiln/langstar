# What the live LangSmith API returns for prompt export and import

**Umbrella issue**: [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807)
**Task**: [tasks.md:8, task 1.2, record what the live API returns](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L8), in the `export-import-prompts-and-assistants` change
**Date**: 2026-10-30

Every claim below names the request that produced it. The workspace is the CLI Testing Workspace, the one `LANGSMITH_WORKSPACE_ID` names; this report leaves its ID out, for the reason `mise-tasks/ci/redact-ids-in-junit` gives. Every LangSmith request carried `X-Api-Key: $LANGSMITH_API_KEY` and `X-Tenant-Id: $LANGSMITH_WORKSPACE_ID` against `https://api.smith.langchain.com`. The Agent Server requests carried `X-Api-Key` against the deployment named below.

The prompts this report pushed were deleted afterwards, and so was the assistant it created. It created no deployment.

## A prompt saved with a model: the shape of the commit read

### What export sends, and what it gets back

```
POST /api/v1/commits/-/<handle>
{"manifest": {"lc": 1, "type": "constructor",
  "id": ["langsmith", "playground", "PromptPlayground"],
  "kwargs": {"first": <StructuredPrompt>, "last": <ChatOpenAI>}}}
```

returned HTTP 200. Reading that commit back:

```
GET /api/v1/commits/-/<handle>/latest?include=model
```

returned `commit_hash`, `examples`, `manifest`, `model_config` and `model_provider`. Comparing the manifest sent against the manifest returned shows one change, and only one: the server rewrote the outer `id` from `["langsmith", "playground", "PromptPlayground"]` to `["langchain", "schema", "runnable", "RunnableSequence"]`. Everything under `kwargs` came back as it went in, the secret reference included.

So export writing the manifest as it reads it, and import pushing that manifest back, carries the whole prompt and its model. The outer `id` export writes is already the form the server stores.

### The chat model sits under `last`, sometimes behind a `RunnableBinding`

On the pushed structured prompt, `manifest.kwargs.first` is the `StructuredPrompt`, `id` `["langchain_core", "prompts", "structured", "StructuredPrompt"]`, and `manifest.kwargs.last` is the chat model itself, `id` `["langchain", "chat_models", "openai", "ChatOpenAI"]`.

On prompts saved through the LangSmith playground, a `RunnableBinding` sits in between. These reads each returned `manifest.kwargs.last.id` as `["langchain", "schema", "runnable", "RunnableBinding"]`, with the `ChatOpenAI` constructor one level further in at `manifest.kwargs.last.kwargs.bound`:

```
GET /api/v1/commits/langchain-ai/chain-of-density/latest?include=model
GET /api/v1/commits/langchain-ai/sql-query-system-prompt/latest?include=model
GET /api/v1/commits/nicobutes/chain-of-density-prompt/latest?include=model
```

This decides how export reads `requires.model_class`. [design.md:132, on where export reads the model's class name](../../openspec/changes/export-import-prompts-and-assistants/design.md#L132) has export look at the manifest's `kwargs.last` and step through a `RunnableBinding` there to its `kwargs.bound`, and [specs/prompt/spec.md, "Find the chat model constructor through the RunnableBinding"](../../openspec/changes/export-import-prompts-and-assistants/specs/prompt/spec.md) carries a scenario for each shape. The pushed prompt above meets the scenario where `kwargs.last` is the constructor with no binding around it, which the live API had not been seen to return before this reading. Reading `kwargs.last.id` without the step writes `RunnableBinding` into `requires.model_class` for every playground-saved prompt, and reading `kwargs.last.kwargs.bound` without the shape check finds nothing on a pushed one.

One sentence in design.md is now narrower than what this report holds. It says a `RunnableBinding` wrapped the chat model in every prompt this report read, which was true of the three playground-saved prompts it had read when that sentence was written. The pushed structured prompt recorded above is a fourth reading, and it carries no binding. The spec's two scenarios already cover both shapes, so no behaviour changes; the sentence is what needs a later edit, through the OpenSpec workflow.

### `model_provider` is the chat model's class name

On the pushed structured prompt and on all three playground-saved prompts, `model_provider` is the string `"ChatOpenAI"`, which is the last element of the chat model constructor's `id` in each. So `model_provider` and `requires.model_class` agree, once export has walked to the constructor.

### `model_config` repeats the model's settings beside the manifest

On the pushed structured prompt, `model_config` is `{"model": "gpt-4o-mini", "temperature": 0.3, "openai_api_key": {"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}}` — the same keys and values the `ChatOpenAI` constructor carries in its own `kwargs`. The API key appears as a LangChain secret reference, which is the shape the credential check skips.

### `include=model` changes the manifest alone

```
GET /api/v1/commits/langchain-ai/chain-of-density/latest
```

Without the parameter, the response still holds `model_config` and `model_provider`, with the same values. What changes is the manifest: its outer `id` becomes `["langchain", "prompts", "chat", "ChatPromptTemplate"]`, the prompt on its own, with no model in it.

So export cannot tell from `model_config` and `model_provider` whether it asked for the model. It has to send `include=model` to get the model into the manifest, which is the only place import can carry it, because no field of the create-commit request holds a model.

For a prompt with no model, the response omits `model_config` and `model_provider` rather than returning them as null. These reads each returned `commit_hash`, `examples` and `manifest` and nothing else:

```
GET /api/v1/commits/gitmaxd/synthetic-training-data/latest?include=model
GET /api/v1/commits/leonanluppi/bug_to_user_story_v1/latest?include=model
GET /api/v1/commits/ohkgi/superb_system_instruction_prompt/latest?include=model
```

A reader of the export file therefore cannot read a missing `model_config` as "the server left it empty"; it means the commit has no model.

### `include=model` and `include_model=true` return the same document

```
GET /api/v1/commits/langchain-ai/chain-of-density/latest?include=model
GET /api/v1/commits/langchain-ai/chain-of-density/latest?include_model=true
```

Sorting both responses with `jq -S` and comparing them shows no difference. The spec types them differently — `include` is a string with no enumerated values, `include_model` a boolean defaulting to false, as `reference/api-specs/langsmith/prompt-endpoints.json` records for `GET /api/v1/commits/{owner}/{repo}/{commit}` — and the live API honours either. Export can keep sending `include=model`, the form the Go SDK uses.

## Pushing a prompt into a second place

Without a second workspace, this part pushed into the same workspace under a fresh handle, as [tasks.md:8](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L8) says to.

### The read-back matches

Pushing the manifest export would write — the `RunnableSequence` form read above — to a fresh handle returned HTTP 200. Reading that handle back with `include=model` returned a manifest identical to the one pushed, `model_provider` `"ChatOpenAI"`, and the same `model_config`.

The two commits' hashes differ: the source commit's is `2cdc03b6…` and the copy's is `b61fd175…`, though their manifests match. So compare has to compare manifests, and a commit hash says nothing about whether two prompts hold the same thing.

### An identical push is refused, and the two refusals mean different things

```
POST /api/v1/commits/-/<handle>   the same manifest, no parent_commit
```

returned HTTP 409 and `{"detail": "Parent commit validation failed: reference ID <uuid>"}`. `commits.CreateCommitReq` in `reference/openapi/langchain/langsmith/openapi.json` carries `description`, `manifest`, `parent_commit` and `skip_webhooks`, and a repo that already holds a commit needs `parent_commit` on the next one.

Sending the same manifest again with `parent_commit` set to the current head returned HTTP 409 and `{"detail": "Nothing to commit: prompt has not changed since latest commit"}`, and the commit list still held one commit.

Import has to tell these apart. "Nothing to commit" means the destination already holds this prompt, which is the result a second run of the same import should report as a match rather than a failure. "Parent commit validation failed" means import sent the wrong parent, or none.

### A commit description round-trips

Pushing a changed manifest with `"description": "a described commit"` returned HTTP 200, and the commit read returned that string under `description`. That is the field the SDK group adds to `CommitRequest` and `CommitManifestResponse`.

### The server sets the repo's tags itself

```
GET /api/v1/repos/-/<handle>
```

returned `readme`, `tags`, `tenant_id`, `description` and `is_public` — every repo field export reads. Its `tags` held `["StructuredPrompt"]`, which nothing in the push set: the server derives them from the manifest. Compare therefore has to expect a destination's `repo.tags` to differ from an export file's without anyone having changed the prompt.

## `POST /api/v1/prompts/invoke_prompt` renders a template and never runs a model

The task asks what this endpoint returns for a prompt with a model. It never sees one. It takes no prompt handle and no model.

`InvokePromptPayload` in `reference/openapi/langchain/langsmith/openapi.json` requires `messages`, a list of two-element string pairs, `template_format`, a string, and `inputs`, an object. It has no other property.

```
POST /api/v1/prompts/invoke_prompt
{"messages": [["system", "You are {role}."], ["human", "{q}"]],
 "template_format": "f-string",
 "inputs": {"role": "a helpful assistant", "q": "hello"}}
```

returned HTTP 200 and

```json
{"result": {"messages": [
  {"content": "You are a helpful assistant.", "additional_kwargs": {}, "response_metadata": {}, "type": "system", "name": null, "id": null},
  {"content": "hello", "additional_kwargs": {}, "response_metadata": {}, "type": "human", "name": null, "id": null}]}}
```

It filled the template's variables and returned the rendered messages. It called no model.

Sending a `prompt` key naming `langchain-ai/chain-of-density` and a `model` key holding a `ChatOpenAI` constructor alongside the required fields returned HTTP 200 and the rendered messages for `messages` alone; the server ignored both extra keys. Omitting `template_format` returned HTTP 422 and `{"detail": ["body.template_format: Field required"]}`.

So `invoke_prompt` cannot stand in for running an exported prompt against its model, and prompt verify cannot use it to prove a copied prompt still answers. Anything that runs a prompt with its model has to call the model provider, which is what the OpenAI key in [tasks.md:16, task 2.1, which adds the destination workspace and the model provider key](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L16) is for. That key is unset here, so this report ran no prompt against a model.

## The workspace secrets endpoints

```
GET /api/v1/workspaces/current/secrets
```

returned HTTP 200 and `[{"key": "TEST_KEY"}]`. The response lists names and no values, which matches `SecretKey` in the spec: one required `key` string and nothing else. `SecretUpsert`, the body `POST /api/v1/workspaces/current/secrets` takes, carries `key` and a nullable `value`.

This is the request prompt import's secret check sends: it can ask which secret names the destination workspace holds, and it cannot read a value back. The CLI Testing Workspace holds `TEST_KEY` and no `OPENAI_API_KEY`, so the secret check for a prompt with an OpenAI model fails in this workspace today. [tasks.md:16, task 2.1, which adds the destination workspace and the model provider key](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L16) adds that secret.

`reference/api-specs/langsmith/workspace-secrets-endpoints.json` holds both paths.

## The Agent Server

The deployment is `pr-integration-test-1791547486`, the reusable test deployment the integration fixtures share, READY since 2026-10-09 and serving the one graph ID `test_graph` from `tests/fixtures/test-graph-deployment/langgraph.json`.

Finding it needs `GET /v2/deployments`. `GET /v1/deployments` returns HTTP 404 with a `detail` body and no `resources`, so code that reads `.resources` from that response sees an empty list and concludes the workspace has no deployment.

### A search for a graph ID the deployment does not serve is a 404

```
POST /assistants/search   {"graph_id": "test_graph_alt", "limit": 3}
```

returned HTTP 404 and `{"detail": "Graph 'test_graph_alt' not found. Expected one of: ['test_graph']"}`. The same request with `"graph_id": "test_graph"` returned HTTP 200 and the matching assistants.

So a wrong graph ID is an error naming the graphs the deployment does serve, rather than an empty result. Assistant compare and verify can report the deployment's own list of graph IDs back to the user from that body, and must read the 404 as a wrong graph ID rather than as "no assistant matches".

### A searched assistant carries `context` beside `config`

```
POST /assistants/search   {"limit": 1}
```

returned each assistant with `assistant_id`, `graph_id`, `version`, `created_at`, `updated_at`, `config`, `context`, `metadata`, `name` and `description`. `context` is a top-level object, `{}` when empty.

### Sending both `config.configurable` and `context` is refused, on create and on update

```
POST /assistants         {"graph_id": "test_graph", "config": {"configurable": {…}}, "context": {…}, …}
PATCH /assistants/<id>   {"config": {"configurable": {"x": "1"}}, "context": {"y": "2"}}
```

each returned HTTP 400 and `{"detail": "Cannot specify both configurable and context. Prefer setting context alone. Context was introduced in LangGraph 0.6.0 and is the long term planned replacement for configurable."}`.

This is why import sends an assistant's settings in one field, as [design.md](../../openspec/changes/export-import-prompts-and-assistants/design.md) decided. Sending both is more than redundant; the server rejects it.

### The server keeps `config.configurable` and `context` mirrored

Creating an assistant with `"config": {"configurable": {"alpha": "1", "beta": "2"}, "recursion_limit": 7}` and no `context` returned an assistant whose `config.configurable` held `alpha` and `beta` **and** whose `context` held the same two keys. `recursion_limit` stayed in `config` alone.

Every later update kept the two in step. A `PATCH` sending `"context": {"gamma": "3"}` left both `config.configurable` and `context` holding `{"gamma": "3"}`. A `PATCH` sending `"config": {"configurable": {"delta": "4"}}` left both holding `{"delta": "4"}`. So an update that sends `configurable` alone does replace `context` too.

A `PATCH` sending `"config": {"recursion_limit": 9}`, with no `configurable` in it, left `context` at its previous `{"delta": "4"}`.

### `config` and `context` are replaced; `metadata` merges

A `PATCH` sending `"context": {"gamma": "3"}` against an assistant whose `context` held `alpha` and `beta` left `context` holding `gamma` alone, and left `config` as `{"configurable": {"gamma": "3"}}` — the `recursion_limit` that `config` had been carrying was gone. A `PATCH` sending `"config": {"recursion_limit": 9}` left `config` as that object alone, with no `configurable` in it. A `PATCH` sending `"context": {}` left both `config` and `context` empty.

`metadata` behaves the other way. Against an assistant whose metadata held `drop` and `keep`, a `PATCH` sending `{"metadata": {"fresh": "yes"}}` returned metadata holding `drop`, `fresh` and `keep`.

Import therefore sends the whole settings object every time, and cannot remove a metadata key by leaving it out.

### A null metadata value stores null; it does not remove the key

A `PATCH` sending `{"metadata": {"drop": null}}` against an assistant whose metadata held `"drop": "later"` returned metadata holding `"drop": null`, beside the untouched `"keep": "yes"`. The key stayed, with null as its value.

So an export file that carries a null metadata value writes a null into the destination, and compare sees a key holding null rather than a missing key. No request in this report removed a metadata key.

### An update clears the description with the empty string, and a null is refused

A `PATCH` sending `{"description": ""}` returned HTTP 200 and left `description` as the empty string. A `PATCH` sending `{"description": null}` returned HTTP 422 and `{"detail": "null is not of type \"string\" …"}`, and left the assistant as it was.

### `version` counts the updates that succeeded

The assistant's `version` was 1 on create and rose by one on each `PATCH` that returned HTTP 200, reaching 9. The two refused updates, the null description and the one sending both `configurable` and `context`, left it where it was.

## Related

- [FRAGMENTS.md](../api-specs/langsmith/FRAGMENTS.md) — the jq queries behind `prompt-endpoints.json` and `workspace-secrets-endpoints.json`
- [402-structured-prompts-openapi-validation.md](402-structured-prompts-openapi-validation.md) — structured output prompts
- [461-openapi-validation.md](461-openapi-validation.md) — playground settings, which prompt verify reads for a saved model configuration
