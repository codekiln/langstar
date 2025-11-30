# ls-secrets Feasibility Scout

**Issue**: #456
**Date**: 2025-11-30
**Status**: Complete

## Executive Summary

**Feasibility**: Go

The LangSmith API provides workspace secrets management via `/api/v1/workspaces/current/secrets` endpoints. The API uses an upsert pattern (POST for both create and update) and only returns secret keys (not values) for security. Implementation complexity is **Medium** (upgraded from Low-Medium due to LLM agent security requirements) with no blocking dependencies.

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

**Location**: `reference/experiments/456-ls-secrets/`
**Files**:
- `README.md` - Experiment documentation and results
- `test_secrets_api.py` - Python script for live API testing
- `experiment_output.log` - Full execution log

**Approach**: Live API testing + OpenAPI spec analysis

### Experiment Execution

**Date**: 2025-11-30
**Script**: Python script (`test_secrets_api.py`) testing complete CRUD lifecycle
**Test Secret Key**: `LANGSTAR_TEST_SECRET_456`

### Critical Finding #1: Permission Requirements

**Discovery**: Secret management operations require `workspaces:manage` permission.

| Operation | Endpoint | Standard Key | Elevated Key |
|-----------|----------|--------------|--------------|
| List secrets (GET) | `/api/v1/workspaces/current/secrets` | ✅ 200 OK | ✅ 200 OK |
| Create secret (POST) | `/api/v1/workspaces/current/secrets` | ❌ 403 Forbidden | ✅ 200 OK |
| Update secret (POST) | `/api/v1/workspaces/current/secrets` | ❌ 403 Forbidden | ✅ 200 OK |
| Delete secret (POST) | `/api/v1/workspaces/current/secrets` | ❌ 403 Forbidden | ✅ 200 OK |

**Error Response** (403 with standard key):
```json
{
  "detail": "Permission denied, you do not have the required permission workspaces:manage"
}
```

**Implication**: CLI must handle 403 gracefully with clear guidance on creating elevated API keys.

### Critical Finding #2: Complete CRUD Validation

✅ **ALL OPERATIONS WORK** with proper permissions!

**Tested and Confirmed**:
1. **Create**: POST with `{"key": "...", "value": "..."}` → 200 OK, `null` response
2. **Read**: GET returns `[{"key": "..."}]` → values never included (security ✅)
3. **Update**: POST with existing key → 200 OK, same as create (true upsert)
4. **Delete**: POST with `{"key": "...", "value": null}` → 200 OK, secret removed

**Key Behaviors**:
- ✅ **Upsert pattern confirmed**: No distinction between create/update in API
- ✅ **Deletion works**: `value: null` successfully removes secrets
- ✅ **Immediate consistency**: Changes visible immediately in GET
- ✅ **Idempotent**: POSTing same key multiple times succeeds
- ✅ **Security**: Values never returned, only keys
- ✅ **Response format**: POST returns `null`, GET returns key array

### Remaining Open Questions

Minor edge cases (not critical for initial implementation):

1. **Key validation**: What characters are allowed?
   - Tested: `LANGSTAR_TEST_SECRET_456` ✅ Works
   - Untested: lowercase, hyphens, dots, spaces
   - Recommend: Follow env var conventions (UPPERCASE_WITH_UNDERSCORES)

2. **Batch operations**: Array input behavior with multiple secrets
3. **Pagination**: Unlikely (secrets are metadata), but untested
4. **Error cases**: Empty keys, very long keys, invalid characters

**Status**: Defer to Phase 3 (SDK Integration Tests) for comprehensive validation.

## 5. Complexity Assessment

**Overall Complexity**: Medium (upgraded from Low-Medium due to LLM agent security requirements)

### Straightforward Aspects (Low Complexity)

- **Simple API**: Only 2 primary endpoints (GET list, POST upsert)
- **Simple schemas**: SecretKey (1 field), SecretUpsert (2 fields)
- **Standard auth**: Reuse existing Langstar SDK auth patterns
- **No pagination**: Simpler implementation (unless many secrets)
- **Clear security model**: Values never returned in responses
- **Well-documented**: OpenAPI spec is complete

### Moderate-High Complexity Aspects

- **LLM Agent Security**: Must prevent secret exposure to LLMs (see dedicated section below)
- **Multiple Input Methods**: Need --from-file, --interactive, stdin, --from-env patterns
- **Output Sanitization**: All outputs must be LLM-safe (no secret values)
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

### CRITICAL: LLM Agent Security Considerations

**Context**: Langstar CLI is expected to be wrapped in Claude Code skills and called by automated agents.

**Security Requirement**: Secret values must NEVER be exposed to LLMs in:
- Command-line arguments (visible in process lists, logs, shell history)
- Tool outputs or responses
- Error messages or debug logs
- Any text that could be read by the LLM

**Insecure Pattern** ❌:
```bash
# NEVER DO THIS - exposes secret to LLM
langstar secrets set ANTHROPIC_API_KEY sk-ant-abc123...
```

**Secure Patterns Required** ✅:

1. **File Input** (Recommended):
   ```bash
   # Secret value read from file, not exposed to LLM
   langstar secrets set ANTHROPIC_API_KEY --from-file ~/.secrets/anthropic.key

   # Or stdin redirect
   langstar secrets set ANTHROPIC_API_KEY < ~/.secrets/anthropic.key
   ```

2. **Interactive Prompt** (Manual Use):
   ```bash
   # Prompts for value with hidden input (like sudo)
   langstar secrets set ANTHROPIC_API_KEY --interactive
   # User types value (hidden), LLM never sees it
   ```

