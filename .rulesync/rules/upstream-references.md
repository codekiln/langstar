---
root: false
targets: ["*"]
description: "Upstream LangChain projects to compare API-facing changes against"
---

# Upstream references

Langstar re-implements behavior that LangChain's own clients already settled. Before you change API-facing behavior (retries, pagination, defaults, flags, output shapes, auth), read the references below. Each has orientation notes in `reference/repo/langchain-ai/<name>/notes/README.md` in your worktree, and one clone shared by all worktrees at `reference/repo/langchain-ai/<name>/code/` in the root checkout. If that `code/` directory is missing from the root checkout, run the `setup-remote-repo-notes-dir` skill's script from your worktree; it clones into the root checkout.

| Reference | Check it for |
|---|---|
| `langsmith-go` | Endpoints, request and response types, and pagination. Its README says it is generated with Stainless, and its resource files (`client.go`, `dataset.go`, `feedback.go`, `run.go`) start "File generated from our OpenAPI spec by Stainless", so the endpoint and type inventory is derived from a spec; `config.go`, `address.go`, `tracing.go` and the `sandbox_*.go` files do not carry that header and are hand-written. Langstar's SDK is not generated; it is written with LLM assistance (`sdk/src/generated/README.md`). An endpoint listed in langsmith-go's `api.md` that langstar lacks is a candidate missing endpoint. Stainless generates the retry default (`MaxRetries: 2` in `internal/requestconfig/requestconfig.go`) and the 10-minute response-header timeout (`default_http_client.go`), so those two values show what a generated Go client does by default. Read langsmith-sdk for the retry policy LangSmith chose. |
| `langsmith-cli` | CLI behavior: which calls retry, flags, output shapes, auth and profiles. |
| `langsmith-sdk` (Python, JS) | LangSmith-specific behavior that LangChain wrote by hand, such as its own retry policy, which differs from the Stainless default that langsmith-go uses. For how the Python and JS clients retry a 429, read `reference/repo/langchain-ai/langsmith-sdk/notes/README.md`, section "How the Python and JS clients retry a 429". |
| `langsmith-mcp-server`, `langgraph` (`libs/cli`) | The MCP tools and the `langgraph` commands (`up`, `dev`, `deploy`) that langstar users would run instead of langstar: their names, flags and output. |

Compare your change with every reference that covers it. In the PR body, say where langstar matches each one and where it differs on purpose, with the reason. `docs/dev/feature-development-process.md` has the research steps.
