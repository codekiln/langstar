# Langstar

## Project Overview

Langstar is a tool for interacting with LangSmith and LangGraph REST APIs.
See @/reference/api-specs/LANGSMITH_API_OVERVIEW.md for overview of APIs.
At a high level, the key deliverables are:
* `./sdk` - a set of rust-based SDKs for calling the LangSmith REST endpoints
* `./cli` - a unified CLI calling the langstar rust SDKs 
* `.devcontainer/features/langstar` - a Devcontainer feature with the CLI installed

## Dev Setup
* see .devcontainer
* git access is provided via a github fine-grained personal access token
  * it's locked down to this repo

This project prefers **Rust-based tools** wherever that is practical.

## Development Workflow

This project follows a GitHub issue-driven development workflow. For complete details, see @docs/dev/github-workflow.md

Key points:
- Create GitHub issues for all work
- Use branch naming convention: `<username>/<issue_num>-<issue_slug>`
- Follow Conventional Emoji Commits for commit messages
- Link PRs to issues using `Fixes #N` or `Closes #N`

## Coding Conventions

All coding conventions and development guidelines can be found in @docs/dev/README.md

For commit message formatting, please follow @docs/dev/git-scm-conventions.md

### Basic principles - see @docs/dev/code-style-principles.md

## Supporting Repository Structures

### `docs/` - Project Documentation
- `dev/` - Development guidelines, workflow docs, ADRs (see @docs/dev/README.md)
- `examples/` - Example code and usage patterns
- `implementation/` - Implementation plans and specifications
- `research/` - Research reports and findings for specific internal issues
- `templates/` - templates for milestone tickets and checklists 
- `usage/` - Usage documentation and guides

#### Testing Standards (Progressive Disclosure)

For testing documentation, **always use progressive disclosure**:
1. **TOC First:** Load `@docs/dev/testing/README.md` (~15-line index)
2. **On-Demand:** Load specific docs only when designing/debugging tests
3. **Example:** Writing CLI tests? Load `@docs/dev/testing/cli-integration-tests.md`

**Do NOT load all testing docs by default.** See `@docs/dev/progressive-disclosure-docs-standards.md` for patterns.

### `reference/` - External Resources & Experiments
- `api-specs/` - API specifications (LangSmith, control-plane)
- `experiments/` - Python experiments for API interaction
- `openapi/langchain/` - OpenAPI JSON specifications
- `repo/` - Remote repository notes (see .claude/skills/setup-remote-repo-notes-dir knowledge management pattern)
- `research/` - Research reports on external codebases

### `tests/` - root-level fixtures for integration tests
- see `tests/fixtures/test-graph-deployment/README.md` info about langsmith test deployment for integration tests

### `wip/` - work in progress
- gitignored
- includes git worktrees for active in dev issues. `scripts/cleanup-closed-issue-worktrees.sh` and .claude/skills/git-worktrees creates and cleans up.
- some txt files with debugging for active issues
