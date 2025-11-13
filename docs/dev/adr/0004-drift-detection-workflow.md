# ADR-0004: Drift Detection Workflow

**Status:** Accepted  
**Date:** 2025-11-13  
**Decision Makers:** Langstar Development Team  
**Related Issues:** #106, #115

## Context

"Drift" occurs when upstream LangChain APIs (LangSmith, LangGraph Cloud) change but our local SDK has not been updated. This can lead to:

- **Outdated functionality**: Missing new endpoints or features
- **Breaking changes**: SDK calls that fail due to changed API contracts
- **Incorrect behavior**: API parameters or responses that don't match expectations
- **Security issues**: Missing security patches or deprecation of vulnerable endpoints

We need a systematic way to:
1. **Detect drift** - Know when upstream APIs have changed
2. **Respond to drift** - Clear workflow for updating our SDK
3. **Prevent surprises** - Catch breaking changes before users do
4. **Maintain quality** - Ensure SDK stays in sync with upstream

### Current State

- No drift detection mechanism exists
- No regular spec update schedule
- Spec updates happen ad-hoc when needed
- No way to know if we're behind upstream

### Requirements

- **Manual workflow** for Phase 2 (automation deferred to Phase 3)
- **Developer-friendly** - simple to check for drift
- **Low overhead** - minimal manual work
- **Actionable** - clear next steps when drift detected
- **Documented** - easy to follow for new contributors

## Decision

We implement a **manual drift detection workflow** with supporting tools and documentation. Developers can check for drift on-demand, and a documented process guides them through updating specs and SDK.

### Drift Detection Script

**Location:** `tools/check_spec_drift.sh`

**Purpose:** Compare local OpenAPI specs with upstream to detect changes

**Implementation:**
```bash
#!/usr/bin/env bash
# Check for drift between local OpenAPI specs and upstream APIs

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
LANGSMITH_URL="https://api.smith.langchain.com/openapi.json"
LANGGRAPH_URL="https://api.langgraph.cloud/openapi.json"
SPECS_DIR="tools/specs"
VERSIONS_FILE="$SPECS_DIR/versions.json"

# Temporary directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo -e "${BLUE}Checking for API drift...${NC}"
echo ""

# Function to check drift for a service
check_drift() {
    local service=$1
    local url=$2
    local local_spec=$3
    
    echo -e "${BLUE}$service API:${NC}"
    
    # Fetch remote spec
    echo "  Fetching remote spec..."
    if ! curl -sSL "$url" -o "$TEMP_DIR/$service.json" 2>/dev/null; then
        echo -e "  ${RED}✗ Failed to fetch remote spec${NC}"
        return 1
    fi
    
    # Calculate checksums
    if command -v sha256sum >/dev/null 2>&1; then
        local_checksum=$(sha256sum "$local_spec" 2>/dev/null | awk '{print $1}' || echo "none")
        remote_checksum=$(sha256sum "$TEMP_DIR/$service.json" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        local_checksum=$(shasum -a 256 "$local_spec" 2>/dev/null | awk '{print $1}' || echo "none")
        remote_checksum=$(shasum -a 256 "$TEMP_DIR/$service.json" | awk '{print $1}')
    else
        echo -e "  ${RED}✗ No checksum tool available (sha256sum or shasum)${NC}"
        return 1
    fi
    
    # Compare
    echo "  Local checksum:  ${local_checksum:0:16}...${local_checksum: -16}"
    echo "  Remote checksum: ${remote_checksum:0:16}...${remote_checksum: -16}"
    
    if [ "$local_checksum" = "$remote_checksum" ]; then
        echo -e "  ${GREEN}✓ Up to date${NC}"
        return 0
    else
        echo -e "  ${YELLOW}⚠ Drift detected!${NC}"
        
        # Show brief diff summary if jq is available
        if command -v jq >/dev/null 2>&1; then
            echo "  Checking for changes..."
            
            # Count endpoints
            local_endpoints=$(jq -r '.paths | keys | length' "$local_spec" 2>/dev/null || echo "unknown")
            remote_endpoints=$(jq -r '.paths | keys | length' "$TEMP_DIR/$service.json" 2>/dev/null || echo "unknown")
            
            if [ "$local_endpoints" != "unknown" ] && [ "$remote_endpoints" != "unknown" ]; then
                local diff=$((remote_endpoints - local_endpoints))
                if [ $diff -gt 0 ]; then
                    echo -e "  ${YELLOW}  +$diff new endpoint(s)${NC}"
                elif [ $diff -lt 0 ]; then
                    echo -e "  ${YELLOW}  $diff removed endpoint(s)${NC}"
                fi
            fi
        fi
        
        return 1
    fi
    
    echo ""
}

# Check both services
drift_detected=0

if [ -f "$SPECS_DIR/langsmith-openapi.json" ]; then
    check_drift "LangSmith" "$LANGSMITH_URL" "$SPECS_DIR/langsmith-openapi.json" || drift_detected=1
else
    echo -e "${YELLOW}⚠ LangSmith spec not found locally${NC}"
    drift_detected=1
fi

echo ""

if [ -f "$SPECS_DIR/langgraph-openapi.json" ]; then
    check_drift "LangGraph" "$LANGGRAPH_URL" "$SPECS_DIR/langgraph-openapi.json" || drift_detected=1
else
    echo -e "${YELLOW}⚠ LangGraph spec not found locally${NC}"
    drift_detected=1
fi

echo ""

# Summary
if [ $drift_detected -eq 0 ]; then
    echo -e "${GREEN}✓ All specs are up to date${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Drift detected or specs missing${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run './tools/fetch_specs.sh' to update specs"
    echo "  2. Review changes: 'git diff tools/specs/'"
    echo "  3. Update upstream changelog: 'tools/specs/CHANGELOG.md'"
    echo "  4. Regenerate SDK: './tools/generate_sdk.sh'"
    echo "  5. Test changes: 'cargo test'"
    echo "  6. Commit: 'git add tools/specs/ sdk/src/generated/'"
    echo ""
    echo "See docs/dev/adr/0004-drift-detection-workflow.md for full workflow."
    exit 1
fi
```

