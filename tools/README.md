# Langstar Tooling

This directory contains development tools for the Langstar project.

## OpenAPI SDK Generation

### Overview

The `generate_sdk.sh` script automates the process of generating Rust client code from OpenAPI specifications provided by LangChain services (LangSmith, LangGraph Cloud).

### Usage

```bash
./tools/generate_sdk.sh
```

### What It Does

1. **Fetches OpenAPI Specifications**
   - LangSmith API: `https://api.smith.langchain.com/openapi.json`
   - LangGraph Cloud API: `https://api.langgraph.cloud/openapi.json`
   - Saves specs to `tools/specs/` directory

2. **Generates Rust Client Code**
   - Uses OpenAPI Generator to create Rust client libraries
   - Outputs to `sdk/src/generated/` directory
   - Creates separate clients for LangSmith and LangGraph

3. **Integration**
   - Generated code is wrapped by ergonomic APIs in `sdk/src/`
   - Authentication, error handling, and convenience methods are added

### Prerequisites

You need one of the following OpenAPI generators:

#### Option 1: OpenAPI Generator CLI (Node.js)

```bash
npm install -g @openapitools/openapi-generator-cli
```

#### Option 2: Docker

The script can use Docker to run the OpenAPI generator without local installation:

```bash
# Install Docker: https://docs.docker.com/get-docker/
docker pull openapitools/openapi-generator-cli
```

#### Option 3: Alternative Rust Generators (Future)

We're evaluating Rust-native OpenAPI generators:
- [progenitor](https://github.com/oxidecomputer/progenitor)
- [openapi-generator-rust](https://github.com/OpenAPITools/openapi-generator/tree/master/modules/openapi-generator/src/main/resources/rust)

### Current Implementation Status

**Prototype Phase**: The initial prototype uses manual client implementations in:
- `sdk/src/client.rs` - HTTP client wrapper
- `sdk/src/prompts.rs` - LangSmith Prompts API

**Future**: Full OpenAPI code generation will be integrated to ensure:
- 100% API coverage
- Automatic updates when APIs change
- Type-safe client code

### Generated Code Structure

After running the script:

```
sdk/src/generated/
├── langsmith/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models/
│   │   └── apis/
│   └── README.md
├── langgraph/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models/
│   │   └── apis/
│   └── README.md
└── README.md
```

### Integration with Workspace

To use the generated code:

1. **Update `sdk/Cargo.toml`** to include generated crates as dependencies
2. **Update `sdk/src/lib.rs`** to re-export generated types and APIs
3. **Wrap with ergonomic APIs** in `sdk/src/` modules

### Automation

Future CI/CD integration:
- Automatically regenerate SDK on API spec changes
- Validate generated code compiles
- Run integration tests against live APIs

See `.github/workflows/codegen.yml` (to be implemented)

## Other Tools

Additional tools will be added here as the project grows:
- Testing utilities
- Deployment scripts
- Documentation generators
