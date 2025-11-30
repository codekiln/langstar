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

This is a **research-only experiment** with no production code changes:

✅ Review OpenAPI specification for secrets endpoints
✅ Understand request/response schemas
✅ Document security patterns (value masking)
✅ Identify API operations available
❌ NO Python scripts written (API behavior understood from spec)
❌ NO live API testing performed

## Open Questions

1. **Delete behavior**: Can we delete a secret by setting `value: null` in POST, or are secrets permanent once created?
2. **Validation**: Does the API validate secret key naming (e.g., uppercase only, no spaces)?
3. **List limits**: Is there pagination for listing secrets if a workspace has many?
4. **Update detection**: How does the API handle updating an existing secret vs creating new?
5. **Encrypted endpoint usage**: When should we use the encrypted endpoint vs the regular endpoint?

## Recommendations for Future Experiments

If implementing `ls-secrets`, consider running these experiments:

1. **Create secret**: Test POST with a new secret key/value
2. **Update secret**: Test POST with existing key and different value
3. **Delete secret**: Test POST with `value: null` to see if secret is deleted
4. **List secrets**: Test GET and observe pagination behavior
5. **Invalid keys**: Test POST with invalid key names (spaces, special chars, etc.)
6. **Encrypted secrets**: Test GET encrypted with agent_builder service

## References

- OpenAPI Spec: `/workspace/reference/openapi/langchain/langsmith/openapi.json`
- Secrets endpoints: Lines containing `/api/v1/workspaces/current/secrets`
- Schema definitions: `SecretKey`, `SecretUpsert`, `InternalSecretsResponse`
