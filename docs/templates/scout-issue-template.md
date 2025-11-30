# Scout Issue Template (Phase 0.0)

**Title**: `[Scout] Research {feature-name} feasibility and API patterns`
**Labels**: `research`, `scout`
**Milestone**: (none - created before milestone exists)

## Purpose

Validate feasibility of **{feature-name}** before committing to a full 8-phase milestone.

## Scope

**Research only - do NOT implement.**

1. **Existing Langstar Code**: Search `./cli` and `./sdk` for partial implementations
2. **Python SDK Precedent**: Analyze `langsmith-sdk/python/langsmith/client.py`
3. **API Endpoints**: Identify REST endpoints and request/response shapes
4. **Experiments**: Run Python scripts to validate API behavior (critical for complex features)
5. **Complexity Assessment**: Rate low/medium/high, identify blockers
6. **Recommendation**: Go / No-Go / Conditional

## Deliverables

### 1. Research Report
`docs/research/{issue-num}-{slug}-scout.md`

### 2. Experiments (when needed)
`reference/experiments/{issue-num}-{slug}/`

Create experiments when:
- API behavior is unclear from documentation
- Need to validate request/response shapes
- Testing edge cases or error handling

Example structure:
```
reference/experiments/{issue-num}-{slug}/
├── README.md           # Experiment overview and findings
├── test_{feature}.py   # Python script calling LangSmith API
└── run_test.sh         # Shell wrapper with env setup
```

### 3. SDK Notes (optional)
`reference/repo/langchain-ai/langsmith-sdk/notes/README.md`

## Success Criteria

- [ ] Research report completed
- [ ] Experiments run (if API behavior unclear)
- [ ] Feasibility assessed (go/no-go/conditional)
- [ ] Technical blockers identified
- [ ] PR merged to main

## Out of Scope

- Implementing Rust code
- Creating sub-issues
- Designing CLI interface
- Creating milestone

## Next Steps

**Go**: Create milestone (`ls-{feature-slug}`), then Phase 0 parent issue.
**No-Go**: Document blockers, close issue.
**Conditional**: Address conditions, re-assess.

## References

- Phase 0.0: `docs/dev/feature-development-process.md#phase-00-pre-epic-scouting-optional`
- Example: Issue #398 (structured output prompts)
