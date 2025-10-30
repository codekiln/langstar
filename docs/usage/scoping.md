# Organization and Workspace Scoping

This guide explains how to use organization and workspace scoping in Langstar to manage team prompts and enterprise deployments.

## Overview

Langstar supports scoping operations to specific organizations and workspaces within LangSmith. This allows you to:

- Access private prompts within your organization
- Limit operations to a specific workspace
- Maintain separation between team and personal prompts
- Comply with enterprise access controls

## Concepts

### Organization vs Workspace

- **Organization**: A top-level entity in LangSmith (e.g., your company or team)
- **Workspace**: A narrower scope within an organization (e.g., a specific project or department)

Workspaces are nested under organizations, so workspace-level scoping is more restrictive than organization-level scoping.

### Public vs Private Prompts

LangSmith prompts can be either:
- **Public**: Visible to anyone with the link
- **Private**: Only visible to members of the organization/workspace

When you scope operations to an organization or workspace, Langstar defaults to showing only **private prompts** to respect your team's privacy.

## Configuration

You can configure organization and workspace IDs using three methods. CLI flags take highest precedence, followed by config file settings, then environment variables.

### Method 1: Environment Variables

Set environment variables in your shell:

```bash
export LANGSMITH_ORGANIZATION_ID="d1e1dfff-39bf-4cea-9a2e-85e970ce40ef"
export LANGSMITH_WORKSPACE_ID="6f52dd84-9870-4f3a-b42d-4eea5fc9dfde"
```

Add to your shell profile (`~/.bashrc`, `~/.zshrc`) to make persistent:

```bash
# LangSmith Configuration
export LANGSMITH_API_KEY="your-api-key"
export LANGSMITH_ORGANIZATION_ID="your-org-id"
export LANGSMITH_WORKSPACE_ID="your-workspace-id"
```

### Method 2: Config File

Add to `~/.config/langstar/config.toml`:

```toml
langsmith_api_key = "your-api-key"
organization_id = "your-org-id"
workspace_id = "your-workspace-id"
```

### Method 3: CLI Flags

Override configuration on a per-command basis:

```bash
langstar prompt list --organization-id "your-org-id"
langstar prompt list --workspace-id "your-workspace-id"
```

## Finding Your Organization and Workspace IDs

### Organization ID

