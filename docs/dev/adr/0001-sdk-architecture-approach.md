# ADR-0001: SDK Architecture Approach

**Status:** Accepted  
**Date:** 2025-11-13  
**Decision Makers:** Langstar Development Team  
**Related Issues:** #106, #115

## Context

Langstar currently uses a 100% hand-written SDK in the prototype phase. As the project matures, we need to decide on a sustainable SDK architecture that balances:

- **API Coverage**: Comprehensive access to all LangSmith and LangGraph APIs
- **Maintainability**: Easy to update when upstream APIs change
- **Ergonomics**: Developer-friendly, idiomatic Rust APIs
- **Type Safety**: Leverage Rust's type system for correctness
- **Scalability**: Support for growing API surface area

The existing infrastructure includes:
- `tools/generate_sdk.sh` - OpenAPI code generation script (not yet executed)
- `tools/specs/` - Directory for OpenAPI specifications (empty)
- `sdk/src/generated/` - Directory for generated code (placeholder only)
- Manual implementations in `sdk/src/` (client.rs, prompts.rs, assistants.rs, deployments.rs, organization.rs)

### Current State Analysis

**Manual SDK Strengths:**
- ✅ Full control over API design and ergonomics
- ✅ Tailored to Rust idioms and conventions
- ✅ Clean abstractions that hide complexity
- ✅ Excellent documentation and examples

**Manual SDK Weaknesses:**
- ❌ Limited API coverage (only implements most-used endpoints)
- ❌ Time-consuming to maintain and extend
- ❌ Risk of drift from upstream API changes
- ❌ No automatic updates when APIs evolve
- ❌ Scaling to full API coverage is impractical

## Decision

