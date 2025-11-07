# LangChain HTTP/REST APIs Overview

This document provides an overview of the various HTTP/REST APIs in the LangChain ecosystem, their purposes, and links to their OpenAPI specifications.

## API Summary Table

| API Name | Base URL (US) | Base URL (EU) | OpenAPI Spec | Purpose |
|----------|---------------|---------------|--------------|---------|
| LangSmith API | `https://api.smith.langchain.com` | `https://eu.api.smith.langchain.com` | [/openapi.json](https://api.smith.langchain.com/openapi.json) | Tracing, evaluation, datasets, org management |
| Control Plane API | `https://api.host.langchain.com` | `https://eu.api.host.langchain.com` | [/openapi.json](https://api.host.langchain.com/openapi.json) | Deployment management for LangGraph Server |
| LangGraph Server API | Per-deployment | Per-deployment | `/docs` on each deployment | Runtime API for assistants, threads, runs |
| SCIM API | `https://api.smith.langchain.com/scim/v2` | `https://eu.api.smith.langchain.com/scim/v2` | SCIM 2.0 compliant | User provisioning (Enterprise) |

---

## 1. LangSmith API

**Base URLs:**
- US: `https://api.smith.langchain.com`
- EU: `https://eu.api.smith.langchain.com`
- Self-hosted: `http(s)://<langsmith-url>/api/v1`

**OpenAPI Specification:**
- JSON: `https://api.smith.langchain.com/openapi.json`
- Interactive docs: `https://api.smith.langchain.com/redoc`

**Authentication:**
- Header: `X-Api-Key` with LangSmith API key

**Purpose:**
The LangSmith API is the primary API for LangSmith platform operations. It provides comprehensive access to tracing, evaluation, and organizational management features.

**Main Capabilities:**

### Tracing & Observability
- **Tracer Sessions** (Projects): Create, read, update, delete tracing sessions
- **Runs**: Log and query execution traces
- **Feedback**: Attach evaluations and annotations to runs
- Prebuilt dashboards, metadata, filter views
- Insights/clustering for pattern analysis

### Datasets & Evaluation
- **Datasets**: Create and manage test datasets
- **Examples**: CRUD operations on test cases
- Upload via CSV or programmatic API
- Version control and sharing
- Download in multiple formats (CSV, JSONL, OpenAI format)

### Organization Management
- **Organizations**: Org settings, billing, usage tracking
- **Workspaces**: Multi-tenant workspace management
- **Members & Roles**: User management and RBAC
- **SSO Configuration**: SAML setup and management
- **API Keys**: Service key generation and management

### Additional Features
- Prompt management and versioning
- Webhooks for event notifications
- Data export (bulk exports to S3, GCS, etc.)
- Annotation queues
- Comparative experiments

**Use Cases:**
- Logging LLM application traces
- Running evaluations on datasets
- Managing users and permissions
- Analyzing application performance
- CI/CD integration for testing

---

## 2. Control Plane API

**Base URLs:**
- US: `https://api.host.langchain.com`
- EU: `https://eu.api.host.langchain.com`
- Self-hosted: `http(s)://<host>/api-host`

**OpenAPI Specification:**
- JSON: `https://api.host.langchain.com/openapi.json`
- Interactive docs: `https://api.host.langchain.com/docs`

**Authentication:**
- Headers: `X-Api-Key` (LangSmith API key) and `X-Tenant-Id` (workspace ID)

**Purpose:**
The Control Plane API manages LangGraph Server deployments. It provides programmatic access to create, update, and monitor deployments as part of CI/CD workflows.

**Main Capabilities:**

### Deployments (`/v2/deployments`)
- **Create**: Initialize new deployments from GitHub repos or Docker images
- **List**: Query deployments with filtering (name, status, type, tags)
- **Get**: Retrieve deployment details
- **Update (PATCH)**: Modify configuration, trigger new revisions
- **Delete**: Remove deployments

**Deployment Types:**
- `dev` / `dev_free`: Development environments (preemptible infrastructure)
- `prod`: Production environments (HA, autoscaling, backups)

### Revisions (`/v2/deployments/{id}/revisions`)
- **List**: View deployment history
- **Get**: Check revision status and details
- **Redeploy**: Rollback to previous revision

**Revision Statuses:**
- `CREATING`, `QUEUED`, `AWAITING_BUILD`, `BUILDING`
- `AWAITING_DEPLOY`, `DEPLOYING`, `DEPLOYED`
- `BUILD_FAILED`, `DEPLOY_FAILED`, `CREATE_FAILED`
- `SKIPPED`, `INTERRUPTED`, `UNKNOWN`

### Integrations
- GitHub integration metadata
- Docker registry configuration

### Monitoring
- Deployment health status
- Metrics collection endpoints
- Build and server logs

**Use Cases:**
- Automated CI/CD deployment pipelines
- Infrastructure-as-code deployment management
- Blue-green or canary deployments
- Deployment monitoring and alerting
- Multi-environment management (dev, staging, prod)

**Example Workflow:**
1. `POST /v2/deployments` - Create deployment
2. `GET /v2/deployments/{id}/revisions/{revision_id}` - Poll until status is `DEPLOYED`
3. `PATCH /v2/deployments/{id}` - Update deployment (creates new revision)
4. `DELETE /v2/deployments/{id}` - Clean up

---

## 3. LangGraph Server API

**Base URL:**
- Per-deployment: `http(s)://<deployment-url>`
- Local dev: `http://localhost:8124`

**OpenAPI Specification:**
- Available at `/docs` on each deployment
- Reference docs: [LangGraph Platform API Reference](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref.html)

**Authentication:**
- Header: `X-Api-Key` (LangSmith API key for the organization)

**Purpose:**
The LangGraph Server API is the runtime API for deployed LangGraph applications. It manages assistants (configured agents), threads (conversation state), and runs (executions).

**Main Capabilities:**

### Assistants
- **Create/List/Get/Update/Delete**: Manage assistant configurations
- **Search**: Find assistants by metadata
- Assistants are graphs with specific configuration settings
- Multiple assistants per graph with different configs

### Threads
- **Create/List/Get/Update/Delete**: Manage conversation threads
- Thread state management and persistence
- Thread history and checkpoints

### Runs
- **Create**: Execute assistant on a thread
- **Stream**: Real-time execution with streaming responses
- **Wait**: Synchronous execution
- **List/Get**: Query run history
- Background and interactive execution modes

### Cron Jobs
- Schedule recurring assistant executions
- Time-based automation

### Webhooks
- Event-driven notifications
- Integration with external systems

### Store Operations
- Key-value storage within threads
- Persistent state management

**Use Cases:**
- Deploying agentic applications
- Managing conversational AI state
- Background task processing
- Real-time agent interactions
- Scheduled automation
- Integration with external systems via webhooks

**Architecture:**
- Built on PostgreSQL (persistence/checkpoints)
- Redis task queue for background runs
- Supports both streaming and batch execution

---

## 4. SCIM API (Enterprise)

**Base URLs:**
- US: `https://api.smith.langchain.com/scim/v2`
- EU: `https://eu.api.smith.langchain.com/scim/v2`
- Self-hosted: `http(s)://<langsmith-url>/scim/v2`

**OpenAPI Specification:**
- SCIM 2.0 compliant (RFC 7644)

**Authentication:**
- Bearer token authentication
- Token generated via `POST /v1/platform/orgs/current/scim/tokens`

**Purpose:**
The SCIM API enables automated user provisioning and deprovisioning between identity providers (IdP) and LangSmith organizations.

**Main Capabilities:**

### User Management
- **Create**: Provision users from IdP
- **Read**: Query user details
- **Update**: Sync user attribute changes
- **Delete**: Deprovision users

### Group Management
- **Create**: Create groups mapped to workspace roles
- **Read**: Query group membership
- **Update**: Modify group assignments
- **Delete**: Remove groups

**Supported Attributes:**
- User: `userName`, `name.givenName`, `name.familyName`, `emails`, `active`
- Group: `displayName`, `members`

**Use Cases:**
- Automated user onboarding/offboarding
- Syncing org structure from IdP
- RBAC enforcement via group membership
- Compliance and audit requirements

**Requirements:**
- Enterprise plan
- SAML SSO configured (Cloud) or OAuth with Client Secret (Self-hosted)
- IdP must support SCIM 2.0 (Okta, Azure AD, etc.)

**Token Management Endpoints:**
- `POST /v1/platform/orgs/current/scim/tokens` - Generate token
- `GET /v1/platform/orgs/current/scim/tokens` - List tokens
- `DELETE /v1/platform/orgs/current/scim/tokens/{id}` - Revoke token

---

## 5. Additional Endpoints

### OpenTelemetry (OTLP) Ingestion

**Base URLs:**
- Traces: `https://api.smith.langchain.com/otel/v1/traces`
- Logs: `https://api.smith.langchain.com/otel/v1/logs`
- Claude Code: `https://api.smith.langchain.com/otel/v1/claude_code`

**Purpose:**
- Ingest traces and logs using OpenTelemetry Protocol (OTLP)
- Alternative to native LangSmith tracing APIs
- Enables integration with OTLP-compatible tools

### Public JSON Schemas

**URLs:**
- Messages: `https://api.smith.langchain.com/public/schemas/v1/message.json`
- Tool definitions: `https://api.smith.langchain.com/public/schemas/v1/tooldef.json`

**Purpose:**
- JSON Schema definitions for standard formats
- OpenAI-compatible message and tool schemas

---

## Regional Differences

### US Region
- LangSmith: `api.smith.langchain.com`
- Control Plane: `api.host.langchain.com`
- UI: `smith.langchain.com`
- Auth: `auth.langchain.com`

### EU Region
- LangSmith: `eu.api.smith.langchain.com`
- Control Plane: `eu.api.host.langchain.com`
- UI: `eu.smith.langchain.com`
- Auth: `eu.auth.langchain.com`

### Self-Hosted
- All APIs available at custom host
- Control Plane at `/api-host` path
- LangSmith API typically at `/api/v1` path
- Configuration depends on ingress setup

---

## Implementation Priorities for Rust SDK/CLI

Based on the scope of issue #92 and common use cases, here are recommended priorities:

### Phase 1: Control Plane API (Current)
**Status:** In progress (issue #92)
- Deployments: List, Get, Create, Update, Delete
- Revisions: List, Get, Redeploy
- Essential for CI/CD workflows

### Phase 2: LangSmith API (Tracing & Datasets)
**High Value Operations:**
- Runs: Create, List, Query
- Feedback: Create, List
- Datasets: Create, List, Get
- Examples: Create, List, Bulk upload
- Tracer Sessions: Create, List, Get

### Phase 3: LangGraph Server API (Runtime)
**Core Operations:**
- Assistants: List, Get, Search
- Threads: Create, Get, List
- Runs: Create, Stream, Wait
- Store: Get, Put, List

### Phase 4: Organization Management
**Admin Operations:**
- Workspaces: List, Create, Update
- Members: List, Invite, Remove
- Roles: List, Assign
- API Keys: Create, List, Delete

### Phase 5: Advanced Features
- SCIM integration (Enterprise customers)
- Bulk exports
- Webhooks
- Cron jobs

---

## Development Notes

### Code Generation Strategy

1. **OpenAPI to Rust SDK:**
   - Use `openapi-generator` or `progenitor` for initial generation
   - Customize generated code for ergonomic Rust APIs
   - Add async support with `tokio` and `reqwest`
   - Implement proper error handling with `thiserror`

2. **SDK to CLI:**
   - Build CLI commands using `clap`
   - Map CLI flags/options to SDK method parameters
   - Support JSON output for scripting
   - Provide table/formatted output for interactive use

3. **Authentication:**
   - Support environment variables (`LANGSMITH_API_KEY`, `WORKSPACE_ID`)
   - Support config file (`~/.langsmith/config`)
   - CLI flag overrides for all auth parameters

4. **Configuration:**
   - Region selection (US/EU/Self-hosted)
   - Custom base URL support
   - Timeout and retry configuration

### Testing Approach

1. **Unit Tests:** Generated from OpenAPI examples
2. **Integration Tests:** Against LangSmith Cloud (US region)
3. **E2E Tests:** Full workflows (create deployment → wait → delete)
4. **Mock Server:** For CI/CD testing without real API calls

---

## References

- [LangSmith Documentation](https://docs.smith.langchain.com)
- [LangGraph Server Documentation](https://langchain-ai.github.io/langgraph/cloud/)
- [Control Plane API Reference](https://api.host.langchain.com/docs)
- [LangSmith API Reference](https://api.smith.langchain.com/redoc)
- [LangGraph Platform API Reference](https://langchain-ai.github.io/langgraph/cloud/reference/api/api_ref.html)
- [SCIM 2.0 Specification (RFC 7644)](https://datatracker.ietf.org/doc/html/rfc7644)
- [OpenTelemetry Protocol (OTLP) Specification](https://opentelemetry.io/docs/specs/otlp/)

---

## Changelog

- **2025-11-03**: Initial document created with overview of all LangChain HTTP/REST APIs
