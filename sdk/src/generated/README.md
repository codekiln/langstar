# Generated OpenAPI Code

This directory will contain OpenAPI-generated Rust client code for LangChain services.

## Generation

Run the following command to generate the code:

```bash
./tools/generate_sdk.sh
```

This will:
1. Fetch the OpenAPI specifications from LangSmith and LangGraph APIs
2. Generate Rust client code using an appropriate OpenAPI generator
3. Place the generated code in this directory

## Note

For the initial prototype, we are implementing manual client wrappers in the parent
modules. The OpenAPI generation will be integrated in a future iteration to ensure
100% API coverage and automatic updates when the APIs change.
