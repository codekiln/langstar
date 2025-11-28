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
