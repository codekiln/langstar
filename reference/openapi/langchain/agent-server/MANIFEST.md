# Agent Server API OpenAPI Specification

## Source

- **URL Pattern**: `https://<deployment-url>/openapi.json`
- **Test Deployment**: the shared `pr-integration-test-*` deployment that the integration tests reuse, built from `tests/fixtures/test-graph-deployment/langgraph.json`. `langstar deployment list -f json` gives its URL in `source_config.custom_url`; the refresh command below looks it up by name.
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

Needs `LANGSMITH_API_KEY` and `LANGSMITH_WORKSPACE_ID`, which `langstar deployment list` uses to read the control plane.

```bash
# Fetch from the shared pr-integration-test-* deployment and indent with 2 spaces.
# The spec is replaced only after the fetch and jq both succeed, so a failed
# request or invalid JSON leaves the committed file as it was.
set -euo pipefail
SPEC=reference/openapi/langchain/agent-server/openapi.json
DEPLOYMENT_URL=$(langstar deployment list --name-contains pr-integration-test- -f json \
  | jq -er '[.resources[] | select(.name | startswith("pr-integration-test-"))][0].source_config.custom_url')
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT
curl -fsS -H "x-api-key: $LANGSMITH_API_KEY" "$DEPLOYMENT_URL/openapi.json" \
  | jq --indent 2 . > "$TMP"
mv "$TMP" "$SPEC"
```

## Related

- **Extracted fragments**: `../../api-specs/agent-server/`
- **API overview**: `../../api-specs/LANGSMITH_API_OVERVIEW.md`
- **Research**: `docs/research/528-graph-api-research.md`
