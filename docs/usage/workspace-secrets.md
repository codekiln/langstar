# Workspace Secrets Management

Securely manage sensitive credentials for LangSmith deployments and evaluations.

## Overview

Workspace secrets provide secure storage for sensitive values such as API keys, database credentials, and authentication tokens that can be referenced by LangSmith deployments and evaluations without exposing the actual values.

### Key Features

- **Secure Storage**: Secret values are encrypted at rest
- **Read-Only Keys**: List endpoint returns only key names, never values
- **Reference-Based**: Secrets are referenced by name in configurations
- **Workspace-Scoped**: Each workspace has its own isolated secret store
- **Permission-Based**: Write operations require `workspaces:manage` permission

### Security Model

The LangSmith secrets API follows a security-first design:

1. **Values are never returned** - The API only returns secret key names, never the actual values
2. **Write-only access** - Once set, secret values cannot be retrieved through the API
3. **Upsert pattern** - A single endpoint handles both create and update operations
4. **Delete via upsert** - Deleting a secret requires posting `{"key": "NAME", "value": null}`

## Environment Variables

### Required

- `LANGSMITH_API_KEY` - Your LangSmith API key with `workspaces:manage` permission ([get one here](https://smith.langchain.com))

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

## CLI Commands

### List Secrets

List all secret keys in your workspace (values are never displayed):

```bash
# List in table format
langstar secrets list

# List in JSON format
langstar secrets list --format json
```

**Example Output (table)**:
```
Fetching workspace secrets...
┌────────────────────┐
│ Key                │
├────────────────────┤
│ ANTHROPIC_API_KEY  │
│ OPENAI_API_KEY     │
│ DATABASE_PASSWORD  │
└────────────────────┘

Found 3 secrets
```

**Example Output (JSON)**:
```json
[
  {"key": "ANTHROPIC_API_KEY"},
  {"key": "OPENAI_API_KEY"},
  {"key": "DATABASE_PASSWORD"}
]
```

### Set (Create or Update) a Secret

The `set` command handles both creating new secrets and updating existing ones.

#### Secure Input Methods

**CRITICAL SECURITY NOTE**: Secret values must NEVER appear in command arguments, shell history, or process listings. The CLI provides four secure methods for providing secret values:

#### Method 1: Interactive Prompt (Recommended for Manual Use)

Most secure for manual operations - prompts for the value with masked input:

```bash
langstar secrets set ANTHROPIC_API_KEY --interactive
# Prompts: Enter secret value for 'ANTHROPIC_API_KEY': ********
```

**When to use**:
- Manual one-time setup
- Testing and development
- When pasting from password managers

#### Method 2: File Input (Recommended for Automation)

Read secret value from a file:

```bash
# Create file with secret value
cat > /tmp/secret.txt <<'EOF'
sk-ant-api03-your-key-here
EOF

# Set secret from file
langstar secrets set ANTHROPIC_API_KEY --from-file /tmp/secret.txt

# Clean up
rm /tmp/secret.txt
```

**When to use**:
- CI/CD pipelines
- Automated deployment scripts
- Secrets management integration

**Security tip**: Use secure temporary files with restricted permissions:
```bash
touch /tmp/secret.txt
chmod 600 /tmp/secret.txt  # Owner read/write only
echo "your-secret-value" > /tmp/secret.txt
langstar secrets set API_KEY --from-file /tmp/secret.txt
shred -u /tmp/secret.txt  # Securely delete
```

#### Method 3: Environment Variable

Read secret value from an environment variable:

```bash
# Set secret from environment variable (do not expose in history)
read -s MY_SECRET  # Prompts without echo
export MY_SECRET
langstar secrets set OPENAI_API_KEY --from-env MY_SECRET

# Or from a password manager
export MY_SECRET="$(pass show openai/api-key)"
langstar secrets set OPENAI_API_KEY --from-env MY_SECRET
```

**When to use**:
- Integration with password managers
- Shell scripts that already have secrets in environment
- Docker/container environments

#### Method 4: Standard Input (Pipe)

Read secret value from stdin (default when no other method is specified):

```bash
# Pipe from file
cat /tmp/secret.txt | langstar secrets set DATABASE_PASSWORD

# Pipe from command
echo "your-secret" | langstar secrets set DATABASE_PASSWORD

# Pipe from password manager
pass show database/prod | langstar secrets set DATABASE_PASSWORD

# Pipe from environment variable
echo "$MY_SECRET" | langstar secrets set API_KEY
```

**When to use**:
- Shell script pipelines
- Integration with existing tooling
- One-liner automation commands

**Anti-pattern** (DO NOT DO THIS):
```bash
# ❌ WRONG: Exposes secret in shell history and process list
langstar secrets set API_KEY --value "sk-ant-..."  # Flag doesn't exist (security)
langstar secrets set API_KEY "sk-ant-..."          # Argument doesn't exist (security)
```

### Delete a Secret

Remove a secret from the workspace:

```bash
langstar secrets delete ANTHROPIC_API_KEY
```

**Example Output**:
```
Deleting secret 'ANTHROPIC_API_KEY'...
✓ Secret 'ANTHROPIC_API_KEY' deleted successfully
```

## SDK Usage

### Setup

Add `langstar-sdk` to your `Cargo.toml`:

```toml
[dependencies]
langstar-sdk = "0.11"  # or latest version
```

### List Secret Keys

```rust
use langstar_sdk::{AuthConfig, LangchainClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = AuthConfig::from_env()?;
    let client = LangchainClient::new(auth)?;

    // List all secret keys (values are never returned)
    let keys = client.list_workspace_secrets().await?;

    for key in keys {
        println!("Secret key: {}", key.key);
    }

    Ok(())
}
```

### Create or Update Secrets

```rust
use langstar_sdk::{AuthConfig, LangchainClient};
use langstar_sdk::secrets::SecretUpsert;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = AuthConfig::from_env()?;
    let client = LangchainClient::new(auth)?;

    // Single secret
    let secrets = vec![
        SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-..."),
    ];
    client.upsert_workspace_secrets(secrets).await?;

    // Batch upsert (more efficient for multiple secrets)
    let secrets = vec![
        SecretUpsert::set("ANTHROPIC_API_KEY", "sk-ant-..."),
        SecretUpsert::set("OPENAI_API_KEY", "sk-..."),
        SecretUpsert::set("DATABASE_URL", "postgresql://..."),
    ];
    client.upsert_workspace_secrets(secrets).await?;

    println!("Secrets updated successfully");
    Ok(())
}
```

### Delete a Secret

```rust
use langstar_sdk::{AuthConfig, LangchainClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = AuthConfig::from_env()?;
    let client = LangchainClient::new(auth)?;

    // Option 1: Convenience method (recommended)
    client.delete_workspace_secret("OLD_API_KEY").await?;

    // Option 2: Using upsert with null value (lower-level)
    use langstar_sdk::secrets::SecretUpsert;
    let secrets = vec![SecretUpsert::delete("OLD_API_KEY")];
    client.upsert_workspace_secrets(secrets).await?;

    println!("Secret deleted successfully");
    Ok(())
}
```

### Error Handling

```rust
use langstar_sdk::{AuthConfig, LangchainClient, LangstarError};
use langstar_sdk::secrets::SecretUpsert;

#[tokio::main]
async fn main() {
    let auth = AuthConfig::from_env().expect("Failed to load auth config");
    let client = LangchainClient::new(auth).expect("Failed to create client");

    let secrets = vec![SecretUpsert::set("API_KEY", "secret-value")];

    match client.upsert_workspace_secrets(secrets).await {
        Ok(()) => println!("✓ Secrets updated successfully"),
        Err(LangstarError::ApiError { status: 403, .. }) => {
            eprintln!("❌ Permission denied: API key needs 'workspaces:manage' permission");
        }
        Err(LangstarError::ApiError { status: 400, message, .. }) => {
            eprintln!("❌ Bad request: {}", message);
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }
}
```

## Security Best Practices

### General Guidelines

1. **Never log secret values** - Only log key names, never the actual values
2. **Use restrictive permissions** - Ensure API keys have minimal required permissions
3. **Rotate secrets regularly** - Implement a secret rotation policy
4. **Audit secret access** - Monitor which deployments/evaluations use which secrets
5. **Use workspace scoping** - Isolate secrets by workspace for multi-tenant scenarios

### CLI-Specific Security

#### ✅ Safe Practices

```bash
# Interactive prompt (safest for manual use)
langstar secrets set API_KEY --interactive

# File with restricted permissions
touch /tmp/secret && chmod 600 /tmp/secret
echo "value" > /tmp/secret
langstar secrets set API_KEY --from-file /tmp/secret
shred -u /tmp/secret

# Environment variable from password manager
export SECRET="$(pass show my/secret)"
langstar secrets set API_KEY --from-env SECRET

# Piped from secure source
pass show my/secret | langstar secrets set API_KEY
```

#### ❌ Anti-Patterns (DO NOT DO)

```bash
# ❌ WRONG: Secret appears in shell history
langstar secrets set API_KEY "sk-ant-secret"  # Not supported (by design)

# ❌ WRONG: Secret visible in process list
langstar secrets set API_KEY --value "sk-ant-secret"  # Flag doesn't exist

# ❌ WRONG: Secret logged to stdout
echo "sk-ant-secret" > /tmp/secret
cat /tmp/secret  # Visible in terminal
langstar secrets set API_KEY --from-file /tmp/secret

# ❌ WRONG: World-readable temporary file
echo "sk-ant-secret" > /tmp/secret  # Default permissions may be too open
langstar secrets set API_KEY --from-file /tmp/secret
```

### SDK-Specific Security

```rust
// ✅ GOOD: Load secrets from environment variables
let api_key = std::env::var("ANTHROPIC_API_KEY")
    .expect("ANTHROPIC_API_KEY not set");
client.upsert_workspace_secrets(vec![
    SecretUpsert::set("ANTHROPIC_API_KEY", api_key),
]).await?;

// ✅ GOOD: Read from secure file
let secret_value = std::fs::read_to_string("/run/secrets/api_key")?;
client.upsert_workspace_secrets(vec![
    SecretUpsert::set("API_KEY", secret_value.trim()),
]).await?;

// ❌ WRONG: Hardcoded secrets in source code
client.upsert_workspace_secrets(vec![
    SecretUpsert::set("API_KEY", "sk-ant-hardcoded-secret"),  // Never do this!
]).await?;

// ❌ WRONG: Logging secret values
let secret = "sk-ant-secret";
println!("Setting secret: {}", secret);  // Exposes in logs
client.upsert_workspace_secrets(vec![
    SecretUpsert::set("API_KEY", secret),
]).await?;
```

### LLM Agent Safety

When using workspace secrets in LLM agent contexts (like Claude Code skills):

#### ✅ Safe: Wrap in Skills/Functions

```bash
# .claude/skills/set-anthropic-key/skill.sh
#!/bin/bash
# Safely prompt for and set Anthropic API key
langstar secrets set ANTHROPIC_API_KEY --interactive
```

**Why safe**: The skill isolates the secret input from the LLM context. The LLM can call the skill but never sees the actual value.

#### ❌ Unsafe: Direct CLI Calls

```bash
# ❌ WRONG: LLM could suggest exposing secrets
# If LLM can directly execute CLI commands with values, it might:
echo "$ANTHROPIC_API_KEY" | langstar secrets set ANTHROPIC_API_KEY
# This could leak the secret into conversation context
```

#### Best Practice: Skill-Based Wrapper

Create a Claude Code skill that handles secret management:

```bash
# .claude/skills/manage-secrets/skill.sh
#!/bin/bash
set -euo pipefail

ACTION="${1:-list}"

case "$ACTION" in
    list)
        # Safe: only lists keys, not values
        langstar secrets list
        ;;
    set)
        KEY="${2:?Key name required}"
        # Safe: interactive prompt keeps value out of LLM context
        langstar secrets set "$KEY" --interactive
        ;;
    *)
        echo "Unknown action: $ACTION"
        exit 1
        ;;
esac
```

## Use Cases and Examples

### Use Case 1: Model Provider API Keys

Store API keys for model providers used in LangSmith deployments:

```bash
# Set provider API keys
langstar secrets set ANTHROPIC_API_KEY --interactive
langstar secrets set OPENAI_API_KEY --interactive
langstar secrets set COHERE_API_KEY --interactive

# List configured providers
langstar secrets list
```

Reference in model configuration (see `langstar model-config` documentation):

```json
{
  "name": "Claude Sonnet",
  "settings": {
    "lc": 1,
    "type": "constructor",
    "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
    "kwargs": {
      "model": "claude-3-5-sonnet-20241022",
      "anthropic_api_key": {
        "lc": 1,
        "type": "secret",
        "id": ["ANTHROPIC_API_KEY"]
      }
    }
  }
}
```

### Use Case 2: CI/CD Pipeline

Automate secret management in deployment pipelines:

```bash
#!/bin/bash
# deploy-secrets.sh
set -euo pipefail

# Load from secure CI/CD secrets storage
ANTHROPIC_KEY="${CI_ANTHROPIC_API_KEY:?Required CI variable not set}"
OPENAI_KEY="${CI_OPENAI_API_KEY:?Required CI variable not set}"

# Write to temporary files with restricted permissions
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "$ANTHROPIC_KEY" > "$TEMP_DIR/anthropic"
echo "$OPENAI_KEY" > "$TEMP_DIR/openai"
chmod 600 "$TEMP_DIR"/*

# Upload to LangSmith workspace
langstar secrets set ANTHROPIC_API_KEY --from-file "$TEMP_DIR/anthropic"
langstar secrets set OPENAI_API_KEY --from-file "$TEMP_DIR/openai"

echo "✓ Secrets deployed successfully"
```

### Use Case 3: Multi-Environment Setup

Manage secrets across different environments:

```bash
#!/bin/bash
# setup-environment.sh
ENVIRONMENT="${1:?Environment required (dev/staging/prod)}"

case "$ENVIRONMENT" in
    dev)
        export LANGSMITH_WORKSPACE_ID="<dev-workspace-id>"
        langstar secrets set ANTHROPIC_API_KEY --from-env DEV_ANTHROPIC_KEY
        ;;
    staging)
        export LANGSMITH_WORKSPACE_ID="<staging-workspace-id>"
        langstar secrets set ANTHROPIC_API_KEY --from-env STAGING_ANTHROPIC_KEY
        ;;
    prod)
        export LANGSMITH_WORKSPACE_ID="<prod-workspace-id>"
        langstar secrets set ANTHROPIC_API_KEY --from-env PROD_ANTHROPIC_KEY
        ;;
    *)
        echo "Unknown environment: $ENVIRONMENT"
        exit 1
        ;;
esac
```

### Use Case 4: Secret Rotation

Rotate secrets across multiple workspaces:

```rust
use langstar_sdk::{AuthConfig, LangchainClient};
use langstar_sdk::secrets::SecretUpsert;

async fn rotate_secret(
    workspace_ids: Vec<String>,
    key_name: &str,
    new_value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for workspace_id in workspace_ids {
        let auth = AuthConfig::from_env()?;
        let client = LangchainClient::new(auth)?
            .with_workspace_id(workspace_id.clone());

        println!("Rotating secret '{}' in workspace {}", key_name, workspace_id);

        let secrets = vec![SecretUpsert::set(key_name, new_value)];
        client.upsert_workspace_secrets(secrets).await?;

        println!("✓ Rotated in workspace {}", workspace_id);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspaces = vec![
        "<dev-workspace-id>".to_string(),
        "<staging-workspace-id>".to_string(),
        "<prod-workspace-id>".to_string(),
    ];

    // Get new secret value from secure source
    let new_key = std::env::var("NEW_ANTHROPIC_KEY")?;

    rotate_secret(workspaces, "ANTHROPIC_API_KEY", &new_key).await?;

    println!("✓ Secret rotation completed across all workspaces");
    Ok(())
}
```

## API Reference

### Endpoints

- **List**: `GET /api/v1/workspaces/current/secrets`
  - Returns array of `{"key": "NAME"}` objects
  - Never returns secret values
  - Requires valid API key

- **Upsert**: `POST /api/v1/workspaces/current/secrets`
  - Request body: `[{"key": "NAME", "value": "VALUE"}, ...]`
  - Creates new secrets or updates existing ones
  - Requires `workspaces:manage` permission
  - Returns 200 on success (empty response body)

- **Delete**: `POST /api/v1/workspaces/current/secrets`
  - Request body: `[{"key": "NAME", "value": null}]`
  - Setting value to `null` deletes the secret
  - Requires `workspaces:manage` permission

For detailed API specifications, see the [OpenAPI spec](https://api.smith.langchain.com/openapi.json).

### SDK Types

#### `SecretKey`

Represents a secret key returned from the list endpoint.

```rust
pub struct SecretKey {
    pub key: String,
}
```

**Fields**:
- `key`: The secret key name (e.g., "ANTHROPIC_API_KEY")

**Note**: Values are never included in this type (API never returns values).

#### `SecretUpsert`

Request type for creating, updating, or deleting secrets.

```rust
pub struct SecretUpsert {
    pub key: String,
    pub value: Option<String>,
}
```

**Fields**:
- `key`: The secret key name
- `value`:
  - `Some(value)`: Create or update the secret with this value
  - `None`: Delete the secret (serializes as explicit `null` in JSON)

**Constructor Methods**:
- `SecretUpsert::set(key, value)`: Create/update a secret
- `SecretUpsert::delete(key)`: Delete a secret

### Client Methods

See `sdk/src/client.rs`:

- `client.list_workspace_secrets() -> Result<Vec<SecretKey>>`
- `client.upsert_workspace_secrets(secrets: Vec<SecretUpsert>) -> Result<()>`
- `client.delete_workspace_secret(key: impl Into<String>) -> Result<()>`

## Troubleshooting

### Permission Denied (403)

**Error**: `Permission denied` or `HTTP 403`

**Cause**: Your API key doesn't have `workspaces:manage` permission.

**Solution**:
1. Go to [LangSmith Settings](https://smith.langchain.com)
2. Generate a new API key with `workspaces:manage` permission
3. Update `LANGSMITH_API_KEY` environment variable

### Secret Not Found After Setting

**Symptom**: Set a secret successfully but it doesn't appear in the list.

**Possible Causes**:
1. **Wrong workspace**: You're setting in one workspace but listing from another
2. **Wrong organization**: Organization ID scoping mismatch

**Solution**:
```bash
# Verify your workspace/organization configuration
echo "Workspace: $LANGSMITH_WORKSPACE_ID"
echo "Organization: $LANGSMITH_ORGANIZATION_ID"

# List secrets to confirm
langstar secrets list
```

### Empty Value Error

**Error**: `Secret value cannot be empty`

**Cause**: The provided secret value is empty or only whitespace.

**Solution**: Ensure your secret source contains a non-empty value:
```bash
# Check file contents before using
cat /tmp/secret.txt | wc -c  # Should be > 0

# Verify environment variable is set
echo "${MY_SECRET:?Variable is not set or empty}"

# For stdin, ensure input is provided
echo "actual-secret-value" | langstar secrets set KEY_NAME
```

### File Not Found

**Error**: `Failed to read secret file: No such file or directory`

**Cause**: The specified file doesn't exist or path is incorrect.

**Solution**:
```bash
# Verify file exists and is readable
ls -la /path/to/secret.txt

# Use absolute paths to avoid confusion
langstar secrets set API_KEY --from-file "$(pwd)/secret.txt"
```

## Related Documentation

- [Model Configuration Management](./model-config.md) - Using secrets in model provider configurations
- [Scoping (Organizations & Workspaces)](./scoping.md) - Understanding workspace and organization scoping
- [SDK Documentation](../sdk/) - Rust SDK API reference
- [LangSmith API Documentation](https://docs.smith.langchain.com/) - Official LangSmith documentation

## API Specification

For the complete OpenAPI specification, see:
- Production: https://api.smith.langchain.com/openapi.json
- Implementation: `docs/implementation/484.2-ls-secrets-design.md`
- Validation: `docs/implementation/490-ls-secrets-openapi-validation.md`
