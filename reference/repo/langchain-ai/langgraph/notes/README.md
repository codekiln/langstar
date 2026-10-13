# langgraph (orientation: libs/cli and SDK pointers)

- Repo: https://github.com/langchain-ai/langgraph
- Commit read: `8a77332d0` (`git -C code rev-parse --short HEAD`)
- Local clone: `reference/repo/langchain-ai/langgraph/code` (read only)
- All paths below are relative to `code/`.

## Layout of libs/

`libs/` holds checkpoint, checkpoint-conformance, checkpoint-postgres, checkpoint-sqlite, cli, langgraph, prebuilt, sdk-js and sdk-py.

## libs/cli: the `langgraph` CLI

- Entry point: `langgraph = "langgraph_cli.cli:cli"` in `libs/cli/pyproject.toml` (`[project.scripts]`).
- Group `cli` is a click group (`libs/cli/langgraph_cli/cli.py:230`, class `NestedHelpGroup`). `cli.add_command(deploy)` is at `cli.py:237`.
- Defaults in `libs/cli/langgraph_cli/constants.py`: `DEFAULT_CONFIG = "langgraph.json"`, `DEFAULT_PORT = 8123`.

### Commands and where defined

| Command | Defined | Help text in source |
|---|---|---|
| `up` | `cli.py:276` (def at 278) | "Launch LangGraph API server." |
| `build` | `cli.py:412` | "Build LangGraph API server Docker image." |
| `dockerfile SAVE_PATH` | `cli.py:529` | "Generate a Dockerfile for the LangGraph API server, with Docker Compose options." |
| `dev` | `cli.py:758` | "Run LangGraph API server in development mode with hot reloading and debugging support" |
| `validate` | `cli.py:871` | "Validate the LangGraph configuration file." |
| `new [PATH]` | `cli.py:918` | "Create a new LangGraph project from a template." (impl `create_new` from `libs/cli/langgraph_cli/templates.py`) |
| `deploy` (group and command) | `libs/cli/langgraph_cli/deploy.py:2251-2267` | "[Beta] Build and deploy a LangGraph image to LangSmith Deployment." Run with no subcommand it forwards to `_deploy_cmd` (`deploy.py:2285`) |
| `deploy list` | `deploy.py:2480` | "[Beta] List LangSmith Deployments." |
| `deploy delete DEPLOYMENT_ID` | `deploy.py:2615` | "[Beta] Delete a LangSmith Deployment." |
| `deploy logs` | `deploy.py:2705` | "[Beta] Fetch LangSmith Deployment logs." |
| `deploy listeners list` | `deploy.py:2534` (group at 2516) | `--json` emits a JSON array instead of a table |
| `deploy revisions list DEPLOYMENT_ID` | `deploy.py:2580` (group at 2563) | `--limit` default 10 |

### Flags (as declared in source)

- Shared `OPT_*` options are at the top of `cli.py` (lines ~29-176):
  - `--config/-c` (default `langgraph.json`, must exist)
  - `--port` (default `DEFAULT_PORT`)
  - `--recreate/--no-recreate` (default False)
  - `--pull/--no-pull` (default True)
  - `--verbose`
  - `--watch`
  - `--debugger-port`
  - `--debugger-base-url`
  - `--postgres-uri`
  - `--docker-compose`
  - `--api-version`
  - `--engine-runtime-mode` (default `combined_queue_worker`; help says `distributed` uses separate executor and orchestrator containers)
- `up` also has `--image`, `--base-image` and `--wait` (implies `--detach`) (`cli.py:257-275`).
- `build`: `--tag/-t` (required), `--base-image`, `--install-command`, `--build-command`, and trailing docker build args passed through (`cli.py:380-411`).
- `dockerfile`: `--add-docker-compose`, `--base-image` (`cli.py:532-546`).
- `dev` (`cli.py:663-757`):
  - `--host` default `127.0.0.1`
  - `--port` default `2024`
  - `--no-reload`
  - `--config` default `langgraph.json`
  - `--n-jobs-per-worker`
  - `--no-browser`
  - `--debug-port`
  - `--wait-for-client`
  - `--studio-url` (help: defaults to `https://smith.langchain.com`)
  - `--allow-blocking`
  - `--tunnel` (Cloudflare)
  - `--server-log-level` default `WARNING`
  - `--ssl-certfile`
  - `--ssl-keyfile`
