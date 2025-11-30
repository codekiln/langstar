# Model Providers Feature - Design Decisions

**Issue**: #472 (Phase 2 of #461)
**Date**: 2025-11-30
**Status**: Draft

## Executive Summary

This document defines the design decisions for implementing model provider configuration management in Langstar, ensuring consistency with existing CLI patterns and seamless integration with the existing configuration system.

**Key Decision**: Use command name `model-config` (not `model-provider`) for CLI consistency with API terminology.

## 1. DX Consistency Analysis

### 1.1 Existing Command Patterns

Analysis of `langstar prompt`, `langstar graph`, and `langstar eval` commands reveals these established patterns:

| Pattern | Implementation | Example |
|---------|---------------|---------|
| **Command structure** | clap Subcommand enum with variants for operations | `List`, `Get`, `Create`, `Update`, `Delete` |
| **Pagination** | `--limit` (u32, default 20), `--offset` (u32, default 0) | `langstar prompt list --limit 50 --offset 100` |
| **Scoping** | `--organization-id`, `--workspace-id` flags override config/env | `langstar prompt list --workspace-id <id>` |
| **Output format** | Global `-f/--format` flag (json\|table) | `langstar prompt list -f json` |
| **Table display** | Uses `tabled` crate with `Tabled` trait, rounded style | See `PromptRow`, `DeploymentRow` |
| **User feedback** | `OutputFormatter::info()`, `.success()`, `.warning()` | Writes to stderr in JSON mode |
| **Boolean flags** | Simple names without `enable-` prefix | `--public`, `--wait`, `--yes` |
| **Resource identifiers** | Positional args for primary ID, flags for optional params | `langstar prompt get <handle>` |
| **Confirmation** | `--yes` / `-y` flag to skip prompts on destructive ops | `langstar graph delete <id> --yes` |

**Reference locations:**
- `cli/src/commands/prompt.rs:15-132` - PromptCommands enum
- `cli/src/commands/graph.rs:13-92` - GraphCommands enum
- `cli/src/output.rs:52-144` - OutputFormatter implementation

### 1.2 Recommended Command Name

**Decision: Use `model-config`** (not `model-provider`)

**Rationale:**
1. API uses `/api/v1/playground-settings`, but "playground-settings" is too UI-centric
2. Data structure is called `PlaygroundSettingsResponse` but represents model configuration
3. CLI commands should be more semantic than API endpoint names
4. "Config" is shorter and more CLI-friendly than "configuration"

**Rejected alternatives:**
- `playground-settings` - Too tied to UI terminology
- `model-provider` - Ambiguous (could mean the provider itself, not configuration)
- `model` - Too generic, conflicts with potential future model management features

### 1.3 Proposed Command Structure

```rust
/// Commands for managing LangSmith model configurations
#[derive(Debug, Subcommand)]
pub enum ModelConfigCommands {
    /// List all model configurations
    List {
        #[arg(short, long, default_value = "20")]
        limit: u32,

        #[arg(short, long, default_value = "0")]
        offset: u32,

        /// Filter by provider (anthropic, openai, azure_openai, bedrock)
        #[arg(long)]
        provider: Option<String>,

        /// Filter by name (substring match)
        #[arg(long)]
        name_contains: Option<String>,
    },

    /// Get a specific model configuration by ID
    Get {
        /// Configuration ID (UUID)
        config_id: String,
    },

    /// Create a new model configuration
    Create {
        /// Configuration name
        #[arg(short, long)]
        name: String,

        /// Provider (anthropic, openai, azure_openai, bedrock)
        #[arg(short, long)]
        provider: String,

        /// Model identifier (e.g., claude-3-5-sonnet-20241022)
        #[arg(short, long)]
        model: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,

        /// Path to JSON file with provider-specific settings
        #[arg(long, value_name = "FILE")]
        settings: Option<PathBuf>,

        /// Rate limit: requests per second
        #[arg(long)]
        rate_limit: Option<u32>,
    },

    /// Update an existing model configuration
    Update {
        /// Configuration ID to update
        config_id: String,

        /// New name
        #[arg(short, long)]
        name: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// Path to JSON file with updated settings
        #[arg(long, value_name = "FILE")]
        settings: Option<PathBuf>,

        /// New rate limit: requests per second
        #[arg(long)]
        rate_limit: Option<u32>,
    },

    /// Delete a model configuration
    Delete {
        /// Configuration ID to delete
        config_id: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}
```

