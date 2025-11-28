# LangSmith API Extracted Fragments

## Source

- **Full Spec**: `../../openapi/langchain/langsmith/openapi.json`
- **Remote URL**: `https://api.smith.langchain.com/openapi.json`

## Purpose

These fragments are extracted subsets of the full OpenAPI spec, optimized for:
- AI context window efficiency (small file sizes)
- Focused reference for specific features
- Quick lookup without loading 635K+ of JSON

## Fragments

| File | Size | Purpose | jq Query | Last Updated |
|------|------|---------|----------|--------------|
| `annotation-queue-endpoints.json` | 2.3K | Annotation queue API endpoints | See below | 2025-11-26 |
| `annotation-queue-schemas.json` | 45K | Annotation queue data types | See below | 2025-11-26 |
| `run-schema.json` | 9.8K | Run/trace data structures | See below | 2025-11-26 |
| `runs-query-endpoint.json` | 1.0K | POST /runs/query endpoint | See below | 2025-11-26 |
| `runs-query-request-schema.json` | 5.5K | Runs query request payload | See below | 2025-11-26 |
| `runs-query-response-schema.json` | 1.1K | Runs query response format | See below | 2025-11-26 |

## Extraction Commands

Run these from the `reference/openapi/langchain/langsmith/` directory:

```bash
# Annotation queue endpoints
jq '.paths | with_entries(select(.key | contains("annotation-queue")))' \
  openapi.json > ../../api-specs/langsmith/annotation-queue-endpoints.json

# Annotation queue schemas
jq '.components.schemas | with_entries(select(.key | test("Annotation|Queue"; "i")))' \
  openapi.json > ../../api-specs/langsmith/annotation-queue-schemas.json

# Run schema
jq '.components.schemas.Run' \
  openapi.json > ../../api-specs/langsmith/run-schema.json

# Runs query endpoint
jq '.paths["/api/v1/runs/query"]' \
  openapi.json > ../../api-specs/langsmith/runs-query-endpoint.json

# Runs query request schema
jq '.components.schemas.RunQueryRequest // .components.schemas.FilterQueryRequest' \
  openapi.json > ../../api-specs/langsmith/runs-query-request-schema.json

# Runs query response schema
jq '.components.schemas.RunQueryResponse // .components.schemas.FilterQueryResponse' \
  openapi.json > ../../api-specs/langsmith/runs-query-response-schema.json
```

## Verification

To verify a fragment matches the source:

```bash
# Example: check annotation queue endpoints exist in source
jq '.paths | keys | map(select(contains("annotation-queue"))) | length' \
  ../../openapi/langchain/langsmith/openapi.json
```

## Adding New Fragments

1. Identify the jq query needed to extract the subset
2. Run extraction command and save to this directory
3. Update this FRAGMENTS.md with file info and jq query
4. Commit both the fragment and updated FRAGMENTS.md

## Related

- **Full spec**: `../../openapi/langchain/langsmith/openapi.json`
- **Spec manifest**: `../../openapi/langchain/langsmith/MANIFEST.md`
- **Research using these**: `../../research/334-annotation-queues-precedent.md`
