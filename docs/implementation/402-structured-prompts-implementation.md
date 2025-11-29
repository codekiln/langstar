# Structured Output Prompts Implementation Plan

**Issue**: [#402](https://github.com/codekiln/langstar/issues/402) - Add structured output prompt support to langstar CLI
**Milestone**: [ls-prompt-structured-outputs](https://github.com/codekiln/langstar/milestone/7)
**Date**: 2025-11-29
**Status**: ✅ Completed

## Executive Summary

This document describes the implementation of structured output prompts in Langstar, enabling users to create prompts with JSON Schema constraints that ensure LLM outputs conform to predefined structures.

### What Was Built

- **SDK Support** - `StructuredPrompt` types with LC-JSON serialization
- **CLI Integration** - `--schema` and `--schema-method` flags
- **Schema Validation** - Client-side JSON Schema validation before push
- **Full Round-trip** - Push and pull structured prompts to/from LangSmith

### Key Deliverables

| Component | Status | PRs |
|-----------|--------|-----|
| SDK Types | ✅ Complete | [#415](https://github.com/codekiln/langstar/pull/415) |
| SDK Client Methods | ✅ Complete | [#420](https://github.com/codekiln/langstar/pull/420) |
| CLI Commands | ✅ Complete | [#431](https://github.com/codekiln/langstar/pull/431) |
| Documentation | ✅ Complete | [#409](https://github.com/codekiln/langstar/issues/409) |

## Research Phase

### Research Report

**Issue**: [#398](https://github.com/codekiln/langstar/issues/398)
**Document**: [398-structured-output-prompts-scout.md](../research/398-structured-output-prompts-scout.md)

Key findings:
1. LangSmith stores prompts as LC-JSON serialized objects
2. `StructuredPrompt` class in Python SDK is the reference implementation
3. JSON Schema must be passed as dict, not Pydantic class
4. Two methods supported: `json_schema` and `function_calling`

### Design Decisions

**Issue**: [#403](https://github.com/codekiln/langstar/issues/403)
**Section**: [Research document Section 11](../research/398-structured-output-prompts-scout.md#11-design-decisions)

Key decisions:
- Use `--schema <FILE>` flag (matches dataset import pattern)
- Default method: `json_schema`
- Client-side validation before push
- No new environment variables required

### OpenAPI Validation

**Issue**: [#404](https://github.com/codekiln/langstar/issues/404)
**Document**: [402-structured-prompts-openapi-validation.md](../research/402-structured-prompts-openapi-validation.md)

Validated against LangSmith OpenAPI spec to ensure API compatibility.

## Implementation Phases

### Phase 1: SDK Types (#405, #415)

**Goal**: Create Rust types for structured prompts with LC-JSON serialization.

**Implementation**: [sdk/src/prompts.rs:18-199](../../sdk/src/prompts.rs)

#### Types Created

1. **`LcJson<T>`** - Generic LC-JSON wrapper
   ```rust
   pub struct LcJson<T> {
       pub lc: u8,
       pub type_: String,
       pub id: Vec<String>,
       pub kwargs: T,
       pub name: Option<String>,
   }
   ```

2. **`StructuredPrompt`** - Main structured prompt type
   ```rust
   pub struct StructuredPrompt {
       pub input_variables: Option<Vec<String>>,
       pub messages: Vec<LcJson<MessagePromptTemplateKwargs>>,
       pub schema_: Value,
       pub structured_output_kwargs: StructuredOutputKwargs,
   }
   ```

3. **`StructuredOutputKwargs`** - Method configuration
   ```rust
   pub struct StructuredOutputKwargs {
       pub method: String,  // "json_schema" or "function_calling"
   }
   ```

4. **Helper Types**
   - `MessagePromptTemplateKwargs` - Message template wrapper
   - `PromptTemplateKwargs` - Base prompt template

#### Schema Validation

```rust
pub fn validate_json_schema(schema: &Value) -> Result<()>
pub fn validate_method(method: &str) -> Result<()>
```

**Validation approach:**
- Uses `jsonschema` crate to compile and validate schemas
- Validates method is `json_schema` or `function_calling`
- Fails fast with clear error messages

### Phase 2: SDK Client Methods (#406, #420)

**Goal**: Implement push/pull methods for structured prompts.

**Implementation**: [sdk/src/client.rs](../../sdk/src/client.rs) (langchain_create_commit)

#### Push Logic

1. Validate schema with `validate_json_schema()`
2. Validate method with `validate_method()`
3. Build `StructuredPrompt` from CLI inputs
4. Wrap in `LcJson` format
5. Serialize to JSON manifest
6. POST to `/api/v1/commits/{owner}/{repo}/`

**Key code**:
```rust
let structured_prompt = StructuredPrompt {
    input_variables: Some(input_variables),
    messages: build_messages(template, input_variables),
    schema_: schema,
    structured_output_kwargs: StructuredOutputKwargs { method },
};

let manifest = structured_prompt.to_lc_json();
```

#### Pull Logic

Pull uses existing `langchain_get_commit()` method - no changes needed. The manifest field contains the full LC-JSON structure.

### Phase 3: CLI Integration (#407, #431)

**Goal**: Add `--schema` and `--schema-method` flags to `prompt push` command.

**Implementation**: [cli/src/commands/prompt.rs:76-113](../../cli/src/commands/prompt.rs)

#### CLI Flags

```rust
Push {
    // Existing flags...

    /// Path to JSON Schema file for structured output
    #[arg(long, value_name = "FILE")]
    schema: Option<std::path::PathBuf>,

    /// Structured output method: json_schema or function_calling
    #[arg(long, default_value = "json_schema")]
    schema_method: String,
}
```

#### Implementation Flow

1. Check if `--schema` flag provided
2. Read schema file from disk
3. Parse JSON with `serde_json`
4. Validate schema and method
5. Call SDK with schema parameters
6. Handle errors with user-friendly messages

**Error handling**:
```rust
let schema: Value = match std::fs::read_to_string(&schema_path) {
    Ok(content) => serde_json::from_str(&content)
        .map_err(|e| anyhow!("Schema file contains invalid JSON: {}", e))?,
    Err(e) => return Err(anyhow!("Failed to read schema file: {}", e)),
};

validate_json_schema(&schema)?;
validate_method(&schema_method)?;
```

### Phase 4: Documentation (#409)

**Goal**: Document the structured output prompts feature.

**Deliverables**:
1. ✅ README updates with examples
2. ✅ Usage guide: [docs/examples/structured-output-prompts.md](../examples/structured-output-prompts.md)
3. ✅ Implementation plan (this document)
4. ✅ Rustdoc comments on SDK types

## Testing Approach

### Unit Tests

**Location**: `sdk/src/prompts.rs`

Tests cover:
- LC-JSON serialization/deserialization
- Schema validation (valid and invalid schemas)
- Method validation
- StructuredPrompt construction

### Integration Tests

**Location**: `sdk/tests/prompts_integration.rs`

Tests cover:
- Push structured prompt to LangSmith (requires API key)
- Pull structured prompt from LangSmith
- Round-trip: push then pull, verify schema preserved

### Manual Testing

```bash
# Create test schema
cat > test-schema.json << 'EOF'
{
  "type": "object",
  "properties": {
    "answer": {"type": "string"},
    "confidence": {"type": "number"}
  },
  "required": ["answer"]
}
EOF

# Push structured prompt
cargo run -- prompt push \
  -o test -r structured-test \
  -t "Answer: {question}" \
  --schema test-schema.json

# Pull and verify
cargo run -- prompt pull test/structured-test
```

## Architecture Decisions

### Why LC-JSON Format?

**Decision**: Use LangChain's LC-JSON serialization format for manifests.

**Rationale**:
- LangSmith stores prompts in this format
- Python SDK uses this format
- Round-trip compatibility with Python ecosystem
- Structured and well-documented format

**Alternative considered**: Custom JSON format
- ❌ Would break Python SDK compatibility
- ❌ Would require custom deserialization on LangSmith side

### Why Client-Side Validation?

**Decision**: Validate JSON Schema on client before pushing.

**Rationale**:
- Fail fast with clear error messages
- Reduce API round-trips for invalid schemas
- Better user experience (immediate feedback)

**Implementation**: Uses `jsonschema` crate
```toml
[dependencies]
jsonschema = "0.21"
```

### Why PathBuf for Schema Argument?

**Decision**: Use `std::path::PathBuf` for `--schema` flag.

**Rationale**:
- Proper path handling across platforms
- Consistent with `dataset import --file` pattern
- Type-safe file path representation

**Alternative considered**: String
- ❌ Less type-safe
- ❌ Requires manual path validation

## Code References

### SDK

| File | Lines | Description |
|------|-------|-------------|
| `sdk/src/prompts.rs` | 18-70 | LC-JSON types and helpers |
| `sdk/src/prompts.rs` | 71-149 | StructuredPrompt types |
| `sdk/src/prompts.rs` | 150-232 | Schema validation functions |
| `sdk/src/client.rs` | (commit method) | Push/pull implementation |

### CLI

| File | Lines | Description |
|------|-------|-------------|
| `cli/src/commands/prompt.rs` | 76-113 | CLI flags definition |
| `cli/src/commands/prompt.rs` | (execute method) | Schema file handling |

### Tests

| File | Description |
|------|-------------|
| `sdk/src/prompts.rs` | Unit tests for types and validation |
| `sdk/tests/prompts_integration.rs` | Integration tests with LangSmith API |

## Future Enhancements

### Not in Scope (Intentional)

1. **Pydantic class support** - Users should export schema to JSON first
2. **Model binding** - No `include_model` parameter (Python SDK feature)
3. **Transform logic** - No RunnableSequence conversion
4. **Schema generation** - No automatic schema inference from templates

### Potential Future Work

1. **Schema library** - Common schemas for typical use cases
2. **Inline schema** - Accept schema as JSON string via `--schema-inline`
3. **Schema validation on pull** - Warn if pulled schema is invalid
4. **Schema diff** - Compare schemas between prompt versions
5. **OpenAPI to JSON Schema** - Convert OpenAPI specs to prompt schemas

## Lessons Learned

### What Went Well

1. **Research first** - Thorough research saved implementation time
2. **Validation early** - Client-side validation prevented many API errors
3. **Type safety** - Rust's type system caught serialization bugs
4. **Incremental PRs** - Splitting work into SDK → CLI → docs worked well

### Challenges

1. **LC-JSON complexity** - Nested structure took time to understand
2. **Schema validation** - Finding the right `jsonschema` crate version
3. **Error messages** - Balancing detail vs. simplicity

### Recommendations for Similar Features

1. Start with comprehensive research and experiments
2. Design CLI flags before implementation
3. Validate against OpenAPI specs early
4. Write unit tests alongside code
5. Document as you go, not after

## Related Issues

### Completed

- [#398](https://github.com/codekiln/langstar/issues/398) - Research
- [#403](https://github.com/codekiln/langstar/issues/403) - Design DX consistency
- [#404](https://github.com/codekiln/langstar/issues/404) - OpenAPI validation
- [#405](https://github.com/codekiln/langstar/issues/405) - SDK types
- [#406](https://github.com/codekiln/langstar/issues/406) - SDK client methods
- [#407](https://github.com/codekiln/langstar/issues/407) - CLI commands
- [#408](https://github.com/codekiln/langstar/issues/408) - Testing
- [#409](https://github.com/codekiln/langstar/issues/409) - Documentation

### Related Milestones

- [ls-prompt-structured-outputs](https://github.com/codekiln/langstar/milestone/7) - Parent milestone

## References

### External Documentation

- [LangChain Structured Output](https://python.langchain.com/docs/how_to/structured_output/)
- [JSON Schema Specification](https://json-schema.org/)
- [LangSmith Prompts API](https://docs.smith.langchain.com/)

### Internal Documentation

- Research: [398-structured-output-prompts-scout.md](../research/398-structured-output-prompts-scout.md)
- Usage guide: [structured-output-prompts.md](../examples/structured-output-prompts.md)
- OpenAPI validation: [402-structured-prompts-openapi-validation.md](../research/402-structured-prompts-openapi-validation.md)

## Summary

The structured output prompts feature is fully implemented and tested. Users can now:

1. Create JSON Schema files defining output structure
2. Push prompts with `--schema` flag
3. Pull prompts and view their schemas
4. Use prompts with LLMs to get structured, validated outputs

The implementation follows Langstar's design principles:
- ✅ Thin wrapper over LangSmith API
- ✅ Type-safe Rust implementation
- ✅ Automation-friendly CLI
- ✅ Clear error messages
- ✅ Comprehensive documentation

**Next steps**: Users should refer to the [usage guide](../examples/structured-output-prompts.md) for detailed examples and best practices.
