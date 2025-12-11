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

## Deployment Schema Notes

### Top-Level Fields

The `Deployment` response model contains these **direct** fields:
- `id` (UUID, read-only)
- `name` (string)
- `source` (enum: "github" | "external_docker")
- `source_config` (object, see below)
- `source_revision_config` (object)
- `secrets` (array of Secret objects)
- `created_at` (datetime)
- `updated_at` (datetime)
- `status` (enum: AWAITING_DATABASE | READY | UNUSED | AWAITING_DELETE | UNKNOWN)
- `latest_revision_id` (UUID, nullable)
- `active_revision_id` (UUID, nullable)
- `image_version` (string, nullable)

### SourceConfig Nested Fields

**IMPORTANT**: `deployment_type` is **NOT** a top-level field. It's nested inside `source_config`:

```json
{
  "source_config": {
    "integration_id": "uuid",
    "repo_url": "string",
    "deployment_type": "dev_free" | "dev" | "prod",  // ← HERE
    "build_on_push": boolean,
    "custom_url": "string",
    "resource_spec": {...},
    "listener_id": "uuid",
    "listener_config": {...},
    "install_command": "string",
    "build_command": "string"
  }
}
```

### SDK Implications

In the Rust SDK (`sdk/src/deployments.rs`):
- `source_config` is typed as `Option<serde_json::Value>` (unparsed JSON)
- To access `deployment_type`, you must parse the JSON: `deployment.source_config?.get("deployment_type")`
- This is why it's not included in the default available columns for text output
- Accessing it requires JSON parsing overhead on every row

### Text Output Decision (Issue #584, PR #692)

**Decision**: Exclude `deployment_type` from available columns because:
1. It requires JSON parsing (performance cost)
2. It's only present for GitHub source deployments (would be null for external_docker)
3. Not worth the complexity for a field that's rarely used in list views
4. Can still be accessed via JSON output format or `deployment get` command

If needed in the future, it can be added as an opt-in column with a performance caveat in the help text.

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
