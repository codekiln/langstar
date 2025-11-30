# ls-secrets Feasibility Scout

**Issue**: #456
**Date**: 2025-11-30
**Status**: Complete

## Executive Summary

**Feasibility**: Go

The LangSmith API provides workspace secrets management via `/api/v1/workspaces/current/secrets` endpoints. The API uses an upsert pattern (POST for both create and update) and only returns secret keys (not values) for security. Implementation complexity is **Low-Medium** with no blocking dependencies.

## 1. Existing Langstar Code

**Finding**: No existing implementation for workspace secrets management.

### Related Code Found

**`sdk/src/deployments.rs:5-12` - DeploymentSecret**
```rust
/// A secret environment variable for a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSecret {
    /// Name of the secret environment variable
    pub name: String,
    /// Value of the secret (will be redacted in responses)
    pub value: String,
}
```

**Important distinction**: `DeploymentSecret` is for **deployment** environment variables, NOT workspace secrets. These are different concepts:
- **Deployment secrets**: Environment variables passed to a specific LangGraph deployment
- **Workspace secrets**: Workspace-level secrets that can be referenced by model provider configurations

Searched `./cli` and `./sdk` directories:
- `cli/src/commands/graph.rs:404-416` - Uses `DeploymentSecret` for deployment env vars
- No code for workspace secrets management (`/api/v1/workspaces/current/secrets`)

**Conclusion**: This is greenfield implementation. Existing `DeploymentSecret` struct is unrelated.

## 2. Python SDK Precedent

**Finding**: No Python SDK methods for workspace secrets management.

The `langsmith-sdk` Python client does NOT expose workspace secrets API:
- Checked `reference/repo/langchain-ai/langsmith-sdk/notes/README.md` - no mentions
- Python SDK repo not fully cloned locally, but grep searches found no references
- Related issue #453 notes that model provider configs reference secrets, but doesn't manage them

**Conclusion**: Langstar would likely be first to implement this API in an SDK. This means:
1. No reference implementation to follow (pro/con)
2. More flexibility in API design (pro)
3. Need to make our own decisions on error handling, validation, etc. (con)

## 3. API Endpoints

Source: `/workspace/reference/openapi/langchain/langsmith/openapi.json`

### Endpoints

| Method | Path | Operation | Purpose |
|--------|------|-----------|---------|
| GET | `/api/v1/workspaces/current/secrets` | List secrets | Returns array of secret keys only (no values) |
| POST | `/api/v1/workspaces/current/secrets` | Upsert secrets | Create or update secrets (upsert pattern) |
| GET | `/api/v1/workspaces/current/secrets/encrypted` | Get encrypted secrets | For agent_builder and external services only |

**Key observation**: No DELETE endpoint. Deletion likely via POST with `value: null` (needs experimental validation).

### Request/Response Schemas

#### GET `/api/v1/workspaces/current/secrets` → SecretKey[]

**Response:**
```json
[
  {
    "key": "ANTHROPIC_API_KEY"
  },
  {
    "key": "OPENAI_API_KEY"
  }
]
```

**Schema:**
```json
{
  "properties": {
    "key": {
      "type": "string",
      "title": "Key"
    }
  },
  "type": "object",
  "required": ["key"],
  "title": "SecretKey"
}
```

**Security**: Values are NEVER returned. Only keys are exposed in responses.

#### POST `/api/v1/workspaces/current/secrets` ← SecretUpsert[]

**Request:**
```json
[
  {
    "key": "ANTHROPIC_API_KEY",
    "value": "sk-ant-..."
  },
  {
    "key": "OPENAI_API_KEY",
    "value": "sk-..."
  }
]
```

**Schema:**
```json
{
  "properties": {
    "key": {
      "type": "string",
      "title": "Key"
    },
    "value": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "type": "null"
        }
      ],
      "title": "Value"
    }
  },
  "type": "object",
  "required": ["key", "value"],
  "title": "SecretUpsert"
}
```

**Response**: Empty object `{}`

**Important**:
- POST is used for both create AND update (upsert pattern)
- `value` can be `null` (likely for deletion)
- Accepts array of secrets (batch operations)

#### GET `/api/v1/workspaces/current/secrets/encrypted`

**Purpose**: Get encrypted secrets for specific services (e.g., Agent Builder)

**Query Parameters:**
- `service` (required): Must be `"agent_builder"` (enum)
- `key_names` (optional): Array of secret keys to return

**Response:**
```json
{
  "encrypted_secrets": "base64_encoded_encrypted_data"
}
```

**Conclusion**: This endpoint is for **internal** use by LangSmith services, not for end-user CLI operations. OUT OF SCOPE for initial implementation.

### Authentication

All endpoints support standard authentication methods:
- `API Key` header
- `Tenant ID` header
- `Bearer Auth` token

Same pattern as existing Langstar SDK implementations.

## 4. Experiments

**Location**: `reference/experiments/456-ls-secrets/README.md`

**Approach**: Spec-based analysis (no live API testing)

### Key Findings from Spec Analysis

