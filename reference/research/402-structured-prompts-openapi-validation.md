# Structured Output Prompts OpenAPI Validation Report

**Issue**: [#404](https://github.com/codekiln/langstar/issues/404) - Validate structured output prompt design against LangSmith OpenAPI spec
**Milestone**: [#7 - ls-prompt-structured-outputs](https://github.com/codekiln/langstar/milestone/7)
**Date**: 2025-11-29

## Executive Summary

This report validates the research findings from [#398](https://github.com/codekiln/langstar/issues/398) against the LangSmith OpenAPI specification. All key findings are **confirmed** by the spec, with important clarifications about the flexible manifest structure.

## Validation Results

### ✅ 1. Endpoint Validation

**Research Finding**: POST /commits/{owner}/{repo}/ endpoint for creating prompt commits

**OpenAPI Validation**:

- **Status**: ✅ **CONFIRMED**
- **Path**: `/api/v1/commits/{owner}/{repo}`
- **Method**: POST
- **Request Schema**: `CreateRepoCommitRequest`
- **Location**: `reference/api-specs/langsmith/prompt-endpoints.json:1-199`

**Schema Details**:

```json
{
  "properties": {
    "manifest": {
      "additionalProperties": true,
      "type": "object",
      "title": "Manifest"
    },
    "parent_commit": {
      "anyOf": [{"type": "string"}, {"type": "null"}],
      "title": "Parent Commit"
    },
    "example_run_ids": {
      "anyOf": [
        {"items": {"type": "string", "format": "uuid"}, "type": "array"},
        {"type": "null"}
      ]
    },
    "skip_webhooks": {
      "anyOf": [{"type": "boolean"}, {"items": {"type": "string", "format": "uuid"}, "type": "array"}],
      "default": false
    }
  },
  "required": ["manifest"]
}
```

**Key Observations**:

- The `manifest` field is **required**
- `manifest` allows arbitrary JSON (`additionalProperties: true`)
- No validation of internal manifest structure by the API

### ✅ 2. Manifest Structure Validation

**Research Finding**: Manifests use LC-JSON format with specific structure

**OpenAPI Validation**:

- **Status**: ✅ **CONFIRMED WITH CLARIFICATION**
- **Schema**: `CreateRepoCommitRequest.manifest` and `CommitManifestResponse.manifest`
- **Type**: `object` with `additionalProperties: true`

**Clarification**:
The OpenAPI spec intentionally **does not constrain** the manifest structure. This is by design - the API treats manifests as opaque JSON blobs, allowing LangChain to evolve the serialization format without API changes.

**Implication for Langstar**:

- Langstar must implement LC-JSON serialization logic in SDK
- No server-side validation of manifest structure
- Client-side schema validation recommended before pushing

### ✅ 3. Schema Field Validation

**Research Finding**: `StructuredPrompt` has `schema_` and `structured_output_kwargs` fields

**OpenAPI Validation**:

- **Status**: ✅ **CONFIRMED AS CLIENT-SIDE CONCERN**
- The OpenAPI spec does **not** define these fields because they exist within the opaque `manifest` object
- These fields are part of the LC-JSON format (LangChain's serialization)

**LC-JSON Structure** (from research experiments):

```json
{
  "lc": 1,
  "type": "constructor",
  "id": ["langchain_core", "prompts", "structured", "StructuredPrompt"],
  "kwargs": {
    "messages": [...],
    "schema_": {
      "type": "object",
      "properties": {...}
    },
    "structured_output_kwargs": {
      "method": "json_schema"
    }
  }
}
```

**Validation**:

- ✅ `schema_` field exists in `kwargs` (stores JSON Schema dict)
- ✅ `structured_output_kwargs.method` stores the method selection

### ✅ 4. Method Options Validation

**Research Finding**: Two methods supported: `"json_schema"` and `"function_calling"`

**OpenAPI Validation**:

- **Status**: ✅ **CONFIRMED AS CLIENT-SIDE CONCERN**
- Not enforced by API (opaque manifest)
- Validated by experiments in #398

**From Research Experiments**:
Both methods work when included in the manifest's `structured_output_kwargs`:

- `"json_schema"` - Use JSON Schema mode
- `"function_calling"` - Use function calling mode

**No API-level enum validation** - this is enforced by LangChain deserialization.

## Discrepancies

### None Found

All research findings align with the OpenAPI spec. The key insight is understanding the **API boundary**:

| Layer     | Validation                                       | Ownership     |
| --------- | ------------------------------------------------ | ------------- |
| API Layer | `manifest` is valid JSON object                  | LangSmith API |
| SDK Layer | LC-JSON format, schema_ structure, method values | Langstar SDK  |

## Key Findings Summary

| Finding                    | Research          | OpenAPI Spec                    | Status         |
| -------------------------- | ----------------- | ------------------------------- | -------------- |
| POST /commits endpoint     | ✅ Documented     | ✅ Exists                       | ✅ Match       |
| Manifest is flexible JSON  | ✅ Documented     | ✅ `additionalProperties: true` | ✅ Match       |
| LC-JSON format             | ✅ Documented     | ⚪ Out of scope                 | ✅ SDK concern |
| `schema_` field            | ✅ In experiments | ⚪ Not in spec                  | ✅ SDK concern |
| `structured_output_kwargs` | ✅ In experiments | ⚪ Not in spec                  | ✅ SDK concern |
| Method options             | ✅ Validated      | ⚪ Not enumerated               | ✅ SDK concern |

**Legend**:

- ✅ Confirmed
- ⚪ Expected to be out of scope
- ❌ Conflict (none found)

## Implementation Guidance

### For Langstar SDK (#405-#406)

1. **Create StructuredPrompt Type**
   - Fields: `messages`, `schema_`, `structured_output_kwargs`
   - Validate `schema_` is valid JSON Schema
   - Validate `method` is "json_schema" or "function_calling"

2. **Implement LC-JSON Serialization**
   - Serialize to format shown in research experiments
   - Use module path: `["langchain_core", "prompts", "structured", "StructuredPrompt"]`
   - Support nested message templates

3. **Add Client Methods**
   - `push_structured_prompt()` - Serialize and POST to /commits
   - `pull_structured_prompt()` - GET from /commits and deserialize

### For Langstar CLI (#407)

1. **Add `--schema` Flag**
   - Accept JSON Schema file path
   - Load and validate schema
   - Pass to SDK push method

2. **Add `--method` Flag**
   - Values: `json_schema`, `function_calling`
   - Default: `json_schema`

## References

### API Spec Fragments

- **Endpoints**: `reference/api-specs/langsmith/prompt-endpoints.json`
- **Schemas**: `reference/api-specs/langsmith/prompt-schemas.json`
- **Full Spec**: `reference/openapi/langchain/langsmith/openapi.json`

### Related Research

- [#398 Research Report](../../docs/research/398-structured-output-prompts-scout.md)
- [#403 Design Consistency](https://github.com/codekiln/langstar/issues/403)

### OpenAPI Details

- **Provenance**: Fetched 2025-11-29 from `https://api.smith.langchain.com/openapi.json`
- **Size**: 638K
- **MANIFEST**: `reference/openapi/langchain/langsmith/MANIFEST.md`

### Extraction Commands Used

```bash
# Extract prompt endpoints
jq '.paths | with_entries(select(.key | test("^/api/v1/(repos|commits)")))' \
  openapi.json > ../../api-specs/langsmith/prompt-endpoints.json

# Extract prompt schemas
jq '.components.schemas | with_entries(select(.key | test("[Rr]epo|[Cc]ommit|[Pp]rompt|[Mm]anifest"; "i")))' \
  openapi.json > ../../api-specs/langsmith/prompt-schemas.json
```

## Conclusion

The research findings from #398 are **validated and accurate**. The OpenAPI spec confirms:

1. ✅ POST /commits endpoint exists with correct structure
2. ✅ Manifest field is flexible JSON (by design)
3. ✅ LC-JSON format is client-side serialization (not API-enforced)
4. ✅ Schema and method handling is SDK responsibility

**No blocking issues** identified. Implementation can proceed to SDK phase (#405-#406).
