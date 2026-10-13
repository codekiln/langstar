---
root: false
targets: ["*"]
description: "Upstream LangChain projects to compare API-facing changes against"
---

# Upstream references

Langstar re-implements behavior that LangChain's own clients already settled. Before you change API-facing behavior (retries, pagination, defaults, flags, output shapes, auth), read the references below. Each has orientation notes in `reference/repo/langchain-ai/<name>/notes/README.md` and a shared clone in `code/`. If `code/` is missing, run the `setup-remote-repo-notes-dir` skill's script.

| Reference | Check it for |
|---|---|
| `langsmith-go` | Endpoints, request and response types, and pagination. LangChain generates it from LangSmith's OpenAPI spec with a tool called Stainless, and langstar's SDK is generated from an OpenAPI spec too. An endpoint listed in its `api.md` that langstar lacks is a candidate missing endpoint. Its retry and timeout settings are the ones Stainless gives every client it generates, so read langsmith-sdk for what LangSmith chose. |
| `langsmith-cli` | CLI behavior: which calls retry, flags, output shapes, auth and profiles. |
| `langsmith-sdk` (Python, JS) | Behavior LangChain wrote by hand for LangSmith that langsmith-go does not generate, such as the 30-second default wait in the Python client's second retry layer. Both clients retry a 429 on ordinary requests, and the notes in `reference/repo/langchain-ai/langsmith-sdk/notes/README.md`, section "How the Python and JS clients retry a 429", give each layer's attempts and waits. |
| `langsmith-mcp-server`, `langgraph` (`libs/cli`) | The MCP tools and the `langgraph` commands (`up`, `dev`, `deploy`) that langstar users would run instead of langstar: their names, flags and output. |

Compare your change with every reference that covers it. In the PR body, say where langstar matches each one and where it differs on purpose, with the reason. `docs/dev/feature-development-process.md` has the research steps.
