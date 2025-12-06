# LangGraph Deployments

Manage LangGraph Cloud deployment lifecycle using the `langstar deployment` commands. Deployments are hosted instances of your LangGraph applications, created and managed via the Control Plane API.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Quickstart](#quickstart)
- [Command Reference](#command-reference)
- [Common Workflows](#common-workflows)
- [Troubleshooting](#troubleshooting)

## Overview

Deployments in LangGraph Cloud represent hosted instances of your LangGraph applications. Each deployment:

- Runs your LangGraph application code from a Git repository
- Exposes an Agent Server API for running graphs and assistants
- Can be created as development (free or paid) or production tier

### Deployment Types

| Type | Description | Use Case |
|------|-------------|----------|
| `dev_free` | Free development deployment | Testing and development |
| `dev` | Paid development deployment | Team development with more resources |
| `prod` | Production deployment | Production workloads with HA and autoscaling |

### Deployment Sources

| Source | Description |
|--------|-------------|
| `github` | Deploy from a GitHub repository |
| `external_docker` | Deploy from an external Docker image |

## Prerequisites

1. **LangSmith API Key**: Set your API key:
   ```bash
   export LANGSMITH_API_KEY=<your-api-key>
   ```

2. **Langstar CLI**: Install langstar CLI (see main README for installation)

3. **GitHub Integration**: For GitHub source deployments, you need a GitHub integration configured in LangSmith

## Quickstart

### List Deployments

```bash
langstar deployment list
```

Output:
```
╭──────────────────────────────────────┬─────────────────────┬──────────┬───────────╮
│ ID                                   │ Name                │ Status   │ Type      │
├──────────────────────────────────────┼─────────────────────┼──────────┼───────────┤
│ 47599969-47ab-49d5-878e-cc6dbcbed059 │ my-deployment       │ READY    │ dev_free  │
│ 8a3b2c1d-1234-5678-abcd-ef0123456789 │ production-app      │ READY    │ prod      │
╰──────────────────────────────────────┴─────────────────────┴──────────┴───────────╯
```

### Create a Deployment

```bash
langstar deployment create \
  --name "my-deployment" \
  --source github \
  --repo-url https://github.com/owner/repo \
  --branch main \
  --deployment-type dev_free
```

### Get Deployment Details

```bash
langstar deployment get <deployment-id>
```

### Delete a Deployment

```bash
langstar deployment delete <deployment-id>
```

---

## Command Reference

### `langstar deployment list`

List all deployments accessible to your API key.

```bash
langstar deployment list [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `-l, --limit <N>` | Maximum deployments to return (default: 20) |
| `--offset <N>` | Skip N deployments for pagination (default: 0) |
| `--deployment-type <TYPE>` | Filter by type: `dev_free`, `dev`, `prod` |
| `--status <STATUS>` | Filter by status: `READY`, `AWAITING_DATABASE`, etc. |
| `--name-contains <SUBSTRING>` | Filter by name substring |
| `-f, --format <FORMAT>` | Output format: `table`, `json` |

**Examples:**
```bash
# List all deployments
langstar deployment list

# List only production deployments
langstar deployment list --deployment-type prod

# List ready deployments with JSON output
langstar deployment list --status READY --format json

# Paginated listing
langstar deployment list --limit 10 --offset 20
```

### `langstar deployment get`

Get details of a specific deployment.

```bash
langstar deployment get <DEPLOYMENT_ID>
```

**Arguments:**
- `<DEPLOYMENT_ID>` - The deployment UUID

**Examples:**
```bash
langstar deployment get 47599969-47ab-49d5-878e-cc6dbcbed059
```

### `langstar deployment create`

Create a new LangGraph deployment.

```bash
langstar deployment create [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | Deployment name (required) |
| `-s, --source <SOURCE>` | Source type: `github`, `external_docker` (default: `github`) |
| `--repo-url <URL>` | Repository URL (required for GitHub source) |
| `--branch <BRANCH>` | Git branch (required for GitHub source) |
| `--integration-id <ID>` | GitHub integration ID (auto-discovered if not provided) |
| `--config-path <PATH>` | Path to langgraph.json in repo (default: `langgraph.json`) |
| `-t, --deployment-type <TYPE>` | Deployment type (default: `dev_free`) |
| `-e, --env <KEY=VALUE>` | Environment variable (can be repeated) |
| `-w, --wait` | Wait for deployment to reach READY status |

**Examples:**
```bash
# Basic deployment
langstar deployment create \
  --name "my-deployment" \
  --source github \
  --repo-url https://github.com/owner/repo \
  --branch main

# Wait for deployment to be ready
langstar deployment create \
  --name "my-deployment" \
  --source github \
  --repo-url https://github.com/owner/repo \
  --branch main \
  --deployment-type dev_free \
  --wait

# Production deployment with environment variables
langstar deployment create \
  --name "production-app" \
  --source github \
  --repo-url https://github.com/owner/repo \
  --branch main \
  --deployment-type prod \
  --env "API_KEY=value1" \
  --env "DEBUG=false"
```

### `langstar deployment delete`

Delete a deployment.

```bash
langstar deployment delete <DEPLOYMENT_ID> [OPTIONS]
```

**Arguments:**
- `<DEPLOYMENT_ID>` - The deployment UUID to delete

**Options:**
| Option | Description |
|--------|-------------|
| `-y, --yes` | Skip confirmation prompt |

**Examples:**
```bash
# Delete with confirmation prompt
langstar deployment delete 47599969-47ab-49d5-878e-cc6dbcbed059

# Delete without confirmation
langstar deployment delete 47599969-47ab-49d5-878e-cc6dbcbed059 --yes
```

---

## Common Workflows

### CI/CD Deployment Pipeline

Create deployments automatically in CI/CD:

```bash
#!/bin/bash
# deploy.sh

DEPLOYMENT_NAME="app-${GITHUB_SHA:0:7}"
DEPLOYMENT_TYPE="${1:-dev_free}"

# Create deployment and wait for ready
langstar deployment create \
  --name "$DEPLOYMENT_NAME" \
  --source github \
  --repo-url "$GITHUB_REPOSITORY" \
  --branch "$GITHUB_REF_NAME" \
  --deployment-type "$DEPLOYMENT_TYPE" \
  --wait

# Get deployment ID for subsequent operations
DEPLOYMENT_ID=$(langstar deployment list \
  --name-contains "$DEPLOYMENT_NAME" \
  --format json | jq -r '.[0].id')

echo "Deployment created: $DEPLOYMENT_ID"
```

### Clean Up Old Deployments

List and delete old development deployments:

```bash
# List dev_free deployments
langstar deployment list --deployment-type dev_free --format json | \
  jq -r '.[].id' | \
  xargs -I {} langstar deployment delete {} --yes
```

### Check Deployment Status

```bash
# Get status of a specific deployment
langstar deployment get <deployment-id> --format json | jq '.status'
```

---

## Troubleshooting

### "Authentication failed" errors

1. Verify your `LANGSMITH_API_KEY` is set correctly
2. Ensure the API key has access to LangGraph Cloud
3. Check the key hasn't expired

### Deployment stuck in AWAITING_DATABASE

This is normal for new deployments. Use `--wait` flag or poll until status changes to READY:

```bash
# Poll until ready
while true; do
  STATUS=$(langstar deployment get <id> --format json | jq -r '.status')
  echo "Status: $STATUS"
  [ "$STATUS" = "READY" ] && break
  sleep 10
done
```

### GitHub integration not found

If `--integration-id` is not provided, langstar attempts to auto-discover it from existing deployments. If you have no existing deployments:

1. Create a deployment manually in the LangSmith UI first, or
2. Find your integration ID in LangSmith settings and provide it with `--integration-id`

---

## See Also

- [Graphs Documentation](./graphs.md) - Inspect graphs within deployments
- [Assistants Documentation](./README.md#langgraph-assistants-deployment-level) - Manage assistants
- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/)
