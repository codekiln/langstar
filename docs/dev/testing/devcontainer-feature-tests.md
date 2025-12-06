# DevContainer Feature Testing

> **📍 Centralized Testing Documentation**
>
> This document is part of the centralized testing documentation suite. See `@docs/dev/testing/README.md` for the complete TOC.

## Overview

This document describes testing and publishing workflows for the Langstar DevContainer feature, including GitHub Actions automation, local testing procedures, and publishing steps.

## Feature Information

- **Feature Directory**: `.devcontainer/features/langstar/`
- **Feature ID**: `langstar`
- **Published Location**: `ghcr.io/codekiln/langstar/langstar:1`

## Workflow Files

### Test Workflow: `.github/workflows/test-features.yml`

**Purpose**: Validates feature metadata and tests feature installation across multiple base images.

**Triggers**:
- Push to `main` branch when `.devcontainer/features/**` changes
- Push of tags matching `v*` pattern
- Pull requests that modify `.devcontainer/features/**`
- Manual dispatch via `workflow_dispatch`

**Jobs**:

#### Job 1: `validate-metadata`
- **Purpose**: Validates feature metadata before testing
- **Runs on**: `ubuntu-latest`
- **Steps**:
  1. Checkout code
  2. Install Node.js 20
  3. Install Dev Container CLI (`@devcontainers/cli`)
  4. Install `jq` for JSON validation
  5. Validate each feature's `devcontainer-feature.json`:
     - Check file exists
     - Validate JSON syntax
     - Verify required fields: `id`, `version`, `name`, `description`
     - Validate option definitions (if present):
       - Each option must have `type` field
       - Each option must have `description` field

**Validation Requirements**:
```json
{
  "id": "langstar",           // Required
  "version": "1.0.0",         // Required
  "name": "Langstar CLI",     // Required
  "description": "...",        // Required
  "options": {                 // Optional, but if present:
    "version": {
      "type": "string",        // Required for each option
      "description": "..."    // Required for each option
    }
  }
}
```

#### Job 2: `test-features`
- **Purpose**: Tests feature installation on multiple base images
- **Runs on**: `ubuntu-latest`
- **Depends on**: `validate-metadata` job
- **Matrix Strategy**: Tests across 6 base images:
  - `mcr.microsoft.com/devcontainers/base:ubuntu-22.04`
  - `mcr.microsoft.com/devcontainers/base:ubuntu-24.04`
  - `mcr.microsoft.com/devcontainers/base:debian-12`
  - `mcr.microsoft.com/devcontainers/base:debian-11`
  - `mcr.microsoft.com/devcontainers/base:alpine-3.19`
  - `mcr.microsoft.com/devcontainers/base:alpine-3.18`

**Test Process**:
1. **Setup**:
   - Checkout code
   - Install Node.js 20
   - Install Dev Container CLI
   - Discover features in `.devcontainer/features/`

2. **For Each Feature**:
   - Create unique temporary directory (`mktemp -d`)
   - Create test `devcontainer.json` referencing local feature path
   - Build and start container: `devcontainer up --workspace-folder <TEST_DIR>`
   - Verify installation: `devcontainer exec --workspace-folder <TEST_DIR> bash -c "command -v 'langstar' && 'langstar' --version"`
   - Clean up containers and test directory

3. **Test Isolation**:
   - Each test runs in a unique temporary directory
   - Each test gets a fresh container instance
   - Containers are explicitly stopped and removed after each test
   - No shared state between tests
   - Tests can run in any order

**Verification Commands**:
```bash
# The workflow verifies:
command -v langstar && langstar --version
```

**Logging**:
- Test logs saved to `logs/test-<base-image>.log`
- Summary logs saved to `logs/summary-<base-image>.log`
- Individual feature logs: `logs/build-<feature>-<base-image>.log` and `logs/exec-<feature>-<base-image>.log`
- Logs uploaded as artifacts:
  - On failure: `test-logs-failure-<base-image>` (30 day retention)
  - On success: `test-logs-success-<base-image>` (7 day retention)

**Success Criteria**:
- Container builds successfully
- Feature installs without errors
- `langstar` command is available in PATH
- `langstar --version` executes successfully

### Release Workflow: `.github/workflows/release-features.yaml`

