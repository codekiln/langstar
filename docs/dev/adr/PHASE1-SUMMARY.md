# SDK Generation Strategy - Phase 1 Summary

**Status:** ✅ Complete  
**Date:** 2025-11-13  
**Issue:** #115 (Phase 1 of #106)

## Overview

This document summarizes the architectural decisions made in Phase 1 for Langstar's SDK generation strategy. Phase 1 focused on research and design, establishing the foundation for implementation in Phase 2.

## Key Decisions

### 1. SDK Architecture: Layered Manual-Over-Generated Approach

**Decision:** Use a three-layer architecture combining generated and manual code.

```
CLI Layer (User-Facing)
    ↓ calls
Manual SDK Layer (Ergonomic Rust)
    ↓ calls
Generated SDK Layer (OpenAPI Generated)
```

**Rationale:**
- **Generated layer** ensures 100% API coverage automatically
- **Manual layer** provides idiomatic Rust experience for common operations
- **CLI layer** offers excellent user experience
- Balance between maintainability and developer experience

**See:** [ADR-0001: SDK Architecture Approach](./adr/0001-sdk-architecture-approach.md)

### 2. Version Tracking: JSON Manifest with Checksums

**Decision:** Track OpenAPI spec versions using `tools/specs/versions.json` manifest file.

**Key Features:**
- SHA-256 checksums for drift detection
- Metadata: fetch date, source URL, spec version, generation details
- Links to git commits for traceability
- Human-readable JSON format

**Example:**
```json
{
  "format_version": "1.0",
  "specs": {
    "langsmith": {
      "spec_file": "langsmith-openapi.json",
      "spec_url": "https://api.smith.langchain.com/openapi.json",
      "spec_checksum": {"algorithm": "sha256", "value": "abc123..."},
      "fetched_at": "2025-11-13T12:00:00Z",
      "sdk_generated": true,
      "git_commit": "abc123d"
    }
  }
}
```

**Rationale:**
- Enables drift detection by comparing checksums
- Provides full audit trail of spec updates
- Supports both manual (Phase 2) and automated (Phase 3) workflows

**See:** [ADR-0002: OpenAPI Spec Versioning](./adr/0002-openapi-spec-versioning.md)

### 3. Changelog Structure: Hierarchical Organization

**Decision:** Three-level changelog hierarchy for different audiences.

```
CHANGELOG.md                   # CLI changes (end users)
  references ↓
sdk/CHANGELOG.md              # SDK changes (developers)
  references ↓
tools/specs/CHANGELOG.md      # API changes (maintainers)
```

**Rationale:**
- Separates concerns for different audiences
- End users see CLI changes without API details
- Maintainers can trace changes from API → SDK → CLI
- Supports standard tools like `git-cliff`

**See:** [ADR-0003: Changelog Integration Structure](./adr/0003-changelog-integration-structure.md)

### 4. Drift Detection: Manual Workflow with Tools

**Decision:** Manual drift detection workflow with supporting scripts.

**Tools:**
- `tools/check_spec_drift.sh` - Detect when upstream APIs changed
- `tools/fetch_specs.sh` - Fetch latest specs
- `docs/dev/runbooks/update-openapi-specs.md` - Comprehensive runbook

**Workflow:**
1. Check for drift (`./tools/check_spec_drift.sh`)
2. Fetch latest specs (`./tools/fetch_specs.sh`)
3. Review changes (`git diff tools/specs/`)
4. Document changes in `tools/specs/CHANGELOG.md`
5. Regenerate SDK (`./tools/generate_sdk.sh`)
6. Update manual wrappers if needed
7. Test and commit

**Rationale:**
- Phase 2 focuses on manual workflow (simpler to implement)
- Phase 3 will automate (CI/CD, scheduled checks)
- Manual review ensures breaking changes are handled carefully

**See:** [ADR-0004: Drift Detection Workflow](./adr/0004-drift-detection-workflow.md)

## Architecture Diagram

