#!/usr/bin/env python3
"""
link_issue.py

Link an existing GitHub issue as a sub-task of another issue (parent-child relationship).

Usage:
    python link_issue.py --parent 92 --child 103
    python link_issue.py --parent 92 --child 103 --repo owner/repo
    python link_issue.py --parent 92 --child 103 --dry-run

Environment Variables:
    GITHUB_TOKEN or GH_TOKEN - GitHub token with repo scope (required)
"""

import argparse
import json
import os
import re
import subprocess
import sys
from typing import Optional


def parse_arguments():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="Link existing GitHub issue as sub-task of another issue"
    )
    parser.add_argument(
        "--parent",
        type=int,
        required=True,
        help="Parent issue number (the issue that will track the child)",
    )
    parser.add_argument(
        "--child",
        type=int,
        required=True,
        help="Child issue number (the sub-task to be tracked)",
    )
    parser.add_argument(
        "--repo",
        type=str,
        help="Repository in owner/name format (auto-detected if not provided)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without making changes",
    )
    return parser.parse_args()


def get_github_token() -> Optional[str]:
    """Get GitHub token from environment."""
    return os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")


def detect_repository() -> Optional[str]:
    """Auto-detect repository from git remote."""
    try:
        result = subprocess.run(
            ["git", "remote", "get-url", "origin"],
            capture_output=True,
            text=True,
            check=True,
        )
        remote_url = result.stdout.strip()

        # Parse GitHub URL
        # Supports: git@github.com:owner/repo.git and https://github.com/owner/repo.git
        if "github.com" in remote_url:
            parts = remote_url.replace(":", "/").replace(".git", "").split("/")
            if len(parts) >= 2:
                return f"{parts[-2]}/{parts[-1]}"
    except Exception:
        pass
    return None


def run_gh_command(args: list) -> Optional[dict]:
    """Run gh CLI command and return JSON output."""
    try:
        result = subprocess.run(
            ["gh"] + args,
            capture_output=True,
            text=True,
            check=True,
        )
        if result.stdout.strip():
            return json.loads(result.stdout)
        return None
    except subprocess.CalledProcessError as e:
        print(f"Error running gh command: {e}")
        if e.stderr:
            print(f"Error output: {e.stderr}")
        return None
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON: {e}")
        return None


def fetch_issue(repo: str, issue_number: int) -> Optional[dict]:
    """Fetch issue details using gh CLI."""
    result = run_gh_command(
        [
            "issue",
            "view",
            str(issue_number),
            "--repo",
            repo,
            "--json",
            "number,title,body,state,id",
        ]
    )
    return result


def link_issues(
    repo: str,
    parent_number: int,
    parent_body: str,
    child_number: int,
    child_title: str,
    dry_run: bool = False,
) -> bool:
    """Link child issue as sub-task of parent issue by updating parent's body with task list."""

    # Check if child is already referenced in parent body
    child_ref = f"#{child_number}"
    if child_ref in parent_body:
        print(f"Note: Child issue {child_ref} is already referenced in parent body")

        # Check if it's in a task list format
        task_pattern = f"- [ ] .*{child_ref}"
        if re.search(task_pattern, parent_body):
            print("  Already in task list format - relationship should exist")
            return True
        else:
            print("  Referenced but not in task list format")

    # Add child to parent's body as a task list item
    # Look for existing task list sections
    if "## Sub-Tasks" in parent_body or "## Subtasks" in parent_body:
        # Append to existing sub-tasks section
        section_pattern = r"(## Sub-?Tasks.*?\n)"
        match = re.search(section_pattern, parent_body, re.IGNORECASE)
        if match:
            insert_pos = match.end()
            new_task = f"- [ ] #{child_number} {child_title}\n"
            updated_body = parent_body[:insert_pos] + new_task + parent_body[insert_pos:]
        else:
            # Fallback: append at end
            updated_body = parent_body + f"\n\n## Sub-Tasks\n- [ ] #{child_number} {child_title}\n"
    else:
        # Create new sub-tasks section at the end
        updated_body = parent_body.rstrip() + f"\n\n## Sub-Tasks\n- [ ] #{child_number} {child_title}\n"

    if dry_run:
        print("[DRY RUN] Would update parent issue body:")
        print(f"  Add task: - [ ] #{child_number} {child_title}")
        return True

    # Update parent issue with new body
    try:
        # Write updated body to temp file
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False) as f:
            f.write(updated_body)
            temp_file = f.name

        try:
            result = subprocess.run(
                [
                    "gh",
                    "issue",
                    "edit",
                    str(parent_number),
                    "--repo",
                    repo,
                    "--body-file",
                    temp_file,
                ],
                capture_output=True,
                text=True,
                check=True,
            )

            print(f"✓ Successfully linked issues")
            print(f"  Updated parent #{parent_number} body with task list")
            print(f"  Added: - [ ] #{child_number} {child_title}")
            return True

        finally:
            os.unlink(temp_file)

    except subprocess.CalledProcessError as e:
        print(f"Error updating parent issue: {e}")
        if e.stderr:
            print(f"Error output: {e.stderr}")
        return False


def main():
    """Main entry point."""
    args = parse_arguments()

    # Check if gh CLI is authenticated (token not needed when using gh CLI)
    # We'll rely on gh CLI's authentication
    token = get_github_token()  # Optional - gh CLI handles auth

    # Auto-detect or use provided repository
    repo = args.repo or detect_repository()
    if not repo:
        print("Error: Could not detect repository")
        print("Please run from within a git repository or specify --repo owner/name")
        sys.exit(1)

    print(f"Repository: {repo}")
    print(f"Parent Issue: #{args.parent}")
    print(f"Child Issue: #{args.child}")
    print()

    # Fetch parent issue
    print("Fetching parent issue...")
    parent_issue = fetch_issue(repo, args.parent)
    if not parent_issue:
        print(f"Error: Could not fetch issue #{args.parent}")
        sys.exit(1)

    print(f"  #{parent_issue['number']}: {parent_issue['title']}")
    print(f"  State: {parent_issue['state']}")
    print()

    # Fetch child issue
    print("Fetching child issue...")
    child_issue = fetch_issue(repo, args.child)
    if not child_issue:
        print(f"Error: Could not fetch issue #{args.child}")
        sys.exit(1)

    print(f"  #{child_issue['number']}: {child_issue['title']}")
    print(f"  State: {child_issue['state']}")
    print()

    # Link issues
    if args.dry_run:
        print("[DRY RUN] Would link:")
    else:
        print("Linking issues...")

    success = link_issues(
        repo,
        parent_issue["number"],
        parent_issue["body"] or "",
        child_issue["number"],
        child_issue["title"],
        dry_run=args.dry_run,
    )

    if success:
        print()
        print("✓ Link operation completed successfully")
        if not args.dry_run:
            print(f"  View parent issue: https://github.com/{repo}/issues/{args.parent}")
            print(f"  View child issue: https://github.com/{repo}/issues/{args.child}")
        sys.exit(0)
    else:
        print()
        print("✗ Link operation failed")
        sys.exit(1)


if __name__ == "__main__":
    main()
