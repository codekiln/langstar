# Implementation Plan: Test Deployment Naming Consolidation

**Issue:** #524
**Date:** 2025-12-04
**Research:** `reference/research/524-integration-test-deployment-consolidation.md`

---

## Goal

Consolidate from 8 test deployment naming patterns to exactly **two types**:

| Type | Pattern | Behavior |
|------|---------|----------|
| **PR/Dev** | `pr-integration-test-{timestamp}` | Get-or-create by prefix, cleaned by cron |
| **Release** | `release-integration-test-{timestamp}` | Always fresh, self-deleting |

---

## Implementation Steps

### Step 1: Update `sdk/src/test_utils.rs`

**1.1 Add constants for prefixes (after line 35):**
```rust
/// Prefix for PR/dev test deployments (reusable via get-or-create)
pub const PR_TEST_DEPLOYMENT_PREFIX: &str = "pr-integration-test-";

/// Prefix for release lifecycle test deployments (create fresh, delete after)
pub const RELEASE_TEST_DEPLOYMENT_PREFIX: &str = "release-integration-test-";
```

**1.2 Add `name_prefix` field to `TestDeploymentConfig` (line 39-49):**
```rust
pub struct TestDeploymentConfig {
    /// Name of the deployment
    pub name: String,
    /// Optional prefix for get-or-create lookup (None = always create fresh)
    pub name_prefix: Option<String>,
    /// Repository owner (e.g., "codekiln")
    pub repository_owner: String,
    /// Repository name (e.g., "langstar")
    pub repository_name: String,
    /// Branch to deploy from (default: "main")
    pub branch: String,
    /// Path to langgraph.json config file
    pub config_path: String,
}
```

**1.3 Update `Default` impl (line 52-67):**
```rust
impl Default for TestDeploymentConfig {
    fn default() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            name: format!("{}{}", PR_TEST_DEPLOYMENT_PREFIX, timestamp),
            name_prefix: Some(PR_TEST_DEPLOYMENT_PREFIX.to_string()),
            repository_owner: std::env::var("REPOSITORY_OWNER")
                .unwrap_or_else(|_| "codekiln".to_string()),
            repository_name: std::env::var("REPOSITORY_NAME")
                .unwrap_or_else(|_| "langstar".to_string()),
            branch: "main".to_string(),
            config_path: "tests/fixtures/test-graph-deployment/langgraph.json".to_string(),
        }
    }
}
```

**1.4 Update `for_release_tests()` (line 70-87):**
```rust
pub fn for_release_tests() -> Self {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    Self {
        name: format!("{}{}", RELEASE_TEST_DEPLOYMENT_PREFIX, timestamp),
        name_prefix: None,  // No prefix search - always create fresh
        ..Default::default()
    }
}
```

**1.5 Update `get_or_create_deployment()` (around line 305-320):**
```rust
// Step 2: Look for existing deployment by prefix or name
let search_pattern = config.name_prefix.as_ref()
    .map(|p| p.to_string())
    .unwrap_or_else(|| config.name.clone());

let filters = DeploymentFilters {
    name_contains: Some(search_pattern.clone()),
    ..Default::default()
};
let deployments = client
    .deployments()
    .list(Some(100), None, Some(filters))
    .await?;

// Find deployment matching prefix (for reuse) or exact name
let deployment = if config.name_prefix.is_some() {
    // Prefix-based search: find ANY matching deployment for reuse
    if let Some(existing) = deployments.resources.iter()
        .find(|d| d.name.starts_with(&search_pattern))
    {
        eprintln!(
            "Found existing deployment: {} ({})",
            existing.name, existing.id
        );
        existing.clone()
    } else {
        // Create new with the generated name
        create_new_deployment(client, config, &integration_id).await?
    }
} else {
    // No prefix: always create fresh
    create_new_deployment(client, config, &integration_id).await?
};
```

---

### Step 2: Update `sdk/tests/integration_deployment_workflow.rs`

**2.1 Import shared utilities (add near top):**
```rust
use langstar_sdk::test_utils::{TestDeploymentConfig, get_or_create_deployment};
```

**2.2 Update `test_deployment_workflow()` (line 87-299):**

Replace lines 122-143:
```rust
// BEFORE
let deployment_name = "langstar-integration-test".to_string();
let create_request = CreateDeploymentRequest { ... };

// AFTER
let config = TestDeploymentConfig::default();
eprintln!("📦 Using shared test deployment config");
eprintln!("   Name: {}", config.name);
eprintln!("   Prefix: {:?}", config.name_prefix);

let (deployment_id, revision_id) = get_or_create_deployment(&client, &config).await?;
```

**2.3 Update `test_deployment_workflow_full_lifecycle()` (line 337-535):**

