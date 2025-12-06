# SDK Integration Tests

> **📍 Documentation Centralized**
>
> Testing documentation has been centralized to prevent context pollution.
> See `@docs/dev/testing/sdk-integration-tests.md` for complete documentation.

## Quick Reference

**Run integration tests:**
```bash
cargo test --test integration_test -- --ignored --nocapture
```

**Prerequisites:**
- `LANGSMITH_API_KEY` - Your LangSmith API key
- `LANGSMITH_WORKSPACE_ID` - Your workspace ID (for deployment tests)

**Full documentation:** `docs/dev/testing/sdk-integration-tests.md`
