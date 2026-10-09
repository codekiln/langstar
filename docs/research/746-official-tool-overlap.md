# Langstar's parity with the official LangChain tools

codekiln wants langstar to be the one command line tool a LangSmith user needs, so they never have to work out whether a task calls for the LangSmith CLI, the LangGraph CLI, an MCP server or Terraform. This document lists what each of those official tools can do, says whether langstar can do it too, and sizes each gap. The open questions at the end ask codekiln which gaps to close first.

Langstar is ahead on assistants, graphs, annotation queues and structured-output prompts, which no official command line tool manages. It is furthest behind on evaluators and experiments, where its `eval` commands are placeholders, and on what only Terraform manages: workspace administration, resource tags and alerts.

The official tools were read from source on 2026-10-09 at these commits, the same ones pinned on the garden page [LangSmith/Q/Which LangChain tools manage each part of a LangSmith setup?](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/LangSmith___Q___Which%20LangChain%20tools%20manage%20each%20part%20of%20a%20LangSmith%20setup%253F.md):

- LangSmith CLI v0.3.0 at [`596db1e`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/root.go#L76-L95), written in Go. The link goes to the list of its command groups.
- LangSmith Terraform provider v0.0.16 at [`0896d0f`](https://github.com/langchain-ai/terraform-provider-langsmith/tree/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources).
- LangGraph CLI and SDK at [`40a2e6d`](https://github.com/langchain-ai/langgraph/tree/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs).
- The LangSmith Remote MCP server's tool list, from the LangChain docs at [`1da22ee`](https://github.com/langchain-ai/docs/blob/1da22ee8525f1a71b17b86974f74f4f28da45347/src/langsmith/langsmith-remote-mcp.mdx#L165-L174).

Langstar was read at `main` as of [`1704f1e` design AI-first command structure for prompt UX](https://github.com/codekiln/langstar/commit/1704f1e).

## Parity map

Each row is something an official tool can do. A gap is small when langstar needs a few more commands against an API it already calls, medium when it needs a new command group against an API it does not call yet, and large when it needs a new area of LangSmith or a different kind of tool.

| Capability | Official tool | Langstar | Gap |
|---|---|---|---|
| List, get, search, push and pull prompts | LangSmith CLI `prompt`; MCP `list_prompts`, `get_prompt_by_name` | Has it: `prompt` | None |
| Delete prompts, read commit history, manage tags | LangSmith CLI `prompt delete`, `commits`, `tag` | Lacks it | Small |
| Create, read, update and delete assistants | None; only the LangGraph SDKs and the Agent Server API | Has it: `assistant` | Langstar is ahead |
| Assistant versions, fixed assistant IDs, create-if-missing | None; only the LangGraph SDKs | Lacks it | Medium |
| Show a deployed graph's structure | None; only the LangGraph SDKs | Has it: `graph` | Langstar is ahead |
| List, get, create and delete deployments | LangGraph CLI `deploy`; Terraform `langsmith_deployment` | Has it: `deployment` | None |
| Deployment logs and revisions | LangGraph CLI `deploy logs`, `deploy revisions list`; Terraform revision data sources | Lacks it | Small |
| Build an image from local code and deploy it | LangGraph CLI `deploy` | Lacks it | Large |
| Run a local Agent Server, build images, start a project from a template | LangGraph CLI `dev`, `up`, `build`, `dockerfile`, `new`, `validate` | Lacks it | Large |
| Query runs with filters | LangSmith CLI `run list`; MCP `fetch_runs` | Has it: `runs query` | None |
| Get one run, export runs, read traces as trees, read thread messages | LangSmith CLI `run`, `trace`, `thread`; MCP `get_thread_history` | Lacks it | Medium |
| Charts, insight reports and issues for a tracing project | LangSmith CLI `chart`, `insights`, `project issues` | Lacks it | Medium |
| Manage annotation queues and their runs | None | Has it: `queue` | Langstar is ahead |
| Create, list, get, update, delete, import and export datasets | LangSmith CLI `dataset`; MCP `list_datasets`, `read_dataset` | Has it: `dataset` | None |
| Create and delete single examples | LangSmith CLI `example` | Partly: `dataset list-examples` reads them | Small |
| List, get, create, update and delete tracing projects | LangSmith CLI `project list`, `delete`; MCP `list_projects` | Has it: `project` | None |
| Code and LLM-as-judge evaluators, the rules that attach them, experiment results | LangSmith CLI `evaluator`, `experiment`; Terraform `langsmith_evaluator`, `langsmith_run_rule`; MCP `list_experiments` | Lacks it: every `eval` command is a placeholder | Large |
| Model configurations | Terraform `langsmith_model_configuration` | Partly: `model-config`, but `create` returns an HTTP 500 | Small |
| Workspace secrets | Terraform `langsmith_workspace_secret` | Has it: `secrets` | None |
| Resource tags | Terraform `langsmith_tag_key`, `langsmith_tag_value`, `langsmith_tagging` | Lacks it | Medium |
| Workspaces, members, roles, access policies and service keys | Terraform `langsmith_workspace`, `langsmith_workspace_membership`, `langsmith_workspace_role`, `langsmith_access_policy`, `langsmith_service_key`; the LangSmith CLI's `workspace` only lists workspaces and sets the default one | Lacks it | Large |
| Alert rules and gateway policies | Terraform `langsmith_alert_rule`, `langsmith_gateway_policy` | Lacks it | Medium |
| Sign in with OAuth, keep named profiles | LangSmith CLI `auth`, `profile` | Partly: API key from the environment or the config file | Medium |
| Call any LangSmith API endpoint | LangSmith CLI `api` | Lacks it | Small |
| Update the tool itself | LangSmith CLI `update` | Lacks it; the install script installs a new version | Small |
| Billing usage | MCP `get_billing_usage` | Lacks it | Small |
| Hub agent and skill repos, Custom Apps, sandboxes | LangSmith CLI `hub`, `apps`, `sandbox` | Lacks it | Large |

Langstar also has a few smaller things the official tools lack. `prompt push --schema` builds a structured-output prompt from a template and a JSON Schema file, where the official `push` takes a finished manifest. `dataset update`, CSV and JSONL import and export, `project get` and `project update` have no official command. `runs query` filters on traces and trees, selects fields and sorts. `deployment create` finds the GitHub integration ID from existing deployments, where Terraform's `langsmith_deployment` makes you supply it.

## What closing each gap would take

### Evaluators and experiments

Every langstar `eval` subcommand (`create`, `run`, `list`, `get`, `export`) has a `TODO` where the work should be, and `list` returns an empty list ([cli/src/commands/eval.rs#L336](../../cli/src/commands/eval.rs#L336)). The official [`langsmith evaluator`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/evaluator.go#L22) lists, gets and deletes evaluators, uploads code evaluators, creates LLM-as-judge rules, and manages the rules that attach evaluators to projects and datasets. [`langsmith experiment`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/experiment.go#L15) lists and gets experiment results. Langstar's `eval` was designed around running evaluations itself, which is a different job from managing the evaluators LangSmith runs.

### Assistants kept in Git

No tool, official or community, keeps assistant definitions in Git and applies them to each deployment, according to the survey in the garden report [LangSmith/Report/26/10/Assistants as Code in LangSmith Deployments](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/LangSmith___Report___26___10___Assistants%20as%20Code%20in%20LangSmith%20Deployments.md). Langstar is the nearest thing, but its `assistant create` cannot choose the assistant's ID or skip an assistant that already exists, and langstar has no version commands. The LangGraph SDK has all of these ([`assistants.py`](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/sdk-py/langgraph_sdk/_async/assistants.py#L314-L376)).

### Local development and deploying from local code

The LangGraph CLI runs an Agent Server on your machine, builds Docker images and starts projects from templates ([cli.py](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/cli/langgraph_cli/cli.py#L276)), and [`langgraph deploy`](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/cli/langgraph_cli/deploy.py#L2474-L2695) builds an image from local code before deploying it. These commands run Python and Docker on the user's machine, so langstar, a single Rust binary, would most likely call the installed `langgraph` command rather than reimplement them.

### What only Terraform manages

Workspace administration, resource tags and alerts are reachable today only through Terraform or the UI ([resource list](https://github.com/langchain-ai/terraform-provider-langsmith/tree/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources)). Terraform describes the state a workspace should be in and changes the workspace to match; langstar runs one change per command. To replace Terraform for someone, langstar would need both the commands and a way to apply a file of desired state.

### Model configurations

`model-config create` now fails with an HTTP 500 server error, one of the breakages listed in [reboot milestone · Issue #746](https://github.com/codekiln/langstar/issues/746). The Terraform [`langsmith_model_configuration`](https://github.com/langchain-ai/terraform-provider-langsmith/blob/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources/model_configuration.md) resource calls the same endpoint, `api/v1/playground-settings`, and LangChain keeps it working, so its source shows the request langstar's `create` should send.

## Resolved Questions

### 1 - Does any official tool manage assistants?

No. Only the LangGraph SDKs, the Agent Server REST API and the UI do. The LangSmith CLI, the LangGraph CLI, the LangSmith MCP servers and the Terraform provider have no assistant commands or resources.

### 2 - Does Terraform cover model configurations and workspace secrets?

Yes. The provider has `langsmith_model_configuration` and `langsmith_workspace_secret`, which the garden page on LangChain tools does not mention. `langsmith_model_configuration` uses the same playground settings endpoint as langstar's `model-config`.

### 3 - Does langstar's `eval` group work today?

No. Each subcommand is a placeholder that does no work.

### 4 - Can the official CLI reach endpoints that have no command of their own?

Yes. `langsmith api` calls any LangSmith REST endpoint directly, so annotation queues, secrets and model configurations are reachable with hand-written JSON. It cannot reach assistants or graphs, which live on each deployment's Agent Server rather than the LangSmith API.

### 5 - What is langstar for?

codekiln, on 2026-10-09: "the main goal of this one cli 'langstar' is to be the one stop shop so people don't need to go to multiple tools to do things in langsmith (e.g. do I need mcp, langsmith cli, langgraph cli, etc)". Langstar therefore keeps every command group it has and closes gaps with the official tools rather than deferring to them.

## Open Questions

### 1 - Which gaps should langstar close first?

> Context from the drafting agent, for question 1.
>
> Recommendation: evaluators and experiments first, because evaluation is one of the main reasons people use LangSmith and langstar's `eval` commands do nothing today. Then reading runs, traces and threads one at a time, which is how people debug an agent. Add the missing prompt commands (delete, commit history and tags) to [ls-prompt-ux milestone · Issue #668](https://github.com/codekiln/langstar/issues/668), which already redesigns every `prompt` command. The HTTP 500 error from `model-config create` is already listed in [reboot milestone · Issue #746](https://github.com/codekiln/langstar/issues/746).

<ANSWER_HERE>

### 2 - Should langstar's `eval` group be rebuilt around the evaluators LangSmith runs?

> Context from the drafting agent, for question 2.
>
> The `eval` placeholders were designed for langstar to run evaluations itself. The official `evaluator` and `experiment` groups manage the evaluators LangSmith runs and read the results. Recommendation: replace the placeholders with commands for evaluators, the rules that attach them, and experiment results, and drop the plan for langstar to run evaluations itself.

<ANSWER_HERE>

### 3 - Should langstar cover the LangGraph CLI's local development commands?

> Context from the drafting agent, for question 3.
>
> `langgraph dev`, `up`, `build` and `new` need Python and Docker on the user's machine. Recommendation: have langstar call the installed `langgraph` command for these, so a user can stay inside langstar, rather than reimplementing them in Rust. This could wait until the LangSmith API gaps are closed.

<ANSWER_HERE>

### 4 - Should langstar manage what only Terraform manages, and by single commands or by applying a file?

> Context from the drafting agent, for question 4.
>
> Workspace administration, resource tags and alerts are in Terraform only. People who use Terraform for these may not want a second tool changing the same settings. Recommendation: leave the Terraform-only areas until someone asks for them. Spend that effort on a langstar command that reads assistant definitions from a file in Git and creates or updates them on each deployment, which no official tool does.

<ANSWER_HERE>

### 5 - Should the prompt command redesign also add delete, commit history and tags?

> Context from the drafting agent, for question 5.
>
> [ls-prompt-ux milestone · Issue #668](https://github.com/codekiln/langstar/issues/668) redesigns every `prompt` command around create, get, update and list, with `help` subcommands. The official `langsmith prompt` also has delete, commit history and tags, which langstar lacks. Recommendation: add those to the milestone, since the redesign already touches every `prompt` command.

<ANSWER_HERE>

### 6 - Should every list command get table output and column selection?

> Context from the drafting agent, for question 6.
>
> [ls-cli-output-dx milestone · Issue #529](https://github.com/codekiln/langstar/issues/529) adds table, json and text output and column selection to every list command. The official CLI offers only `pretty` and `json` output and `-o` to write JSON to a file, so this would put langstar ahead. Recommendation: go ahead, and apply it to new command groups as they are added.

<ANSWER_HERE>
