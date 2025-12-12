# LangSmith Environment Variables and Authentication

Canonical reference for LangSmith environment variables and their mapping to API authentication headers.

## Overview

Langstar uses three core environment variables for authentication across different LangSmith APIs:

- `LANGSMITH_API_KEY` - API key for LangSmith and LangGraph services
- `LANGSMITH_ORGANIZATION_ID` - Organization ID for scoping operations
- `LANGSMITH_WORKSPACE_ID` - Workspace ID for narrower scoping (also used as Tenant ID)

## Environment Variable to Header Mapping

| Environment Variable        | HTTP Header         | Usage                                                              |
| --------------------------- | ------------------- | ------------------------------------------------------------------ |
| `LANGSMITH_API_KEY`         | `X-Api-Key`         | Required for most APIs                                             |
| `LANGSMITH_ORGANIZATION_ID` | `x-organization-id` | Optional scoping header for LangSmith API                          |
| `LANGSMITH_WORKSPACE_ID`    | `X-Tenant-Id`       | Required for Control Plane API; optional scoping for LangSmith API |

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

| API               | Required Headers                | Required Env Vars                             | Optional Env Vars                                     |
| ----------------- | ------------------------------- | --------------------------------------------- | ----------------------------------------------------- |
| LangSmith API     | `X-Api-Key`                     | `LANGSMITH_API_KEY`                           | `LANGSMITH_ORGANIZATION_ID`, `LANGSMITH_WORKSPACE_ID` |
| Control Plane API | `X-Api-Key`, `X-Tenant-Id`      | `LANGSMITH_API_KEY`, `LANGSMITH_WORKSPACE_ID` | None                                                  |
| Agent Server API  | `X-Api-Key`                     | `LANGSMITH_API_KEY`                           | None                                                  |
| SCIM API          | `Authorization: Bearer <token>` | (Different auth method)                       | N/A                                                   |
| OpenTelemetry     | `X-Api-Key`                     | `LANGSMITH_API_KEY`                           | None                                                  |

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

### Minimal Setup (LangSmith API only)

```bash
export LANGSMITH_API_KEY="<your-api-key>"
```

### With Organization Scoping

```bash
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_ORGANIZATION_ID="<your-org-id>"
```

### With Workspace Scoping

```bash
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_ORGANIZATION_ID="<your-org-id>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
```

### Control Plane API Setup (Required)

```bash
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"  # Required!
```

### Integration Tests Setup (All Required)

```bash
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_ORGANIZATION_ID="<your-org-id>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
```

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
