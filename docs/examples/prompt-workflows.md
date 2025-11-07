# LangSmith Prompt Workflows

This guide demonstrates common workflows for managing LangSmith prompts using Langstar. Prompts in LangSmith support organization and workspace scoping, making them ideal for team collaboration.

## Table of Contents

- [Personal Prompts](#personal-prompts)
- [Team Prompts (Organization-Scoped)](#team-prompts-organization-scoped)
- [Project Prompts (Workspace-Scoped)](#project-prompts-workspace-scoped)
- [Public vs Private Prompts](#public-vs-private-prompts)
- [Searching and Filtering](#searching-and-filtering)
- [Working with Multiple Organizations](#working-with-multiple-organizations)

## Personal Prompts

### Scenario

You're working on personal projects and want to manage your own prompts without organization scoping.

### Configuration

```bash
# Minimal setup - just your API key
export LANGSMITH_API_KEY="<your-api-key>"
```

Or in `~/.langstar/config.toml`:

```toml
[langstar]
langsmith_api_key = "<your-api-key>"
```

### Workflow

#### List All Your Prompts

```bash
# Lists all public prompts + your personal prompts
langstar prompt list
```

**Output example:**
```
Owner/Name                          Updated
alice/my-summarization-prompt       2024-01-15
alice/code-review-prompt            2024-01-14
public/general-qa-prompt            2024-01-10
bob/shared-research-prompt          2024-01-12
```

#### Get Prompt Details

```bash
# Get details of your prompt
langstar prompt get alice/my-summarization-prompt
```

**Output example:**
```
Name: my-summarization-prompt
Owner: alice
Description: Summarizes long documents with key points
Updated: 2024-01-15T10:30:00Z

Template:
Summarize the following document, highlighting key points:

{document}

Provide a bulleted list of main ideas.
```

#### Search Your Prompts

```bash
# Search across all accessible prompts
langstar prompt search "summarization"
```

**Output example:**
```
Owner/Name                          Matches
alice/my-summarization-prompt       1
public/text-summarization-v2        1
```

#### JSON Output for Scripting

```bash
# Get prompt list as JSON
langstar prompt list --format json > prompts.json

# Extract prompt names with jq
langstar prompt list --format json | jq '.[] | .name'
```

## Team Prompts (Organization-Scoped)

### Scenario

You're part of a team organization and want to work with shared prompts within your organization.

### Configuration

```bash
# Set both API key and organization ID
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_ORGANIZATION_ID="<org-id>"
```

Or in `~/.langstar/config.toml`:

```toml
[langstar]
langsmith_api_key = "<your-api-key>"
organization_id = "<org-id>"
organization_name = "My Team"  # Optional, informational
```

### Workflow

#### List Organization's Private Prompts

```bash
# When scoped, defaults to private prompts
langstar prompt list
```

**Output example:**
```
Owner/Name                          Updated
team/customer-support-v1            2024-01-15
team/sales-outreach                 2024-01-14
alice/draft-prompt                  2024-01-13
```

**Note:** Only prompts within your organization are shown.

#### List Organization's Public Prompts

```bash
# Use --public flag to access public prompts
langstar prompt list --public
```

**Output example:**
```
Owner/Name                          Updated
team/public-template-v1             2024-01-15
team/shared-qa-prompt               2024-01-12
```

#### Search Within Organization

```bash
# Search private prompts in org
langstar prompt search "customer"

# Search public prompts in org
langstar prompt search "customer" --public
```

#### Temporarily Override Organization

```bash
# Use different org for one command
langstar prompt list --organization-id "<different-org-id>"

# Or remove scoping for one command
unset LANGSMITH_ORGANIZATION_ID
langstar prompt list
```

## Project Prompts (Workspace-Scoped)

### Scenario

Your organization has multiple workspaces for different projects, and you want to work with prompts specific to one project.

### Configuration

```bash
# Set API key and workspace ID
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<workspace-id>"
```

Or in `~/.langstar/config.toml`:

```toml
[langstar]
langsmith_api_key = "<your-api-key>"
workspace_id = "<workspace-id>"
workspace_name = "Project Phoenix"  # Optional
```

**Note:** Workspace scoping is narrower than organization scoping. Setting a workspace ID automatically scopes to its parent organization.

### Workflow

#### List Workspace Prompts

```bash
# Lists private prompts in the workspace
langstar prompt list
```

**Output example:**
```
Owner/Name                          Updated
project/phoenix-summary-v1          2024-01-15
project/phoenix-qa-prompt           2024-01-14
alice/phoenix-experiment            2024-01-13
```

**Note:** Only prompts in this workspace are shown, not the entire organization.

#### Search Within Workspace

```bash
# Search within specific workspace
langstar prompt search "phoenix"
```

#### Override Workspace Temporarily

```bash
# Use different workspace for one command
langstar prompt list --workspace-id "<different-workspace-id>"

# Or search organization-wide
langstar prompt search "query" --organization-id "<org-id>"
```

## Public vs Private Prompts

### Understanding Public/Private Scoping

- **No scoping**: See all public prompts + your personal prompts
- **Organization scoping**: Defaults to private prompts in org
- **Workspace scoping**: Defaults to private prompts in workspace

### Workflow Examples

#### Scenario 1: Finding Public Templates

```bash
# No scoping - see all public prompts
export LANGSMITH_API_KEY="<key>"
unset LANGSMITH_ORGANIZATION_ID
unset LANGSMITH_WORKSPACE_ID

langstar prompt search "template"
```

#### Scenario 2: Team Collaboration with Private Prompts

```bash
# Organization-scoped - see team's private prompts
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_ORGANIZATION_ID="<org-id>"

langstar prompt list  # Private prompts
```

#### Scenario 3: Mixing Public and Private

```bash
# Search private prompts
export LANGSMITH_ORGANIZATION_ID="<org-id>"
langstar prompt search "customer"

# Search public prompts
langstar prompt search "customer" --public

# Compare public implementations
langstar prompt get public/customer-support-template
langstar prompt get team/customer-support-v1
```

## Searching and Filtering

### Search Examples

#### Basic Text Search

```bash
# Search by keyword
langstar prompt search "summarization"

# Search in scoped environment
export LANGSMITH_WORKSPACE_ID="<workspace-id>"
langstar prompt search "qa"
```

#### Limit Results

```bash
# Using head-style limiting (if supported)
langstar prompt list --format json | jq '.[:10]'  # First 10 results

# Search with grep for name filtering
langstar prompt list --format json | jq '.[] | select(.name | contains("prod"))'
```

#### Export Search Results

```bash
# Export all prompt names
langstar prompt list --format json | jq -r '.[] | .name' > prompt-list.txt

# Export prompts matching criteria
langstar prompt search "customer" --format json > customer-prompts.json

# Pretty-print specific prompt
langstar prompt get team/important-prompt --format json | jq '.'
```

### Filtering by Organization/Workspace

```bash
# Filter to specific organization
langstar prompt list --organization-id "<org-a-id>" --format json > org-a-prompts.json
langstar prompt list --organization-id "<org-b-id>" --format json > org-b-prompts.json

# Compare
diff <(jq -r '.[] | .name' org-a-prompts.json | sort) \
     <(jq -r '.[] | .name' org-b-prompts.json | sort)

# Filter to specific workspace
langstar prompt list --workspace-id "<workspace-id>"
```

## Working with Multiple Organizations

### Scenario

You're a consultant or work across multiple organizations and need to manage prompts in different contexts.

### Configuration

Create a base config with your API key:

```toml
# ~/.langstar/config.toml
[langstar]
langsmith_api_key = "<your-api-key>"
# No default organization - specify per command
```

### Workflow

#### List Prompts from Multiple Organizations

```bash
# Client A
langstar prompt list --organization-id "<client-a-org-id>" > client-a.txt

# Client B
langstar prompt list --organization-id "<client-b-org-id>" > client-b.txt

# Client C workspace
langstar prompt list --workspace-id "<client-c-workspace-id>" > client-c.txt
```

#### Create Scripts for Each Client

```bash
#!/bin/bash
# client-a-prompts.sh
export LANGSMITH_ORGANIZATION_ID="<client-a-org-id>"
langstar prompt "$@"
```

```bash
#!/bin/bash
# client-b-prompts.sh
export LANGSMITH_ORGANIZATION_ID="<client-b-org-id>"
langstar prompt "$@"
```

Usage:

```bash
chmod +x client-a-prompts.sh client-b-prompts.sh

./client-a-prompts.sh list
./client-b-prompts.sh search "customer"
```

#### Using Environment Modules

```bash
# client-a.env
export LANGSMITH_ORGANIZATION_ID="<client-a-org-id>"
export LANGSMITH_ORGANIZATION_NAME="Client A"

# client-b.env
export LANGSMITH_ORGANIZATION_ID="<client-b-org-id>"
export LANGSMITH_ORGANIZATION_NAME="Client B"
```

Usage:

```bash
# Load client A environment
source client-a.env
langstar prompt list

# Switch to client B
source client-b.env
langstar prompt list
```

#### Compare Prompts Across Organizations

```bash
#!/bin/bash
# compare-prompts.sh

# Get prompt lists from multiple orgs
langstar prompt list --organization-id "<org-1-id>" --format json > org1.json
langstar prompt list --organization-id "<org-2-id>" --format json > org2.json

# Find common prompt names
comm -12 \
  <(jq -r '.[] | .name' org1.json | sort) \
  <(jq -r '.[] | .name' org2.json | sort)
```

## Best Practices

### 1. Use Configuration Files for Persistent Settings

Instead of exporting environment variables every time:

```bash
# ~/.langstar/config.toml
[langstar]
langsmith_api_key = "<key>"
organization_id = "<most-used-org-id>"
```

Override when needed:

```bash
langstar prompt list  # Uses config org
langstar prompt list --organization-id "<other-org-id>"  # Override
```

### 2. Leverage JSON Output for Automation

```bash
# Script to check for outdated prompts (>30 days)
langstar prompt list --format json | jq -r '
  .[] |
  select(
    (now - (.updated | fromdate)) > (30 * 24 * 60 * 60)
  ) |
  .name
'
```

### 3. Document Your Scoping Strategy

Create a `PROMPTS.md` in your project:

```markdown
# Prompt Management

Our team uses LangSmith prompts scoped to workspaces:

- `langstar-dev` workspace: Development/testing prompts
- `langstar-staging` workspace: Staging prompts
- `langstar-prod` workspace: Production prompts

## Listing Prompts

```bash
# Development
langstar prompt list --workspace-id "<dev-workspace-id>"

# Production
langstar prompt list --workspace-id "<prod-workspace-id>"
```
```

### 4. Use Descriptive Names

```bash
# ❌ Bad
langstar prompt get team/prompt1
langstar prompt get team/p2

# ✅ Good
langstar prompt get team/customer-support-qa-v2
langstar prompt get team/product-description-generator
```

### 5. Search Before Creating

```bash
# Check if similar prompt exists
langstar prompt search "customer support"

# If found, examine it
langstar prompt get team/customer-support-qa-v1

# Then decide: reuse, modify, or create new
```

## Common Patterns

### Pattern: Environment-Based Prompts

```bash
# Different workspaces for different environments
case "$APP_ENV" in
  development)
    export LANGSMITH_WORKSPACE_ID="<dev-workspace-id>"
    ;;
  staging)
    export LANGSMITH_WORKSPACE_ID="<staging-workspace-id>"
    ;;
  production)
    export LANGSMITH_WORKSPACE_ID="<prod-workspace-id>"
    ;;
esac

langstar prompt list
```

### Pattern: Prompt Discovery

```bash
# Find prompts related to a feature
langstar prompt search "authentication" --format json | \
  jq -r '.[] | "\(.name): \(.description)"'
```

### Pattern: Audit Trail

```bash
# Export all organization prompts for audit
langstar prompt list --organization-id "<org-id>" --format json > \
  "prompts-$(date +%Y-%m-%d).json"
```

## Troubleshooting

### "No prompts found" but I have prompts

**Check scoping:**

```bash
langstar config
```

If scoped to organization/workspace, try:

```bash
# Remove scoping
unset LANGSMITH_ORGANIZATION_ID
unset LANGSMITH_WORKSPACE_ID
langstar prompt list

# Or use --public flag
export LANGSMITH_ORGANIZATION_ID="<org-id>"
langstar prompt list --public
```

### Can't access team prompts

**Verify organization ID:**

```bash
# Check your configuration
langstar config

# Try explicit organization ID
langstar prompt list --organization-id "<org-id>"
```

### Slow searches

**Use workspace scoping to narrow results:**

```bash
# Instead of organization-wide search
langstar prompt search "query" --organization-id "<org-id>"

# Use workspace for faster results
langstar prompt search "query" --workspace-id "<workspace-id>"
```

## Additional Resources

- [Configuration Guide](../configuration.md) - Complete configuration reference
- [README](../../README.md) - Quick start guide
- [LangSmith Documentation](https://docs.smith.langchain.com/) - Official LangSmith docs
- [Multi-Service Usage](./multi-service-usage.md) - Using prompts with assistants