### Spec Fetching Script

**Location:** `tools/fetch_specs.sh`

**Purpose:** Fetch latest OpenAPI specs from upstream and update `versions.json`

**Implementation:**
```bash
#!/usr/bin/env bash
# Fetch latest OpenAPI specifications from LangChain services

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
LANGSMITH_URL="https://api.smith.langchain.com/openapi.json"
LANGGRAPH_URL="https://api.langgraph.cloud/openapi.json"
SPECS_DIR="tools/specs"
VERSIONS_FILE="$SPECS_DIR/versions.json"

echo -e "${BLUE}Fetching OpenAPI specifications...${NC}"
echo ""

# Create directory
mkdir -p "$SPECS_DIR"

# Function to fetch spec
fetch_spec() {
    local service=$1
    local url=$2
    local output=$3
    
    echo -e "${BLUE}Fetching $service spec...${NC}"
    echo "  URL: $url"
    
    if command -v curl >/dev/null 2>&1; then
        curl -sSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    else
        echo -e "${RED}✗ Neither curl nor wget available${NC}"
        exit 1
    fi
    
    # Calculate checksum
    if command -v sha256sum >/dev/null 2>&1; then
        checksum=$(sha256sum "$output" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        checksum=$(shasum -a 256 "$output" | awk '{print $1}')
    else
        checksum="unknown"
    fi
    
    echo "  Checksum: ${checksum:0:16}...${checksum: -16}"
    echo -e "${GREEN}  ✓ Saved to $output${NC}"
    echo ""
}

# Fetch specs
fetch_spec "LangSmith" "$LANGSMITH_URL" "$SPECS_DIR/langsmith-openapi.json"
fetch_spec "LangGraph" "$LANGGRAPH_URL" "$SPECS_DIR/langgraph-openapi.json"

# Update versions.json (if it exists, create placeholder otherwise)
echo -e "${BLUE}Updating version tracking...${NC}"

if [ -f "$VERSIONS_FILE" ]; then
    echo "  Versions file exists, run './tools/generate_sdk.sh' to update metadata"
else
    echo "  Creating placeholder versions.json"
    cat > "$VERSIONS_FILE" << 'EOF'
{
  "format_version": "1.0",
  "last_updated": "TBD - run ./tools/generate_sdk.sh",
  "specs": {
    "langsmith": {
      "spec_version": "unknown",
      "spec_file": "langsmith-openapi.json",
      "spec_url": "https://api.smith.langchain.com/openapi.json",
      "fetched_at": "TBD",
      "fetched_by": "tools/fetch_specs.sh",
      "spec_checksum": {
        "algorithm": "sha256",
        "value": "TBD"
      },
      "sdk_generated": false,
      "notes": "Spec fetched, awaiting SDK generation"
    },
    "langgraph": {
      "spec_version": "unknown",
      "spec_file": "langgraph-openapi.json",
      "spec_url": "https://api.langgraph.cloud/openapi.json",
      "fetched_at": "TBD",
      "fetched_by": "tools/fetch_specs.sh",
      "spec_checksum": {
        "algorithm": "sha256",
        "value": "TBD"
      },
      "sdk_generated": false,
      "notes": "Spec fetched, awaiting SDK generation"
    }
  }
}
EOF
fi

echo -e "${GREEN}✓ Specs fetched successfully${NC}"
echo ""
echo "Next steps:"
echo "  1. Review changes: 'git diff tools/specs/'"
echo "  2. Update upstream changelog: 'tools/specs/CHANGELOG.md'"
echo "  3. Regenerate SDK: './tools/generate_sdk.sh'"
echo "  4. Test changes: 'cargo test'"
```

