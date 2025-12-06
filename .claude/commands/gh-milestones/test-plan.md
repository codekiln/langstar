---
description: Generate comprehensive test plan for a milestone using progressive disclosure
argument-hint: <milestone-name-or-number>
---

# Generate Comprehensive Test Plan for Milestone

This command helps AI agents create test plans following langstar testing standards.

## Usage

```
/gh-milestones:test-plan <milestone-name-or-number>
```

**Examples:**
- `/gh-milestones:test-plan ls-runs-query`
- `/gh-milestones:test-plan 14`
- `/gh-milestones:test-plan https://github.com/codekiln/langstar/milestone/14`

## Command Behavior

When this command runs, you should:

### Step 1: Load Testing Documentation TOC

Load `@docs/dev/testing/README.md` to understand available testing docs.

### Step 2: Fetch Milestone Information

```bash
# If milestone name provided
gh api repos/codekiln/langstar/milestones --jq ".[] | select(.title==\"$MILESTONE_NAME\")"

# If milestone number provided
gh api repos/codekiln/langstar/milestones/$MILESTONE_NUMBER
```

Fetch the parent issue for context:
```bash
gh issue view <parent-issue-number> --json title,body,labels
```

### Step 3: Identify Milestone Type

Analyze the milestone to determine feature type. Ask the user if unclear:

**Feature types:**
- **SDK feature** - New API client methods, data types
- **CLI feature** - New commands or subcommands
- **Infrastructure** - CI/CD, devcontainer, build system
- **Documentation** - Docs-only changes

Use the AskUserQuestion tool if type is unclear from milestone description.

### Step 4: Load Relevant Testing Docs (Progressive Disclosure)

Based on feature type, load specific docs:

**For SDK features:**
- `@docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` (always)
- `@docs/dev/testing/sdk-integration-tests.md`
- `@docs/dev/testing/mocking-patterns.md`
- `@docs/dev/testing/test-fixtures.md` (if using test deployments)

**For CLI features:**
- `@docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` (always)
- `@docs/dev/testing/cli-integration-tests.md`
- `@docs/dev/testing/crud-lifecycle-pattern.md` (if CRUD operations)
- `@docs/dev/testing/test-fixtures.md` (if using test deployments)

**For infrastructure:**
- `@docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` (always)
- `@docs/dev/testing/devcontainer-feature-tests.md` (if devcontainer)
- Relevant CI/CD documentation

**For documentation:**
- Testing may not be applicable, inform user

### Step 5: Generate Test Plan

Create a comprehensive test plan document at `docs/implementation/<milestone-name>-test-plan.md`:

**Required sections:**

1. **Test Strategy Overview**
   - Milestone summary
   - Feature type
   - Testing approach (unit + integration mix)

2. **Unit Tests**
   - What to mock
   - Test cases to cover
   - Expected coverage
   - Files: `sdk/tests/` or in-module tests

3. **Integration Tests**
   - Prerequisites (env vars, test deployments)
   - CRUD lifecycle scenarios
   - Test data management
   - Cleanup strategy
   - Files: `sdk/tests/*_test.rs` or `cli/tests/*_command_test.rs`

4. **Test Data & Fixtures**
   - Test deployment requirements
   - Naming conventions
   - Shared vs fresh fixtures

5. **Success Criteria**
   - Coverage targets
   - CI requirements
   - Manual testing checklist

6. **References**
   - Link to loaded testing docs
   - Related test examples from codebase

### Step 6: Output Test Plan

Present the generated test plan to the user with:
- Summary of testing approach
- Link to generated test plan document
- Checklist of tasks to implement tests
- Reminder about Toyota andon cord principle

## Example Output

```markdown
## Test Plan Generated for ls-runs-query

**Feature type:** CLI feature with SDK components
**Test approach:** Unit tests (mocking) + Integration tests (CRUD lifecycle)

### Key Requirements:
- [ ] Unit tests for query building logic (httpmock)
- [ ] Integration test: Create run with SDK -> verify with SDK
- [ ] Integration test: Create run with SDK → Query via CLI → Verify in results
- [ ] Integration test: Test pagination with large result sets
- [ ] Integration test: Test filtering with various query params
- [ ] All tests must verify actual behavior (not just exit codes)

### Toyota Andon Cord Reminder:
All tests must pass before merge. No exceptions for "unrelated failures."

See `@docs/dev/testing/HIGH_LEVEL_TESTING_GUIDELINES.md` for requirements.
```

## Notes

- **Customization:** Tailor test plan to specific milestone needs
- **Standards Compliance:** Ensure test plan follows HIGH_LEVEL_TESTING_GUIDELINES.md
