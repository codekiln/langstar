#!/usr/bin/env python3
"""
create_subissues.py

Break down GitHub issues into sub-issues by parsing task lists from parent issue descriptions.

Usage:
    python create_subissues.py --issue 42
    python create_subissues.py --issue 42 --repo owner/repo
    python create_subissues.py --issue 42 --dry-run

Environment Variables:
    GITHUB_TOKEN or GH_TOKEN - GitHub token with repo scope (required)

"""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from typing import Dict, List, Optional, Tuple


def main():
    """Main entry point for the script."""
    args = parse_arguments()

    # Get GitHub token
    token = get_github_token()
    if not token:
        print("Error: No GitHub token found in environment")
        print("Please set GITHUB_TOKEN or GH_TOKEN environment variable")
        print("Token must have 'repo' scope")
        sys.exit(1)

    # Auto-detect or use provided repository
    repo = args.repo or detect_repository()
    if not repo:
        print("Error: Could not detect repository")
        print("Please run from within a git repository or specify --repo owner/name")
        sys.exit(1)

    print(f"Repository: {repo}")
    print(f"Parent Issue: #{args.issue}")
    print()

    # Fetch parent issue
    print("Fetching parent issue...")
    parent_issue = fetch_issue(repo, args.issue, token)
    if not parent_issue:
        print(f"Error: Could not fetch issue #{args.issue}")
        sys.exit(1)

    print(f"  Title: {parent_issue['title']}")
    print()

    # Parse tasks from issue body
    tasks = parse_tasks(parent_issue['body'] or '')
    if not tasks:
        print("No tasks found in issue description.")
        print("Supported formats:")
        print("  - [ ] Task name")
        print("  1. Task name")
        print("  * Task name")
        sys.exit(0)

    print(f"Found {len(tasks)} task(s) to convert into sub-issues:")
    print()
    for i, task in enumerate(tasks, 1):
        print(f"{i}. {task}")
    print()

    # Prepare sub-issue data
    sub_issues = prepare_sub_issues(
        tasks,
        parent_issue,
        inherit_labels=args.inherit_labels,
        inherit_assignees=args.inherit_assignees
    )

    # Show what will be inherited
    if args.inherit_labels and parent_issue.get('labels'):
        label_names = [label['name'] for label in parent_issue['labels']]
        print(f"Labels to inherit: {', '.join(label_names)}")

    if args.inherit_assignees and parent_issue.get('assignees'):
        assignee_logins = [assignee['login'] for assignee in parent_issue['assignees']]
        print(f"Assignees to inherit: {', '.join(assignee_logins)}")

    if args.inherit_labels or args.inherit_assignees:
        print()

    # Dry-run mode
    if args.dry_run:
        print("DRY RUN MODE - No issues will be created")
        print("Preview of sub-issues that would be created:")
        print()
        for i, sub_issue in enumerate(sub_issues, 1):
            print(f"{i}. {sub_issue['title']}")
            if sub_issue.get('labels'):
                print(f"   Labels: {', '.join(sub_issue['labels'])}")
            if sub_issue.get('assignees'):
                print(f"   Assignees: {', '.join(sub_issue['assignees'])}")
        sys.exit(0)

    # Confirm with user
    if not args.yes:
        response = input(f"Create these {len(tasks)} sub-issue(s)? (y/n): ")
        if response.lower() != 'y':
            print("Cancelled")
            sys.exit(0)
    else:
        print(f"Auto-confirming creation of {len(tasks)} sub-issue(s) (--yes flag)")
        print()

    print()
    print("Creating sub-issues...")
    print()

    # Get repository node ID
    repo_id = get_repository_id(repo, token)
    if not repo_id:
        print(f"Error: Could not get repository node ID for {repo}")
        sys.exit(1)

    # Create sub-issues
    results = create_sub_issues(
        repo_id,
        parent_issue['id'],
        sub_issues,
        token
    )

    # Report results
    print()
    print("=" * 50)
    print("Summary:")
    print(f"  ✓ Successful: {results['success']}")
    if results['failed'] > 0:
        print(f"  ✗ Failed: {results['failed']}")
    print("=" * 50)

    # Exit with error code if any failed
    sys.exit(0 if results['failed'] == 0 else 1)


