# ADR-0003: Changelog Integration Structure

**Status:** Accepted  
**Date:** 2025-11-13  
**Decision Makers:** Langstar Development Team  
**Related Issues:** #106, #115

## Context

Langstar operates at three distinct levels, each with its own changes and updates:

1. **Upstream API Changes** (LangSmith/LangGraph APIs)
   - New endpoints, parameters, models
   - Breaking changes, deprecations
   - Bug fixes in upstream services

2. **SDK Changes** (Langstar SDK)
   - Generated code updates
   - Manual wrapper modifications
   - API design improvements

3. **CLI Changes** (User-Facing Tool)
   - New commands and options
   - Output format changes
   - User experience improvements

We need a changelog structure that:
- Tracks changes at each level independently
- Shows relationships between changes (e.g., CLI change driven by upstream API change)
- Provides appropriate detail for different audiences (API developers vs. end users)
- Supports standard Rust changelog tools (e.g., `git-cliff`)

### Current State

- Single `CHANGELOG.md` at root tracks CLI changes only
- Uses [Conventional Commits](https://www.conventionalcommits.org/) with emojis
- Managed by `git-cliff` (see `cliff.toml`)
- No tracking of upstream API changes
- No SDK-level changelog

### Audiences

1. **End Users**: Need to know about CLI changes (new commands, breaking changes)
2. **SDK Consumers**: Need to know about SDK API changes (if SDK is published separately)
3. **Maintainers**: Need to track upstream API changes to understand drift and updates

## Decision

We implement a **hierarchical changelog structure** with three levels, each in its own file:

```
CHANGELOG.md                      # User-facing CLI changes (primary)
  references ↓
sdk/CHANGELOG.md                  # SDK API changes
  references ↓
tools/specs/CHANGELOG.md          # Upstream API changes tracked
```

### File Structure

```
langstar/
├── CHANGELOG.md                  # CLI changelog (user-facing)
├── cliff.toml                    # git-cliff config for CLI changelog
├── sdk/
│   ├── CHANGELOG.md              # SDK API changelog
│   └── cliff.toml                # git-cliff config for SDK (optional)
└── tools/
    └── specs/
        ├── CHANGELOG.md          # Upstream API changes
        └── versions.json         # Version tracking (see ADR-0002)
```

## Changelog Specifications

### 1. CLI Changelog (CHANGELOG.md)

**Location:** `/CHANGELOG.md`  
**Audience:** End users (CLI users)  
**Purpose:** Document user-visible changes to CLI commands and behavior

**Format:** Keep a Changelog 1.0.0 + Conventional Commits with Emoji

**Example:**
```markdown
# Changelog

All notable changes to Langstar CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- ✨ feat(cli): new `langstar deployments status` command (#123)
  - Shows real-time deployment health
  - Supports JSON output with `--json`
  - *SDK Change:* Depends on new `DeploymentClient::get_status()` (see sdk/CHANGELOG.md)
  - *Upstream:* New `/deployments/{id}/status` endpoint

### Changed
- 🔧 chore(cli): update to SDK v0.4.0 (#125)
  - See sdk/CHANGELOG.md for API changes
  - No breaking changes in CLI interface

### Fixed
- 🐛 fix(cli): handle connection timeouts gracefully (#124)

## [0.3.0] - 2025-11-10

### Added
- ✨ feat(cli): support for LangGraph deployments (#110)
  - New `langstar deployments` command group
  - List, create, delete deployments

[... rest of changelog ...]
```

**Generation:** Automated via `git-cliff` from conventional commits

**Cross-References:**
- Link to SDK changes when CLI change depends on SDK update
- Link to upstream API changes when relevant
- Reference GitHub issues and PRs

### 2. SDK Changelog (sdk/CHANGELOG.md)

**Location:** `/sdk/CHANGELOG.md`  
**Audience:** SDK consumers (if published), Langstar maintainers  
**Purpose:** Document SDK API changes (both generated and manual code)

**Format:** Keep a Changelog 1.0.0

**Example:**
```markdown
# Langstar SDK Changelog

All notable changes to the Langstar SDK will be documented in this file.

## [Unreleased]

### Added
- ✨ New `DeploymentClient::get_status()` method (#123)
  - Returns real-time deployment health status
  - *Upstream:* New `/deployments/{id}/status` endpoint (see tools/specs/CHANGELOG.md)
  - *Generated:* Added to generated client v2025.11.13
  - *Manual:* Added ergonomic wrapper with retry logic

### Changed
- ⬆️ Updated generated SDK from langsmith-openapi.json (checksum: abc123...def456)
  - Breaking: `Prompt.repo_handle` now required (was optional)
  - See tools/specs/CHANGELOG.md for upstream API changes
  - Migration guide: Update all `Prompt` constructors to include `repo_handle`

### Fixed
- 🐛 fix(sdk): correct error handling in `PromptClient::commit()` (#122)

### Breaking Changes
- ⚠️ `Prompt.repo_handle` is now required (API contract change)

## [0.3.0] - 2025-11-10

### Added
- ✨ LangGraph deployment support
  - New `DeploymentClient` with CRUD operations
  - Generated from langgraph-openapi.json (2025-11-10)

[... rest of changelog ...]
```

**Generation:** 
- Manual or semi-automated (git-cliff with SDK-specific config)
- Generated code changes noted with checksum references
- Links to `versions.json` for traceability

**Cross-References:**
- Link to upstream API changes (tools/specs/CHANGELOG.md)
- Note which spec version generated the code
- Reference commit hashes from versions.json

### 3. Upstream API Changelog (tools/specs/CHANGELOG.md)

**Location:** `/tools/specs/CHANGELOG.md`  
**Audience:** Langstar maintainers  
**Purpose:** Track upstream LangChain API changes

**Format:** Chronological log of API updates

**Example:**
```markdown
# Upstream API Changes

This file tracks changes to upstream LangChain APIs (LangSmith, LangGraph Cloud).
Changes are detected when we update OpenAPI specifications.

## LangSmith API

### 2025-11-13 (Checksum: 999xyz...000abc)

**Added:**
- New endpoint: `GET /api/v1/deployments/{id}/status`
  - Returns deployment health metrics
  - Parameters: `id` (required), `include_metrics` (optional)
  - Response: `DeploymentStatus` model

**Changed:**
- `Prompt` model: `repo_handle` field now required (was optional)
  - **Breaking Change:** All prompt operations must provide `repo_handle`
  - Migration: Ensure all client code provides this field

**Deprecated:**
- `GET /api/v1/repos/list` endpoint
  - Use `GET /api/v1/repos/` instead
  - Will be removed in 2026-01-01

**Fixed:**
- `Assistant` model: Corrected type of `metadata` field (was string, now object)

**Notes:**
- Spec fetched from: https://api.smith.langchain.com/openapi.json
- Previous checksum: abc123...def456
- Spec file: langsmith-openapi.json
- See versions.json for full metadata

---

### 2025-11-10 (Checksum: abc123...def456)

**Added:**
- Initial OpenAPI spec committed
- 45 endpoints documented
- 23 models defined

**Notes:**
- First spec version in version control
- Baseline for future comparisons

---

## LangGraph Cloud API

### 2025-11-13 (Checksum: 777abc...999def)

**Added:**
- New endpoint: `POST /deployments/{id}/secrets`
  - Manage deployment secrets
  - Parameters: `id` (required), `secrets` (required)

**Notes:**
- Spec fetched from: https://api.langgraph.cloud/openapi.json
- Previous checksum: 555jkl...888mno
- Spec file: langgraph-openapi.json

---

### 2025-11-10 (Checksum: 555jkl...888mno)

**Added:**
- Initial OpenAPI spec committed
- 12 endpoints documented
- 8 models defined

**Notes:**
- First spec version in version control
- Baseline for future comparisons
```

**Generation:** Manual updates when specs are refreshed

**Structure:**
- Organized by service (LangSmith, LangGraph)
- Chronological within each service
- Each entry includes checksum for traceability
- Links to `versions.json` for full metadata

**Content:**
- Detected changes when comparing specs
- Breaking vs. non-breaking changes
- Migration guidance for breaking changes
- References to spec files and checksums

## Workflow Integration

### Developer Workflow: Updating Upstream Specs

```bash
# 1. Fetch latest specs
./tools/fetch_specs.sh

# 2. Compare specs (manual review)
git diff tools/specs/langsmith-openapi.json
git diff tools/specs/langgraph-openapi.json

# 3. Document changes in upstream changelog
# Edit tools/specs/CHANGELOG.md
# Add entry with checksum, date, and detected changes

# 4. Regenerate SDK
./tools/generate_sdk.sh

# 5. Review generated code changes
git diff sdk/src/generated/

# 6. Update SDK changelog if needed
# Edit sdk/CHANGELOG.md
# Document any breaking changes or important additions

# 7. Update manual SDK wrappers if needed
# Modify sdk/src/*.rs files

# 8. Test
cargo test

# 9. Update CLI if needed
# Modify cli/src/ files

# 10. Update CLI changelog
# Edit CHANGELOG.md (or let git-cliff generate it)

# 11. Commit all changes
git add tools/specs/ sdk/ cli/ CHANGELOG.md
git commit -m "⬆️ upgrade(sdk): update OpenAPI specs and regenerate SDK"
```

### Automated Changelog Generation

**CLI Changelog (CHANGELOG.md):**
- Automatically generated by `git-cliff`
- Based on conventional commits
- Configuration in `cliff.toml`
- Run on release: `git cliff --tag v0.4.0 > CHANGELOG.md`

**SDK Changelog (sdk/CHANGELOG.md):**
- Can be semi-automated with custom git-cliff config
- Or maintained manually for better curation
- Should reference spec versions and checksums

**Upstream API Changelog (tools/specs/CHANGELOG.md):**
- Manually maintained (Phase 2)
- Could be partially automated in Phase 3 using spec diff tools

## Cross-Referencing Strategy

### From CLI Changelog to SDK Changelog

```markdown
### Added
- ✨ feat(cli): new `langstar deployments status` command (#123)
  - *SDK Change:* Uses `DeploymentClient::get_status()` (see sdk/CHANGELOG.md #0.4.0)
```

### From SDK Changelog to Upstream Changelog

```markdown
### Changed
- ⬆️ Updated generated SDK from langsmith-openapi.json (checksum: abc123...def456)
  - *Upstream:* New `/deployments/{id}/status` endpoint
  - See tools/specs/CHANGELOG.md (2025-11-13 entry)
  - See versions.json for full metadata
```

### From Upstream Changelog to versions.json

```markdown
### 2025-11-13 (Checksum: 999xyz...000abc)

**Notes:**
- Spec file: langsmith-openapi.json
- See versions.json for full metadata (fetched_at, git_commit, etc.)
```

## Consequences

### Positive

1. **Clear Separation**: Each level has its own changelog appropriate to its audience
2. **Traceability**: Can trace changes from upstream API → SDK → CLI
3. **Audience-Specific**: End users see CLI changes, maintainers see full context
4. **Standard Format**: Uses Keep a Changelog and Conventional Commits
5. **Tool Support**: Compatible with git-cliff and other changelog generators
6. **Historical Record**: Captures upstream API evolution over time
7. **Migration Guidance**: Breaking changes documented with migration paths
8. **Automation Ready**: Structure supports automated generation (Phase 3)

### Negative

1. **Maintenance Overhead**: Three changelogs to maintain instead of one
2. **Manual Work**: Upstream changelog requires manual review of spec diffs
3. **Sync Burden**: Must keep changelogs in sync with actual changes
4. **Documentation Debt**: Risk of outdated changelogs if not maintained
5. **Complexity**: Developers must understand which changelog to update

### Mitigation Strategies

1. **Clear Guidelines**: Document which changelog to update for which changes
2. **Templates**: Provide changelog entry templates for common scenarios
3. **PR Reminders**: PR template reminds developers to update relevant changelogs
4. **Automated Checks**: CI check that CHANGELOG.md is updated in PRs
5. **Tooling**: Scripts to help generate changelog entries (Phase 3)
6. **Ownership**: Clear ownership of each changelog (e.g., upstream changelog by maintainers only)

## Alternatives Considered

### Alternative 1: Single Unified Changelog

**Description:** One CHANGELOG.md with sections for API, SDK, and CLI changes.

**Pros:**
- Single file to maintain
- Easier to see all changes at once
- No need to cross-reference files

**Cons:**
- Mixes audience concerns (users see API details they don't need)
- Harder to extract relevant changes for specific audience
- Cluttered and verbose
- Not standard practice

**Rejected Because:** Different audiences need different levels of detail. End users don't care about upstream API changes.

### Alternative 2: No Upstream Changelog

**Description:** Track only SDK and CLI changes, ignore upstream API changes.

**Pros:**
- Less maintenance work
- Simpler structure
- Focuses on user-facing changes

**Cons:**
- Lose historical record of upstream changes
- Harder to debug drift issues
- Can't understand why SDK changed
- No traceability to upstream

**Rejected Because:** Need historical record for debugging and understanding drift.

### Alternative 3: Automated Changelog Generation Only

**Description:** Use tools like `conventional-changelog` or `git-cliff` for all changelogs.

**Pros:**
- Fully automated
- No manual maintenance
- Always up-to-date
- Consistent format

**Cons:**
- Commit messages must be perfect
- Can't capture upstream API changes (no commits)
- Less curated, more mechanical
- Harder to add context and migration guidance

**Rejected Because:** Need manual curation for upstream changes and important context.

### Alternative 4: Changelog in Git Commits Only

**Description:** No separate changelog files, use `git log` to see changes.

**Pros:**
- No separate files to maintain
- Git log is always accurate
- Single source of truth

**Cons:**
- Not standard practice in Rust ecosystem
- Harder for users to browse changes
- Requires learning git to see changelog
- Can't reference from documentation
- No curation or grouping

**Rejected Because:** Standard practice is to maintain CHANGELOG.md for user convenience.

### Alternative 5: Changelog in Documentation Site

**Description:** Host changelog on separate docs site (e.g., with mdBook).

**Pros:**
- Better formatting and navigation
- Can include more context and examples
- Searchable and linkable

**Cons:**
- Separate infrastructure to maintain
- Not in version control (or requires separate docs repo)
- Harder to keep in sync
- Overkill for current project size

**Rejected Because:** File-based changelogs in repo are simpler and sufficient.

## References

- [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [git-cliff](https://git-cliff.org/)
- [Rust Release Notes](https://github.com/rust-lang/rust/blob/master/RELEASES.md) (example)
- [Issue #106: SDK Generation Strategy](https://github.com/codekiln/langstar/issues/106)
- [Issue #115: Phase 1 Research & Design](https://github.com/codekiln/langstar/issues/115)

## Implementation Notes

See [ADR-0002: OpenAPI Spec Versioning](./0002-openapi-spec-versioning.md) for how spec versions link to changelog entries.

See [ADR-0004: Drift Detection Workflow](./0004-drift-detection-workflow.md) for the full developer workflow including changelog updates.

## Phase 2 Implementation Tasks

1. Create `sdk/CHANGELOG.md` with initial structure
2. Create `tools/specs/CHANGELOG.md` with initial structure
3. Add SDK changelog entry for first generated SDK
4. Add upstream changelog entry for first fetched specs
5. Update PR template to remind about changelog updates
6. Document changelog guidelines in contribution docs
7. Update `cliff.toml` if needed for better CLI changelog generation
