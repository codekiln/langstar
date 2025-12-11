# Scout Issue Template (Phase 0.0)

**Title**: `[Scout] Research {feature-name} API patterns and technical context`
**Labels**: `research`, `scout`
**Milestone**: (none - created before milestone exists)

## Purpose

Gather preliminary research and technical knowledge about **{feature-name}** to inform milestone planning and ticket authoring.

## Scope

**Research and experimentation only - do not implement.**

1. **Existing Langstar Code**: Search `./cli` and `./sdk` for related implementations
2. **Python SDK Precedent**: Analyze `langsmith-sdk/python/langsmith/client.py` patterns
3. **API Endpoints**: Identify REST endpoints and request/response shapes
4. **Experiments**: Run Python scripts to explore API behavior and validate assumptions
5. **Technical Patterns**: Document key patterns, conventions, and integration points
6. **Milestone Planning Insights**: Identify how to structure Phase 0 parent issue and initial tickets

## Deliverables

### 1. Research Report
`docs/research/{issue-num}-{slug}-scout.md`

**Required sections:**
- Executive summary (purpose and key findings)
- Existing langstar code analysis
- Python SDK precedent analysis
- API endpoints and patterns
- Experimentation findings (if applicable)
- Technical insights for milestone planning
- Recommended structure for Phase 0 parent issue
- Suggested initial sub-issues

### 2. Experiments (as needed)
`reference/experiments/{issue-num}-{slug}/`

Create experiments to:
- Explore API behavior through hands-on testing
- Validate assumptions about request/response patterns
- Test edge cases and error handling
- Test integration patterns

Example structure:
```
reference/experiments/{issue-num}-{slug}/
├── README.md           # Experiment overview and findings
├── test_{feature}.py   # Python script for API pattern exploration
└── run_test.sh         # Shell wrapper with env setup
```

### 3. SDK Notes (optional)
`reference/repo/langchain-ai/langsmith-sdk/notes/README.md`

Document key SDK patterns, method signatures, and conventions that inform Rust implementation.

## Success Criteria

- [ ] Research report completed with technical insights
- [ ] Experiments run (if needed to validate assumptions)
- [ ] SDK precedent patterns documented
- [ ] API endpoints and request/response shapes identified
- [ ] Technical context gathered for milestone planning
- [ ] Clear recommendations for Phase 0 parent issue structure
- [ ] Suggested initial sub-issues identified
- [ ] PR merged to main

## Out of Scope

- Implementing Rust code
- Creating the milestone or sub-issues
- Detailed CLI interface design
- Final API design decisions

These decisions are informed by scout research but made during milestone planning.

## Next Steps

After scout research is complete:
1. Use findings to create milestone (`ls-{feature-slug}`)
2. Author Phase 0 parent issue using technical context from scout
3. Structure initial sub-issues based on patterns discovered
4. Reference scout research in milestone documentation

## References

- Phase 0.0: `docs/dev/feature-development-process.md#phase-00-pre-epic-scouting-optional`
- Example: Issue #398 (structured output prompts)