## 2. Flag Naming Conventions

### 2.1 Standard Flags (Inherited from Existing Patterns)

| Flag | Type | Default | Purpose | Precedent |
|------|------|---------|---------|-----------|
| `--limit` | u32 | 20 | Pagination limit | prompt.rs:20, graph.rs:18 |
| `--offset` | u32 | 0 | Pagination offset | prompt.rs:24, graph.rs:22 |
| `--organization-id` | String | None | Scope to organization | prompt.rs:28 |
| `--workspace-id` | String | None | Scope to workspace | prompt.rs:32 |
| `-f, --format` | String | table | Output format (global flag) | main.rs:25 |

### 2.2 Model-Config-Specific Flags

| Flag | Short | Type | Default | Purpose |
|------|-------|------|---------|---------|
| `--name` | `-n` | String | Required | Configuration name |
| `--provider` | `-p` | String | Required (create) | Provider type filter/value |
| `--model` | `-m` | String | Required (create) | Model identifier |
| `--description` | `-d` | String | None | Human-readable description |
| `--settings` | | PathBuf | None | Path to settings JSON file |
| `--rate-limit` | | u32 | None | Requests per second |
| `--name-contains` | | String | None | Name filter (list operation) |
| `--yes` | `-y` | bool | false | Skip confirmation |

**Design principles:**
1. **Required vs optional**: Only essential fields are required; all configuration is optional
2. **Short flags**: Only for frequently used flags (`-n`, `-p`, `-m`, `-d`)
3. **Naming**: Use snake_case for multi-word flags (`--name-contains`, `--rate-limit`)
4. **File inputs**: Use `--settings` (not `--settings-file`) following `--schema` precedent (prompt.rs:100)
5. **Boolean flags**: No value required, presence = true (prompt.rs:36, graph.rs:89)

### 2.3 Provider Values

**Known provider values** (based on scout research):

| Provider | Value | Notes |
|----------|-------|-------|
| Anthropic | `anthropic` | Default for demos/examples |
| OpenAI | `openai` | Standard OpenAI endpoint |
| Azure OpenAI | `azure_openai` | Microsoft Azure hosted |
| AWS Bedrock | `bedrock` | AWS Bedrock Converse API |
| AWS Bedrock (legacy) | `bedrock_legacy` | Legacy Bedrock API |

**Important**:
- **CLI accepts any string value** for `--provider` - not restricted to this list
- **First-class support**: Providers listed above have verified API structures and will be used in documentation/examples
- **Extensibility**: If LangSmith adds new providers or the user has custom provider configurations, the CLI will pass through any value without validation
- **Validation**: Provider names and settings structure are validated by the SDK/API, not the CLI

**Design principle**: The CLI is a thin layer that passes user intent to the API. Provider-specific validation belongs at the API boundary, not in the CLI tool.

## 3. Output Format Options

### 3.1 JSON Format

**When**: `langstar model-config list -f json`

**Structure**: Direct passthrough of SDK response
```json
[
  {
    "id": "uuid",
    "name": "Production Claude Config",
    "description": "Claude 3.5 Sonnet for production use",
    "settings": {
      "id": ["langchain", "chat_models", "anthropic", "ChatAnthropic"],
      "lc": 1,
      "type": "constructor",
      "kwargs": {
        "model": "claude-3-5-sonnet-20241022",
        "temperature": 0.7,
        "max_tokens": 8192,
        "anthropic_api_key": {
          "id": ["ANTHROPIC_API_KEY"],
          "lc": 1,
          "type": "secret"
        }
      }
    },
    "options": {
      "requests_per_second": 10
    },
    "created_at": "2025-11-30T12:00:00.123456Z",
    "updated_at": "2025-11-30T12:00:00.123456Z"
  }
]
```

**Rationale**: JSON mode users need complete data for programmatic use.

### 3.2 Table Format

**When**: `langstar model-config list` (default)

**Structure**: Simplified view using `tabled` crate

```rust
#[derive(Debug, Tabled)]
struct ModelConfigRow {
    #[tabled(rename = "Name")]
    name: String,

    #[tabled(rename = "ID")]
    id: String,  // Truncated to 8 chars

    #[tabled(rename = "Provider")]
    provider: String,  // Extracted from settings.id path

    #[tabled(rename = "Model")]
    model: String,  // Extracted from settings.kwargs.model

    #[tabled(rename = "Rate Limit")]
    rate_limit: String,  // "10/s" or "None"

    #[tabled(rename = "Updated")]
    updated_at: String,  // Date only (YYYY-MM-DD)
}
```

