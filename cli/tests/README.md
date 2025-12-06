# CLI Integration Tests

> **📍 Documentation Centralized**
>
> See `docs/dev/testing/cli-integration-tests.md` for complete documentation.

## Quick Reference

**Run integration tests:**
```bash
cargo test --features integration-tests --test assistant_command_test --test graph_command_test -- --nocapture
```

**Prerequisites:**
- `LANGSMITH_API_KEY` - Your LangSmith API key
- `LANGSMITH_WORKSPACE_ID` - Your workspace ID

**Full documentation:** `docs/dev/testing/cli-integration-tests.md`
