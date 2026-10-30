# What the live LangSmith API returns for prompt export and import

**Umbrella issue**: [#807 Export, import, compare, and verify prompts and assistants across LangSmith environments](https://github.com/codekiln/langstar/issues/807)
**Task**: [tasks.md:8, task 1.2, record what the live API returns](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L8), in the `export-import-prompts-and-assistants` change
**Date**: 2026-10-30
**Status**: Partial. The questions about the commit read and about `invoke_prompt` are answered. The questions that need a push, and every question about the Agent Server, are not. [What this report does not answer](#what-this-report-does-not-answer) says why, question by question.

Every claim below names the request that produced it. The workspace is the CLI Testing Workspace, the one `LANGSMITH_WORKSPACE_ID` names; this report leaves its ID out, for the reason `mise-tasks/ci/redact-ids-in-junit` gives. Every request carried `X-Api-Key: $LANGSMITH_API_KEY` and `X-Tenant-Id: $LANGSMITH_WORKSPACE_ID`, and the host was `https://api.smith.langchain.com`.

## No prompt in the test workspace carries a model

This request listed the workspace's own prompts and returned only the `langstar-structured-test-*` repos the integration tests leave behind, every one of them a structured prompt with no model.

```
GET /api/v1/repos/?limit=100&is_public=false
```

So the reads below use `langchain-ai/chain-of-density`, a public prompt that is saved with a model, at commit `8c0ba491ce9b01f707e56db935d7eeee7aa6ff29c04258b458928cf7c8ebe35b`. Its prompt is a `ChatPromptTemplate`, not a `StructuredPrompt`. [What this report does not answer](#what-this-report-does-not-answer) says what that leaves open.

## The commit read with `include=model`

```
GET /api/v1/commits/langchain-ai/chain-of-density/latest?include=model
```

The response holds `commit_hash`, `examples`, `manifest`, `model_config` and `model_provider`.

The manifest's outer `id` is `["langchain", "schema", "runnable", "RunnableSequence"]`. Its `kwargs` holds `first` and `last`. `first` is the prompt, `id` `["langchain", "prompts", "chat", "ChatPromptTemplate"]`.

`last` is **not** the chat model constructor. Its `id` is `["langchain", "schema", "runnable", "RunnableBinding"]`, and its `kwargs` holds `bound` and `kwargs`. The chat model constructor sits one level further in, at `manifest.kwargs.last.kwargs.bound`, whose `id` is `["langchain", "chat_models", "openai", "ChatOpenAI"]` and whose `kwargs` holds `model`, `temperature`, `stream_usage` and `openai_api_key`. The binding's own `kwargs` is the empty object on this commit.

This matters for `requires.model_class`. [design.md:132, on where export reads the model's class name](../../openspec/changes/export-import-prompts-and-assistants/design.md#L132) says export reads it from the last element of the chat model constructor's `id`. On this commit the last element of `manifest.kwargs.last.id` is `RunnableBinding`, and the class name `ChatOpenAI` is the last element of `manifest.kwargs.last.kwargs.bound.id`. Export has to walk through the `RunnableBinding` to reach the constructor. Whether LangSmith ever returns the constructor directly under `last`, with no binding around it, is open: the two other model-saved prompts this report read, `langchain-ai/sql-query-system-prompt` and `nicobutes/chain-of-density-prompt`, both carry the binding.

```
GET /api/v1/commits/langchain-ai/sql-query-system-prompt/latest?include=model
GET /api/v1/commits/nicobutes/chain-of-density-prompt/latest?include=model
```

Each of those returns the same arrangement: outer `RunnableSequence`, `first` a `ChatPromptTemplate`, `last` a `RunnableBinding`, `bound` a `ChatOpenAI`, and `model_provider` `"ChatOpenAI"`.

### `model_provider` matches the bound class name

On the `chain-of-density` read above, `model_provider` is the string `"ChatOpenAI"`. That is the last element of `manifest.kwargs.last.kwargs.bound.id`. It is not the last element of `manifest.kwargs.last.id`, which is `RunnableBinding`. So `model_provider` and the class name export writes to `requires.model_class` agree, as long as export reads the class name from the bound constructor.

### `model_config` sits beside the manifest and repeats the model's settings

On the same read, `model_config` is:

```json
{
  "temperature": 0,
  "openai_api_key": {"id": ["OPENAI_API_KEY"], "lc": 1, "type": "secret"},
  "model": "gpt-4.1-mini",
  "stream_usage": true
}
```

Those are the same keys and values the bound `ChatOpenAI` constructor carries in its own `kwargs`. The API key appears as a LangChain secret reference, not as a value, which is the shape the credential check skips.

### `include=model` changes the manifest only

```
GET /api/v1/commits/langchain-ai/chain-of-density/latest
```

Without the parameter, the response still holds `model_config` and `model_provider`, with the same values as above. What changes is the manifest: its outer `id` becomes `["langchain", "prompts", "chat", "ChatPromptTemplate"]`, the prompt on its own, with no `RunnableSequence` and no model in it.

So export cannot tell from `model_config` and `model_provider` alone whether it asked for the model. It has to send `include=model` to get the model into the manifest, which is the only place import can carry it, because no field of the create-commit request holds a model.

For a prompt with no model, the response omits `model_config` and `model_provider` altogether rather than returning them as null. These reads each returned `commit_hash`, `examples` and `manifest` and nothing else:

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

Sorting both responses with `jq -S` and comparing them shows no difference. The spec types them differently — `include` is a string with no enumerated values and `include_model` is a boolean defaulting to false, as `reference/api-specs/langsmith/prompt-endpoints.json` records for `GET /api/v1/commits/{owner}/{repo}/{commit}` — and the live API honours either. Export can keep sending `include=model`, the form the Go SDK uses, with nothing lost.

## `POST /api/v1/prompts/invoke_prompt` renders a template and never runs a model

The task asks what this endpoint returns for a prompt with a model. It never sees one. The endpoint takes no prompt handle and no model.

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

Sending a `prompt` key naming `langchain-ai/chain-of-density` and a `model` key holding a `ChatOpenAI` constructor alongside the required fields returned HTTP 200 and the rendered messages for the `messages` field alone; the server ignored both extra keys.

Omitting `template_format` returned HTTP 422 and `{"detail": ["body.template_format: Field required"]}`.

So `invoke_prompt` cannot stand in for running an exported prompt against its model, and prompt verify cannot use it to prove a copied prompt still answers. Anything that runs a prompt with its model has to call the model provider, which is what the OpenAI key in [tasks.md:16, task 2.1, which adds the destination workspace and the model provider key](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L16) is for.

## The workspace secrets endpoints

```
GET /api/v1/workspaces/current/secrets
```

returned HTTP 200 and `[{"key": "TEST_KEY"}]`. The response lists names and no values, which matches `SecretKey` in the spec: one required `key` string and nothing else. `SecretUpsert`, the body `POST /api/v1/workspaces/current/secrets` takes, carries `key` and a nullable `value`.

This is the request prompt import's secret check sends: it can ask which secret names the destination workspace holds, and it cannot read a value back. The CLI Testing Workspace holds `TEST_KEY` and no `OPENAI_API_KEY`, so the secret check for a prompt with an OpenAI model fails in this workspace today. [tasks.md:16, task 2.1, which adds the destination workspace and the model provider key](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L16) adds that secret.

`reference/api-specs/langsmith/workspace-secrets-endpoints.json` holds both paths, extracted in task 1.1.

## What this report does not answer

An agent wrote this report, and the permission rules it runs under refused every request that writes to the live LangSmith workspace or creates a deployment. The questions below each need such a request, so this report leaves them open rather than guessing, and their boxes in `tasks.md` stay unticked.

**Whether a structured prompt saved with a model has the arrangement described above.** The reads above used `langchain-ai/chain-of-density`, whose `first` is a `ChatPromptTemplate`. This report found no public prompt whose `first` is a `StructuredPrompt`: a scan of the first five hundred public prompts by likes, through `GET /api/v1/repos/?limit=100&offset=<0..400>&is_public=true&with_latest_manifest=true&sort_field=num_likes&sort_direction=desc`, returned a null `latest_commit_manifest` on every row, so the scan proved nothing either way. Answering this needs `POST /api/v1/commits/-/<handle>` to push a `StructuredPrompt` with a `ChatOpenAI` beside it, then the commit read above.

**Whether a read-back after a push returns the same manifest, `model_config` and `model_provider`, and what pushing an identical manifest again returns.** This needs `POST /api/v1/repos` and `POST /api/v1/commits/-/<handle>`. The task's fallback, pushing to the existing test workspace under a fresh handle, is just as much a write. `GET /api/v1/workspaces` returned one workspace, the CLI Testing Workspace, so the second workspace the task would otherwise use does not exist until [tasks.md:16, task 2.1, which adds the destination workspace and the model provider key](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L16) adds it.

**Every question about the Agent Server**: what `POST /assistants/search` returns for a graph ID the deployment does not serve and what it returns for `context`; whether an assistant update that sends a metadata key with a null value removes that key; whether an update that sends `config` or `context` replaces the assistant's object or merges into it; whether an update can clear the description; whether a create and an update that send both `config.configurable` and `context` are refused; and whether an update that sends `configurable` alone replaces `context` too. `GET /v1/deployments?limit=50` on `https://api.host.langchain.com` returned an empty `resources` list, so the workspace serves no Agent Server to ask. Answering these needs a deployment, and `langstar deployment create` writes to the control plane.

**What a prompt with a model returns when it runs.** `OPENAI_API_KEY` is set neither in the environment nor in the workspace's secrets, and `invoke_prompt` runs no model in any case, as the section above shows.

## What a later run should do

Give the run permission to write to the CLI Testing Workspace and to create a deployment, then:

- Push a `StructuredPrompt` with a `ChatOpenAI` beside it under a fresh handle, read it back with `include=model`, and record the outer `id`, whether a `RunnableBinding` wraps the model, and the `model_config` and `model_provider` beside the manifest.
- Push that manifest again under a second fresh handle, compare the read-back against the first, then push the identical manifest to the same handle once more and record the status and body.
- Create a deployment from `tests/fixtures/test-graph-deployment/langgraph.json`, then work through the Agent Server questions listed above against it and delete the deployment afterwards.
- Set an OpenAI key and run one imported structured prompt against a small model, which is the check [tasks.md:116, task 7.3, the structured output check](../../openspec/changes/export-import-prompts-and-assistants/tasks.md#L116) describes.

## Related

- [FRAGMENTS.md](../api-specs/langsmith/FRAGMENTS.md) — the jq queries behind `prompt-endpoints.json` and `workspace-secrets-endpoints.json`
- [402-structured-prompts-openapi-validation.md](402-structured-prompts-openapi-validation.md) — structured output prompts
- [461-openapi-validation.md](461-openapi-validation.md) — playground settings, which prompt verify reads for a saved model configuration
