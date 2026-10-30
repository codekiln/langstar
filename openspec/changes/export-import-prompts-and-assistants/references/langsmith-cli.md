# LangSmith CLI (Go)

Read at [langsmith-cli 26135038](https://github.com/langchain-ai/langsmith-cli/tree/26135038671155f05ddd725703de7bdd43c7aa9f).

[`prompt pull`](https://github.com/langchain-ai/langsmith-cli/blob/26135038671155f05ddd725703de7bdd43c7aa9f/internal/cmd/prompt.go#L301) takes `OWNER/REPO[:COMMIT_OR_TAG]` and writes this envelope to stdout or `--output`:

```json
{"owner": "...", "repo": "...", "commit_hash": "...", "description": "...", "manifest": {}}
```

It reads the commit with empty `CommitGetParams`, so the model is never requested.

[`prompt push`](https://github.com/langchain-ai/langsmith-cli/blob/26135038671155f05ddd725703de7bdd43c7aa9f/internal/cmd/prompt.go#L359) reads the bare manifest from `--file` or stdin and sends it as `manifest`, with optional `--parent-commit` and `--description`. It does not create the repo and has no tag flag. A pull envelope fed to push is sent as the manifest itself.

[`resolveClientOptions`](https://github.com/langchain-ai/langsmith-cli/blob/26135038671155f05ddd725703de7bdd43c7aa9f/internal/cmd/root.go#L161) picks one workspace per run from `--workspace`, `LANGSMITH_WORKSPACE_ID`, `LANGSMITH_TENANT_ID` or the profile, highest first.

What langstar takes from it: a file between a read run and a write run, a `:tag` or `--commit` to choose the commit, and output to stdout or a file.
