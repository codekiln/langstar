# Control Plane API Experiment - Issue #178

**Date**: 2025-11-20
**Issue**: [#178](https://github.com/codekiln/langstar/issues/178)
**Objective**: Test the LangSmith Control Plane API deployment workflow using the official example code

## Overview

This experiment runs the example Python code from the LangSmith Control Plane API documentation to understand the basic deployment workflow via the API.

**Primary Documentation**:
- https://docs.langchain.com/langsmith/api-ref-control-plane
- https://api.host.langchain.com/docs (FastAPI interactive docs)

**Related Issues**:
- Parent: #171 (LangSmith Deployments support)
- Related: #160 feature branches (for context only)
- NOT related to: #170 (switching to external_docker)

## Experiment Scope

This is a **research-only** experiment with no production code changes:

✅ Test the basic deployment workflow from FastAPI docs
✅ Observe API behavior and responses
✅ Document findings and learnings
✅ Note any differences from expectations
❌ No changes to langstar CLI/SDK code (separate issue if needed)

## Setup

### Environment

- **Python**: 3.11.14
- **Location**: `/workspace/wip/codekiln-178-experiment-control-plane-api/experiments/`
- **Dependencies installed**:
  - `requests==2.32.5`
  - `python-dotenv==1.2.1`

### Source Code

**Example code source**: https://docs.langchain.com/langsmith/api-ref-control-plane
**Local file**: `experiments/test_deployment_workflow.py`

The example code is copied verbatim from the documentation with only minor comments added for context.

## Workflow Steps

The example code tests the 4-step deployment workflow:

### 1. POST /v2/deployments - Create Deployment

**Function**: `create_deployment()`

**Request**:
- Endpoint: `{CONTROL_PLANE_HOST}/v2/deployments`
- Method: POST
- Headers: `X-Api-Key`, `X-Tenant-Id`, `Content-Type`

**Request Body**:
```python
{
    "name": "my_deployment",
    "source": "github",
    "source_config": {
        "integration_id": INTEGRATION_ID,
        "repo_url": "https://github.com/langchain-ai/langgraph-example",
        "deployment_type": "dev",
        "build_on_push": False,
        "custom_url": None,
        "resource_spec": None,
    },
    "source_revision_config": {
        "repo_ref": "main",
        "langgraph_config_path": "langgraph.json",
        "image_uri": None,
    },
    "secrets": [
        {"name": "OPENAI_API_KEY", "value": "test_openai_api_key"},
        {"name": "ANTHROPIC_API_KEY", "value": "test_anthropic_api_key"},
        {"name": "TAVILY_API_KEY", "value": "test_tavily_api_key"},
    ],
}
```

**Expected Response**: Status 201, returns `id` (deployment ID) and `latest_revision_id`

### 2. GET /v2/deployments/{deployment_id} - Retrieve Deployment

**Function**: `get_deployment(deployment_id)`

**Request**:
- Endpoint: `{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}`
- Method: GET
- Headers: `X-Api-Key`, `X-Tenant-Id`

**Expected Response**: Status 200, returns full deployment object

**Additional Function**: `list_revisions(deployment_id)` - Lists all revisions for the deployment

### 3. Poll GET /v2/deployments/{deployment_id}/revisions/{revision_id}

**Function**: `wait_for_deployment(deployment_id, revision_id)`

**Polling Strategy**:
- Interval: 60 seconds
- Timeout: 1800 seconds (30 minutes)
- Success: status == "DEPLOYED"
- Failure: "FAILED" in status

**Request**:
- Endpoint: `{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}/revisions/{revision_id}`
- Method: GET
- Headers: `X-Api-Key`, `X-Tenant-Id`

**Expected Response**: Status 200, returns revision object with `status` field

**Possible Status Values**:
- CREATING, QUEUED, AWAITING_BUILD, BUILDING
- AWAITING_DEPLOY, DEPLOYING, DEPLOYED
- CREATE_FAILED, BUILD_FAILED, DEPLOY_FAILED
- SKIPPED, INTERRUPTED, UNKNOWN

### 4. PATCH /v2/deployments/{deployment_id} - Update Deployment

**Function**: `patch_deployment(deployment_id)`

**Request**:
- Endpoint: `{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}`
- Method: PATCH
- Headers: `X-Api-Key`, `X-Tenant-Id`, `Content-Type`

**Request Body**:
```python
{
    "source_config": {
        "build_on_push": True,
    },
    "source_revision_config": {
        "repo_ref": "main",
        "langgraph_config_path": "langgraph.json",
    },
}
```

**Note**: Including `source_revision_config` creates a new revision

**Expected Response**: Status 200, returns updated deployment with new `latest_revision_id`

### 5. DELETE /v2/deployments/{deployment_id} - Cleanup

**Function**: `delete_deployment(deployment_id)`

**Request**:
- Endpoint: `{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}`
- Method: DELETE
- Headers: `X-Api-Key`, `X-Tenant-Id`

**Expected Response**: Status 204 (No Content)

## Execution Flow

The example code orchestrates a complete lifecycle:

```python
# 1. Create deployment and get first revision
deployment_id = create_deployment()
revisions = list_revisions(deployment_id)
latest_revision_id = revisions["resources"][0]["id"]

# 2. Wait for first revision to deploy
wait_for_deployment(deployment_id, latest_revision_id)

# 3. Update deployment (creates second revision)
patch_deployment(deployment_id)
revisions = list_revisions(deployment_id)
latest_revision_id = revisions["resources"][0]["id"]

# 4. Wait for second revision to deploy
wait_for_deployment(deployment_id, latest_revision_id)

# 5. Cleanup
delete_deployment(deployment_id)
```

## Test Results

### Execution Attempt

**Status**: IN PROGRESS

**Command**:
```bash
cd experiments
./run_test.sh
```

### Issues Encountered and Resolved

1. **Repository API Response Format**
   - Issue: Script expected `full_name` field but API returns separate `owner` and `name` fields
   - Fix: Updated repository matching to use `owner` and `name` fields separately
   - Line: experiments/test_deployment_workflow.py:107-110

2. **Missing Required Field**
   - Issue: API requires `secrets` field in deployment creation request
   - Fix: Added empty `secrets` array to request body
   - Note: Secrets not needed for testing deployment workflow itself
   - Line: experiments/test_deployment_workflow.py:149

3. **Deployment Name Validation**
   - Issue: Deployment names must be alphanumeric with dashes, starting with letter
   - Fix: Changed from underscore to dash separator (langstar-test-deployment)
   - Line: experiments/test_deployment_workflow.py:133

4. **Duplicate Deployment Name**
   - Issue: Deployment creates LangSmith project with same name, must be unique
   - Fix: Added timestamp to deployment name for uniqueness
   - Current format: `{REPOSITORY_NAME}-test-{timestamp}`
   - Line: experiments/test_deployment_workflow.py:132-133

### Script Enhancements Made

1. **Dynamic Integration Discovery**
   - Successfully queries `/v1/integrations/github/install` to list integrations
   - Successfully queries `/v1/integrations/github/{id}/repos` to find codekiln/langstar
   - Found integration ID: 5275541c-f05b-4274-8b92-7077179c0302

2. **Environment Variable Support**
   - Uses `LANGSMITH_API_KEY` from environment
   - Uses `LANGCHAIN_WORKSPACE_ID` (fallback to `LANGSMITH_WORKSPACE_ID`)
   - Uses `REPOSITORY_OWNER` (default: codekiln) and `REPOSITORY_NAME` (default: langstar)
   - Uses `CONTROL_PLANE_HOST` (default: https://api.host.langchain.com)

3. **Test Configuration**
   - Repository: codekiln/langstar
   - Config path: tests/fixtures/test-graph-deployment/langgraph.json
   - Source: github
   - Deployment type: dev

### Current Status

**Test is running** - Waiting for deployment to reach DEPLOYED status.

The workflow polls `/v2/deployments/{id}/revisions/{revision_id}` every 60 seconds until status is DEPLOYED. This typically takes 3-5 minutes for initial deployment creation.

### Observations

[To be updated when test completes]

### API Behavior

[To be filled with observed request/response patterns]

### Timing

[To be filled with observed deployment timing]

### Differences from Expectations

[To be filled with any unexpected behavior]

## Key Findings

[To be summarized after experiment completion]

## Questions Raised

[To be filled with any questions that arise during testing]

## Recommendations

[To be filled with suggestions for langstar implementation]

## Related Code

### Example Helper Script

The langchain-ai/cicd-pipeline-example repository contains a similar helper script:
- Location: `/workspace/reference/repo/langchain-ai/cicd-pipeline-example/code/.github/scripts/langgraph_api.py`
- Notable differences from documentation example:
  - Uses `external_docker` source type
  - Different API endpoint structure
  - Additional deployment management functions

## References

- [Control Plane API Documentation](https://docs.langchain.com/langsmith/api-ref-control-plane)
- [FastAPI Interactive Docs](https://api.host.langchain.com/docs)
- [OpenAPI Specification](https://api.host.langchain.com/openapi.json)
- [Example CI/CD Repository](https://github.com/langchain-ai/cicd-pipeline-example)
