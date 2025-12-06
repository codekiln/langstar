<!--
  SIZE LIMIT: This file SHOULD remain under 200 lines (currently ~235).
  Last checked: 2025-12-06
  If significantly exceeding limit, extract content to sub-document.
-->

# Debugging Tests

This document covers common test failures and how to debug them.

## Running Tests

### Run All Tests

```bash
cargo test --workspace --all-features
```

### Run Specific Test File

```bash
# SDK integration test
cargo test -p langstar-sdk --test prompts_test

# CLI integration test
cargo test -p langstar --test prompt_command_test
```

### Run Single Test

```bash
cargo test test_prompt_crud_lifecycle
```

### Run with Output Visible

```bash
cargo test test_name -- --nocapture
```

### Run Ignored Integration Tests

```bash
cargo test --features integration-tests -- --ignored
```

## Common Failure Patterns

### Pattern 1: Missing Environment Variables

**Symptom:**
```
LANGSMITH_API_KEY environment variable must be set
```

**Solution:**
```bash
# Set required variables
export LANGSMITH_API_KEY=<your-api-key>
export LANGSMITH_WORKSPACE_ID=<your-workspace-id>
export LANGSMITH_ORGANIZATION_ID=<your-org-id>
```

**In CI:** These are set as repository secrets. Check `.github/workflows/`.

### Pattern 2: Test Skipped (No Output)

**Symptom:** Test shows as passing but didn't run

**Cause:** Test has graceful skip for missing env vars:
```rust
let org_id = match get_org_id_or_skip() {
    Some(id) => id,
    None => return,  // Silently skips
};
```

**Solution:** Set the required environment variables and re-run.

### Pattern 3: Authentication Errors

**Symptom:**
```
Error: AuthenticationError: Invalid API key
```

**Causes:**
1. Expired or revoked API key
2. Key lacks required permissions
3. Wrong workspace/org ID

**Solution:**
1. Verify key at https://smith.langchain.com/settings
2. Check workspace ID matches the key's permissions
3. Generate new key if expired

### Pattern 4: Resource Not Found

**Symptom:**
```
Error: NotFoundError: Prompt 'test-prompt' not found
```

**Causes:**
1. Previous test didn't clean up
2. Test ran against wrong workspace
3. Resource was deleted externally

**Solution:**
1. Ensure tests use unique names with timestamps
2. Verify `LANGSMITH_WORKSPACE_ID` is correct
3. Clean up orphaned test resources

### Pattern 5: Rate Limiting

**Symptom:**
```
Error: RateLimited: Too many requests
```

**Solution:**
1. Wait and retry
2. Run fewer tests in parallel: `cargo test -- --test-threads=1`
3. Add delays between tests if needed

### Pattern 6: CI Passes, Local Fails

**Causes:**
1. Different environment variables
2. Different Rust version
3. Cached test data

**Debug steps:**
```bash
# Match CI Rust version
rustup default stable

# Clear caches
cargo clean

# Run same commands as CI
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace --all-features
```

### Pattern 7: Local Passes, CI Fails

**Causes:**
1. Hard-coded local paths
2. Local test data not available in CI
3. Timing-dependent tests

**Debug steps:**
1. Check for hard-coded paths
2. Verify test creates its own data
3. Add explicit waits for async operations

## Debugging Techniques

### View Full Test Output

```bash
cargo test test_name -- --nocapture 2>&1 | tee test.log
```

### Print Debug Info in Tests

```rust
#[test]
fn test_something() {
    println!("Debug: org_id = {}", org_id);
    eprintln!("stderr: response = {:?}", response);
    // ... test code
}
```

### Run with RUST_BACKTRACE

```bash
RUST_BACKTRACE=1 cargo test test_name
```

### Inspect API Responses

```rust
let output = cmd.output().expect("Failed to run");
println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
println!("status: {}", output.status);
```

## Integration Test Prerequisites Checklist

Before running integration tests:

- [ ] `LANGSMITH_API_KEY` is set and valid
- [ ] `LANGSMITH_WORKSPACE_ID` is set and matches your key
- [ ] `LANGSMITH_ORGANIZATION_ID` is set (for org-scoped tests)
- [ ] API key has write permissions (for create/delete tests)
- [ ] Network access to `api.smith.langchain.com`
- [ ] Not rate limited (wait if recent heavy usage)

## Test Output Interpretation

### Success Indicators

```
test test_name ... ok
```

### Failure with Details

```
test test_name ... FAILED

---- test_name stdout ----
thread 'test_name' panicked at 'assertion failed: found
    left: `0`,
   right: `5`', tests/prompt_test.rs:123
```

### Ignored (Skipped)

```
test test_name ... ignored
```

This means the test has `#[ignore]` attribute or was conditionally skipped.

## Related Documentation

- **Test Fixtures:** `test-fixtures.md` - Environment setup
- **CLI Tests:** `cli-integration-tests.md` - CLI test patterns
- **SDK Tests:** `sdk-integration-tests.md` - SDK test patterns
