# Generated OpenAPI Code (Deprecated)

This directory is a placeholder from an earlier architectural approach that explored OpenAPI code generation.

## Current Status

**The SDK is LLM-assisted** in the parent `sdk/src/` directory. OpenAPI code generation has been abandoned in favor of LLM-assisted Rust implementations.

## Why LLM-Assisted Implementation?

The project switched to LLM-assisted implementation for:

- **Better ergonomics** - APIs designed specifically for Rust idioms
- **Flexibility** - Easier to adapt to API changes and quirks
- **Maintainability** - Clearer code that's easier to understand and modify
- **Control** - Full control over error handling, types, and API design

## Where is the SDK?

The Rust SDK modules are in `sdk/src/`:

- `client.rs` - HTTP client with authentication
- `prompts.rs` - LangSmith Prompts API
- `assistants.rs` - LangGraph Assistants API
- `deployments.rs` - LangGraph Deployments API
- `runs.rs` - LangSmith Runs/Traces API
- `datasets.rs` - LangSmith Datasets API
- `evaluations.rs` - LangSmith Evaluations API
- And other service-specific modules

## Historical Context

This directory remains as a historical marker. The decision to abandon code generation is documented in:

- [#114](https://github.com/codekiln/langstar/issues/114) - Architecture planning (closed, wontfix)
- [#115](https://github.com/codekiln/langstar/issues/115) - Research & Design (closed)
- [#116](https://github.com/codekiln/langstar/issues/116) - Implementation (closed)
- [#117](https://github.com/codekiln/langstar/issues/117) - Automation (closed)
