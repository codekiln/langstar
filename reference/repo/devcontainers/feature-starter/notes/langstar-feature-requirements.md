# Langstar DevContainer Feature - Implementation Requirements

Date: 2025-11-21
Purpose: Document requirements for creating a devcontainer feature to install the langstar CLI

## Overview

A devcontainer feature is a self-contained, shareable unit of installation code that can be added to any devcontainer configuration. This document outlines what's needed to publish a `langstar` feature from the langstar repository.

## Research Summary

### Feature Structure

Based on analysis of:
- `devcontainers/feature-starter` - Template repository
- `devcontainers/features` - Official features collection
- Particularly the `github-cli` feature as a reference

**Required structure:**
```
src/langstar/
├── devcontainer-feature.json    # Feature metadata
├── install.sh                    # Installation script
├── README.md                     # Auto-generated documentation
└── NOTES.md                      # Optional: Additional notes
```

### Key Components

#### 1. devcontainer-feature.json

**Purpose**: Defines feature metadata, options, and dependencies

**Required fields:**
- `id`: Unique identifier (e.g., "langstar")
- `name`: Human-readable name (e.g., "Langstar CLI")
- `version`: Semantic version (e.g., "1.0.0")
- `description`: Brief description of the feature

**Optional but recommended:**
- `documentationURL`: Link to documentation
- `options`: Configurable parameters
  - `version`: Which version to install (default: "latest")
  - Can include additional options like install location
- `installsAfter`: Dependencies (e.g., `common-utils`, `git`)
- `customizations`: Tool-specific settings (e.g., VS Code)

**Example for langstar:**
```json
{
    "id": "langstar",
    "version": "1.0.0",
    "name": "Langstar CLI",
    "documentationURL": "https://github.com/codekiln/langstar",
    "description": "Installs the Langstar CLI for LangGraph Cloud deployment management. Auto-detects latest version and installs from GitHub releases.",
    "options": {
        "version": {
            "type": "string",
            "proposals": ["latest", "none"],
            "default": "latest",
            "description": "Select version of Langstar CLI, if not latest."
        }
    },
    "installsAfter": [
        "ghcr.io/devcontainers/features/common-utils"
    ]
}
```

#### 2. install.sh

**Purpose**: Installation script executed during devcontainer build

**Key characteristics:**
- Runs as root user
- Receives options as environment variables (uppercased and sanitized)
- Should be idempotent (safe to run multiple times)
- Should handle different architectures (x86_64, ARM64)
- Should detect and use existing installers when available

**Environment variables:**
- `VERSION`: From options.version (uppercased)
- `_REMOTE_USER`: Effective remote user
- `_REMOTE_USER_HOME`: Remote user's home directory
- `_CONTAINER_USER`: Container user
- `_CONTAINER_USER_HOME`: Container user's home directory

**Best practices observed:**
- Error handling with `set -e`
- Architecture detection
- Version validation
- GPG signature verification for security
- Clear error messages
- Cleanup after installation

**For langstar:** Can leverage existing `scripts/install.sh` installer!

#### 3. README.md

**Auto-generated** by the release workflow
- Merges content from NOTES.md (if present)
- Adds metadata and usage examples
- Updated via GitHub Actions on release

### Publishing Workflow

#### GitHub Actions Release Workflow

**File**: `.github/workflows/release-features.yaml`

**Key steps:**
1. Use `devcontainers/action@v1` with:
   - `publish-features: "true"`
   - `base-path-to-features: "./src"`
   - `generate-docs: "true"`

2. Auto-generates documentation
3. Creates PR with updated READMEs
4. Publishes to GitHub Container Registry (GHCR)

**Required permissions:**
```yaml
permissions:
  contents: write
  pull-requests: write
  packages: write
```

**Workflow trigger options:**
- `workflow_dispatch`: Manual trigger (recommended for testing)
- `push: branches: [main]`: Auto-release on main
- `release: types: [published]`: Release-based trigger

#### Publishing to GHCR

Features are published to: `ghcr.io/<owner>/<repo>/<feature-id>:<version>`

**For langstar:** `ghcr.io/codekiln/langstar/langstar:1`

**Steps after first publish:**
1. Navigate to package settings in GHCR
2. Change visibility from `private` to `public`
3. URL format: `https://github.com/users/<owner>/packages/container/<repo>%2F<feature-id>/settings`

#### Adding to Public Index (Optional)

To make the feature discoverable:
1. Go to https://github.com/devcontainers/devcontainers.github.io
2. Open PR to modify `collection-index.yml`
3. Add entry for the langstar feature collection

**Benefits:**
- Appears in VS Code Dev Containers UI
- Discoverable in GitHub Codespaces
- Listed on https://containers.dev/features

### Usage by Consumers

