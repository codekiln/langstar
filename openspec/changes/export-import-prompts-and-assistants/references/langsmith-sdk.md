# LangSmith Python SDK

Read at [langsmith-sdk 47c553ca](https://github.com/langchain-ai/langsmith-sdk/tree/47c553caf08c1989e2be3ff1886da3eb801f709f).

[`pull_prompt_commit`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L10107) adds `?include_model=true` when asked. The result, [`PromptCommit`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/schemas.py#L1035), holds the manifest plus `model_config` and `model_provider`.

With the model included, the manifest is a `RunnableSequence`. Its `first` step is the `ChatPromptTemplate` or `StructuredPrompt` (with `schema_`), and its `last` step is a `RunnableBinding` around the chat model constructor and its settings. The [integration fixture `prompt_with_model`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/tests/integration_tests/test_prompts.py#L48) shows one.

The chat model refers to its API key as a secret reference, `{"lc": 1, "type": "secret", "id": ["OPENAI_API_KEY"]}`. [`_manifest_has_secrets`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L434) finds these by matching objects whose keys are exactly `lc`, `type` and `id`.

[`create_commit`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L9915) resolves a missing parent to the latest commit, posts `{parent_commit, manifest, description}`, then posts each tag to `/repos/{owner}/{repo}/tags`. [`push_prompt`](https://github.com/langchain-ai/langsmith-sdk/blob/47c553caf08c1989e2be3ff1886da3eb801f709f/python/langsmith/client.py#L10309) creates the repo first when it does not exist.

What langstar takes from it: the shape of a manifest with its model, how to find secret names, and the order of create repo, commit with parent, then tags.
