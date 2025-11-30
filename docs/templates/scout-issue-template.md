# Scout Issue Template (Phase 0.0)

Use this template when creating a Pre-Epic Scouting issue to validate feasibility before committing to a full milestone.

---

**Title**: `[Scout] Research {feature-name} feasibility and API patterns`

**Labels**: `research`, `scout`

**Milestone**: (leave empty - no milestone exists yet)

---

## Purpose

Exploratory research to validate feasibility of implementing **{feature-name}** in langstar CLI before committing to a full 8-phase milestone.

## Context

{Brief explanation of why scouting is needed. Examples:}
- This feature requires API endpoints we haven't used before
- Unclear if langsmith-sdk Python provides precedent for this functionality
- Unknown complexity of implementing {specific technical aspect}
- Need to validate assumptions about API capabilities

## Scope

**Do NOT propose an implementation.** Focus on research only:

1. **Scout Existing Langstar Code**:
   - Search `./cli` and `./sdk` for partial implementations
   - Identify any existing types, methods, or patterns that overlap
   - Document what's already done vs. what's needed

2. **Analyze Python SDK Precedent**:
   - Use `setup-remote-repo-notes-dir` skill to clone langsmith-sdk
   - Locate relevant methods in `python/langsmith/client.py`
   - Document method signatures, parameters, patterns
   - Review tests for usage examples

3. **Identify API Endpoints**:
   - Use LangSmith UI or documentation to understand feature behavior
   - Identify which REST API endpoints are involved
   - Document request/response shapes (preliminary)

4. **Assess Complexity**:
   - Rate implementation complexity (low/medium/high)
   - Identify technical blockers or unknowns
   - List dependencies on other unimplemented features

5. **Make Go/No-Go Recommendation**:
   - **Go**: Feature is feasible, proceed to Phase 0 (Epic Setup)
   - **No-Go**: Feature has blockers, defer or cancel
   - **Conditional**: Feature is feasible if {conditions met}

## Deliverables

### 1. Research Report

**Location**: `docs/research/{issue-num}-{slug}-scout.md`

**Required sections**:
- Executive Summary (go/no-go recommendation)
- Existing Langstar Implementation Analysis
- Python SDK Precedent Analysis
- API Endpoint Documentation
- Complexity Assessment
- Technical Blockers (if any)
- Open Questions for Implementation
- Recommendation

### 2. Updated Reference Notes

**Location**: `reference/repo/langchain-ai/langsmith-sdk/notes/README.md`

**Content**: Document key SDK findings:
- Which files/modules are relevant
- Key types or patterns discovered
- Links to specific line numbers in SDK code

### 3. Optional: Experiment Scripts

**Location**: `reference/experiments/{issue-num}-{slug}/`

**Content**: Python scripts to test API behavior (optional):
- `README.md` - Experiment overview
- `test_{feature}.py` - Python script calling LangSmith API
- `run_test.sh` - Shell wrapper with environment setup

**When to create experiments**:
- API behavior is unclear from documentation
- Need to validate assumptions about request/response shapes
- Testing edge cases or error handling

## Success Criteria

- [ ] Research report completed at `docs/research/{issue-num}-{slug}-scout.md`
- [ ] Feasibility clearly assessed (go/no-go/conditional)
- [ ] Technical blockers identified (if any)
- [ ] API complexity understood
- [ ] Recommendation for next steps documented
- [ ] Reference notes updated with SDK findings
- [ ] PR created and merged to main

## Out of Scope

This scout issue should NOT:
- ❌ Implement any Rust code
- ❌ Create sub-issues for implementation phases
- ❌ Design the CLI interface
- ❌ Validate against OpenAPI spec (that's Phase 3)
- ❌ Create the milestone or parent issue

## Next Steps After Scout

**If Go Recommendation**:
1. Review scout findings with team
2. Create GitHub milestone
3. Create parent issue (Phase 0) referencing this scout report
4. Break down into 8 sub-issues (Phases 1-8)
5. Optionally: Retroactively attach this scout issue to milestone

**If No-Go Recommendation**:
1. Document blockers and reasoning
2. Close this issue with explanation
3. Revisit when blockers are resolved

**If Conditional Recommendation**:
1. Address conditions/blockers
2. Re-assess feasibility
3. Proceed to "Go" or "No-Go" path

---

## References

- **Phase 0.0 Documentation**: `@docs/dev/feature-development-process.md#phase-00-pre-epic-scouting-optional`
- **Example Scout Issue**: [Issue #398](https://github.com/codekiln/langstar/issues/398) - Structured output prompts scout
- **setup-remote-repo-notes-dir Skill**: `.claude/skills/setup-remote-repo-notes-dir/SKILL.md`

---

## Notes

This is a **Phase 0.0** issue, created BEFORE the parent milestone issue. The research output informs whether to proceed with a full 8-phase milestone.