**Purpose**: Publishes features to GitHub Container Registry (GHCR) and generates documentation.

**Triggers**:
- Manual dispatch only (`workflow_dispatch`)
- Only runs from `main` branch

**Permissions Required**:
- `contents: write` - For creating PRs with generated docs
- `pull-requests: write` - For PR creation
- `packages: write` - For publishing to GHCR

**Process**:
1. Checkout repository
2. Publish features using `devcontainers/action@v1`:
   - `publish-features: "true"`
   - `base-path-to-features: "./.devcontainer/features"`
   - `generate-docs: "true"`
   - Uses `GITHUB_TOKEN` for authentication

**What Happens**:
- Features are published to GHCR: `ghcr.io/codekiln/langstar/langstar:1`
- Documentation is auto-generated
- PR is created with updated READMEs (if docs changed)

**Important Notes**:
- First publish creates package as **private** by default
- Must manually change visibility to **public** in GHCR package settings
- URL format: `https://github.com/users/codekiln/packages/container/langstar%2Flangstar/settings`

## Feature Structure

### File: `.devcontainer/features/langstar/devcontainer-feature.json`
```json
{
    "id": "langstar",
    "name": "Langstar CLI",
    "version": "1.0.0",
    "description": "Installs the Langstar CLI...",
    "documentationURL": "https://github.com/codekiln/langstar",
    "options": {
        "version": {
            "type": "string",
            "default": "latest",
            "description": "Version of Langstar CLI to install..."
        }
    },
    "installsAfter": [
        "ghcr.io/devcontainers/features/common-utils"
    ]
}
```

### File: `.devcontainer/features/langstar/install.sh`
- Executes the official installer script from `scripts/install.sh`
- Handles root vs non-root installation
- Verifies installation with `command -v langstar && langstar --version`
- Uses `VERSION` environment variable (from feature options)

**Key Installation Logic**:
```bash
# Detects installation prefix based on privileges
if [ "$(id -u)" -eq 0 ] || [ -w "/usr/local/bin" ]; then
    INSTALLER_PREFIX="/usr/local/bin"
else
    INSTALLER_PREFIX="$HOME/.local/bin"
fi

# Calls official installer
curl -fsSL https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | \
    bash -s -- --prefix "${INSTALLER_PREFIX}" [--version "${VERSION}"]
```

### File: `.devcontainer/features/langstar/test.sh`
- Smoke tests for manual verification
- Tests:
  1. Binary in PATH
  2. `--version` command works
  3. `--help` command works
  4. Version matches requested version (if specified)

## Local Testing

### Pre-Commit Testing

1. **Validate Feature Metadata**:
   ```bash
   # Install dependencies
   npm install -g @devcontainers/cli
   sudo apt-get install -y jq

   # Validate JSON syntax
   jq empty .devcontainer/features/langstar/devcontainer-feature.json

   # Check required fields
   jq -r '.id, .version, .name, .description' .devcontainer/features/langstar/devcontainer-feature.json
   ```

2. **Test Feature Installation Locally**:
   ```bash
   # Create test directory
   TEST_DIR=$(mktemp -d)

   # Create test devcontainer.json
   cat > "${TEST_DIR}/.devcontainer.json" <<EOF
   {
     "name": "Test Langstar",
     "image": "mcr.microsoft.com/devcontainers/base:ubuntu-22.04",
     "features": {
       "$(pwd)/.devcontainer/features/langstar": {}
     }
   }
   EOF

   # Build and test
   devcontainer up --workspace-folder "${TEST_DIR}"
   devcontainer exec --workspace-folder "${TEST_DIR}" langstar --version

   # Cleanup
   CONTAINER_NAME=$(docker ps -a --filter "label=devcontainer.local_folder=${TEST_DIR}" --format "{{.Names}}" | head -1)
   docker rm -f "${CONTAINER_NAME}" || true
   rm -rf "${TEST_DIR}"
   ```

3. **Test with Specific Version**:
   ```bash
   # Modify test devcontainer.json to include version option
   jq '.features["'$(pwd)'/.devcontainer/features/langstar"].version = "v0.4.0"' \
     "${TEST_DIR}/.devcontainer.json" > "${TEST_DIR}/.devcontainer.json.tmp" && \
     mv "${TEST_DIR}/.devcontainer.json.tmp" "${TEST_DIR}/.devcontainer.json"

   # Rebuild and verify version
   devcontainer build --workspace-folder "${TEST_DIR}"
   devcontainer exec --workspace-folder "${TEST_DIR}" langstar --version
   ```

