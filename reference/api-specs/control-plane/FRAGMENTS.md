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
| `deployment-endpoints.json` | 19K | All /deployments API endpoints | See below | 2025-12-11 |
| `deployment-schemas.json` | 6.2K | All deployment-related schemas | See below | 2025-12-11 |
| `deployment-schema.json` | 3.1K | Single Deployment schema | See below | 2025-12-11 |
| `source-config-schema.json` | 4.3K | SourceConfig schema | See below | 2025-12-11 |

## Extraction Commands

Run these from the `reference/openapi/langchain/control-plane/` directory:

```bash
# Deployment endpoints
jq '.paths | with_entries(select(.key | test("/deployments")))' \
  openapi.json > ../../api-specs/control-plane/deployment-endpoints.json

# Deployment schemas
jq '.components.schemas | with_entries(select(.key | test("Deployment"; "i")))' \
  openapi.json > ../../api-specs/control-plane/deployment-schemas.json

# Deployment schema (single)
jq '.components.schemas.Deployment' \
  openapi.json > ../../api-specs/control-plane/deployment-schema.json

# SourceConfig schema
jq '.components.schemas.SourceConfig' \
  openapi.json > ../../api-specs/control-plane/source-config-schema.json
```

## Key Schema Facts

### Deployment Response Model

The `Deployment` schema has these **top-level** fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | UUID | Yes | System-generated identifier |
| `name` | string | Yes | User-assigned name |
| `source` | enum | Yes | "github" or "external_docker" |
| `source_config` | object | Yes | Contains `deployment_type` and other config |
| `source_revision_config` | object | Yes | Git ref or image URI config |
| `secrets` | array | Yes | Environment variable secrets |
| `created_at` | datetime | Yes | Creation timestamp |
| `updated_at` | datetime | Yes | Last update timestamp |
| `status` | enum | Yes | AWAITING_DATABASE, READY, UNUSED, AWAITING_DELETE, UNKNOWN |
| `latest_revision_id` | UUID | Yes | Latest revision ID |
| `active_revision_id` | UUID | Yes | Currently deployed revision ID |
| `image_version` | string | No | Optional image version |

### SourceConfig Nested Object

The `source_config` field contains these properties (all optional/nullable):

| Field | Type | Applies To | Description |
|-------|------|------------|-------------|
| `integration_id` | UUID | GitHub | GitHub integration ID |
| `repo_url` | string | GitHub | Repository URL |
| `deployment_type` | enum | GitHub | "dev_free", "dev", or "prod" |
| `build_on_push` | boolean | GitHub | Auto-deploy on push |
| `custom_url` | string | Both | Custom deployment URL |
| `resource_spec` | object | Both | Resource allocation config |
| `listener_id` | UUID | Both | Listener configuration ID |
| `listener_config` | object | Both | Listener settings |
| `install_command` | string | GitHub | Custom install command |
| `build_command` | string | GitHub | Custom build command |

**Note**: `deployment_type` is nested inside `source_config`, not a top-level field.

## Adding New Fragments

1. Identify the jq query needed to extract the subset
2. Run extraction command and save to this directory
3. Update this FRAGMENTS.md with file info and jq query
4. Commit both the fragment and updated FRAGMENTS.md

## Related

- **Full spec**: `../../openapi/langchain/control-plane/openapi.json`
- **Spec manifest**: `../../openapi/langchain/control-plane/MANIFEST.md`
