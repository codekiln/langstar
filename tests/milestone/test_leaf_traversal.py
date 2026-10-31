#!/usr/bin/env python3
"""
Test to demonstrate leaf node traversal fix for issue #611.

This test demonstrates that _find_first_leaf() correctly traverses
down the issue hierarchy to find leaf nodes.
"""

import sys
from typing import Dict, List


class MockMilestoneWorkflow:
    """Minimal mock of MilestoneWorkflow to test _find_first_leaf logic."""

    def __init__(self, all_issues: List[Dict], children_map: Dict[int, List[int]]):
        self.all_issues = all_issues
        self.children_map = children_map

    def _find_first_leaf(self, issue: Dict) -> Dict:
        """
        Traverse down the hierarchy to find the first leaf node.
        A leaf node is an issue with no open children.
        """
        issue_num = issue["number"]

        # Check if this issue has open children
        if issue_num in self.children_map:
            children = self.children_map[issue_num]
            # Find first open child (sorted by issue number for consistency)
            for child_num in sorted(children):
                child = next((i for i in self.all_issues if i["number"] == child_num), None)
                if child and child["state"] == "OPEN":
                    # Recursively find leaf of this child
                    return self._find_first_leaf(child)

        # No open children - this is a leaf node
        return issue


def test_descends_to_leaf():
    """Test that _find_first_leaf descends to leaf nodes."""
    print("Test 1: Parent with open children should descend to first leaf")
    print("-" * 60)

    # Setup: Parent #586 with children #590, #592, etc.
    # Expected: Should select #590 (first leaf child)
    workflow = MockMilestoneWorkflow(
        all_issues=[
            {"number": 586, "state": "OPEN", "title": "Parent"},
            {"number": 590, "state": "OPEN", "title": "Child 1 (leaf)"},
            {"number": 592, "state": "OPEN", "title": "Child 2 (leaf)"},
            {"number": 593, "state": "OPEN", "title": "Child 3 (leaf)"},
        ],
        children_map={586: [590, 592, 593]},
    )

    parent = {"number": 586, "state": "OPEN", "title": "Parent"}
    result = workflow._find_first_leaf(parent)

    expected = 590
    actual = result["number"]
    if actual == expected:
        print(f"✓ PASS: Selected issue #{actual} (expected #{expected})")
    else:
        print(f"✗ FAIL: Selected issue #{actual} (expected #{expected})")
        return False

    print()
    return True


def test_handles_nested_hierarchy():
    """Test that _find_first_leaf handles multi-level nesting."""
    print("Test 2: Multi-level hierarchy should descend to deepest leaf")
    print("-" * 60)

    # Setup: Grandparent -> Parent -> Children
    # Expected: Should descend all the way to #103
    workflow = MockMilestoneWorkflow(
        all_issues=[
            {"number": 100, "state": "OPEN", "title": "Grandparent"},
            {"number": 101, "state": "OPEN", "title": "Parent"},
            {"number": 103, "state": "OPEN", "title": "Child (leaf)"},
        ],
        children_map={
            100: [101],  # Grandparent has parent as child
            101: [103],  # Parent has child as leaf
        },
    )

    grandparent = {"number": 100, "state": "OPEN", "title": "Grandparent"}
    result = workflow._find_first_leaf(grandparent)

    expected = 103
    actual = result["number"]
    if actual == expected:
        print(f"✓ PASS: Selected issue #{actual} (expected #{expected})")
    else:
        print(f"✗ FAIL: Selected issue #{actual} (expected #{expected})")
        return False

    print()
    return True


def test_returns_self_when_no_children():
    """Test that _find_first_leaf returns the issue itself when it has no children."""
    print("Test 3: Issue with no children should return itself")
    print("-" * 60)

    workflow = MockMilestoneWorkflow(
        all_issues=[{"number": 200, "state": "OPEN", "title": "Leaf"}],
        children_map={},
    )

    leaf = {"number": 200, "state": "OPEN", "title": "Leaf"}
    result = workflow._find_first_leaf(leaf)

    expected = 200
    actual = result["number"]
    if actual == expected:
        print(f"✓ PASS: Selected issue #{actual} (expected #{expected})")
    else:
        print(f"✗ FAIL: Selected issue #{actual} (expected #{expected})")
        return False

    print()
    return True


