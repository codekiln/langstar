# ADR-0002: OpenAPI Spec Versioning

**Status:** Accepted  
**Date:** 2025-11-13  
**Decision Makers:** Langstar Development Team  
**Related Issues:** #106, #115

## Context

Langstar generates SDK code from OpenAPI specifications provided by LangChain services (LangSmith and LangGraph Cloud). We need a system to:

1. **Track which spec version generated which SDK code**
2. **Detect when upstream APIs have changed** (new endpoints, modified parameters, breaking changes)
3. **Maintain history** of spec updates and SDK generation
4. **Enable reproducible builds** - know exactly which spec generated current SDK
5. **Support developer workflow** for updating specs and regenerating code

### Current State

- OpenAPI specs are fetched on-demand by `tools/generate_sdk.sh`
- No version tracking exists
- No historical record of spec updates
- No way to know if local SDK matches upstream API
- `tools/specs/` directory doesn't exist yet (will be created when generation runs)

### Requirements

- Track spec version, source URL, fetch date
- Compute checksums to detect changes
- Link spec versions to git commits
- Support manual spec updates (Phase 2)
- Prepare for automated spec updates (Phase 3)
- Human-readable format for easy review

## Decision

We implement a **JSON-based version tracking system** with a `versions.json` manifest file that records metadata about each OpenAPI specification.

### File Location

**Primary manifest:**
```
tools/specs/versions.json
```

This file is version-controlled and commits alongside spec updates.

### Manifest Format

```json
{
  "format_version": "1.0",
  "last_updated": "2025-11-13T12:00:00Z",
  "specs": {
    "langsmith": {
      "spec_version": "unknown",
      "spec_file": "langsmith-openapi.json",
      "spec_url": "https://api.smith.langchain.com/openapi.json",
      "fetched_at": "2025-11-13T12:00:00Z",
      "fetched_by": "tools/generate_sdk.sh",
      "spec_checksum": {
        "algorithm": "sha256",
        "value": "abc123def456..."
      },
      "sdk_generated": true,
      "sdk_generation": {
        "tool": "openapi-generator-cli",
        "tool_version": "7.10.0",
        "generator": "rust",
        "generated_at": "2025-11-13T12:05:00Z",
        "output_dir": "sdk/src/generated/langsmith"
      },
      "git_commit": "abc123d",
      "notes": "Initial generation for Phase 2 implementation"
    },
    "langgraph": {
      "spec_version": "unknown",
      "spec_file": "langgraph-openapi.json",
      "spec_url": "https://api.langgraph.cloud/openapi.json",
      "fetched_at": "2025-11-13T12:00:00Z",
      "fetched_by": "tools/generate_sdk.sh",
      "spec_checksum": {
        "algorithm": "sha256",
        "value": "789ghi012jkl..."
      },
      "sdk_generated": true,
      "sdk_generation": {
        "tool": "openapi-generator-cli",
        "tool_version": "7.10.0",
        "generator": "rust",
        "generated_at": "2025-11-13T12:05:00Z",
        "output_dir": "sdk/src/generated/langgraph"
      },
      "git_commit": "abc123d",
      "notes": "Initial generation for Phase 2 implementation"
    }
  }
}
```

### Field Definitions

#### Root Level
- `format_version`: Version of this manifest format (for future evolution)
- `last_updated`: Timestamp of last change to this file
- `specs`: Map of service name to spec metadata

#### Per-Spec Metadata
- `spec_version`: API version from spec (if available in OpenAPI), otherwise "unknown"
- `spec_file`: Filename of the saved spec
- `spec_url`: Source URL where spec was fetched
- `fetched_at`: ISO 8601 timestamp of when spec was fetched
- `fetched_by`: Tool or person who fetched the spec
- `spec_checksum`: SHA-256 checksum of the spec file
  - `algorithm`: Hash algorithm used
  - `value`: Hex-encoded hash value
- `sdk_generated`: Boolean indicating if SDK was generated from this spec
- `sdk_generation`: Details of SDK generation (optional, only if `sdk_generated` is true)
  - `tool`: Name of generator tool
  - `tool_version`: Version of generator tool
  - `generator`: Generator type (e.g., "rust", "typescript")
  - `generated_at`: When SDK was generated
  - `output_dir`: Where generated code was placed