### Post-Commit Testing

After pushing changes, verify:

1. **GitHub Actions Workflow Runs**:
   - Check Actions tab for `Test DevContainer Features` workflow
   - Verify `validate-metadata` job passes
   - Verify `test-features` job passes for all 6 base images

2. **Review Test Logs**:
   - Download artifacts if tests fail
   - Check `logs/test-*.log` for detailed output
   - Verify container build logs show successful installation
   - Verify execution logs show `langstar --version` succeeds

3. **Test on Multiple Base Images**:
   The workflow automatically tests on:
   - Ubuntu 22.04, 24.04
   - Debian 11, 12
   - Alpine 3.18, 3.19

### Manual Testing Checklist

- [ ] Feature metadata validates (JSON syntax, required fields)
- [ ] Feature installs on Ubuntu 22.04
- [ ] Feature installs on Ubuntu 24.04
- [ ] Feature installs on Debian 11
- [ ] Feature installs on Debian 12
- [ ] Feature installs on Alpine 3.18
- [ ] Feature installs on Alpine 3.19
- [ ] `langstar` command available in PATH
- [ ] `langstar --version` works
- [ ] `langstar --help` works
- [ ] Version pinning works (test with `"version": "v0.4.0"`)
- [ ] Latest version works (test with `"version": "latest"` or no version option)
- [ ] Containers clean up properly after tests

### Common Issues to Test For

1. **Installation Path Issues**:
   - Root vs non-root installation
   - PATH not including installation directory
   - Binary permissions

2. **Version Resolution Issues**:
   - Latest version resolution
   - Specific version download
   - Invalid version handling

3. **Architecture Detection**:
   - x86_64 (amd64) detection
   - ARM64 (aarch64) detection
   - Wrong architecture binary download

4. **Network Issues**:
   - GitHub Releases API access
   - Binary download failures
   - Checksum verification failures

5. **Container Cleanup**:
   - Containers properly removed after tests
   - No resource leaks
   - Test isolation maintained

## Publishing Process

### When to Publish

Publish features manually via workflow dispatch when:
- Feature changes are merged to `main`
- New feature version is ready
- Documentation needs regeneration

### Publishing Steps

1. **Navigate to Actions**:
   - Go to GitHub Actions → `Release dev container features & Generate Documentation`
   - Click "Run workflow" → Select `main` branch → Click "Run workflow"

2. **Monitor Workflow**:
   - Wait for workflow to complete
   - Check for any errors

3. **Review Generated PR** (if docs changed):
   - Review auto-generated documentation PR
   - Merge if acceptable

4. **Make Package Public** (first time only):
   - Navigate to GHCR package settings
   - Change visibility from `private` to `public`

### Publishing Checklist

- [ ] Feature tested and passing CI
- [ ] Feature metadata validated
- [ ] All base images tested successfully
- [ ] Workflow dispatch triggered
- [ ] Package published to GHCR
- [ ] Package visibility set to public (if first publish)
- [ ] Documentation PR reviewed and merged (if created)

## Key Takeaways

1. **Always validate metadata** before testing
2. **Test on multiple base images** (workflow does this automatically)
3. **Verify installation** with `command -v langstar && langstar --version`
4. **Ensure proper cleanup** of containers and test directories
5. **Check test logs** in artifacts if tests fail
6. **Test both latest and pinned versions** if version handling changed
7. **Verify PATH** includes installation directory
8. **Test root and non-root** installation scenarios

## Related Documentation

- `.github/workflows/test-features.yml` - Test workflow
- `.github/workflows/release-features.yaml` - Release workflow
- `docs/devcontainer-feature.md` - Feature documentation
- `.github/workflows/TEST-ISOLATION.md` - Test isolation guide
- `.devcontainer/features/langstar/` - Feature source
- `scripts/install.sh` - Official installer
- [.devcontainer/features/langstar/TESTING-GITHUB-ACTIONS.md](../../../.devcontainer/features/langstar/TESTING-GITHUB-ACTIONS.md) - Quick reference (redirects here)
