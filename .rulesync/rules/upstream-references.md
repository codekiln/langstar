---
root: false
targets: ["*"]
description: "Upstream LangChain projects to compare API-facing changes against"
---

# Upstream references

Langstar re-implements behavior that LangChain's own clients already settled. Before you change API-facing behavior (retries, pagination, defaults, flags, output shapes, auth), read the references below. Each has orientation notes in `reference/repo/langchain-ai/<name>/notes/README.md` and a shared clone in `code/`. If `code/` is missing, run the `setup-remote-repo-notes-dir` skill's script.

| Reference | Authoritative for |
|---|---|
| `langsmith-go` | Endpoints, request and response types, pagination. It is generated from LangSmith's OpenAPI spec (Stainless), as langstar's SDK is, so it is also the check for spec drift. Its retry and transport defaults are generic Stainless behavior, not LangSmith decisions. |
| `langsmith-cli` | CLI behavior: which calls retry, flags, output shapes, auth and profiles. |
| `langsmith-sdk` (Python, JS) | LangSmith-specific behavior the generated SDK lacks, such as 429 handling with `Retry-After`. |
| `langsmith-mcp-server`, `langgraph` (`libs/cli`) | The tools langstar aims to replace. |

Compare your change with every reference that covers it. In the PR body, say where langstar matches each one and where it differs on purpose, with the reason. `docs/dev/feature-development-process.md` has the research steps.