### Developer Runbook

**Location:** `docs/dev/runbooks/update-openapi-specs.md` (to be created in Phase 2)

**Content:**

```markdown
# Runbook: Updating OpenAPI Specifications

This runbook guides you through updating Langstar's OpenAPI specifications
when upstream LangChain APIs change.

## When to Update

- **Scheduled**: Monthly check for updates (1st of each month)
- **On-Demand**: When you need a new endpoint or feature
- **Reactive**: When users report API errors or missing functionality
- **Proactive**: When LangChain announces API changes

## Prerequisites

- Access to the internet (to fetch specs)
- Git working tree clean
- All tests passing

## Step-by-Step Workflow

### 1. Check for Drift

```bash
./tools/check_spec_drift.sh
```

**If no drift detected:** You're done! ✓

**If drift detected:** Continue to next step.

### 2. Fetch Latest Specs

```bash
./tools/fetch_specs.sh
```

This fetches the latest OpenAPI specs from:
- LangSmith: https://api.smith.langchain.com/openapi.json
- LangGraph Cloud: https://api.langgraph.cloud/openapi.json

### 3. Review Changes

```bash
git diff tools/specs/langsmith-openapi.json
git diff tools/specs/langgraph-openapi.json
```

**Look for:**
- ✅ New endpoints (additions)
- ⚠️ Removed endpoints (deletions)
- ⚠️ Changed parameters (modifications to existing endpoints)
- ⚠️ Changed models (modifications to request/response schemas)
- 🚨 Breaking changes (required parameters added, types changed)

**Use tools:**
```bash
# Count endpoints
jq '.paths | keys | length' tools/specs/langsmith-openapi.json

# List all endpoints
jq -r '.paths | keys[]' tools/specs/langsmith-openapi.json

# Compare endpoint counts
diff \
  <(git show HEAD:tools/specs/langsmith-openapi.json | jq -r '.paths | keys[]' | sort) \
  <(jq -r '.paths | keys[]' tools/specs/langsmith-openapi.json | sort)
```

### 4. Document Upstream Changes

Edit `tools/specs/CHANGELOG.md`:

```markdown
## LangSmith API

### YYYY-MM-DD (Checksum: [new checksum])

**Added:**
- New endpoint: `POST /api/v1/new-feature`
  - Description of what it does
  - Parameters: ...

**Changed:**
- `GET /api/v1/existing`: Added optional parameter `new_param`
  - Impact: Existing calls still work

**Breaking Changes:**
- `POST /api/v1/thing`: Parameter `old_param` renamed to `new_param`
  - Migration: Update all callers to use `new_param`

**Deprecated:**
- `GET /api/v1/old-endpoint`: Use `/api/v1/new-endpoint` instead

**Notes:**
- Spec fetched from: [URL]
- Previous checksum: [old checksum]
```

### 5. Regenerate SDK

```bash
./tools/generate_sdk.sh
```

This will:
- Update `versions.json` with new checksums and metadata
- Regenerate SDK code in `sdk/src/generated/`
- Show summary of changes

### 6. Review Generated Code

```bash
git diff sdk/src/generated/
```

**Check for:**
- New API methods
- Changed method signatures
- New models or changed model fields
- Deprecated methods

### 7. Update Manual SDK Wrappers

If breaking changes detected, update manual wrappers in `sdk/src/`:

```bash
# Example: Update prompts.rs if Prompt model changed
vim sdk/src/prompts.rs

