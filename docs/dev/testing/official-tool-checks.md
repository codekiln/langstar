# Checking langstar with the official LangChain tools

langstar's CLI and SDK send requests through the same code, so a bug in that code passes both sets of langstar tests. A check here creates something with langstar and reads it back with LangChain's own CLI, which LangChain wrote separately. These checks add an outside reader to the create-then-read tests in [CRUD Lifecycle Testing Pattern](crud-lifecycle-pattern.md), where langstar's CLI and SDK check each other.

## The tools

`mise.toml` pins them; `mise install` installs them.

| Tool | Provides |
|------|----------|
| [langsmith-cli](https://github.com/langchain-ai/langsmith-cli) | `langsmith` |
| [langgraph-cli](https://pypi.org/project/langgraph-cli/) | `langgraph` |
| [langgraph-sdk](https://pypi.org/project/langgraph-sdk/) | Python library; run it with `uv run --with "langgraph-sdk==$LANGGRAPH_SDK_VERSION"` |

mise's `minimum_release_age` setting holds back releases younger than five days only for a version range; an exact version installs at any age. Before you change a version here, check that the new release is at least five days old.

## Which tool checks which command

| langstar command group | Official tool that can read it back |
|------------------------|-------------------------------------|
| `prompt` | `langsmith prompt` |
| `dataset` | `langsmith dataset`, `langsmith example` |
| `project` | `langsmith project` |
| `runs` | `langsmith run`, `langsmith trace` |
| `eval` | `langsmith evaluator`, `langsmith experiment` |
| `queue` | none |
| `deployment`, `assistant`, `graph` | `langgraph` (local server, build, `deploy list`/`delete`/`logs`/`revisions list`), `langgraph-sdk` (assistants, threads, runs on a deployment) |
| `secrets`, `model-config`, `config` | none |

The official `langsmith thread` and `langsmith hub` commands have no langstar command to check yet.

Only `prompt` has a check today; the table lists what the official tools could check. Before writing a check, run `langsmith <group> --help` to see the subcommands.

## Checks

Each check is a mise task under `mise-tasks/oracle/`. In testing, an oracle supplies the expected result, and here LangChain's own CLI is the oracle. Each task gets its credentials from `fnox exec` and cleans up what it creates.

| Task | Checks |
|------|--------|
| `mise run oracle:prompt-roundtrip` | `langstar prompt push` against `langsmith prompt pull` |

## langstar bugs the prompt check found

- [#820 prompt push --owner - fails when it creates a new repo](https://github.com/codekiln/langstar/issues/820): the task creates the repo with `langsmith prompt create` first.
- [#821 prompt push without --schema to an empty repo fails](https://github.com/codekiln/langstar/issues/821): the task pushes with `--schema`.
