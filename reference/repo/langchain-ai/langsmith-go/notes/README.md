# langsmith-go orientation note

- Repo: https://github.com/langchain-ai/langsmith-go
- Commit read: [59bb42c release: 0.28.0 (#237)](https://github.com/langchain-ai/langsmith-go/commit/59bb42c30052067fe488f9bb8370df9037c715ef); SDK version `0.28.0` (`internal/version.go`)
- All paths below are relative to `code/`.

## What it is

- Go client for the LangSmith REST API, module `github.com/langchain-ai/langsmith-go`, package name `langsmith`. `README.md` says it requires Go 1.22+, and `go.mod` declares `go 1.26.7`, which is the version Go enforces.
- `README.md` says it is "generated with Stainless"; most `*.go` files start with "File generated from our OpenAPI spec by Stainless" (`client.go`, `run.go`). `CONTRIBUTING.md` line 16: "Most of the SDK is generated code."
- `.stats.yml` holds only `configured_endpoints: 174` (no spec URL or hash). `api.md` has 175 `title="..."` method lines.
- Releases via release-please (`release-please-config.json`, `internal/version.go`).

## Client, transport, options

- `client.go`: `NewClient(opts...)` prepends `DefaultClientOptions()` then appends `withNormalizedBaseURL()`. `Client` has one service field per resource: ProductFeedback, Fleet, Sessions, Examples, Datasets, Runs, Threads, Charts, Traces, Evaluators, OnlineEvaluators, Feedback, Public, AnnotationQueues, Info, Workspaces, Repos, Commits, Settings, Issues, Sandboxes.
- `option/requestoption.go`: `WithBaseURL`, `WithMaxRetries`, `WithRequestTimeout`, `WithHeader`, `WithAPIKey`, `WithTenantID`, `WithEnvironmentProduction`. `option/middleware.go` holds middleware support.
- `internal/requestconfig/requestconfig.go`: `RequestConfig`, `Execute()` (retry loop), error mapping. Sets `Accept: application/json`, `X-Stainless-Retry-Count`, `X-Stainless-Timeout` headers plus platform headers.
- `default_http_client.go`: default `http.Client` clones `http.DefaultTransport` with `ResponseHeaderTimeout = 10 * time.Minute` (`defaultResponseHeaderTimeout`); skipped if the default transport is wrapped.
- Responses with status >= 400 become `*apierror.Error` (`internal/apierror`, used in `requestconfig.go` `Execute`).

## Retry behavior (`internal/requestconfig/requestconfig.go`)

- Default `MaxRetries: 2` (in `NewRequestConfig`), so up to 3 attempts. Override with `option.WithMaxRetries(n)`; negative panics.
- `shouldRetry`: no retry if the request body cannot be replayed (`Body != nil && GetBody == nil`); retry when there is no response (connection/transport error); response header `x-should-retry: true`/`false` overrides status logic; otherwise retry on 408 (`StatusRequestTimeout`), 409 (`StatusConflict`), 429 (`StatusTooManyRequests`) and any status >= 500. So 429 retries.
- `retryDelay`: if `parseRetryAfterHeader` succeeds, use it (clamped to >= 0). Headers in order: `Retry-After-Ms` (milliseconds), then `Retry-After` (seconds as float, or an HTTP-date parsed with `time.RFC1123`).
- Otherwise exponential backoff: `0.5s * 2^retryCount`, capped at `maxDelay = 16 * time.Second`, minus random jitter in `[0, delay/4)`.
- Retry count is sent in `X-Stainless-Retry-Count` unless the caller changed that header.
- langsmith-go retries 502, 503 and 504 like any other status of 500 or above.

## Pagination (`packages/pagination/pagination.go`)

- Types: `OffsetPaginationTopLevelArray`, `OffsetPaginationIssues`, `OffsetPaginationRepos`, `OffsetPaginationCommits`, `OffsetPaginationOnlineEvaluators`, `OffsetPaginationInsightsClusteringJobs`, `CursorPagination` (with `CursorPaginationCursors`), `ItemsCursorPostPagination` (field `next_cursor`), `ItemsCursorGetPagination`. Each has an `...AutoPager` with `Next/Current/Err/Index`.
- Offset style (`OffsetPaginationTopLevelArray.GetNextPage`): returns nil when the page is empty; next `offset` = current `offset` query value + number of items in the page.
- Tests: `paginationauto_test.go`, `paginationmanual_test.go`.

## Auth, tenant, env vars, base URL

- `client.go` `DefaultClientOptions()` reads, in this order: profile file options, then `LANGSMITH_ENDPOINT`, `LANGSMITH_API_KEY`, `LANGSMITH_TENANT_ID`, `LANGSMITH_WORKSPACE_ID`, `LANGCHAIN_CUSTOM_HEADERS` (newline-separated `Name: value` lines). Env vars override profile values (comment in `client.go`). Note: `LANGSMITH_WORKSPACE_ID` is also applied via `option.WithTenantID`, so it sets the same header as `LANGSMITH_TENANT_ID`; `README.md`'s env table lists only API_KEY, TENANT_ID, ENDPOINT.
- Headers (`option/requestoption.go`): `WithAPIKey` sets `X-API-Key` (and clears `Authorization` and `X-User-Id`); `WithTenantID` sets `X-Tenant-Id`. No organization-ID option or header was found in `option/requestoption.go`.
- Default base URL: `https://api.smith.langchain.com/` (`WithEnvironmentProduction`).
- `baseurl.go`: `withNormalizedBaseURL` strips a trailing `/api/v1` or `/api` path segment from the configured base URL, because generated request paths already carry `api/v1/...` or `v2/...`. Tracing ingest keeps the unstripped base URL (`client.go` `tracing()` uses `withoutBaseURLNormalization`).
- OAuth: `internal/auth/oauth.go` derives `X-User-Id` from the JWT `sub` claim; `oauth_discovery.go` handles discovery; `config.go` uses `oauthClientID = "langsmith-cli"`.

## Config profiles (hand-written, `config.go`)

- File: `~/.langsmith/config.json`, or the path in `LANGSMITH_CONFIG_FILE` (`configPath`).
- Profile fields (`configProfile`): `api_key`, `api_url`, `workspace_id`, `oauth` (access/refresh token).
- Profile selection (`resolveProfileName`): `LANGSMITH_PROFILE`, then `current_profile` in the file, then profile named `default`. `option WithProfile(name)` in `config.go` selects one explicitly for one client.
- A profile only supplies endpoint/auth/workspace when the matching env var is unset (`loadProfileOptionsForProfile`).
- Auth header from profile: `Authorization: Bearer <access_token>` if OAuth token present, else `X-API-Key` (`authHeaderFromProfile`). Tokens refresh with `tokenRefreshLeeway = time.Minute`, `tokenRefreshTimeout = 10 * time.Second`; file locking in `config_filelock_*.go`; `Client.AuthHeaders(ctx, ...)` exposes resolved headers.

## Endpoints and resource files

- `api.md` lists every method as `title="<verb> <path>"`; paths mix `/api/v1/...`, `/api/v2/...` and `/v1/...` (e.g. `post /api/v1/runs/query`, `post /api/v2/runs/query`, `post /api/v1/runs/batch`, `get /v1/fleet/threads/...`).
- One file per resource (`run.go`, `dataset.go`, `example.go`, `feedback.go`, `session.go`, `repo.go`, `commit.go`, `workspace.go`, `annotationqueue.go`, `onlineevaluator.go`, `evaluator.go`, `sandbox*.go`, ...), each with a `_test.go`.

## Generated vs hand-written

- Generated (Stainless header): most resource files, `client.go`, `default_http_client.go`, `option/`, `internal/requestconfig`, `packages/pagination`.
- No generated header (hand-written, from a header check of root `*.go`): `config.go`, `config_filelock_*.go`, `baseurl.go`, `address.go`, `field.go`, `insights.go`, `oauth_discovery.go`, `tracer.go`, `tracing.go`, and most `sandbox_*.go` (commands, files, tunnel, snapshots, mounts, wait, verify, errors, tracing, internal, resource, service_url).
- Other hand-written areas: `lib/langsmithtracing/` (run tracing/multipart ingestion client, re-exported by `tracing.go`), `instrumentation/`, `integration/`, `examples/` and sandbox command retries (`sandbox_command_start_retry_test.go`) (not read in detail).
