# LangSmith API Specifications

This directory contains documentation and extracted fragments from LangSmith ecosystem OpenAPI specifications.

## Quick Links

- **[LANGSMITH_API_OVERVIEW.md](./LANGSMITH_API_OVERVIEW.md)** - Quick reference table for all 4 APIs
- **[LANGSMITH_APIS_DETAILS.md](./LANGSMITH_APIS_DETAILS.md)** - Comprehensive catalog with endpoints, auth, and examples

## Directory Structure

```
api-specs/
├── README.md                      # This file
├── LANGSMITH_API_OVERVIEW.md      # Quick reference (4 APIs)
├── LANGSMITH_APIS_DETAILS.md      # Detailed catalog
│
├── langsmith/                     # LangSmith core API fragments
│   ├── FRAGMENTS.md               # Index of fragments with jq queries
│   ├── annotation-queue-*.json    # Annotation queue endpoints/schemas
│   ├── run-schema.json            # Run data structures
│   └── runs-query-*.json          # Runs query endpoint/schemas
│
├── control-plane/                 # Control Plane API fragments
│   └── FRAGMENTS.md               # Index (no fragments yet)
│
└── agent-server/                  # Agent Server API fragments (per-deployment)
    ├── FRAGMENTS.md               # Index with jq queries
    ├── graph-endpoint.json        # GET /assistants/{id}/graph
    ├── subgraphs-endpoints.json   # GET /assistants/{id}/subgraphs
    ├── assistants-search-endpoint.json  # POST /assistants/search
    ├── schemas-endpoint.json      # GET /assistants/{id}/schemas
    ├── graph-schemas.json         # GraphSchema, Subgraphs types
    └── assistant-*.json           # Assistant schema with graph_id
```

## Design Pattern

This structure follows the **"canonical source + derived fragments"** pattern:

| Location | Content | Version Control | Purpose |
|----------|---------|-----------------|---------|
| `../openapi/langchain/{api}/openapi.json` | Full specs | Committed (large) | Source of truth for jq queries |
| `../openapi/langchain/{api}/MANIFEST.md` | Provenance | Committed | When/how specs were fetched |
| `api-specs/{api}/*.json` | Fragments | Committed (small) | AI context grounding |
| `api-specs/{api}/FRAGMENTS.md` | Index | Committed | jq queries for reproducibility |

### Benefits

1. **AI-friendly**: Small fragments fit in context windows
2. **Reproducible**: jq queries documented for re-extraction
3. **Traceable**: MANIFEST.md and FRAGMENTS.md track provenance
4. **Maintainable**: Full specs can be refreshed without losing fragments

## Usage

### Finding API Information

```bash
# Quick lookup of base URLs
cat LANGSMITH_API_OVERVIEW.md

# Detailed endpoint information
cat LANGSMITH_APIS_DETAILS.md

# Specific fragment (e.g., annotation queue schemas)
cat langsmith/annotation-queue-schemas.json | jq '.AnnotationQueueSchema'
```

### Querying Full Specs

```bash
# List all dataset endpoints
jq '.paths | keys | map(select(contains("dataset")))' \
  ../openapi/langchain/langsmith/openapi.json

# Get schema for a specific type
jq '.components.schemas.Dataset' \
  ../openapi/langchain/langsmith/openapi.json
```

### Extracting New Fragments

See `{api}/FRAGMENTS.md` for extraction commands and patterns.

## Related Directories

- `../openapi/` - Full OpenAPI specification files
- `../research/` - Research reports using these specs
- `../repo/langchain-ai/` - Cloned LangChain repositories for reference

## Updating Specs

1. **Refresh full spec**:
   ```bash
   curl -o ../openapi/langchain/langsmith/openapi.json \
     https://api.smith.langchain.com/openapi.json
   ```

2. **Update MANIFEST.md** with new provenance entry

3. **Re-extract fragments** if needed (see FRAGMENTS.md for commands)

4. **Commit all changes** together
