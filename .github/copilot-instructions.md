# GitHub Copilot Custom Instructions for Langstar

## Code Review Philosophy

Focus on **functional correctness, security, and maintainability** over minor documentation inconsistencies or stylistic preferences.

## Documentation Review Guidelines

### Approximations and Rounding

- **Do NOT flag minor numerical inconsistencies** in documentation when values are approximations
- Documentation often uses rounded numbers (e.g., "~2000 lines" vs "~1,573 lines", "~15 lines" vs "~14 lines")
- These approximations are intentional and acceptable for readability
- Only flag numerical inconsistencies if they represent:
  - Orders of magnitude differences (e.g., "~100" vs "~10,000")
  - Critical accuracy issues (e.g., API version numbers, dependency versions)
  - Functional errors (e.g., incorrect code examples, broken links)

### Documentation Style

- Accept reasonable variations in documentation style across files
- Focus on whether documentation is **clear and helpful**, not whether it matches exact formatting conventions
- Do NOT suggest changes for:
  - Minor wording differences
  - Approximate vs exact numbers in explanatory text
  - Different levels of detail in different contexts

## Code Review Priorities

### High Priority (Always Flag)

1. **Security vulnerabilities**: Exposed secrets, unsafe operations, injection risks
2. **Functional bugs**: Logic errors, incorrect behavior, broken functionality
3. **Breaking changes**: API contract violations, backward compatibility issues
4. **Test failures**: Any failing tests, missing test coverage for new code
5. **Build errors**: Compilation failures, dependency issues, CI failures

### Medium Priority (Flag When Significant)

1. **Performance issues**: Inefficient algorithms, unnecessary allocations, blocking operations
2. **Error handling**: Missing error handling, inappropriate error types
3. **Code quality**: Complex logic that could be simplified, unclear naming
4. **Maintainability**: Code duplication, missing documentation for complex logic

### Low Priority (Generally Skip)

1. **Minor documentation inconsistencies**: Approximate numbers, wording variations
2. **Stylistic preferences**: Formatting choices, naming style variations
3. **Optimization opportunities**: Micro-optimizations that don't impact performance
4. **Documentation completeness**: Missing docs for self-explanatory code

## Rust-Specific Guidelines

### Code Style

- Trust `cargo fmt` and `cargo clippy` for style enforcement
- Do NOT suggest style changes that conflict with automated tooling
- Focus on functional issues rather than style preferences

### Testing Standards

- All code must have appropriate tests (see `docs/dev/testing/`)
- Flag missing tests for new functionality
- Do NOT flag test style variations unless they impact test quality

## Project-Specific Context

### Development Workflow

- This project follows GitHub issue-driven development (see `docs/dev/github-workflow.md`)
- PRs should link to issues using `Fixes #N` or `Closes #N`
- Branch naming: `<username>/<issue_num>-<issue_slug>`

### Testing Philosophy

- Follow the **Toyota Andon Cord principle**: any failing test stops the merge
- Pre-commit requirements are enforced (see `CLAUDE.md`)
- Focus on test correctness, not minor test style variations

### Documentation Patterns

- This project uses **progressive disclosure** for documentation (see `docs/dev/progressive-disclosure-docs-standards.md`)
- Documentation may reference other docs using `@` prefix or plain paths
- Approximate numbers and rounded metrics are acceptable in documentation

## Review Tone

- Be constructive and helpful
- Focus on actionable feedback
- Avoid nitpicking minor inconsistencies
- Prioritize issues that impact code quality, security, or functionality

## Example: What NOT to Flag

❌ **Don't flag**: "The line count stated here as '~15 lines' is inconsistent with the actual README.md file which is 14 lines"

✅ **Accept**: Approximate numbers in documentation are intentional for readability

❌ **Don't flag**: "The metrics are inconsistent across files. In AGENTS.md line 74, it states '~2000 lines' but the validation report line 61 states '~1,573 lines'"

✅ **Accept**: Different files may use different approximations; both are reasonable

## Example: What TO Flag

✅ **Flag**: "This function can panic if the input is empty - consider adding error handling"

✅ **Flag**: "This code exposes an API key in a log statement - this is a security risk"

✅ **Flag**: "This test is missing - the new function `create_user()` has no test coverage"

✅ **Flag**: "This change breaks backward compatibility - existing code using `old_api()` will fail"

