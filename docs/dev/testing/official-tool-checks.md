# Checking langstar with the official LangChain tools

A langstar command can be wrong in a way its own SDK cannot see, because the CLI and SDK share the same request code. The official tools are a second implementation: create with langstar, read back with the official tool, and compare. This extends [crud-lifecycle-pattern.md](crud-lifecycle-pattern.md).

## The tools

`mise.toml` pins them; `mise install` installs them.

| Tool | Provides |
|------|----------|
| [langsmith-cli](https://github.com/langchain-ai/langsmith-cli) | `langsmith` |
| [langgraph-cli](https://pypi.org/project/langgraph-cli/) | `langgraph` |
| [langgraph-sdk](https://pypi.org/project/langgraph-sdk/) | Python library; run it with `uv run --with "langgraph-sdk==$LANGGRAPH_SDK_VERSION"` |

Bump a pin by hand and check that the release is older than five days: `minimum_release_age` does not apply to exact pins.

## Which tool checks which command

| langstar command group | Official tool that can read it back |
|------------------------|-------------------------------------|
| `prompt` | `langsmith prompt` |
| `dataset` | `langsmith dataset`, `langsmith example` |
| `project` | `langsmith project` |
| `runs` | `langsmith run`, `langsmith trace` |
| `eval` | `langsmith evaluator`, `langsmith experiment` |
| `queue` | none |
| `langsmith thread`, `langsmith hub` | no langstar command yet |
| `deployment`, `assistant`, `graph` | `langgraph` (local server, build, `deploy list`/`delete`/`logs`/`revisions list`), `langgraph-sdk` (assistants, threads, runs on a deployment) |
| `secrets`, `model-config`, `config` | none |

The table says what a tool can reach, not what a check covers today. Run `langsmith <group> --help` to confirm a subcommand exists before writing a check.

## Checks

Each check is a mise task under `mise-tasks/oracle/`, named for the official tool acting as the reference result. It gets its credentials from `fnox exec` and cleans up what it creates.

| Task | Checks |
|------|--------|
| `mise run oracle:prompt-roundtrip` | `langstar prompt push` against `langsmith prompt pull` |

## Bugs the first check found

- [#820](https://github.com/codekiln/langstar/issues/820): `langstar prompt push --owner -` to a new repo fails. The task creates the repo with `langsmith prompt create` first.
- [#821](https://github.com/codekiln/langstar/issues/821): a push without `--schema` to an empty repo fails. The task pushes with `--schema`.