- `git_commit`: Short commit hash of the git commit that added/updated this spec
- `notes`: Human-readable notes about this spec update

### Checksum Calculation

Checksums are computed using SHA-256 over the entire spec file:

```bash
# On Linux/macOS
sha256sum tools/specs/langsmith-openapi.json

# On all platforms with openssl
openssl dgst -sha256 tools/specs/langsmith-openapi.json
```

### Version Tracking Workflow

#### 1. Initial Setup (Phase 2)
```bash
# Create versions.json with initial structure
./tools/generate_sdk.sh  # Script updated to create/update versions.json
git add tools/specs/versions.json tools/specs/*.json
git commit -m "🔧 chore(sdk): add OpenAPI spec version tracking"
```

#### 2. Manual Spec Update (Developer Workflow)
```bash
# 1. Fetch latest specs
./tools/fetch_specs.sh  # New script (Phase 2)

# 2. Review changes
git diff tools/specs/langsmith-openapi.json
git diff tools/specs/versions.json

# 3. Regenerate SDK
./tools/generate_sdk.sh

# 4. Test changes
cargo test

# 5. Commit
git add tools/specs/ sdk/src/generated/
git commit -m "⬆️ upgrade(sdk): update OpenAPI specs from upstream"
```

#### 3. Checking for Drift
```bash
# Compare local spec checksum with upstream
./tools/check_spec_drift.sh  # New script (Phase 2)
```

Example output:
```
Checking for API drift...

LangSmith API:
  Local checksum:    abc123def456...
  Remote checksum:   abc123def456...
  Status:            ✓ Up to date

LangGraph API:
  Local checksum:    789ghi012jkl...
  Remote checksum:   999xyz000abc...
  Status:            ⚠ Drift detected!
  
Run './tools/fetch_specs.sh' to update specs.
```

### Integration with generate_sdk.sh

The `tools/generate_sdk.sh` script will be updated to:

1. Read existing `versions.json` (if it exists)
2. Fetch OpenAPI specs from URLs
3. Calculate checksums of fetched specs
4. Compare with stored checksums to detect changes
5. Update `versions.json` with new metadata
6. Generate SDK code
7. Update `sdk_generation` section in `versions.json`
8. Print summary of changes

Example enhanced output:
```
OpenAPI SDK Generation for Langstar

Step 1: Fetching OpenAPI specifications
ℹ Checking for spec updates...
✓ LangSmith spec unchanged (checksum: abc123...)
⚠ LangGraph spec updated (new checksum: 999xyz...)
✓ Saved specs to tools/specs/
✓ Updated versions.json

Step 2: Generating Rust client code
ℹ Generating LangSmith client (up to date)...
✓ Generated to sdk/src/generated/langsmith/
ℹ Generating LangGraph client (updated)...
✓ Generated to sdk/src/generated/langgraph/

Step 3: Summary
  Specs fetched: 2
  Specs changed: 1 (langgraph)
  SDK regenerated: 1 (langgraph)
  
Next steps:
  1. Review changes: git diff tools/specs/ sdk/src/generated/
  2. Test: cargo test
  3. Commit: git add tools/specs/ sdk/src/generated/
```

## Consequences

### Positive

1. **Traceability**: Clear record of which spec version generated which SDK
2. **Change Detection**: Easy to detect when upstream APIs have changed
3. **Reproducibility**: Can recreate SDK from specific spec version
4. **Audit Trail**: Full history of spec updates in git
5. **Automated Checks**: Can build tools to detect drift automatically
6. **Human-Readable**: JSON format is easy to read and review
7. **Future-Proof**: Format version allows for evolution
8. **CI/CD Ready**: Manifest can be parsed by automation tools

### Negative

1. **Manual Updates Required**: Developers must remember to update specs (Phase 2)
2. **File Size**: Storing full specs in git increases repo size
3. **Merge Conflicts**: Changes to versions.json or specs can cause conflicts
4. **Checksum Maintenance**: Must recalculate on every spec update
5. **Additional Files**: One more file to maintain in the repo

### Mitigation Strategies