Once published, users can add to their `devcontainer.json`:

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/base:ubuntu",
    "features": {
        "ghcr.io/codekiln/langstar/langstar:1": {
            "version": "latest"
        }
    }
}
```

## Implementation Plan

### Phase 1: Feature Structure Setup

**Goal**: Create the basic feature structure in the langstar repository

**Tasks:**
1. Create `src/langstar/` directory structure
2. Create `devcontainer-feature.json` with appropriate metadata
3. Create `install.sh` that leverages existing `scripts/install.sh`
4. Create optional `NOTES.md` with usage instructions

**Deliverable**: Feature files ready for publishing

### Phase 2: GitHub Actions Workflow

**Goal**: Automate feature publishing to GHCR

**Tasks:**
1. Create `.github/workflows/release-features.yaml`
2. Configure workflow with appropriate permissions
3. Set up workflow dispatch trigger for manual releases
4. Configure base path to `./src`

**Deliverable**: Automated publishing workflow

### Phase 3: Testing and Validation

**Goal**: Verify the feature works correctly

**Tasks:**
1. Manually trigger the release workflow
2. Verify feature is published to GHCR
3. Set package visibility to public
4. Create test devcontainer using the feature
5. Verify langstar CLI installs correctly
6. Test with different architectures (x86_64, ARM64)

**Deliverable**: Validated, working feature

### Phase 4: Documentation and Discovery (Optional)

**Goal**: Make the feature discoverable and well-documented

**Tasks:**
1. Update langstar README to mention the feature
2. Add feature usage examples
3. Submit PR to devcontainers.github.io for public index
4. Document feature in langstar documentation

**Deliverable**: Public, discoverable feature

## Technical Considerations

### Leveraging Existing Installer

**Current langstar installer**: `/workspace/scripts/install.sh`

**Benefits:**
- Already handles architecture detection
- Downloads from GitHub releases
- Verifies checksums
- Handles multiple install locations
- Idempotent design

**Integration approach:**
The feature's `install.sh` can:
1. Download the langstar installer script
2. Execute it with appropriate flags
3. Or: Inline the installer logic directly

**Example approach:**
```bash
#!/bin/bash
set -e

VERSION=${VERSION:-latest}

# Download and execute the official installer
curl -fsSL https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash -s -- --prefix /usr/local
```

### Version Management

**Feature versioning** (in devcontainer-feature.json):
- Independent of langstar CLI version
- Follows semver for the feature itself
- Example: `1.0.0`, `1.1.0`, `2.0.0`

**CLI version option** (in options.version):
- What version of langstar to install
- Default: `"latest"`
- Can specify exact versions: `"0.4.0"`

### Architecture Support

**Must support:**
- Linux x86_64 (most common)
- Linux ARM64 (Apple Silicon, ARM servers)

**Current langstar releases:**
- Check: https://github.com/codekiln/langstar/releases
- Verify both architectures are available

**Install.sh should detect:**
```bash
ARCHITECTURE=$(uname -m)
case $ARCHITECTURE in
    x86_64) ARCH="x86_64-unknown-linux-musl" ;;
    aarch64|arm64) ARCH="aarch64-unknown-linux-musl" ;;
    *) echo "Unsupported architecture: $ARCHITECTURE"; exit 1 ;;
esac
```

### Dependencies

**Recommended installsAfter:**
- `ghcr.io/devcontainers/features/common-utils`
  - Provides common utilities
  - Sets up user environment properly

**Why not more dependencies:**
- Langstar installer is self-contained
- Only needs basic utilities (curl, bash)
- Keep it lightweight

## Repository Structure Impact

**New files to create:**
```
langstar/
├── .devcontainer/
│   ├── devcontainer.json (existing - project's own config)
│   └── features/
│       └── langstar/
│           ├── devcontainer-feature.json
│           ├── install.sh
│           └── NOTES.md (optional)
├── .github/
│   └── workflows/
│       └── release-features.yaml
└── README.md (update with feature documentation)
```

**Note on directory structure:**
Unlike the reference repositories which use `src/` for features, this repository uses `.devcontainer/features/` to:
- Keep feature definitions within the devcontainer ecosystem
- Avoid confusion with the project's own `.devcontainer/devcontainer.json`
- Prevent confusion with source code directories (`sdk/`, `cli/`)

**Changes to existing files:**
- Update root README.md with feature installation instructions
- Add feature documentation to docs/

## Open Questions

1. **Versioning strategy**:
   - Should feature version match CLI version?
   - Recommendation: No - keep independent

2. **Feature vs existing installer**:
   - Should we maintain two installation methods?
   - Recommendation: Yes - feature for devcontainers, installer for general use

3. **Release cadence**:
   - Publish feature with every CLI release?
   - Recommendation: Publish feature updates as needed, not necessarily with every CLI release

4. **Private vs public initially**:
   - Start with public or test private first?
   - Recommendation: Start private, test thoroughly, then make public

## Next Steps

1. Create GitHub epic issue for this project
2. Break down into sub-issues for each phase
3. Begin Phase 1: Feature structure setup
4. Set up CI/CD workflow for testing
5. Publish initial version to GHCR (private)
6. Validate and test thoroughly
7. Make public and submit to devcontainers index

## References

- [Dev Container Features Specification](https://containers.dev/implementors/features/)
- [Features Distribution Spec](https://containers.dev/implementors/features-distribution/)
- [devcontainers/feature-starter](https://github.com/devcontainers/feature-starter)
- [devcontainers/features](https://github.com/devcontainers/features)
- [devcontainers/action](https://github.com/devcontainers/action)
