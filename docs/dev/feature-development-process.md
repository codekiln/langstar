# Standard Feature Development Process

This document codifies the best practices and standard phases for implementing new LangSmith or LangGraph API features as CLI commands in Langstar. These patterns have been established through successful implementations including:

- **#298 ls-runs-query** - Runs query and filtering
- **#334 ls-annotation-queues** - Annotation queue management
- **#201 devcontainer-feature** - Infrastructure milestone (different pattern)

---

## Overview

Each API → CLI feature follows an **8-phase process**:

| Phase | Name | Goal | Deliverable |
|-------|------|------|-------------|
| 0 | Epic Setup | Establish tracking structure | Parent issue, milestone, sub-issues |
| 1 | Research | Understand SDK precedent | Research report in `reference/research/` |
| 2 | Design | Ensure DX consistency and integration | Design decisions documented in research report |
| 3 | OpenAPI Validation | Verify design against spec | Validation report + extracted schemas |
| 4 | SDK Types | Implement Rust types | `sdk/src/{feature}.rs` types |
| 5 | SDK Client | Implement client methods | Client methods in SDK |
| 6 | CLI Commands | Implement CLI commands | `cli/src/commands/{feature}.rs` |
| 7 | Testing | Ensure quality | Unit tests (mocked) + integration tests |
| 8 | Documentation | Document usage | README updates, implementation docs |

---

## Phase 0: Epic Setup

### 0.1 Create Parent Issue (Epic)

Create a milestone-level issue following the naming convention:

**Format**: `{api-name} milestone - {description}`

**Example**: `ls-runs-query milestone - Be able to list and filter runs using langstar CLI`

**Required sections**:
- Overview/TL;DR
- Goals / Success Criteria
- User Stories (epic-level and concrete)
- API Endpoints & Examples
- Design & Implementation Plan (high-level)
- Testing strategy
- Documentation plan

### 0.2 Create GitHub Milestone

Create a matching milestone linking to the parent issue:

```bash
# Via GitHub UI or API
gh api repos/:owner/:repo/milestones -f title="ls-runs-query" \
  -f description="Parent issue: #298"
```

### 0.3 Create Sub-Issues Using gh-sub-issue Skill

Use the **gh-sub-issue skill** (`.claude/skills/gh-sub-issue/SKILL.md`) to create the standard phase sub-issues:

```bash
# Create all phase sub-issues
gh sub-issue create --parent 298 --title "298.1-research Research langsmith-sdk runs query precedent"
gh sub-issue create --parent 298 --title "298.2-design Design DX consistency and configuration integration"
gh sub-issue create --parent 298 --title "298.3-openapi-validation Validate runs query design against LangSmith OpenAPI spec"
gh sub-issue create --parent 298 --title "298.4-sdk-runs-types Implement Run types and QueryRunsRequest in SDK"
gh sub-issue create --parent 298 --title "298.5-sdk-runs-client Implement query_runs client method with pagination"
gh sub-issue create --parent 298 --title "298.6-cli-runs-command Implement langstar runs query CLI command"
gh sub-issue create --parent 298 --title "298.7-runs-testing Add comprehensive tests for runs query"
gh sub-issue create --parent 298 --title "298.8-runs-docs Documentation for runs query feature"

# Verify hierarchy
gh sub-issue list 298 --relation children
```

### 0.4 Attach Milestone to ALL Issues

**CRITICAL**: Every issue (epic AND all sub-issues) MUST have the milestone attached for accurate progress tracking.

---

## Phase 1: Research SDK Precedent

### 1.1 Set Up Research Workspace

Use the **setup-remote-repo-notes-dir skill** (`.claude/skills/setup-remote-repo-notes-dir/SKILL.md`) to clone the relevant SDK:

```bash
# For LangSmith features
.claude/skills/setup-remote-repo-notes-dir/scripts/setup_repo_notes.sh https://github.com/langchain-ai/langsmith-sdk

# For LangGraph features
.claude/skills/setup-remote-repo-notes-dir/scripts/setup_repo_notes.sh https://github.com/langchain-ai/langgraph
```

