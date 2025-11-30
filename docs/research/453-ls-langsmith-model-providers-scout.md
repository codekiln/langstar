# ls-langsmith-model-providers Feasibility Scout

**Issue**: #453
**Date**: 2025-11-30
**Status**: Complete

## Executive Summary

**Feasibility**: Go

The LangSmith API provides full CRUD operations for model provider configurations via the `/api/v1/playground-settings` endpoints. The API is documented in the OpenAPI spec and the data structures are well-defined. Implementation complexity is **Medium** due to the dynamic nature of the `settings` object and the dependency on workspace secrets for credential management.

## 1. Existing Langstar Code

**Finding**: No existing implementation for model providers/playground settings.

Searched `./cli` and `./sdk` directories:
- `cli/src/commands/eval.rs` - References `model_config` but for evaluation configuration, not provider management
- `sdk/src/evaluations.rs` - Similar context, unrelated to playground settings

**Conclusion**: This is greenfield implementation.

## 2. Python SDK Precedent

**Finding**: No Python SDK methods for playground settings.

The `langsmith-sdk` Python client does NOT expose playground settings management:
- `client.py` has `_get_settings()` for tenant settings only
- No `list_playground_settings`, `create_playground_settings`, etc. methods
- Test files reference `["langsmith", "playground", "PromptPlayground"]` in LangChain serialization format

**Conclusion**: Langstar would be first to implement this API in an SDK. This means:
1. No reference implementation to follow
2. More flexibility in API design
3. Opportunity to set precedent for other SDKs

## 3. API Endpoints

### Endpoints

| Method | Path | Operation |
|--------|------|-----------|
| GET | `/api/v1/playground-settings` | List all settings for tenant |
| POST | `/api/v1/playground-settings` | Create new settings |
| PATCH | `/api/v1/playground-settings/{id}` | Update existing settings |
| DELETE | `/api/v1/playground-settings/{id}` | Delete settings |

### Request/Response Schemas

#### PlaygroundSettingsResponse
```json
{
  "id": "uuid",
  "settings": { /* LangChain serialized model config */ },
  "options": {
    "requests_per_second": null | integer
  },
  "name": "string | null",
  "description": "string | null",
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

#### PlaygroundSettingsCreateRequest
```json
{
  "name": "string | null",
  "description": "string | null",
  "settings": { /* required, LangChain serialized model config */ },
  "options": { "requests_per_second": null | integer }
}
```

#### PlaygroundSettingsUpdateRequest
All fields optional.

### Settings Object Structure

The `settings` field uses LangChain's serialization format:

```json
{
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
}
```

### Supported Providers (from sample data)

| Provider | ID Path | Key Parameters |
|----------|---------|----------------|
| Anthropic | `langchain.chat_models.anthropic.ChatAnthropic` | model, temperature, max_tokens, top_k, top_p |
| OpenAI | `langchain.chat_models.openai.ChatOpenAI` | model, temperature, top_p, base_url, presence_penalty, frequency_penalty |
| Azure OpenAI | `langchain.chat_models.azure_openai.AzureChatOpenAI` | deployment_name, azure_endpoint, openai_api_version |
| AWS Bedrock | `langchain_aws.chat_models.ChatBedrockConverse` | model_id, region_name |
| AWS Bedrock (legacy) | `langchain.chat_models.bedrock.ChatBedrock` | model_id, region_name |

### Secret References

Credentials are NOT stored directly. Instead, they reference workspace secrets:

```json
{
  "anthropic_api_key": {
    "id": ["ANTHROPIC_API_KEY"],
    "lc": 1,
    "type": "secret"
  }
}
```

**Important**: Secrets management is a separate concern. The playground settings API only stores *references* to secrets, not the secrets themselves.

## 4. Complexity Assessment

**Complexity**: Medium

### Straightforward Aspects
- Standard CRUD operations
- OpenAPI spec is complete
- Authentication follows existing patterns (API Key, Tenant ID, Bearer)
- Basic schema validation is typed

### Complex Aspects
1. **Dynamic settings object**: The `settings` field is `additionalProperties: true` - a generic JSON object
2. **Provider-specific validation**: Each provider has different required/optional fields
3. **Secret references**: Must validate that referenced secrets exist (separate API)
4. **LangChain serialization format**: Need to understand and potentially validate the `id`, `lc`, `type`, `kwargs` structure

### Dependencies
- **Secrets management**: OUT OF SCOPE per scout requirements. Must be separate milestone.
- Existing SDK authentication infrastructure can be reused

## 5. Experiments

### Experiment 1: API Exploration (via sample data)

Sample response from `GET /api/v1/playground-settings` is available at:
`reference/research/453-ls-langsmith-model-providers-playground-settings.json`

Key observations:
1. UUIDs are used for `id` field
2. Timestamps use ISO 8601 format with microseconds
3. All providers follow consistent LangChain serialization format
4. Rate limiting via `options.requests_per_second`

### Recommended Future Experiments

Before full implementation, consider running these experiments:

1. **Create/Update flow**: Test POST/PATCH with minimal payload
2. **Secret validation**: Test what happens when referencing non-existent secrets
3. **Provider validation**: Test what validation (if any) the API performs on `settings.id` path

## 6. Recommendation

**Decision**: Go

**Rationale**:
1. Full CRUD API exists and is documented in OpenAPI spec
2. Clear business value: CLI management of model configurations for Prompt Hub
3. Medium complexity is manageable
4. No blocking dependencies (secrets management is separate)
5. Opportunity to be first SDK implementation

### Proposed Phases

| Phase | Description |
|-------|-------------|
| 0 | Parent epic issue, milestone creation |
| 1 | SDK: List playground settings |
| 2 | SDK: Get single playground setting by ID |
| 3 | SDK: Create playground setting |
| 4 | SDK: Update playground setting |
| 5 | SDK: Delete playground setting |
| 6 | CLI: `langstar model-config list` |
| 7 | CLI: `langstar model-config create/update/delete` |
| 8 | Documentation and integration tests |

### Out of Scope (Separate Milestones)

1. **Secrets management**: CRUD for workspace secrets (separate milestone `ls-secrets`)
2. **Prompt-model linking**: Associating prompts with specific model configs
3. **Provider validation**: Type-safe provider-specific configuration builders

## References

- Scout Issue: #453
- Sample Data: `reference/research/453-ls-langsmith-model-providers-playground-settings.json`
- OpenAPI Spec: `reference/openapi/langchain/langsmith/openapi.json`
- LangSmith Docs: [Configure prompt settings](https://docs.langchain.com/langsmith/managing-model-configurations)
