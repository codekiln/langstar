# LangSmith Go SDK

Read at [langsmith-go 59bb42c3](https://github.com/langchain-ai/langsmith-go/tree/59bb42c30052067fe488f9bb8370df9037c715ef).

[`CommitGetParams`](https://github.com/langchain-ai/langsmith-go/blob/59bb42c30052067fe488f9bb8370df9037c715ef/commit.go#L337) takes `include`, a comma-separated list of `model` and `is_draft`, and marks the boolean `include_model` deprecated.

[`CommitGetResponse`](https://github.com/langchain-ai/langsmith-go/blob/59bb42c30052067fe488f9bb8370df9037c715ef/commit.go#L262) returns `commit_hash`, `description`, `examples`, `is_draft`, `manifest`, `model_config` and `model_provider`.

[`CommitNewParams`](https://github.com/langchain-ai/langsmith-go/blob/59bb42c30052067fe488f9bb8370df9037c715ef/commit.go#L324) sends `manifest`, `parent_commit`, `description` and `skip_webhooks`. There is no field for the model outside the manifest.

[`RepoNewParams`](https://github.com/langchain-ai/langsmith-go/blob/59bb42c30052067fe488f9bb8370df9037c715ef/repo.go#L310) requires `repo_handle` and `is_public` and takes `description`, `readme` and `tags`.

What langstar takes from it: `include=model` on export, and the repo fields an export carries.