**Result structure**:
```
reference/repo/langchain-ai/langsmith-sdk/
├── notes/          # Your research notes (committed)
└── code/           # Cloned SDK (gitignored)
```

### 1.2 Analyze Python SDK

Key files to examine:
- `code/python/langsmith/client.py` - Main client implementation
- `code/python/langsmith/schemas.py` - Data models
- `code/python/langsmith/_internal/` - Internal utilities
- `code/python/tests/` - Test patterns and examples

**Questions to answer**:
1. What is the method signature?
2. What parameters are supported?
3. How is pagination handled?
4. What are the request/response shapes?
5. How are errors handled?
6. What conveniences does the SDK provide?

### 1.3 Write Research Report

Create report at `reference/research/{issue-num}-{slug}-precedent.md`:

**Required sections**:
- Executive Summary
- Method Signature Analysis
- Parameter Documentation
- Request/Response Shapes
- Pagination Strategy
- Error Handling
- Recommendations for Rust Implementation

**Example**: `reference/research/298-ls-runs-query-precedent.md`

---

## Phase 2: Design

Before diving into implementation, analyze how the new feature will integrate with Langstar's existing architecture and user experience. This phase ensures consistency across the CLI and surfaces design decisions early.

### 2.1 DX Consistency Analysis

Evaluate how the feature aligns with existing Langstar commands and patterns:

**Questions to answer**:
1. Which existing commands have similar functionality? (e.g., `runs query` vs `deployments list`)
2. What flag naming conventions are already established? (e.g., `-p/--project`, `-o/--output`)
3. What output formats are supported and how should this feature use them?
4. How do similar commands handle pagination, filtering, and sorting?
5. What error messages and exit codes are used for similar error conditions?

**Review existing patterns in**:
- `cli/src/commands/` - Command structure and arguments
- `cli/src/config.rs` - Configuration loading patterns
- Existing command help text (`langstar <command> --help`)

**Document in research report**:
- Consistency decisions (which patterns to follow)
- Intentional deviations (with rationale)
- New patterns being introduced (if any)

### 2.2 Configuration Integration

Analyze how the feature integrates with Langstar's configuration system:

**Questions to answer**:
1. Which environment variables does this feature need? (existing vs new)
2. Does it need workspace/organization scoping like other features?
3. What's the precedence order? (CLI flags > env vars > config file > defaults)
4. Are there sensible defaults that match the UI behavior?

**Review configuration precedents in**:
- `cli/src/config.rs` - Existing configuration patterns
- Environment variable documentation in README
- How similar features handle missing configuration

**Configuration checklist**:
- [ ] Uses existing env vars where applicable (`LANGSMITH_API_KEY`, `LANGSMITH_WORKSPACE_ID`)
- [ ] New env vars follow naming convention (`LANGSMITH_*` or `LANGGRAPH_*`)
- [ ] Defaults match reasonable expectations
- [ ] Error messages guide users to configure missing values

### 2.3 Business Purpose Research

Understand what this feature accomplishes from a user's perspective in the LangSmith/LangGraph UI:

**Questions to answer**:
1. What workflow does this feature support in the UI?
2. What business problem does it solve for users?
3. How do users currently accomplish this task? (UI clicks, existing CLI, API calls)
4. What would be the ideal CLI experience for this workflow?

**Research methods**:
- Explore the feature in LangSmith/LangGraph UI
- Review official documentation for the feature
- Consider common user scenarios and edge cases

**Document in research report**:
- UI workflow description (what users do in the web interface)
- Key user scenarios (the "jobs to be done")
- How the CLI can improve or complement the UI workflow

### 2.4 Design Decisions Summary

Add a "Design Decisions" section to your research report:

```markdown
## Design Decisions

### DX Consistency
- Following `runs query` pattern for [reason]
- Using `-f/--filter` flag consistent with [existing command]
- Output formats: json (default for piping), table (default for terminal)

### Configuration
- Requires: LANGSMITH_API_KEY (existing), LANGSMITH_PROJECT_NAME (existing)
- New env var: [none / LANGSMITH_NEW_VAR for reason]
- Defaults: [list sensible defaults]

### Business Purpose
- Supports workflow: [describe UI workflow]
- Key scenarios: [list 2-3 primary use cases]
- CLI advantage: [why CLI is better than UI for this]
```

