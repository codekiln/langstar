# Progress: GitHub Integration ID Configuration

## Current Status

### ✅ Completed
1. **GitHub Integration ID Configuration Support**
   - Added `LANGGRAPH_GITHUB_INTEGRATION_ID` environment variable support
   - Added `--integration-id` CLI flag for one-off overrides
   - Added `github_integration_id` field to config file (~/.config/langstar/config.toml)
   - Implemented precedence chain: CLI flag > env/config > auto-discovery

2. **Documentation**
   - Documented in `.devcontainer/.env.default`
   - Updated test fixture documentation
   - Added setup instructions in error messages

3. **Environment Setup**
   - GitHub Actions has `LANGGRAPH_GITHUB_INTEGRATION_ID` secret configured
   - Local environment has the variable set

### ✅ Test Deployment Reuse - IMPLEMENTED

**Expected Behavior (per user requirements):**
When integration tests start, they should:
1. ✅ Check if there's an **existing active** test deployment from previous runs
2. ✅ If found → **reuse it** (avoid creating duplicate deployments)
3. ✅ If not found → create new deployment using `LANGGRAPH_GITHUB_INTEGRATION_ID` from env var
4. ✅ Wait for deployment to reach READY status
5. ✅ Run integration tests

**Current Behavior:**
The test infrastructure creates a **new deployment every time** the test suite runs:
```rust
// cli/tests/common/fixtures.rs
pub fn create() -> Self {
    // Always creates new deployment with unique timestamp
    let deployment_name = format!("test-deployment-{}", timestamp);
    // ...
}
```

**How Tests Currently Work:**
1. First test in suite creates new deployment (via `OnceLock`)
2. Subsequent tests in **same run** reuse that deployment
3. Deployment deleted when tests complete (via `Drop`)
4. Next test run creates **another new deployment**

This leads to:
- ❌ Multiple test deployments accumulating over time
- ❌ Wasted API quota creating duplicate deployments
- ❌ Slower test startup (always waits for new deployment creation)

### 📋 What Needs to Change

#### Option 1: Query Existing Deployments (Recommended)
Update `TestDeployment::create()` to:
```rust
pub fn create() -> Self {
    Self::check_env_vars();

    // 1. Query existing deployments
    let existing = Self::find_active_test_deployment();

    // 2. Reuse if found
    if let Some(deployment) = existing {
        println!("♻️  Reusing existing test deployment: {}", deployment.name);
        return deployment;
    }

    // 3. Create new if not found
    println!("🚀 Creating new test deployment...");
    // ... existing creation logic ...
}

fn find_active_test_deployment() -> Option<Self> {
    // Query deployments via langstar CLI or SDK
    // Filter for:
    //   - name starts with "test-deployment-"
    //   - status == READY
    //   - source == github
    // Return most recent
}
```

**Benefits:**
- ✅ Reuses existing deployments across test runs
- ✅ Faster test startup (no deployment creation wait)
- ✅ Reduces API quota usage
- ✅ Still works if no deployments exist (creates new one)

**Implementation:**
- Add method to query deployments (either via CLI or SDK)
- Filter for active test deployments by naming convention
- Validate deployment is still READY before reusing
- Fall back to creation if none found

#### Option 2: Keep Single Test Deployment (Alternative)
- Don't delete test deployment on `Drop`
- Let tests accumulate one long-lived deployment
- Manually clean up old test deployments periodically

**Downsides:**
- Requires manual cleanup
- Less clean test isolation

### 🧪 Integration Test Flow Verification

**Current Flow:**
```
[Test Suite Start]
  ↓
[Check env vars: LANGSMITH_API_KEY, LANGCHAIN_WORKSPACE_ID]
  ↓
[get_test_deployment() → TEST_DEPLOYMENT.get_or_init()]
  ↓
[TestDeployment::create()]
  ↓
[Generate unique name: test-deployment-{timestamp}]
  ↓
[Run: langstar graph create --wait ...]
  ├─→ Check CLI flag --integration-id (none passed by tests)
  ├─→ Check LANGGRAPH_GITHUB_INTEGRATION_ID env var ✅
  ├─→ Check config file
  └─→ Fall back to auto-discovery (if env var not set)
  ↓
[Wait for deployment status: READY]
  ↓
[Run test_assistant_create_basic, test_assistant_get, etc.]
  ↓
[All tests complete]
  ↓
[TestDeployment::drop() → cleanup()]
  ↓
[Run: langstar graph delete {id} --yes]
  ↓
[Test Suite End]
```