Replace lines 372-397:
```rust
// BEFORE
let deployment_name = format!("{}-test-{}", repository_name, timestamp);
let create_request = CreateDeploymentRequest { ... };

// AFTER
let config = TestDeploymentConfig::for_release_tests();
eprintln!("📦 Creating fresh release test deployment");
eprintln!("   Name: {}", config.name);

let (deployment_id, revision_id) = get_or_create_deployment(&client, &config).await?;
```

---

### Step 3: Update `cli/tests/graph_command_test.rs`

**3.1 Add import (if not already present):**
```rust
use langstar_sdk::test_utils::TestDeploymentConfig;
```

**3.2 Update `test_graph_create_basic()` (line 297):**
```rust
// BEFORE
let deployment_name = format!("cli-test-deployment-{}", timestamp);

// AFTER
let config = TestDeploymentConfig::for_release_tests();
let deployment_name = config.name.clone();
```

**3.3 Update `test_graph_create_with_wait()` (line 354):**
```rust
// BEFORE
let deployment_name = format!("cli-test-deployment-wait-{}", timestamp);

// AFTER
let config = TestDeploymentConfig::for_release_tests();
let deployment_name = config.name.clone();
```

**3.4 Update `test_graph_lifecycle()` (line 426):**
```rust
// BEFORE
let deployment_name = format!("cli-test-lifecycle-{}", timestamp);

// AFTER
let config = TestDeploymentConfig::for_release_tests();
let deployment_name = config.name.clone();
```

**3.5 Update `test_graph_create_with_env_secrets()` (line 614):**
```rust
// BEFORE
let deployment_name = format!("cli-test-deployment-env-{}", timestamp);

// AFTER
let config = TestDeploymentConfig::for_release_tests();
let deployment_name = config.name.clone();
```

---

### Step 4: Simplify Cleanup Workflow

**File:** `.github/workflows/cleanup-test-deployments.yml`

**Update pattern matching (lines 67-76):**
```yaml
# BEFORE - multiple patterns needed
for pattern in "integration-test" "langstar-test" "cli-test"; do
  echo "  Checking pattern: $pattern"
  if pattern_result=$(./target/release/langstar graph list \
    --name-contains "$pattern" \
    --format json 2>/dev/null); then
    # Merge results
  fi
done

# AFTER - single pattern catches both types
echo "🔍 Searching for test deployments..."
if ! raw_deployments=$(./target/release/langstar graph list \
  --name-contains "integration-test" \
  --format json); then
  echo "❌ Error: Failed to list deployments"
  exit 1
fi
deployments="$raw_deployments"
```

---

### Step 5: Update Documentation

**5.1 Update `sdk/tests/README.md` (lines 16-22):**
```markdown
### Test Deployment Naming

Integration tests use exactly **two deployment types**:

- **`pr-integration-test-{timestamp}`** - Shared via get-or-create by prefix
  - Used by most tests (SDK and CLI)
  - Searches for existing `pr-integration-test-*` deployment
  - Reuses if found and ready, creates new if not
  - Cleaned up by periodic cron job (4-hour threshold)

- **`release-integration-test-{timestamp}`** - Full lifecycle tests
  - Used by `TestDeploymentConfig::for_release_tests()`
  - Always creates fresh deployment
  - Self-deleting after test completes
```

**5.2 Update `cli/tests/README.md` (fixtures section):**
Update to reference the two standard types and `TestDeploymentConfig`.

**5.3 Update `docs/research/i512-integration-test-get-or-create-status-v0.11.0.md`:**
Add note at top referencing this consolidation.

---

## Files Summary

### Files to Modify

| File | Changes |
|------|---------|
| `sdk/src/test_utils.rs` | Add prefixes, name_prefix field, update get-or-create |
| `sdk/tests/integration_deployment_workflow.rs` | Use TestDeploymentConfig |
| `cli/tests/graph_command_test.rs` | Use TestDeploymentConfig::for_release_tests() |
| `.github/workflows/cleanup-test-deployments.yml` | Simplify to single pattern |
| `sdk/tests/README.md` | Update naming docs |
| `cli/tests/README.md` | Update fixtures docs |

### Files Created (this PR)

| File | Purpose |
|------|---------|
| `reference/research/524-integration-test-deployment-consolidation.md` | Research findings |
| `docs/implementation/524-test-deployment-naming-consolidation.md` | This implementation plan |

---

## Testing

1. **Unit tests**: `cargo test --workspace --lib` - verify test_utils changes compile
2. **Integration tests**: Run in CI to verify deployments are created/reused correctly
3. **Cleanup verification**: Manually trigger cleanup workflow to verify pattern matching

---

## Migration Notes

- Existing `pr-integration-test` (without timestamp) deployments will NOT be found by prefix search
- First run after migration will create a new `pr-integration-test-{ts}` deployment
- Old deployment will be cleaned up by cron after 4 hours
- No breaking changes to test behavior, only naming standardization
