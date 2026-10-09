# Agent Server API OpenAPI Specification

## Source

- **URL Pattern**: `https://<deployment-url>/openapi.json`
- **Test Deployment**: the shared `pr-integration-test-*` deployment that the integration tests reuse. Its URL is left out of this repository; look it up with the refresh command below.
- **Fetched**: 2026-10-09
- **Version**: 0.1.0

## Nature of This API

Unlike the Control Plane and LangSmith APIs which have centralized endpoints, the Agent Server API is served **per-deployment**. Each LangGraph deployment hosts its own Agent Server API at the deployment's runtime URL.

Key characteristics:
- OpenAPI spec is at `/openapi.json` on each deployment
- Interactive docs at `/docs`
- Core API structure is consistent across deployments
- Schema enums (e.g., `graph_id` values) vary based on deployed graphs

## Authentication

Same as other LangSmith APIs:
```bash
curl -H "x-api-key: $LANGSMITH_API_KEY" "https://<deployment-url>/openapi.json"
```

## Refresh Command

```bash
# Fetch from any active deployment
DEPLOYMENT_URL=$(langstar graph list --limit 1 -f json | jq -r '.resources[0].source_config.custom_url')
curl -H "x-api-key: $LANGSMITH_API_KEY" "$DEPLOYMENT_URL/openapi.json" \
  -o reference/openapi/langchain/agent-server/openapi.json
```

## Related

- **Extracted fragments**: `../../api-specs/agent-server/`
- **API overview**: `../../api-specs/LANGSMITH_API_OVERVIEW.md`
- **Research**: `docs/research/528-graph-api-research.md`
