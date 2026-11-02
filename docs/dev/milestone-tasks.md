# Milestone tasks

`mise run milestone:prep-next [milestone]` picks the next open issue in a milestone, labels it `ready` and creates its worktree. The `gh-milestones-prep-next` skill calls it; the code is `mise-tasks/milestone/prep-next`, and `mise run milestone:prep-next --help` lists its argument.

`mise run milestone:prep-next` runs these steps in this order:

- finds the closed issue with the highest number in the milestone and removes its `wip` label;
- picks the closed issue's next open sibling, or when it has none its parent's next open sibling, or when that is missing too the open issue with the lowest number;
- goes down from that issue into its lowest-numbered open sub-issue, and again into that one's, until it reaches an issue with no open sub-issues, and picks that issue;
- labels that issue `ready`;
- creates its worktree under the root checkout;
- prints the issue's context.

Without an argument it takes the milestone from the current branch's issue. It needs `gh` and `git`. Where the `gh-sub-issue` extension is installed it reads each issue's parent and sub-issues to walk the hierarchy; without it the task picks the open issue with the next higher number than the closed one, or the lowest-numbered open issue when none is higher, and does not go down into sub-issues.

`mise run milestone:test` runs the tests in `tests/milestone/`.
