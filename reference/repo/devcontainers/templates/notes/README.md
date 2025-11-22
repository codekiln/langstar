# devcontainers/templates - CI Testing Analysis

## Repository Information

- **Repository**: [devcontainers/templates](https://github.com/devcontainers/templates)
- **Date Created**: 2025-11-22
- **Cloned to**: `/workspace/reference/repo/devcontainers/templates/code`
- **Analysis Context**: Issue #246 - Comparative research for devcontainer testing patterns

## Purpose

Analyze the official Microsoft devcontainers/templates repository to understand how templates are tested (compared to features). While langstar is a feature, understanding template testing provides additional context for comprehensive testing strategies.

## Key Findings

### CI Workflow Structure

The repository uses **one main test workflow**:

1. **test-pr.yaml** - Tests changed templates on PRs
   - Uses `dorny/paths-filter@v3` for change detection
   - More complex testing than features
   - Includes template configuration handling
   - Tests VS Code extension stubbing

### Test Workflow Differences from Features

**Templates are more complex**:
1. **Template Configuration** - Must handle `devcontainer-template.json` options
2. **Smoke Tests** - More involved than feature tests
3. **Extension Stubbing** - Fakes the VS Code Server for validation
4. **Test Directory Structure** - Templates have `test/<template>/` with project files

### Test Command Pattern

Unlike features, templates use multiple devcontainer CLI commands:

```bash
# Install CLI
npm install -g @devcontainers/cli

# Build and start container
devcontainer up --id-label test-container=<template-id> --workspace-folder src/<template>/

# Execute test script inside container
devcontainer exec --workspace-folder src/<template>/ --id-label test-container=<template-id> /bin/sh -c './test-project/test.sh'

# Clean up
docker rm -f $(docker container ls -f "label=test-container=<template-id>" -q)
```

### Template Configuration Handling

Before testing, the workflow **configures template options**:

```bash
# Extract options from devcontainer-template.json
options=$(jq -r '.options | keys[]' devcontainer-template.json)

# Replace option placeholders with default values
option_key="\${templateOption:$option}"
option_value=$(jq -r ".options | .${option} | .default" devcontainer-template.json)

# Find and replace in all files
find ./ -type f -print0 | xargs -0 sed -i "s/${option_key}/${option_value}/g"
```

This is **not needed for features** - features are simpler.

### Test Structure Per Template

Each template has:

1. **test/<template-name>/** - Test project files
   - Contains a sample project that uses the template
   - Example: test/rust/ contains a Rust project

2. **test/<template-name>/test.sh** - Test script
   - More complex than feature tests
   - Tests actual project functionality
   - Example: Build and run a Rust project

### VS Code Extension Stubbing

Templates test extension validation by:
1. Creating fake `~/.vscode-server` directory structure
2. Extracting extensions from devcontainer.json
3. Creating dummy extension directories
4. Allows `checkExtension()` to validate extensions exist

**Not needed for features** - features don't validate extensions.

## CI Workflow Details (test-pr.yaml)

### Structure

```yaml
jobs:
  detect-changes:    # Detect which templates changed
  test:              # Test changed templates
```

### Change Detection Job

Same as features:
- Uses `dorny/paths-filter@v3`
- Outputs JSON array of changed templates

### Test Job Steps

1. **Checkout** - Standard checkout
2. **Install devcontainer CLI** - `npm install -g @devcontainers/cli`
3. **Configure template** - Replace option placeholders
4. **Run Smoke Test**:
   - Copy test files to template directory
   - Build devcontainer with `devcontainer up`
   - Stub VS Code extensions
   - Execute test.sh inside container
   - Clean up Docker containers

## Comparison: Features vs Templates Testing

| Aspect | Features | Templates |
|--------|----------|-----------|
| **Test command** | `devcontainer features test` | `devcontainer up` + `devcontainer exec` |
| **Configuration** | Not needed | Must replace option placeholders |
| **Test complexity** | Simple smoke tests | Full project tests |
| **Extension handling** | Not validated | Stubbed and validated |
| **Matrix testing** | 7 base images | Single image per template |
| **Test directory** | test/feature/test.sh | test/template/ (project files) |
| **Test library** | dev-container-features-test-lib | Manual bash scripts |

## Actionable Insights for Langstar

### What to Adopt from Templates

**Nothing** - Langstar is a **feature**, not a template.

Template testing is more complex because:
- Templates create entire project scaffolds
- Templates have configurable options with placeholders
- Templates need to validate project functionality

Features (like langstar) are simpler:
- Install tools/dependencies
- Configure environment
- Run version checks

### Confirmed: Features Testing is Correct Approach

The comparison confirms that **feature testing** (as documented in devcontainers/features analysis) is the right approach for langstar:

✅ Use `devcontainer features test` command
✅ Simple test.sh with version checks
✅ Test across multiple base images
✅ Use dev-container-features-test-lib
✅ No need for complex project scaffolding

### What NOT to Do

❌ Don't use `devcontainer up` + `devcontainer exec` pattern (that's for templates)
❌ Don't create complex test projects (that's for templates)
❌ Don't stub VS Code extensions (features don't need this)
❌ Don't handle template option placeholders (features don't have them)

## References

- Workflow file: `/workspace/reference/repo/devcontainers/templates/code/.github/workflows/test-pr.yaml`
- Dev Container CLI docs: https://github.com/devcontainers/cli
