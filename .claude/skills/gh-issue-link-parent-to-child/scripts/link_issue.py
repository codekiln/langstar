#!/usr/bin/env python3
"""
link_issue.py

Link an existing GitHub issue as a sub-task of another issue (parent-child relationship).

IMPORTANT: GitHub's API only supports setting parentIssueId during issue creation,
NOT when updating existing issues. This script closes and recreates the child issue
with the same content but with parentIssueId set.

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
import subprocess
import sys
from typing import Optional, Dict, Tuple


def parse_arguments():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="Link existing GitHub issue as sub-task of another issue (closes and recreates child)"
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
        help="Child issue number (will be closed and recreated)",
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


def run_gh_api(query: str, token: str, variables: Optional[Dict] = None) -> Optional[Dict]:
    """Run GitHub GraphQL API query."""
    try:
        cmd = ["gh", "api", "graphql", "-f", f"query={query}"]

        if variables:
            for key, value in variables.items():
                if isinstance(value, list):
                    cmd.extend(["-f", f"{key}={json.dumps(value)}"])
                else:
                    cmd.extend(["-f", f"{key}={value}"])

        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True,
        )

        return json.loads(result.stdout) if result.stdout.strip() else None
    except subprocess.CalledProcessError as e:
        print(f"Error running GraphQL query: {e}")
        if e.stderr:
            print(f"Error output: {e.stderr}")
        return None
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON: {e}")
        return None


def fetch_repository_id(repo: str, token: str) -> Optional[str]:
    """Fetch repository GraphQL node ID."""
    owner, name = repo.split("/")
    query = """
    query($owner: String!, $name: String!) {
      repository(owner: $owner, name: $name) {
        id
      }
    }
    """

    response = run_gh_api(query, token, {"owner": owner, "name": name})
    if response and "data" in response and response["data"]["repository"]:
        return response["data"]["repository"]["id"]
    return None


def fetch_issue_details(repo: str, issue_number: int, token: str) -> Optional[Dict]:
    """Fetch full issue details including GraphQL node ID."""
    owner, name = repo.split("/")
    query = """
    query($owner: String!, $name: String!, $number: Int!) {
      repository(owner: $owner, name: $name) {
        issue(number: $number) {
          id
          number
          title
          body
          state
          labels(first: 100) {
            nodes {
              id
              name
            }
          }
          assignees(first: 10) {
            nodes {
              id
              login
            }
          }
        }
      }
    }
    """

    response = run_gh_api(query, token, {"owner": owner, "name": name, "number": issue_number})
    if response and "data" in response and response["data"]["repository"]["issue"]:
        return response["data"]["repository"]["issue"]
    return None


def close_issue(repo: str, issue_number: int, comment: str) -> bool:
    """Close an issue with a comment."""
    try:
        # Add comment first
        subprocess.run(
            ["gh", "issue", "comment", str(issue_number), "--repo", repo, "--body", comment],
            capture_output=True,
            text=True,
            check=True,
        )

        # Then close
        subprocess.run(
            ["gh", "issue", "close", str(issue_number), "--repo", repo],
            capture_output=True,
            text=True,
            check=True,
        )

        return True
    except subprocess.CalledProcessError as e:
        print(f"Error closing issue: {e}")
        if e.stderr:
            print(f"Error output: {e.stderr}")
        return False


def create_issue_with_parent(
    repo_id: str,
    parent_id: str,
    title: str,
    body: str,
    label_ids: list,
    assignee_ids: list,
    token: str
) -> Tuple[bool, Optional[int], Optional[str]]:
    """Create a new issue with parent relationship."""

    # Build mutation with optional fields
    mutation_parts = ['$repoId: ID!', '$parentId: ID!', '$title: String!', '$body: String']
    input_parts = ['repositoryId: $repoId', 'parentIssueId: $parentId', 'title: $title', 'body: $body']

    variables = {
        'repoId': repo_id,
        'parentId': parent_id,
        'title': title,
        'body': body or ''
    }

    if label_ids:
        mutation_parts.append('$labelIds: [ID!]')
        input_parts.append('labelIds: $labelIds')
        variables['labelIds'] = label_ids

    if assignee_ids:
        mutation_parts.append('$assigneeIds: [ID!]')
        input_parts.append('assigneeIds: $assigneeIds')
        variables['assigneeIds'] = assignee_ids

    mutation = f"""
    mutation({', '.join(mutation_parts)}) {{
      createIssue(input: {{
        {', '.join(input_parts)}
      }}) {{
        issue {{
          id
          number
          title
          url
        }}
      }}
    }}
    """

    response = run_gh_api(mutation, token, variables)
    if not response or 'data' not in response:
        error_msg = "Failed to create issue"
        if response and 'errors' in response:
            error_msg = response['errors'][0].get('message', error_msg)
        return False, None, error_msg

    created = response['data']['createIssue']['issue']
    return True, created['number'], created['url']


def main():
    """Main entry point."""
    args = parse_arguments()

    # Get GitHub token
    token = get_github_token()
    if not token:
        print("Error: No GitHub token found")
        print("Please set GITHUB_TOKEN or GH_TOKEN environment variable")
        sys.exit(1)

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

    # Fetch repository ID
    print("Fetching repository ID...")
    repo_id = fetch_repository_id(repo, token)
    if not repo_id:
        print("Error: Could not fetch repository ID")
        sys.exit(1)

    # Fetch parent issue
    print("Fetching parent issue...")
    parent_issue = fetch_issue_details(repo, args.parent, token)
    if not parent_issue:
        print(f"Error: Could not fetch issue #{args.parent}")
        sys.exit(1)

    print(f"  #{parent_issue['number']}: {parent_issue['title']}")
    print(f"  State: {parent_issue['state']}")
    print()

    # Fetch child issue
    print("Fetching child issue...")
    child_issue = fetch_issue_details(repo, args.child, token)
    if not child_issue:
        print(f"Error: Could not fetch issue #{args.child}")
        sys.exit(1)

    print(f"  #{child_issue['number']}: {child_issue['title']}")
    print(f"  State: {child_issue['state']}")
    print()

    # Extract label and assignee IDs
    label_ids = [label['id'] for label in child_issue['labels']['nodes']]
    assignee_ids = [assignee['id'] for assignee in child_issue['assignees']['nodes']]

    if args.dry_run:
        print("[DRY RUN] Would perform:")
        print(f"  1. Close issue #{args.child}")
        print(f"  2. Recreate with same content but parentIssueId set to #{args.parent}")
        print(f"  3. Preserve {len(label_ids)} labels and {len(assignee_ids)} assignees")
        sys.exit(0)

    # Warn user
    print("⚠️  WARNING: This will close and recreate the child issue!")
    print(f"    Issue #{args.child} will be closed and a new issue created.")
    print(f"    Content, labels, and assignees will be preserved.")
    print()

    # Close the child issue
    print(f"Closing issue #{args.child}...")
    close_comment = f"""This issue is being closed and recreated to establish a parent-child relationship with #{args.parent}.

