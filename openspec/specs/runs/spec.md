# runs Specification

## Purpose

`langstar runs query` finds runs and traces in LangSmith projects through the LangSmith `POST /api/v2/runs/query` endpoint. Self-hosted LangSmith needs `v0.16` or later.

## Requirements

### Requirement: Query runs by project

`langstar runs query` SHALL return runs from the projects whose UUIDs the user passes with `-p` or `--project`, which a user can repeat. It SHALL warn about and skip each `--project` value that is not a UUID, and SHALL stop with exit status 1 before sending a request, saying that `runs query` needs at least one project UUID, when no UUID is left. The v2 endpoint cannot query across all projects. `--is-root` SHALL limit the results to the root run of each trace.

#### Scenario: Two projects

- **WHEN** a user runs `langstar runs query --project <project-id-1> --project <project-id-2>`
- **THEN** langstar returns runs from both projects

#### Scenario: Project given by name

- **WHEN** a user runs `langstar runs query --project my-project`
- **THEN** langstar warns that `my-project` is not a valid UUID
- **AND** stops with exit status 1 without sending a request

#### Scenario: No project

- **WHEN** a user runs `langstar runs query` with no `--project`
- **THEN** langstar says `runs query needs at least one project UUID: pass --project <UUID>`
- **AND** exits with status 1

### Requirement: Filter runs with flags

`langstar runs query` SHALL turn each `--tag` into `has(tags, "<tag>")`, each `--meta KEY=VALUE` into `eq(metadata["KEY"], "VALUE")`, and `--status` into `eq(status, "<status>")`, and join them and any raw `--filter` expression with `and`. It SHALL send `--errors-only` as the request's `has_error` field, because the v2 filter language cannot compare `error`. It SHALL warn about and skip a `--meta` value that has no `=`.

#### Scenario: Tag and status together

- **WHEN** a user runs `langstar runs query --tag production --status error`
- **THEN** langstar sends the filter `has(tags, "production") and eq(status, "error")`

### Requirement: Filter runs with LangSmith filter expressions

`langstar runs query` SHALL pass `--trace-filter` to LangSmith as a filter on the root run of each trace and `--tree-filter` as a filter on the other runs in the trace. `--run-type` SHALL limit the results to one of `tool`, `chain`, `llm`, `retriever`, `embedding`, `prompt` or `parser`.

#### Scenario: LLM runs only

- **WHEN** a user runs `langstar runs query --run-type llm`
- **THEN** every run in the results has the run type `llm`

### Requirement: Limit runs to a time window

`langstar runs query` SHALL return runs from the last 7 days unless the user picks another window. `--since` SHALL take a relative duration such as `15m`, `1h`, `7d` or `2w`, or an ISO 8601 timestamp. `--until` SHALL take an ISO 8601 timestamp. `--preset` SHALL take one of `1h`, `3h`, `6h`, `12h`, `1d`, `2d`, `7d` or `14d`, and `--since` SHALL win over it. `--no-time-filter` SHALL widen the window to the last 400 days, because v2 returns only the last day when it gets no start time and rejects a window longer than 401 days.

#### Scenario: Relative start

- **WHEN** a user runs `langstar runs query --since 1h`
- **THEN** langstar returns runs that started in the last hour

#### Scenario: Unreadable start time

- **WHEN** a user passes a `--since` value that is neither a duration nor an ISO 8601 timestamp
- **THEN** langstar warns that it cannot read the value
- **AND** uses `--preset`, or the last 7 days when there is no preset

### Requirement: Page, sort and trim the results

`langstar runs query` SHALL return up to `-l` or `--limit` runs (default 100), fetching as many pages from LangSmith as that takes. `--order` SHALL show runs by start time, newest first (`desc`, the default) or oldest first (`asc`). The v2 endpoint always returns the newest runs first and has no sort field, so `--order asc` SHALL fetch the newest `--limit` runs and print them oldest first. `--select` SHALL take a comma-separated list of fields and ask LangSmith to return only those fields. Without `--select`, a table SHALL ask for the fields its columns read, and JSON SHALL ask for every field except `share_url` and `attachments`, which carry credential-bearing URLs and appear only when `--select` names them.

#### Scenario: Oldest first

- **WHEN** a user runs `langstar runs query --project <project-id> --order asc --limit 5`
- **THEN** langstar returns the newest 5 runs, with the oldest of them first

#### Scenario: More runs than one page

- **WHEN** a user runs `langstar runs query --limit 250`
- **AND** the projects hold more than 250 matching runs
- **THEN** langstar returns 250 runs

### Requirement: Show runs as a table or JSON

`langstar runs query` SHALL choose its output with `-o` or `--output`: `table` (the default), `json` for compact JSON, or `json-pretty` for indented JSON. The table SHALL show each run's short ID, name, run type, status, total tokens, duration and start time in the timezone from langstar's config, followed by how many runs it found. Before the table it SHALL print the projects, time window, filter and limit it used.

#### Scenario: JSON for a script

- **WHEN** a user runs `langstar runs query -o json`
- **THEN** standard output holds one JSON array of runs, with the v2 field names such as `project_id`, and nothing else

### Requirement: Scope a query to an organization or workspace

`langstar runs query` SHALL take `--organization-id` and `--workspace-id`, each overriding the value from langstar's config and environment for this query. When a user passes both, it SHALL warn that it will use the workspace within that organization.

#### Scenario: Override the workspace

- **WHEN** a user runs `langstar runs query --workspace-id <your-workspace-id>`
- **THEN** langstar queries runs in that workspace instead of the configured one