**Example output:**
```
╭────────────────────────┬──────────┬───────────┬────────────────────────────┬────────────┬────────────╮
│ Name                   │ ID       │ Provider  │ Model                      │ Rate Limit │ Updated    │
├────────────────────────┼──────────┼───────────┼────────────────────────────┼────────────┼────────────┤
│ Production Claude      │ d1e1dfff │ anthropic │ claude-3-5-sonnet-20241022 │ 10/s       │ 2025-11-30 │
│ Dev OpenAI             │ a2b3c4d5 │ openai    │ gpt-4o                     │ None       │ 2025-11-29 │
╰────────────────────────┴──────────┴───────────┴────────────────────────────┴────────────┴────────────╯

Found 2 model configurations
```

**Design decisions:**
- **Truncate IDs**: Show first 8 chars (UUIDs are 36 chars, too wide for tables)
- **Extract provider**: Parse from `settings.id` array (last element before provider class name)
- **Extract model**: Parse from `settings.kwargs.model`
- **Format rate limit**: Show as "N/s" or "None" for readability
- **Date only**: Use YYYY-MM-DD format (not full timestamp) for table width

**Precedent**: See prompt.rs:134-147 (PromptRow), graph.rs:94-109 (DeploymentRow)

### 3.3 Detailed View (Get Operation)

**Table format**: Human-readable formatted output
```
MODEL CONFIGURATION DETAILS
─────────────────────────────────────────
Name:        Production Claude Config
ID:          d1e1dfff-39bf-4cea-9a2e-85e970ce40ef
Provider:    Anthropic
Model:       claude-3-5-sonnet-20241022
Description: Claude 3.5 Sonnet for production use

Settings:
  temperature:    0.7
  max_tokens:     8192
  top_k:          40
  top_p:          0.95
  anthropic_api_key: SECRET:ANTHROPIC_API_KEY

Options:
  Rate limit:     10 requests/second

Timestamps:
  Created:        2025-11-30 12:00:00 UTC
  Updated:        2025-11-30 12:00:00 UTC
```

**Precedent**: See prompt.rs:322 and following (Get command output)

## 4. Error Handling and User Feedback

### 4.1 Error Handling Patterns

Following existing `CliError` patterns (cli/src/error.rs):

| Error Type | When | Example Message |
|------------|------|-----------------|
| `Sdk(LangstarError)` | SDK operation fails | `Failed to fetch model configurations: API returned 401 Unauthorized` |
| `Config(String)` | Invalid configuration | `Invalid provider: foo. Valid providers: anthropic, openai, azure_openai, bedrock` |
| `Other(anyhow::Error)` | Unexpected errors | `Failed to read settings file: No such file or directory` |

**Validation strategy:**
- **Minimal CLI validation**: Validate only file existence and JSON parsing
- **Delegate to SDK**: Provider names, model IDs, settings structure validated by SDK
- **User-friendly errors**: Catch SDK errors and re-present with helpful context

**Example error flow:**
```rust
// CLI layer: validate file exists and is readable JSON
if let Some(settings_path) = &settings {
    let settings_content = fs::read_to_string(settings_path)
        .map_err(|e| CliError::Other(anyhow::anyhow!(
            "Failed to read settings file {}: {}",
            settings_path.display(),
            e
        )))?;

    let _: serde_json::Value = serde_json::from_str(&settings_content)
        .map_err(|e| CliError::Other(anyhow::anyhow!(
            "Invalid JSON in settings file: {}",
            e
        )))?;
}

// SDK layer: validate provider-specific structure
client.model_config().create(...)
    .await?;
```

**Precedent**: See prompt.rs:467-477 (schema file reading and validation)

### 4.2 User Feedback Patterns

Using `OutputFormatter` methods (output.rs:112-143):

| Method | Use Case | Output Destination |
|--------|----------|-------------------|
| `formatter.info(msg)` | Progress updates | stderr (JSON mode) / stdout (table mode) |
| `formatter.success(msg)` | Success confirmations | stderr (JSON mode) / stdout (table mode) |
| `formatter.warning(msg)` | Non-fatal issues | stderr (JSON mode) / stdout (table mode) |
| `formatter.error(msg)` | Error messages | Always stderr |