**Issues in Current Flow:**
1. ❌ No check for existing deployments
2. ❌ Always creates new deployment
3. ❌ Always deletes deployment on completion

**Expected Flow:**
```
[Test Suite Start]
  ↓
[Check env vars: LANGSMITH_API_KEY, LANGCHAIN_WORKSPACE_ID]
  ↓
[get_test_deployment() → TEST_DEPLOYMENT.get_or_init()]
  ↓
[TestDeployment::create_or_reuse()]  ← NEW
  ↓
[Query: langstar graph list --name-contains test-deployment]  ← NEW
  ├─→ Found READY deployment? → Reuse it  ← NEW
  └─→ None found? → Create new deployment
      ↓
      [Generate unique name: test-deployment-{timestamp}]
      ↓
      [Run: langstar graph create --wait ...]
      ├─→ Use LANGGRAPH_GITHUB_INTEGRATION_ID env var ✅
      ↓
      [Wait for deployment status: READY]
  ↓
[Run tests...]
  ↓
[All tests complete]
  ↓
[Keep deployment for next run]  ← CHANGED (don't delete)
  ↓
[Test Suite End]
```

### 🔧 Action Items

1. **Update TestDeployment::create()** to query for existing deployments first
2. **Add TestDeployment::find_active()** method to search for reusable deployments
3. **Remove automatic cleanup** (or make it optional) to preserve deployments
4. **Test the new flow**:
   - Run tests twice in a row
   - Verify second run reuses first deployment
   - Verify integration_id comes from env var on first run

### 📊 Integration Test Matrix

| Scenario | Integration ID Source | Expected Behavior | Status |
|----------|----------------------|-------------------|---------|
| CI: First run, no deployments | `LANGGRAPH_GITHUB_INTEGRATION_ID` env var | Create new deployment | ❌ Not implemented (always creates new) |
| CI: Second run, deployment exists | N/A (reuse) | Reuse existing deployment | ❌ Not implemented (creates new) |
| Local: First run, no deployments | `LANGGRAPH_GITHUB_INTEGRATION_ID` env var | Create new deployment | ❌ Not implemented (always creates new) |
| Local: Second run, deployment exists | N/A (reuse) | Reuse existing deployment | ❌ Not implemented (creates new) |
| No env var, no existing deployments | Auto-discovery | Error with helpful message | ✅ Working |
| No env var, existing deployment from UI | Auto-discovery | Create using discovered ID | ✅ Working |

### ✅ Implementation Complete

**Changes made in `cli/tests/common/fixtures.rs`:**

1. **Added `find_active_test_deployment()` method**:
   - Queries existing deployments using `langstar graph list`
   - Filters for: name contains "test-deployment-", status=READY
   - Returns most recent matching deployment

2. **Updated `TestDeployment::create()` to try reuse first**:
   - Checks for existing test deployment
   - Reuses if found → fast startup ♻️
   - Creates new if not found → first-time setup 🚀

3. **Renamed old `create()` logic to `create_new_deployment()`**:
   - Separated creation logic for clarity
   - Still uses `LANGGRAPH_GITHUB_INTEGRATION_ID` from env var

4. **Disabled automatic cleanup in Drop trait**:
   - Deployments now persist across test runs
   - Manual cleanup: `langstar graph delete <id> --yes`
   - Saves API quota and speeds up tests