### Complete System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    End User (CLI User)                          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                     CLI Layer (cli/src/)                        │
│  • Commands: langstar prompts list, deployments create, etc.   │
│  • Output formatting: tables, JSON, colors                      │
│  • Argument parsing and validation                              │
└────────────────────────────┬────────────────────────────────────┘
                             │ uses
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              Manual SDK Layer (sdk/src/*.rs)                    │
│  • PromptClient, AssistantClient, DeploymentClient             │
│  • Builder patterns, smart defaults                             │
│  • Error handling, authentication                               │
│  • Human-written documentation                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │ calls
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│       Generated SDK Layer (sdk/src/generated/)                  │
│  • langsmith-client (OpenAPI generated)                         │
│  • langgraph-client (OpenAPI generated)                         │
│  • 100% API coverage, auto-updated                              │
│  • Direct mapping to HTTP endpoints                             │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTP calls
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              Upstream LangChain APIs                            │
│  • LangSmith API (api.smith.langchain.com)                      │
│  • LangGraph Cloud API (api.langgraph.cloud)                    │
└─────────────────────────────────────────────────────────────────┘
```

### Version Tracking and Update Flow

```
┌──────────────────┐
│ Upstream API     │ Changes detected via checksum comparison
│ OpenAPI Spec     │
└────────┬─────────┘
         │ fetch
         ▼
┌──────────────────┐
│ tools/specs/     │ SHA-256 checksum stored in versions.json
│ *.json           │
└────────┬─────────┘
         │ generate
         ▼
┌──────────────────┐
│ Generated SDK    │ Auto-generated Rust code
│ Code             │
└────────┬─────────┘
         │ wrap
         ▼
┌──────────────────┐
│ Manual SDK       │ Ergonomic Rust wrappers
│ Wrappers         │
└────────┬─────────┘
         │ use
         ▼
┌──────────────────┐
│ CLI Commands     │ User-facing interface
└──────────────────┘

Each layer has its own CHANGELOG tracking changes
```

## Alternatives Considered

### SDK Architecture
- ❌ **Pure Generated SDK**: Non-idiomatic, poor DX
- ❌ **Pure Manual SDK**: Doesn't scale, high maintenance
- ❌ **Hybrid (Cherry-Pick)**: No clear guidelines, inconsistent
- ✅ **Layered Manual-Over-Generated**: Best of both worlds

### Version Tracking
- ❌ **Embedded in Code**: Hard to query, no central view
- ❌ **Separate Files Per Spec**: More files to manage
- ❌ **Git Tags**: Lacks metadata
- ❌ **Database**: Overkill, not VCS-friendly
- ✅ **JSON Manifest**: Simple, human-readable, VCS-friendly

### Changelog Structure
- ❌ **Single Unified**: Mixes concerns, cluttered
- ❌ **No Upstream Changelog**: Loses traceability
- ❌ **Auto-Only**: Lacks curation and context
- ❌ **Git Commits Only**: Not standard, hard to browse
- ✅ **Hierarchical**: Clear separation, appropriate detail per audience

### Drift Detection
- 🔮 **Automated Polling** (Phase 3): Future automation goal
- ❌ **Webhook Notifications**: Not available from upstream
- ❌ **Manual Visual Inspection**: Too time-consuming
- 🤔 **Semantic Diff Tool**: Future enhancement consideration
- ✅ **Manual + Scripts**: Practical for Phase 2

## Benefits

### For End Users
- ✅ Ergonomic CLI commands with sensible defaults
- ✅ Comprehensive coverage of all LangChain APIs
- ✅ Fast updates when new features are released
- ✅ Clear changelog of user-visible changes

### For SDK Consumers (if published separately)
- ✅ Idiomatic Rust APIs following best practices
- ✅ Type-safe client with compile-time guarantees
- ✅ Rich documentation and examples
- ✅ Fallback to low-level generated API when needed

### For Maintainers
- ✅ Automated SDK generation reduces manual work
- ✅ Clear process for handling upstream changes
- ✅ Full traceability from API → SDK → CLI
- ✅ Systematic drift detection
- ✅ Historical record of all changes

## Implementation Roadmap

### Phase 1: Research & Design ✅ COMPLETE
- [x] Research SDK architecture options
- [x] Design version tracking system
- [x] Design changelog structure
- [x] Design drift detection workflow
- [x] Document all decisions in ADRs

### Phase 2: Implementation (Issue #116) - NEXT
**Goal:** Working SDK generation with version tracking

**Tasks:**
1. Create `tools/specs/versions.json` with initial structure
2. Update `tools/generate_sdk.sh` to read/write `versions.json`
3. Create `tools/check_spec_drift.sh` script
4. Create `tools/fetch_specs.sh` script
5. Create `docs/dev/runbooks/update-openapi-specs.md` runbook
6. Generate initial SDK from OpenAPI specs
7. Refactor existing manual SDK to wrap generated SDK
8. Create `sdk/CHANGELOG.md` and `tools/specs/CHANGELOG.md`
9. Test thoroughly to ensure no regressions
10. Document developer workflow

**Acceptance Criteria:**
- [ ] SDK generation working with both LangSmith and LangGraph specs
- [ ] Version tracking operational
- [ ] Drift detection scripts functional
- [ ] Manual SDK successfully wraps generated SDK
- [ ] All existing tests passing
- [ ] No breaking changes in CLI interface
- [ ] Changelogs established and documented

### Phase 3: Automation (Issue #117) - FUTURE
**Goal:** Automated drift detection and updates

**Tasks (deferred, low priority):**
1. CI/CD workflow to fetch latest specs weekly
2. Automated drift detection (compare checksums)
3. Automated PR creation for spec updates
4. Notification system for breaking changes
5. Scheduled spec updates (weekly/monthly)
6. Integration with semantic diff tools

**Acceptance Criteria:**
- [ ] Weekly CI job checks for drift
- [ ] Automated PRs created when drift detected
- [ ] Breaking changes flagged automatically
- [ ] Maintainers notified of updates
- [ ] No manual intervention needed for routine updates

## Files Created

This Phase 1 work created the following documentation:

```
docs/dev/adr/
├── README.md                              # ADR overview and index
├── 0001-sdk-architecture-approach.md     # Layered architecture decision
├── 0002-openapi-spec-versioning.md       # Version tracking design
├── 0003-changelog-integration-structure.md # Changelog hierarchy
└── 0004-drift-detection-workflow.md      # Drift detection process
```

**Total Documentation:** ~62,000 characters across 5 files

## References

### Related Issues
- [#106](https://github.com/codekiln/langstar/issues/106) - Parent: SDK Generation Strategy
- [#115](https://github.com/codekiln/langstar/issues/115) - This Phase: Research & Design
- [#116](https://github.com/codekiln/langstar/issues/116) - Next: Implementation
- [#117](https://github.com/codekiln/langstar/issues/117) - Future: Automation

### External Resources
- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)
- [OpenAPI Generator - Rust](https://openapi-generator.tech/docs/generators/rust/)
- [Progenitor](https://github.com/oxidecomputer/progenitor) - Alternative Rust OpenAPI generator
- [Keep a Changelog](https://keepachangelog.com/)
- [Semantic Versioning](https://semver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [git-cliff](https://git-cliff.org/) - Changelog generator

### Existing Langstar Documentation
- [Architecture Documentation](../architecture.md)
- [GitHub Workflow](./github-workflow.md)
- [Git Commit Conventions](./git-scm-conventions.md)
- [Code Style Principles](./code-style-principles.md)

## Team Notes

### Key Takeaways

1. **Balanced Approach**: The layered architecture provides both automation and ergonomics
2. **Prepared for Growth**: System designed to scale as LangChain APIs evolve
3. **Low Risk**: Phase 2 implementation can be done incrementally
4. **Clear Path**: Detailed implementation tasks defined for Phase 2
5. **Future-Proof**: Architecture supports automation in Phase 3

### Questions for Phase 2

Before starting Phase 2 implementation, consider:

1. **Generator Choice**: Use `openapi-generator-cli` or explore `progenitor`?
2. **Workspace Structure**: Keep generated clients as separate workspace members?
3. **Testing Strategy**: How to test generated SDK? Mock server? Integration tests?
4. **Documentation**: Auto-generate docs for generated layer? Manual for wrappers?
5. **Versioning**: Separate versions for generated vs. manual SDK? Or unified?

### Success Criteria for Phase 1

- [x] ✅ Comprehensive research completed
- [x] ✅ All major decisions documented in ADRs
- [x] ✅ Alternatives evaluated and trade-offs understood
- [x] ✅ Clear implementation path defined for Phase 2
- [x] ✅ No blocking unknowns remaining
- [x] ✅ Team aligned on approach

**Phase 1 is COMPLETE and ready for Phase 2 implementation.**

---

*Last Updated: 2025-11-13*  
*Prepared by: Langstar Development Team*  
*Status: Approved and Ready for Implementation*