---

## Phase 3: OpenAPI Validation

### 3.1 OpenAPI Spec Management Pattern

Langstar uses a **canonical source + derived fragments** pattern for managing OpenAPI specs, inspired by the `setup-remote-repo-notes-dir` skill:

```
reference/
├── openapi/langchain/              # Canonical full specs (source of truth)
│   ├── langsmith/
│   │   ├── openapi.json            # Full spec (635K)
│   │   └── MANIFEST.md             # Provenance metadata
│   └── control-plane/
│       ├── openapi.json            # Full spec (70K)
│       └── MANIFEST.md
│
└── api-specs/                      # Extracted fragments + documentation
    ├── README.md                   # Index and usage guide
    ├── LANGSMITH_API_OVERVIEW.md   # Quick reference (4 APIs)
    ├── LANGSMITH_APIS_DETAILS.md   # Detailed catalog
    ├── langsmith/
    │   ├── FRAGMENTS.md            # jq extraction queries (reproducible)
    │   └── *.json                  # Extracted fragments
    └── control-plane/
        └── FRAGMENTS.md
```

**Benefits**:
- **Separation**: Canonical specs vs AI-friendly fragments
- **Reproducibility**: jq queries documented in `FRAGMENTS.md`
- **Provenance**: `MANIFEST.md` tracks when/how specs were fetched
- **AI-friendly**: Small fragments fit context windows for grounding

### 3.2 Fetch or Update OpenAPI Specification

```bash
# LangSmith API - fetch to canonical location
curl -o reference/openapi/langchain/langsmith/openapi.json \
  https://api.smith.langchain.com/openapi.json

# LangGraph Cloud API (Control Plane)
curl -o reference/openapi/langchain/control-plane/openapi.json \
  https://api.host.langchain.com/openapi.json

# Update MANIFEST.md with provenance
echo "| $(date +%Y-%m-%d) | Refresh | $(du -h reference/openapi/langchain/langsmith/openapi.json | cut -f1) | Updated from remote |" \
  >> reference/openapi/langchain/langsmith/MANIFEST.md
```

### 3.3 Extract Relevant Schemas with jq

Extract fragments to `reference/api-specs/langsmith/` and document in `FRAGMENTS.md`:

```bash
# Navigate to canonical spec
cd reference/openapi/langchain/langsmith

# Extract endpoint definition
jq '.paths["/api/v1/runs/query"]' openapi.json \
  > ../../api-specs/langsmith/runs-query-endpoint.json

# Extract request schema
jq '.components.schemas.BodyParamsForRunsQuerySchema' openapi.json \
  > ../../api-specs/langsmith/runs-query-request-schema.json

# Extract response schema
jq '.components.schemas.ListRunsResponse' openapi.json \
  > ../../api-specs/langsmith/runs-query-response-schema.json

# Extract entity schema (e.g., Run)
jq '.components.schemas.Run' openapi.json \
  > ../../api-specs/langsmith/run-schema.json
```

**IMPORTANT**: After extracting, update `reference/api-specs/langsmith/FRAGMENTS.md`:

```markdown
| File | Size | Purpose | jq Query | Last Updated |
|------|------|---------|----------|--------------|
| `runs-query-endpoint.json` | 1.0K | POST /runs/query endpoint | `.paths["/api/v1/runs/query"]` | YYYY-MM-DD |
```

### 3.4 Validate Research Against Spec

Create validation report at `reference/research/{issue-num}-openapi-validation.md`:

**Required validations**:
1. HTTP method matches
2. Path matches (with version prefix)
3. Request body schema matches research
4. Response schema matches research
5. Field types are correctly identified
6. Required vs optional fields

**Example jq queries for validation** (run from `reference/openapi/langchain/langsmith/`):
```bash
# Check endpoint method
jq '.paths["/api/v1/annotation-queues/{queue_id}/runs"].post' openapi.json

# Check request body schema
jq '.paths["/api/v1/annotation-queues/{queue_id}/runs"].post.requestBody.content["application/json"].schema' \
  openapi.json

# List all paths for a feature
jq '.paths | keys | map(select(contains("annotation-queue")))' openapi.json
```