**Example usage:**
```rust
formatter.info("Fetching model configurations...");
let configs = client.model_config().list(...).await?;
formatter.success(&format!("Found {} configurations", configs.len()));
```

**Why stderr for info/success in JSON mode:**
- Keeps stdout clean for machine-readable JSON output
- Follows Unix convention (cargo, git, curl all do this)
- Documented at output.rs:99-144

### 4.3 Confirmation Prompts

For destructive operations (`delete`), prompt for confirmation unless `--yes` flag is provided:

```rust
if !yes {
    print!("Are you sure you want to delete model configuration '{}' (id: {})? [y/N]: ",
           name, config_id);
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("Cancelled.");
        return Ok(());
    }
}
```

**Precedent**: Common pattern in CLI tools, similar to `langstar graph delete` (graph.rs:84-91)

### 4.4 Helpful Hints

Provide contextual hints when operations return empty results:

```rust
if configs.is_empty() {
    println!("\nNo model configurations found.");
    println!("💡 Hint: Create your first configuration with:");
    println!("  langstar model-config create \\");
    println!("    --name \"My Config\" \\");
    println!("    --provider anthropic \\");
    println!("    --model claude-3-5-sonnet-20241022 \\");
    println!("    --settings settings.json");
}
```

**Precedent**: See prompt.rs:298-305 (empty results hint)

## 5. Configuration Integration

### 5.1 Environment Variables

**Existing pattern** (config.rs:56-96):
```
Priority order:
1. CLI flags (--organization-id, --workspace-id)
2. Environment variables
3. Config file (~/.config/langstar/config.toml)
4. Default values
```

**Model-config feature uses existing variables** - no new env vars needed:

| Variable | Purpose | Used By |
|----------|---------|---------|
| `LANGSMITH_API_KEY` | LangSmith authentication | All LangSmith operations |
| `LANGSMITH_ORGANIZATION_ID` | Scope to organization | Optional scoping |
| `LANGSMITH_WORKSPACE_ID` | Scope to workspace | Optional scoping |
| `LANGSTAR_OUTPUT_FORMAT` | Default output format | Global CLI option |
| `LANGSTAR_TIMEZONE` | Timezone for timestamps | Table output formatting |

**Rationale**: Model configurations are tenant-scoped resources. They use the same authentication and scoping as prompts, datasets, and other LangSmith resources.

### 5.2 Config File Integration

**No changes needed** to config file structure (config.rs:8-31).

Model-config commands will use:
- `langsmith_api_key` for authentication
- `organization_id` / `workspace_id` for scoping (if configured)
- `output_format` for default display mode
- `timezone` for timestamp formatting in table mode

### 5.3 Scoping Behavior

**Apply same scoping logic as prompts** (prompt.rs:172-209):

```rust
impl ModelConfigCommands {
    fn apply_scoping(
        client: LangchainClient,
        flag_org_id: &Option<String>,
        flag_workspace_id: &Option<String>,
    ) -> LangchainClient {
        let mut client = client;

        // Warn if both org and workspace IDs are specified
        if flag_org_id.is_some() && flag_workspace_id.is_some() {
            eprintln!("⚠ Warning: Both organization and workspace IDs specified");
            eprintln!("  → Using workspace scope (narrower scope takes precedence)");
        }

        if let Some(org_id) = flag_org_id {
            client = client.with_organization_id(org_id.clone());
        }

        if let Some(workspace_id) = flag_workspace_id {
            client = client.with_workspace_id(workspace_id.clone());
        }

        client
    }
}
```

**Behavior**:
- No scoping: Operate at tenant level (personal workspace)
- `--organization-id`: Operate within organization
- `--workspace-id`: Operate within specific workspace
- Both flags: Workspace takes precedence (narrower scope)

**Precedent**: See prompt.rs:172-209

### 5.4 Sensible Defaults

| Setting | Default | Rationale |
|---------|---------|-----------|
| `--limit` | 20 | Standard pagination default across all commands |
| `--offset` | 0 | Standard pagination starting point |
| Output format | `table` | More human-readable for interactive use |
| Scoping | None (tenant) | Safest default, user opts into narrower scopes |
| Rate limit | None | No artificial rate limiting unless user specifies |
| Description | None | Optional field, not required for basic usage |

