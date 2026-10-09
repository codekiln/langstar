---
root: true
targets: ["*"]
description: "Langstar: project, workflow, testing, secrets"
---

# Langstar

Langstar is a Rust SDK (`sdk/`) and CLI (`cli/`) for the LangSmith and LangGraph REST APIs, plus a devcontainer feature that installs the CLI (`.devcontainer/features/langstar`). `reference/api-specs/LANGSMITH_API_OVERVIEW.md` summarizes the APIs. Prefer Rust-based tools where practical, and configure the dev environment through `.devcontainer/`.

## Agent config

rulesync generates the agent config from `.rulesync/` using `rulesync.jsonc`. `CLAUDE.md`, `AGENTS.md`, `.claude/skills/` and `.agents/skills/` are its output. Edit `.rulesync/rules/` or `.rulesync/skills/`, run `rulesync generate` with the version pinned in `mise.toml` (`"$(mise which rulesync)" generate` picks it even when an older rulesync is earlier on `PATH`), and commit the source with the output.

## Workflow

- Every change starts from a GitHub issue and lands in one PR whose body says `Fixes #N`. Details are in `docs/dev/github-workflow.md`.
- Name branches `m<milestone>-p<parent>-i<issue>-<slug>`, dropping the parts that don't apply.
- Work in a git worktree under `.worktrees/<branch-name>` (gitignored) and leave the root checkout on `main`. The `git-worktrees` and `gh-start-issue` skills set one up.
- Commit messages and PR titles use Conventional Emoji Commits (`✨ feat(scope): ...`), per `docs/dev/git-scm-conventions.md`.
- Give a PR the milestone of its issue.
- Coding conventions are in `docs/dev/README.md`. Prefer an explicit setting to an implicit default, and link the docs for it.

## Testing

A failing test stops the merge, whether or not your change caused it. Run this before every commit:

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo nextest run --profile ci --all-features --workspace
```

Integration tests need `LANGSMITH_API_KEY`, `LANGSMITH_ORGANIZATION_ID` and `LANGSMITH_WORKSPACE_ID`; `docs/dev/environment-variables.md` maps them to API headers. A test checks behavior; an exit code alone proves nothing. `docs/dev/testing/README.md` indexes the testing docs: read `HIGH_LEVEL_TESTING_GUIDELINES.md` and the one or two others your task needs. The `test-runner-worktree` skill runs the tests inside a worktree.

## Secrets

Write placeholders such as `<your-api-key>` wherever an API key, token, organization or workspace ID, or organization name would go, in files, issues, PRs and comments alike. If a secret reaches a GitHub comment, delete the comment, since an edit keeps it in the history, and rotate the secret.

## Background tasks

Wait for a background command through its completion notice or a bounded check, such as `tail -n 50 <log>` or a polling loop with a timeout. `tail -f` runs until killed and leaves a stray process behind; stop it at once if one starts.

## Output budget

When `CLAUDE_CODE_MAX_OUTPUT_TOKENS` is set, keep each response under it. If a task needs more, ask whether to summarize it or split it.
