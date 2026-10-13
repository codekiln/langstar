# Milestone tasks

`mise run milestone:prep-next [milestone]` picks the next open issue in a milestone, labels it `ready` and creates its worktree. The `gh-milestones-prep-next` skill calls it; the code is `mise-tasks/milestone/prep-next`, and `mise run milestone:prep-next --help` lists its argument.

`mise run milestone:prep-next` does five things in order:

- finds the closed issue with the highest number in the milestone and removes its `wip` label;
- picks the next issue: the completed issue's next open sibling, else its parent's next open sibling, else the open issue with the lowest number, and then goes down from that issue to its first open sub-issue that has no open sub-issues;
- labels that issue `ready`;
- creates its worktree under the root checkout;
- prints the issue's context.

Without an argument it takes the milestone from the current branch's issue. It needs `gh` and `git`, and uses the `gh-sub-issue` extension when installed.

`mise run milestone:test` runs the tests in `tests/milestone/`.