1. **Upsert pattern confirmed**: POST handles both create and update
2. **Batch operations**: POST accepts array of secrets
3. **No get-by-id**: Cannot fetch a single secret by key
4. **Delete unclear**: `value: null` in schema suggests deletion, but needs validation
5. **No pagination**: List endpoint doesn't show pagination in spec

### Open Questions

These should be answered before full implementation (Phase 2-3):

1. **Delete behavior**: Does POST with `value: null` delete a secret?
2. **Key validation**: What characters are allowed in secret keys? (Likely uppercase + underscores)
3. **Conflict handling**: What happens if you POST an existing key? (Overwrites value)
4. **List limits**: Are there limits on number of secrets per workspace?
5. **Idempotency**: Is POST idempotent for updates?

**Recommended pre-implementation experiments**:
- Test create: `[{"key": "TEST_SECRET", "value": "test123"}]`
- Test update: Same key, different value
- Test delete: `[{"key": "TEST_SECRET", "value": null}]`
- Test list: Verify created/updated/deleted secrets appear correctly

## 5. Complexity Assessment

**Overall Complexity**: Low-Medium

### Straightforward Aspects (Low Complexity)

- **Simple API**: Only 2 primary endpoints (GET list, POST upsert)
- **Simple schemas**: SecretKey (1 field), SecretUpsert (2 fields)
- **Standard auth**: Reuse existing Langstar SDK auth patterns
- **No pagination**: Simpler implementation (unless many secrets)
- **Clear security model**: Values never returned in responses
- **Well-documented**: OpenAPI spec is complete

### Moderate Complexity Aspects

- **Upsert logic**: Single endpoint for create/update requires conditional logic
- **Batch operations**: POST accepts arrays, need to handle multiple secrets
- **Delete ambiguity**: No explicit DELETE, must test null value behavior
- **Error handling**: Unknown API error responses (need to test)
- **CLI UX**: Need good UX for create/update/delete/list operations

### Dependencies

**None identified!**

- Independent of model providers (they just reference secrets)
- Independent of deployments (DeploymentSecret is separate)
- Standard auth already implemented in SDK
- No special workspace configuration needed

### Technical Risks (Low)

1. **Delete behavior unknown**: Mitigated by pre-implementation testing
2. **No Python SDK precedent**: Mitigated by well-documented OpenAPI spec
3. **Validation rules unclear**: Mitigated by following common patterns (uppercase, underscores)

## 6. Recommendation

**Decision**: Go

**Rationale**:
1. ✅ Simple, well-documented API with only 2 primary endpoints
2. ✅ Low-medium complexity is manageable
3. ✅ No blocking dependencies
4. ✅ Clear business value: Required for model provider configurations (#453)
5. ✅ Security model is sound (values never exposed)
6. ✅ Can be implemented incrementally

### Proposed 8-Phase Milestone

Following the standard feature development process (`docs/dev/feature-development-process.md`):

| Phase | Description | Deliverables |
|-------|-------------|--------------|
| **0** | Epic setup | Epic issue, milestone, sub-issues |
| **1** | SDK: List secrets | `SecretKey` struct, `list_secrets()` method |
| **2** | SDK: Upsert secrets | `SecretUpsert` struct, `upsert_secrets()` method |
| **3** | SDK: Integration tests | Test list, create, update, delete (if supported) |
| **4** | CLI: List command | `langstar secrets list` |
| **5** | CLI: Set command | `langstar secrets set KEY VALUE` (upsert) |
| **6** | CLI: Delete command | `langstar secrets delete KEY` (if supported) |
| **7** | CLI: Integration tests | End-to-end CLI testing |
| **8** | Documentation | Usage guide, examples |

### Optional Phase Extensions

These could be added later as separate issues:

- **Interactive mode**: `langstar secrets set KEY` prompts for value (hides input)
- **Bulk import**: `langstar secrets import -f secrets.env`
- **Bulk export**: `langstar secrets export` (keys only, no values)
- **Validation**: Warn if key doesn't follow conventions (e.g., not uppercase)

### Out of Scope

1. **Encrypted endpoint**: `/api/v1/workspaces/current/secrets/encrypted` is for internal services only
2. **Secret rotation**: Automated rotation/expiration (if API supports it)
3. **Secret templates**: Pre-configured secret sets for common providers
4. **Model provider integration**: That's a separate milestone (already scouted in #453)

## 7. Success Criteria

This scout issue is complete when:

- [x] Research report documents API endpoints and schemas
- [x] Complexity assessed (Low-Medium)
- [x] Technical blockers identified (None)
- [x] Go/No-Go recommendation provided (Go)
- [x] Experiments documented (spec analysis in `reference/experiments/456-ls-secrets/`)
- [x] PR merged to main

## References

- **Scout Issue**: #456
- **Parent Issue**: #453 (identified secrets as dependency)
- **OpenAPI Spec**: `reference/openapi/langchain/langsmith/openapi.json`
- **Experiments**: `reference/experiments/456-ls-secrets/README.md`
- **Feature Development Process**: `docs/dev/feature-development-process.md`
- **Related Code**:
  - `sdk/src/deployments.rs:5-12` (DeploymentSecret - different concept)
  - `cli/src/commands/graph.rs:404-416` (uses DeploymentSecret)