GitHub's API only supports setting `parentIssueId` during issue creation, not when updating existing issues.

The issue will be recreated with the same content, labels, and assignees."""

    if not close_issue(repo, args.child, close_comment):
        print("Error: Failed to close child issue")
        sys.exit(1)

    print(f"  ✓ Closed issue #{args.child}")
    print()

    # Recreate with parent relationship
    print(f"Creating new issue as sub-task of #{args.parent}...")

    # Update body to reference old issue
    updated_body = f"""*[Recreated from #{args.child} to establish parent-child relationship]*

{child_issue['body'] or ''}"""

    success, new_number, url = create_issue_with_parent(
        repo_id,
        parent_issue['id'],
        child_issue['title'],
        updated_body,
        label_ids,
        assignee_ids,
        token
    )

    if not success:
        print(f"Error: Failed to create new issue: {url}")
        sys.exit(1)

    print(f"  ✓ Created issue #{new_number}")
    print(f"  URL: {url}")
    print()

    print("✓ Link operation completed successfully")
    print(f"  Old issue #{args.child} closed")
    print(f"  New issue #{new_number} created as sub-task of #{args.parent}")
    print(f"  View parent: https://github.com/{repo}/issues/{args.parent}")
    print(f"  View child: https://github.com/{repo}/issues/{new_number}")

    sys.exit(0)


if __name__ == "__main__":
    main()
