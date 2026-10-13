# langsmith-cli

- Repo: https://github.com/langchain-ai/langsmith-cli
- Commit read: `2613503` (HEAD of the local clone)
- All paths below are relative to `code/`.

## What it is

- An "agent-first CLI for querying and managing LangSmith resources" (`README.md`). Go module `github.com/langchain-ai/langsmith-cli`, entry point `cmd/langsmith/main.go`, built on cobra (`go.mod`).
- It is a consumer of langsmith-go: `go.mod` requires `github.com/langchain-ai/langsmith-go v0.27.1`. `internal/client/client.go` defines `Client` as a wrapper holding `SDK *langsmith.Client`. Typed calls go through `c.SDK.*` (e.g. `c.SDK.Runs.Query`, `c.SDK.Sessions.List`); endpoints the SDK lacks use raw helpers (`RawGet/RawPost/RawPatch/RawDelete`).
- Note: the local langsmith-go clone is at tag `v0.28.0`; the SDK facts below were read from `git show v0.27.1:...` in that clone, the version this CLI pins.

## Command definitions

- Root and group registration: `internal/cmd/root.go` (`NewRootCmd`). Groups registered there: `feedback`, `project`, `trace`, `run`, `chart`, `thread`, `dataset`, `example`, `evaluator`, `experiment`, `sandbox`, `insights`, `hub`, `apps`, `prompt`, `auth`, `profile`, `workspace`, `self-update`, `api`.
- One file per group in `internal/cmd/` (`trace.go`, `run.go`, `dataset.go`, `sandbox*.go`, `hub_*.go`, `apps_*.go`, `prompt.go`, ...). Sub-subgroups: `evaluator rule` (`evaluator_rule.go`), `insights config` (`insights_config.go`), `project issues` (`issues.go`), `sandbox snapshot` (`sandbox_snapshot.go`), `prompt tag` (`prompt.go`).
- `setup`, `setup claude`, `setup codex` exist in `internal/cmd/setup*.go`; `auth login` is in `login.go`, `auth info`/`auth token` in `auth.go`.
- `langsmith api` (`internal/cmd/api/api.go`) is a passthrough: `api ls`, `api info METHOD PATH`, or `api <path>` with `-X`, `-F`, `--input`, `--include`.
- A declarative helper, `internal/structured/` (`command.go`, `render.go`, `specs.go`), defines some commands as input/result/render specs (used by `sandbox`, `auth`).

## Flags and output

- Persistent root flags (`internal/cmd/root.go`): `--api-key` (hidden), `--api-url`, `--profile`, `--workspace` (alias `--workspace-id`, hidden), `--format` (default `pretty`; help text: "pretty or json").
- Per-command `--output/-o` writes JSON to a file on trace/run/thread commands (e.g. `internal/cmd/trace.go`). `--limit/-n`, `--include-metadata`, `--include-io`, `--include-feedback`, `--full` are on trace/run list (`trace.go`, `run.go`).
- `--jq` is added to commands built through `internal/structured/command.go` (and `sandbox create`, `sandbox_box.go`). `structured.Render` (`internal/structured/render.go`): if `--jq` is set, runs it with gojq; else if format is not `pretty` (or no text spec), pretty-printed JSON via `json.Encoder` with 2-space indent; else the command's text renderer.
- `internal/output/output.go`: `OutputJSON` (indented JSON to stdout or file, prints `{"status": "written", "path": ...}` to stderr when writing a file), `OutputJSONL`, `OutputTable` (borderless tablewriter), `OutputTree` (treeprint), `PrintOutput`.
- JSON shape for `trace list` is an array of run maps (`extractRunsToMaps`, `internal/cmd/trace.go`); with `--show-hierarchy` it is `[{trace_id, run_count, runs}]`.
- Pretty mode may print a stderr tip about `langsmith feedback`, at most once per 24h (`showFeedbackHint`, `root.go`).

## Auth, profiles, config

- Config file: `~/.langsmith/config.json`, overridable with `LANGSMITH_CONFIG_FILE` (`internal/config/config.go`, `DefaultConfigPath`). JSON shape: `current_profile` and `profiles` map; each profile has `api_key`, `api_url`, `workspace_id`, `oauth{access_token, refresh_token, expires_at, issuer}`. Saved atomically with mode `0600`; a warning is printed to stderr if the file is group/world accessible.
- Default API URL: `https://api.smith.langchain.com` (`config.DefaultAPIURL`).
- Profile selection (`ResolveProfileName`, `config.go`): `--profile`, then `LANGSMITH_PROFILE`, then `current_profile`, then a profile named `default`.
- Resolution in `resolveClientOptions` (`internal/cmd/root.go`):
  - API URL: default, then profile `api_url`, then `LANGSMITH_ENDPOINT` (ignored with a warning if `--profile` selected a profile with an `api_url`), then `--api-url`. `NormalizeURL` strips a trailing `/api/v1` (`client.go`).
  - Workspace: profile `workspace_id`, then `LANGSMITH_TENANT_ID`, then `LANGSMITH_WORKSPACE_ID`, then `--workspace`.
  - Credentials: `--api-key`, then `LANGSMITH_API_KEY` (warns if `--profile` was also given), then profile OAuth tokens, then profile `api_key`.
  - No credentials: error "not authenticated; run 'langsmith auth login', set LANGSMITH_API_KEY, or pass --api-key".
