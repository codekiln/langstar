# LangGraph Assistant Workflows

This guide demonstrates common workflows for managing LangGraph assistants using Langstar. Unlike LangSmith prompts, assistants are **deployment-level resources** with simpler configuration.

## Table of Contents

- [Understanding Assistants](#understanding-assistants)
- [Basic Assistant Management](#basic-assistant-management)
- [Creating Assistants](#creating-assistants)
- [Updating Assistants](#updating-assistants)
- [Searching and Filtering](#searching-and-filtering)
- [Managing Multiple Deployments](#managing-multiple-deployments)
- [Configuration Management](#configuration-management)

## Understanding Assistants

### Key Concepts

**Deployment-Level Resources:**
- Each assistant belongs to a specific LangGraph deployment
- Your API key determines which deployment you're accessing
- No organization or workspace scoping needed

**Differences from Prompts:**
- ❌ No `--organization-id` flags
- ❌ No `--workspace-id` flags
- ✅ Simple: Just API key + assistant commands
- ✅ Automatic scoping based on API key

### How It Works

```
User → API Key → Deployment → Assistants
```

Your `LANGSMITH_API_KEY` is tied to a specific deployment, so all assistant operations automatically scope to that deployment.

## Basic Assistant Management

### Configuration

Minimal setup - just your API key:

```bash
export LANGSMITH_API_KEY="<your-api-key>"
```

Or in `~/.langstar/config.toml`:

```toml
[langstar]
langsmith_api_key = "<your-api-key>"
```

**That's it!** No additional configuration needed.

### Workflow

#### List All Assistants

```bash
# List all assistants in your deployment
langstar assistant list
```

**Output example:**
```
ID                Name                    Graph ID            Created
asst_abc123...    Customer Support Bot    graph_xyz789...     2024-01-15
asst_def456...    Sales Assistant         graph_uvw012...     2024-01-14
asst_ghi789...    Research Helper         graph_rst345...     2024-01-13
```

#### List with Pagination

```bash
# First page (10 results)
langstar assistant list --limit 10

# Second page
langstar assistant list --limit 10 --offset 10

# Large page for complete view
langstar assistant list --limit 100
```

#### Get Assistant Details

```bash
# Get full details of an assistant
langstar assistant get asst_abc123
```

**Output example:**
```
ID: asst_abc123
Name: Customer Support Bot
Graph ID: graph_xyz789
Config:
  temperature: 0.7
  max_tokens: 1000
  model: gpt-4
Created: 2024-01-15T10:30:00Z
Updated: 2024-01-15T14:20:00Z
```

#### Search Assistants

```bash
# Search by name
langstar assistant search "customer"
```

**Output example:**
```
ID                Name                    Graph ID
asst_abc123...    Customer Support Bot    graph_xyz789...
asst_jkl012...    Customer Onboarding     graph_mno345...
```

#### JSON Output for Scripting

```bash
# List as JSON
langstar assistant list --format json > assistants.json

# Search as JSON
langstar assistant search "bot" --format json | jq '.[] | .name'

# Get specific assistant as JSON
langstar assistant get asst_abc123 --format json | jq '.config'
```

## Creating Assistants

### Simple Creation

```bash
# Create with minimal config
langstar assistant create \
  --graph-id graph_xyz789 \
  --name "My New Assistant"
```

**Output:**
```
Created assistant: asst_new123
Name: My New Assistant
Graph ID: graph_xyz789
```

### Creation with Configuration

#### Inline JSON Configuration

```bash
# Create with inline config
langstar assistant create \
  --graph-id graph_xyz789 \
  --name "Configured Bot" \
  --config '{"temperature": 0.7, "max_tokens": 2000}'
```

#### Configuration from File

Create `assistant-config.json`:

```json
{
  "temperature": 0.7,
  "max_tokens": 2000,
  "model": "gpt-4",
  "system_prompt": "You are a helpful customer support assistant."
}
```

```bash
# Create with file config
langstar assistant create \
  --graph-id graph_xyz789 \
  --name "File Config Bot" \
  --config-file ./assistant-config.json
```

### Creation Patterns

#### Pattern: Environment-Specific Assistants

```bash
# Development
langstar assistant create \
  --graph-id graph_dev \
  --name "Dev Test Bot" \
  --config '{"temperature": 1.0}'  # Higher temp for testing

# Production
langstar assistant create \
  --graph-id graph_prod \
  --name "Prod Support Bot" \
  --config '{"temperature": 0.3}'  # Lower temp for consistency
```

#### Pattern: Multiple Variants

```bash
# Conservative variant
langstar assistant create \
  --graph-id graph_abc \
  --name "Conservative Bot" \
  --config '{"temperature": 0.2}'

# Creative variant
langstar assistant create \
  --graph-id graph_abc \
  --name "Creative Bot" \
  --config '{"temperature": 0.9}'

# Balanced variant
langstar assistant create \
  --graph-id graph_abc \
  --name "Balanced Bot" \
  --config '{"temperature": 0.5}'
```

#### Pattern: Script-Based Creation

```bash
#!/bin/bash
# create-assistants.sh

GRAPH_ID="graph_xyz789"

# Create multiple assistants from array
declare -a assistants=(
  "Customer-Support:0.3"
  "Sales-Assistant:0.5"
  "Research-Helper:0.7"
)

for assistant in "${assistants[@]}"; do
  name="${assistant%%:*}"
  temp="${assistant##*:}"

  echo "Creating $name with temperature $temp..."
  langstar assistant create \
    --graph-id "$GRAPH_ID" \
    --name "$name" \
    --config "{\"temperature\": $temp}"
done
```

## Updating Assistants

### Update Name

```bash
# Update assistant name
langstar assistant update asst_abc123 --name "Updated Bot Name"
```

### Update Configuration

#### Inline Configuration Update

```bash
# Update config with inline JSON
langstar assistant update asst_abc123 \
  --config '{"temperature": 0.9, "max_tokens": 3000}'
```

#### Configuration Update from File

```bash
# Update from file
langstar assistant update asst_abc123 \
  --config-file ./new-config.json
```

### Update Both Name and Config

```bash
# Update name and config together
langstar assistant update asst_abc123 \
  --name "Improved Bot v2" \
  --config '{"temperature": 0.8}'
```

### Update Patterns

#### Pattern: Gradual Temperature Adjustment

```bash
#!/bin/bash
# adjust-temperature.sh

ASSISTANT_ID="asst_abc123"

# Start conservative
langstar assistant update "$ASSISTANT_ID" --config '{"temperature": 0.3}'
sleep 3600  # Test for 1 hour

# Increase slightly
langstar assistant update "$ASSISTANT_ID" --config '{"temperature": 0.5}'
sleep 3600

# Final adjustment
langstar assistant update "$ASSISTANT_ID" --config '{"temperature": 0.7}'
```

#### Pattern: A/B Testing Configuration

```bash
# Variant A (current)
VARIANT_A="asst_abc123"

# Variant B (new approach)
VARIANT_B=$(langstar assistant create \
  --graph-id graph_xyz \
  --name "Test Variant B" \
  --config '{"temperature": 0.9}' \
  --format json | jq -r '.id')

# Test both...
# Then update winner to production name
langstar assistant update "$VARIANT_B" --name "Production Bot v2"
```

## Searching and Filtering

### Basic Search

```bash
# Search by keyword
langstar assistant search "support"
```

### Search with Limits

```bash
# Limit search results
langstar assistant search "bot" --limit 5
```

### Scripted Filtering

#### Filter by Name Pattern

```bash
# Get all assistants, filter with jq
langstar assistant list --format json | \
  jq '.[] | select(.name | contains("Production"))'
```

#### Filter by Graph ID

```bash
# Find all assistants using specific graph
GRAPH_ID="graph_xyz789"
langstar assistant list --format json | \
  jq --arg gid "$GRAPH_ID" '.[] | select(.graph_id == $gid)'
```

#### Filter by Creation Date

```bash
# Assistants created in last 7 days
langstar assistant list --format json | \
  jq --arg date "$(date -d '7 days ago' -u +%Y-%m-%dT%H:%M:%SZ)" \
    '.[] | select(.created >= $date)'
```

### Export and Analysis

#### Export Assistant List

```bash
# Export all assistants
langstar assistant list --format json > assistants-$(date +%Y-%m-%d).json
```

#### Analyze Assistant Configurations

```bash
# Extract all temperature settings
langstar assistant list --format json | \
  jq -r '.[] | "\(.name): \(.config.temperature // "not set")"'

# Find assistants with high temperature
langstar assistant list --format json | \
  jq '.[] | select(.config.temperature > 0.8) | .name'
```

#### Count Assistants by Graph

```bash
# Group and count assistants by graph_id
langstar assistant list --format json | \
  jq -r '.[].graph_id' | sort | uniq -c
```

## Managing Multiple Deployments

### Understanding Multi-Deployment Scenarios

If you have multiple LangGraph deployments (dev, staging, prod), each has its own API key.

### Configuration Strategy

#### Approach 1: Environment Variables

```bash
# Development
export LANGSMITH_API_KEY="<dev-key>"
langstar assistant list

# Staging
export LANGSMITH_API_KEY="<staging-key>"
langstar assistant list

# Production
export LANGSMITH_API_KEY="<prod-key>"
langstar assistant list
```

#### Approach 2: Environment-Specific Scripts

```bash
#!/bin/bash
# dev-assistant.sh
export LANGSMITH_API_KEY="<dev-key>"
langstar assistant "$@"
```

```bash
#!/bin/bash
# prod-assistant.sh
export LANGSMITH_API_KEY="<prod-key>"
langstar assistant "$@"
```

Usage:

```bash
chmod +x dev-assistant.sh prod-assistant.sh

./dev-assistant.sh list
./prod-assistant.sh list
```

#### Approach 3: Environment Files

```bash
# dev.env
export LANGSMITH_API_KEY="<dev-key>"
export ENV_NAME="development"

# staging.env
export LANGSMITH_API_KEY="<staging-key>"
export ENV_NAME="staging"

# prod.env
export LANGSMITH_API_KEY="<prod-key>"
export ENV_NAME="production"
```

Usage:

```bash
# Load dev environment
source dev.env
echo "Environment: $ENV_NAME"
langstar assistant list

# Switch to production
source prod.env
echo "Environment: $ENV_NAME"
langstar assistant list
```

### Workflow: Promoting Assistants Across Deployments

```bash
#!/bin/bash
# promote-assistant.sh
# Copies an assistant from staging to production

STAGING_KEY="<staging-key>"
PROD_KEY="<prod-key>"
ASSISTANT_ID="$1"

if [ -z "$ASSISTANT_ID" ]; then
  echo "Usage: $0 <assistant-id>"
  exit 1
fi

# Get assistant details from staging
echo "Fetching assistant from staging..."
export LANGSMITH_API_KEY="$STAGING_KEY"
DETAILS=$(langstar assistant get "$ASSISTANT_ID" --format json)

NAME=$(echo "$DETAILS" | jq -r '.name')
GRAPH_ID=$(echo "$DETAILS" | jq -r '.graph_id')
CONFIG=$(echo "$DETAILS" | jq -c '.config')

# Create in production
echo "Creating in production..."
export LANGSMITH_API_KEY="$PROD_KEY"
langstar assistant create \
  --graph-id "$GRAPH_ID" \
  --name "$NAME" \
  --config "$CONFIG"

echo "Assistant promoted to production!"
```

Usage:

```bash
chmod +x promote-assistant.sh
./promote-assistant.sh asst_staging123
```

### Workflow: Comparing Deployments

```bash
#!/bin/bash
# compare-deployments.sh

# Export dev assistants
export LANGSMITH_API_KEY="<dev-key>"
langstar assistant list --format json > dev-assistants.json

# Export prod assistants
export LANGSMITH_API_KEY="<prod-key>"
langstar assistant list --format json > prod-assistants.json

# Compare
echo "Dev assistants:"
jq -r '.[].name' dev-assistants.json | sort

echo ""
echo "Prod assistants:"
jq -r '.[].name' prod-assistants.json | sort

echo ""
echo "Differences:"
diff <(jq -r '.[].name' dev-assistants.json | sort) \
     <(jq -r '.[].name' prod-assistants.json | sort)
```

## Configuration Management

### Storing Assistant Configurations

#### Pattern: Configuration Templates

Create reusable configuration templates:

```bash
# configs/conservative.json
{
  "temperature": 0.2,
  "max_tokens": 1000,
  "top_p": 0.9
}

# configs/balanced.json
{
  "temperature": 0.5,
  "max_tokens": 2000,
  "top_p": 0.95
}

# configs/creative.json
{
  "temperature": 0.9,
  "max_tokens": 3000,
  "top_p": 1.0
}
```

Usage:

```bash
# Create assistants with templates
langstar assistant create \
  --graph-id graph_abc \
  --name "Conservative Bot" \
  --config-file configs/conservative.json

langstar assistant create \
  --graph-id graph_abc \
  --name "Creative Bot" \
  --config-file configs/creative.json
```

#### Pattern: Versioned Configurations

```bash
# configs/v1/support-bot.json
{
  "temperature": 0.5,
  "max_tokens": 1500
}

# configs/v2/support-bot.json
{
  "temperature": 0.7,
  "max_tokens": 2000,
  "system_prompt": "Enhanced instructions..."
}
```

```bash
# Update to v2
langstar assistant update asst_abc123 \
  --config-file configs/v2/support-bot.json
```

### Backing Up Assistants

```bash
#!/bin/bash
# backup-assistants.sh

BACKUP_DIR="backups/$(date +%Y-%m-%d)"
mkdir -p "$BACKUP_DIR"

# Export assistant list
langstar assistant list --format json > "$BACKUP_DIR/assistants.json"

# Export each assistant's full details
for id in $(jq -r '.[].id' "$BACKUP_DIR/assistants.json"); do
  langstar assistant get "$id" --format json > "$BACKUP_DIR/$id.json"
done

echo "Backup complete: $BACKUP_DIR"
```

### Restoring Assistants

```bash
#!/bin/bash
# restore-assistant.sh

BACKUP_FILE="$1"

if [ ! -f "$BACKUP_FILE" ]; then
  echo "Usage: $0 <backup-json-file>"
  exit 1
fi

NAME=$(jq -r '.name' "$BACKUP_FILE")
GRAPH_ID=$(jq -r '.graph_id' "$BACKUP_FILE")
CONFIG=$(jq -c '.config' "$BACKUP_FILE")

langstar assistant create \
  --graph-id "$GRAPH_ID" \
  --name "$NAME (Restored)" \
  --config "$CONFIG"
```

## Deleting Assistants

### Basic Deletion

```bash
# Delete with confirmation prompt
langstar assistant delete asst_abc123
```

### Force Delete (Skip Confirmation)

```bash
# Delete without prompt
langstar assistant delete asst_abc123 --force
```

### Bulk Deletion

```bash
#!/bin/bash
# delete-old-assistants.sh

# Find assistants older than 30 days
OLD_ASSISTANTS=$(langstar assistant list --format json | \
  jq -r --arg date "$(date -d '30 days ago' -u +%Y-%m-%dT%H:%M:%SZ)" \
    '.[] | select(.created < $date) | .id')

for id in $OLD_ASSISTANTS; do
  echo "Deleting old assistant: $id"
  langstar assistant delete "$id" --force
done
```

### Safe Deletion with Backup

```bash
#!/bin/bash
# safe-delete.sh

ASSISTANT_ID="$1"

# Backup first
langstar assistant get "$ASSISTANT_ID" --format json > \
  "deleted-backup-$ASSISTANT_ID-$(date +%Y-%m-%d).json"

# Then delete
langstar assistant delete "$ASSISTANT_ID"
```

## Best Practices

### 1. Use Descriptive Names

```bash
# ❌ Bad
langstar assistant create --graph-id g1 --name "Bot1"

# ✅ Good
langstar assistant create \
  --graph-id graph_customer_support \
  --name "Customer Support Bot - Production v2.1"
```

### 2. Version Your Assistants

```bash
# Include version in name
langstar assistant create \
  --graph-id graph_abc \
  --name "Sales Assistant v1.3"
```

### 3. Store Configurations in Version Control

```bash
# configs/assistants/
├── customer-support.json
├── sales-assistant.json
└── research-helper.json
```

```bash
# Create from version-controlled config
langstar assistant create \
  --graph-id graph_abc \
  --name "Customer Support" \
  --config-file configs/assistants/customer-support.json
```

### 4. Tag or Prefix by Environment

```bash
# Development
langstar assistant create --name "[DEV] Test Bot" ...

# Staging
langstar assistant create --name "[STAGING] Test Bot" ...

# Production
langstar assistant create --name "[PROD] Customer Bot" ...
```

### 5. Document Assistant Purpose

Keep a registry:

```markdown
# assistants.md

## Active Assistants

- **asst_abc123**: Customer Support Bot
  - Graph: graph_support_v2
  - Purpose: Handle customer inquiries
  - Config: temperature=0.3 (conservative)

- **asst_def456**: Sales Assistant
  - Graph: graph_sales_v1
  - Purpose: Sales outreach and follow-ups
  - Config: temperature=0.7 (balanced)
```

## Troubleshooting

### "Authentication failed" errors

**Check API key:**

```bash
# Verify configuration
langstar config

# Ensure LANGSMITH_API_KEY is set
env | grep LANGSMITH_API_KEY
```

**Common mistake:**

```bash
# ❌ Wrong: Using LangSmith key for assistants
export LANGSMITH_API_KEY="<key>"
langstar assistant list  # Fails or uses wrong deployment

# ✅ Correct: Use LangGraph key
export LANGSMITH_API_KEY="<langgraph-key>"
langstar assistant list
```

### "No assistants found" but I have assistants

**Check deployment:**

Your `LANGSMITH_API_KEY` may be for a different deployment:

```bash
# Verify you're using the correct key
echo $LANGSMITH_API_KEY

# Try a different key if you have multiple deployments
export LANGSMITH_API_KEY="<different-deployment-key>"
langstar assistant list
```

### Assistant creation fails

**Verify graph ID:**

```bash
# Graph ID must exist in your deployment
langstar assistant create \
  --graph-id graph_nonexistent \
  --name "Test"
# Error: Graph not found

# Check available graphs (if you have graph listing capability)
# Or verify the graph ID with your deployment configuration
```

## Additional Resources

- [Configuration Guide](../configuration.md) - Complete configuration reference
- [README](../../README.md) - Quick start guide
- [LangGraph Cloud Documentation](https://langchain-ai.github.io/langgraph/cloud/) - Official LangGraph docs
- [Multi-Service Usage](./multi-service-usage.md) - Using assistants with prompts
- [Architecture Documentation](../architecture.md) - How assistants differ from prompts
