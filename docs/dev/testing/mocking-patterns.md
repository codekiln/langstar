<!--
  SIZE LIMIT: This file SHOULD remain under 150 lines (currently ~180).
  Last checked: 2025-12-06
  If significantly exceeding limit, extract content to sub-document.
-->

# Mocking Patterns

This document provides guidance on when and how to mock external dependencies in tests.

## When to Mock vs. Integration Test

### Use Integration Tests (Real API) When:

- ✅ Testing end-to-end feature correctness
- ✅ Verifying API contract compliance
- ✅ Testing CRUD operations (see `crud-lifecycle-pattern.md`)
- ✅ Validating authentication flows
- ✅ Confirming data persists correctly
- ✅ Testing the feature for the first time

**Langstar preference:** Use integration tests for features that interact with LangSmith/LangGraph APIs. This catches API contract changes and ensures real behavior works.

### Use Mocking When:

- ✅ Testing error handling paths (network failures, 500 errors, rate limits)
- ✅ Testing edge cases hard to reproduce with real APIs
- ✅ Testing pagination logic with controlled data volumes
- ✅ Testing retry logic and backoff behavior
- ✅ Speed-sensitive unit tests that don't need API verification
- ✅ Offline development when API access is unavailable

### Current State

Langstar currently uses integration tests with real API calls for most test coverage. This is intentional—it provides higher confidence that features work in production.

Unit tests exist for serialization/deserialization logic (see `sdk/src/prompts.rs` tests module) but don't mock HTTP calls.

## Mock Server Pattern (httpmock)

If you need to add mock-based tests, use the `httpmock` crate:

```rust
// Cargo.toml
[dev-dependencies]
httpmock = "0.7"
```

### Basic Example

```rust
use httpmock::prelude::*;

#[test]
fn test_list_prompts_handles_empty_response() {
    // Start mock server
    let server = MockServer::start();

    // Configure mock response
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/repos/");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"repos": []}"#);
    });

    // Create client pointing to mock server
    let client = create_test_client(&server.base_url());

    // Execute and verify
    let result = client.prompts().list(None, None, None).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());

    // Verify mock was called
    mock.assert();
}
```

### Error Handling Example

```rust
#[test]
fn test_handles_rate_limit_error() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/repos/");
        then.status(429)
            .header("retry-after", "60")
            .body(r#"{"error": "Rate limited"}"#);
    });

    let client = create_test_client(&server.base_url());
    let result = client.prompts().list(None, None, None).await;

    assert!(result.is_err());
    // Verify error type is rate limit
    assert!(matches!(result.unwrap_err(), LangstarError::RateLimited(_)));
}
```

### Pagination Example

```rust
#[test]
fn test_pagination_with_multiple_pages() {
    let server = MockServer::start();

    // First page
    server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/repos/")
            .query_param("offset", "0");
        then.status(200)
            .body(r#"{"repos": [{"id": "1"}], "total": 3}"#);
    });

    // Second page
    server.mock(|when, then| {
        when.method(GET)
            .path("/api/v1/repos/")
            .query_param("offset", "1");
        then.status(200)
            .body(r#"{"repos": [{"id": "2"}], "total": 3}"#);
    });

    // Test pagination logic
    let client = create_test_client(&server.base_url());
    let all_results = client.prompts().list_all().await.unwrap();
    assert_eq!(all_results.len(), 2);
}
```

## Test Isolation

When using mocks:

1. **Start fresh server per test** - Use `MockServer::start()` in each test
2. **Don't share mock state** - Each test should be independent
3. **Assert mock calls** - Use `mock.assert()` to verify expected calls happened
4. **Clear expectations** - Mock server is dropped when test ends

## Mixing Mocks and Integration Tests

It's valid to have both:

- **Integration tests** in `cli/tests/*_command_test.rs` - Real API, full CRUD lifecycle
- **Unit tests with mocks** in `sdk/src/*.rs` `#[cfg(test)]` - Error paths, edge cases

Example structure:

```
sdk/
├── src/
│   └── prompts.rs           # Unit tests with mocks (error handling)
└── tests/
    └── prompts_test.rs      # Integration tests (real API)

cli/
└── tests/
    └── prompt_command_test.rs  # Integration tests (real API + CLI)
```

## When NOT to Mock

- ❌ Don't mock to make tests faster if it reduces confidence
- ❌ Don't mock when you're unsure about API contract
- ❌ Don't mock CRUD lifecycle tests (use `crud-lifecycle-pattern.md`)
- ❌ Don't mock first test of a new feature

**Rule of thumb:** If mocking would hide a real bug, don't mock.

## Related Documentation

- **CRUD Lifecycle Pattern:** `crud-lifecycle-pattern.md` - Always use real API
- **High-Level Guidelines:** `HIGH_LEVEL_TESTING_GUIDELINES.md` - Test design principles
- **SDK Integration Tests:** `sdk-integration-tests.md` - Real API test patterns