# Run tests frequently
cargo test
```

**Common updates needed:**
- Add new methods to wrapper clients
- Update existing methods if signatures changed
- Add migration helpers for breaking changes
- Update documentation and examples

### 8. Update SDK Changelog

Edit `sdk/CHANGELOG.md`:

```markdown
## [Unreleased]

### Changed
- ⬆️ Updated generated SDK from langsmith-openapi.json (checksum: abc123...)
  - New endpoint: `DeploymentClient::get_status()`
  - Breaking: `Prompt.repo_handle` now required
  - See tools/specs/CHANGELOG.md for upstream changes
  - Migration: Update all `Prompt` constructors to include `repo_handle`

### Added
- ✨ New `DeploymentClient::get_status()` method
  - Wrapper for new `/deployments/{id}/status` endpoint
```

### 9. Update CLI (If Needed)

If new functionality exposed to users:

```bash
# Add new CLI commands
vim cli/src/commands/deployments.rs

# Test CLI
cargo run -- deployments status test-id
```

### 10. Update CLI Changelog

If CLI changes made, update `CHANGELOG.md` (or let git-cliff generate it):

```markdown
## [Unreleased]

### Added
- ✨ feat(cli): new `langstar deployments status` command
  - Shows real-time deployment health
  - Uses new upstream API endpoint
```

### 11. Run Tests

```bash
# Unit tests
cargo test

# Integration tests (if you have API access)
LANGCHAIN_API_KEY=your-key cargo test --test integration

# Clippy
cargo clippy -- -D warnings

# Format
cargo fmt --check
```

### 12. Commit Changes

```bash
git add tools/specs/ sdk/ cli/ CHANGELOG.md
git commit -m "⬆️ upgrade(sdk): update OpenAPI specs from upstream

- LangSmith spec updated (checksum: abc123...)
- LangGraph spec updated (checksum: def456...)
- Added new DeploymentClient::get_status() method
- Breaking: Prompt.repo_handle now required

See tools/specs/CHANGELOG.md for detailed upstream changes."
```

### 13. Create Pull Request

```bash
# Push branch
git push origin your-branch

# Create PR on GitHub
# Link to related issues
# Add checklist from PR template
```

## Checklist

Use this checklist when updating specs:

- [ ] Checked for drift with `./tools/check_spec_drift.sh`
- [ ] Fetched latest specs with `./tools/fetch_specs.sh`
- [ ] Reviewed spec changes with `git diff`
- [ ] Documented upstream changes in `tools/specs/CHANGELOG.md`
- [ ] Regenerated SDK with `./tools/generate_sdk.sh`
- [ ] Reviewed generated code changes
- [ ] Updated manual SDK wrappers if needed
- [ ] Updated `sdk/CHANGELOG.md`
- [ ] Updated CLI commands if needed
- [ ] Updated `CHANGELOG.md` if CLI changed
- [ ] Ran all tests (`cargo test`)
- [ ] Ran clippy (`cargo clippy`)
- [ ] Ran formatter (`cargo fmt`)
- [ ] Committed changes with descriptive message
- [ ] Created PR with checklist

## Troubleshooting

### Fetch fails

**Problem:** `./tools/fetch_specs.sh` fails to fetch specs

**Solutions:**
- Check internet connection
- Try fetching manually: `curl https://api.smith.langchain.com/openapi.json`
- Check if LangChain services are down
- Try again later

### Generation fails

**Problem:** `./tools/generate_sdk.sh` fails

**Solutions:**
- Check if OpenAPI generator is installed
- Try with Docker: `docker pull openapitools/openapi-generator-cli`
- Check spec validity: Upload to https://editor.swagger.io/
- Review error messages

### Tests fail after update

**Problem:** Tests fail after regenerating SDK

**Solutions:**
- Check for breaking changes in upstream API
- Update test fixtures and mocks
- Update manual wrapper code
- Review migration guide in `tools/specs/CHANGELOG.md`

### Merge conflicts

**Problem:** Merge conflicts in spec files or generated code

**Solutions:**
- **Spec files:** Accept upstream version, re-fetch if needed
- **Generated code:** Delete and regenerate
- **versions.json:** Manually merge, preserve all metadata
- **Changelogs:** Manually merge, preserve all entries

## Schedule

