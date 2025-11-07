# Multi-Service Usage Guide

This guide demonstrates how to use both LangSmith prompts and LangGraph assistants together effectively, leveraging the strengths of each service.

## Table of Contents

- [Understanding the Services](#understanding-the-services)
- [Configuration for Both Services](#configuration-for-both-services)
- [When to Use Each Service](#when-to-use-each-service)
- [Complementary Workflows](#complementary-workflows)
- [Integration Patterns](#integration-patterns)
- [Best Practices](#best-practices)

## Understanding the Services

### LangSmith Prompts

**Purpose:** Prompt management and versioning
- **Scope:** Organization/workspace hierarchical model
- **Use for:** Storing, versioning, and sharing prompt templates
- **Key feature:** Multi-tenancy support

### LangGraph Assistants

**Purpose:** Graph-based agent deployment and execution
- **Scope:** Deployment-level resources
- **Use for:** Running deployed LangGraph applications
- **Key feature:** Simple, deployment-focused model

### Architectural Differences

| Aspect | LangSmith Prompts | LangGraph Assistants |
|--------|-------------------|----------------------|
| **Scoping** | Org/Workspace | Deployment |
| **Headers** | `x-organization-id`, `X-Tenant-Id` | `x-api-key` only |
| **Configuration** | Optional scoping IDs | API key only |
| **Multi-tenancy** | Yes | No |
| **Typical use** | Prompt development | Agent execution |

## Configuration for Both Services

### Minimal Setup

```bash
# Set both API keys
export LANGSMITH_API_KEY="<your-langsmith-key>"
export LANGSMITH_API_KEY="<your-langgraph-key>"
```

Or in `~/.langstar/config.toml`:

```toml
[langstar]
# LangSmith configuration
langsmith_api_key = "<your-langsmith-key>"

# LangGraph configuration
langsmith_api_key = "<your-langgraph-key>"
```

### Complete Setup with Scoping

```bash
# LangSmith with organization scoping
export LANGSMITH_API_KEY="<your-langsmith-key>"
export LANGSMITH_ORGANIZATION_ID="<org-id>"
export LANGSMITH_WORKSPACE_ID="<workspace-id>"

# LangGraph (no scoping needed)
export LANGSMITH_API_KEY="<your-langgraph-key>"
```

Or in `~/.langstar/config.toml`:

```toml
[langstar]
# LangSmith configuration (with scoping)
langsmith_api_key = "<your-langsmith-key>"
organization_id = "<org-id>"
workspace_id = "<workspace-id>"

# LangGraph configuration (simple)
langsmith_api_key = "<your-langgraph-key>"
```

### Verifying Configuration

```bash
langstar config
```

**Expected output:**

```
Configuration file: ~/.langstar/config.toml

LangSmith Configuration (for 'prompt' commands):
  API key: configured
  Organization ID: <org-id> (scopes prompt operations)
  Workspace ID: <workspace-id> (narrows scope further)
  → Prompt commands will use workspace-scoped resources

LangGraph Configuration (for 'assistant' commands):
  API key: configured
  → Assistant commands use deployment-level resources
  → No organization/workspace scoping available
```

## When to Use Each Service

### Use LangSmith Prompts When:

- ✅ Developing and iterating on prompts
- ✅ Sharing prompts across a team
- ✅ Versioning prompt templates
- ✅ Managing prompts across organizations/workspaces
- ✅ Need hierarchical access control

**Example scenarios:**
- Storing reusable prompt templates for your team
- Managing prompt versions across dev/staging/prod workspaces
- Collaborating on prompt engineering across an organization

### Use LangGraph Assistants When:

- ✅ Deploying graph-based agents
- ✅ Running LangGraph applications
- ✅ Managing deployed assistant configurations
- ✅ Need simple, deployment-level resources
- ✅ Executing conversational workflows

**Example scenarios:**
- Deploying a customer support chatbot
- Running a multi-step research assistant
- Managing production agent configurations

### Use Both Together When:

- ✅ Prompts from LangSmith feed into LangGraph agents
- ✅ Developing prompts (LangSmith) then deploying agents (LangGraph)
- ✅ Need both prompt management and agent execution

**Example scenarios:**
- Develop customer service prompts in LangSmith, deploy as LangGraph assistant
- Store prompt templates in LangSmith workspace, reference in deployed agents
- Iterate on prompts in one service, use in production agents in the other

## Complementary Workflows

### Workflow 1: Prompt Development → Agent Deployment

**Scenario:** Develop a customer service prompt in LangSmith, then deploy it as a LangGraph assistant.

**Steps:**

```bash
# 1. Develop prompt in LangSmith workspace
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_WORKSPACE_ID="<dev-workspace>"

langstar prompt list
langstar prompt get team/customer-service-v3

# 2. Test and iterate on prompt
# (Use LangSmith web UI or API to test)

# 3. Once finalized, deploy as LangGraph assistant
export LANGSMITH_API_KEY="<key>"

langstar assistant create \
  --graph-id graph_customer_service \
  --name "Customer Service Bot v3" \
  --config '{"prompt_template": "customer-service-v3"}'

# 4. Verify deployment
langstar assistant list
langstar assistant get <assistant-id>
```

### Workflow 2: Multi-Environment Prompt Management

**Scenario:** Manage prompts across dev/staging/prod workspaces, deploy corresponding assistants.

**Setup:**

```bash
# Environment-specific configurations
# dev.env
export LANGSMITH_WORKSPACE_ID="<dev-workspace-id>"
export LANGSMITH_API_KEY="<dev-deployment-key>"

# staging.env
export LANGSMITH_WORKSPACE_ID="<staging-workspace-id>"
export LANGSMITH_API_KEY="<staging-deployment-key>"

# prod.env
export LANGSMITH_WORKSPACE_ID="<prod-workspace-id>"
export LANGSMITH_API_KEY="<prod-deployment-key>"
```

**Workflow:**

```bash
# Development
source dev.env
langstar prompt list                      # Dev prompts
langstar assistant list                   # Dev assistants

# Promote to staging
source staging.env
langstar prompt list                      # Staging prompts
langstar assistant create --graph-id ... --name "Staging Bot"

# Promote to production
source prod.env
langstar prompt list                      # Prod prompts
langstar assistant create --graph-id ... --name "Production Bot"
```

### Workflow 3: Team Collaboration

**Scenario:** Team develops shared prompts in organization workspace, individual members deploy to their own assistants.

**Team Lead (Prompt Management):**

```bash
# Manage shared prompts
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_ORGANIZATION_ID="<org-id>"

langstar prompt list
langstar prompt get team/shared-template-v1
```

**Team Members (Assistant Deployment):**

```bash
# Each member has their own LangGraph deployment
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_ORGANIZATION_ID="<org-id>"
export LANGSMITH_API_KEY="<member-deployment-key>"

# Reference shared prompt
langstar prompt get team/shared-template-v1

# Deploy to personal assistant
langstar assistant create \
  --graph-id graph_personal \
  --name "My Assistant using Shared Template"
```

### Workflow 4: A/B Testing Across Services

**Scenario:** Test prompt variations in LangSmith, deploy best performers as LangGraph assistants.

```bash
#!/bin/bash
# ab-test-prompts.sh

# 1. List prompt variants in LangSmith
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_WORKSPACE_ID="<workspace>"

langstar prompt search "variant" --format json > variants.json

# 2. For each variant, create a test assistant in LangGraph
export LANGSMITH_API_KEY="<key>"

for variant in $(jq -r '.[].name' variants.json); do
  echo "Creating assistant for $variant..."
  langstar assistant create \
    --graph-id graph_test \
    --name "Test: $variant" \
    --config "{\"prompt\": \"$variant\"}"
done

# 3. Test assistants and measure performance
# (External testing/measurement)

# 4. Deploy winning variant to production
langstar assistant create \
  --graph-id graph_prod \
  --name "Production Bot - Best Variant"
```

## Integration Patterns

### Pattern 1: Prompt Library + Agent Deployment

**Structure:**

```
LangSmith (Prompt Library)
├── customer-service-greeting
├── customer-service-escalation
├── customer-service-closing
└── ...

LangGraph (Deployed Agents)
├── Customer Service Bot (uses all above prompts)
├── Sales Assistant (uses subset)
└── ...
```

**Implementation:**

```bash
# Store prompts in LangSmith
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_WORKSPACE_ID="<prompt-library>"

langstar prompt list | grep "customer-service"

# Deploy agent that references these prompts
export LANGSMITH_API_KEY="<key>"

langstar assistant create \
  --graph-id graph_customer_service \
  --name "Customer Service Bot" \
  --config-file configs/customer-bot.json
```

**Config file (`configs/customer-bot.json`):**

```json
{
  "prompts": {
    "greeting": "customer-service-greeting",
    "escalation": "customer-service-escalation",
    "closing": "customer-service-closing"
  },
  "temperature": 0.3
}
```

### Pattern 2: Environment-Specific Deployments

**Structure:**

```
LangSmith Workspaces          LangGraph Deployments
├── dev-workspace       →     dev-deployment
├── staging-workspace   →     staging-deployment
└── prod-workspace      →     prod-deployment
```

**Implementation:**

```bash
#!/bin/bash
# deploy-to-env.sh

ENV="$1"  # dev, staging, or prod

case "$ENV" in
  dev)
    export LANGSMITH_WORKSPACE_ID="<dev-workspace>"
    export LANGSMITH_API_KEY="<dev-deployment-key>"
    ;;
  staging)
    export LANGSMITH_WORKSPACE_ID="<staging-workspace>"
    export LANGSMITH_API_KEY="<staging-deployment-key>"
    ;;
  prod)
    export LANGSMITH_WORKSPACE_ID="<prod-workspace>"
    export LANGSMITH_API_KEY="<prod-deployment-key>"
    ;;
  *)
    echo "Usage: $0 {dev|staging|prod}"
    exit 1
    ;;
esac

echo "Deploying to $ENV..."
langstar prompt list
langstar assistant list
```

### Pattern 3: Shared Prompt Repository

**Use Case:** Multiple teams share a common prompt repository, each deploys to their own assistants.

**Setup:**

```bash
# Central prompt repository (organization-scoped)
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_ORGANIZATION_ID="<central-org-id>"

langstar prompt list --public  # Shared public prompts
```

**Team A Deployment:**

```bash
# Team A has their own deployment
export LANGSMITH_API_KEY="<team-a-key>"

langstar assistant create \
  --graph-id graph_team_a \
  --name "Team A Bot" \
  --config '{"shared_prompts": true}'
```

**Team B Deployment:**

```bash
# Team B has their own deployment
export LANGSMITH_API_KEY="<team-b-key>"

langstar assistant create \
  --graph-id graph_team_b \
  --name "Team B Bot" \
  --config '{"shared_prompts": true}'
```

## Best Practices

### 1. Separate API Keys

Use different API keys for LangSmith and LangGraph:

```bash
# ✅ Good: Separate keys
export LANGSMITH_API_KEY="<langsmith-specific-key>"
export LANGSMITH_API_KEY="<langgraph-specific-key>"

# ⚠️ Acceptable but not ideal: Same key for both
export LANGSMITH_API_KEY="<shared-key>"
# LangGraph falls back to LANGSMITH_API_KEY
```

**Benefits:**
- Better security (principle of least privilege)
- Easier key rotation
- Clear separation of concerns

### 2. Match Workspace to Deployment Environment

```bash
# Development
export LANGSMITH_WORKSPACE_ID="<dev-workspace>"
export LANGSMITH_API_KEY="<dev-deployment>"

# Production
export LANGSMITH_WORKSPACE_ID="<prod-workspace>"
export LANGSMITH_API_KEY="<prod-deployment>"
```

**Benefits:**
- Consistent environment behavior
- Easier promotion pipeline
- Reduced configuration errors

### 3. Document Prompt-to-Assistant Mappings

Keep a registry file:

```markdown
# services-mapping.md

## Prompt to Assistant Mappings

### Customer Service Bot (asst_abc123)
- Graph: graph_customer_service_v2
- Deployment: production
- Prompts used:
  - `team/customer-greeting-v3` (LangSmith)
  - `team/customer-escalation-v2` (LangSmith)
  - `team/customer-closing-v1` (LangSmith)

### Sales Assistant (asst_def456)
- Graph: graph_sales_v1
- Deployment: production
- Prompts used:
  - `team/sales-outreach-v4` (LangSmith)
  - `team/sales-followup-v2` (LangSmith)
```

### 4. Use Configuration Files for Complex Setups

```toml
# ~/.langstar/config.toml
[langstar]
# Default to production
langsmith_api_key = "<prod-langsmith-key>"
organization_id = "<prod-org>"
workspace_id = "<prod-workspace>"
langsmith_api_key = "<prod-deployment-key>"
```

Override for development:

```bash
# dev-override.env
export LANGSMITH_WORKSPACE_ID="<dev-workspace>"
export LANGSMITH_API_KEY="<dev-deployment-key>"

source dev-override.env
langstar prompt list       # Dev workspace
langstar assistant list    # Dev deployment
```

### 5. Script Common Multi-Service Operations

```bash
#!/bin/bash
# sync-prompt-to-assistant.sh
# Syncs a prompt from LangSmith to assistant config

PROMPT_NAME="$1"
ASSISTANT_ID="$2"

# Get prompt details from LangSmith
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_WORKSPACE_ID="<workspace>"

PROMPT_DETAILS=$(langstar prompt get "$PROMPT_NAME" --format json)
PROMPT_TEMPLATE=$(echo "$PROMPT_DETAILS" | jq -r '.template')

# Update assistant with new prompt
export LANGSMITH_API_KEY="<key>"

langstar assistant update "$ASSISTANT_ID" \
  --config "{\"prompt_template\": \"$PROMPT_TEMPLATE\"}"

echo "Synced prompt '$PROMPT_NAME' to assistant '$ASSISTANT_ID'"
```

## Common Pitfalls

### Pitfall 1: Wrong API Key for Service

❌ **Problem:**

```bash
# Trying to use LangSmith key for assistants
export LANGSMITH_API_KEY="<key>"
langstar assistant list  # May fail or use wrong deployment
```

✅ **Solution:**

```bash
# Use correct key for each service
export LANGSMITH_API_KEY="<langsmith-key>"
export LANGSMITH_API_KEY="<langgraph-key>"

langstar prompt list      # Uses LANGSMITH_API_KEY
langstar assistant list   # Uses LANGSMITH_API_KEY
```

### Pitfall 2: Scoping Confusion

❌ **Problem:**

```bash
# Trying to scope assistants (doesn't work)
export LANGSMITH_ORGANIZATION_ID="<org-id>"
langstar assistant list  # Organization ID is ignored!
```

✅ **Solution:**

```bash
# Only scope prompts, not assistants
export LANGSMITH_ORGANIZATION_ID="<org-id>"
langstar prompt list  # Scoped to organization

unset LANGSMITH_ORGANIZATION_ID
langstar assistant list  # No scoping for assistants
```

### Pitfall 3: Environment Mismatch

❌ **Problem:**

```bash
# Dev workspace but prod deployment
export LANGSMITH_WORKSPACE_ID="<dev-workspace>"
export LANGSMITH_API_KEY="<prod-deployment>"

# This creates confusion - dev prompts, prod assistants
```

✅ **Solution:**

```bash
# Match workspace to deployment
export LANGSMITH_WORKSPACE_ID="<dev-workspace>"
export LANGSMITH_API_KEY="<dev-deployment>"
```

## Troubleshooting Multi-Service Issues

### Issue: Commands using wrong API key

**Symptoms:**
- Assistant commands return unexpected results
- Authentication errors despite having API key set

**Solution:**

```bash
# Check which keys are configured
langstar config

# Verify environment
env | grep LANGSMITH_API_KEY
env | grep LANGSMITH_API_KEY

# Set explicit keys
export LANGSMITH_API_KEY="<langsmith-key>"
export LANGSMITH_API_KEY="<langgraph-key>"
```

### Issue: Can't find resources in one service

**Symptoms:**
- Prompts found in LangSmith but not reflected in assistants
- Assistants exist but prompts missing

**Remember:**
- These are **separate services**
- Prompts in LangSmith don't automatically appear in LangGraph
- Manual synchronization required

**Solution:**
- Document mappings between services
- Use scripts to sync configurations
- Keep registry of prompt-to-assistant relationships

### Issue: Configuration conflicts

**Symptoms:**
- Unexpected scoping behavior
- Resources from wrong environment

**Solution:**

```bash
# Clear all configuration
unset LANGSMITH_API_KEY
unset LANGSMITH_ORGANIZATION_ID
unset LANGSMITH_WORKSPACE_ID
unset LANGSMITH_API_KEY

# Set minimal configuration
export LANGSMITH_API_KEY="<key>"
export LANGSMITH_API_KEY="<key>"

# Test each service independently
langstar prompt list
langstar assistant list
```

## Additional Resources

- [Prompt Workflows](./prompt-workflows.md) - Detailed LangSmith prompt examples
- [Assistant Workflows](./assistant-workflows.md) - Detailed LangGraph assistant examples
- [Configuration Guide](../configuration.md) - Complete configuration reference
- [Architecture Documentation](../architecture.md) - Why services are designed differently
- [Troubleshooting Guide](../troubleshooting.md) - Common issues and solutions
