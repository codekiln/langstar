#!/usr/bin/env python3
"""
Automate moving to next issue in a GitHub milestone workflow.

This script:
1. Finds the last completed issue in a milestone
2. Cleans up labels (removes 'wip' from completed)
3. Finds the next issue using intelligent traversal
4. Applies 'ready' label to next issue
5. Creates worktree for next issue
6. Displays issue context
"""

import argparse
import json
import os
import re
import subprocess
import sys
import traceback
from typing import Optional, Dict, List, Tuple


class MilestoneWorkflow:
    """Manages milestone workflow operations."""

    def __init__(self, milestone_arg: Optional[str] = None):
        self.milestone_arg = milestone_arg
        self.owner: Optional[str] = None
        self.repo: Optional[str] = None
        self.milestone_title: Optional[str] = None
        self.milestone_num: Optional[int] = None
        self.all_issues: List[Dict] = []
        self.parent_map: Dict[int, int] = {}  # child -> parent
        self.children_map: Dict[int, List[int]] = {}  # parent -> [children]
        self.use_simple_ordering = False

    def run_command(self, cmd: List[str], check=True, capture_output=True) -> subprocess.CompletedProcess:
        """Run a shell command and return result."""
        try:
            result = subprocess.run(
                cmd,
                check=check,
                capture_output=capture_output,
                text=True
            )
            return result
        except subprocess.CalledProcessError as e:
            if check:
                print(f"❌ Command failed: {' '.join(cmd)}", file=sys.stderr)
                print(f"   Error: {e.stderr}", file=sys.stderr)
                raise
            return e

    def get_repo_info(self) -> Tuple[str, str]:
        """Get current repository owner and name."""
        result = self.run_command([
            "gh", "repo", "view",
            "--json", "owner,name",
            "--jq", "{owner: .owner.login, name: .name}"
        ])

        data = json.loads(result.stdout)
        self.owner = data["owner"]
        self.repo = data["name"]
        return self.owner, self.repo

    def detect_milestone(self) -> Tuple[str, int]:
        """Detect or parse milestone from arguments or current branch."""
        if not self.milestone_arg:
            # Try to detect from current branch
            current_branch = self.run_command(["git", "branch", "--show-current"]).stdout.strip()

            # Extract issue number from branch (format: user/NNN-slug)
            match = re.search(r'/(\d+)-', current_branch)
            if match:
                issue_num = int(match.group(1))

                # Get milestone from this issue
                result = self.run_command([
                    "gh", "issue", "view", str(issue_num),
                    "--json", "milestone"
                ])

                milestone_data = json.loads(result.stdout).get("milestone")
                if milestone_data:
                    self.milestone_title = milestone_data["title"]
                    self.milestone_num = milestone_data["number"]
                    print(f"📍 Detected milestone from current branch: {self.milestone_title} (#{self.milestone_num})")
                    return self.milestone_title, self.milestone_num

            print("❌ Error: Could not auto-detect milestone", file=sys.stderr)
            print("   Usage: prep-next.py [milestone-name-or-number]", file=sys.stderr)
            print("   Or ensure you're on a branch associated with a milestone issue", file=sys.stderr)
            sys.exit(1)

        # Milestone explicitly provided
        if self.milestone_arg.isdigit():
            # Numeric milestone number
            self.milestone_num = int(self.milestone_arg)
            result = self.run_command([
                "gh", "api", f"repos/{self.owner}/{self.repo}/milestones/{self.milestone_num}"
            ], check=False)

            if result.returncode != 0:
                print(f"❌ Error: Milestone #{self.milestone_num} not found", file=sys.stderr)
                sys.exit(1)

            data = json.loads(result.stdout)
            self.milestone_title = data["title"]
        else:
            # Milestone name
            self.milestone_title = self.milestone_arg
            result = self.run_command([
                "gh", "api", f"repos/{self.owner}/{self.repo}/milestones",
                "--jq", f'.[] | select(.title == "{self.milestone_title}") | .number'
            ])

            milestone_num_str = result.stdout.strip()
            if not milestone_num_str:
                print(f"❌ Error: Milestone '{self.milestone_title}' not found", file=sys.stderr)
                sys.exit(1)

            self.milestone_num = int(milestone_num_str)

        print(f"📍 Milestone: {self.milestone_title} (#{self.milestone_num})")
        return self.milestone_title, self.milestone_num

    def fetch_milestone_issues(self) -> List[Dict]:
        """Fetch all issues in the milestone."""
        print("\n📋 Fetching issues in milestone...")

        result = self.run_command([
            "gh", "issue", "list",
            "--milestone", self.milestone_title,
            "--state", "all",
            "--json", "number,title,state,labels",
            "--jq", "sort_by(.number)"
        ])

        self.all_issues = json.loads(result.stdout)

        if not self.all_issues:
            print(f"❌ Error: No issues found in milestone '{self.milestone_title}'", file=sys.stderr)
            sys.exit(1)

        total = len(self.all_issues)
        open_count = sum(1 for i in self.all_issues if i["state"] == "OPEN")
        closed_count = total - open_count

        print(f"   Total issues: {total}")
        print(f"   Open: {open_count}")
        print(f"   Closed: {closed_count}")

        return self.all_issues

    def find_last_completed(self) -> Optional[Dict]:
        """Find the most recently closed issue."""
        print("\n🔍 Finding last completed issue...")

        closed_issues = [i for i in self.all_issues if i["state"] == "CLOSED"]

        if not closed_issues:
            print("⚠️  No closed issues found in milestone")
            print("   Starting from first open issue...")
            return None

        # Get highest issue number that's closed
        last_closed = max(closed_issues, key=lambda x: x["number"])

        print(f"   Last completed: #{last_closed['number']} - {last_closed['title']}")

        # Check if it has wip label
        labels = [label["name"] for label in last_closed.get("labels", [])]
        if "wip" in labels:
            print(f"   Removing 'wip' label from #{last_closed['number']}...")
            self.run_command([
                "gh", "issue", "edit", str(last_closed["number"]),
                "--remove-label", "wip"
            ], check=False)

        return last_closed

    def build_hierarchy(self):
        """Build parent-child relationship tree."""
        print("\n🌳 Building issue hierarchy...")

        # Check if gh-sub-issue extension is available
        result = self.run_command(
            ["gh", "extension", "list"],
            check=False
        )

        if "gh-sub-issue" not in result.stdout:
            print("⚠️  Warning: gh-sub-issue extension not installed")
            print("   Install with: gh extension install https://github.com/dlvhdr/gh-sub-issue")
            print("   Falling back to simple sequential ordering...")
            self.use_simple_ordering = True
            return

        # Build parent-child mappings
        for issue in self.all_issues:
            issue_num = issue["number"]

            # Get parent
            result = self.run_command([
                "gh", "sub-issue", "list", str(issue_num),
                "--relation", "parent",
                "--json", "number"
            ], check=False)

            if result.returncode == 0 and result.stdout.strip():
                try:
                    parents_data = json.loads(result.stdout)
                    parent_issues = parents_data.get("subIssues", []) if parents_data else []
                    if parent_issues:
                        parent_num = parent_issues[0]["number"]
                        self.parent_map[issue_num] = parent_num
                except (json.JSONDecodeError, KeyError, TypeError):
                    # Silently skip - parent relationship is optional and failures are non-critical
                    pass

            # Get children
            result = self.run_command([
                "gh", "sub-issue", "list", str(issue_num),
                "--relation", "children",
                "--json", "number"
            ], check=False)

            if result.returncode == 0 and result.stdout.strip():
                try:
                    children_data = json.loads(result.stdout)
                    child_issues = children_data.get("subIssues", []) if children_data else []
                    if child_issues:
                        self.children_map[issue_num] = [c["number"] for c in child_issues]
                except (json.JSONDecodeError, KeyError, TypeError) as e:
                    print(f"⚠️  Failed to parse children for issue {issue_num}: {type(e).__name__}: {e}", file=sys.stderr)

        print("   ✓ Hierarchy built successfully")

    def find_next_issue(self, last_closed: Optional[Dict]) -> Optional[Dict]:
        """Find next issue using intelligent traversal."""
        print("\n🎯 Finding next issue to work on...")

        if self.use_simple_ordering:
            # Simple fallback: just get next open issue after last closed
            open_issues = [i for i in self.all_issues if i["state"] == "OPEN"]
            open_issues.sort(key=lambda x: x["number"])

            if last_closed:
                next_issues = [i for i in open_issues if i["number"] > last_closed["number"]]
                next_issue = next_issues[0] if next_issues else (open_issues[0] if open_issues else None)
            else:
                next_issue = open_issues[0] if open_issues else None
        else:
            # Intelligent traversal: sibling-first depth-first search
            next_issue = None

            if last_closed:
                current_num = last_closed["number"]

                # Step 1: Check for next sibling
                parent_num = self.parent_map.get(current_num)
                if parent_num and parent_num in self.children_map:
                    siblings = self.children_map[parent_num]
                    try:
                        current_idx = siblings.index(current_num)
                        # Look for next open sibling
                        for sibling_num in siblings[current_idx + 1:]:
                            sibling = next((i for i in self.all_issues if i["number"] == sibling_num), None)
                            if sibling and sibling["state"] == "OPEN":
                                next_issue = sibling
                                break
                    except ValueError:
                        pass

                # Step 2: If no next sibling, traverse up to parent's next sibling
                if not next_issue and parent_num:
                    grandparent_num = self.parent_map.get(parent_num)
                    if grandparent_num and grandparent_num in self.children_map:
                        parent_siblings = self.children_map[grandparent_num]
                        try:
                            parent_idx = parent_siblings.index(parent_num)
                            for uncle_num in parent_siblings[parent_idx + 1:]:
                                uncle = next((i for i in self.all_issues if i["number"] == uncle_num), None)
                                if uncle and uncle["state"] == "OPEN":
                                    next_issue = uncle
                                    break
                        except ValueError:
                            pass

            # Step 3: Fallback - just get first open issue
            if not next_issue:
                open_issues = [i for i in self.all_issues if i["state"] == "OPEN"]
                open_issues.sort(key=lambda x: x["number"])
                next_issue = open_issues[0] if open_issues else None

        if not next_issue:
            print("🎉 No more open issues in milestone!")
            print("   All issues completed. Milestone ready for release.")
            return None

        print(f"   Next issue: #{next_issue['number']} - {next_issue['title']}")
        return next_issue

    def apply_ready_label(self, issue: Dict):
        """Apply 'ready' label to next issue."""
        print("\n🏷️  Updating labels...")

        result = self.run_command([
            "gh", "issue", "edit", str(issue["number"]),
            "--add-label", "ready"
        ], check=False)

        if result.returncode == 0:
            print(f"   ✓ Added 'ready' label to #{issue['number']}")
        else:
            print("   ⚠️  Could not add 'ready' label (may not exist in repo)")

    def create_worktree(self, issue: Dict) -> Tuple[str, str]:
        """Create worktree for next issue."""
        print(f"\n📂 Creating worktree for #{issue['number']}...")

        # Generate branch name and worktree path
        title = issue["title"]
        issue_slug = re.sub(r'[^a-z0-9 -]', '', title.lower())
        issue_slug = re.sub(r'\s+', '-', issue_slug)
        issue_slug = issue_slug.strip('-')[:50]

        if not issue_slug:
            issue_slug = "issue"

        username = "claude"
        branch_name = f"{username}/{issue['number']}-{issue_slug}"
        worktree_path = f"wip/{username}-{issue['number']}-{issue_slug}"

        # Check if worktree already exists
        if os.path.exists(worktree_path):
            print(f"⚠️  Worktree already exists at {worktree_path}")
            print("   To recreate:")
            print(f"     git worktree remove {worktree_path}")
            print("     python scripts/gh-milestones/prep-next.py")
            sys.exit(1)

        # Check if branch already exists
        local_check = self.run_command(
            ["git", "show-ref", "--verify", f"refs/heads/{branch_name}"],
            check=False
        )

        remote_check = self.run_command(
            ["git", "ls-remote", "--heads", "origin", branch_name],
            check=False
        )

        if local_check.returncode == 0 or (remote_check.returncode == 0 and remote_check.stdout.strip()):
            print(f"⚠️  Branch {branch_name} already exists")
            print("   To delete:")
            print(f"     git branch -D {branch_name}  # local")
            print(f"     git push origin --delete {branch_name}  # remote")
            sys.exit(1)

        # Fetch latest main
        print("   Fetching latest main...")
        self.run_command(["git", "fetch", "origin", "main"])

        # Create worktree
        result = self.run_command([
            "git", "worktree", "add",
            "-b", branch_name,
            worktree_path,
            "origin/main"
        ], check=False)

        if result.returncode != 0:
            print("❌ Error: Failed to create worktree", file=sys.stderr)
            sys.exit(1)

        print("   ✓ Worktree created successfully")

        # Update tmux window name if in tmux
        if os.environ.get("TMUX"):
            tmux_name = f"💻i{issue['number']}"
            self.run_command(
                ["tmux", "rename-window", tmux_name],
                check=False
            )
            print(f"   ✓ Tmux window renamed to: {tmux_name}")

        return branch_name, worktree_path

    def display_summary(self, issue: Dict, branch_name: str, worktree_path: str):
        """Display final summary and next steps."""
        print("\n" + "=" * 60)
        print("✅ Ready to Start Next Issue")
        print("=" * 60)
        print()
        print(f"📍 Milestone: {self.milestone_title} (#{self.milestone_num})")
        print(f"📋 Issue: #{issue['number']} - {issue['title']}")
        print(f"🌿 Branch: {branch_name}")
        print(f"📂 Worktree: {worktree_path}")
        print()
        print("=" * 60)
        print("📝 Issue Details")
        print("=" * 60)
        print()

        # Display issue body
        result = self.run_command([
            "gh", "issue", "view", str(issue["number"]),
            "--json", "body",
            "--jq", ".body // \"(No description provided)\""
        ])
        print(result.stdout.strip())

        print()
        print("=" * 60)
        print("🚀 Next Steps")
        print("=" * 60)
        print()
        print("1. Navigate to worktree:")
        print(f"   cd {worktree_path}")
        print()
        print("2. Review issue requirements and implement changes")
        print()
        print("3. When ready to submit:")
        print("   /pr-workflow")
        print()

    def run(self):
        """Execute the full workflow."""
        try:
            self.get_repo_info()
            self.detect_milestone()
            self.fetch_milestone_issues()
            last_closed = self.find_last_completed()
            self.build_hierarchy()
            next_issue = self.find_next_issue(last_closed)

            if not next_issue:
                return 0

            self.apply_ready_label(next_issue)
            branch_name, worktree_path = self.create_worktree(next_issue)
            self.display_summary(next_issue, branch_name, worktree_path)

            return 0
        except KeyboardInterrupt:
            print("\n\n⚠️  Interrupted by user", file=sys.stderr)
            return 130
        except SystemExit as e:
            # Return the exit code from SystemExit to preserve intended control flow
            return e.code if e.code is not None else 1
        except Exception as e:
            print(f"\n❌ Unexpected error: {type(e).__name__}: {e}", file=sys.stderr)
            print("\nFull traceback:", file=sys.stderr)
            traceback.print_exc()
            return 1


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Automate moving to next issue in a GitHub milestone workflow"
    )
    parser.add_argument(
        "milestone",
        nargs="?",
        help="Milestone name or number (optional, will auto-detect from current branch)"
    )

    args = parser.parse_args()

    workflow = MilestoneWorkflow(args.milestone)
    sys.exit(workflow.run())


if __name__ == "__main__":
    main()