- `LANGSMITH_PROJECT` is the default for `--project` (`internal/cmd/filters.go`, `helpers.go`).
- `auth login` uses OAuth (`login.go`, `internal/client/oauth.go`); flags `--no-browser`, `--timeout`, `--prompt-workspace`.
- Profile auth is delegated to the SDK: `langsmith.WithProfile(name)` in `NewWithOptions` (`client.go`); a comment there says the SDK refreshes OAuth tokens under a lock shared across processes. The SDK also reads `LANGSMITH_API_KEY`, `LANGSMITH_TENANT_ID`, `LANGSMITH_WORKSPACE_ID`, `LANGSMITH_ENDPOINT` itself (langsmith-go `client.go` at v0.27.1).

## Retries

The CLI has no retry loop of its own. It uses the SDK's, and turns it off in specific places.

- SDK policy (langsmith-go `internal/requestconfig/requestconfig.go` at v0.27.1): `MaxRetries: 2` by default; retries connection errors (nil response), 408, 409, 429 and >=500; an `x-should-retry` header of `true`/`false` overrides the status check. Delay: `Retry-After-Ms`, then `Retry-After` (seconds or RFC1123 date), used verbatim; otherwise `0.5s * 2^retryCount`, capped at 16s, minus up to 25% jitter. `option.WithMaxRetries(n)` changes it.
- Typed SDK calls (e.g. `trace list` via `c.SDK.Runs...`) inherit this unchanged. Comment in `client.go` on `RawPostWithRetries` says so.
- Raw helpers in `internal/client/client.go`: `doHTTP` sends via `c.SDK.Execute` with `option.WithMaxRetries(0)` unless `sdkRetries` is true. `RawGet/RawPost/RawPatch/RawDelete`, `RawDo` and `FetchCustomAppSource` make one attempt. `RawPostWithRetries` opts into the SDK policy and is documented for read-only POSTs only; used by `trace messages` (`message.go`, `/api/v2/traces/messages`).
- Raw requests get a 30s timeout (`rawRequestTimeout`, `client.go`).
- `langsmith api` same-host requests go through `executeViaSDK` (`internal/cmd/api/request.go`) so they get the SDK policy (comment: 408/409/429/5xx, honouring Retry-After). It buffers every response body so a non-JSON error page (e.g. an edge 429) is still printed. Cross-host requests use a credential-free `RawDo` with a single attempt.
- `feedback` (`product_feedback.go`) passes `option.WithMaxRetries(0)` and an `IdempotencyKey`; on 429 it reports `retry after <Retry-After>` if the header is present.
- Unauthenticated path (`doUnauthenticated`, `client.go`) uses a plain `http.Client`, no retries.

## Pagination

- Runs v1 (`queryRuns`, `internal/cmd/helpers.go`): loops `c.SDK.Runs.Query`, follows `resp.Cursors.Next` into `params.Cursor` until the limit is reached or no cursor remains.
- Runs v2 (`queryRunsV2`, `helpers.go`): uses the SDK's `Runs.QueryV2AutoPaging` iterator, stopping at `limit`. Comment says it paginates on `next_cursor`.
- v1 vs v2 is chosen by `UseV2API` (`client.go`): `GET /info` version; non-release version (Cloud) means v2, semver `0.x` means v2 from minor 16 (`minSelfHostedV2Minor`), major != 0 means v2. Result cached per invocation.
- `thread list` queries root runs with page size 100 and `math.MaxInt32` as the cap, then groups by `thread_id` (`thread.go`). Default window is the last 24h.
- `thread messages` exposes `--cursor` and `--limit` (default 10, max 100); JSON output has `cursors.next` / `cursors.prev` parsed from an SSE stream (`thread.go`, `parseSSEGroups`).
- `project issues list` pages with `--limit` (default 50, max 500) and `--offset`; the caller advances manually (`issues.go`).
- `trace list` default `--limit` is 20, newest first (`trace.go`).

## HTTP client wrapping

- All auth'd raw requests are routed through the SDK client (`c.SDK.Execute` with an absolute URL) so the SDK attaches auth and refreshes OAuth under its lock; the comment in `doHTTP` says refreshing in the CLI raced other processes on the single-use refresh token.
- `requestURL` (`client.go`) requires a root-relative path and an http(s) base.
- Platform paths: `PlatformPath` gives `/api/v1/platform/...` for single-origin endpoints (URL ending in `/api` or `/api/v1`), else `/v1/platform/...` (`derivePlatformPrefix`).
- `api` middleware: `option.WithMiddleware` in `executeViaSDK` records the last response.

## Not verified

- Behaviour of commands beyond the files cited; no tests or binary were run.