def parse_arguments():
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        description='Break down GitHub issues into sub-issues'
    )
    parser.add_argument(
        '--issue',
        type=int,
        required=True,
        help='Issue number to break down'
    )
    parser.add_argument(
        '--repo',
        type=str,
        help='Repository in owner/name format (auto-detected if not provided)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Preview mode without creating sub-issues'
    )
    parser.add_argument(
        '--inherit-labels',
        action='store_true',
        default=True,
        help='Inherit labels from parent issue (default: true)'
    )
    parser.add_argument(
        '--inherit-assignees',
        action='store_true',
        default=True,
        help='Inherit assignees from parent issue (default: true)'
    )
    parser.add_argument(
        '--yes',
        '-y',
        action='store_true',
        help='Automatically confirm creation without prompting'
    )
    return parser.parse_args()


def get_github_token() -> Optional[str]:
    """Get GitHub token from environment."""
    return os.environ.get('GITHUB_TOKEN') or os.environ.get('GH_TOKEN') or os.environ.get('GITHUB_PAT')


def detect_repository() -> Optional[str]:
    """Auto-detect repository from git context."""
    try:
        result = subprocess.run(
            ['gh', 'repo', 'view', '--json', 'nameWithOwner', '-q', '.nameWithOwner'],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return None


def run_gh_api(query: str, token: str, variables: Optional[Dict] = None) -> Optional[Dict]:
    """Execute a GraphQL query using gh CLI."""
    # Set token
    env = os.environ.copy()
    env['GH_TOKEN'] = token

    # Build GraphQL request
    graphql_request = {'query': query}
    if variables:
        graphql_request['variables'] = variables

    # Write to temp file and pass via --input
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(graphql_request, f)
        temp_file = f.name

    try:
        cmd = ['gh', 'api', 'graphql', '--input', temp_file]
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=True,
            env=env
        )
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"GraphQL Error: {e.stderr}")
        return None
    except json.JSONDecodeError:
        print("Error: Invalid JSON response from GitHub API")
        return None
    finally:
        # Clean up temp file
        try:
            os.unlink(temp_file)
        except:
            pass


