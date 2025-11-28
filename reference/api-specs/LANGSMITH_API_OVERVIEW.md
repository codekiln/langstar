# LangSmith API Overview

Quick reference for the four core LangChain ecosystem REST APIs.

| API | Base URL (US) | Base URL (EU) | OpenAPI Spec | Purpose |
|-----|---------------|---------------|--------------|---------|
| LangSmith API | `https://api.smith.langchain.com` | `https://eu.api.smith.langchain.com` | `https://api.smith.langchain.com/openapi.json` | Tracing, evaluation, datasets, org management |
| LangSmith Deployment Control Plane API | `https://api.host.langchain.com` | `https://eu.api.host.langchain.com` | `https://api.host.langchain.com/openapi.json` | Deployment management for LangGraph Server |
| LangSmith Deployment Agent Server API | Per-deployment | Per-deployment | `/docs` on each deployment | Runtime API for assistants, threads, runs |
| SCIM API | `https://api.smith.langchain.com/scim/v2` | `https://eu.api.smith.langchain.com/scim/v2` | SCIM 2.0 compliant | User provisioning (Enterprise) |

For detailed information including authentication, endpoints, schemas, and usage examples, see [LANGSMITH_APIS_DETAILS.md](./LANGSMITH_APIS_DETAILS.md).

