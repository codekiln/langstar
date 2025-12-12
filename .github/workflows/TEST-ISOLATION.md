# DevContainer Feature Test Isolation

## Overview

This document describes the test isolation strategy implemented in the `test-features.yml` workflow to ensure that each feature test runs in a completely isolated environment with no shared state.

## Isolation Guarantees

### 1. Unique Temporary Directories

Each feature test creates a unique temporary directory using `mktemp -d`:

```bash
TEST_DIR=$(mktemp -d -t "test-${feature}-XXXXXX")
```

**Benefits:**

- Prevents file system conflicts between tests
- Ensures each test has its own workspace
- Allows parallel test execution in the future
- Automatic cleanup by removing the directory

### 2. Fresh Container Instances

Each test runs in a completely fresh Docker container:

```bash
devcontainer up --workspace-folder "${TEST_DIR}"
```

The Dev Container CLI creates a new container for each workspace folder, ensuring:

- No shared environment variables from previous tests
- No cached dependencies or artifacts
- No leftover processes or services
- Fresh base image for each test

### 3. Explicit Container Cleanup

After each test (success or failure), containers are explicitly stopped and removed:

```bash
# Find containers associated with the test workspace
CONTAINER_NAME=$(docker ps -a --filter "label=devcontainer.local_folder=${TEST_DIR}" --format "{{.Names}}" | head -1)
if [ -n "$CONTAINER_NAME" ]; then
  docker rm -f "${CONTAINER_NAME}"
fi
```

**Why explicit cleanup is necessary:**

- Simply removing the workspace directory does NOT stop or remove containers
- Without cleanup, containers accumulate during test runs
- Stale containers can cause resource exhaustion
- Explicit cleanup ensures deterministic state between tests

### 4. No Shared Volumes

Each test uses only volumes created by the Dev Container CLI for that specific workspace:

- Volume names are derived from workspace paths
- Volumes are removed when containers are removed
- No persistent volumes are used across tests

### 5. Test Order Independence

Because each test:

- Uses a unique temporary directory
- Creates a fresh container
- Cleans up completely after execution

Tests can run in **any order** without affecting each other.

## Implementation Details

### Success Path Cleanup

When a test passes:

```bash
echo "✓ Feature ${feature} passed all tests"

# Clean up containers and test directory to ensure isolation
CONTAINER_NAME=$(docker ps -a --filter "label=devcontainer.local_folder=${TEST_DIR}" --format "{{.Names}}" | head -1)
if [ -n "$CONTAINER_NAME" ]; then
  echo "Stopping and removing container: ${CONTAINER_NAME}"
  docker rm -f "${CONTAINER_NAME}" || true
fi

rm -rf "${TEST_DIR}"
```

### Failure Path Cleanup

When a test fails, cleanup still occurs to prevent resource leaks:

```bash
# Clean up containers before failing
CONTAINER_NAME=$(docker ps -a --filter "label=devcontainer.local_folder=${TEST_DIR}" --format "{{.Names}}" | head -1)
if [ -n "$CONTAINER_NAME" ]; then
  docker rm -f "${CONTAINER_NAME}" || true
fi

rm -rf "${TEST_DIR}"
exit 1
```

### Container Identification

Containers are identified using Docker labels set by the Dev Container CLI:

- `devcontainer.local_folder=${TEST_DIR}` - Primary label
- `devcontainer.config_file=${TEST_DIR}/.devcontainer.json` - Fallback label

This ensures we find and clean up the correct containers even if naming conventions change.

## Verification

To verify test isolation is working correctly:

### 1. Check No Containers Remain

After workflow completion, no test containers should remain:

```bash
docker ps -a --filter "label=devcontainer.local_folder"
```

Should return no results (or only containers from other workflows).

### 2. Check Tests Pass in Any Order

Modify the feature discovery to test in different orders:

```bash
# Original order
features=$(find ${{ env.FEATURE_BASE_PATH }} -mindepth 1 -maxdepth 1 -type d -exec basename {} \;)

# Reversed order
features=$(find ${{ env.FEATURE_BASE_PATH }} -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | sort -r)

# Random order
features=$(find ${{ env.FEATURE_BASE_PATH }} -mindepth 1 -maxdepth 1 -type d -exec basename {} \; | shuf)
```

Tests should pass regardless of order.

### 3. Check Resource Usage

Monitor disk and memory usage during test runs. Should see:

- Temporary directories created and removed
- Container count stays constant (not growing)
- Disk space released after each test

## Best Practices Alignment

This implementation follows **Best Practice #4** from the research document:

> Use fresh, clean containers for each CI test, preventing contamination from previous runs and catching dependency/path issues.

**Reference:** [reference/research/241-devcontainer-feature-ci-testing/devcontainer-feature-ci-testing-best-practices-2025-11-22.md](../../reference/research/241-devcontainer-feature-ci-testing/devcontainer-feature-ci-testing-best-practices-2025-11-22.md#L142-L144)

## Troubleshooting

### Containers Not Being Cleaned Up

If you see containers accumulating:

1. Check the cleanup code is present in all exit paths
2. Verify the Docker label filters are correct
3. Add debugging output to see what containers are found:

```bash
echo "Looking for containers with label: devcontainer.local_folder=${TEST_DIR}"
docker ps -a --filter "label=devcontainer.local_folder=${TEST_DIR}"
```

### Cleanup Failures

The cleanup commands use `|| true` to prevent cleanup failures from stopping the workflow:

```bash
docker rm -f "${CONTAINER_NAME}" || true
```

This ensures:

- Tests can still fail/succeed based on their actual results
- Cleanup issues don't mask test failures
- Workflow completes even if cleanup has issues

## Future Enhancements

Potential improvements to test isolation:

1. **Parallel Test Execution**: Since tests are now fully isolated, they could run in parallel using GitHub Actions matrix strategy
2. **Volume Verification**: Add explicit checks that no volumes persist between tests
3. **Network Isolation**: Consider using custom Docker networks per test
4. **Resource Limits**: Set memory/CPU limits per test container to prevent resource starvation

## Related Issues

- #252 - 240.6-devcontainer-feature Isolate Test Runs (this issue)
- #240 - 201.3-devcontainer-feature-ci automated CI testing for devcontainer features (parent issue)
- #241 - Research best practices for devcontainer feature CI testing