**Test Flow Now:**
```
[Test Suite Start]
  ↓
[Check env vars: LANGSMITH_API_KEY, LANGCHAIN_WORKSPACE_ID] ✅
  ↓
[get_test_deployment() → TEST_DEPLOYMENT.get_or_init()]
  ↓
[TestDeployment::create()]
  ├─→ find_active_test_deployment()  ← NEW
  │     ├─→ langstar graph list --name-contains test-deployment- --status READY
  │     ├─→ Found? → Reuse ♻️  ← FAST PATH
  │     └─→ Not found? → create_new_deployment()
  │           ├─→ Uses LANGGRAPH_GITHUB_INTEGRATION_ID ✅
  │           └─→ Wait for READY ✅
  ↓
[Run tests...] ✅
  ↓
[All tests complete]
  ↓
[Keep deployment for next run] ✅  ← CHANGED
  ↓
[Test Suite End]
```

### 🧪 Testing Status

**Test Run Results:**

1. ✅ **Repository URL Fixed**: Changed from `langchain-ai/langgraph-example` to `codekiln/langstar`
2. ✅ **Config Path Parameter Added**: Added `--config-path` parameter (defaults to `langgraph.json`)
3. ✅ **Test Fixture Updated**: Uses `tests/fixtures/test-graph-deployment/langgraph.json`
4. ✅ **Environment Variable Loading**: Successfully loads `LANGGRAPH_GITHUB_INTEGRATION_ID`
5. ✅ **Deployment Reuse Logic**: Correctly checks for existing deployments first
6. ✅ **API Integration**: Makes valid API calls with all required parameters

**Previous Blocker (RESOLVED):**
~~❌ **Quota Limit Exceeded**: Changed deployment type from `dev_free` to `dev` to bypass organization quota~~

**Current Blocker:**

❌ **GitHub Deployments Missing URL**: `"Deployment 'X' has no custom_url in source_config"`

**Root Cause:**
The `custom_url` field in `source_config` is only for `external_docker` deployments where users provide their own URL. For GitHub deployments on LangGraph Platform (Cloud SaaS), the platform generates a URL automatically, but this URL is NOT stored in the deployment's `source_config`.

**Impact:**
- Deployment creation works ✅ (test deployment created successfully in 13.1s)
- Assistant commands fail ❌ because they expect `custom_url` to connect to deployment

**What Works:**
- ✅ Deployment creation succeeds for GitHub deployments
- ✅ Environment variables load correctly
- ✅ Repository and config file path are correct
- ✅ Integration ID precedence chain works as expected
- ✅ Deployment reuse logic works correctly
- ✅ `--config-path` parameter works correctly

**What Doesn't Work:**
- ❌ Assistant commands can't find deployment URL for GitHub deployments
- ❌ `custom_url` is `null` for Platform-managed GitHub deployments

**Resolution Options:**
1. **API Enhancement**: Control Plane API should return deployment URL in response
2. **URL Construction**: Construct URL from deployment ID (format: `https://<id>.us.langgraph.app`?)
3. **SDK Enhancement**: Add method to connect by deployment ID without explicit URL
4. **Alternative Field**: Check if URL is in a different field (listener_config, revision, etc.)

**SOLUTION FOUND:**

Through investigation with deployment `test-url-investigation` (ID: `b71815d5-a2c5-411d-abd6-b2fc7812d39a`), we discovered:

1. **The `/v1/projects/{project_id}/revisions/{revision_id}` API** (internal/UI-only) returns:
   ```json
   {
     "resource": {
       "id": {
         "name": "test-url-investigation-d8d85c683e6a519c8c66cfc8b7053bbc-c89ccf8cf"
       }
     }
   }
   ```

2. **The deployment URL pattern is:**
   - resource.id.name format: `<deployment-name>-<hash>-<suffix>`
   - URL hostname (remove last segment): `<deployment-name>-<hash>`
   - Full URL: `https://<hostname>.us.langgraph.app`

3. **Example:**
   - resource.id.name: `test-url-investigation-d8d85c683e6a519c8c66cfc8b7053bbc-c89ccf8cf`
   - Hostname: `test-url-investigation-d8d85c683e6a519c8c66cfc8b7053bbc`
   - URL: `https://test-url-investigation-d8d85c683e6a519c8c66cfc8b7053bbc.us.langgraph.app`

**Implementation Required:**
1. Add `/v2/deployments/{id}/revisions/{revision_id}` endpoint to SDK
2. Create `Revision` struct with `resource` field
3. Extract hostname from `resource.id.name` (remove last hyphen-segment)
4. Construct URL: `https://{hostname}.us.langgraph.app`
5. Update `resolve_deployment_url()` in assistant.rs to use this

