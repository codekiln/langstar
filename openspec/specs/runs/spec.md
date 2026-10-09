# runs Specification

## Purpose

`langstar runs query` finds runs and traces in LangSmith projects through the LangSmith API.

## Requirements

### Requirement: Query runs by project

`langstar runs query` SHALL return runs from the projects named by `-p` or `--project`, which a user can repeat, and from every project when the user passes none. It SHALL accept a project only as a UUID, and SHALL warn about and skip any `--project` value that is not one. `--is-root` SHALL limit the results to the root run of each trace.

#### Scenario: Two projects

- **WHEN** a user runs `langstar runs query --project <project-id-1> --project <project-id-2>`
- **THEN** langstar returns runs from both projects

#### Scenario: Project given by name

- **WHEN** a user runs `langstar runs query --project my-project`
- **THEN** langstar warns that `my-project` is not a valid UUID
- **AND** queries without that project

### Requirement: Filter runs with flags

`langstar runs query` SHALL turn each `--tag` into `has(tags, "<tag>")`, each `--meta KEY=VALUE` into `eq(metadata["KEY"], "VALUE")`, `--status` into `eq(status, "<status>")` and `--errors-only` into `eq(error, true)`, and join them and any raw `--filter` expression with `and`. It SHALL warn about and skip a `--meta` value that has no `=`.

#### Scenario: Tag and status together

- **WHEN** a user runs `langstar runs query --tag production --status error`
- **THEN** langstar sends the filter `has(tags, "production") and eq(status, "error")`

### Requirement: Filter runs with LangSmith filter expressions

`langstar runs query` SHALL pass `--trace-filter` to LangSmith as a filter on the root run of each trace and `--tree-filter` as a filter on the other runs in the trace. `--run-type` SHALL limit the results to one of `tool`, `chain`, `llm`, `retriever`, `embedding`, `prompt` or `parser`.

#### Scenario: LLM runs only

- **WHEN** a user runs `langstar runs query --run-type llm`
- **THEN** every run in the results has the run type `llm`

### Requirement: Limit runs to a time window

`langstar runs query` SHALL return runs from the last 7 days unless the user picks another window. `--since` SHALL take a relative duration such as `15m`, `1h`, `7d` or `2w`, or an ISO 8601 timestamp. `--until` SHALL take an ISO 8601 timestamp. `--preset` SHALL take one of `1h`, `3h`, `6h`, `12h`, `1d`, `2d`, `7d` or `14d`, and `--since` SHALL win over it. `--no-time-filter` SHALL remove the time window.

#### Scenario: Relative start

- **WHEN** a user runs `langstar runs query --since 1h`
- **THEN** langstar returns runs that started in the last hour

#### Scenario: Unreadable start time

- **WHEN** a user passes a `--since` value that is neither a duration nor an ISO 8601 timestamp
- **THEN** langstar warns that it cannot read the value
- **AND** uses `--preset`, or the last 7 days when there is no preset

### Requirement: Page, sort and trim the results

`langstar runs query` SHALL return up to `-l` or `--limit` runs (default 100), fetching as many pages from LangSmith as that takes. `--order` SHALL sort runs by start time, newest first (`desc`, the default) or oldest first (`asc`). `--select` SHALL take a comma-separated list of fields and ask LangSmith to return only those fields.

#### Scenario: More runs than one page

- **WHEN** a user runs `langstar runs query --limit 250`
- **AND** the projects hold more than 250 matching runs
- **THEN** langstar returns 250 runs

### Requirement: Show runs as a table or JSON

`langstar runs query` SHALL choose its output with `-o` or `--output`: `table` (the default), `json` for compact JSON, or `json-pretty` for indented JSON. The table SHALL show each run's short ID, name, run type, status, total tokens, duration and start time in the timezone from langstar's config, followed by how many runs it found. Before the table it SHALL print the projects, time window, filter and limit it used.

#### Scenario: JSON for a script

- **WHEN** a user runs `langstar runs query -o json`
- **THEN** standard output holds one JSON array of runs and nothing else

### Requirement: Scope a query to an organization or workspace

`langstar runs query` SHALL take `--organization-id` and `--workspace-id`, each overriding the value from langstar's config and environment for this query. When a user passes both, it SHALL warn that it will use the workspace within that organization.

#### Scenario: Override the workspace

- **WHEN** a user runs `langstar runs query --workspace-id <your-workspace-id>`
- **THEN** langstar queries runs in that workspace instead of the configured one
