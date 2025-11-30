# Langstar Tooling

This directory contains development utilities for the Langstar project.

## Historical Note: OpenAPI SDK Generation

The project previously explored automated OpenAPI code generation but has since adopted a manual SDK implementation approach. The decision to abandon automated generation is documented in these issues:

- [#114](https://github.com/codekiln/langstar/issues/114) - Architecture planning (closed, wontfix)
- [#115](https://github.com/codekiln/langstar/issues/115) - Research & Design (closed)
- [#116](https://github.com/codekiln/langstar/issues/116) - Implementation with Version Tracking (closed)
- [#117](https://github.com/codekiln/langstar/issues/117) - Automate API Drift Detection (closed)

### Why Manual Implementation?

The manual approach was chosen for:
- **Better ergonomics** - Hand-crafted APIs tailored to Rust idioms
- **Flexibility** - Easier to handle API quirks and evolution
- **Maintainability** - Simpler to understand and modify
- **Control** - Full control over error handling and type design

### Current SDK Structure

The Rust SDK is manually implemented in `sdk/src/`:
- `client.rs` - HTTP client wrapper with authentication
- `prompts.rs` - LangSmith Prompts API
- `assistants.rs` - LangGraph Assistants API
- `deployments.rs` - LangGraph Deployments API
- `runs.rs` - LangSmith Runs/Traces API
- `datasets.rs` - LangSmith Datasets API
- `evaluations.rs` - LangSmith Evaluations API
- And other service-specific modules

The OpenAPI specifications are still referenced for accuracy but not used for code generation.

### Legacy Files

The following files remain for historical reference but are not actively used:
- `generate_sdk.sh` - Original generation script
- `sdk/src/generated/` - Placeholder directory

## Other Tools

Additional development utilities may be added here as the project grows.