1. Navigate to [smith.langchain.com](https://smith.langchain.com)
2. Click on your organization name in the top navigation
3. Go to "Settings" → "General"
4. Copy the Organization ID

Alternatively, check your browser URL when viewing organization settings:
```
https://smith.langchain.com/settings?organizationId=<your-org-id>
```

### Workspace ID

1. Navigate to your workspace in LangSmith
2. Go to workspace settings
3. Copy the Workspace ID

Or check the URL when viewing a workspace:
```
https://smith.langchain.com/prompts?workspaceId=<your-workspace-id>
```

## Default Behavior

### Without Scoping

When no organization or workspace ID is configured:

```bash
langstar prompt list
# Returns: All public prompts + your personal private prompts
```

### With Organization Scoping

When organization ID is set:

```bash
export LANGSMITH_ORGANIZATION_ID="your-org-id"
langstar prompt list
# Returns: Private prompts within the organization ONLY
```

To access public prompts when scoped:

```bash
langstar prompt list --public
# Returns: Public prompts within the organization
```

### With Workspace Scoping

When workspace ID is set (narrower scope):

```bash
export LANGSMITH_WORKSPACE_ID="your-workspace-id"
langstar prompt list
# Returns: Private prompts within the workspace ONLY
```

Workspace ID takes precedence over organization ID when both are set.

## Common Workflows

### Listing Team Prompts

List all private prompts in your organization:

```bash
langstar prompt list --organization-id "your-org-id"
```

With more results:

```bash
langstar prompt list --organization-id "your-org-id" --limit 100
```

### Searching Within a Workspace

Search for prompts in a specific workspace:

```bash
langstar prompt search "rag" --workspace-id "your-workspace-id"
```

### Accessing Public Prompts When Scoped

When working within an organization but need to access public prompts:

```bash
# List public prompts in the LangChain org
langstar prompt list --organization-id "langchain-org-id" --public

# Search public prompts
langstar prompt search "retrieval" --organization-id "langchain-org-id" --public
```

### Getting a Specific Prompt

Organization scoping is required when accessing private prompts:

```bash
# Private prompt (requires org scoping)
langstar prompt get "my-team/internal-prompt" --organization-id "your-org-id"

# Public prompt (no scoping needed)
langstar prompt get "langchain-ai/rag-answer-w-sources"
```

### Pushing a Prompt to Your Organization

Create or update a prompt in your organization:

```bash
langstar prompt push \
  --owner "my-team" \
  --repo "my-prompt" \
  --template "You are a helpful assistant. Context: {context}" \
  --input-variables "context" \
  --organization-id "your-org-id"
```

## Precedence Rules

When multiple configuration methods are used, Langstar follows this precedence order:

1. **CLI flags** (highest priority)
2. **Config file** (`~/.config/langstar/config.toml`)
3. **Environment variables** (lowest priority)

### Example

```bash
# Environment variable
export LANGSMITH_ORGANIZATION_ID="org-from-env"

# Config file contains: organization_id = "org-from-config"

# Command with flag
langstar prompt list --organization-id "org-from-cli"
# Uses: "org-from-cli" (CLI flag wins)

# Command without flag
langstar prompt list
# Uses: "org-from-config" (config file beats env var)
```

### Workspace Takes Precedence Over Organization

When both workspace and organization IDs are set, the workspace scope is used (narrower scope wins):

```bash
langstar prompt list \
  --organization-id "your-org-id" \
  --workspace-id "your-workspace-id"
# Scopes to workspace only (organization ID ignored)
```

## Output Format

Langstar shows scoping information in table output:

```bash
$ langstar prompt list --organization-id "your-org-id"

Handle                          Likes  Downloads  Public  Description
my-team/rag-prompt                 5         12      no   RAG retrieval prompt...
my-team/customer-support-bot      15        120      no   Customer support template...
```

The "Public" column indicates whether prompts are public or private.

## Troubleshooting

### Error: "Prompt not found"

If you get a 404 error when accessing a prompt:

1. **Verify the prompt handle**: Check spelling and format (`owner/repo-name`)
2. **Check scoping**: Private prompts require organization ID:
   ```bash
   langstar prompt get "my-team/prompt" --organization-id "your-org-id"
   ```
3. **Verify access**: Ensure your API key has access to the organization

### No Results Returned

If `langstar prompt list` returns no results when scoped:

1. **Check your organization/workspace ID**: Verify the ID is correct
2. **Try public flag**: Check if prompts are public:
   ```bash
   langstar prompt list --organization-id "your-org-id" --public
   ```
3. **Verify API key permissions**: Ensure your API key has access to the organization

### Permission Denied Errors

If you get permission errors:

1. **Check API key**: Verify your `LANGSMITH_API_KEY` is valid
2. **Check organization membership**: Ensure you're a member of the organization
3. **Check workspace access**: Verify you have access to the workspace

## Best Practices

### 1. Use Environment Variables for Default Scoping

Set organization/workspace IDs in your environment for consistent scoping:

```bash
# In ~/.bashrc or ~/.zshrc
export LANGSMITH_ORGANIZATION_ID="your-org-id"
export LANGSMITH_WORKSPACE_ID="your-workspace-id"
```

### 2. Use CLI Flags for One-Off Operations

Override scoping temporarily when needed:

```bash
# Usually working in workspace A
export LANGSMITH_WORKSPACE_ID="workspace-a-id"

# Temporarily access workspace B
langstar prompt list --workspace-id "workspace-b-id"
```

### 3. Use Config File for Team Consistency

Share config file templates with your team:

```toml
# team-config.toml
organization_id = "team-org-id"
workspace_id = "team-workspace-id"
```

### 4. Be Explicit About Public Prompts

When scoped, always use `--public` flag to clarify intent:

```bash
# Clear: Looking for public prompts in this org
langstar prompt list --organization-id "langchain-ai" --public
```

### 5. Document Your IDs

Keep organization and workspace IDs documented for your team:

```markdown
# Team LangSmith Configuration

- Organization ID: `d1e1dfff-39bf-4cea-9a2e-85e970ce40ef`
- Production Workspace ID: `6f52dd84-9870-4f3a-b42d-4eea5fc9dfde`
- Staging Workspace ID: `abc123...`
```

## Integration with CI/CD

### GitHub Actions

Set secrets in your repository:

```yaml
- name: List team prompts
  run: langstar prompt list
  env:
    LANGSMITH_API_KEY: ${{ secrets.LANGSMITH_API_KEY }}
    LANGSMITH_ORGANIZATION_ID: ${{ secrets.LANGSMITH_ORGANIZATION_ID }}
    LANGSMITH_WORKSPACE_ID: ${{ secrets.LANGSMITH_WORKSPACE_ID }}
```

### Docker

Pass environment variables to containers:

```bash
docker run -e LANGSMITH_API_KEY \
           -e LANGSMITH_ORGANIZATION_ID \
           -e LANGSMITH_WORKSPACE_ID \
           langstar prompt list
```

### Shell Scripts

Use variables for flexibility:

```bash
#!/bin/bash
ORG_ID="${LANGSMITH_ORGANIZATION_ID:-default-org-id}"
WORKSPACE_ID="${LANGSMITH_WORKSPACE_ID:-default-workspace-id}"

langstar prompt list \
  --organization-id "$ORG_ID" \
  --workspace-id "$WORKSPACE_ID" \
  --format json
```

## Related Documentation

- [Configuration Guide](../dev/README.md)
- [API Documentation](../../README.md#usage)
- [LangSmith Organizations](https://docs.smith.langchain.com/)

## Summary

**Key Takeaways:**

1. Organization and workspace scoping control access to team prompts
2. When scoped, operations default to **private prompts only**
3. Use `--public` flag to explicitly access public prompts when scoped
4. CLI flags > config file > environment variables (precedence order)
5. Workspace scoping is narrower than organization scoping

**Quick Reference:**

```bash
# List private prompts in organization
langstar prompt list --organization-id "org-id"

# List public prompts in organization
langstar prompt list --organization-id "org-id" --public

# List prompts in workspace (narrower scope)
langstar prompt list --workspace-id "workspace-id"

# Search within workspace
langstar prompt search "query" --workspace-id "workspace-id"

# Get private prompt (requires scoping)
langstar prompt get "team/prompt" --organization-id "org-id"
```
