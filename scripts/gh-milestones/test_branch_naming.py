#!/usr/bin/env python3
"""
Tests for branch naming convention in prep-next.py.

Tests the branch name generation logic that creates names in the format:
- m<milestone_id>-p<parent_id>-i<issue_num>-<slug> (full)
- p<parent_id>-i<issue_num>-<slug> (parent only)
- m<milestone_id>-i<issue_num>-<slug> (milestone only)
- i<issue_num>-<slug> (standalone)
"""

import sys
import re


def generate_branch_name(
    issue_num: int,
    issue_slug: str,
    milestone_id: int = None,
    parent_id: int = None,
) -> str:
    """
    Generate branch name following project conventions.

    This function mirrors the logic in prep-next.py:create_worktree().

    Args:
        issue_num: GitHub issue number
        issue_slug: URL-safe slug from issue title
        milestone_id: Optional milestone number
        parent_id: Optional parent issue number

    Returns:
        Branch name in format: [m<id>-][p<id>-]i<num>-<slug>
    """
    branch_parts = []
    if milestone_id:
        branch_parts.append(f"m{milestone_id}")
    if parent_id:
        branch_parts.append(f"p{parent_id}")
    branch_parts.append(f"i{issue_num}")
    branch_parts.append(issue_slug)

    return "-".join(branch_parts)


def test_standalone_issue():
    """Test branch name for standalone issue (no milestone or parent)."""
    print("Test 1: Standalone issue (no milestone or parent)")
    print("-" * 60)

    result = generate_branch_name(
        issue_num=234,
        issue_slug="add-user-auth"
    )
    expected = "i234-add-user-auth"

    if result == expected:
        print(f"✓ PASS: Generated '{result}'")
        return True
    else:
        print(f"✗ FAIL: Expected '{expected}', got '{result}'")
        return False


def test_issue_with_milestone():
    """Test branch name for issue with milestone but no parent."""
    print("\nTest 2: Issue with milestone only")
    print("-" * 60)

    result = generate_branch_name(
        issue_num=234,
        issue_slug="add-user-auth",
        milestone_id=8
    )
    expected = "m8-i234-add-user-auth"

    if result == expected:
        print(f"✓ PASS: Generated '{result}'")
        return True
    else:
        print(f"✗ FAIL: Expected '{expected}', got '{result}'")
        return False


def test_issue_with_parent():
    """Test branch name for issue with parent but no milestone."""
    print("\nTest 3: Issue with parent only")
    print("-" * 60)

    result = generate_branch_name(
        issue_num=234,
        issue_slug="add-user-auth",
        parent_id=123
    )
    expected = "p123-i234-add-user-auth"

    if result == expected:
        print(f"✓ PASS: Generated '{result}'")
        return True
    else:
        print(f"✗ FAIL: Expected '{expected}', got '{result}'")
        return False


def test_issue_with_milestone_and_parent():
    """Test branch name for issue with both milestone and parent."""
    print("\nTest 4: Issue with both milestone and parent")
    print("-" * 60)

    result = generate_branch_name(
        issue_num=234,
        issue_slug="add-user-auth",
        milestone_id=8,
        parent_id=123
    )
    expected = "m8-p123-i234-add-user-auth"

    if result == expected:
        print(f"✓ PASS: Generated '{result}'")
        return True
    else:
        print(f"✗ FAIL: Expected '{expected}', got '{result}'")
        return False


def test_extract_issue_from_standalone():
    """Test extracting issue number from standalone format."""
    print("\nTest 5: Extract issue from standalone format")
    print("-" * 60)

    branch = "i234-add-user-auth"
    match = re.search(r'i(\d+)-', branch)

    if match and match.group(1) == "234":
        print(f"✓ PASS: Extracted issue #234 from '{branch}'")
        return True
    else:
        print(f"✗ FAIL: Could not extract issue from '{branch}'")
        return False


