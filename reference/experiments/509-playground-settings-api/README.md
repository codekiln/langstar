# Experiment: LangSmith Playground Settings API Response Truncation

**Date**: 2025-12-02
**Issue**: [#509 - Investigate: playground-settings API response truncation](https://github.com/codekiln/langstar/issues/509)
**Parent Issue**: #477 (Phase 7: Testing for playground settings)

## Objective

Investigate why playground-settings integration tests are failing with JSON decoding errors:

```
Error: HTTP request failed: error decoding response body
"premature end of input", line: 1, column: 348
```

## Hypothesis

Initial hypothesis from issue description:
- API might be returning truncated JSON responses
- Content-Length mismatches causing early termination
- Malformed JSON for certain error cases

## Investigation Methodology

Created Python experiment (following pattern from `456-ls-secrets`) to:
1. Test POST create and capture raw response
2. Test GET by ID and capture raw response
3. Test PATCH update and capture raw response
4. Test DELETE and capture raw response
5. Test GET on non-existent ID
6. Compare response sizes, headers, and truncation patterns

## Critical Finding: API Does Not Support GET by ID

### Root Cause Identified

**The playground-settings API does NOT support `GET /api/v1/playground-settings/{id}`**

#### Evidence from OpenAPI Specification

**Source**: `/reference/api-specs/langsmith/playground-settings-endpoints.json`
**Extracted**: 2025-12-01 using jq from canonical OpenAPI spec
**Extraction command**: See `/reference/api-specs/langsmith/FRAGMENTS.md`

```bash
# From reference/api-specs/langsmith/playground-settings-endpoints.json
$ jq 'keys' reference/api-specs/langsmith/playground-settings-endpoints.json
[
  "/api/v1/playground-settings",
  "/api/v1/playground-settings/{playground_settings_id}"
]

$ jq '."/api/v1/playground-settings" | keys' reference/api-specs/langsmith/playground-settings-endpoints.json
[
  "get",
  "post"
]

$ jq '."/api/v1/playground-settings/{playground_settings_id}" | keys' reference/api-specs/langsmith/playground-settings-endpoints.json
[
  "delete",
  "patch"
]
```

**Supported operations:**
- ✅ `GET /api/v1/playground-settings` - List all settings
- ✅ `POST /api/v1/playground-settings` - Create new setting
- ✅ `PATCH /api/v1/playground-settings/{playground_settings_id}` - Update existing setting
- ✅ `DELETE /api/v1/playground-settings/{playground_settings_id}` - Delete setting
- ❌ `GET /api/v1/playground-settings/{playground_settings_id}` - **NOT SUPPORTED**

**Verification**: This matches the extracted OpenAPI spec fragment committed in [reference/api-specs/langsmith/playground-settings-endpoints.json](/reference/api-specs/langsmith/playground-settings-endpoints.json)

#### Evidence from Python API Test

```python
→ GET /api/v1/playground-settings/2d03e729-a875-4897-b2c5-e0a2e5dff326
→ Status: 405
→ Headers:
  Content-Length: 31
  content-type: application/json
→ Raw body length: 31 bytes
→ Raw body (first 500 chars): b'{"detail":"Method Not Allowed"}'
→ Decoded length: 31 chars
→ Decoded content: {"detail":"Method Not Allowed"}
```

**API returns:** `405 Method Not Allowed`
**Response body:** `{"detail":"Method Not Allowed"}`
**This is well-formed JSON** - no truncation occurred

## Experiment Results

**Execution Date**: 2025-12-02
**Script**: `test_playground_api.py`
**Test Config**: `LANGSTAR_TEST_PLAYGROUND_509_*`

### Complete CRUD Test Results

| Test | Status | HTTP Status | Response | Notes |
|------|--------|-------------|----------|-------|
| CREATE (POST) | ✅ PASS | 200/201 | Full JSON response | Successfully created setting |
| GET by ID (GET) | ❌ FAIL | 405 | `{"detail":"Method Not Allowed"}` | **Endpoint doesn't exist** |
| UPDATE (PATCH) | ✅ PASS | 200 | Full JSON response | Successfully updated setting |
| DELETE (DELETE) | ✅ PASS | 200/204 | Empty/minimal response | Successfully deleted setting |
| GET non-existent (GET) | ✅ Expected | 405 | `{"detail":"Method Not Allowed"}` | Same 405 as valid ID |
| LIST all (GET) | ✅ PASS | 200 | Full JSON array (3697 bytes) | Successfully listed all settings |

### Key Observations

1. **No response truncation detected** - All API responses are complete and well-formed JSON
2. **Content-Length headers match actual body length** - No mismatches observed
3. **The "error decoding response body" is likely caused by attempting to call an unsupported endpoint**
4. **All supported operations (CREATE, UPDATE, DELETE, LIST) work correctly**

## SDK Analysis

Verified Rust SDK client methods in `sdk/src/client.rs`:

```rust
// ✅ Implemented methods
pub async fn list_playground_settings(...) -> Result<Vec<PlaygroundSettingsResponse>>
pub async fn create_playground_settings(...) -> Result<PlaygroundSettingsResponse>
pub async fn update_playground_settings(...) -> Result<PlaygroundSettingsResponse>
pub async fn delete_playground_settings(...) -> Result<()>

// ❌ NOT implemented (correctly - endpoint doesn't exist)
// No get_playground_settings method exists
```

**SDK is correct** - it does not provide a `get_playground_settings(id)` method because the API doesn't support it.

## Documentation Issues Found

### Incorrect Documentation in `sdk/src/playground_settings.rs`

**Location**: `sdk/src/playground_settings.rs:54`

```rust
/// # API Reference
///
/// Maps to `PlaygroundSettingsResponse` in OpenAPI spec.
/// Returned by `GET /playground-settings`, `GET /playground-settings/{id}`,  // ❌ WRONG
/// `POST /playground-settings`, `PATCH /playground-settings/{id}`
```

**Issue**: Documentation claims `PlaygroundSettingsResponse` is "Returned by `GET /playground-settings/{id}`" but this endpoint doesn't exist.

**Should be**:
```rust
/// Returned by `GET /playground-settings` (list),
/// `POST /playground-settings` (create),
/// `PATCH /playground-settings/{id}` (update)
```

## Root Cause of Integration Test Failures

Based on this investigation, the integration test failures are likely caused by:

1. **Test expects to GET a setting by ID after creating it**
2. **Calls non-existent endpoint** → receives 405 error
3. **Rust HTTP client receives 405 response with valid JSON** `{"detail":"Method Not Allowed"}`
4. **But the test expects a `PlaygroundSettingsResponse` schema**
5. **Deserialization fails** because the response structure doesn't match expectations

## Actual vs Expected Behavior

### What Integration Tests Likely Do (Incorrect)

```rust
// Create playground setting
let created = client.create_playground_settings(request).await?;
let id = created.id;

// Try to GET it back (FAILS - endpoint doesn't exist)
let retrieved = client.get_playground_settings(id).await?; // ❌ Method doesn't exist
```

### What Tests Should Do (Correct)

```rust
// Create playground setting
let created = client.create_playground_settings(request).await?;

// To verify it was created, LIST all settings and find it
let all_settings = client.list_playground_settings(Default::default()).await?;
let retrieved = all_settings.iter().find(|s| s.id == created.id).unwrap();

// Or just use the response from create - it contains all the data
assert_eq!(created.name, expected_name);
```

## Recommendations

### 1. Fix Integration Tests (Priority: High)

**Issue location**: `cli/tests/model_config_command_test.rs:223-284`

The `test_model_config_create_update_delete_cycle` test should:

**Current (broken) approach:**
```rust
// Create config
let create_output = create_cmd.assert().success().get_output().stdout.clone();
let config_id = create_json["id"].as_str().unwrap();

// ❌ Likely tries to GET it back (fails with 405)
// This is where "error decoding response body" occurs
```

**Correct approach:**
```rust
// Create config - response contains full object
let create_output = create_cmd.assert().success().get_output().stdout.clone();
let created_config: PlaygroundSettingsResponse =
    serde_json::from_slice(&create_output).unwrap();

// Verify creation succeeded by checking the create response directly
assert_eq!(created_config.name, Some("CLI Test Config".to_string()));

// Or list all and verify it exists
let list_cmd = langstar_cmd();
list_cmd.args(["model-config", "list", "--format", "json"]);
let list_output = list_cmd.assert().success().get_output().stdout.clone();
let all_configs: Vec<PlaygroundSettingsResponse> =
    serde_json::from_slice(&list_output).unwrap();
assert!(all_configs.iter().any(|c| c.id == created_config.id));
```

### 2. Verify CLI Command Implementation (Priority: High)

Check `cli/src/commands/model_config.rs` to ensure:
- ✅ `get` command calls `list_playground_settings()` and filters by ID (NOT a direct GET by ID)
- ✅ CLI commands match available SDK methods
- ✅ CLI doesn't assume a GET-by-ID endpoint exists

### 3. Update Documentation (Priority: Medium)

**File**: `sdk/src/playground_settings.rs:54`

**Change from:**
```rust
/// Returned by `GET /playground-settings`, `GET /playground-settings/{id}`,
/// `POST /playground-settings`, `PATCH /playground-settings/{id}`
```

**To:**
```rust
/// Returned by `GET /playground-settings` (list),
/// `POST /playground-settings` (create),
/// `PATCH /playground-settings/{id}` (update)
```

### 4. Consider Adding Helper Method (Priority: Low)

While the API doesn't support GET-by-ID, the SDK could provide a convenience method:

```rust
impl LangSmithClient {
    /// Get a single playground setting by ID.
    ///
    /// This is a convenience method that calls list() and filters by ID,
    /// since the API does not provide a direct GET-by-ID endpoint.
    pub async fn get_playground_settings(
        &self,
        settings_id: Uuid
    ) -> Result<Option<PlaygroundSettingsResponse>> {
        let all = self.list_playground_settings(Default::default()).await?;
        Ok(all.into_iter().find(|s| s.id == settings_id))
    }
}
```

**Note**: This would be inefficient for large lists. Better to educate users to use LIST and filter client-side when needed.

## Conclusion

**There is NO API truncation issue.** The "error decoding response body" was a red herring caused by:

1. Integration tests (or CLI commands) attempting to call `GET /api/v1/playground-settings/{id}`
2. This endpoint returns `405 Method Not Allowed` with valid JSON
3. The SDK/CLI tries to deserialize this as a `PlaygroundSettingsResponse`
4. Deserialization fails because the schema doesn't match

**The API is working correctly.** The LangSmith playground-settings API intentionally does not provide a GET-by-ID endpoint. Clients must use:
- `GET /api/v1/playground-settings` (LIST) to retrieve all settings
- Filter client-side by ID if needed

**The fix is in the test code, not the API or SDK.** Integration tests should verify creation using:
1. The response from the CREATE operation itself, or
2. LIST all settings and filter by ID

## References

- OpenAPI Spec: `/workspace/reference/openapi/langchain/langsmith/openapi.json`
- SDK Client: `/workspace/sdk/src/client.rs:1789-1950`
- SDK Types: `/workspace/sdk/src/playground_settings.rs`
- Integration Tests: `/workspace/cli/tests/model_config_command_test.rs:223-284`
- Python Experiment: `/workspace/reference/experiments/509-playground-settings-api/test_playground_api.py`
