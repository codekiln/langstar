# Milestone tasks

`mise run milestone:prep-next [milestone]` picks the next open issue in an active milestone, labels it `ready` and creates its worktree. The `gh-milestones-prep-next` skill calls it; the code is `mise-tasks/milestone/prep-next`, and `mise run milestone:prep-next --help` lists its argument.

`mise run milestone:prep-next` does five things in order:

- finds the last completed issue in the milestone and removes its `wip` label;
- picks the next open issue: the next open sibling of the completed issue, or, when none is open, the first open issue among the children of the next sibling;
- labels that issue `ready`;
- creates its worktree under the root checkout;
- prints the issue's context.

Without an argument it takes the milestone from the current branch's issue. It needs `gh` and `git`, and uses the `gh-sub-issue` extension when installed.

`mise run milestone:test` runs the tests in `tests/milestone/`.