### 3.5 Document Discrepancies

Any differences between research and OpenAPI spec MUST be documented:
- **Corrections** to research findings
- **Discoveries** not in research
- **Confirmations** of research

---

## Phase 4: SDK Types

### 4.1 Create Types Module

**File**: `sdk/src/{feature}.rs`

**Pattern from existing code** (see `sdk/src/runs.rs`, `sdk/src/deployments.rs`):

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enum types (match OpenAPI enum values exactly)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    Tool,
    Chain,
    Llm,
    // ...
}

/// Main entity struct (based on OpenAPI schema)
#[derive(Debug, Clone, Deserialize)]
pub struct Run {
    // Required fields (non-optional)
    pub id: Uuid,
    pub name: String,

    // Optional fields
    pub description: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
}

/// Request struct for queries
#[derive(Debug, Clone, Default, Serialize)]
pub struct QueryRunsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    // ...
}

/// Paginated response
#[derive(Debug, Clone, Deserialize)]
pub struct ListRunsResponse {
    pub runs: Vec<Run>,
    pub cursors: Option<Cursors>,
}
```

### 4.2 Register in lib.rs

```rust
// sdk/src/lib.rs
pub mod runs;
pub use runs::{Run, RunType, QueryRunsRequest, ListRunsResponse};
```

---

## Phase 5: SDK Client Methods

### 5.1 Add Client Methods

**Pattern**: Methods in `sdk/src/client.rs` or feature-specific modules

```rust
impl LangchainClient {
    /// Query runs with filtering and pagination
    pub async fn query_runs(&self, request: QueryRunsRequest) -> Result<ListRunsResponse, Error> {
        let url = format!("{}/api/v1/runs/query", self.langsmith_base_url);

        let response = self.http_client
            .post(&url)
            .header("X-Api-Key", &self.api_key)
            .json(&request)
            .send()
            .await?;

        // Handle response...
    }
}
```

### 5.2 Handle Pagination

Follow the cursor-based pagination pattern:
```rust
/// Stream all runs with automatic pagination
pub fn query_runs_stream(&self, request: QueryRunsRequest) -> impl Stream<Item = Result<Run, Error>> {
    // Implementation using cursors.next
}
```

---

## Phase 6: CLI Commands

### 6.1 Create Command Module

**File**: `cli/src/commands/{feature}.rs`

**Pattern from existing commands** (see `cli/src/commands/runs.rs`):

```rust
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum RunsCommand {
    /// Query runs with filtering
    Query(QueryArgs),
}

#[derive(Debug, Args)]
pub struct QueryArgs {
    /// Project name or ID to query runs from
    #[arg(short, long)]
    pub project: Option<String>,

    /// Filter expression
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Output format (json, table)
    #[arg(short = 'o', long, default_value = "table")]
    pub format: String,
}
```

### 6.2 Register in CLI

```rust
// cli/src/commands/mod.rs
pub mod runs;

// cli/src/main.rs
#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage runs
    #[command(subcommand)]
    Runs(runs::RunsCommand),
}
```

### 6.3 Follow Configuration Patterns

Use the established config pattern from `cli/src/config.rs`:
- Environment variables take precedence
- Support both `LANGSMITH_API_KEY` and `LANGGRAPH_API_KEY`
- Support organization/workspace scoping

---

## Phase 7: Testing

### 7.1 Unit Tests with Mocking

**Location**: In-module tests or `sdk/tests/`

**Use httpmock** for API mocking:
```rust
#[cfg(test)]
mod tests {
    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_query_runs_success() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/v1/runs/query");
            then.status(200)
                .json_body(json!({
                    "runs": [],
                    "cursors": null
                }));
        });

        // Test client against mock server
    }
}
```

### 7.2 Integration Tests

**Location**: `sdk/tests/{feature}_test.rs` or `cli/tests/{feature}_command_test.rs`

**Requirements**:
- Mark with `#[cfg_attr(not(feature = "integration-tests"), ignore)]`
- Use `LANGSMITH_API_KEY` and `LANGSMITH_WORKSPACE_ID` env vars
- Clean up any created resources
- Document prerequisites in test file header

