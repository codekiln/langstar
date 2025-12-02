# Model Configuration Management

Manage LangSmith model provider configurations (playground settings) for use in the Prompt Hub playground.

## Overview

Model configurations store model provider settings including:
- Model selection (e.g., `claude-3-5-sonnet-20241022`, `gpt-4-turbo`)
- Model parameters (temperature, max_tokens, etc.)
- API key references (stored as secrets)
- Rate limiting options

These configurations are used when testing prompts in the LangSmith Prompt Hub playground.

## Environment Variables

### Required

- `LANGSMITH_API_KEY` - Your LangSmith API key ([get one here](https://smith.langchain.com))

### Optional (for organization/workspace scoping)

- `LANGSMITH_ORGANIZATION_ID` - Scope operations to a specific organization
- `LANGSMITH_WORKSPACE_ID` - Scope operations to a specific workspace

### Example Setup

```bash
# Minimal setup (personal workspace)
export LANGSMITH_API_KEY="<your-api-key>"

# With workspace scoping (team workspace)
export LANGSMITH_API_KEY="<your-api-key>"
export LANGSMITH_WORKSPACE_ID="<your-workspace-id>"
```

## JSON Configuration Format

Model configurations follow the LangChain serialization format (LC-JSON):

```json
{
  "name": "Configuration Name",
  "description": "Optional description",
  "settings": {
    "lc": 1,
    "type": "constructor",
    "id": ["langchain", "chat_models", "<provider>", "<class>"],
    "kwargs": {
      "model": "model-name",
      "temperature": 0.0,
      "<provider>_api_key": {
        "lc": 1,
        "type": "secret",
        "id": ["SECRET_NAME"]
      }
    }
  },
  "options": {
    "requests_per_second": 10
  }
}
```

### Field Descriptions

- `name` (optional) - Human-readable name for the configuration
- `description` (optional) - Description of the configuration's purpose
- `settings` (required) - LangChain-serialized model configuration
  - `lc` - LangChain format version (always 1)
  - `type` - Constructor type (always "constructor")
  - `id` - Array identifying the LangChain class: `[package, module, provider, class]`
  - `kwargs` - Model-specific parameters
- `options` (optional) - Additional options
  - `requests_per_second` - Rate limit for batch operations

### Secret References

API keys are referenced by name, not stored directly:

```json
"<provider>_api_key": {
  "lc": 1,
  "type": "secret",
  "id": ["SECRET_NAME"]
}
```

The secret must exist in LangSmith workspace secrets. See workspace secrets management documentation for details.

## Commands

### List Configurations

List all model configurations in your workspace:

```bash
# List with default pagination (20 items)
langstar model-config list

# List with custom pagination
langstar model-config list --limit 50 --offset 100

# JSON output for scripting
langstar model-config list --format json
```

**Example Output (Table):**

```
ID                                   NAME                PROVIDER    MODEL
550e8400-e29b-41d4-a716-446655440000 Claude 3.5 Sonnet   anthropic   claude-3-5-sonnet-20241022
660e8400-e29b-41d4-a716-446655440001 GPT-4 Turbo         openai      gpt-4-turbo
770e8400-e29b-41d4-a716-446655440002 Bedrock Claude      bedrock     anthropic.claude-3-5-sonnet-20241022-v2:0
```

### Get Configuration

Get details of a specific configuration:

```bash
langstar model-config get <config-id>

# JSON output
langstar model-config get <config-id> --format json
```

### Create Configuration

Create a new configuration from a JSON file:

```bash
langstar model-config create --file config.json
```

See [Provider Examples](#provider-examples) below for complete configuration files.

### Update Configuration

Update an existing configuration:

```bash
# Update from JSON file (full update)
langstar model-config update <config-id> --file updates.json

# Update only the name
langstar model-config update <config-id> --name "New Configuration Name"

# Update only the description
langstar model-config update <config-id> --description "Updated description"

# Update name and description together
langstar model-config update <config-id> \
  --name "New Name" \
  --description "New description"
```

**Note:** `--file` cannot be combined with `--name` or `--description`. Use either file-based updates (for settings changes) or flag-based updates (for metadata only).

### Delete Configuration

Delete a configuration:

```bash
# Interactive confirmation
langstar model-config delete <config-id>

# Skip confirmation
langstar model-config delete <config-id> --yes
```

## Provider Examples

### Anthropic Claude

Create a configuration for Anthropic's Claude models:

**File: `anthropic-config.json`**

```json
{
  "name": "Claude 3.5 Sonnet",
  "description": "Production configuration for Claude 3.5 Sonnet",
  "settings": {
    "lc": 1,
    "type": "constructor",
    "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
    "kwargs": {
      "model": "claude-3-5-sonnet-20241022",
      "temperature": 0.0,
      "max_tokens": 4096,
      "anthropic_api_key": {
        "lc": 1,
        "type": "secret",
        "id": ["ANTHROPIC_API_KEY"]
      }
    }
  },
  "options": {
    "requests_per_second": 10
  }
}
```

```bash
langstar model-config create --file anthropic-config.json
```

**Available Claude Models:**
- `claude-3-5-sonnet-20241022` - Latest Claude 3.5 Sonnet
- `claude-3-5-haiku-20241022` - Latest Claude 3.5 Haiku
- `claude-3-opus-20240229` - Claude 3 Opus

### OpenAI

Create a configuration for OpenAI models:

**File: `openai-config.json`**

```json
{
  "name": "GPT-4 Turbo",
  "description": "OpenAI GPT-4 Turbo configuration",
  "settings": {
    "lc": 1,
    "type": "constructor",
    "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
    "kwargs": {
      "model": "gpt-4-turbo",
      "temperature": 0.7,
      "max_tokens": 2048,
      "openai_api_key": {
        "lc": 1,
        "type": "secret",
        "id": ["OPENAI_API_KEY"]
      }
    }
  },
  "options": {
    "requests_per_second": 5
  }
}
```

```bash
langstar model-config create --file openai-config.json
```

**Available OpenAI Models:**
- `gpt-4-turbo` - GPT-4 Turbo
- `gpt-4` - GPT-4
- `gpt-3.5-turbo` - GPT-3.5 Turbo

### Azure OpenAI

Create a configuration for Azure OpenAI service:

**File: `azure-openai-config.json`**

```json
{
  "name": "Azure GPT-4",
  "description": "Azure OpenAI GPT-4 deployment",
  "settings": {
    "lc": 1,
    "type": "constructor",
    "id": ["langchain", "chat_models", "azure_openai", "AzureChatOpenAI"],
    "kwargs": {
      "deployment_name": "gpt-4-deployment",
      "model": "gpt-4",
      "temperature": 0.5,
      "azure_endpoint": "https://<resource-name>.openai.azure.com/",
      "api_version": "2024-02-15-preview",
      "azure_ad_token": {
        "lc": 1,
        "type": "secret",
        "id": ["AZURE_OPENAI_API_KEY"]
      }
    }
  },
  "options": {
    "requests_per_second": 5
  }
}
```

```bash
langstar model-config create --file azure-openai-config.json
```

**Note:** Replace `<resource-name>` with your Azure OpenAI resource name.

### AWS Bedrock

Create a configuration for AWS Bedrock models:

**File: `bedrock-config.json`**

```json
{
  "name": "Bedrock Claude",
  "description": "AWS Bedrock Claude configuration",
  "settings": {
    "lc": 1,
    "type": "constructor",
    "id": ["langchain_aws", "chat_models", "bedrock_converse", "ChatBedrockConverse"],
    "kwargs": {
      "model": "anthropic.claude-3-5-sonnet-20241022-v2:0",
      "region_name": "us-east-1",
      "temperature": 0.0,
      "max_tokens": 4096
    }
  },
  "options": {
    "requests_per_second": 3
  }
}
```

```bash
langstar model-config create --file bedrock-config.json
```

**Note:** AWS credentials are managed through standard AWS authentication (IAM roles, environment variables, or AWS config files), not through LangSmith secrets.

**Available Bedrock Models:**
- `anthropic.claude-3-5-sonnet-20241022-v2:0` - Claude 3.5 Sonnet
- `anthropic.claude-3-haiku-20240307-v1:0` - Claude 3 Haiku
- `meta.llama3-70b-instruct-v1:0` - Llama 3 70B

## Example Workflows

### Workflow 1: Set Up Model Configs for Evaluation

Create multiple model configurations to compare performance:

```bash
# Create configs for different models
langstar model-config create --file claude-config.json
langstar model-config create --file gpt4-config.json
langstar model-config create --file bedrock-config.json

# List all configs to verify
langstar model-config list
```

### Workflow 2: Update Rate Limits

Adjust rate limits for a high-volume testing scenario:

**File: `update-rate-limit.json`**

```json
{
  "options": {
    "requests_per_second": 50
  }
}
```

```bash
langstar model-config update <config-id> --file update-rate-limit.json
```

### Workflow 3: Rename Configuration

Update a configuration name for better organization:

```bash
langstar model-config update <config-id> --name "Production Claude 3.5"
```

### Workflow 4: Clean Up Old Configurations

Remove unused configurations:

```bash
# List configs to find IDs
langstar model-config list

# Delete specific configs
langstar model-config delete <old-config-id> --yes
langstar model-config delete <test-config-id> --yes
```

### Workflow 5: Export/Backup Configurations

Export all configurations as JSON for backup:

```bash
langstar model-config list --format json > model-configs-backup.json
```

### Workflow 6: Migrate Configurations

Copy a configuration to a different workspace:

```bash
# Export from source workspace
export LANGSMITH_WORKSPACE_ID="<source-workspace-id>"
langstar model-config get <config-id> --format json > config.json

# Import to target workspace
export LANGSMITH_WORKSPACE_ID="<target-workspace-id>"
langstar model-config create --file config.json
```

**Note:** You'll need to manually adjust the JSON from the `get` output to match the `create` input format (remove `id`, `created_at`, `updated_at` fields).

## Tips and Best Practices

### Naming Conventions

Use descriptive names that indicate:
- Provider (Anthropic, OpenAI, etc.)
- Model (Claude 3.5, GPT-4, etc.)
- Purpose (Production, Testing, Evaluation)

**Examples:**
- `Production Claude 3.5 Sonnet`
- `Testing GPT-4 Turbo Low Temp`
- `Evaluation Bedrock Claude`

### Rate Limiting

Set appropriate `requests_per_second` based on:
- Provider rate limits
- Billing tier
- Use case (testing vs production)

**Common values:**
- `3-5` - Conservative for testing
- `10-20` - Moderate for evaluations
- `50+` - High volume (ensure provider supports it)

### Secret Management

- API keys are **referenced**, not stored in configurations
- Create workspace secrets before creating configs
- Use consistent secret names across configs (e.g., `ANTHROPIC_API_KEY`)
- See workspace secrets documentation for secret CRUD operations

### Configuration Updates

- Use `--file` for settings changes (model, temperature, etc.)
- Use `--name`/`--description` flags for metadata-only updates
- Can't mix file and flag updates (validation error)

### JSON Validation

Before creating a config:
1. Validate JSON syntax
2. Ensure required fields are present
3. Check secret references match existing secrets
4. Verify model names are correct for provider

```bash
# Validate JSON syntax
jq empty < config.json && echo "Valid JSON" || echo "Invalid JSON"
```

## Troubleshooting

### Error: "Model configuration with ID X not found"

The `get` command searches through all configurations. If pagination is needed:
- Large workspaces may have many configs
- The get command automatically handles pagination
- If still not found, verify the ID is correct

### Error: "At least one of --file, --name, or --description must be provided"

The `update` command requires something to update:
- Use `--file` for full updates
- Use `--name` and/or `--description` for metadata updates
- Cannot use `--file` with `--name`/`--description`

### Error: Secret not found

Secrets must exist before being referenced in configs:
1. Create the secret in workspace secrets first
2. Then reference it in the model config
3. Secret names in configs must match exactly

### Provider-Specific Issues

**Anthropic:**
- Ensure `ANTHROPIC_API_KEY` secret exists
- Verify model name matches current model IDs

**OpenAI:**
- Ensure `OPENAI_API_KEY` secret exists
- Check model availability in your OpenAI account

**Azure OpenAI:**
- Replace `<resource-name>` with actual resource
- Verify deployment name matches Azure portal
- Check API version is supported

**Bedrock:**
- AWS credentials via IAM, not LangSmith secrets
- Ensure model access is enabled in Bedrock console
- Verify region has model availability

## API Reference

For SDK usage and implementation details, see:
- SDK types: `sdk/src/playground_settings.rs`
- CLI implementation: `cli/src/commands/model_config.rs`
- OpenAPI spec: `reference/openapi/langchain/langsmith/openapi.json`

## Related Documentation

- [Workspace Secrets Management](./workspace-secrets.md) (when available)
- [Prompt Hub Workflows](./prompt-hub.md) (when available)
- [LangSmith Scoping](./scoping.md)
