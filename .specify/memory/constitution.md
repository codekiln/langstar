<!--
SYNC IMPACT REPORT
==================
Version Change: 1.0.0 → 1.1.0
- Added two new core principles (MINOR version bump)
- Extended governance for AI automation and multi-environment support

Modified Principles: N/A
Added Sections:
  - Principle VI: AI Automation First (human time is precious)
  - Principle VII: Multi-Environment Development (local/codespaces/GitHub Actions)

Removed Sections: N/A

Templates Status:
  ✅ spec-template.md - Aligned with all principles
  ✅ plan-template.md - Constitution Check section references this document
  ✅ tasks-template.md - Task categorization includes automation and CI/CD tasks
  ⚠️  Follow-up: Consider creating Claude Code skills for common SOPs

Follow-up TODOs:
  - Document standard operating procedures that should be automated via skills
  - Ensure GitHub Actions workflows validate constitution compliance
-->

# Langstar Constitution

## Core Principles

### I. Rust-First Development

All implementation MUST be in Rust unless explicitly justified otherwise. The project prioritizes Rust-based tools for better performance and seamless integration with the Rust ecosystem.

**Rationale**: Rust provides memory safety, performance, and a rich ecosystem. Consistency in language choice reduces cognitive overhead and simplifies tooling. Exceptions require documented justification demonstrating clear advantages that outweigh ecosystem consistency.

**Non-negotiable requirements**:
- Primary implementation language is Rust
- Build system is Cargo
- Rust-based tools preferred (e.g., ripgrep over grep, fd over find)
- Deviation requires explicit approval with documented rationale

### II. Issue-Driven Development (NON-NEGOTIABLE)

Every change MUST originate from a GitHub issue. No direct commits to main branch.

**Rationale**: Traceability ensures every change has documented context and purpose. Issue-driven development provides clear audit trails, enables better project planning, and facilitates team collaboration.

**Non-negotiable requirements**:
- GitHub issue created before work begins
- Branch naming: `<username>/<issue_num>-<issue_slug>`
- Pull requests MUST reference issues using `Fixes #N` or `Closes #N`
- All work tracked through issue lifecycle (open → in progress → closed)

### III. Conventional Emoji Commits

All commits MUST follow Conventional Emoji Commits specification.

**Rationale**: Standardized commit messages enable automated changelog generation, semantic versioning, and clear communication of change intent. Emojis provide visual scanning efficiency in commit history.