3. **Environment Variable** (Less Secure):
   ```bash
   # Reads from env var, slightly better than CLI arg
   ANTHROPIC_API_KEY=sk-ant-... langstar secrets set ANTHROPIC_API_KEY --from-env
   ```

**Output Requirements**:
- `set` command must NOT echo the secret value
- Error messages must NOT include secret values
- Success message: "✅ Secret ANTHROPIC_API_KEY updated" (no value)
- List command already safe (API returns keys only, not values)

**Mandatory Security Review**:

This issue requires a **dedicated security review ticket** in Phase 1.5 (before Phase 2 Design):
- Review CLI interface for secret exposure vectors
- Design secure input methods (--from-file, --interactive)
- Establish guidelines for LLM-safe command outputs
- Document secure patterns in skills/commands that wrap langstar
- Test with Claude Code skill wrapper to verify no leakage

**Complexity Impact**: Raises CLI complexity from Low-Medium to **Medium** due to:
- Need for multiple input methods (flags, files, stdin, interactive)
- Output sanitization requirements
- Security testing and validation
- Documentation for skill developers

**Blocker Status**: Not a blocker, but **must be addressed in Phase 1.5 (before Phase 2 Design)** before CLI implementation.

**Decision**: Go

**Rationale**:
1. ✅ Simple, well-documented API with only 2 primary endpoints
2. ✅ Medium complexity is manageable (increased due to LLM security requirements)
3. ✅ No blocking dependencies
4. ✅ Clear business value: Required for model provider configurations (#453)
5. ✅ Sound API security model (values never exposed in responses)
6. ✅ Can be implemented incrementally with security-first approach

### Proceed to Phase 0?

**YES - Recommend creating milestone and parent issue.**

**Supporting factors**:
1. ✅ Simple, well-documented API (2 endpoints)
2. ✅ Medium complexity is manageable (upgraded due to LLM agent security)
3. ✅ No blocking technical dependencies
4. ✅ Clear business value (enables model provider configs #453)
5. ✅ Sound API security model (values never exposed in responses)
6. ✅ Can be implemented incrementally

**Risk factors** (mitigable):
1. ⚠️ No Python SDK precedent (mitigated by complete OpenAPI spec)
2. ⚠️ Delete behavior unclear (mitigated by pre-implementation experiments)
3. ⚠️ Validation rules unknown (mitigated by following common patterns)
4. ⚠️ **LLM security critical** (mitigated by dedicated Phase 1.5 security review)

### Considerations for Phase 0 (If Proceeding)

When creating the parent issue and milestone, consider:

**MANDATORY: Security Review Sub-Issue**:

Create a **dedicated Phase 1.5 security review sub-issue** BEFORE Phase 2 (Design):
- **Title**: `{parent}.1.5-security Security review for LLM agent safety`
- **Scope**:
  - Review CLI interface for secret exposure vectors
  - Design secure input methods (--from-file, --interactive, stdin)
  - Establish output sanitization requirements
  - Create guidelines for Claude Code skills that wrap langstar
  - Threat modeling: LLM reads command history, logs, process lists
  - Test plan: Verify no secret leakage to LLM in any scenario
- **Deliverable**: Security design document in `docs/implementation/`
- **Blocks**: Phase 2 (Design) and all subsequent CLI phases

**Core Scope** (Phases 3-8):
- SDK types: `SecretKey`, `SecretUpsert`
- SDK methods: `list_secrets()`, `upsert_secrets()`
- CLI commands with secure input:
  - `langstar secrets list` (already safe - API returns keys only)
  - `langstar secrets set KEY --from-file <path>` (secure)
  - `langstar secrets set KEY --interactive` (secure for manual use)
  - `langstar secrets set KEY < <(echo $VALUE)` (stdin support)
  - `langstar secrets delete KEY` (pending `value: null` validation)

**Optional Enhancements** (separate issues after core):
- Bulk operations: `langstar secrets import --from-env-file .env`
- Bulk export: `langstar secrets export > keys.txt` (keys only, no values)
- Validation: Warn if key doesn't follow conventions (e.g., not uppercase)

**Out of Scope** (different milestones):
1. **Encrypted endpoint**: `/api/v1/workspaces/current/secrets/encrypted` (internal use only)
2. **Secret rotation**: Automated rotation/expiration
3. **Secret templates**: Pre-configured sets for common providers
4. **Model provider integration**: Separate milestone (already scouted #453)

**Pre-Implementation Experiments Recommended** (Phase 3):
- Test `value: null` for deletion
- Test key naming validation (uppercase, underscores)
- Test conflict handling (update existing key)

## 7. Success Criteria

**This Phase 0.0 scout issue is complete when:**

- [x] Research report documents API endpoints and schemas
- [x] Complexity assessed and rated (Medium)
- [x] Technical blockers identified (None found)
- [x] Feasibility recommendation provided (Go - proceed to Phase 0)
- [x] Experiments documented (spec analysis in `reference/experiments/456-ls-secrets/`)
- [x] PR merged directly to main (no milestone attached)

**Next step**: If stakeholders agree, create milestone and parent issue (Phase 0) following `docs/dev/feature-development-process.md`

## References

- **Scout Issue**: #456
- **Parent Issue**: #453 (identified secrets as dependency)
- **OpenAPI Spec**: `reference/openapi/langchain/langsmith/openapi.json`
- **Experiments**: `reference/experiments/456-ls-secrets/README.md`
- **Feature Development Process**: `docs/dev/feature-development-process.md`
- **Related Code**:
  - `sdk/src/deployments.rs:5-12` (DeploymentSecret - different concept)
  - `cli/src/commands/graph.rs:404-416` (uses DeploymentSecret)
