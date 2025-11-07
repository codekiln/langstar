# Configuration Guide

This guide provides comprehensive documentation for configuring Langstar to work with LangSmith and LangGraph Cloud services.

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration Methods](#configuration-methods)
- [Configuration File](#configuration-file)
- [Environment Variables](#environment-variables)
- [Service-Specific Configuration](#service-specific-configuration)
- [Configuration Precedence](#configuration-precedence)
- [Common Scenarios](#common-scenarios)
- [Viewing Configuration](#viewing-configuration)
- [Migration Guide](#migration-guide)

## Quick Start

The fastest way to get started depends on which Langstar commands you plan to use:

### For LangSmith Prompts Only

```bash
export LANGSMITH_API_KEY="<your-api-key>"
langstar prompt list
```

### For LangGraph Assistants Only

```bash
export LANGSMITH_API_KEY="<your-api-key>"
langstar assistant list
```

### For Both Services

```bash
export LANGSMITH_API_KEY="<your-langsmith-key>"
export LANGSMITH_API_KEY="<your-api-key>"

langstar prompt list      # Uses LANGSMITH_API_KEY
langstar assistant list   # Uses LANGSMITH_API_KEY
```

## Configuration Methods

Langstar supports three configuration methods, evaluated in this order of precedence:

1. **Command-line flags** (highest priority)
2. **Configuration file** (`~/.langstar/config.toml`)
3. **Environment variables** (lowest priority)

### When to Use Each Method

**Command-line flags:**
- One-time operations
- Overriding default configuration for a single command
- Scripting with varying parameters

**Configuration file:**
- Persistent settings used regularly
- Team environments with shared configuration
- Multiple API keys or projects

**Environment variables:**
- CI/CD pipelines
- Containerized environments
- Quick temporary overrides
- Development environments

## Configuration File

### Location

Langstar looks for configuration in these locations (in order):

1. `./config.toml` (current directory)
2. `~/.langstar/config.toml` (user home)
3. `~/.config/langstar/config.toml` (XDG config)

### Format

The configuration file uses TOML format:

```toml
[langstar]
# General settings
output_format = "table"  # or "json"

# LangSmith configuration (for prompt commands)
langsmith_api_key = "<your-langsmith-key>"
organization_id = "<your-org-id>"        # Optional
workspace_id = "<your-workspace-id>"     # Optional

# LangGraph configuration (for assistant commands)
# Assistants use the same langsmith_api_key
```

### Complete Example

```toml
[langstar]
# Output format for all commands (can be overridden with --format flag)
output_format = "table"

# LangSmith API configuration
langsmith_api_key = "<your-langsmith-key>"

# Optional: Organization scoping for prompts
# When set, prompt operations default to private prompts in this org
organization_id = "<your-org-id>"
organization_name = "My Organization"  # Informational only

# Optional: Workspace scoping for prompts
# Narrows scope from organization to specific workspace
workspace_id = "<your-workspace-id>"
workspace_name = "My Workspace"  # Informational only

# LangGraph API configuration
# Separate key for LangGraph assistants
# Assistants use the same langsmith_api_key
```

## Environment Variables

### LangSmith Service

Used by `langstar prompt *` commands:

#### Required

```bash
export LANGSMITH_API_KEY="<your-api-key>"
```

#### Optional (Organization/Workspace Scoping)

```bash
# Organization scoping
export LANGSMITH_ORGANIZATION_ID="<your-org-id>"
export LANGSMITH_ORGANIZATION_NAME="<org-name>"  # Informational

# Workspace scoping (requires organization)
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
export LANGSMITH_WORKSPACE_NAME="<workspace-name>"  # Informational
```

### LangGraph Service

Used by `langstar assistant *` commands:

#### Required

```bash
# Preferred
export LANGSMITH_API_KEY="<your-api-key>"

# Or fallback to LangSmith key
export LANGSMITH_API_KEY="<your-api-key>"
```

#### No Additional Variables Needed

LangGraph assistants are deployment-level resources:
- ❌ No `LANGSMITH_ORGANIZATION_ID` needed
- ❌ No `LANGSMITH_WORKSPACE_ID` needed
- ✅ Scoped automatically by your API key

### General Settings

```bash
# Output format (table or json)
export LANGSTAR_OUTPUT_FORMAT="json"
```

## Service-Specific Configuration

### LangSmith Prompts

#### Purpose

LangSmith prompts support hierarchical multi-tenancy:
- **Organizations** contain multiple workspaces
- **Workspaces** contain prompts
- Scoping controls which prompts you can access

#### Configuration Options

| Option | Required | Purpose |
|--------|----------|---------|
| `langsmith_api_key` | Yes | Authentication |
| `organization_id` | No | Scope to organization's prompts |
| `workspace_id` | No | Scope to workspace's prompts |

#### Scoping Behavior

**No scoping (default):**
```bash
export LANGSMITH_API_KEY="<key>"
langstar prompt list
# Returns: All public prompts + your personal prompts
```

**Organization scoping:**
```bash
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_ORGANIZATION_ID="<org-id>"
langstar prompt list
# Returns: Private prompts in organization (default)
langstar prompt list --public
# Returns: Public prompts in organization
```

**Workspace scoping:**
```bash
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_WORKSPACE_ID="<workspace-id>"
langstar prompt list
# Returns: Private prompts in workspace
```

#### Command-Line Overrides

```bash
# Override organization for one command
langstar prompt list --organization-id "<different-org-id>"

# Override workspace
langstar prompt search "query" --workspace-id "<workspace-id>"

# Access public prompts when scoped
langstar prompt list --organization-id "<org-id>" --public
```

### LangGraph Assistants

#### Purpose

LangGraph assistants are deployment-level resources:
- Each API key is tied to a specific deployment
- No additional scoping configuration needed
- Simpler model for graph-based applications

#### Configuration Options

| Option | Required | Purpose |
|--------|----------|---------|
| `langsmith_api_key` | Yes | Authentication + deployment scoping |

#### How It Works

```bash
export LANGSMITH_API_KEY="<key>"
langstar assistant list
# Returns: All assistants in the deployment tied to this API key
```

The API key automatically determines:
- Which deployment you're accessing
- Which assistants you can see and manage
- Authentication and authorization

#### No Additional Configuration

These do NOT apply to assistants:
- ❌ `organization_id` - Not used
- ❌ `workspace_id` - Not used
- ❌ No `--organization-id` flags
- ❌ No `--workspace-id` flags

## Configuration Precedence

When the same setting is defined in multiple places, Langstar uses this precedence order:

1. **Command-line flags** (highest)
2. **Configuration file**
3. **Environment variables** (lowest)

### Examples

**Scenario 1: API key override**

```bash
# Config file has: langsmith_api_key = "key-from-file"
export LANGSMITH_API_KEY="key-from-env"

langstar prompt list
# Uses: key-from-env (environment overrides file)
```

**Scenario 2: Organization override**

```bash
# Config file has: organization_id = "org-from-file"
export LANGSMITH_ORGANIZATION_ID="org-from-env"

langstar prompt list --organization-id "org-from-flag"
# Uses: org-from-flag (flag overrides everything)
```

**Scenario 3: Multiple sources**

```toml
# config.toml
[langstar]
output_format = "table"
langsmith_api_key = "key-from-file"
```

```bash
export LANGSMITH_ORGANIZATION_ID="org-from-env"

langstar prompt list --workspace-id "workspace-from-flag" --format json
```

Result:
- API key: `key-from-file` (config file)
- Organization: `org-from-env` (environment)
- Workspace: `workspace-from-flag` (command flag)
- Output format: `json` (command flag)

## Common Scenarios

### Personal Use (Single User)

**Setup:**

```bash
# Create config file
mkdir -p ~/.langstar
cat > ~/.langstar/config.toml <<EOF
[langstar]
langsmith_api_key = "<your-langsmith-key>"
# Assistants use the same langsmith_api_key
output_format = "table"
EOF
```

**Usage:**

```bash
# Just run commands, configuration is automatic
langstar prompt list
langstar assistant list
```

### Team Environment (Organization Scoped)

**Setup:**

```toml
# ~/.langstar/config.toml
[langstar]
langsmith_api_key = "<your-key>"
langsmith_api_key = "<your-key>"

# Scope prompts to team organization
organization_id = "<team-org-id>"
```

**Usage:**

```bash
# Lists private prompts in team organization
langstar prompt list

# Access public prompts
langstar prompt list --public

# Assistants work the same (no scoping)
langstar assistant list
```

### Multi-Organization (Consulting/Enterprise)

**Setup:**

```toml
# ~/.langstar/config.toml
[langstar]
langsmith_api_key = "<your-key>"
langsmith_api_key = "<your-key>"

# Default organization (can be overridden)
organization_id = "<default-org-id>"
```

**Usage:**

```bash
# Use default org
langstar prompt list

# Override for client A
langstar prompt list --organization-id "<client-a-org-id>"

# Override for client B workspace
langstar prompt search "query" --workspace-id "<client-b-workspace-id>"
```

### CI/CD Pipeline

**Setup:**

```yaml
# .github/workflows/test.yml
env:
  LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
  LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
  LANGSTAR_OUTPUT_FORMAT: json
```

**Usage:**

```bash
# Environment variables configured in CI
langstar prompt list --format json | jq '.[] | .name'
langstar assistant search "test-bot" --format json
```

### Development vs Production

**Setup:**

```bash
# Development
export LANGSMITH_API_KEY="<dev-key>"
export LANGSMITH_API_KEY="<dev-key>"

# Production (use different keys)
export LANGSMITH_API_KEY="<prod-key>"
export LANGSMITH_API_KEY="<prod-key>"
```

**Usage:**

```bash
# Test in development
langstar assistant list

# Deploy to production (different API key automatically scopes to prod deployment)
export LANGSMITH_API_KEY="<prod-key>"
langstar assistant create --graph-id "<graph-id>" --name "Prod Bot"
```

## Viewing Configuration

Check your current configuration at any time:

```bash
langstar config
```

**Example output:**

```
Configuration file: ~/.langstar/config.toml

LangSmith Configuration (for 'prompt' commands):
  API key: configured
  Organization ID: <your-org-id> (scopes prompt operations)
  Workspace ID: <your-workspace-id> (narrows scope further)
  → Prompt commands will use workspace-scoped resources

LangGraph Configuration (for 'assistant' commands):
  API key: configured
  → Assistant commands use deployment-level resources
  → No organization/workspace scoping available

Output Format: table
```

**What it shows:**
- Configuration file location
- Which API keys are set (without exposing the actual keys)
- Organization/workspace scoping status
- Output format setting

**What it doesn't show:**
- Actual API key values (security)
- Environment variable values (use `env | grep LANG` to check)

## Migration Guide

### From Environment-Only to Config File

**Before:**

```bash
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_ORGANIZATION_ID="<org-id>"
export LANGSMITH_API_KEY="<key>"
```

**After:**

```bash
# Create config file
mkdir -p ~/.langstar
cat > ~/.langstar/config.toml <<EOF
[langstar]
langsmith_api_key = "<key>"
organization_id = "<org-id>"
langsmith_api_key = "<key>"
EOF

# Remove environment variables if desired
unset LANGSMITH_API_KEY
unset LANGSMITH_ORGANIZATION_ID
unset LANGSMITH_API_KEY
```

**Benefits:**
- Configuration persists across shell sessions
- Easier to manage multiple settings
- Can still override with environment or flags

### From Shared to Per-Service Keys

**Before:**

```bash
export LANGSMITH_API_KEY="<same-key-for-both>"
# Langstar falls back to LANGSMITH_API_KEY for both services
```

**After:**

```bash
export LANGSMITH_API_KEY="<langsmith-key>"
export LANGSMITH_API_KEY="<langgraph-key>"
```

**When to do this:**
- Using different API keys for different services
- Separate deployment permissions
- Enhanced security (principle of least privilege)

### Adding Organization Scoping

**Before:**

```toml
[langstar]
langsmith_api_key = "<key>"
```

```bash
langstar prompt list  # Returns all public + personal prompts
```

**After:**

```toml
[langstar]
langsmith_api_key = "<key>"
organization_id = "<org-id>"
```

```bash
langstar prompt list  # Returns private prompts in org
langstar prompt list --public  # Returns public prompts in org
```

**When to do this:**
- Working with team prompts
- Need to filter to organization's resources
- Enterprise deployment with multiple organizations

## Troubleshooting

### Configuration Not Found

**Symptom:**

```
Error: Missing required configuration: LANGSMITH_API_KEY
```

**Solution:**

1. Check if API key is set:
   ```bash
   env | grep LANGSMITH_API_KEY
   langstar config
   ```

2. Set via environment:
   ```bash
   export LANGSMITH_API_KEY="<your-key>"
   ```

3. Or create config file:
   ```bash
   mkdir -p ~/.langstar
   cat > ~/.langstar/config.toml <<EOF
   [langstar]
   langsmith_api_key = "<your-key>"
   EOF
   ```

### Wrong API Key for Service

**Symptom:**

```bash
langstar assistant list
Error: Authentication failed
```

**Solution:**

Check which key is being used:

```bash
langstar config
```

Ensure `LANGSMITH_API_KEY` is set (not just `LANGSMITH_API_KEY`):

```bash
export LANGSMITH_API_KEY="<your-api-key>"
```

### Unexpected Scoping Behavior

**Symptom:**

```bash
langstar prompt list
# Returns fewer prompts than expected
```

**Solution:**

Check if organization/workspace scoping is active:

```bash
langstar config
```

If scoped, either:
- Add `--public` flag to access public prompts
- Remove scoping to access all prompts:
  ```bash
  unset LANGSMITH_ORGANIZATION_ID
  unset LANGSMITH_WORKSPACE_ID
  ```

### Configuration File Not Used

**Symptom:**

Config file exists but settings aren't applied.

**Solution:**

1. Check file location:
   ```bash
   ls ~/.langstar/config.toml
   langstar config  # Shows which file is used
   ```

2. Verify TOML syntax:
   ```bash
   cat ~/.langstar/config.toml
   ```

3. Check for environment variable overrides:
   ```bash
   env | grep LANGSMITH
   env | grep LANGGRAPH
   ```

## Security Best Practices

### Protecting API Keys

**DO:**
- ✅ Use config file with restricted permissions:
  ```bash
  chmod 600 ~/.langstar/config.toml
  ```
- ✅ Use environment variables in CI/CD
- ✅ Use different keys for dev/staging/production
- ✅ Rotate keys regularly

**DON'T:**
- ❌ Commit API keys to version control
- ❌ Share API keys in documentation
- ❌ Use production keys in development
- ❌ Store keys in world-readable files

### Using Config Files Safely

```bash
# Create config file with restricted permissions
mkdir -p ~/.langstar
cat > ~/.langstar/config.toml <<EOF
[langstar]
langsmith_api_key = "<your-key>"
langsmith_api_key = "<your-key>"
EOF

# Restrict access to owner only
chmod 600 ~/.langstar/config.toml

# Verify permissions
ls -la ~/.langstar/config.toml
# Should show: -rw------- (owner read/write only)
```

### Environment Variables in Scripts

```bash
#!/bin/bash
# Load keys from secure location
source ~/.langstar_secrets

# Use keys (not echoed or logged)
langstar prompt list > output.json

# Don't print or log the actual keys
# ❌ echo $LANGSMITH_API_KEY
# ✅ echo "API key is set: $([ -n "$LANGSMITH_API_KEY" ] && echo "yes" || echo "no")"
```

## Additional Resources

- [README.md](../README.md) - Quick start guide
- [Architecture Documentation](./architecture.md) - How configuration is implemented
- [Troubleshooting Guide](./troubleshooting.md) - Common issues and solutions
- [LangSmith Documentation](https://docs.smith.langchain.com/) - LangSmith API details
- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/) - LangGraph API details