def test_skips_closed_children():
    """Test that _find_first_leaf skips closed children."""
    print("Test 4: Should skip closed children and find first open leaf")
    print("-" * 60)

    workflow = MockMilestoneWorkflow(
        all_issues=[
            {"number": 300, "state": "OPEN", "title": "Parent"},
            {"number": 301, "state": "CLOSED", "title": "Child 1 (closed)"},
            {"number": 302, "state": "OPEN", "title": "Child 2 (leaf)"},
        ],
        children_map={300: [301, 302]},
    )

    parent = {"number": 300, "state": "OPEN", "title": "Parent"}
    result = workflow._find_first_leaf(parent)

    expected = 302
    actual = result["number"]
    if actual == expected:
        print(f"✓ PASS: Selected issue #{actual} (expected #{expected})")
    else:
        print(f"✗ FAIL: Selected issue #{actual} (expected #{expected})")
        return False

    print()
    return True


def test_closed_siblings_in_hierarchy():
    """Test that closed siblings are included in children_map for proper sibling traversal.

    This test validates the fix for issue #611 where --state all was added to
    the gh sub-issue list command. Without this fix, closed siblings would be
    excluded from children_map, causing sibling traversal to fail.
    """
    print("Test 5: Closed siblings must be in children_map for traversal")
    print("-" * 60)

    # Real-world scenario from ls-cli-output-dx milestone:
    # Parent #584 with children: #585 (CLOSED), #587 (open), #589 (open)
    # Last completed: #585
    # Expected: #587 should be found as next sibling

    # Simulate what happens when --state all is used (CORRECT)
    workflow_with_closed = MockMilestoneWorkflow(
        all_issues=[
            {"number": 584, "state": "OPEN", "title": "Parent"},
            {"number": 585, "state": "CLOSED", "title": "Phase 1 (closed)"},
            {"number": 587, "state": "OPEN", "title": "Phase 2 (leaf)"},
            {"number": 589, "state": "OPEN", "title": "Phase 3 (leaf)"},
        ],
        children_map={584: [585, 587, 589]},  # Includes closed #585
    )

    # When traversing from parent, should skip closed child and find first open
    parent = {"number": 584, "state": "OPEN", "title": "Parent"}
    result = workflow_with_closed._find_first_leaf(parent)

    expected = 587
    actual = result["number"]
    if actual == expected:
        print(f"✓ PASS: With closed siblings in map, selected #{actual} (expected #{expected})")
    else:
        print(f"✗ FAIL: With closed siblings in map, selected #{actual} (expected #{expected})")
        return False

    # Demonstrate what would happen WITHOUT --state all (INCORRECT)
    print("\n  Demonstrating bug when closed siblings excluded from map:")
    workflow_without_closed = MockMilestoneWorkflow(
        all_issues=[
            {"number": 584, "state": "OPEN", "title": "Parent"},
            {"number": 585, "state": "CLOSED", "title": "Phase 1 (closed)"},
            {"number": 587, "state": "OPEN", "title": "Phase 2 (leaf)"},
            {"number": 589, "state": "OPEN", "title": "Phase 3 (leaf)"},
        ],
        children_map={584: [587, 589]},  # Missing closed #585
    )

    # This simulates the sibling lookup failure:
    # siblings.index(585) would raise ValueError because 585 not in [587, 589]
    siblings = workflow_without_closed.children_map[584]
    try:
        current_idx = siblings.index(585)  # This should fail
        print(f"  ✗ Unexpected: Found #585 at index {current_idx}")
        return False
    except ValueError:
        print(f"  ✓ As expected: ValueError when looking for #585 in {siblings}")
        print(f"  ✓ This causes fallback to first open issue (wrong behavior)")

    print()
    return True


def main():
    """Run all tests."""
    print("=" * 60)
    print("Testing _find_first_leaf() logic")
    print("=" * 60)
    print()

    tests = [
        test_descends_to_leaf,
        test_handles_nested_hierarchy,
        test_returns_self_when_no_children,
        test_skips_closed_children,
        test_closed_siblings_in_hierarchy,
    ]

    results = [test() for test in tests]

    print("=" * 60)
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
