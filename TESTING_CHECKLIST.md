# Testing Checklist for /gh-milestones:test-plan Command

## Pre-Merge Verification

### File Structure
- [x] `.claude/commands/gh-milestones/test-plan.md` created
- [x] `.claude/commands/gh-milestones/README.md` created/updated
- [x] `docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` updated
- [x] Files have correct frontmatter (description, argument-hint)

### Content Validation
- [x] Command specification is complete
- [x] All 6 steps are documented
- [x] Progressive disclosure pattern is enforced
- [x] Example output is provided
- [x] Testing documentation references are correct

## Post-Merge Testing

After this PR is merged and a new Claude Code session starts:

### 1. Command Discovery
```bash
# Verify command appears in available commands
/help
# Should show: /gh-milestones:test-plan
```

### 2. Basic Invocation
```bash
# Test with milestone name
/gh-milestones:test-plan ls-test-improvement

# Test with milestone number
/gh-milestones:test-plan 14
```

### 3. Verify Behavior

The command should:
- [ ] Load `@docs/dev/testing/README.md` first
- [ ] Fetch milestone information via `gh api`
- [ ] Identify milestone type correctly
- [ ] Load only relevant testing docs (2-4 docs, not all 8)
- [ ] Generate test plan at `docs/implementation/<milestone>-test-plan.md`
- [ ] Include all required sections:
  - [ ] Test Strategy Overview
  - [ ] Unit Tests
  - [ ] Integration Tests
  - [ ] Test Data & Fixtures
  - [ ] Success Criteria
  - [ ] References
- [ ] Keep context usage under 5000 tokens
- [ ] Provide actionable checklist
- [ ] Include Toyota andon cord reminder

### 4. Edge Cases

Test with different milestone types:
- [ ] SDK feature milestone
- [ ] CLI feature milestone
- [ ] Infrastructure milestone
- [ ] Documentation milestone

### 5. Progressive Disclosure Verification

Monitor which testing docs get loaded:
- [ ] Should NOT load all 8 docs from `docs/dev/testing/`
- [ ] Should load HIGH_LEVEL_TESTING_GUIDELINES.md always
- [ ] Should load 1-3 additional relevant docs based on feature type

### 6. Generated Test Plan Quality

Review generated test plan document:
- [ ] Contains concrete, specific test cases
- [ ] References actual codebase patterns
- [ ] Follows CRUD lifecycle pattern where applicable
- [ ] Includes cleanup strategies
- [ ] Specifies environment variables needed
- [ ] References Toyota andon cord principle

## Success Criteria

All checks must pass:
- [x] Command file exists with correct structure
- [x] README documents the command
- [x] HIGH_LEVEL_TESTING_GUIDELINES.md references command
- [ ] Post-merge: Command invocable
- [ ] Post-merge: Generated test plans follow standards
- [ ] Post-merge: Context usage <5000 tokens verified

## Notes

- Slash commands are loaded at CLI startup, not dynamically
- Testing must occur after PR merge in a new Claude Code session
- Context window monitoring is critical for progressive disclosure
- Generated test plans should be reviewed for quality and specificity