We adopt a **Layered "Manual-Over-Generated" Architecture** with three distinct layers:

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Layer (User-Facing)                 │
│              (cli/src/commands/, cli/src/main.rs)           │
│                                                              │
│  • Ergonomic command-line interface                         │
│  • Output formatting (tables, JSON, YAML)                   │
│  • User experience optimizations                            │
└────────────────────────┬────────────────────────────────────┘
                         │ calls
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Manual SDK Layer (High-Level API)              │
│         (sdk/src/*.rs - client, prompts, assistants)        │
│                                                              │
│  • Idiomatic Rust APIs with builder patterns               │
│  • Domain-specific abstractions                            │
│  • Authentication and error handling                        │
│  • Convenience methods and smart defaults                  │
│  • Human-curated documentation                             │
└────────────────────────┬────────────────────────────────────┘
                         │ calls
                         ▼
┌─────────────────────────────────────────────────────────────┐
│            Generated SDK Layer (Low-Level API)              │
│              (sdk/src/generated/langsmith/,                 │
│               sdk/src/generated/langgraph/)                 │
│                                                              │
│  • Pure OpenAPI-generated Rust clients                     │
│  • 100% API coverage (all endpoints)                       │
│  • Auto-generated from OpenAPI specs                       │
│  • Mechanical, non-idiomatic (but complete)                │
│  • Direct mapping to HTTP endpoints                        │
└─────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

#### 1. Generated SDK Layer (Bottom)
**Location:** `sdk/src/generated/langsmith/`, `sdk/src/generated/langgraph/`
**Purpose:** Comprehensive, auto-generated API clients

**Characteristics:**
- Generated from OpenAPI specifications using `openapi-generator-cli`
- One-to-one mapping with upstream API endpoints
- Minimal human intervention
- Complete API coverage (every endpoint, every parameter)
- May be verbose or non-idiomatic
- Used as the foundation for manual SDK layer

**Example:**
```rust
// Generated code (may be verbose)
pub struct LangsmithApiClient {
    // ... auto-generated fields
}

impl LangsmithApiClient {
    pub async fn list_repos_api_v1_repos_get(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        query: Option<String>,
        // ... all possible parameters
    ) -> Result<ListReposResponse, Error> {
        // ... auto-generated implementation
    }
}
```

#### 2. Manual SDK Layer (Middle)
**Location:** `sdk/src/*.rs`
**Purpose:** Ergonomic, idiomatic Rust API

**Characteristics:**
- Hand-written Rust code wrapping generated SDK
- Builder patterns, smart defaults, convenience methods
- Domain-specific abstractions (e.g., `PromptClient`, `DeploymentClient`)
- Rich error handling with context
- Human-written documentation and examples
- Focused on common use cases and workflows

**Example:**
```rust
// Manual ergonomic wrapper
pub struct PromptClient<'a> {
    client: &'a LangchainClient,
    generated_client: &'a GeneratedLangsmithClient, // Wraps generated layer
}

impl<'a> PromptClient<'a> {
    /// List all prompts with smart defaults
    pub async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Prompt>> {
        // Calls generated layer with sensible defaults
        self.generated_client
            .list_repos_api_v1_repos_get(
                Some(limit.unwrap_or(20) as i32),
                Some(offset.unwrap_or(0) as i32),
                None, // query
                // ... other defaults
            )
            .await
            .map_err(|e| LangstarError::from(e))
            .map(|resp| resp.repos)
    }
}
```

#### 3. CLI Layer (Top)
**Location:** `cli/src/`
**Purpose:** User-facing command-line interface

**Characteristics:**
- Consumes manual SDK layer only
- Command structure and argument parsing
- Output formatting (tables, JSON, colors)
- Interactive features and prompts
- User experience optimizations

**Example:**
```rust
// CLI command using manual SDK
pub async fn list_prompts(client: &LangchainClient, args: &ListArgs) -> Result<()> {
    let prompt_client = PromptClient::new(client);
    let prompts = prompt_client.list(args.limit, args.offset).await?;
    
    // Format and display results
    print_table(prompts)?;
    Ok(())
}
```

### Transition Strategy

**Phase 1 (Current - Prototype):**
- ✅ Manual SDK only
- ✅ No generated code
- Focus: Prove concept, validate API design

**Phase 2 (Implementation - Issue #116):**
- Generate low-level SDK from OpenAPI specs
- Refactor existing manual SDK to wrap generated SDK
- Maintain backward compatibility in CLI
- Test extensively to ensure no regressions

**Phase 3 (Maturity):**
- All new endpoints: implement via generated SDK first, add manual wrappers as needed
- Manual layer focuses on high-value abstractions
- Generated layer provides comprehensive fallback

### Code Organization

```
sdk/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API exports
│   ├── client.rs                   # HTTP client wrapper (manual)
│   ├── auth.rs                     # Authentication (manual)
│   ├── error.rs                    # Error types (manual)
│   ├── prompts.rs                  # Prompts client (manual wrapper)
│   ├── assistants.rs               # Assistants client (manual wrapper)
│   ├── deployments.rs              # Deployments client (manual wrapper)
│   ├── organization.rs             # Organization client (manual wrapper)
│   └── generated/
│       ├── README.md               # Generation instructions
│       ├── langsmith/              # Generated LangSmith client
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── models/         # Generated models
│       │       └── apis/           # Generated API clients
│       └── langgraph/              # Generated LangGraph client
│           ├── Cargo.toml
│           └── src/
│               ├── lib.rs
│               ├── models/
│               └── apis/
└── tests/                          # Integration tests
```

### Workspace Configuration

Generated clients will be separate crates within the workspace:

```toml
# Root Cargo.toml
[workspace]
members = [
    "cli",
    "sdk",
    "sdk/src/generated/langsmith",
    "sdk/src/generated/langgraph",
]

# sdk/Cargo.toml
[dependencies]
langsmith-client = { path = "src/generated/langsmith" }
langgraph-client = { path = "src/generated/langgraph" }
```

## Consequences

### Positive

1. **Comprehensive API Coverage**: Generated layer ensures 100% API coverage automatically
2. **Ergonomic APIs**: Manual layer provides idiomatic Rust experience for common operations
3. **Easy Updates**: Re-generate when OpenAPI specs change, manual layer adapts incrementally
4. **Clear Separation**: Each layer has well-defined responsibilities
5. **Gradual Migration**: Can transition from manual-only to layered approach incrementally
6. **Future-Proof**: Scales as LangChain APIs grow and evolve
7. **Type Safety**: Full compile-time guarantees from OpenAPI schemas
8. **Fallback Options**: Advanced users can access low-level generated API directly

### Negative

1. **Additional Complexity**: Three layers instead of one requires more cognitive overhead
2. **Build Time**: Generating and compiling generated code adds to build times
3. **Coordination Overhead**: Changes to OpenAPI specs require updating manual wrappers
4. **Testing Surface**: Need to test both generated and manual layers
5. **Documentation Split**: Need to document both layers separately
6. **Initial Migration Effort**: Refactoring current manual SDK to wrap generated code takes time

### Mitigation Strategies

1. **Clear Documentation**: Document layer boundaries and when to use each
2. **CI/CD Integration**: Automate spec fetching and code generation (Phase 3)
3. **Incremental Adoption**: Migrate endpoints to layered approach gradually
4. **Smart Defaults**: Manual layer hides complexity of generated layer
5. **Testing Strategy**: Focus integration tests on manual layer, unit tests on generated layer

## Alternatives Considered

### Alternative 1: Pure Generated SDK

**Description:** Use only OpenAPI-generated code with no manual wrappers.

**Pros:**
- Simplest architecture
- Fastest updates when API changes
- No manual maintenance
- Complete API coverage automatically

**Cons:**
- Non-idiomatic Rust APIs (verbose, awkward)
- Poor developer experience
- No domain-specific abstractions
- Difficult to add ergonomic helpers
- Auto-generated documentation often inadequate

**Rejected Because:** Developer experience would suffer significantly. Generated code is mechanical and often not idiomatic, making it hard to use correctly.

### Alternative 2: Pure Manual SDK

**Description:** Continue with 100% hand-written SDK, no code generation.

**Pros:**
- Full control over API design
- Best possible developer experience
- Idiomatic Rust throughout
- High-quality documentation

**Cons:**
- Cannot scale to full API coverage
- Time-consuming to maintain
- Risk of drift from upstream APIs
- No automation when APIs change
- Requires continuous manual effort

**Rejected Because:** Does not scale. As LangChain APIs grow (hundreds of endpoints), manual implementation becomes impractical.

### Alternative 3: Hybrid Approach (Cherry-Pick)

**Description:** Generate code for some endpoints, manually implement others.

**Pros:**
- Flexibility to choose per endpoint
- Best of both worlds for selected APIs

**Cons:**
- No clear criteria for which approach to use
- Inconsistent developer experience
- Difficult to maintain consistency
- Complex decision-making for each new endpoint

**Rejected Because:** Lack of clear guidelines leads to confusion. The layered approach provides clearer separation.

### Alternative 4: Code Generation with Templates

**Description:** Use customized OpenAPI generator templates to produce idiomatic Rust.

**Pros:**
- Automated generation of idiomatic code
- Consistent style across all endpoints
- Less manual work than pure manual approach

**Cons:**
- Complex template development and maintenance
- Generator limitations and bugs
- Still lacks domain-specific abstractions
- Hard to customize for specific use cases
- Template updates required for generator upgrades

**Rejected Because:** Template customization is complex and brittle. Generated code cannot match hand-crafted domain abstractions.

## References

- [OpenAPI Generator - Rust](https://openapi-generator.tech/docs/generators/rust/)
- [Progenitor - Rust OpenAPI Client Generator](https://github.com/oxidecomputer/progenitor)
- [Oxide Computer - API Client Design](https://rfd.shared.oxide.computer/rfd/0317)
- [Issue #106: SDK Generation Strategy](https://github.com/codekiln/langstar/issues/106)
- [Issue #115: Phase 1 Research & Design](https://github.com/codekiln/langstar/issues/115)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## Implementation Notes

See [ADR-0002: OpenAPI Spec Versioning](./0002-openapi-spec-versioning.md) for how we track which OpenAPI spec version generated which SDK code.

See [ADR-0004: Drift Detection Workflow](./0004-drift-detection-workflow.md) for how we detect when upstream APIs have changed.
