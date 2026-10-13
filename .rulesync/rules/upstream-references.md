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
| `langsmith-sdk` (Python, JS) | Behavior LangChain wrote by hand for LangSmith that langsmith-go lacks. Example: after a 429 with no `Retry-After` header, the Python client waits 30 seconds, doubled on each further attempt (`python/langsmith/client.py`, `request_with_retries`), and the JS client waits 10 seconds (`js/src/client.ts`, `handle429`). Each note names the file where it found each one. |
| `langsmith-mcp-server`, `langgraph` (`libs/cli`) | The tools langstar aims to replace. |

Compare your change with every reference that covers it. In the PR body, say where langstar matches each one and where it differs on purpose, with the reason. `docs/dev/feature-development-process.md` has the research steps.
