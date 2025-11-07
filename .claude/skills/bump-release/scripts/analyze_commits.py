#!/usr/bin/env python3
"""
Analyze git commits since last release tag to determine semantic version bump.
Parses Conventional Emoji Commits format.
"""

import re
import subprocess
import sys
from enum import Enum
from typing import List, Tuple, Optional


class BumpType(Enum):
    """Semantic version bump types"""
    NONE = 0
    PATCH = 1
    MINOR = 2
    MAJOR = 3


class CommitParser:
    """Parse Conventional Emoji Commits"""

    # Conventional Emoji Commit patterns
    # Format: <emoji> <type>[optional scope]: <description>
    COMMIT_PATTERN = re.compile(
        r'^[\U0001F300-\U0001FAFF\u2600-\u26FF\u2700-\u27BF]?\s*'  # Optional emoji
        r'(\w+)'  # type (required)
        r'(?:\([^)]+\))?'  # optional scope in parentheses
        r'!?'  # optional breaking change indicator
        r':\s*'  # colon separator
        r'(.+)'  # description
    , re.UNICODE)

    # Breaking change indicators
    BREAKING_PATTERNS = [
        re.compile(r'^BREAKING[- ]CHANGE:', re.MULTILINE | re.IGNORECASE),
        re.compile(r'^\U0001F6A8\s', re.MULTILINE),  # 🚨 emoji
        re.compile(r'^\U0001F4A5\s', re.MULTILINE),  # 💥 emoji
    ]

    # Commit type to bump type mapping
    TYPE_TO_BUMP = {
        'feat': BumpType.MINOR,
        'feature': BumpType.MINOR,
        'fix': BumpType.PATCH,
        'bugfix': BumpType.PATCH,
        'perf': BumpType.PATCH,
        'docs': BumpType.NONE,
        'style': BumpType.NONE,
        'refactor': BumpType.NONE,
        'test': BumpType.NONE,
        'build': BumpType.NONE,
        'ci': BumpType.NONE,
        'chore': BumpType.NONE,
        'revert': BumpType.PATCH,
    }

    @classmethod
    def parse_commit(cls, commit_msg: str) -> BumpType:
        """
        Parse a commit message and determine the bump type.

        Args:
            commit_msg: Full commit message (subject + body)

        Returns:
            BumpType indicating the semantic version bump
        """
        # Check for breaking changes first (highest priority)
        for pattern in cls.BREAKING_PATTERNS:
            if pattern.search(commit_msg):
                return BumpType.MAJOR

        # Parse commit type from first line
        first_line = commit_msg.split('\n')[0].strip()
        match = cls.COMMIT_PATTERN.match(first_line)

        if not match:
            # Non-conventional commit, treat as NONE
            return BumpType.NONE

        commit_type = match.group(1).lower()

        # Check for breaking change indicator (!)
        if '!' in first_line and '!:' not in first_line:
            return BumpType.MAJOR

        # Return bump type based on commit type
        return cls.TYPE_TO_BUMP.get(commit_type, BumpType.NONE)


def get_last_tag() -> Optional[str]:
    """
    Get the most recent git tag.

    Returns:
        Tag name or None if no tags exist
    """
    try:
        result = subprocess.run(
            ['git', 'describe', '--tags', '--abbrev=0'],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return None


def get_commits_since_tag(tag: Optional[str] = None) -> List[str]:
    """
    Get commit messages since the specified tag (or all commits if no tag).

    Args:
        tag: Git tag to compare against, or None for all commits

    Returns:
        List of full commit messages
    """
    if tag:
        range_spec = f'{tag}..HEAD'
    else:
        range_spec = 'HEAD'

    try:
        result = subprocess.run(
            ['git', 'log', range_spec, '--pretty=format:%B%n---COMMIT-SEPARATOR---'],
            capture_output=True,
            text=True,
            check=True
        )

        # Split commits and filter empty ones
        commits = [
            c.strip()
            for c in result.stdout.split('---COMMIT-SEPARATOR---')
            if c.strip()
        ]
        return commits
    except subprocess.CalledProcessError as e:
        print(f"Error fetching commits: {e}", file=sys.stderr)
        return []


def determine_bump_type(commits: List[str]) -> BumpType:
    """
    Analyze commits and determine the highest semantic version bump needed.

    Args:
        commits: List of commit messages

    Returns:
        Highest BumpType needed (MAJOR > MINOR > PATCH > NONE)
    """
    highest_bump = BumpType.NONE

    for commit in commits:
        bump = CommitParser.parse_commit(commit)
        if bump.value > highest_bump.value:
            highest_bump = bump
            # Short-circuit if we find a breaking change
            if highest_bump == BumpType.MAJOR:
                break

    return highest_bump


def parse_version(version_str: str) -> Tuple[int, int, int]:
    """
    Parse a semantic version string.

    Args:
        version_str: Version string (e.g., "1.2.3" or "v1.2.3")

    Returns:
        Tuple of (major, minor, patch)
    """
    # Strip 'v' prefix if present
    version_str = version_str.lstrip('v')

    # Split and parse
    parts = version_str.split('.')
    if len(parts) != 3:
        raise ValueError(f"Invalid version format: {version_str}")

    return tuple(int(p) for p in parts)


def bump_version(current: str, bump_type: BumpType) -> str:
    """
    Calculate the new version based on bump type.

    Args:
        current: Current version string
        bump_type: Type of bump to apply

    Returns:
        New version string (without 'v' prefix)
    """
    major, minor, patch = parse_version(current)

    if bump_type == BumpType.MAJOR:
        return f"{major + 1}.0.0"
    elif bump_type == BumpType.MINOR:
        return f"{major}.{minor + 1}.0"
    elif bump_type == BumpType.PATCH:
        return f"{major}.{minor}.{patch + 1}"
    else:
        return f"{major}.{minor}.{patch}"


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description='Analyze commits and determine semantic version bump'
    )
    parser.add_argument(
        '--current-version',
        help='Current version (e.g., 1.2.3 or v1.2.3)',
        default=None
    )
    parser.add_argument(
        '--format',
        choices=['bump-type', 'new-version', 'json'],
        default='bump-type',
        help='Output format'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Show detailed analysis'
    )

    args = parser.parse_args()

    # Fetch last tag
    last_tag = get_last_tag()
    current_version = args.current_version or last_tag or '0.0.0'

    if args.verbose:
        print(f"Last tag: {last_tag or 'None'}", file=sys.stderr)
        print(f"Current version: {current_version}", file=sys.stderr)

    # Get commits since last tag
    commits = get_commits_since_tag(last_tag)

    if not commits:
        print("No commits found since last release", file=sys.stderr)
        sys.exit(1)

    if args.verbose:
        print(f"Found {len(commits)} commits", file=sys.stderr)

    # Determine bump type
    bump_type = determine_bump_type(commits)

    if args.verbose:
        print(f"Determined bump type: {bump_type.name}", file=sys.stderr)

    # Output based on format
    if args.format == 'bump-type':
        print(bump_type.name.lower())
    elif args.format == 'new-version':
        new_version = bump_version(current_version, bump_type)
        print(new_version)
    elif args.format == 'json':
        import json
        new_version = bump_version(current_version, bump_type)
        output = {
            'current_version': current_version.lstrip('v'),
            'bump_type': bump_type.name.lower(),
            'new_version': new_version,
            'commit_count': len(commits),
            'last_tag': last_tag
        }
        print(json.dumps(output, indent=2))

    # Exit with code based on bump type (useful for automation)
    sys.exit(bump_type.value)


if __name__ == '__main__':
    main()
