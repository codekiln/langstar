# Langstar and the official LangChain tools

Since langstar started, LangChain has released official tools that do most of what langstar does. The LangSmith CLI, the LangGraph CLI and the LangSmith Terraform provider now cover `runs`, `deployment`, `model-config`, `secrets`, `eval`, most of `prompt` and `dataset`, and part of `project`. Only langstar has commands for assistants, graphs and annotation queues, or builds a structured-output prompt from a template and a JSON Schema. The open questions at the end are codekiln's decisions about which command groups to keep.

The official tools were read from source on 2026-10-09 at these commits, the same ones pinned on the garden page [LangSmith/Q/Which LangChain tools manage each part of a LangSmith setup?](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/LangSmith___Q___Which%20LangChain%20tools%20manage%20each%20part%20of%20a%20LangSmith%20setup%253F.md):

- LangSmith CLI v0.3.0 at [`596db1e`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/root.go#L76-L95), written in Go. The link goes to the list of its command groups.
- LangSmith Terraform provider v0.0.16 at [`0896d0f`](https://github.com/langchain-ai/terraform-provider-langsmith/tree/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources).
- LangGraph CLI and SDK at [`40a2e6d`](https://github.com/langchain-ai/langgraph/tree/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs).

Langstar was read at `main` as of [`1704f1e` design AI-first command structure for prompt UX](https://github.com/codekiln/langstar/commit/1704f1e).

## Summary

| langstar group | Official tool that covers it | Coverage | What only langstar does |
|---|---|---|---|
| `prompt` | LangSmith CLI `prompt` | The official CLI does more | Builds a structured-output prompt from a template file and a JSON Schema file |
| `assistant` | None | Nothing official | Everything: list, search, get, create, update, delete |
| `graph` | None | Nothing official | Shows a deployed graph's structure from the command line |
| `deployment` | LangGraph CLI `deploy`, Terraform `langsmith_deployment` | Both cover it | Finds the GitHub integration ID from existing deployments |
| `runs` | LangSmith CLI `run`, `trace`, `thread` | The official CLI does more | A few filter options: trace and tree filters, field selection, sort order |
| `queue` | None | Nothing official | Everything: annotation queues and the runs in them |
| `dataset` | LangSmith CLI `dataset`, `example` | Mostly covered | Dataset update, and import and export as CSV or JSONL |
| `project` | LangSmith CLI `project` | Partly covered | Project get, create and update |
| `eval` | LangSmith CLI `evaluator`, `experiment`; Terraform `langsmith_evaluator` | Covered, and langstar's group is unfinished | Nothing; every langstar `eval` subcommand is a placeholder |
| `model-config` | Terraform `langsmith_model_configuration` | Covered, through Terraform config files | One-off commands from a shell, with no Terraform config |
| `secrets` | Terraform `langsmith_workspace_secret` | Covered, through Terraform config files | One-off commands from a shell, with no Terraform config |
| `config`, `version` | Not applicable | Langstar's own settings | Not applicable |

The official CLI also has groups langstar never had: `chart`, `insights`, `sandbox`, `hub`, `apps`, `workspace`, `profile`, and `api`, which calls any LangSmith REST endpoint directly. Because of `api`, the official CLI can reach every LangSmith API endpoint langstar's groups call, including queues, secrets and model configurations, with hand-written JSON. The summary table counts only commands built for one kind of resource, such as `langsmith dataset list`, and leaves out what `langsmith api` can reach.

## Command groups

### `prompt`

Langstar has `list`, `get`, `search`, `push` and `pull` ([cli/src/commands/prompt.rs](../../cli/src/commands/prompt.rs)). The official [`langsmith prompt`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/prompt.go#L31) has `list`, `get`, `create`, `delete`, `pull`, `push`, `commits` and a `tag` group with `list`, `create` and `update`. Its `list --query` does what langstar's `search` does.

The official `push` takes a whole manifest as JSON from a file or stdin ([prompt.go#L367](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/prompt.go#L367)). Langstar's `push` builds that manifest for you: it takes a template, a template format, and a `--schema` JSON Schema file with `--schema-method json_schema` or `function_calling`, validates the schema, and sends a `StructuredPrompt`. Only langstar builds a structured-output prompt this way. Writing the manifest by hand for the official CLI means writing LangChain's serialized `StructuredPrompt` format.

Langstar lacks delete, commit history and tags.

### `assistant`

Langstar has `list`, `search`, `get`, `create`, `update` and `delete`, against each deployment's Agent Server ([cli/src/commands/assistant.rs](../../cli/src/commands/assistant.rs)). No official command line tool, MCP server or Terraform resource manages assistants. Only the LangGraph SDKs ([`assistants.py`](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/sdk-py/langgraph_sdk/_async/assistants.py#L314-L376)), the Agent Server REST API and the UI do. No community tool turned up in the search behind the garden report [LangSmith/Report/26/10/Assistants as Code in LangSmith Deployments](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/LangSmith___Report___26___10___Assistants%20as%20Code%20in%20LangSmith%20Deployments.md) either.

In the garden report on assistants as code, the missing piece is a tool that keeps assistant specs in Git and applies them to each deployment. Langstar's `assistant` group has list, search, get, create, update and delete, and would need new options to do that job. The SDK supports that through a caller-chosen `assistant_id`, `if_exists`, version listing and `set_latest`. Langstar's `create` takes none of these, and langstar has no version commands.

### `graph`

Langstar's `list` and `get` show the graphs in a deployment and their structure, with an `xray` option for subgraphs ([cli/src/commands/graph.rs](../../cli/src/commands/graph.rs)). The SDK can fetch a graph, and LangGraph Studio draws one, but no official command line tool prints one.

### `deployment`

Langstar has `list`, `get`, `create` and `delete` against the control plane API ([cli/src/commands/deployment.rs](../../cli/src/commands/deployment.rs)). `create` builds from a GitHub repository or an external Docker image, finds the GitHub integration ID from existing deployments if you don't give one, and can wait until the deployment is ready.

The LangGraph CLI's [`langgraph deploy`](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/cli/langgraph_cli/deploy.py#L2474-L2695) builds an image and creates or updates a deployment, with `list`, `delete`, `logs` and `revisions list`; all are marked beta. The Terraform [`langsmith_deployment`](https://github.com/langchain-ai/terraform-provider-langsmith/blob/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources/deployment.md) resource builds from a GitHub repository and has data sources for revisions. Between them they cover every langstar `deployment` command and add logs and revisions. Langstar alone looks up the GitHub integration ID from existing deployments; Terraform's `langsmith_deployment` takes that ID as its `integration_id` input.

### `runs`

Langstar has a single `query` command ([cli/src/commands/runs.rs](../../cli/src/commands/runs.rs)). The official CLI has [`run`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/run.go#L15), [`trace`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/trace.go#L17) and [`thread`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/thread.go#L24), each with `list`, `get` and `export` or `messages`, and its filters include a raw `--filter` query, tags, metadata, run type, errors, latency and tokens. Langstar's `query` adds trace and tree filters, field selection, sort order and named time ranges. Those are small additions to a group the official CLI does better.

### `queue`

Langstar manages annotation queues: `list`, `create`, `get`, `update`, `delete`, and `add-runs`, `remove-run` and `items` for the runs in a queue ([cli/src/commands/queue.rs](../../cli/src/commands/queue.rs)). The official CLI has no queue commands apart from raw `langsmith api` calls, and Terraform has no queue resource. The LangSmith SDK covers queues.

### `dataset`

Langstar has `create`, `list`, `get`, `update`, `delete`, `import`, `list-examples` and `export` ([cli/src/commands/dataset.rs](../../cli/src/commands/dataset.rs)). The official [`langsmith dataset`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/dataset.go#L17) has `list`, `get`, `create`, `delete`, `export` and `upload`, both in JSON, and [`langsmith example`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/example.go#L18) lists, creates and deletes single examples. Langstar adds dataset update and CSV and JSONL formats. Terraform has no dataset resource.

### `project`

Langstar has `list`, `get`, `create`, `update` and `delete` ([cli/src/commands/project.rs](../../cli/src/commands/project.rs)). The official [`langsmith project`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/project.go#L15) has `list`, `delete` and an `issues` group, but no `get`. Langstar's `get` looks a project up by ID or name and shows its details. LangSmith creates a project the first time an application sends it a trace, so few people need langstar's `project create`. The official CLI lacks `get` and `update`. Terraform only has a data source that looks a project up by name.

### `eval`

Every langstar `eval` subcommand (`create`, `run`, `list`, `get`, `export`) is a placeholder: each one has a `TODO` where the work should be, and `list` returns an empty list ([cli/src/commands/eval.rs#L336](../../cli/src/commands/eval.rs#L336)). The official [`langsmith evaluator`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/evaluator.go#L22) lists, gets and deletes evaluators, uploads code evaluators, creates LLM-as-judge rules, and manages the rules that attach them to projects and datasets. [`langsmith experiment`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/experiment.go#L15) lists and gets results. Terraform has [`langsmith_evaluator`](https://github.com/langchain-ai/terraform-provider-langsmith/blob/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources/evaluator.md) and `langsmith_run_rule`.

### `model-config`

Langstar has `list`, `get`, `create`, `update` and `delete` against the playground settings API ([cli/src/commands/model_config.rs](../../cli/src/commands/model_config.rs)). `model-config create` now fails with an HTTP 500 server error, one of the breakages listed in [reboot milestone · Issue #746](https://github.com/codekiln/langstar/issues/746). The Terraform [`langsmith_model_configuration`](https://github.com/langchain-ai/terraform-provider-langsmith/blob/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources/model_configuration.md) resource calls the same endpoint, `api/v1/playground-settings`, and LangChain keeps it working, so its source shows the request langstar's `create` should send. The garden page [LangSmith/Q/Which LangChain tools manage each part of a LangSmith setup?](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/LangSmith___Q___Which%20LangChain%20tools%20manage%20each%20part%20of%20a%20LangSmith%20setup%253F.md) does not list this resource.

### `secrets`

Langstar has `list`, `set` and `delete` for workspace secrets ([cli/src/commands/secrets.rs](../../cli/src/commands/secrets.rs)). The Terraform [`langsmith_workspace_secret`](https://github.com/langchain-ai/terraform-provider-langsmith/blob/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources/workspace_secret.md) resource and the `langsmith_workspace_secrets` data source cover the same ground, and model configurations can refer to a secret by key. The official CLI has no secret commands. The garden page on LangChain tools does not list this resource either.

### `config` and `version`

These manage langstar's own config file and print its version. They stay or go with the rest of langstar.

## Resolved Questions

### 1 - Does any official tool manage assistants?

No. Only the LangGraph SDKs, the Agent Server REST API and the UI do. The LangSmith CLI, the LangGraph CLI, the LangSmith MCP servers and the Terraform provider have no assistant commands or resources.

### 2 - Does Terraform cover model configurations and workspace secrets?

Yes. The provider has `langsmith_model_configuration` and `langsmith_workspace_secret`, which the garden page on LangChain tools does not mention. `langsmith_model_configuration` uses the same playground settings endpoint as langstar's `model-config`.

### 3 - Does langstar's `eval` group work today?

No. Each subcommand is a placeholder that does no work.

### 4 - Can the official CLI reach endpoints that have no command of their own?

Yes. `langsmith api` calls any LangSmith REST endpoint directly, so annotation queues, secrets and model configurations are reachable with hand-written JSON. It cannot reach assistants or graphs, which live on each deployment's Agent Server rather than the LangSmith API.

## Open Questions

### 1 - Which command groups should langstar keep and invest in?

> Context from the drafting agent, for question 1.
>
> Recommendation: keep `assistant`, `graph` and `queue`, because no official command line tool covers them. Keep `prompt push` with `--schema`, because no official tool builds a structured-output prompt from a template and a schema. `assistant` is where langstar could matter most: the missing piece, in the garden report on assistants, is applying assistant specs kept in Git to each deployment, and langstar would need a caller-chosen `assistant_id`, `if_exists` and version commands to do it.

<ANSWER_HERE>

### 2 - Which command groups should langstar deprecate in favour of an official tool?

> Context from the drafting agent, for question 2.
>
> Recommendation: deprecate `runs` in favour of `langsmith run`, `trace` and `thread`; `deployment` in favour of `langgraph deploy` or Terraform; and `model-config` and `secrets` in favour of Terraform. `dataset` and `project` are closer calls: the official CLI lacks dataset update, CSV import, and project get and update. Every group langstar keeps is one more group someone has to update when LangSmith changes its API, which is the work under way in [reboot milestone · Issue #746](https://github.com/codekiln/langstar/issues/746).

<ANSWER_HERE>

### 3 - Should langstar delete the `eval` group or finish it?

> Context from the drafting agent, for question 3.
>
> Recommendation: delete it. Every `eval` command is a placeholder that does no work, and the official `evaluator` and `experiment` groups and Terraform's `langsmith_evaluator` already do what those commands were meant to do.

<ANSWER_HERE>

### 4 - Does the ls-prompt-ux milestone still make sense?

> Context from the drafting agent, for question 4.
>
> [ls-prompt-ux milestone · Issue #668](https://github.com/codekiln/langstar/issues/668) redesigns every `prompt` command around CRUD verbs and `help` subcommands. The official `langsmith prompt` already has create, get, list, delete, pull, push, commits and tags. Recommendation: narrow the milestone to the part only langstar does, building and pushing structured-output prompts, and drop the redesign of list, search and get.

<ANSWER_HERE>

### 5 - Does the ls-cli-output-dx milestone still make sense?

> Context from the drafting agent, for question 5.
>
> [ls-cli-output-dx milestone · Issue #529](https://github.com/codekiln/langstar/issues/529) adds table, json and text output and column selection to every list command. Its list of commands includes `deployment`, `dataset` and `runs`, which are candidates for deprecation. The official CLI offers `pretty` and `json` output and `-o` to write JSON to a file. Recommendation: decide which groups to deprecate first, then limit the milestone to the groups langstar keeps.

<ANSWER_HERE>
