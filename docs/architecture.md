# Architecture Documentation

This document explains Langstar's architecture, design decisions, and implementation details.

## Table of Contents

- [Overview](#overview)
- [Design Principles](#design-principles)
- [Multi-Service SDK](#multi-service-sdk)
- [Resource Scoping Models](#resource-scoping-models)
- [HTTP Client Implementation](#http-client-implementation)
- [Error Handling](#error-handling)
- [CLI Design](#cli-design)
- [Configuration System](#configuration-system)
- [Future Considerations](#future-considerations)

## Overview

Langstar is a unified Rust CLI for the LangChain ecosystem, providing ergonomic access to both LangSmith and LangGraph Cloud services. The architecture emphasizes:

- **Service separation**: Clear boundaries between LangSmith and LangGraph
- **Spec-driven development**: Code generation from OpenAPI specifications
- **Thin wrapper pattern**: Minimal abstraction over upstream APIs
- **Type safety**: Leveraging Rust's type system for correctness

### System Architecture

```
┌─────────────────────────────────────┐
│      Langstar CLI (User-Facing)    │
├─────────────────────────────────────┤
│  Commands │ Config │ Output │ Main │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│        Langstar SDK (Library)       │
├──────────────────┬──────────────────┤
│  LangSmith API   │  LangGraph API   │
│  (org/workspace) │  (deployment)    │
└────────┬─────────┴──────┬───────────┘
         │                │
         ▼                ▼
┌─────────────────┐  ┌──────────────────┐
│  LangSmith      │  │  LangGraph       │
│  REST API       │  │  REST API        │
└─────────────────┘  └──────────────────┘
```

## Design Principles

### 1. Spec-Driven Development

**Principle:** Generate code from OpenAPI specs, don't hand-write API wrappers.

**Implementation:**
- OpenAPI specs → Code generation → Rust types
- Guarantees API coverage and correctness
- Reduces maintenance burden

**Benefits:**
- API changes detected at generation time
- Type-safe API clients
- Consistent error handling

**Example:**

```rust
// Hand-written types (v0.2.0)
// Note: OpenAPI spec generation is planned for future versions (see #114)
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub owner: String,
    // ... other fields matching the API
}
```

**Note:** While the design principle is spec-driven development, v0.2.0 uses hand-written types.
Full OpenAPI code generation is tracked in [#114](https://github.com/codekiln/langstar/issues/114).

### 2. Thin Wrapper Pattern

**Principle:** Add only lightweight ergonomic helpers, no business logic duplication.

**What we add:**
- Authentication helpers
- Configuration management
- Error conversion
- Pagination helpers

**What we don't add:**
- Business logic (delegate to upstream)
- Caching (keep stateless)
- Complex transformations (preserve API contracts)

**Example:**

```rust
// Thin wrapper: adds auth, delegates to generated client
impl LangstarClient {
    pub fn list_prompts(&self, params: ListPromptsParams) -> Result<Vec<Prompt>> {
        let headers = self.build_langsmith_headers()?;  // Our code
        self.generated_client.list_prompts(params, headers)  // Generated
    }
}
```

### 3. Automation-First

**Principle:** Design for both human and AI agent usage.

**Implementation:**
- JSON output for scripting
- Table output for humans
- Exit codes for CI/CD
- Stable command interface

**Example:**

```bash
# Human-friendly
langstar prompt list

# Machine-friendly
langstar prompt list --format json | jq '.[] | .name'

# CI/CD-friendly
if langstar assistant list --format json | jq -e '.[] | select(.name == "prod-bot")'; then
  echo "Prod bot exists"
fi
```

### 4. Zero Surprises

**Principle:** Type-safe, predictable behavior with clear error messages.

**Implementation:**
- Strong types (no stringly-typed APIs)
- Clear error messages with context
- Consistent command patterns
- Explicit configuration (no magic defaults)

**Example:**

```rust
// Type-safe configuration
pub struct Config {
    pub langsmith_api_key: Option<String>,
    pub organization_id: Option<String>,
    // ...
}

// Clear error messages
return Err(Error::MissingApiKey {
    service: "LangSmith",
    command: "prompt list",
    hint: "Set LANGSMITH_API_KEY or add to config file",
});
```

### 5. Service Separation

**Principle:** Clean boundaries between LangSmith and LangGraph APIs.

**Why:** These services have fundamentally different resource scoping models.

**Implementation:**
- Separate client methods (`langsmith_*` vs `langgraph_*`)
- Service-specific configuration sections
- Different header handling
- Clear documentation boundaries

## Multi-Service SDK

### Service Abstraction

The SDK provides unified access to multiple services while maintaining their distinct characteristics.

```rust
pub struct LangstarClient {
    http_client: reqwest::Client,
    langsmith_config: LangSmithConfig,
    langgraph_config: LangGraphConfig,
}

impl LangstarClient {
    // LangSmith methods (with scoping)
    pub fn langsmith_request(&self, ...) -> Result<Response> {
        let headers = self.build_langsmith_headers()?;
        // Adds: x-api-key, x-organization-id, X-Tenant-Id
        self.http_client.request(...)
    }

    // LangGraph methods (no scoping)
    pub fn langgraph_request(&self, ...) -> Result<Response> {
        let headers = self.build_langgraph_headers()?;
        // Adds: x-api-key only
        self.http_client.request(...)
    }
}
```

### Header Management

Different services require different HTTP headers:

**LangSmith:**
```http
x-api-key: <LANGSMITH_API_KEY>
x-organization-id: <org-id>  # Optional, for scoping
X-Tenant-Id: <workspace-id>   # Optional, for narrower scoping
```

**LangGraph:**
```http
x-api-key: <LANGSMITH_API_KEY>
# No additional scoping headers
```

## Resource Scoping Models

### Why Different Scoping?

LangSmith and LangGraph have different architectural requirements:

| Service | Resource Model | Multi-tenancy | Typical Users |
|---------|----------------|---------------|---------------|
| **LangSmith** | Hierarchical (org→workspace→prompts) | Yes | Enterprise teams with multiple projects |
| **LangGraph** | Flat (deployment→assistants) | No | Deployment-specific applications |

### LangSmith: Hierarchical Model

```
Organization (x-organization-id)
├── Workspace A (X-Tenant-Id)
│   ├── Prompt 1
│   ├── Prompt 2
│   └── Prompt 3
├── Workspace B
│   ├── Prompt 4
│   └── Prompt 5
└── Workspace C
    └── Prompt 6
```

**API Request Flow:**

```
User → Langstar CLI
       ↓ (adds headers)
HTTP Request:
  x-api-key: <key>
  x-organization-id: <org-id>     # Optional
  X-Tenant-Id: <workspace-id>     # Optional
       ↓
LangSmith API → Filtered results based on scoping
```

**SDK Implementation:**

```rust
fn build_langsmith_headers(&self) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", self.langsmith_config.api_key.parse()?);

    // Add scoping headers if configured
    if let Some(org_id) = &self.langsmith_config.organization_id {
        headers.insert("x-organization-id", org_id.parse()?);
    }
    if let Some(workspace_id) = &self.langsmith_config.workspace_id {
        headers.insert("X-Tenant-Id", workspace_id.parse()?);
    }

    Ok(headers)
}
```

### LangGraph: Deployment Model

```
API Key → Deployment
          ├── Assistant 1
          ├── Assistant 2
          └── Assistant 3
```

**API Request Flow:**

```
User → Langstar CLI
       ↓ (adds API key only)
HTTP Request:
  x-api-key: <key>    # Tied to specific deployment
       ↓
LangGraph API → Returns assistants for this deployment
```

**SDK Implementation:**

```rust
fn build_langgraph_headers(&self) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    // Use LANGSMITH_API_KEY or fall back to LANGSMITH_API_KEY
    let api_key = self.langgraph_config.api_key
        .as_ref()
        .or(self.langsmith_config.api_key.as_ref())
        .ok_or(Error::MissingApiKey)?;

    headers.insert("x-api-key", api_key.parse()?);

    // No additional scoping headers
    Ok(headers)
}
```

### Design Implication: No Scoping for Assistants

**Key Insight:** LangGraph assistants are deployment-level resources.

This is NOT a limitation or missing feature—it's the intended design:

1. **API Key = Deployments**: Your API key determines which deployments you can access
2. **Simpler Model**: No need for organization/workspace hierarchy
3. **Clear Boundaries**: Each deployment is independent
4. **Access Control**: Managed at API key/deployment level

**Why this matters for Langstar:**

```bash
# ✅ Correct: No scoping flags
langstar assistant list

# ❌ Wrong: Trying to scope (these flags don't exist)
# langstar assistant list --organization-id <id>  # No such flag!
```

## HTTP Client Implementation

### Client Architecture

```rust
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            inner,
            base_url: base_url.into(),
        })
    }

    pub async fn request<T>(
        &self,
        method: Method,
        path: &str,
        headers: HeaderMap,
        body: Option<T>,
    ) -> Result<Response>
    where
        T: Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.inner.request(method, url);

        // Add headers
        req = req.headers(headers);

        // Add body if present
        if let Some(body) = body {
            req = req.json(&body);
        }

        let resp = req.send().await?;
        self.handle_response(resp).await
    }
}
```

### Error Handling Strategy

**Principle:** Convert HTTP errors to domain errors with context.

```rust
pub enum Error {
    // Authentication errors
    MissingApiKey { service: String, command: String, hint: String },
    InvalidApiKey { service: String },

    // HTTP errors
    HttpError { status: StatusCode, message: String },

    // Scoping errors
    InvalidScoping { issue: String, suggestion: String },

    // Network errors
    NetworkError { source: reqwest::Error },
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::NetworkError { source: err }
        } else if let Some(status) = err.status() {
            Error::HttpError {
                status,
                message: format!("HTTP request failed: {}", err),
            }
        } else {
            Error::NetworkError { source: err }
        }
    }
}
```

### Response Handling

```rust
async fn handle_response(&self, resp: Response) -> Result<Response> {
    let status = resp.status();

    if status.is_success() {
        return Ok(resp);
    }

    // Convert HTTP errors to domain errors
    match status {
        StatusCode::UNAUTHORIZED => Err(Error::InvalidApiKey {
            service: "unknown",
        }),
        StatusCode::FORBIDDEN => Err(Error::HttpError {
            status,
            message: "Access forbidden - check API key permissions".to_string(),
        }),
        StatusCode::NOT_FOUND => Err(Error::HttpError {
            status,
            message: "Resource not found - check scoping or resource ID".to_string(),
        }),
        StatusCode::TOO_MANY_REQUESTS => Err(Error::HttpError {
            status,
            message: "Rate limit exceeded - wait before retrying".to_string(),
        }),
        _ => Err(Error::HttpError {
            status,
            message: format!("Unexpected error: {}", status),
        }),
    }
}
```

## CLI Design

### Command Structure

```
langstar
├── config              # Show configuration
├── prompt              # LangSmith prompts (org/workspace scoped)
│   ├── list            # List prompts
│   ├── get             # Get prompt details
│   └── search          # Search prompts
└── assistant           # LangGraph assistants (deployment-level)
    ├── list            # List assistants
    ├── get             # Get assistant details
    ├── search          # Search assistants
    ├── create          # Create assistant
    ├── update          # Update assistant
    └── delete          # Delete assistant
```

### Command Implementation Pattern

```rust
#[derive(Parser)]
#[command(name = "langstar")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current configuration
    Config,

    /// Manage LangSmith prompts (supports org/workspace scoping)
    #[command(subcommand)]
    Prompt(PromptCommands),

    /// Manage LangGraph assistants (deployment-level resources)
    #[command(subcommand)]
    Assistant(AssistantCommands),
}
```

### Flag Differences by Service

**Prompt commands (scoping supported):**

```rust
#[derive(Parser)]
struct PromptListArgs {
    /// Organization ID (optional)
    #[arg(long)]
    organization_id: Option<String>,

    /// Workspace ID (optional)
    #[arg(long)]
    workspace_id: Option<String>,

    /// Show public prompts (when scoped)
    #[arg(long)]
    public: bool,

    /// Output format
    #[arg(long, default_value = "table")]
    format: OutputFormat,
}
```

**Assistant commands (no scoping):**

```rust
#[derive(Parser)]
struct AssistantListArgs {
    /// Limit number of results
    #[arg(long)]
    limit: Option<usize>,

    /// Offset for pagination
    #[arg(long)]
    offset: Option<usize>,

    /// Output format
    #[arg(long, default_value = "table")]
    format: OutputFormat,

    // Note: No organization_id, workspace_id flags!
}
```

## Configuration System

### Configuration Precedence

1. **Command-line flags** (highest)
2. **Configuration file**
3. **Environment variables** (lowest)

### Implementation

```rust
pub struct Config {
    // LangSmith configuration
    pub langsmith_api_key: Option<String>,
    pub organization_id: Option<String>,
    pub workspace_id: Option<String>,

    // LangGraph configuration
    pub langsmith_api_key: Option<String>,

    // General settings
    pub output_format: OutputFormat,
}

impl Config {
    pub fn load() -> Result<Self> {
        // 1. Start with defaults
        let mut config = Config::default();

        // 2. Load from config file (if exists)
        if let Some(file_config) = Self::load_from_file()? {
            config.merge(file_config);
        }

        // 3. Overlay environment variables
        if let Ok(key) = env::var("LANGSMITH_API_KEY") {
            config.langsmith_api_key = Some(key);
        }
        // ... other env vars

        // 4. CLI flags applied in command handlers (highest priority)

        Ok(config)
    }
}
```

## Future Considerations

### If LangGraph Adds Organization Scoping

**Current:** LangGraph uses deployment-level resources (v0.2.0).

**If changed:** The following would need updates:

1. **SDK Changes:**
   ```rust
   // Would need to add scoping headers
   fn build_langgraph_headers(&self) -> Result<HeaderMap> {
       let mut headers = HeaderMap::new();
       headers.insert("x-api-key", ...);

       // NEW: Add scoping if LangGraph supports it
       if let Some(org_id) = &self.langgraph_config.organization_id {
           headers.insert("x-organization-id", org_id.parse()?);
       }

       Ok(headers)
   }
   ```

2. **CLI Changes:**
   ```rust
   // Would need to add scoping flags to assistant commands
   #[derive(Parser)]
   struct AssistantListArgs {
       #[arg(long)]
       organization_id: Option<String>,  // NEW

       #[arg(long)]
       workspace_id: Option<String>,  // NEW

       // ... existing flags
   }
   ```

3. **Documentation Changes:**
   - Update README scoping sections
   - Update configuration guide
   - Update architecture documentation
   - Update examples

**Important:** This should be treated as a feature addition, not a bug fix, as the current implementation correctly reflects LangGraph's actual API design.

### Extensibility

The architecture supports adding new services:

```rust
impl LangstarClient {
    // Pattern for new services
    pub fn new_service_request(&self, ...) -> Result<Response> {
        let headers = self.build_new_service_headers()?;
        self.http_client.request(...)
    }

    fn build_new_service_headers(&self) -> Result<HeaderMap> {
        // Service-specific header logic
    }
}
```

## Summary

### Key Architectural Decisions

1. **Multi-service SDK**: Separate but unified access to LangSmith and LangGraph
2. **Service-specific scoping**: LangSmith hierarchical, LangGraph flat
3. **Thin wrapper**: Minimal abstraction over upstream APIs
4. **Type safety**: Strong types throughout
5. **Clear boundaries**: Explicit service separation in code and documentation

### Design Trade-offs

| Decision | Benefit | Trade-off |
|----------|---------|-----------|
| Service separation | Clear, predictable behavior | Slight code duplication |
| Thin wrapper | Easy maintenance, stays in sync | Some verbose APIs |
| Type safety | Catch errors at compile time | More boilerplate |
| No caching | Simple, stateless | More network requests |
| Spec-driven | Guaranteed API coverage | Code generation complexity |

## Additional Resources

- [Configuration Guide](./configuration.md) - Configuration system details
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions
- [Examples](./examples/) - Real-world usage patterns
- [README](../README.md) - Quick start guide
