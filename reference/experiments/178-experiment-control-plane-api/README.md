# Experiment: LangSmith Control Plane API - Deployment Lifecycle Testing

**Date**: 2025-11-20
**Issue**: [#178 - Experiment: Test LangSmith Control Plane API example code](https://github.com/codekiln/langstar/issues/178)
**Parent Issue**: [#171 - LangSmith Deployments support](https://github.com/codekiln/langstar/issues/171)

## Objective

Test the complete CRUD (Create, Read, Update, Delete) lifecycle of LangSmith deployments via the Control Plane API to:

1. **Enable integration testing** - Validate that Langstar CLI/SDK can programmatically manage deployments for automated testing
2. **Support CLI operations** - Understand the API well enough to implement deployment management commands in Langstar
3. **Access deployed agents** - Determine if/how to access the Agent API (LangGraph deployment URL) after programmatically creating a deployment

## Key Finding: Agent API Access ✅

**DISCOVERY**: The deployment URL for accessing the deployed agent is returned in the `custom_url` field of the deployment response:

```json
{
  "source_config": {
    "custom_url": "https://<deployment-name>-<hash>.<region>.langgraph.app"
  }
}
```

This URL is the LangGraph Agent API endpoint that enables:
- ✅ Interaction with the deployed agent
- ✅ Running integration tests against the deployment
- ✅ Validation of deployment functionality

## Resources

### Documentation
- **Control Plane API Reference**: [LangSmith Deployment API Docs](https://docs.langchain.com/langsmith/api-ref-control-plane)
- **FastAPI Interactive Docs**: https://api.host.langchain.com/docs
- **OpenAPI Specification**: https://api.host.langchain.com/openapi.json
- **Example CI/CD Repository**: https://github.com/langchain-ai/cicd-pipeline-example

### Related Work
- **Issue #160**: Deployment create/delete functionality (initial implementation)
- **Issue #170**: Investigation of external_docker deployment type (NOT part of this experiment)
- **Test Fixture**: `tests/fixtures/test-graph-deployment/langgraph.json`

## Experiment Scope

This was a **research-only experiment** with no production code changes:

✅ Test the basic deployment workflow from official documentation
✅ Observe actual API behavior and response structures
✅ Document findings and implementation details
✅ Identify differences between documentation and reality
❌ No changes to langstar CLI/SDK code (those are separate issues)

**Important**: This experiment tested **GitHub source** deployments, NOT `external_docker` deployments.

## Test Script

Created `test_deployment_workflow.py` - Python implementation of the official documentation example with enhancements:

**Core Functions** (from documentation):
- `create_deployment()` - POST /v2/deployments
- `get_deployment(deployment_id)` - GET /v2/deployments/{id}
- `list_revisions(deployment_id)` - GET /v2/deployments/{id}/revisions
- `get_revision(deployment_id, revision_id)` - GET /v2/deployments/{id}/revisions/{revision_id}
- `patch_deployment(deployment_id)` - PATCH /v2/deployments/{id}
- `delete_deployment(deployment_id)` - DELETE /v2/deployments/{id}
- `wait_for_deployment()` - Poll until revision status reaches DEPLOYED

**Added Functions** (for investigation):
- `list_deployments(name_contains)` - GET /v2/deployments
- `list_github_integrations()` - GET /v1/integrations/github/install
- `list_github_repositories(integration_id)` - GET /v1/integrations/github/{id}/repos
- `find_integration_for_repo(owner, repo)` - Dynamically discover integration ID

**CLI Interface**:
```bash
./run_test.sh list                           # List all deployments
./run_test.sh list --name-contains <pattern> # Filter by name
./run_test.sh get <deployment-id>            # Get deployment details (JSON)
./run_test.sh get-latest-revision <id>       # Get latest revision (JSON)
./run_test.sh run-workflow                   # Run full test workflow
```

## Key Findings

### 1. Deployment Creation Requirements

**Required Fields**:
- `name` - Must be alphanumeric with dashes, starting with letter
- `source` - "github" or "external_docker"
- `source_config` - Integration ID, repo URL, deployment type
- `source_revision_config` - Repo ref, config path, or image URI
- `secrets` - **Required field** (can be empty array `[]`)

**Naming Restrictions**:
- ❌ Underscores not allowed
- ✅ Dashes allowed
- ⚠️ Must start with alphabetic character
- ⚠️ Creates matching LangSmith project (must be globally unique)

### 2. GitHub Integration Discovery

**Repository API Format**:
- Returns separate `owner` and `name` fields
- Does NOT return `full_name` field (contrary to expectations)

**Sample Response**:
```json
{
  "owner": "<repository-owner>",
  "name": "<repository-name>",
  "url": "https://github.com/<owner>/<name>",
  "default_branch": "main"
}
```

### 3. Deployment Status Lifecycle

**Deployment Statuses**: `AWAITING_DATABASE` → `READY`

**Revision Statuses**:
`CREATING` → `QUEUED` → `AWAITING_BUILD` → `BUILDING` → `AWAITING_DEPLOY` → `DEPLOYING` → `DEPLOYED`

**Failure States**: `CREATE_FAILED`, `BUILD_FAILED`, `DEPLOY_FAILED`

**Build Time**: ~14 minutes from creation to DEPLOYED (for test graph fixture)

### 4. Accessing the Agent API

**Deployment Response Structure**:
```json
{
  "id": "<deployment-uuid>",
  "name": "<deployment-name>",
  "status": "READY",
  "source_config": {
    "custom_url": "https://<deployment-name>-<hash>.<region>.langgraph.app",
    ...
  },
  "latest_revision_id": "<revision-uuid>",
  "active_revision_id": "<revision-uuid>"
}
```

**Implementation Pattern**:
1. Create deployment via POST
2. Poll until revision reaches DEPLOYED
3. GET deployment details
4. Extract `source_config.custom_url`
5. Use URL for agent API access

### 5. Issues Resolved During Testing

| Issue | Problem | Solution |
|-------|---------|----------|
| Repository field format | Expected `full_name` | Use separate `owner` and `name` fields |
| Missing required field | API rejected without `secrets` | Add `"secrets": []` to request |
| Name validation | Underscores rejected | Use dashes only: `langstar-test` |
| Duplicate names | Project already exists | Add timestamp: `langstar-test-{timestamp}` |

## Workflow Validation

The complete workflow was successfully tested:

1. ✅ **Create Deployment** - POST with GitHub source, dynamic integration discovery
2. ✅ **Poll for Deployment** - GET revision status every 60s until DEPLOYED (~14 min)
3. ✅ **Update Deployment** - PATCH creates new revision
4. ✅ **Poll for New Revision** - New revision successfully deploys
5. ✅ **Cleanup** - DELETE removes deployment and resources

## Recommendations for Langstar

### CLI Commands

```bash
langstar deployment create --name <name> --repo <owner/repo> --config-path <path>
langstar deployment list [--name-contains <pattern>]
langstar deployment get <deployment-id>
langstar deployment get-url <deployment-id>  # Returns agent API URL
langstar deployment delete <deployment-id>
langstar deployment wait <deployment-id>     # Poll until DEPLOYED
```

### Integration Testing Pattern

```rust
// 1. Create test deployment
let deployment = create_deployment("integration-test-{timestamp}").await?;

// 2. Wait for DEPLOYED
wait_for_deployed(deployment.id, deployment.latest_revision_id).await?;

// 3. Get agent URL
let details = get_deployment(deployment.id).await?;
let agent_url = details.source_config.custom_url;

// 4. Run tests against agent_url
run_integration_tests(&agent_url).await?;

// 5. Cleanup
delete_deployment(deployment.id).await?;
```

### Required Headers

All API calls must include:
```
X-Api-Key: <langsmith-api-key>
X-Tenant-Id: <workspace-id>
Content-Type: application/json
```

## Limitations

### Not Tested
- ❌ External Docker deployments
- ❌ Custom resource specifications
- ❌ Production deployment type
- ❌ Build/install commands
- ❌ Listener configurations

### Important Gotchas
- ⚠️ Deployment names create matching LangSmith projects (must be unique)
- ⚠️ Build times: 10-15 minutes for initial deployment
- ⚠️ `secrets` field required (use `[]` if no secrets needed)
- ⚠️ Repository API returns `owner`/`name` separately
- ⚠️ Integration ID must be discovered dynamically

## Conclusion

**Success**: Validated complete deployment lifecycle and discovered how to access deployed agent APIs.

**Key Achievement**: The `custom_url` field provides the LangGraph Agent API endpoint, enabling programmatic testing and interaction with deployed agents.

**Next Steps**:
1. Implement deployment management in Langstar CLI
2. Add integration tests using ephemeral deployments
3. Document Control Plane API integration in SDK

## Files

- `test_deployment_workflow.py` - Complete test script with CLI
- `run_test.sh` - Wrapper script with environment setup
- `../openapi/langchain/langsmith-deployment-control-plane-api-openapi.json` - API spec

## Usage

```bash
# List all deployments
./run_test.sh list

# Get deployment details
./run_test.sh get <deployment-id>

# Get latest revision
./run_test.sh get-latest-revision <deployment-id>

# Run full workflow
./run_test.sh run-workflow
```

## References

- [GitHub Issue #178](https://github.com/codekiln/langstar/issues/178)
- [GitHub Issue #171](https://github.com/codekiln/langstar/issues/171)
- [Control Plane API Docs](https://docs.langchain.com/langsmith/api-ref-control-plane)
- [FastAPI Docs](https://api.host.langchain.com/docs)
- [OpenAPI Spec](https://api.host.langchain.com/openapi.json)
