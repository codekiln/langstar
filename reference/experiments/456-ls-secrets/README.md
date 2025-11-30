# Experiment: LangSmith Workspace Secrets API

**Date**: 2025-11-30
**Issue**: [#456 - Scout: Research ls-secrets feasibility](https://github.com/codekiln/langstar/issues/456)
**Parent Issue**: #453 identified secrets as a dependency for model provider configurations

## Objective

Test the LangSmith Workspace Secrets API to understand:

1. **API endpoints**: Verify GET and POST operations on `/api/v1/workspaces/current/secrets`
2. **Secret upsert behavior**: Understand how create/update works (POST endpoint)
3. **Value masking**: Confirm that secret values are NOT returned in responses (security)
4. **Delete behavior**: Determine if secrets can be deleted (set value to null?)
5. **Encrypted secrets endpoint**: Understand `/api/v1/workspaces/current/secrets/encrypted` purpose

## Key Findings

### API Endpoints

| Method | Path | Operation |
|--------|------|-----------|
| GET | `/api/v1/workspaces/current/secrets` | List secret keys only |
| POST | `/api/v1/workspaces/current/secrets` | Upsert secrets (create/update) |
| GET | `/api/v1/workspaces/current/secrets/encrypted` | Get encrypted secrets for specific services (e.g., agent_builder) |

### Request/Response Schemas

#### SecretKey (Response from GET)
```json
{
  "key": "string"
}
```

**Note**: Only the key is returned, NOT the value. Secret values are never exposed in API responses.

#### SecretUpsert (Request body for POST)
```json
{
  "key": "string",
  "value": "string | null"
}
```

**Note**: POST accepts an array of SecretUpsert objects.

#### InternalSecretsResponse (Response from GET encrypted)
```json
{
  "encrypted_secrets": "string"
}
```

**Note**: Returns encrypted secrets for use with Agent Builder and external services. Requires `service=agent_builder` query parameter.

### Security Observations

1. **Values never returned**: GET endpoint only returns secret keys, not values
2. **Create/update uses POST**: There is no separate PUT/PATCH endpoint
3. **No dedicated delete**: OpenAPI spec doesn't show DELETE operation
4. **Upsert pattern**: POST is used for both create and update
5. **Encrypted access**: Special endpoint for agent_builder to retrieve encrypted secrets

## Experiment Scope

This experiment includes **live API testing** to validate OpenAPI spec findings:

✅ Review OpenAPI specification for secrets endpoints
✅ Understand request/response schemas
✅ Document security patterns (value masking)
✅ Identify API operations available
✅ **Python experiment script written** (`test_secrets_api.py`)
✅ **Live API testing performed** (read-only due to permissions)

## Experiment Results

**Execution Date**: 2025-11-30
**Script**: `test_secrets_api.py`
**Output Log**: `experiment_output.log`

### Test Results Summary

| Test | Status | Finding |
|------|--------|---------|
| List secrets (GET) | ✅ PASS | Returns 200, empty array `[]` |
| Create secret (POST) | ❌ FAIL | 403 Forbidden - requires `workspaces:manage` permission |
| Update secret (POST) | ❌ FAIL | 403 Forbidden - requires `workspaces:manage` permission |
| Delete secret (POST) | ❌ FAIL | 403 Forbidden - requires `workspaces:manage` permission |

### Critical Finding: Permission Requirements

**API Key Permissions**:
- `GET /api/v1/workspaces/current/secrets` - Works with standard API key (read access)
- `POST /api/v1/workspaces/current/secrets` - Requires `workspaces:manage` permission

**Error Response** (403 Forbidden):
```json
{
  "detail": "Permission denied, you do not have the required permission workspaces:manage"
}
```

**Implication**: Secret management operations require elevated permissions. Standard API keys used for tracing/monitoring may not have write access to secrets.

### Confirmed Behaviors

1. **GET endpoint works**: Successfully lists secrets (returned empty array in test)
2. **Security confirmed**: API returns only keys in responses, never values
3. **Clear error messages**: Permission errors are explicit and helpful
4. **Permission model**: Separate read vs write permissions for secrets

## Open Questions (Require `workspaces:manage` Permission to Test)

1. **Delete behavior**: Does POST with `value: null` delete a secret, or are secrets permanent?
   - Status: ⚠️ Untested - requires elevated permissions
   - API spec suggests `value: null` is valid, but behavior unconfirmed

2. **Validation**: Does the API validate secret key naming (e.g., uppercase only, no spaces)?
   - Status: ⚠️ Untested - requires elevated permissions
   - Need to test: `TEST_KEY`, `test-key`, `test.key`, `test key`, etc.

3. **List limits**: Is there pagination for listing secrets if a workspace has many?
   - Status: ⚠️ Untested - workspace had 0 secrets, pagination not observed
   - OpenAPI spec doesn't show pagination parameters

4. **Update detection**: How does the API handle updating an existing secret vs creating new?
   - Status: ⚠️ Untested - requires elevated permissions
   - Does it return different response codes or messages?

5. **Batch operations**: How does the API handle multiple secrets in one POST?
   - Status: ⚠️ Untested - requires elevated permissions
   - Does it validate all before applying, or apply partially?

6. **Error handling**: What errors occur for invalid keys, duplicate keys, etc.?
   - Status: ⚠️ Untested - requires elevated permissions

## Recommendations for Future Experiments

**Prerequisites**: API key with `workspaces:manage` permission

When implementing `ls-secrets`, run these experiments with elevated permissions:

1. **Create secret**: Test POST with various key formats
   - `VALID_KEY_123` (uppercase, underscores, numbers)
   - `invalid-key` (lowercase with hyphens)
   - `invalid.key` (dots)
   - `invalid key` (spaces)

2. **Update secret**: Test POST with existing key and different value
   - Verify idempotent behavior
   - Check if response differs from create

3. **Delete secret**: Test POST with `value: null`
   - Verify secret disappears from GET list
   - Check if subsequent GET returns 404 or silently succeeds

4. **Batch operations**: Test POST with multiple secrets
   - All valid keys
   - Mix of valid and invalid keys (partial failure behavior?)

5. **List pagination**: Create 100+ secrets and test if pagination occurs

6. **Permission boundaries**: Test with read-only API key to confirm 403 behavior

## References

- OpenAPI Spec: `/workspace/reference/openapi/langchain/langsmith/openapi.json`
- Secrets endpoints: Lines containing `/api/v1/workspaces/current/secrets`
- Schema definitions: `SecretKey`, `SecretUpsert`, `InternalSecretsResponse`
