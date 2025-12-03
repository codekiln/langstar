# LangGraph Deployments and Revisions

This document is the single source of truth for understanding LangGraph Cloud deployment and revision statuses.

## Overview

LangGraph Cloud has two distinct status types that are often confused:

| Entity | Status Enum | Terminal Success | Where Visible |
|--------|-------------|-----------------|---------------|
| **Deployment** | `DeploymentStatus` | `Ready` | `langstar graph list` |
| **Revision** | `RevisionStatus` | `Deployed` | Revision polling, API responses |

## Deployment vs Revision

A **Deployment** is the container for your LangGraph application. It has an ID, name, and configuration.

A **Revision** is a specific build/version of the deployment. Each time you create or patch a deployment, a new revision is created that goes through build and deploy stages.

```
Deployment (e.g., "pr-integration-test")
├── Revision 1 (initial) → Deployed
├── Revision 2 (after patch) → Deployed
└── Revision 3 (latest) → Building...
```

## DeploymentStatus

The overall status of a deployment resource.

| Status | Description |
|--------|-------------|
| `AwaitingDatabase` | Database provisioning in progress |
| `Ready` | **Terminal success** - Deployment is operational |
| `Unused` | Deployment is inactive |
| `AwaitingDelete` | Deletion in progress |
| `Unknown` | Status cannot be determined |

**API Value**: `SCREAMING_SNAKE_CASE` (e.g., `"READY"`, `"AWAITING_DATABASE"`)

**Rust Enum**: `DeploymentStatus::Ready`

## RevisionStatus

The build/deploy status of a specific revision.

| Status | Description |
|--------|-------------|
| `Queued` | Waiting in build queue |
| `Building` | Docker image build in progress |
| `BuildSucceeded` | Build completed, awaiting deployment |
| `BuildFailed` | Build failed (check logs) |
| `AwaitingDeploy` | Waiting to be deployed |
| `Deploying` | Container deployment in progress |
| `Deployed` | **Terminal success** - Revision is live |
| `DeployFailed` | Deployment failed (check logs) |
| `Cancelled` | Build/deploy was cancelled |
| `Unknown` | Status cannot be determined |

**API Value**: `SCREAMING_SNAKE_CASE` (e.g., `"DEPLOYED"`, `"BUILDING"`)

**Rust Enum**: `RevisionStatus::Deployed`

## Typical Revision Lifecycle

```
Queued → Building → BuildSucceeded → AwaitingDeploy → Deploying → Deployed
                 ↘ BuildFailed
                                                              ↘ DeployFailed
```

## Test Fixtures Behavior

The `get_or_create_deployment()` function in `sdk/src/test_utils.rs`:

1. **Lists deployments** without filtering by `DeploymentStatus` (finds deployments in any status)
2. **Finds by name** to locate existing deployment
3. **Gets latest revision** and checks its `RevisionStatus`
4. **Waits for `RevisionStatus::Deployed`** if not already deployed

This ensures tests can reuse deployments even when they're still building from a previous run.

## Common Confusion Points

### "Why does `langstar graph list` show Ready but tests wait for Deployed?"

These are different statuses for different entities:
- `Ready` = The **deployment** is operational (`DeploymentStatus`)
- `Deployed` = The **revision** has finished deploying (`RevisionStatus`)

A deployment can be `Ready` while its latest revision is still `Building`.

### "Which status should I filter by?"

- **Finding existing deployments**: Don't filter by status (or filter by `DeploymentStatus` if needed)
- **Waiting for a build to complete**: Poll `RevisionStatus` until `Deployed`

## References

- Rust types: `sdk/src/deployments.rs` (`DeploymentStatus`, `RevisionStatus`)
- Test utilities: `sdk/src/test_utils.rs`
- CLI fixtures: `cli/tests/common/fixtures.rs`
