# LangSmith Environment Variables and Authentication

Canonical reference for LangSmith environment variables and their mapping to API authentication headers.

## Overview

Langstar uses three core environment variables for authentication across different LangSmith APIs:

- `LANGSMITH_API_KEY` - API key for LangSmith and LangGraph services
- `LANGSMITH_ORGANIZATION_ID` - Organization ID for scoping operations
- `LANGSMITH_WORKSPACE_ID` - Workspace ID for narrower scoping (also used as Tenant ID)

## Loading the Variables Locally

On your own computer, [fnox](https://fnox.jdx.dev) reads the LangSmith credentials from 1Password once and keeps an encrypted copy in your gitignored `fnox.local.toml`, so tests run without asking 1Password each time. The committed `fnox.toml` says only which 1Password item and field holds each credential. This is the setup the [fnox guide](https://fnox.jdx.dev/guide/golden-path.html) recommends.

| File | Committed | Contents |
|------|-----------|----------|
| `fnox.toml` | Yes | References for `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID` and `LANGSMITH_WORKSPACE_ID`, each with `if_missing = "error"` |
| `fnox.local.toml.example` | Yes | Template for the local layer, with a placeholder vault |
| `fnox.local.toml` | No | The `langsmith` provider, which names your vault, and the encrypted cache that `fnox sync` writes |
| `mise.toml` | Yes | Pins `fnox`, `1password-cli`, `age` and `cargo-nextest`. Sets the non-secret fixture names `TEST_GRAPH_ID`, `REPOSITORY_OWNER` and `REPOSITORY_NAME` in `[env]` |

The vault name stays out of the repository. Only the provider in your `fnox.local.toml` names it.

`langstar deployment create` looks up the GitHub integration ID from the workspace's existing GitHub deployments, so developers keep no integration ID in 1Password. The lookup needs at least one GitHub deployment in the workspace; in a workspace with none, the command stops with an error.

### One-time machine setup

Do this once per machine. Every project on the machine then shares the key and the provider.

```bash
# Generate your personal age key
mkdir -p ~/.config/fnox
age-keygen -o ~/.config/fnox/age.txt

# Add the machine-wide sync provider
fnox provider add sync-age age --global
```

Then edit `~/.config/fnox/config.toml` so the provider uses your public key and key file:

```toml
[providers.sync-age]
type = "age"
recipients = ["age1..."] # the "public key:" line from ~/.config/fnox/age.txt
key_file = "~/.config/fnox/age.txt"
```

Sign in to the 1Password CLI (`op signin`, or turn on the desktop app integration) and check that `op vault list` shows the vault that holds the LangSmith item.

#### Optional: keep the age key in the Secure Enclave

On a Mac with Touch ID, the age key can live in the Secure Enclave, so it cannot be copied off the machine. Depending on the access-control policy, every decryption may then ask for Touch ID, which stalls unattended agent runs. Keep the plain key file if agents run tests for you. Only the provider changes:

```bash
brew install age-plugin-se
age-plugin-se keygen --access-control=any-biometry -o ~/.config/fnox/age-se.txt
```

```toml
[providers.sync-age]
type = "age"
recipients = ["age1se1..."] # the public key that keygen printed
key_file = "~/.config/fnox/age-se.txt"
```

Re-sync after you switch keys, because the old cache is encrypted to the old key. See [fnox: Apple Secure Enclave](https://fnox.jdx.dev/guide/sync.html#apple-secure-enclave-touch-id) for the access-control options and for YubiKey and TPM keys.

### Per-checkout setup

Keep the real `fnox.local.toml` in the main clone and link to it from each worktree, so one `fnox sync` fills the cache for every worktree. A worktree sits inside the main clone, and fnox lets the worktree's own `fnox.toml` replace each secret from the main clone's cache, so without the link fnox in a worktree asks 1Password for every secret:

```bash
# In the main clone
cp fnox.local.toml.example fnox.local.toml
"${EDITOR:-vi}" fnox.local.toml               # set vault = "<your-1password-vault>"
fnox sync --provider sync-age --local-file    # asks 1Password once
fnox check --all

# In each new worktree, right after `git worktree add`
mise run worktree:link-secrets
```

`mise-tasks/worktree/link-secrets` creates the relative symlink `fnox.local.toml -> ../../fnox.local.toml`. Running it again changes nothing, and it refuses to replace a real `fnox.local.toml`. Use `mise run worktree:link-secrets --check` to test a worktree without changing it. `fnox sync` writes through the symlink, so a sync run from any worktree updates the shared cache. If you keep a single checkout, skip this step.

### Run commands with the secrets

```bash
fnox exec -- cargo nextest run --profile ci --all-features --workspace
```

`fnox exec` decrypts the cache with your age key and does not call 1Password. When fnox cannot find a secret, `fnox exec` stops before the command starts, so a missing credential stops the test run instead of letting integration tests skip. Silently skipped tests are why the project requires all three variables; see [#660 Final testing verification for ls-prompt-structured-outputs milestone](https://github.com/codekiln/langstar/issues/660). For an interactive shell, `eval "$(fnox activate zsh)"` loads the secrets when you `cd` into the project. See [fnox shell integration](https://fnox.jdx.dev/guide/shell-integration.html).

### Re-sync after a rotation

The cache does not refresh itself. After a value changes in 1Password, such as a rotated API key, or after a reference is added to `fnox.toml`, re-sync:

```bash
fnox sync --provider sync-age --local-file --force
fnox check --all
```

### Devcontainer and Codespaces (secondary)

The devcontainer still reads the variables from `.devcontainer/.env` through `docker-compose.yml`, and Codespaces reads them from Codespaces secrets. See [getting-started.md](./getting-started.md). This path keeps working, but host development through fnox is the primary path.

### CI

CI reads GitHub Actions secrets (`secrets.LANGSMITH_API_KEY`, `secrets.LANGSMITH_ORGANIZATION_ID`, `secrets.LANGSMITH_WORKSPACE_ID`) and does not use fnox. To read secrets from 1Password, CI would need a 1Password service account, and that account can read every item in each vault it is granted. Whether to accept that belongs in its own issue.

## Environment Variable to Header Mapping

| Environment Variable | HTTP Header | Usage |
|---------------------|-------------|-------|
| `LANGSMITH_API_KEY` | `X-Api-Key` | Required for most APIs |
| `LANGSMITH_ORGANIZATION_ID` | `x-organization-id` | Optional scoping header for LangSmith API |
| `LANGSMITH_WORKSPACE_ID` | `X-Tenant-Id` | Required for Control Plane API; optional scoping for LangSmith API |

## API Requirements by Environment Variable

### LangSmith API

**Required:**
- `LANGSMITH_API_KEY` → `X-Api-Key` header

**Optional (for scoping):**
- `LANGSMITH_ORGANIZATION_ID` → `x-organization-id` header
- `LANGSMITH_WORKSPACE_ID` → `X-Tenant-Id` header

**Notes:**
- Both `x-organization-id` and `X-Tenant-Id` can be used together for workspace-scoped requests
- When workspace ID is set, it provides narrower scoping than organization ID alone
- Scoping headers affect which resources are accessible (e.g., private prompts vs public prompts)

### LangSmith Deployment Control Plane API

**Required:**
- `LANGSMITH_API_KEY` → `X-Api-Key` header
- `LANGSMITH_WORKSPACE_ID` → `X-Tenant-Id` header

**Documentation:** https://api.host.langchain.com/docs

**Notes:**
- Both headers are required for all Control Plane API requests
- The `X-Tenant-Id` header uses the workspace ID value
- This API manages LangGraph deployments, revisions, and integrations

### LangSmith Deployment Agent Server API

**Required:**
- `LANGSMITH_API_KEY` → `X-Api-Key` header

**Notes:**
- No scoping headers are used (assistants are deployment-level resources)
- The API key is tied to a specific deployment
- All operations are automatically scoped to that deployment

### SCIM API

**Required:**
- Bearer token (different authentication method)

**Notes:**
- SCIM API uses `Authorization: Bearer <token>` header
- Not using the standard LangSmith environment variables
- Enterprise-only feature for user provisioning

### OpenTelemetry Endpoints

**Required:**
- `LANGSMITH_API_KEY` → `X-Api-Key` header

**Notes:**
- No scoping headers needed
- Used for ingesting traces, logs, and Claude Code telemetry

## Complete API Requirements Table

| API | Required Headers | Required Env Vars | Optional Env Vars |
|-----|----------------|-------------------|------------------|
| LangSmith API | `X-Api-Key` | `LANGSMITH_API_KEY` | `LANGSMITH_ORGANIZATION_ID`, `LANGSMITH_WORKSPACE_ID` |
| Control Plane API | `X-Api-Key`, `X-Tenant-Id` | `LANGSMITH_API_KEY`, `LANGSMITH_WORKSPACE_ID` | None |
| Agent Server API | `X-Api-Key` | `LANGSMITH_API_KEY` | None |
| SCIM API | `Authorization: Bearer <token>` | (Different auth method) | N/A |
| OpenTelemetry | `X-Api-Key` | `LANGSMITH_API_KEY` | None |

## Testing Context

**Why all three variables are required for tests:**

As documented in issue #660, silent skip patterns in tests led to requiring all three environment variables (`LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID`, `LANGSMITH_WORKSPACE_ID`) for all integration tests, even though individual tests may not need all three.

This approach was chosen because:
1. It's simpler to explain than requiring different variables for different tests
2. It ensures tests have all necessary credentials available
3. It prevents silent test skips that create false confidence

**For test writers:**
- Always require all three variables explicitly using `.expect()` (no silent skips)
- See `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` for testing standards
- Reference `docs/dev/testing/test-fixtures.md` for test deployment setup

## Quick Reference

| Need | Command |
|------|---------|
| Run all tests with the secrets | `fnox exec -- cargo nextest run --profile ci --all-features --workspace` |
| Check that every secret resolves | `fnox check --all` |
| Refresh the cache after a rotation | `fnox sync --provider sync-age --local-file --force` |
| See which config files fnox loaded | `fnox config-files` |

Integration tests need `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID` and `LANGSMITH_WORKSPACE_ID`, and `fnox exec` sets each of them.

## Implementation Details

The mapping is implemented in:
- `sdk/src/auth.rs` - `AuthConfig` struct that loads environment variables
- `sdk/src/client.rs` - HTTP client methods that add headers based on `AuthConfig`

**Key implementation notes:**
- `LANGSMITH_WORKSPACE_ID` maps to `X-Tenant-Id` header (workspace ID = tenant ID)
- Control Plane API methods require `workspace_id` to be set (see `control_plane_*` methods)
- LangSmith API methods optionally add scoping headers if set (see `langsmith_*` methods)
- LangGraph API methods do not add scoping headers (deployment-level resources)

## See Also

- `reference/api-specs/LANGSMITH_APIS_DETAILS.md` - Complete API specifications catalog
- `reference/api-specs/LANGSMITH_API_OVERVIEW.md` - Quick API overview
- `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` - Testing standards and requirements
- `docs/dev/testing/test-fixtures.md` - Test deployment setup
- `docs/dev/testing/debugging-tests.md` - Troubleshooting test failures
- Issue #660 - Context on why all three variables are required for tests
