---
root: true
targets: ["*"]
description: "Langstar: project, workflow, testing, secrets"
---

# Langstar

Langstar's goal is to be the one tool people use for all of their LangSmith work, in place of the LangSmith MCP server, the LangSmith CLI and the LangGraph CLI.

Langstar is a Rust SDK (`sdk/`) and CLI (`cli/`) for the LangSmith and LangGraph REST APIs, plus a devcontainer feature that installs the CLI (`.devcontainer/features/langstar`). `reference/api-specs/LANGSMITH_API_OVERVIEW.md` summarizes the APIs. Prefer Rust-based tools where practical, and configure the dev environment through `.devcontainer/`.

## Agent config

To change what agents read, edit `.rulesync/rules/` or `.rulesync/skills/`, run `"$(mise which rulesync)" generate`, and commit only the `.rulesync/` source. Git ignores the files rulesync writes (`CLAUDE.md`, `AGENTS.md`, `.claude/skills/` and `.agents/skills/`), and the lefthook hooks in `lefthook.yml` regenerate them after every pull and checkout. In a new clone, run `mise install`, then `"$(mise which lefthook)" install` and `"$(mise which rulesync)" generate` once; the devcontainer does this for you. Agents get skills only, never commands. `mise which rulesync` prints the path of the rulesync version pinned in `mise.toml`, so `"$(mise which rulesync)" generate` runs that version even when an older rulesync is earlier on `PATH`. An older rulesync drops `license`, `argument-hint` and `allowed-tools` from the generated skills and writes a `.codex/` directory.

## Workflow

- Open a GitHub issue for each piece of work before you start it, and close the issue from one PR whose body says `Fixes #<issue number>`. Details are in `docs/dev/github-workflow.md`.
- Name branches `m<milestone>-p<parent>-i<issue>-<slug>`, dropping the parts that don't apply.
- Work in a git worktree and leave the root checkout on `main`. The `git-worktrees` and `gh-start-issue` skills set one up.
- Write commit messages and PR titles as Conventional Emoji Commits (`✨ feat(scope): ...`), following `docs/dev/git-scm-conventions.md`.
- Give a PR the milestone of its issue.
- Design a new feature as an OpenSpec change under `openspec/changes/`, using the `openspec-propose` skill. `openspec/specs/` describes what each command group does today. `docs/dev/feature-development-process.md` has the process.
- Coding conventions are in `docs/dev/README.md`. Prefer an explicit setting to an implicit default, and link the docs for it.

## Testing

Merge a PR only when every test passes, including a test that was already failing before your change. Run this before every commit:

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo nextest run --profile ci --all-features --workspace
```

Integration tests need `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID` and `LANGSMITH_WORKSPACE_ID`; `docs/dev/environment-variables.md` maps them to API headers. Each test asserts on what the CLI command or SDK call returned, such as fields in its JSON output or the resource it created, as well as on its exit code. `docs/dev/testing/README.md` indexes the testing docs: read `HIGH_LEVEL_TESTING_GUIDELINES.md` and the one or two others your task needs. The `test-runner-worktree` skill runs the tests inside a worktree.

## Secrets

Write placeholders such as `<your-api-key>` wherever an API key, token, organization or workspace ID, or organization name would go, in files, issues, PRs and comments alike. If you post a secret in a GitHub comment, delete the comment and rotate the secret. GitHub keeps each earlier version of an edited comment in its edit history, so deleting the comment is the way to take the secret down.

## Background tasks

To see whether a background command has finished, read the end of its log with `tail -n 50 <log>`, or poll in a loop that gives up after a timeout. Use `tail -n` instead of `tail -f`, which keeps running until something kills it and leaves a process behind.

## Output budget

When `CLAUDE_CODE_MAX_OUTPUT_TOKENS` is set, keep each response under that many tokens. When a task needs a longer response, ask the user whether to summarize the response or split the task into smaller requests.