- `new`: `--template` (`cli.py:913`).
- `deploy` options come from `_deploy_base_options` in `deploy.py`, including `--deployment-id`, `--deployment-type`, `--no-wait`, `--tag`, `--image`, `--api-key`, `--name`, `--agent-id` and `--agent-environment`.
  - `--agent-id` and `--agent-environment` are required together and cannot combine with `--name` or `--deployment-id` (`deploy.py:2327-2337`).
  - `--agent-environment` choices: `development`, `staging`, `production` (`deploy.py:2008-2013`).
- `deploy list`: `--name-contains` (`deploy.py:2475`).
- `deploy delete`: `--force` skips the prompt (`deploy.py:2608`).
- `deploy logs` (`deploy.py:2651-2705`):
  - `--type` choice `deploy` or `build` (default `deploy`)
  - `--revision-id`
  - `--level` (DEBUG, INFO, WARNING, ERROR, CRITICAL)
  - `--limit` default 100
  - `--query/-q`
  - `--start-time`
  - `--end-time`
  - `--follow/-f`

## Config file (langgraph.json)

- Typed in `libs/cli/langgraph_cli/schemas.py`: `class Config(TypedDict, total=False)` at line 615. Fields seen at the top of the class include `python_version`, `node_version`, `api_version`, `base_image`, `image_distro`, `pip_config_file`, `pip_installer`, `source`, `dockerfile_lines` and `dependencies`. Read the class for the rest (`graphs`, `env`, `store`, `auth`, `http` and others have their own TypedDicts there).
- JSON Schema output: `libs/cli/schemas/schema.json` (also `schema.v0.json`, `version.schema.json`), generated by `libs/cli/generate_schema.py` via `make update-schema` (`libs/cli/Makefile:37`).
- `schema.json` says `python_version` enum is `3.11`, `3.12`, `3.13`.
- The `--config` help text (`cli.py:44-98`) describes `dependencies`, `graphs`, `env`, `python_version`, `pip_config_file` and `dockerfile_lines`.
- Loading and validation logic is in `libs/cli/langgraph_cli/config.py` (not read in depth).

## Auth, API keys, env vars (CLI)

- `deploy` API key resolution:
  - `--api-key` has `envvar="LANGGRAPH_HOST_API_KEY"` (`deploy.py:1973-1980`).
  - `_API_KEY_ENV_NAMES` at `deploy.py:~90` lists `LANGGRAPH_HOST_API_KEY`, `LANGSMITH_API_KEY`, `LANGCHAIN_API_KEY`.
  - The loop at `deploy.py:~1885-1895` checks the project `.env` values first, then `os.environ`, for each name.
  - If none is found it errors ("No LangSmith API key found...") in no-input mode, otherwise it prompts with hidden input (`deploy.py:1896-1903`).
- Other deploy env vars:
  - `LANGSMITH_TENANT_ID` (becomes the `X-Tenant-ID` header)
  - `LANGSMITH_ENDPOINT`
  - `LANGGRAPH_HOST_URL` (hidden `--host-url`)
  - `LANGSMITH_DEPLOYMENT_NAME` (`--name`)
  - `LANGSMITH_AGENT_ID`
  - `LANGSMITH_AGENT_ENVIRONMENT`
- `deploy.py:45-77` holds a reserved env-var list (e.g. `LANGSMITH_API_KEY`, `LANGGRAPH_CLOUD_LICENSE_KEY`, `LANGCHAIN_API_KEY`).
- `up` help (`cli.py:297-298`): local dev needs `LANGSMITH_API_KEY`. Production needs a license key in `LANGGRAPH_CLOUD_LICENSE_KEY`.
- Analytics opt-out: `LANGGRAPH_CLI_NO_ANALYTICS == "1"` (`libs/cli/langgraph_cli/analytics.py:89`). Source label from `LANGGRAPH_CLI_ANALYTICS_SOURCE` (`analytics.py:35`).