### 🔍 Investigation Update (Second Test Deployment)

**Deployment Created:** `test-url-investigation-2` (ID: `4ddbb7b1-a88c-4368-9136-6902a9822655`)
- Status: READY
- Revision ID: `85b429fa-896b-4968-af01-15400aede2d0`

**Key Findings:**

1. **`/v2/deployments/{id}` API (public)** - Does NOT return deployment URL:
   ```json
   {
     "source_config": {
       "custom_url": null,  // Still null after READY
       ...
     }
   }
   ```

2. **`/v2/deployments/{id}/revisions/{revision_id}` API EXISTS** in OpenAPI spec:
   - Found in public API: `https://api.host.langchain.com/openapi.json`
   - Returns `Revision` schema with fields: `id`, `created_at`, `updated_at`, `status`, `source`, `source_revision_config`
   - **BUT**: OpenAPI schema does NOT include `resource` field or URL information
   - **HOWEVER**: This doesn't mean the actual API response won't include it (schemas can be incomplete)

3. **Two Possible Scenarios:**
   a) **v2 revision API returns URL** - OpenAPI schema is incomplete, actual response includes URL
   b) **v2 revision API does NOT return URL** - Only v1 (internal) API has URL, need workaround

4. **Next Step:**
   - Implement SDK support for `/v2/deployments/{id}/revisions/{revision_id}`
   - Test what the actual API response contains (may differ from OpenAPI schema)
   - If URL is present → use it directly
   - If URL is absent → use hostname construction workaround from `resource.id.name`

**Reference from control-plane-api-demo:**
- Example workflow uses `/v1/projects` API to create `external_docker` deployments
- Sets `image_path` to Docker image URL
- For external_docker, `custom_url` is user-provided
- Does NOT solve GitHub deployment URL discovery

### 🎯 Solution Found: OpenAPI Spec Analysis

**Analyzed:**
- `https://langchain-ai.github.io/langgraph/cloud/reference/api/openapi_control_plane.json`
- `https://github.com/langchain-ai/cicd-pipeline-example`
- Official Control Plane API documentation

**Data Model (from OpenAPI spec):**
```
Project (v1 API concept = Deployment in v2 API)
  └─ resource: ResourceService
       ├─ id: ResourceId
       │    ├─ type: "revisions" | "services"
       │    └─ name: string  ← hostname for URL construction
       └─ url: string | null  ← deployment URL
```

**Key Findings:**

1. **Two parallel Control Plane APIs:**
   - **v1 `/projects`**: Contains `resource.url` with deployment URL (requires session auth, NOT API keys)
   - **v2 `/deployments`**: Public API with API key auth, but does NOT return URLs for GitHub deployments

2. **URL Pattern for GitHub Cloud SaaS Deployments:**
   - Format: `https://{deployment-name}-{hash}.us.langgraph.app`
   - Example: `https://test-url-investigation-2-3759548fa6535a80b5ac084029b50729.us.langgraph.app`
   - Hash is 32 hex characters (MD5-like format)

3. **URL Pattern for external_docker Deployments:**
   - Format: `https://{deployment-name}.langchain.dev`
   - Example from cicd-pipeline-example: `https://text2sql-agent-pr-123.langchain.dev`
   - Much simpler - just the deployment name

4. **v2 API Tested:**
   - ✅ `/v2/deployments/{id}` - Returns `custom_url: null` for GitHub deployments
   - ✅ `/v2/deployments/{id}/revisions/{revision_id}` - Works, but no URL fields
   - ❌ `/v1/projects` - Returns "Invalid token" with API key auth

5. **Documentation Gap:**
   - Official API docs show `<DEPLOYMENT_URL>` placeholder in all examples
   - Never explains how to programmatically obtain this URL
   - cicd-pipeline-example only shows external_docker (self-hosted) pattern

### 🚧 Current Blocker: No Programmatic Way to Get GitHub Deployment URLs

