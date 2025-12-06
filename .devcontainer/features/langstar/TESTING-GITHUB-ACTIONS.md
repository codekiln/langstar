# Testing DevContainer Features

> **📍 Documentation Centralized**
>
> See `@docs/dev/testing/devcontainer-feature-tests.md` for complete documentation.

## Quick Reference

**Test locally:**
```bash
# Create test directory
TEST_DIR=$(mktemp -d)

# Create devcontainer.json
cat > "${TEST_DIR}/.devcontainer.json" <<EOF
{
  "image": "mcr.microsoft.com/devcontainers/base:ubuntu-22.04",
  "features": {
    "$(pwd)/.devcontainer/features/langstar": {}
  }
}
EOF

# Build and test
devcontainer up --workspace-folder "${TEST_DIR}"
devcontainer exec --workspace-folder "${TEST_DIR}" langstar --version
```

**Workflow:** `.github/workflows/test-features.yml`

**Full documentation:** `docs/dev/testing/devcontainer-feature-tests.md`