def test_extract_issue_from_full_format():
    """Test extracting issue number from full format."""
    print("\nTest 6: Extract issue from full format")
    print("-" * 60)

    branch = "m8-p123-i234-add-user-auth"
    match = re.search(r'i(\d+)-', branch)

    if match and match.group(1) == "234":
        print(f"✓ PASS: Extracted issue #234 from '{branch}'")
        return True
    else:
        print(f"✗ FAIL: Could not extract issue from '{branch}'")
        return False


def test_extract_milestone_and_parent():
    """Test extracting milestone and parent from branch name."""
    print("\nTest 7: Extract milestone and parent")
    print("-" * 60)

    branch = "m8-p123-i234-add-user-auth"

    milestone_match = re.search(r'm(\d+)-', branch)
    parent_match = re.search(r'p(\d+)-', branch)
    issue_match = re.search(r'i(\d+)-', branch)

    success = True
    if milestone_match and milestone_match.group(1) == "8":
        print(f"✓ Extracted milestone #8")
    else:
        print(f"✗ Failed to extract milestone")
        success = False

    if parent_match and parent_match.group(1) == "123":
        print(f"✓ Extracted parent #123")
    else:
        print(f"✗ Failed to extract parent")
        success = False

    if issue_match and issue_match.group(1) == "234":
        print(f"✓ Extracted issue #234")
    else:
        print(f"✗ Failed to extract issue")
        success = False

    if success:
        print(f"✓ PASS: All components extracted from '{branch}'")
    else:
        print(f"✗ FAIL: Some components failed to extract")

    return success


def test_worktree_path_generation():
    """Worktree paths land under the root checkout, even when run from a linked worktree."""
    print("\nTest 8: Worktree path generation")
    print("-" * 60)

    import importlib.util
    import os
    import subprocess
    import tempfile

    spec = importlib.util.spec_from_file_location(
        "prep_next", os.path.join(os.path.dirname(os.path.abspath(__file__)), "prep-next.py")
    )
    prep_next = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(prep_next)

    branch = generate_branch_name(234, "add-user-auth", milestone_id=8, parent_id=123)

    with tempfile.TemporaryDirectory() as tmp:
        root = os.path.realpath(os.path.join(tmp, "repo"))
        os.makedirs(root)
        git = lambda *args, cwd=root: subprocess.run(
            ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
        )
        git("init", "-q", "-b", "main")
        git("-c", "user.name=t", "-c", "user.email=t@example.com", "commit", "-q", "--allow-empty", "-m", "init")
        linked = os.path.join(root, ".worktrees", "existing")
        git("worktree", "add", "-q", "-b", "existing", linked)

        expected = os.path.join(root, ".worktrees", branch)
        original_cwd = os.getcwd()
        results = {}
        try:
            for label, cwd in (("root checkout", root), ("linked worktree", linked)):
                os.chdir(cwd)
                results[label] = os.path.realpath(os.path.dirname(prep_next.worktree_path_for(branch))) + "/" + branch
        finally:
            os.chdir(original_cwd)

    ok = True
    for label, got in results.items():
        if got == expected:
            print(f"✓ PASS: from the {label}, path is '{got}'")
        else:
            print(f"✗ FAIL: from the {label}, expected '{expected}', got '{got}'")
            ok = False
    return ok


def main():
    """Run all tests."""
    print("=" * 60)
    print("Testing Branch Naming Convention")
    print("=" * 60)

    tests = [
        test_standalone_issue,
        test_issue_with_milestone,
        test_issue_with_parent,
        test_issue_with_milestone_and_parent,
        test_extract_issue_from_standalone,
        test_extract_issue_from_full_format,
        test_extract_milestone_and_parent,
        test_worktree_path_generation,
    ]

    results = [test() for test in tests]

    print("\n" + "=" * 60)
    if all(results):
        print(f"✓ All {len(results)} tests passed!")
        print("=" * 60)
        return 0
    else:
        failed = len([r for r in results if not r])
        print(f"✗ {failed} of {len(results)} tests failed")
        print("=" * 60)
        return 1


if __name__ == "__main__":
    sys.exit(main())
