---
root: false
targets: ["*"]
description: "Upstream LangChain projects to compare API-facing changes against"
---

# Upstream references

Langstar re-implements behavior that LangChain's own clients already settled. Before you change API-facing behavior (retries, pagination, defaults, flags, output shapes, auth), read the references below. Each has orientation notes in `reference/repo/langchain-ai/<name>/notes/README.md` and a shared clone in `code/`. If `code/` is missing, run the `setup-remote-repo-notes-dir` skill's script.

| Reference | Check it for |
|---|---|
| `langsmith-go` | Endpoints, request and response types, and pagination. Its README says it is generated with Stainless, and its source files start "File generated from our OpenAPI spec by Stainless", so it is an inventory derived from a spec. Langstar's SDK is not generated; it is written with LLM assistance (`sdk/src/generated/README.md`). An endpoint listed in langsmith-go's `api.md` that langstar lacks is a candidate missing endpoint. Its retry and timeout defaults live in Stainless-generated code (`internal/requestconfig/requestconfig.go`), so they are generic generated-client behavior and not a LangSmith decision; read langsmith-sdk for what LangSmith chose. |
| `langsmith-cli` | CLI behavior: which calls retry, flags, output shapes, auth and profiles. |
| `langsmith-sdk` (Python, JS) | LangSmith-specific behavior that LangChain wrote by hand, such as its own retry policy, which differs from the Stainless default that langsmith-go uses. For how the Python and JS clients retry a 429, read `reference/repo/langchain-ai/langsmith-sdk/notes/README.md`, section "How the Python and JS clients retry a 429". |
| `langsmith-mcp-server`, `langgraph` (`libs/cli`) | The MCP tools and the `langgraph` commands (`up`, `dev`, `deploy`) that langstar users would run instead of langstar: their names, flags and output. |

Compare your change with every reference that covers it. In the PR body, say where langstar matches each one and where it differs on purpose, with the reason. `docs/dev/feature-development-process.md` has the research steps.