**Philosophy**: Defaults should enable quick usage without sacrificing safety.

## 6. UI Workflow and Business Purpose

### 6.1 LangSmith UI Workflow

Based on scout research (docs/research/453-ls-langsmith-model-providers-scout.md):

**UI User Journey:**
1. User navigates to **Playground** in LangSmith web UI
2. User opens **Settings** panel for a prompt
3. User selects **Model Provider** (Anthropic, OpenAI, etc.)
4. User configures **Provider Settings**:
   - Model identifier (e.g., `claude-3-5-sonnet-20241022`)
   - Temperature, max tokens, etc.
   - **API Key**: Select from workspace secrets dropdown
5. User saves configuration with optional name/description
6. Configuration appears in "Saved Configurations" dropdown for reuse

**Key insight**: Model configurations are **reusable templates** that can be:
- Shared across prompts in the same workspace
- Applied to different prompts with one click
- Modified centrally and applied to multiple prompts

### 6.2 CLI Use Cases

**Use Case 1: List existing configurations** (most common)
```bash
# See all model configs in my workspace
langstar model-config list

# Filter to only Anthropic configs
langstar model-config list --provider anthropic

# Export to JSON for scripting
langstar model-config list -f json > configs.json
```

**Use Case 2: Inspect configuration details**
```bash
# View full configuration including settings
langstar model-config get d1e1dfff-39bf

# Export single config to JSON
langstar model-config get d1e1dfff-39bf -f json > prod-claude-config.json
```

**Use Case 3: Create standardized configurations** (CI/CD)
```bash
# Create production Claude config from template
cat > claude-prod-settings.json <<EOF
{
  "model": "claude-3-5-sonnet-20241022",
  "temperature": 0.7,
  "max_tokens": 8192,
  "anthropic_api_key": {
    "id": ["ANTHROPIC_API_KEY"],
    "lc": 1,
    "type": "secret"
  }
}
EOF

langstar model-config create \
  --name "Production Claude 3.5" \
  --provider anthropic \
  --model claude-3-5-sonnet-20241022 \
  --description "Standard production config" \
  --settings claude-prod-settings.json \
  --rate-limit 10
```

**Use Case 4: Update configurations** (version upgrades)
```bash
# Update model version for existing config
cat > new-settings.json <<EOF
{
  "model": "claude-3-opus-20250201",
  "temperature": 0.7,
  "max_tokens": 8192,
  "anthropic_api_key": {
    "id": ["ANTHROPIC_API_KEY"],
    "lc": 1,
    "type": "secret"
  }
}
EOF

langstar model-config update d1e1dfff-39bf \
  --name "Production Claude 3 Opus" \
  --settings new-settings.json
```

**Use Case 5: Clean up unused configurations**
```bash
# Delete test configuration
langstar model-config delete a2b3c4d5-67ef --yes

# Interactive deletion (prompts for confirmation)
langstar model-config delete a2b3c4d5-67ef
```

### 6.3 Key Differences: CLI vs UI

| Aspect | UI Workflow | CLI Workflow |
|--------|------------|--------------|
| **Audience** | Data scientists, prompt engineers | DevOps, automation, CI/CD |
| **Interaction** | Interactive forms, dropdowns | Scriptable commands, JSON files |
| **Secrets** | Select from dropdown | Reference by name in JSON |
| **Validation** | Real-time feedback | Server-side validation on submit |
| **Bulk operations** | One at a time | Scriptable loops, batch creation |
| **Configuration storage** | Browser-based editing | Version-controlled JSON files |

**CLI advantage**: Version-controlled model configurations as code, reproducible across environments.

## 7. Future Considerations

### 7.1 Out of Scope for Initial Implementation

Based on scout phase recommendations (docs/research/453-ls-langsmith-model-providers-scout.md:193-196):

1. **Secrets Management** (separate milestone: `ls-secrets`)
   - CRUD for workspace secrets
   - Secret validation and testing
   - Not needed for model-config: uses secret references, not values

2. **Prompt-Model Linking**
   - Associating prompts with specific model configs
   - Requires additional API research
   - Different resource domain

3. **Provider Validation Builders**
   - Type-safe provider-specific configuration builders
   - Complex: each provider has different required/optional fields
   - Current approach: free-form JSON validated by server

### 7.2 Potential Enhancements