**Pattern** (from `cli/tests/README.md`):
```rust
/// Integration test for runs query
///
/// Prerequisites:
/// - LANGSMITH_API_KEY set
/// - LANGSMITH_WORKSPACE_ID set
///
/// Run with: cargo test --features integration-tests
#[cfg_attr(not(feature = "integration-tests"), ignore)]
#[tokio::test]
async fn test_query_runs_integration() {
    // ...
}
```

### 7.3 Pre-Commit Validation

**ALWAYS run before committing**:
```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features && \
cargo fmt --check
```

---

## Phase 8: Documentation

### 8.1 Implementation Plan

Create `docs/implementation/{issue-num}-{slug}-implementation-plan.md`:
- Executive summary
- Research sources with links
- Implementation phases with code snippets
- Testing plan
- Future enhancements

### 8.2 Update README

Add new commands to main README:
- Command syntax
- Example usage
- Environment variables

### 8.3 In-Code Documentation

- Rustdoc comments on all public items
- Examples in doc comments where helpful
- Link to research reports for complex decisions

---

## Consistency Checklist

### Configuration Consistency

- [ ] Uses same env var names as existing features (`LANGSMITH_API_KEY`, etc.)
- [ ] Follows same precedence: env var > config file > defaults
- [ ] Supports organization/workspace scoping consistently
- [ ] Uses same output format options (`json`, `table`)

### Code Style Consistency

- [ ] Follows existing module structure
- [ ] Uses same error handling patterns
- [ ] Uses same serde patterns (rename_all, skip_serializing_if)
- [ ] Follows same test organization

### Testing Consistency

- [ ] Unit tests use httpmock
- [ ] Integration tests use feature flag
- [ ] Tests document prerequisites
- [ ] Tests clean up resources

### Documentation Consistency

- [ ] Research reports in `reference/research/`
- [ ] Canonical OpenAPI specs in `reference/openapi/langchain/{api}/`
- [ ] MANIFEST.md updated with provenance for spec fetches
- [ ] Extracted fragments in `reference/api-specs/{api}/`
- [ ] FRAGMENTS.md updated with jq queries for extractions
- [ ] Implementation plans in `docs/implementation/`
- [ ] Same markdown structure

---

## Tools & Skills Reference

| Tool/Skill | Purpose | Location |
|------------|---------|----------|
| **gh-sub-issue** | Manage issue hierarchies | `.claude/skills/gh-sub-issue/SKILL.md` |
| **setup-remote-repo-notes-dir** | Research SDK codebases | `.claude/skills/setup-remote-repo-notes-dir/SKILL.md` |
| **jq** | OpenAPI spec validation | System tool |
| **httpmock** | Rust HTTP mocking | Cargo dependency |
| **langgraph-docs MCP** | LangGraph/LangSmith docs | MCP server |

---

## Example Sub-Issue Checklist Template

When creating sub-issues, use this template for scope:

### Research Sub-Issue
```markdown
## Scope
1. Set up research workspace with setup-remote-repo-notes-dir
2. Analyze Python SDK implementation
3. Document method signatures and parameters
4. Identify pagination patterns
5. Review tests for usage examples

## Deliverable
Research report at `reference/research/{num}-{slug}-precedent.md`
```

### OpenAPI Validation Sub-Issue
```markdown
## Scope
1. Fetch OpenAPI spec to reference/api-specs/
2. Extract relevant endpoint and schema definitions with jq
3. Validate research findings against spec
4. Document confirmations, corrections, discoveries

## Deliverable
Validation report at `reference/research/{num}-openapi-validation.md`
```

---

## Related Documentation

- [GitHub Workflow](./github-workflow.md) - Issue-driven development
- [Git SCM Conventions](./git-scm-conventions.md) - Commit message format
- [Code Style Principles](./code-style-principles.md) - Explicit over implicit
- [Procedures](./procedures.md) - Pre-commit checklist

---

## Success Metrics

A feature is complete when:
1. All sub-issues closed
2. Research and validation reports committed
3. SDK types and methods implemented
4. CLI commands functional
5. Unit tests passing (100% of new code)
6. Integration tests passing
7. Documentation updated
8. Milestone closed
