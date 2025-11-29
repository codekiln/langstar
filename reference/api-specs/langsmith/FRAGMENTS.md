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
| `dataset-endpoints.json` | 61K | Dataset API endpoints | See below | 2025-11-28 |
| `dataset-schemas.json` | 43K | Dataset data types | See below | 2025-11-28 |
| `example-endpoints.json` | 30K | Example API endpoints | See below | 2025-11-28 |
| `example-schemas.json` | 29K | Example data types | See below | 2025-11-28 |
| `evals-endpoints.json` | 45K | Evaluation/feedback API endpoints | See below | 2025-11-28 |
| `evals-schemas.json` | 38K | Evaluation/feedback data types | See below | 2025-11-28 |
| `prompt-endpoints.json` | 45K | Prompt repository/commit API endpoints | See below | 2025-11-29 |
| `prompt-schemas.json` | 37K | Prompt/commit/manifest data types | See below | 2025-11-29 |

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

# Dataset endpoints (all /api/v1/datasets paths)
jq '.paths | with_entries(select(.key | test("^/api/v1/datasets")))' \
  openapi.json > ../../api-specs/langsmith/dataset-endpoints.json

# Dataset schemas (all schemas containing "dataset" in name)
jq '.components.schemas | with_entries(select(.key | test("[Dd]ataset"; "i")))' \
  openapi.json > ../../api-specs/langsmith/dataset-schemas.json

# Example endpoints (all /api/v1/examples paths)
jq '.paths | with_entries(select(.key | test("^/api/v1/examples")))' \
  openapi.json > ../../api-specs/langsmith/example-endpoints.json

# Example schemas (all schemas containing "example" in name)
jq '.components.schemas | with_entries(select(.key | test("[Ee]xample"; "i")))' \
  openapi.json > ../../api-specs/langsmith/example-schemas.json

# Evaluation/feedback endpoints (all feedback/evaluator paths)
jq '.paths | with_entries(select(.key | test("feedback|evaluator"; "i")))' \
  openapi.json > ../../api-specs/langsmith/evals-endpoints.json

# Evaluation/feedback schemas (all schemas containing "feedback", "evaluator", or "eval")
jq '.components.schemas | with_entries(select(.key | test("feedback|evaluator|eval"; "i")))' \
  openapi.json > ../../api-specs/langsmith/evals-schemas.json

# Prompt repository and commit endpoints (all /api/v1/repos and /api/v1/commits paths)
jq '.paths | with_entries(select(.key | test("^/api/v1/(repos|commits)")))' \
  openapi.json > ../../api-specs/langsmith/prompt-endpoints.json

# Prompt/commit/manifest schemas (all schemas containing "repo", "commit", "prompt", or "manifest")
jq '.components.schemas | with_entries(select(.key | test("[Rr]epo|[Cc]ommit|[Pp]rompt|[Mm]anifest"; "i")))' \
  openapi.json > ../../api-specs/langsmith/prompt-schemas.json
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
- **Validation reports**:
  - `../../research/298-openapi-validation.md` - Runs query validation
  - `../../research/334-openapi-validation.md` - Annotation queues validation
  - `../../research/347-openapi-validation.md` - Evaluations/feedback validation
  - `../../research/402-structured-prompts-openapi-validation.md` - Structured output prompts validation