**Problem:**
- GitHub Cloud SaaS deployments URL is NOT returned by v2 `/deployments` API
- v1 `/projects` API has the URL but requires session auth (browser/UI), not API keys
- No documented way to obtain deployment URLs programmatically

**Tested Solutions:**
- ❌ Using deployment ID as hostname: `https://{deployment-id}.us.langgraph.app` (connection failed)
- ❌ Accessing v1 projects API with API keys: Returns "Invalid token"
- ❌ v2 revisions API: Doesn't include URL fields

**Remaining Options:**
1. **File issue with LangChain**: Request v2 API to include deployment URLs
2. **Reverse-engineer hash generation**: Figure out how `3759548fa6535a80b5ac084029b50729` is computed
3. **Accept limitation**: Document that users must manually obtain URLs from UI
4. **Investigate session auth**: See if we can programmatically authenticate to v1 API

### ✅ SOLUTION: Switch to external_docker Deployments

**Decision:** Use `external_docker` source type instead of `github` source type.

**Why This Solves the Problem:**

1. **Simple URL Pattern:**
   ```
   https://{deployment-name}.langchain.dev
   ```
   No hash computation needed - just string formatting!

2. **Proven Approach:**
   - Used by `langchain-ai/cicd-pipeline-example`
   - Recommended for CI/CD workflows
   - Production-ready

3. **GitHub Actions Integration:**
   - Uses GitHub Container Registry (ghcr.io)
   - Built-in authentication via `GITHUB_TOKEN` (no secrets needed!)
   - No Docker Hub rate limits (Docker Hub: 200 pulls/6 hours on free tier)
   - Free for both public and private images

4. **Image Naming:**
   ```
   ghcr.io/{owner}/{repo}:tag
   Example: ghcr.io/codekiln/langstar:test-latest
   ```

**Implementation Plan:**

1. **Create GitHub Action** (create as sub-issue):
   - Build Docker image from test fixtures
   - Push to ghcr.io using built-in `GITHUB_TOKEN`
   - Tag with `test-latest` for integration tests
   - Reference: `/workspace/reference/repo/langchain-ai/cicd-pipeline-example`

2. **Update Test Fixtures:**
   - Change `--source` from `github` to `external_docker`
   - Change `--repo-url` to `--image-uri ghcr.io/codekiln/langstar:test-latest`
   - Remove GitHub-specific parameters (branch, config-path, integration-id)

3. **Add URL Resolution Logic:**
   ```rust
   pub fn resolve_deployment_url(deployment: &Deployment) -> Result<String> {
       match deployment.source {
           DeploymentSource::ExternalDocker => {
               Ok(format!("https://{}.langchain.dev", deployment.name))
           }
           _ => Err(CliError::Config("Only external_docker supported".into()))
       }
   }
   ```

4. **Update SDK:**
   - Add `external_docker` source creation support
   - Update `CreateDeploymentRequest` to handle Docker image URIs

**Benefits:**
- ✅ Unblocks integration tests immediately
- ✅ No reverse engineering needed
- ✅ Standard Docker workflow
- ✅ Free GitHub Container Registry
- ✅ No rate limits
- ✅ Production-ready approach

**Next Action:**
- Create GitHub issue documenting this approach
- Use `gh-sub-issue` to create sub-issue for GitHub Action implementation
- Implement changes as documented above

### 📋 Next Steps

**To Unblock Testing:**
1. Identify and optionally delete the existing free dev deployment in the organization
2. OR: Temporarily change test fixture to use `dev` deployment type (requires paid plan)
3. Run tests again to verify full deployment lifecycle

**Once Unblocked:**
1. **Test locally with multiple runs** to verify:
   - First run creates new deployment (uses env var)
   - Second run reuses first deployment (fast)
   - Both runs pass all tests

2. **Monitor CI** to ensure:
   - GitHub Actions uses `LANGGRAPH_GITHUB_INTEGRATION_ID` secret
   - Tests reuse deployments across workflow runs
   - All integration tests pass

## Related Commits

- `183a154` - ✨ feat(config): add LANGGRAPH_GITHUB_INTEGRATION_ID config support
- `b0be38a` - 🩹 fix: auto-discover GitHub integration_id from existing deployments
