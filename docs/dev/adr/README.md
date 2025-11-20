# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) documenting key architectural decisions made in the Langstar project.

## What is an ADR?

An Architecture Decision Record (ADR) is a document that captures an important architectural decision made along with its context and consequences.

## ADR Format

Each ADR follows this structure:

```markdown
# ADR-XXXX: Title

**Status:** [Proposed | Accepted | Deprecated | Superseded]
**Date:** YYYY-MM-DD
**Decision Makers:** [Who made this decision]
**Related Issues:** [GitHub issue references]

## Context

What is the issue that we're seeing that is motivating this decision or change?

## Decision

What is the change that we're proposing and/or doing?

## Consequences

What becomes easier or more difficult to do because of this change?

### Positive
- Benefit 1
- Benefit 2

### Negative
- Drawback 1
- Drawback 2

## Alternatives Considered

What other options were considered and why were they not chosen?

## References

Links to related documentation, discussions, or resources.
```

## Current ADRs

### SDK Generation Strategy (Phase 1)

- [ADR-0001: SDK Architecture Approach](./0001-sdk-architecture-approach.md) - Layered SDK architecture with manual-over-generated approach
- [ADR-0002: OpenAPI Spec Versioning](./0002-openapi-spec-versioning.md) - Version tracking system for OpenAPI specifications
- [ADR-0003: Changelog Integration Structure](./0003-changelog-integration-structure.md) - Hierarchical changelog organization
- [ADR-0004: Drift Detection Workflow](./0004-drift-detection-workflow.md) - Manual workflow for detecting API drift

## Related Documentation

- [SDK Generation Strategy (Parent Issue)](https://github.com/codekiln/langstar/issues/106)
- [Phase 1: Research & Design](https://github.com/codekiln/langstar/issues/115)
- [Phase 2: Implementation](https://github.com/codekiln/langstar/issues/116)
- [Phase 3: Automation](https://github.com/codekiln/langstar/issues/117)
