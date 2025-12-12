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

| Method | Path                                           | Operation                                                         |
| ------ | ---------------------------------------------- | ----------------------------------------------------------------- |
| GET    | `/api/v1/workspaces/current/secrets`           | List secret keys only                                             |
| POST   | `/api/v1/workspaces/current/secrets`           | Upsert secrets (create/update)                                    |
| GET    | `/api/v1/workspaces/current/secrets/encrypted` | Get encrypted secrets for specific services (e.g., agent_builder) |

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
**Test Secret**: `LANGSTAR_TEST_SECRET_456`
**Output Log**: Available locally (not committed)

### Complete CRUD Test Results

✅ **ALL OPERATIONS SUCCESSFUL** with `workspaces:manage` permission!

| Test                  | Status  | API Response                                   | Behavior                         |
| --------------------- | ------- | ---------------------------------------------- | -------------------------------- |
| List secrets (GET)    | ✅ PASS | 200 OK, `[]`                                   | Returns empty array initially    |
| Create secret (POST)  | ✅ PASS | 200 OK, `null`                                 | Secret created successfully      |
| Verify creation (GET) | ✅ PASS | 200 OK, `[{"key":"LANGSTAR_TEST_SECRET_456"}]` | Secret appears in list           |
| Update secret (POST)  | ✅ PASS | 200 OK, `null`                                 | Same endpoint as create (upsert) |
| Verify update (GET)   | ✅ PASS | 200 OK, `[{"key":"LANGSTAR_TEST_SECRET_456"}]` | Secret still in list             |
| Delete secret (POST)  | ✅ PASS | 200 OK, `null`                                 | POST with `value: null`          |
| Verify deletion (GET) | ✅ PASS | 200 OK, `[]`                                   | Secret removed from list         |

### Critical Finding: Permission Requirements

**API Key Permissions**:

- `GET /api/v1/workspaces/current/secrets` - Works with standard API key (read access)
- `POST /api/v1/workspaces/current/secrets` - Requires `workspaces:manage` permission

**Initial Test** (standard API key):

```json
{
  "detail": "Permission denied, you do not have the required permission workspaces:manage"
}
```

**Second Test** (elevated API key): All operations succeeded ✅

### Confirmed Behaviors

1. ✅ **GET endpoint works**: Returns array of `{"key": "..."}` objects (values never included)
2. ✅ **POST upsert pattern**: Same endpoint for create and update (idempotent)
3. ✅ **Deletion works**: POST with `value: null` successfully removes secret
4. ✅ **Security confirmed**: API never returns secret values in any response
5. ✅ **Response format**: POST returns `null` on success, GET returns key array
6. ✅ **Clear error messages**: Permission errors are explicit about required scopes
7. ✅ **Immediate consistency**: Changes visible immediately in subsequent GET requests

## Answered Questions

1. ✅ **Delete behavior**: POST with `value: null` successfully deletes secrets
   - Confirmed: Secret disappears from GET list after deletion
   - Response: 200 OK with `null` body

2. ✅ **Update vs create**: Same endpoint and response for both (true upsert)
   - No way to distinguish create from update in response
   - Both return 200 OK with `null` body
   - Idempotent: POSTing same key multiple times works without error

3. ✅ **Immediate consistency**: Changes are immediately visible
   - Created secrets appear in GET immediately
   - Updated secrets persist immediately
   - Deleted secrets disappear from GET immediately

## Remaining Open Questions

1. **Key validation**: What characters are allowed in secret keys?
   - Tested: `LANGSTAR_TEST_SECRET_456` (uppercase, underscores, numbers) ✅ Works
   - Untested: lowercase, hyphens, dots, spaces, special chars
   - Recommended: Follow env var conventions (UPPERCASE_WITH_UNDERSCORES)

2. **Batch operations**: How does the API handle multiple secrets in one POST?
   - OpenAPI spec shows array input `[{key, value}, {key, value}]`
   - Untested: Does it validate all before applying, or apply partially?
   - Untested: What happens if one key is invalid in a batch?

3. **List pagination**: Is there pagination for many secrets?
   - OpenAPI spec doesn't show pagination parameters
   - Tested workspace had 0-1 secrets, couldn't observe pagination
   - Likely no pagination (secrets are metadata, not large datasets)

4. **Error handling edge cases**:
   - Untested: Empty key string `""`
   - Untested: Very long key names (>255 chars?)
   - Untested: Duplicate keys in same POST request
   - Untested: Invalid characters in key names

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
