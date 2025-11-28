# Control Plane API Extracted Fragments

## Source

- **Full Spec**: `../../openapi/langchain/control-plane/openapi.json`
- **Remote URL**: `https://api.host.langchain.com/openapi.json`

## Purpose

These fragments are extracted subsets of the full OpenAPI spec, optimized for:
- AI context window efficiency (small file sizes)
- Focused reference for specific features
- Quick lookup without loading full spec

## Fragments

| File | Size | Purpose | jq Query | Last Updated |
|------|------|---------|----------|--------------|
| *(none yet)* | - | - | - | - |

## Extraction Commands

Run these from the `reference/openapi/langchain/control-plane/` directory:

```bash
# Example: Extract deployment endpoints
jq '.paths | with_entries(select(.key | contains("deployment")))' \
  openapi.json > ../../api-specs/control-plane/deployments-endpoints.json

# Example: Extract deployment schemas
jq '.components.schemas | with_entries(select(.key | test("Deployment"; "i")))' \
  openapi.json > ../../api-specs/control-plane/deployments-schemas.json
```

## Adding New Fragments

1. Identify the jq query needed to extract the subset
2. Run extraction command and save to this directory
3. Update this FRAGMENTS.md with file info and jq query
4. Commit both the fragment and updated FRAGMENTS.md

## Related

- **Full spec**: `../../openapi/langchain/control-plane/openapi.json`
- **Spec manifest**: `../../openapi/langchain/control-plane/MANIFEST.md`