## CLI HTTP client (control plane for `deploy`)

- `libs/cli/langgraph_cli/host_backend.py`, class `HostBackendClient` (httpx).
- Base URLs: `CLOUD_CONTROL_PLANE_URL = "https://api.host.langchain.com"`, `CLOUD_DASHBOARD_URL = "https://smith.langchain.com"`, `CONTROL_PLANE_PATH = "/api-host"`.
- Headers: `X-Api-Key`, `Accept: application/json`, and `X-Tenant-ID` when a tenant is set (`host_backend.py:139-144`).
- Retry and timeout: `httpx.HTTPTransport(retries=3)`, `timeout=30` (`host_backend.py:147-151`). httpx transport retries cover connection errors only. No status-code retry loop is in this file.
- Errors: `HTTPStatusError` and `TransportError` become `HostBackendError` (`host_backend.py:175-184`).
- Pagination: no offset. Only a `limit` query param. `MAX_PAGE_SIZE = 100`, used for `GET /v2/listeners`. `list_revisions` defaults `limit=1`.
- Endpoints used (all relative to the control plane URL):
  - `POST` and `GET /v2/deployments`
  - `GET`, `PATCH` and `DELETE /v2/deployments/{id}`
  - `POST /v2/deployments/{id}/push-token`
  - `GET /v2/deployments/{id}/revisions`
  - `GET /v2/listeners`
  - `POST /v1/projects/{project_id}/revisions/{revision_id}/build_logs`
  - `POST /v1/projects/{project_id}/deploy_logs`

## SDK clients: where they live

- `libs/sdk-py`: package `langgraph-sdk` (`libs/sdk-py/pyproject.toml`, "SDK for interacting with LangGraph API"). This is the client for the LangGraph server / LangSmith Deployment (Agent Server) API.
  - `libs/sdk-py/langgraph_sdk/client.py` only re-exports. The code is in `_async/` and `_sync/`.
  - Each has `client.py` (factory), `assistants.py`, `threads.py`, `runs.py`, `cron.py`, `store.py`, `http.py`.
  - Shared helpers: `_shared/utilities.py`.
- `libs/sdk-js`: only a README saying the package moved to https://github.com/langchain-ai/langgraphjs (`libs/sdk-js/README.md`). No client code here.
- The checkpoint, prebuilt and langgraph libs are in-process libraries, not HTTP clients of the server (not read in depth).

### sdk-py details (`libs/sdk-py/langgraph_sdk`)

- Factories: `get_client(url=...)` in `_async/client.py`, `get_sync_client` in `_sync/client.py`. `url=None` tries an in-process ASGI connection to a running server (`_async/client.py`).
- API key: `_get_api_key` in `_shared/utilities.py:26-48`. Precedence is explicit string, then `LANGGRAPH_API_KEY`, `LANGSMITH_API_KEY`, `LANGCHAIN_API_KEY`. Passing `api_key=None` skips the environment. Sent as header `x-api-key`. `x-api-key` is in `RESERVED_HEADERS` and custom headers cannot set it. `User-Agent` is `langgraph-sdk-py/<version>`.
- Retries: `httpx.AsyncHTTPTransport(retries=5)` (async, `_async/client.py`) and `httpx.HTTPTransport(retries=5)` (sync, `_sync/client.py:75`). Connection-level only.
- Timeout default: `httpx.Timeout(connect=5, read=300, write=300, pool=5)`.
- Reconnect: `request_reconnect` in `_async/http.py:138-185` follows a `Location` header with `reconnect_limit: int = 5`.
- Pagination: `limit: int = 10` and `offset: int = 0` on search methods, e.g. `_async/threads.py:304-305`, `_async/assistants.py:501-502` (search) and a `limit` in thread history (`threads.py:691`). Sent as `limit` and `offset` in the request body (`threads.py:352-353`).
- Paths are the server API, e.g. `/assistants/{assistant_id}`, `/assistants/{assistant_id}/graph`, `/assistants/{assistant_id}/schemas` (`_async/assistants.py`).

## Not verified here

Request bodies for each server route, `config.py` validation rules, the `up` docker-compose generation, and the `langgraph` and `prebuilt` libraries.