**Monthly Check:** 1st of each month
- Assign to rotating maintainer
- Run drift check
- Update if needed
- Create PR for review

**Quarterly Review:** Every 3 months
- Review all changelogs
- Update documentation
- Plan major SDK refactorings if needed
```

## Consequences

### Positive

1. **Clear Process**: Developers know exactly how to check for and handle drift
2. **Tooling Support**: Scripts automate tedious parts (fetching, checksum calculation)
3. **Actionable Guidance**: Runbook provides step-by-step instructions
4. **Quality Assurance**: Checklist ensures nothing is forgotten
5. **Historical Record**: Changelogs track all changes over time
6. **Onboarding**: New contributors can follow runbook independently
7. **Flexibility**: Manual process allows for human judgment on breaking changes
8. **Low Risk**: Each step is tested before committing

### Negative

1. **Manual Effort**: Requires developer time and attention
2. **Not Proactive**: Only detects drift when explicitly checked
3. **Risk of Neglect**: May be forgotten without scheduled reminders
4. **No Notifications**: Won't know about upstream changes until we check
5. **Documentation Burden**: Runbook must be kept up-to-date

### Mitigation Strategies

1. **Scheduled Checks**: Monthly calendar reminder to check for drift
2. **PR Template**: Reminds developers to check for drift when working on SDK
3. **CI Check**: GitHub Action to run drift check weekly (Phase 3)
4. **Clear Ownership**: Assign rotation of developers to monthly check
5. **Documentation Updates**: Review runbook quarterly

## Alternatives Considered

### Alternative 1: Automated Polling

**Description:** CI/CD job runs daily to check for drift and creates PR automatically.

**Pros:**
- Fully automated
- Immediate notification of changes
- No manual work needed
- Never miss an update

**Cons:**
- Requires CI/CD infrastructure (Phase 3 work)
- May create noise with frequent PRs
- Still need manual review of changes
- Upstream may change multiple times per day

**Deferred to Phase 3:** This is the automation goal, but Phase 2 focuses on manual workflow.

### Alternative 2: Webhook Notifications

**Description:** LangChain notifies us when API changes (if they provided webhooks).

**Pros:**
- Real-time notifications
- No polling needed
- Immediate awareness

**Cons:**
- LangChain doesn't provide webhooks
- Depends on upstream infrastructure
- May not be reliable

**Rejected Because:** Not available from upstream.

### Alternative 3: Manual Visual Inspection

**Description:** Developers manually download and review specs without tools.

**Pros:**
- Maximum control
- Deep understanding of changes
- No tools needed

**Cons:**
- Very time-consuming
- Error-prone
- Not scalable
- Easy to miss small changes

**Rejected Because:** Tooling significantly improves efficiency and accuracy.

### Alternative 4: Semantic Diff Tool

**Description:** Use OpenAPI diff tool to show semantic changes, not just text diff.

**Example:** `oasdiff` (https://github.com/Tufin/oasdiff)

**Pros:**
- Shows breaking vs. non-breaking changes
- Better than text diff
- Machine-readable output
- Can detect subtle changes

**Cons:**
- Additional tool to install
- Learning curve
- May not detect all changes
- Still need manual review

**Future Consideration:** Could add in Phase 3 for better diff analysis.

## References

- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)
- [oasdiff - OpenAPI Diff Tool](https://github.com/Tufin/oasdiff)
- [Swagger Editor](https://editor.swagger.io/)
- [jq Manual](https://stedolan.github.io/jq/manual/)
- [Issue #106: SDK Generation Strategy](https://github.com/codekiln/langstar/issues/106)
- [Issue #115: Phase 1 Research & Design](https://github.com/codekiln/langstar/issues/115)

## Related ADRs

- [ADR-0001: SDK Architecture Approach](./0001-sdk-architecture-approach.md)
- [ADR-0002: OpenAPI Spec Versioning](./0002-openapi-spec-versioning.md)
- [ADR-0003: Changelog Integration Structure](./0003-changelog-integration-structure.md)

## Phase 2 Implementation Tasks

1. Create `tools/check_spec_drift.sh` script
2. Create `tools/fetch_specs.sh` script
3. Create `docs/dev/runbooks/update-openapi-specs.md` runbook
4. Update `tools/generate_sdk.sh` to update `versions.json`
5. Test scripts with real upstream specs
6. Add CI check for uncommitted spec changes
7. Add monthly calendar reminder for spec check
8. Document workflow in team README
