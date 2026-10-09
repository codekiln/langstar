# Tmux Manager/Worker Workflow

A manager agent in window 0 of a tmux session starts one worker agent per window, and each worker takes one GitHub issue to a merged pull request. The layout follows two of codekiln's AI rules in the `logseq-encode-garden` repository: [My/AI/Rule/Dev/Workflow/Git Worktree PR](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___Dev___Workflow___Git%20Worktree%20PR.md) and [My/AI/Rule/Dev/Workflow/Git Worktree PR/Tmux](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___Dev___Workflow___Git%20Worktree%20PR___Tmux.md).

The point of the layout: opening the tmux session picker shows everything the agents are working on, one line per agent.

## Layout

| Tmux thing | Rule | Example |
|------------|------|---------|
| Session | Named for an area of responsibility: a whole repository, or an area within one | `langstar` |
| Window 0 | The manager agent, started with `claude -n langstar-manager` | `langstar-manager` |
| Window 1..n | One worker agent per window, named for its job | `i738-tmux-workflow` |
| Pane 0 of a window | The agent | `claude -n i738-tmux-workflow` |

- At most one agent per window, and the agent is always in the first pane. Other panes (logs, a shell) come after it.
- The window name tells you what the agent is for. Name the window for the job, for example `i738-tmux-workflow`.
- Each agent runs `claude -n <name>`, where `<name>` matches its window name. The Claude session name and the window name then identify the same agent.

## Manager

The manager is the agent in window 0. It wakes workers up and puts them to sleep:

- **Wake a worker:** create a window named for the job, start `claude -n <same name>` in its first pane, and hand it a brief (the issue, the branch name, the finish line).
- **Put a worker to sleep:** once its PR has merged and it reports done, close its window.

The manager picks the issues, records which worker owns which, and answers worker questions.

## Workers reach the manager with SendMessage

A worker sends a message to the manager with Claude's `SendMessage` tool, addressed to the manager's name exactly as `ListAgents` prints it. The display name from `claude -n` can differ from that routable name, so a worker looks the manager up with `ListAgents` (or uses the address the manager gave it in its brief) instead of guessing; a guess fails with "No agent named ... is reachable". It never uses `tmux send-keys`. `tmux send-keys -t <manager pane>` types into the manager's input box, where the text lands in the middle of whatever the human is typing. `SendMessage` delivers a separate message that the manager reads in turn.

Claude Code does not load the `SendMessage` tool until it is asked to. Run `ToolSearch` with the query `select:SendMessage` before the first message.

## Worker lifecycle

1. **Start.** Work from one GitHub issue, in a worktree whose branch name carries the issue (`m<milestone>-p<parent>-i<issue>-<slug>`). See [GitHub Workflow](./github-workflow.md#step-2-branch-creation).
2. **Commit.** Use Conventional Emoji Commits, with the last line `#<N> <Issue Title>`. See [Git SCM Conventions](./git-scm-conventions.md#ticket-references).
3. **Open the PR** with `Fixes #<N>` in the body and the issue's milestone.
4. **Open it in the browser** (`gh pr view --web`) so codekiln can read it. Then tell the manager the PR is ready with `SendMessage`.
5. **Answer review comments.** codekiln leaves comments in the browser. The worker fixes each one, pushes, replies to the comment and resolves the thread.
6. **After merge, clean up.** Sync the root checkout with `git pull --ff-only` on `main`, remove the worktree with `git worktree remove`, delete the local branch, and tell the manager the job is done.

The root checkout stays on `main` and clean throughout. A worker never edits or commits there.

If codekiln asks for tuicr, a terminal tool for reviewing a diff, the worker opens the PR diff in tuicr in a new pane of its own window, after the agent's first pane.

## Stacked PRs

When a branch builds on another unmerged branch (its parent):

1. Branch the child from the parent's branch, not from `main`.
2. Open the child's PR with `--base <parent-branch>`. CI runs on a pull request into any base branch, since [🔧 build(ci): run CI on stacked PRs whose base is not main (#758)](https://github.com/codekiln/langstar/pull/758) merged.
3. When the parent squash-merges, fetch the merged `main` tip with `git fetch origin`, then rebase the child onto it, dropping the parent's commits: `git rebase --onto origin/main <old-parent-tip>`.
4. Push the rebased child with `git push --force-with-lease`, so the PR's remote head no longer holds the parent's commits.
5. Retarget the PR with `gh pr edit --base main`.

## Ready checklist

Open each PR ready for review. When a PR starts as a draft, run `gh pr ready <n>` before reporting it: Copilot does not review drafts, and codekiln reviews only PRs that are ready.

Ask codekiln to review a PR only after these two steps:

1. Resolve every Copilot review thread.
2. Run the `codekiln-review` skill on the PR: `/codekiln-review codekiln/langstar <n>`. It is codekiln's personal skill, installed at `~/.claude/skills/codekiln-review`, not part of this repository.

## Related files

- [GitHub Workflow](./github-workflow.md) - issue-driven development process
- [Git SCM Conventions](./git-scm-conventions.md) - commit message conventions
- [Debugging Tests](./testing/debugging-tests.md) - includes `claude --chrome` for LangSmith UI state
- `.devcontainer/.tmux.conf` - tmux configuration
