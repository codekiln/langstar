# Milestone tasks

`mise run milestone:prep-next [milestone]` moves to the next issue in an active milestone. The `gh-milestones-prep-next` skill calls it; the code is `mise-tasks/milestone/prep-next`, and `mise run milestone:prep-next --help` lists its argument.

It finds the last completed issue in the milestone, removes its `wip` label, walks the issue hierarchy sibling-first and depth-first to the next open issue, labels that issue `ready`, creates its worktree under the root checkout and prints the issue's context. Without an argument it takes the milestone from the current branch's issue. It needs `gh` and `git`, and uses the `gh-sub-issue` extension when installed.

`mise run milestone:test` runs the tests in `tests/milestone/`.
