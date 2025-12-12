# Sessions vs Projects Terminology Research - langchain-ai/docs

**Research Date**: 2025-12-05
**Related Issue**: #574
**Related PR**: #575
**Purpose**: Additional triangulation of "sessions" vs "projects" terminology beyond langsmith-sdk

## Executive Summary

The official LangChain documentation consistently uses "project" terminology in user-facing content while revealing that the underlying API uses "tracer_sessions" endpoints. This confirms the terminology split and provides strong evidence for why LangChain chose "projects" as the public-facing term.

## Key Findings

### 1. Official Definition of "Project"

**Source**: `src/langsmith/observability-concepts.mdx:60`

> A _project_ is a collection of traces. You can think of a project as a container for all the traces that are related to a single application or service. You can have multiple projects, and each project can have multiple traces.

**Analysis**:

- Clear, user-friendly definition
- No mention of "sessions" in the public definition
- Emphasizes the organizational/containment aspect
- Positions projects as application-level containers

**Contrast with "Session" Definition**:
The documentation does use "session" terminology, but only for:

- **Thread/Conversation Sessions** (lines 40-43): Multi-turn conversations use `session_id`, `thread_id`, or `conversation_id` as metadata keys
- This is a DIFFERENT concept from the API's `/sessions` endpoint for projects

### 2. API Endpoint Revelation

**Source**: `src/langsmith/observability-concepts.mdx:116-117`

Deleting projects can be done via:

- **API Endpoint**: `delete_tracer_sessions`
- **Python SDK**: `delete_project()`
- **JS/TS SDK**: `deleteProject()`

**Analysis**:
This is the smoking gun that confirms:

1. The REST API internally calls projects "tracer_sessions"
2. Both SDKs deliberately translate this to "project" terminology
3. LangChain made a conscious decision to abstract away "sessions" terminology
4. The abstraction is consistent across multiple language SDKs

### 3. Environment Variable Naming

**Source**: `src/langsmith/log-traces-to-project.mdx:10`

> LangSmith uses the concept of a `Project` to group traces. If left unspecified, the project is set to `default`. You can set the `LANGSMITH_PROJECT` environment variable to configure a custom project name

**Environment Variables**:

- Primary: `LANGSMITH_PROJECT`
- Legacy (JS SDK < 0.2.16): `LANGCHAIN_PROJECT`
- No references to `LANGSMITH_SESSION` or similar

**Analysis**:

- User-facing configuration consistently uses "project" terminology
- Even legacy variables use "project" not "session"
- This dates back to early SDK versions, showing long-term commitment to "project" terminology

### 4. Documentation File Structure

**Files mentioning "project"**: 20+ files including:

- `log-traces-to-project.mdx`
- `observability-concepts.mdx` (core concepts)
- `trace-with-api.mdx`, `trace-with-langchain.mdx`
- `export-traces.mdx`, `data-export.mdx`
- `fetch-perf-metrics-experiment.mdx`

**Files mentioning "session" in project context**: 1 file

- `observability-concepts.mdx:116` - API endpoint reference only

**Analysis**:

- Documentation overwhelmingly prefers "project" terminology
- "Session" only appears when referencing the underlying API
- File names, headings, and user instructions all use "project"

## Semantic Analysis: Why Not "Sessions"?

### What "Session" Typically Means

1. **Stateful Time Period**: A session implies a beginning and end, often tied to user login/logout
2. **Temporary Nature**: Sessions are typically ephemeral, cleared after timeout
3. **Single Interaction Sequence**: Usually represents one continuous period of activity

### What "Project" Better Represents

1. **Persistent Container**: Projects are long-lived organizational units
2. **Application-Level Scope**: Maps to how developers think about their applications
3. **Collection of Related Work**: Natural fit for grouping traces from a single service

### User Feedback (PR Comment Context)

From PR #575 comment by @codekiln:

> Calling them "sessions" just seems wrong. a tracing project is a collection of traces. A session is typically a stateful period of time. These don't seem compatible to me.

**This perfectly articulates why LangChain made the abstraction**:

- "Session" has wrong semantic connotations for trace collection
- "Project" better matches how developers conceptualize their applications
- The terminology should serve users, not expose API implementation details

## Historical Context Clues

**From**: `src/langsmith/log-traces-to-project.mdx:17`

> The `LANGSMITH_PROJECT` flag is only supported in JS SDK versions >= 0.2.16, use `LANGCHAIN_PROJECT` instead if you are using an older version.

**Inference**:

- Very early SDKs used `LANGCHAIN_PROJECT`
- Later standardized to `LANGSMITH_PROJECT`
- Never had a "session" variant
- Suggests "project" terminology was chosen from day one

## Terminology Consistency Across the Ecosystem

### User-Facing Layer

- **Environment Variables**: `LANGSMITH_PROJECT`, `LANGCHAIN_PROJECT`
- **SDK Methods**: `delete_project()`, `deleteProject()`
- **Documentation**: "log-traces-to-project", "Projects" section
- **UI**: LangSmith UI URLs use `/projects/p/{id}`

### API/Internal Layer

- **REST Endpoints**: `/sessions`, `delete_tracer_sessions`
- **Schema Classes**: `TracerSession`, `TracerSessionResult`
- **Query Parameters**: Some internal filtering may use session terminology

## Implications for Rust SDK (langstar)

### Recommended Approach: Follow LangChain's Precedent

1. **Public API**: Use `Project` struct and `*_project()` methods
2. **Internal Implementation**: Map to `/sessions` REST endpoints
3. **Documentation**: Consistently use "project" terminology
4. **Comments**: May note that API uses "sessions" internally

### Rationale

- **User Expectations**: Users coming from Python/JS SDKs expect "project"
- **Semantic Accuracy**: "Project" better describes the concept
- **Consistency**: Aligns with official documentation and other SDKs
- **Future-Proof**: If LangChain ever renames API endpoints, SDK interface doesn't need to change

## Related Concepts: Thread/Conversation Sessions

**Important Distinction**: The docs DO use "session" for a different concept:

```python
# Thread metadata keys (from observability-concepts.mdx:42)
metadata = {
    "session_id": "conv-123",      # OR
    "thread_id": "conv-123",        # OR
    "conversation_id": "conv-123"   # Same concept, different keys
}
```

**This is about**: Linking multiple traces into a conversation thread
**This is NOT**: The same as project/session terminology

## Conclusion

The LangChain documentation provides strong supporting evidence for the terminology split first discovered in the Python SDK:

1. **Deliberate Abstraction**: LangChain consciously chose "project" over "session"
2. **Consistent Translation**: API's "sessions" → SDK's "projects" across multiple languages
3. **User-Centric Design**: Terminology chosen based on semantic fit, not API implementation
4. **Long-term Commitment**: Pattern established early and maintained across versions

**Recommendation**: The Rust SDK should confidently use "project" terminology, following the well-established precedent set by Python and JavaScript SDKs and documented extensively in official docs.

## References

- `/src/langsmith/observability-concepts.mdx` - Core LangSmith concepts
- `/src/langsmith/log-traces-to-project.mdx` - Project configuration guide
- `/src/langsmith/trace-with-api.mdx` - API tracing examples
- Python SDK analysis in issue #574 scout document
- PR #575 comment discussion
