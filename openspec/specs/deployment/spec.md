# deployment Specification

## Purpose

`langstar deployment` manages the deployments in a LangSmith workspace through the control plane API.

## Requirements

### Requirement: List deployments

`langstar deployment list` SHALL list the workspace's deployments, taking `--limit` (default 20) and `--offset` (default 0). In table format it SHALL show each deployment's name, ID, status, source and creation date, and report how many it found. In JSON format it SHALL print an object with a `resources` array and the `offset`. When no deployment matches, it SHALL say that no deployments were found.

#### Scenario: Page through deployments

- **WHEN** a user runs `langstar deployment list --limit 10 --offset 10`
- **THEN** langstar shows at most 10 deployments, starting after the first 10

### Requirement: Filter the deployment list

`langstar deployment list` SHALL narrow the list with `--name-contains` (a substring of the name), `--status` (one of `READY`, `AWAITING_DATABASE`, `UNUSED`, `AWAITING_DELETE`, `AWAITING_FINAL_DELETE` or `UNKNOWN`, in any case) and `--deployment-type` (one of `dev_free`, `dev` or `prod`). It SHALL exit with an error that lists the valid values when a user passes any other status or type.

#### Scenario: Filter by status

- **WHEN** a user runs `langstar deployment list --status ready`
- **THEN** langstar lists only the deployments whose status is `READY`

#### Scenario: Unknown deployment type

- **WHEN** a user runs `langstar deployment list --deployment-type staging`
- **THEN** langstar exits with an error that lists `dev_free`, `dev` and `prod`

### Requirement: Choose the columns of text output

In text format, `langstar deployment list` SHALL print one tab-separated line per deployment. `--columns` SHALL choose the columns from `name`, `id`, `status`, `source` and `created_at`; without it, every column appears. `--show-columns` SHALL print the available column names and exit without listing deployments.

#### Scenario: Choose text columns

- **WHEN** a user runs `langstar deployment list -f text --columns name,status`
- **THEN** each output line holds a deployment's name and status, separated by a tab

#### Scenario: Unknown column

- **WHEN** a user passes a column name that is not in the list of available columns
- **THEN** langstar exits with an error that lists the available columns

### Requirement: Redact secret values

Every `langstar deployment` subcommand that prints a deployment SHALL replace each secret value with `<redacted>`, in every output format.

#### Scenario: Deployment with secrets

- **WHEN** a user runs `langstar deployment get <deployment-id>` on a deployment that has secrets
- **THEN** the output names each secret
- **AND** shows `<redacted>` as its value

### Requirement: Get one deployment

`langstar deployment get <deployment-id>` SHALL print the deployment as JSON, as the control plane returns it, with its secret values redacted.

#### Scenario: Get by ID

- **WHEN** a user runs `langstar deployment get <deployment-id>`
- **THEN** langstar prints that deployment's name, status, source configuration and revisions as JSON

### Requirement: Create a deployment from a GitHub repository

`langstar deployment create` SHALL take a required `--name` and a `--source` of `github` (the default) or `external_docker`, and SHALL exit with an error for any other source. For a `github` source it SHALL require `--repo-url` and `--branch`, and take `--config-path` (default `langgraph.json`) and `--deployment-type` (default `dev_free`). On success it SHALL print the new deployment's ID and status.

#### Scenario: Create from a branch

- **WHEN** a user runs `langstar deployment create --name my-agent --repo-url https://github.com/<owner>/<repo> --branch main`
- **THEN** langstar creates a `dev_free` deployment named `my-agent` that builds `langgraph.json` from the `main` branch
- **AND** prints the deployment's ID and status

#### Scenario: Branch missing

- **WHEN** a user runs `langstar deployment create` with a `github` source and no `--branch`
- **THEN** langstar exits with an error saying a branch is required

### Requirement: Find the GitHub integration ID

For a `github` source, `langstar deployment create` SHALL use the GitHub integration ID passed with `--integration-id`. Without the flag, it SHALL look the ID up itself, and when it finds none, it SHALL exit with an error that says how to supply the ID. Pull request [#754 replaces this lookup with a call to LangSmith's `GET /v1/integrations/github/install`](https://github.com/codekiln/langstar/pull/754) and keeps `--integration-id` as an override; it has not merged yet.

#### Scenario: ID passed on the command line

- **WHEN** a user runs `langstar deployment create` for a GitHub source with `--integration-id <integration-id>`
- **THEN** langstar creates the deployment with that integration ID

#### Scenario: No ID found

- **WHEN** a user creates a GitHub deployment without `--integration-id`
- **AND** langstar finds no integration ID for the workspace
- **THEN** langstar exits with an error that says how to supply the ID

### Requirement: Set deployment environment variables

`langstar deployment create` SHALL take `-e` or `--env` as `KEY=VALUE`, any number of times, and store each as a secret on the new deployment. It SHALL exit with an error when a value has no `=`.

#### Scenario: Malformed variable

- **WHEN** a user passes `--env OPENAI_API_KEY`
- **THEN** langstar exits with an error saying it expected `KEY=VALUE`

### Requirement: Wait for a new deployment to be ready

With `-w` or `--wait`, `langstar deployment create` SHALL check the new deployment's status every 10 seconds for the first 30 seconds and every 30 seconds after that, printing each status, until the status is `READY`. It SHALL then print the deployment, or in table format the total wait time.

#### Scenario: Wait until ready

- **WHEN** a user runs `langstar deployment create ... --wait`
- **THEN** langstar prints the status after each check
- **AND** exits once the deployment reports `READY`

### Requirement: Delete a deployment after confirmation

`langstar deployment delete <deployment-id>` SHALL ask the user to type `yes` before deleting the deployment, and SHALL skip the question when the user passes `-y` or `--yes`. In JSON format it SHALL print an object with `status` set to `deleted` and the `deployment_id`.

#### Scenario: User declines

- **WHEN** a user runs `langstar deployment delete <deployment-id>` and types anything other than `yes`
- **THEN** langstar prints that it cancelled and leaves the deployment in place

#### Scenario: Confirmation skipped

- **WHEN** a user runs `langstar deployment delete <deployment-id> --yes`
- **THEN** langstar deletes the deployment without asking
