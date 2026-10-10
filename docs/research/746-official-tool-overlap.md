# Langstar's parity with the official LangChain tools

codekiln wants langstar to be the one command line tool a LangSmith user needs, so they never have to work out whether a task calls for the LangSmith CLI, the LangGraph CLI, an MCP server or Terraform. This document lists what each of those official tools can do, says whether langstar can do it too, and sizes each gap. The open questions at the end ask codekiln which gaps to close first.

Langstar is ahead on assistants and graphs, which no official command line tool manages, on annotation queues, which the LangSmith CLI reaches only through hand-written `langsmith api` requests, and on building a structured-output prompt from a template and a JSON Schema file. It is furthest behind on evaluators and experiments, where its `eval` commands are placeholders, and on what only Terraform manages: workspace administration, resource tags and alerts.

The official tools were read from source on 2026-10-09 at these commits, the same ones pinned on the garden page [LangSmith/Q/Which LangChain tools manage each part of a LangSmith setup?](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/LangSmith___Q___Which%20LangChain%20tools%20manage%20each%20part%20of%20a%20LangSmith%20setup%253F.md):

- LangSmith CLI at [`596db1e`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/root.go#L76-L95), written in Go, five commits after the v0.3.0 release. The link goes to the list of its command groups.
- LangSmith Terraform provider at [`0896d0f`](https://github.com/langchain-ai/terraform-provider-langsmith/tree/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources), two commits after the v0.0.16 release.
- LangGraph CLI and SDK at [`40a2e6d`](https://github.com/langchain-ai/langgraph/tree/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs).
- The LangSmith Remote MCP server's tool list, from the LangChain docs at [`1da22ee`](https://github.com/langchain-ai/docs/blob/1da22ee8525f1a71b17b86974f74f4f28da45347/src/langsmith/langsmith-remote-mcp.mdx#L165-L174).

Langstar was read at `main` as of [`7b09eb2` document tmux manager/worker and PR workflow (#759)](https://github.com/codekiln/langstar/commit/7b09eb2).

## Parity map

Each row is something an official tool can do. A gap is small when langstar needs a few more commands against an API it already calls, medium when it needs a new command group against an API it does not call yet, and large when it needs a new area of LangSmith or a different kind of tool.

| Capability | Official tool | Langstar | Gap |
|---|---|---|---|
| List, get, search, push and pull prompts | LangSmith CLI `prompt`; MCP `list_prompts`, `get_prompt_by_name` | Has it: `prompt` | None |
| Delete prompts, read commit history, manage tags | LangSmith CLI `prompt delete`, `commits`, `tag` | Lacks it | Small |
| Create, read, update and delete assistants | None; only the LangGraph SDKs and the Agent Server API | Has it: `assistant` | Langstar is ahead |
| Assistant versions, fixed assistant IDs, create-if-missing | None; only the LangGraph SDKs | Lacks it | Small |
| Show a deployed graph's structure | None; only the LangGraph SDKs | Has it: `graph` | Langstar is ahead |
| List, get, create and delete deployments | LangGraph CLI `deploy`; Terraform `langsmith_deployment` | Has it: `deployment` | None |
| Change a deployment's source revision, environment variables and secrets | Terraform `langsmith_deployment` | Lacks it; the SDK's update call ([`deployments.rs`](../../sdk/src/deployments.rs#L385)) changes only the source settings, its request has no environment variables or secrets ([`PatchDeploymentRequest`](../../sdk/src/deployments.rs#L231-L238)), and no command uses it | Small |
| Deployment logs and revisions, and the self-hosted Kubernetes clusters a deployment can be placed in (LangSmith calls the program it runs in each cluster a listener) | LangGraph CLI `deploy logs`, `deploy revisions list`, `deploy listeners list`; Terraform revision data sources | Lacks it | Small |
| Build an image from local code and deploy it | LangGraph CLI `deploy` | Lacks it | Large |
| Run a local Agent Server, build images, start a project from a template | LangGraph CLI `dev`, `up`, `build`, `dockerfile`, `new`, `validate` | Lacks it | Large |
| Query runs with filters | LangSmith CLI `run list`; MCP `fetch_runs` | Has it: `runs query` | None |
| Get one run, export runs, read traces as trees, read thread messages | LangSmith CLI `run`, `trace list`, `get`, `export`, `messages`, `thread`; MCP `get_thread_history` | Lacks it | Medium |
| Total the tokens, latency, costs and feedback of a project's traces | LangSmith CLI `trace stats` | Lacks it | Small |
| Set up Claude Code or Codex to send their traces to LangSmith | LangSmith CLI `trace setup` | Lacks it; this writes the coding agent's local settings rather than calling LangSmith | Medium |
| Charts, insight reports and issues for a tracing project | LangSmith CLI `chart`, `insights`, `project issues` | Lacks it | Medium |
| Manage annotation queues and their runs | None; the LangSmith CLI reaches them only through hand-written `langsmith api` requests | Has it: `queue` | Langstar is ahead |
| List, get, create and delete datasets | LangSmith CLI `dataset`; MCP `list_datasets`, `read_dataset` | Has it: `dataset` | None |
| Upload and export a dataset as one JSON file | LangSmith CLI `dataset upload`, `export` | Partly: `dataset import` and `export` read and write JSONL and CSV, so a file from `langsmith dataset export` cannot be imported as it is | Small |
| List the examples in a dataset | LangSmith CLI `example list`; MCP `list_examples` | Has it: `dataset list-examples` | None |
| Read, create and delete single examples | LangSmith CLI `example create`, `delete`; MCP `read_example` | Lacks it | Small |
| List and delete tracing projects | LangSmith CLI `project list`, `delete`; MCP `list_projects` | Has it: `project` | None |
| Look up a tracing project by name | Terraform `langsmith_project` data source | Has it: `project get` | None |
| Code and LLM-as-judge evaluators, the rules that attach them, experiment results | LangSmith CLI `evaluator`, `experiment`; Terraform `langsmith_evaluator`, `langsmith_run_rule`; MCP `list_experiments` | Lacks it: every `eval` command is a placeholder | Large |
| Model configurations | Terraform `langsmith_model_configuration` | Has it: `model-config` | None |
| Workspace secrets | Terraform `langsmith_workspace_secret` | Has it: `secrets` | None |
| Resource tags | Terraform `langsmith_tag_key`, `langsmith_tag_value`, `langsmith_tagging` | Lacks it | Medium |
| Workspaces, organization and workspace members, roles, access policies and the roles they attach to, and service keys | Terraform `langsmith_workspace`, `langsmith_org_membership`, `langsmith_workspace_membership`, `langsmith_workspace_role`, `langsmith_access_policy`, `langsmith_access_policy_attachment`, `langsmith_service_key`; the LangSmith CLI's `workspace` only lists workspaces and sets the default one | Lacks it | Large |
| Alert rules and gateway policies | Terraform `langsmith_alert_rule`, `langsmith_gateway_policy` | Lacks it | Medium |
| Sign in with OAuth, keep named profiles | LangSmith CLI `auth`, `profile` | Partly: `config` keeps one API key, organization ID, workspace ID, output format and timezone in a config file, and each can come from the environment instead | Medium |
| Call any LangSmith API endpoint | LangSmith CLI `api` | Lacks it | Small |
| Update the tool itself | LangSmith CLI `self-update` | Lacks it; the install script installs a new version | Small |
| Send feedback about the tool to the people who make it | LangSmith CLI `feedback` | Lacks it; people report langstar problems as GitHub issues | None needed |
| Billing usage | MCP `get_billing_usage` | Lacks it | Medium |
| Read the LangSmith server's version, license expiry and instance settings | Terraform `langsmith_info` data source | Lacks it | Small |
| Hub agent and skill repos, Custom Apps, sandboxes and their image registries | LangSmith CLI `hub`, `apps`, `sandbox`; Terraform `langsmith_sandbox_registry` | Lacks it | Large |

In the areas it shares with the official tools, langstar has commands they lack. `prompt push --schema` builds a structured-output prompt from a template and a JSON Schema file, where the official `push` takes a finished manifest. Only langstar can update a dataset with `dataset update`, import and export a dataset as CSV or JSONL with `dataset import` and `dataset export`, and create or update a tracing project with `project create` and `project update`. `runs query` takes one filter for the root run of each trace and another for the runs beneath it, and lets you choose which fields to print and how to sort them. `deployment create` asks the LangSmith API for the workspace's GitHub integrations and picks the one that can reach the repository ([`deployment.rs`](../../cli/src/commands/deployment.rs#L610-L634)), where Terraform's `langsmith_deployment` makes you supply the integration ID.

## What closing each gap would take

### Evaluators and experiments

Every langstar `eval` subcommand (`create`, `run`, `list`, `get`, `export`) has a `TODO` where the work should be, and `list` returns an empty list ([cli/src/commands/eval.rs#L413](../../cli/src/commands/eval.rs#L413)). The official [`langsmith evaluator`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/evaluator.go#L22) lists, gets and deletes evaluators, uploads code evaluators, creates LLM-as-judge rules, and manages the rules that attach evaluators to projects and datasets. [`langsmith experiment`](https://github.com/langchain-ai/langsmith-cli/blob/596db1ef1f320252f9148cc9ed4cd41392d0daee/internal/cmd/experiment.go#L15) lists and gets experiment results. Langstar's `eval` was designed around running evaluations itself, which is a different job from managing the evaluators LangSmith runs.

### Assistants kept in Git

To keep assistant definitions in Git and apply them to each deployment, langstar's `assistant create` needs to set the assistant's ID and to skip an assistant that already exists, and langstar needs commands for assistant versions. The LangGraph SDK has all of these ([`assistants.py`](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/sdk-py/langgraph_sdk/_async/assistants.py#L314-L376)). According to the survey in the garden report [LangSmith/Report/26/10/Assistants as Code in LangSmith Deployments](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/LangSmith___Report___26___10___Assistants%20as%20Code%20in%20LangSmith%20Deployments.md), no tool, official or community, keeps assistants in Git this way today.

### Local development and deploying from local code

The LangGraph CLI runs an Agent Server on your machine, builds Docker images and starts projects from templates ([cli.py](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/cli/langgraph_cli/cli.py#L276)), and [`langgraph deploy`](https://github.com/langchain-ai/langgraph/blob/40a2e6d845054cc0cc17a6a169ca6e7394e5231c/libs/cli/langgraph_cli/deploy.py#L2474-L2695) builds an image from local code before deploying it. The LangGraph CLI is a Python program, and its commands need different things on the user's machine: `up`, `build` and `deploy` run Docker, `dev` runs the Agent Server as a Python process, and `new` copies a project template. So langstar, a single Rust binary, would most likely call the installed `langgraph` command rather than reimplement them.

### Workspace administration, resource tags and alerts

Langstar can close these gaps by calling the same LangSmith API endpoints the Terraform provider calls ([resource list](https://github.com/langchain-ai/terraform-provider-langsmith/tree/0896d0f6fa7e882493389c4b3b53acb713fab03f/docs/resources)). Today people manage workspaces, members, roles, tags and alerts in the LangSmith UI, in Terraform, or with hand-written requests through `langsmith api`. Terraform applies a file that describes how a whole workspace should look, while langstar changes one setting per command, so a team that wants its workspace described in one file and applied in one step will keep using Terraform.

## Resolved Questions

### 1 - Does any official tool manage assistants?

No. Only the LangGraph SDKs, the Agent Server REST API and the UI do. The LangSmith CLI, the LangGraph CLI, the LangSmith MCP servers and the Terraform provider have no assistant commands or resources.

### 2 - Does Terraform cover model configurations and workspace secrets?

Yes. The provider has `langsmith_model_configuration` and `langsmith_workspace_secret`, which the garden page on LangChain tools does not mention. `langsmith_model_configuration` uses the same playground settings endpoint as langstar's `model-config`.

### 3 - Does langstar's `eval` group work today?

No. Each subcommand is a placeholder that does no work.

### 4 - Can the LangSmith CLI call LangSmith API endpoints it has no command for?

Yes. `langsmith api` calls any LangSmith REST endpoint directly, so annotation queues, secrets and model configurations are reachable with hand-written JSON. It cannot reach assistants or graphs, which live on each deployment's Agent Server rather than the LangSmith API.

### 5 - What is langstar for?

codekiln, on 2026-10-09: "the main goal of this one cli 'langstar' is to be the one stop shop so people don't need to go to multiple tools to do things in langsmith (e.g. do I need mcp, langsmith cli, langgraph cli, etc)". So we keep every langstar command group and add the commands langstar lacks, until a LangSmith user can do in langstar everything the official tools do.

## Open Questions

### 1 - Which gaps should langstar close first?

> Context from the drafting agent, for question 1.
>
> Recommendation: evaluators and experiments first, because evaluation is one of the main reasons people use LangSmith and langstar's `eval` commands do nothing today. Then reading runs, traces and threads one at a time, which is how people debug an agent.

<ANSWER_HERE>

### 2 - Should langstar's `eval` group be rebuilt around the evaluators LangSmith runs?

> Context from the drafting agent, for question 2.
>
> The `eval` placeholders were designed for langstar to run evaluations itself. The official `evaluator` and `experiment` groups manage the evaluators LangSmith runs and read the results. Recommendation: replace the placeholders with commands for evaluators, the rules that attach them, and experiment results, and drop the plan for langstar to run evaluations itself.

<ANSWER_HERE>

### 3 - Should langstar cover the LangGraph CLI's local development commands?

> Context from the drafting agent, for question 3.
>
> `langgraph dev`, `up`, `build` and `new` run inside the LangGraph CLI's Python program, and `up` and `build` also need Docker. Recommendation: have langstar call the installed `langgraph` command for these, so a user can stay inside langstar, rather than reimplementing them in Rust. This could wait until the LangSmith API gaps are closed.

<ANSWER_HERE>

### 4 - In what order should langstar add workspace administration, resource tags and alerts?

> Context from the drafting agent, for question 4.
>
> codekiln has no Terraform state backend, so langstar should manage workspace administration, resource tags and alerts by calling the LangSmith API, the same API the Terraform provider calls. Recommendation: add them in order of size. Resource tags first, a medium gap, because they label prompts, datasets, projects and deployments that langstar already manages. Then alert rules and gateway policies, also medium. Workspaces, members, roles, access policies and service keys last, the large gap.

<ANSWER_HERE>

### 5 - Should the prompt command redesign also add delete, commit history and tags?

> Context from the drafting agent, for question 5.
>
> We plan in [ls-prompt-ux milestone · Issue #668](https://github.com/codekiln/langstar/issues/668) to redesign every `prompt` command around create, get, update and list, with `help` subcommands. The official `langsmith prompt` also has delete, commit history and tags, which langstar lacks. Recommendation: add those to the milestone, since whoever does the redesign will already be rewriting every `prompt` command.

<ANSWER_HERE>

### 6 - Should every list command get table output and column selection?

> Context from the drafting agent, for question 6.
>
> We plan in [ls-cli-output-dx milestone · Issue #529](https://github.com/codekiln/langstar/issues/529) to give every list command table, JSON and text output and a flag to choose columns. The LangSmith CLI offers only `pretty` and `json` output, and `-o` to write JSON to a file, so this would put langstar ahead. Recommendation: go ahead, and give each new command group the same output options when we add it.

<ANSWER_HERE>
