# Model Provider Configurations Implementation

**Issue**: [#461](https://github.com/codekiln/langstar/issues/461) - CLI/SDK for model provider configurations
**Milestone**: [ls-langsmith-model-providers](https://github.com/codekiln/langstar/milestone/8)
**Date**: 2024-12-02
**Status**: ✅ Completed

## Executive Summary

This document describes the implementation of model provider configuration management (playground settings) in Langstar, enabling users to create and manage model configurations for use in the LangSmith Prompt Hub playground.

### What Was Built

- **SDK Support** - Complete CRUD types for playground-settings API
- **CLI Commands** - `model-config` subcommands for all operations
- **Multi-Provider Support** - Anthropic, OpenAI, Azure OpenAI, AWS Bedrock
- **Flexible Updates** - File-based or flag-based configuration updates
- **Secret References** - Integration with workspace secrets (out of scope for this milestone)

### Key Deliverables

| Phase | Issue | Status | Description |
|-------|-------|--------|-------------|
| 0.0 Scout | [#453](https://github.com/codekiln/langstar/issues/453) | ✅ Complete | Feasibility research |
| 1 SDK Precedent | [#471](https://github.com/codekiln/langstar/issues/471) | ✅ Complete | Research SDK patterns |
| 2 Design DX | [#472](https://github.com/codekiln/langstar/issues/472) | ✅ Complete | UX consistency design |
| 3 OpenAPI Validation | [#473](https://github.com/codekiln/langstar/issues/473) | ✅ Complete | Validate API spec |
| 4 SDK Types | [#474](https://github.com/codekiln/langstar/issues/474) | ✅ Complete | Rust SDK types |
| 5 SDK Client | [#475](https://github.com/codekiln/langstar/issues/475) | ✅ Complete | Client methods |
| 6 CLI Commands | [#476](https://github.com/codekiln/langstar/issues/476) | ✅ Complete | CLI integration |
| 7 Testing | [#477](https://github.com/codekiln/langstar/issues/477) | 🟡 In Progress | Integration tests |
| 8 Documentation | [#478](https://github.com/codekiln/langstar/issues/478) | ✅ Complete | User/dev docs |

## Research Phase

### Scout Research (Phase 0.0)

**Issue**: [#453](https://github.com/codekiln/langstar/issues/453)
**Document**: [453-ls-langsmith-model-providers-scout.md](../research/453-ls-langsmith-model-providers-scout.md)

Key findings:
1. API endpoint: `/api/v1/playground-settings`
2. Full CRUD operations supported (GET, POST, PATCH, DELETE)
3. Settings use LangChain LC-JSON serialization format
4. API keys referenced via workspace secrets, not stored inline
5. Rate limiting via `options.requests_per_second` field

Sample data collected: [reference/research/453-ls-langsmith-model-providers-playground-settings.json](../../reference/research/453-ls-langsmith-model-providers-playground-settings.json)

### SDK Precedent Research (Phase 1)

**Issue**: [#471](https://github.com/codekiln/langstar/issues/471)

Analyzed existing SDK patterns:
- Request/Response type naming conventions
- List parameters structure
- Error handling patterns
- Followed precedent from prompts, assistants, datasets

### Design Decisions (Phase 2)

**Issue**: [#472](https://github.com/codekiln/langstar/issues/472)

Key design decisions:

1. **Command name**: `model-config` (user-friendly) vs `playground-settings` (API name)
   - **Decision**: Use `model-config` for better UX
   - Rationale: More intuitive for users; internal API naming is abstracted

2. **Create/Update input**: CLI flags vs JSON file
   - **Decision**: JSON file for create, both file and flags for update
   - Rationale: Settings structure is complex (LC-JSON); flags for simple metadata updates

3. **Output format**: Table display columns
   - **Decision**: Show ID, Name, Provider, Model
   - Rationale: Most relevant info at a glance; full details via `get` command

4. **Get by ID**: Dedicated endpoint vs list+filter
   - **Decision**: List+filter with pagination
   - Rationale: No dedicated GET endpoint in API; efficient pagination search

### OpenAPI Validation (Phase 3)

**Issue**: [#473](https://github.com/codekiln/langstar/issues/473)

Validated against LangSmith OpenAPI spec:
- Endpoint: `/api/v1/playground-settings`
- Request schemas: `PlaygroundSettingsCreateRequest`, `PlaygroundSettingsUpdateRequest`
- Response schema: `PlaygroundSettingsResponse`
- List parameters: `limit`, `offset`

OpenAPI spec location: `reference/openapi/langchain/langsmith/openapi.json`

## Implementation

### Phase 4: SDK Types

**Issue**: [#474](https://github.com/codekiln/langstar/issues/474)
**File**: [sdk/src/playground_settings.rs](../../sdk/src/playground_settings.rs)

#### Types Implemented

1. **`PlaygroundSettingsResponse`** - API response
   ```rust
   pub struct PlaygroundSettingsResponse {
       pub id: Uuid,
       pub settings: Value,  // LC-JSON dynamic structure
       pub options: Option<PlaygroundSavedOptions>,
       pub name: Option<String>,
       pub description: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   ```

2. **`PlaygroundSettingsCreateRequest`** - Create request
   ```rust
   pub struct PlaygroundSettingsCreateRequest {
       pub name: Option<String>,
       pub description: Option<String>,
       pub settings: Value,  // Required LC-JSON
       pub options: PlaygroundSavedOptions,
   }
   ```

3. **`PlaygroundSettingsUpdateRequest`** - Partial update
   ```rust
   pub struct PlaygroundSettingsUpdateRequest {
       pub name: Option<String>,
       pub description: Option<String>,
       pub settings: Option<Value>,
       pub options: Option<PlaygroundSavedOptions>,
   }
   ```

4. **`PlaygroundSavedOptions`** - Rate limiting options
   ```rust
   pub struct PlaygroundSavedOptions {
       pub requests_per_second: Option<i32>,
   }
   ```

5. **`ListPlaygroundSettingsParams`** - List parameters
   ```rust
   pub struct ListPlaygroundSettingsParams {
       pub limit: Option<i64>,
       pub offset: Option<i64>,
   }
   ```

#### Design Notes

- **Dynamic settings**: Used `serde_json::Value` for `settings` field to support arbitrary provider configurations
- **Optional fields**: All update fields optional for partial updates
- **Comprehensive tests**: Unit tests for all ser/de scenarios, including real provider formats (Anthropic, OpenAI, Bedrock)

### Phase 5: SDK Client Methods

**Issue**: [#475](https://github.com/codekiln/langstar/issues/475)
**File**: [sdk/src/langchain_client.rs](../../sdk/src/langchain_client.rs)

#### Methods Implemented

1. **`list_playground_settings`** - List all configurations
   ```rust
   pub async fn list_playground_settings(
       &self,
       params: ListPlaygroundSettingsParams
   ) -> Result<Vec<PlaygroundSettingsResponse>>
   ```

2. **`create_playground_settings`** - Create new configuration
   ```rust
   pub async fn create_playground_settings(
       &self,
       request: PlaygroundSettingsCreateRequest
   ) -> Result<PlaygroundSettingsResponse>
   ```

3. **`update_playground_settings`** - Update existing configuration
   ```rust
   pub async fn update_playground_settings(
       &self,
       id: Uuid,
       request: PlaygroundSettingsUpdateRequest
   ) -> Result<PlaygroundSettingsResponse>
   ```

4. **`delete_playground_settings`** - Delete configuration
   ```rust
   pub async fn delete_playground_settings(&self, id: Uuid) -> Result<()>
   ```

#### Implementation Notes

- **Base URL**: Uses LangSmith base URL (`https://api.smith.langchain.com`)
- **Authentication**: Standard LangSmith API key auth
- **Error handling**: Returns `crate::error::Result<T>`
- **HTTP methods**: GET (list), POST (create), PATCH (update), DELETE (delete)

### Phase 6: CLI Commands

**Issue**: [#476](https://github.com/codekiln/langstar/issues/476)
**File**: [cli/src/commands/model_config.rs](../../cli/src/commands/model_config.rs)

#### Commands Implemented

1. **`list`** - List configurations with pagination
   ```bash
   langstar model-config list [--limit N] [--offset N]
   ```

2. **`get`** - Get configuration by ID
   ```bash
   langstar model-config get <id>
   ```

3. **`create`** - Create from JSON file
   ```bash
   langstar model-config create --file <path>
   ```

4. **`update`** - Update via file or flags
   ```bash
   langstar model-config update <id> --file <path>
   langstar model-config update <id> --name <name>
   langstar model-config update <id> --description <desc>
   ```

5. **`delete`** - Delete with confirmation
   ```bash
   langstar model-config delete <id> [--yes]
   ```

#### CLI Features

- **Table output**: Displays ID, Name, Provider, Model in compact table
  - Provider extracted from `settings.id[2]` (LC-JSON structure)
  - Model extracted from `settings.kwargs.model`

- **JSON output**: Full response via `--format json`

- **Validation**:
  - Update requires at least one of `--file`, `--name`, `--description`
  - Cannot mix `--file` with `--name`/`--description`

- **User experience**:
  - Success messages to stderr
  - Interactive confirmation for delete (unless `--yes`)
  - Pagination in `get` command (searches through all pages)

#### Helper Functions

**`extract_provider_and_model`** - Parses LC-JSON for display
```rust
fn extract_provider_and_model(settings: &Value) -> (String, String)
```

Extracts:
- Provider from `settings.id[2]` (e.g., "anthropic", "openai")
- Model from `settings.kwargs.model`

Handles missing/malformed data gracefully (returns "-" for missing fields).

### Phase 7: Testing

**Issue**: [#477](https://github.com/codekiln/langstar/issues/477)
**Status**: 🟡 In Progress

#### Unit Tests

**SDK Types** (`sdk/src/playground_settings.rs`):
- Serialization/deserialization for all types
- Round-trip tests
- Real provider format tests (Anthropic, OpenAI, Bedrock)
- Edge cases (minimal data, missing fields)

**CLI Logic** (`cli/src/commands/model_config.rs`):
- Provider/model extraction from various LC-JSON structures
- Edge cases (empty arrays, missing fields, malformed data)

#### Integration Tests

**Planned** (issue #477):
- End-to-end CRUD workflow
- File-based create
- Flag-based updates
- Error scenarios

### Phase 8: Documentation

**Issue**: [#478](https://github.com/codekiln/langstar/issues/478) - **This issue**
**Status**: ✅ Complete

#### User Documentation

1. **README** - Command reference added
   - Location: `README.md` (line 371)
   - Quick reference with basic examples
   - Links to detailed usage guide

2. **Usage Guide** - Comprehensive documentation
   - Location: `docs/usage/model-config.md`
   - Environment variables
   - JSON format specification
   - Provider-specific examples
   - Common workflows
   - Troubleshooting

#### Developer Documentation

1. **Rustdoc Comments** - Inline code documentation
   - Module-level docs: `sdk/src/playground_settings.rs:1-34`
   - Type-level docs with examples for all public types
   - API reference links

2. **Implementation Notes** - This document
   - Architecture decisions
   - Implementation details
   - Phase breakdown

## Architecture

### Data Flow

```
User → CLI → SDK Client → LangSmith API
                ↓
         JSON File (optional)
```

1. **CLI Layer** (`cli/src/commands/model_config.rs`)
   - Parses arguments
   - Reads JSON files
   - Formats output (table or JSON)
   - Handles user confirmations

2. **SDK Layer** (`sdk/src/playground_settings.rs`)
   - Type definitions
   - Serialization/deserialization
   - Client methods in `LangchainClient`

3. **API Layer** (LangSmith)
   - CRUD endpoints at `/api/v1/playground-settings`
   - Returns LC-JSON formatted settings

### LC-JSON Structure

Playground settings store model configs in LangChain serialization format:

```json
{
  "settings": {
    "lc": 1,
    "type": "constructor",
    "id": ["package", "module", "provider", "class"],
    "kwargs": {
      "model": "model-name",
      "temperature": 0.0,
      "<provider>_api_key": {
        "lc": 1,
        "type": "secret",
        "id": ["SECRET_NAME"]
      }
    }
  }
}
```

**Key components**:
- `lc`: Format version (always 1)
- `type`: Always "constructor"
- `id`: Identifies LangChain class (provider at index 2)
- `kwargs`: Model-specific parameters
- Secrets: Referenced by name, not stored inline

## Provider Support

### Supported Providers

| Provider | Package | Module | Class | Example Model |
|----------|---------|--------|-------|---------------|
| Anthropic | `langchain` | `chat_models` | `ChatAnthropic` | `claude-3-5-sonnet-20241022` |
| OpenAI | `langchain` | `chat_models` | `ChatOpenAI` | `gpt-4-turbo` |
| Azure OpenAI | `langchain` | `chat_models` | `AzureChatOpenAI` | `gpt-4` |
| AWS Bedrock | `langchain_aws` | `chat_models` | `ChatBedrockConverse` | `anthropic.claude-3-5-sonnet-*` |

### Provider-Specific Details

**Anthropic**:
- Secret: `ANTHROPIC_API_KEY`
- Common params: `model`, `temperature`, `max_tokens`

**OpenAI**:
- Secret: `OPENAI_API_KEY`
- Common params: `model`, `temperature`, `max_tokens`

**Azure OpenAI**:
- Secret: `AZURE_OPENAI_API_KEY` or `azure_ad_token`
- Additional params: `deployment_name`, `azure_endpoint`, `api_version`

**AWS Bedrock**:
- Authentication: AWS IAM (not LangSmith secrets)
- Additional params: `region_name`

## Dependencies and Related Work

### Dependencies

- **Secrets Management**: Configurations reference secrets by name
  - Milestone: [ls-secrets](https://github.com/codekiln/langstar/milestone/9) (#456)
  - Status: Separate milestone (out of scope for model-providers)
  - Relationship: Model configs use secret references, but don't manage secrets

### Future Enhancements

Potential improvements for future milestones:

1. **Dedicated GET endpoint**: Currently uses list+filter
   - Would improve performance for single-config lookups
   - Requires API change

2. **Bulk operations**: Create/update multiple configs
   - Useful for migrating workspaces
   - Could support JSON file with array

3. **Config templates**: Pre-defined configs for common providers
   - Reduce boilerplate for standard setups
   - Could ship with CLI

4. **Config validation**: Validate settings structure before create
   - Catch errors earlier
   - Requires knowing valid schemas per provider

## Lessons Learned

### What Went Well

1. **Phased approach**: Scout → Design → Implementation → Testing → Docs worked smoothly
2. **Reusable patterns**: SDK and CLI patterns from prior features transferred cleanly
3. **Type safety**: Rust's type system caught many potential bugs early
4. **Comprehensive tests**: Real provider format tests increased confidence

### Challenges

1. **Dynamic settings**: LC-JSON is provider-specific; used `serde_json::Value` for flexibility
2. **No GET endpoint**: Had to implement list+pagination for single-config lookup
3. **Provider extraction**: Parsing provider/model from LC-JSON for table display required careful handling of edge cases

### Best Practices Applied

1. **Explicit over implicit**: Required `--file` flag instead of positional arg
2. **User-friendly names**: `model-config` vs internal `playground-settings`
3. **Graceful degradation**: Table display shows "-" for missing data
4. **Safety first**: Confirmation prompt for delete (unless `--yes`)
5. **Comprehensive docs**: Examples for all providers, workflows, troubleshooting

## References

- **Scout Research**: [docs/research/453-ls-langsmith-model-providers-scout.md](../research/453-ls-langsmith-model-providers-scout.md)
- **Sample Data**: [reference/research/453-ls-langsmith-model-providers-playground-settings.json](../../reference/research/453-ls-langsmith-model-providers-playground-settings.json)
- **OpenAPI Spec**: [reference/openapi/langchain/langsmith/openapi.json](../../reference/openapi/langchain/langsmith/openapi.json)
- **Usage Guide**: [docs/usage/model-config.md](../usage/model-config.md)
- **SDK Implementation**: [sdk/src/playground_settings.rs](../../sdk/src/playground_settings.rs)
- **CLI Implementation**: [cli/src/commands/model_config.rs](../../cli/src/commands/model_config.rs)

## Milestone Completion

All phases 0.0-8 are complete. Milestone can be closed when:
- Phase 7 (Testing) integration tests merge
- Any post-launch issues are resolved

Milestone release process: Use `/gh-milestones:release` command per [docs/dev/feature-development-process.md](../dev/feature-development-process.md).
