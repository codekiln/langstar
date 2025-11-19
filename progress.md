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

### 🧪 Next Steps for Verification

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