**Phase 1** (if time permits):
- `--dry-run` flag for create/update (validate without saving)
- Provider-specific examples in help text
- Richer table output (show temperature, top_p in table)

**Phase 2** (future milestone):
- `langstar model-config template <provider>` - Generate example settings JSON
- `langstar model-config validate <file>` - Validate settings file against LangChain schema
- `langstar model-config clone <id>` - Duplicate existing config with new name

**Phase 3** (advanced):
- Interactive config creation with prompts (like `gh auth login`)
- Configuration diffing: `langstar model-config diff <id1> <id2>`
- Export/import: `langstar model-config export --all > backup.json`

### 7.3 Integration Points

**With other Langstar features:**
- `langstar eval`: Could reference model configs by ID (future enhancement)
- `langstar prompt`: Could set default model config for a prompt (future API)
- `langstar runs`: Could filter runs by model config ID (if API supports)

**With external tools:**
- CI/CD: Version-controlled configs, automated deployment
- Monitoring: Export configs to JSON for change tracking
- Documentation: Generate config inventory for compliance

## 8. Implementation Phases

Based on scout phase recommendations (docs/research/453-ls-langsmith-model-providers-scout.md:179-191):

### Phase 3: SDK - List model configurations
**Deliverable**: `sdk/src/model_config.rs` with `list()` method
**API**: `GET /api/v1/playground-settings`

### Phase 4: SDK - Get model configuration by ID
**Deliverable**: Add `get()` method to `sdk/src/model_config.rs`
**API**: `GET /api/v1/playground-settings/{id}`
**Note**: Requires ID from list operation (no dedicated GET endpoint in scout findings)

### Phase 5: SDK - Create model configuration
**Deliverable**: Add `create()` method
**API**: `POST /api/v1/playground-settings`
**Schema**: `PlaygroundSettingsCreateRequest`

### Phase 6: SDK - Update model configuration
**Deliverable**: Add `update()` method
**API**: `PATCH /api/v1/playground-settings/{id}`
**Schema**: `PlaygroundSettingsUpdateRequest` (all fields optional)

### Phase 7: SDK - Delete model configuration
**Deliverable**: Add `delete()` method
**API**: `DELETE /api/v1/playground-settings/{id}`

### Phase 8: CLI - `langstar model-config list`
**Deliverable**: `cli/src/commands/model_config.rs` with List variant
**Features**: Pagination, provider filter, table/JSON output

### Phase 9: CLI - `langstar model-config create/get/update/delete`
**Deliverable**: Complete ModelConfigCommands implementation
**Features**: All CRUD operations, confirmation prompts, user feedback

### Phase 10: Documentation and Integration Tests
**Deliverable**:
- `docs/usage/model-config.md` - User-facing documentation
- `cli/tests/model_config_command_test.rs` - Integration tests
- Update `README.md` with model-config examples

## 9. Open Questions

1. **Secret validation**: Should CLI validate that referenced secrets exist before creating config?
   - **Recommendation**: No, let server handle validation. Keeps CLI thin.

2. **Provider enum**: Should CLI have a strict list of providers or accept any string?
   - **Recommendation**: Accept any string, validate server-side. Allows for new providers without CLI changes.

3. **Settings file format**: Should we support YAML in addition to JSON?
   - **Recommendation**: JSON only initially. YAML adds complexity for limited benefit.

4. **Table truncation**: How to handle very long model names or descriptions?
   - **Recommendation**: Truncate to 50 chars with "..." (show first 47 chars plus "...", see prompt.rs:160-166)

5. **Default rate limit**: Should we recommend a default in help text?
   - **Recommendation**: No default, user opt-in. Rate limits depend on quota and use case.

## 10. References

- **Scout Report**: `docs/research/453-ls-langsmith-model-providers-scout.md`
- **Sample API Response**: `reference/research/453-ls-langsmith-model-providers-playground-settings.json`
- **OpenAPI Spec**: `reference/openapi/langchain/langsmith/openapi.json`
- **Prompt Commands**: `cli/src/commands/prompt.rs` (primary reference implementation)
- **Graph Commands**: `cli/src/commands/graph.rs` (secondary reference)
- **Output Formatter**: `cli/src/output.rs`
- **Config Module**: `cli/src/config.rs`
- **Parent Issue**: #461 (Model Providers Epic)
- **Scout Issue**: #453 (Phase 0.0)