**Required format**:
```
<emoji> <type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Commit types**:
- `✨ feat` - New feature (MINOR version bump)
- `🩹 fix` - Bug fix (PATCH version bump)
- `🚨 BREAKING CHANGE` - API-breaking change (MAJOR version bump)
- `📚 docs` - Documentation changes
- `♻️ refactor` - Code refactoring
- `🧪 test` - Test additions/modifications
- `🔧 build` - Build system or dependency changes
- `🎨 style` - Formatting changes
- `⚡️ perf` - Performance improvements

### IV. Devcontainer-Based Development

Development MUST occur within the project's devcontainer environment.

**Rationale**: Devcontainers ensure consistent development environments across team members, eliminate "works on my machine" problems, and provide reproducible builds. Version-controlled devcontainer configuration becomes the single source of truth for tooling requirements.

**Non-negotiable requirements**:
- All development occurs in devcontainer unless explicitly impossible
- Devcontainer configuration version-controlled in `.devcontainer/`
- Tool versions managed via `mise.toml`
- Required tools installed during Docker build (cached for performance)

### V. Spec-Driven Development for Complexity

Complex features MUST use Spec-Kit workflow; simple changes SHOULD NOT.

**Rationale**: Detailed specifications reduce ambiguity and improve AI agent implementation accuracy for complex features. Over-specification of trivial changes creates unnecessary overhead. The complexity threshold determines the appropriate process.

**Complexity threshold**:

**USE Spec-Kit for**:
- Features requiring detailed planning across multiple components
- Features with ambiguous or evolving requirements
- Major architectural changes
- Features implemented by AI agents requiring clear specifications

**SKIP Spec-Kit for**:
- Simple bug fixes with clear root cause
- Trivial changes (typos, formatting, single-line fixes)
- Well-understood modifications with established patterns
- Documentation updates without architectural impact

**Spec-Kit workflow** (when applicable):
1. `/speckit.specify` - Define requirements and acceptance criteria
2. `/speckit.plan` - Create technical implementation plan
3. `/speckit.tasks` - Generate actionable task list
4. `/speckit.implement` - Execute implementation

### VI. AI Automation First (NON-NEGOTIABLE)

Human time is precious. Standard operating procedures MUST be automated using Claude Code skills whenever possible. Repetitive tasks that can be delegated to AI MUST NOT consume human attention.

**Rationale**: Humans should focus on high-value decision-making, architecture, and creative problem-solving. Routine operations (status updates, branch creation, boilerplate generation, workflow orchestration) should be delegated to AI agents. This maximizes human productivity and reduces cognitive load from repetitive tasks.

**Non-negotiable requirements**:
- Standard operating procedures documented in skills (`.claude/skills/`)
- Repetitive workflows automated via Claude Code skills or GitHub Actions
- Skills MUST be version-controlled and documented
- Manual execution of automatable tasks is discouraged unless automation impossible

**Examples of required automation**:
- GitHub issue status updates → skill automation
- Branch creation and PR generation → skill automation
- Boilerplate code generation → skill automation
- Documentation updates from code changes → skill automation
- Project board management → skill automation

**Human involvement required for**:
- Architectural decisions
- Design trade-off evaluation
- Code review and approval
- Security and compliance review
- Complex debugging requiring creative problem-solving

### VII. Multi-Environment Development

Development MUST be portable across three environments: local devcontainer, GitHub Codespaces, and GitHub Actions. Maximum development work SHOULD occur in GitHub Actions.

**Rationale**: Environment portability ensures development can occur anywhere while maintaining consistency. GitHub Actions provides the most reproducible, auditable, and cost-effective environment for automated development work. Local and Codespaces environments enable human oversight and interactive debugging when needed.

**Environment priorities** (in order of preference):
1. **GitHub Actions** (preferred) - Automated development, testing, and deployment
2. **GitHub Codespaces** - Cloud-based development requiring human interaction
3. **Local Devcontainer** - Local development with full hardware access

**Non-negotiable requirements**:
- Devcontainer configuration MUST work identically in all three environments
- GitHub Actions workflows MUST validate constitution compliance
- Tool versions MUST be consistent across environments (via `mise.toml`)
- Environment-specific overrides documented and minimized
- CI/CD pipelines execute in GitHub Actions, not local machines

**Environment-specific guidelines**:

**GitHub Actions** (primary automation target):
- Use for: automated builds, tests, deployments, PR automation, issue management
- Claude Code GitHub Actions integration for AI-driven development
- All workflows version-controlled in `.github/workflows/`
- Secrets managed via GitHub repository/organization secrets

**GitHub Codespaces**:
- Use for: interactive development requiring human oversight
- Cloud-based with consistent devcontainer environment
- Suitable for code review, exploratory debugging, pair programming

**Local Devcontainer**:
- Use for: offline development, hardware-dependent testing, initial setup
- Identical configuration to Codespaces
- Required for contributors without Codespaces access

## Development Standards

### Code Review Requirements

All pull requests MUST:
- Pass automated CI/CD checks
- Include tests for new functionality or bug fixes
- Reference the originating GitHub issue
- Follow Conventional Emoji Commits format in title
- Provide clear description of changes and test plan
- Receive at least one approval before merge

### Branch Management

- Branch naming convention strictly enforced: `<username>/<issue_num>-<issue_slug>`
- Branches created from latest `main` branch
- Branches deleted after successful merge
- No direct commits to `main` branch
- Merge strategy: Squash and merge (recommended) to maintain clean history

### Documentation Requirements

Code changes MUST include documentation updates when:
- Public APIs added or modified
- Behavior changes affect external interfaces
- New configuration options introduced
- Architecture or design patterns established

Documentation location: `docs/dev/` for development guidelines, inline for code documentation.

## Quality Assurance

### Testing Requirements

All changes MUST include appropriate tests:
- Unit tests for isolated functionality
- Integration tests for cross-component interactions
- Test coverage for bug fixes demonstrating issue resolution
- Tests MUST pass before merge approval

### Pull Request Standards

PR descriptions MUST include:
- Summary of changes made
- Related issue link with closing keywords
- Test plan demonstrating verification
- Screenshots for UI changes (if applicable)
- Breaking changes explicitly documented

## Governance

This constitution supersedes all other development practices and conventions. All team members MUST verify compliance during code reviews.

### Amendment Process

Constitution amendments require:
1. GitHub issue documenting proposed change and rationale
2. Discussion period allowing team input
3. Approval from project maintainers
4. Version bump following semantic versioning rules
5. Migration plan if changes affect existing workflows

### Versioning Policy

Constitution versions follow semantic versioning (MAJOR.MINOR.PATCH):
- **MAJOR**: Backward-incompatible governance changes or principle removals
- **MINOR**: New principles added or materially expanded guidance
- **PATCH**: Clarifications, wording improvements, non-semantic refinements

### Compliance Review

All pull requests and code reviews MUST verify compliance with:
- Core Principles (I-VII)
- Development Standards
- Quality Assurance requirements

Any complexity introduced MUST be explicitly justified against the principle of simplicity. When in doubt, favor simpler approaches.

GitHub Actions workflows MUST validate constitution compliance automatically where possible (e.g., commit message format, branch naming, issue linkage).

### Runtime Guidance

For day-to-day development guidance, refer to:
- `CLAUDE.md` - Project overview and quick reference
- `docs/dev/README.md` - Development documentation index
- `docs/dev/github-workflow.md` - Detailed workflow procedures
- `docs/dev/git-scm-conventions.md` - Commit message standards
- `docs/dev/spec-kit.md` - Spec-driven development workflows

**Version**: 1.1.0 | **Ratified**: 2025-10-26 | **Last Amended**: 2025-10-26