1. **Documentation**: Clear developer workflow in runbook (see ADR-0004)
2. **Git LFS**: Could use Git LFS for large spec files (if needed)
3. **Automation**: Phase 3 will automate spec updates via CI/CD
4. **Tooling**: Scripts to automate checksum calculation and version updates
5. **PR Templates**: Remind developers to update specs when needed

## Alternatives Considered

### Alternative 1: Embedded Version in Generated Code

**Description:** Embed spec version and checksum in generated code comments.

```rust
// Generated from langsmith-openapi.json
// Spec checksum: abc123def456...
// Generated at: 2025-11-13T12:00:00Z
pub mod langsmith_client {
    // ...
}
```

**Pros:**
- Version info travels with generated code
- No separate manifest file needed
- Easy to see version in IDE

**Cons:**
- Harder to query version info programmatically
- No centralized view of all spec versions
- Difficult to track history
- Requires parsing code to extract version

**Rejected Because:** Need centralized, machine-readable version tracking.

### Alternative 2: Spec Versions in Separate Files

**Description:** Create individual version files for each spec.

```
tools/specs/
├── langsmith-openapi.json
├── langsmith-openapi.version.json
├── langgraph-openapi.json
└── langgraph-openapi.version.json
```

**Pros:**
- One version file per spec
- No merge conflicts between specs
- Easy to see version alongside spec

**Cons:**
- Multiple files to manage
- Harder to get overview of all versions
- More files to commit and track
- No single source of truth

**Rejected Because:** Single manifest provides better overview and is easier to manage.

### Alternative 3: Git Tags for Versions

**Description:** Use git tags to mark spec versions.

```bash
git tag langsmith-spec-2025-11-13
git tag langgraph-spec-2025-11-13
```

**Pros:**
- Uses existing git infrastructure
- Clear versioning in git history
- Easy to check out specific version

**Cons:**
- No checksum tracking
- No generation metadata
- Clutters git tag namespace
- Harder to query programmatically
- Requires separate metadata storage anyway

**Rejected Because:** Git tags don't provide enough metadata (checksums, generation info, etc.).

### Alternative 4: Database-Based Tracking

**Description:** Store version info in SQLite database.

**Pros:**
- Easy to query
- Efficient storage
- Relational structure

**Cons:**
- Binary format (not human-readable)
- Not version-controlled (or conflicts on every update)
- Requires database tooling
- Overkill for simple version tracking
- Harder to review in PRs

**Rejected Because:** Overkill for this use case. JSON is simpler and version-control friendly.

### Alternative 5: Spec URL with Version Parameter

**Description:** Include version in spec URL if API supports it.

```
https://api.smith.langchain.com/openapi.json?version=2025-11-13
```

**Pros:**
- Explicit version in URL
- Could fetch specific versions
- Upstream-controlled versioning

**Cons:**
- LangChain APIs don't currently support versioned specs
- Still need local version tracking
- Assumes upstream versioning system
- No control over upstream versioning

**Rejected Because:** Depends on upstream API features that don't exist. Need local solution.

## References

- [Semantic Versioning](https://semver.org/)
- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)
- [Git LFS for Large Files](https://git-lfs.github.com/)
- [SHA-256 Checksum](https://en.wikipedia.org/wiki/SHA-2)
- [Issue #106: SDK Generation Strategy](https://github.com/codekiln/langstar/issues/106)
- [Issue #115: Phase 1 Research & Design](https://github.com/codekiln/langstar/issues/115)

## Implementation Notes

See [ADR-0004: Drift Detection Workflow](./0004-drift-detection-workflow.md) for detailed developer workflow using this versioning system.

See [ADR-0003: Changelog Integration Structure](./0003-changelog-integration-structure.md) for how spec version changes flow into changelogs.

## Phase 2 Implementation Tasks

1. Create `tools/specs/versions.json` with initial structure
2. Update `tools/generate_sdk.sh` to read/write `versions.json`
3. Add checksum calculation to generation script
4. Create `tools/fetch_specs.sh` helper script
5. Create `tools/check_spec_drift.sh` helper script
6. Document developer workflow in runbook
7. Add version tracking to CI/CD (Phase 3)