def fetch_issue(repo: str, issue_number: int, token: str) -> Optional[Dict]:
    """Fetch issue details from GitHub."""
    owner, name = repo.split('/')

    query = """
    query($owner: String!, $name: String!, $number: Int!) {
      repository(owner: $owner, name: $name) {
        issue(number: $number) {
          id
          number
          title
          body
          labels(first: 10) {
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

    variables = {
        'owner': owner,
        'name': name,
        'number': issue_number
    }

    response = run_gh_api(query, token, variables)
    if not response or 'data' not in response:
        return None

    issue = response['data']['repository']['issue']

    # Flatten labels and assignees
    issue['labels'] = issue['labels']['nodes']
    issue['assignees'] = issue['assignees']['nodes']

    return issue


def parse_tasks(body: str) -> List[str]:
    """Parse task lists from issue body.

    Supports:
    - Markdown checkboxes: - [ ] Task or - [x] Task
    - Numbered lists: 1. Task
    - Bullet points: * Task or - Task
    """
    tasks = []

    # Split into lines
    lines = body.split('\n')

    for line in lines:
        line = line.strip()

        # Skip empty lines
        if not line:
            continue

        # Markdown checkbox (unchecked only by default)
        if re.match(r'^-\s*\[\s*\]', line):
            task = re.sub(r'^-\s*\[\s*\]\s*', '', line).strip()
            if task:
                tasks.append(task)

        # Numbered list
        elif re.match(r'^\d+\.\s+', line):
            task = re.sub(r'^\d+\.\s+', '', line).strip()
            if task:
                tasks.append(task)

        # Bullet point (but not checkbox)
        elif re.match(r'^[\*-]\s+(?!\[)', line):
            task = re.sub(r'^[\*-]\s+', '', line).strip()
            # Avoid duplicating checkbox items
            if task and not task.startswith('['):
                tasks.append(task)

    return tasks


def prepare_sub_issues(
    tasks: List[str],
    parent_issue: Dict,
    inherit_labels: bool = True,
    inherit_assignees: bool = True
) -> List[Dict]:
    """Prepare sub-issue data structures."""
    sub_issues = []

    for task in tasks:
        sub_issue = {
            'title': task,
            'body': f"Sub-task of #{parent_issue['number']}"
        }

        # Inherit labels
        if inherit_labels and parent_issue.get('labels'):
            sub_issue['labelIds'] = [label['id'] for label in parent_issue['labels']]
            sub_issue['labels'] = [label['name'] for label in parent_issue['labels']]

        # Inherit assignees
        if inherit_assignees and parent_issue.get('assignees'):
            sub_issue['assigneeIds'] = [assignee['id'] for assignee in parent_issue['assignees']]
            sub_issue['assignees'] = [assignee['login'] for assignee in parent_issue['assignees']]

        sub_issues.append(sub_issue)

    return sub_issues


def get_repository_id(repo: str, token: str) -> Optional[str]:
    """Get repository node ID."""
    owner, name = repo.split('/')

    query = """
    query($owner: String!, $name: String!) {
      repository(owner: $owner, name: $name) {
        id
      }
    }
    """

    variables = {
        'owner': owner,
        'name': name
    }

    response = run_gh_api(query, token, variables)
    if not response or 'data' not in response:
        return None

    return response['data']['repository']['id']


def create_sub_issue(
    repo_id: str,
    parent_id: str,
    sub_issue: Dict,
    token: str
) -> Tuple[bool, Optional[str], Optional[str]]:
    """Create a single sub-issue.

    Returns:
        (success, url, error_message)
    """
    # Build mutation dynamically based on what fields we have
    label_ids = sub_issue.get('labelIds', [])
    assignee_ids = sub_issue.get('assigneeIds', [])

    # Build the mutation with optional fields
    mutation_parts = ['$repoId: ID!', '$title: String!', '$body: String', '$parentId: ID!']
    input_parts = ['repositoryId: $repoId', 'title: $title', 'body: $body', 'parentIssueId: $parentId']

    if label_ids:
        mutation_parts.append('$labelIds: [ID!]')
        input_parts.append('labelIds: $labelIds')

    if assignee_ids:
        mutation_parts.append('$assigneeIds: [ID!]')
        input_parts.append('assigneeIds: $assigneeIds')

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

    variables = {
        'repoId': repo_id,
        'title': sub_issue['title'],
        'body': sub_issue.get('body', ''),
        'parentId': parent_id
    }

    if label_ids:
        variables['labelIds'] = label_ids

    if assignee_ids:
        variables['assigneeIds'] = assignee_ids

    response = run_gh_api(mutation, token, variables)
    if not response or 'data' not in response:
        error_msg = "Failed to create sub-issue"
        if response and 'errors' in response:
            error_msg = response['errors'][0].get('message', error_msg)
        return False, None, error_msg

    created = response['data']['createIssue']['issue']
    return True, created['url'], None


def create_sub_issues(
    repo_id: str,
    parent_id: str,
    sub_issues: List[Dict],
    token: str
) -> Dict:
    """Create multiple sub-issues and return results."""
    results = {
        'success': 0,
        'failed': 0,
        'created': []
    }

    for i, sub_issue in enumerate(sub_issues, 1):
        print(f"Creating sub-issue {i}/{len(sub_issues)}: {sub_issue['title']}")

        success, url, error = create_sub_issue(repo_id, parent_id, sub_issue, token)

        if success:
            print(f"  ✓ Created: {url}")
            results['success'] += 1
            results['created'].append(url)
        else:
            print(f"  ✗ Failed: {error}")
            results['failed'] += 1

    return results


if __name__ == '__main__':
    main()
