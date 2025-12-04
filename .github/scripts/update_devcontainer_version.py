#!/usr/bin/env python3
"""
Update devcontainer feature version in devcontainer.json.

This script updates the version field for the langstar devcontainer feature
while preserving comments and formatting in the JSONC file.
"""

import sys
import re
from pathlib import Path


def update_devcontainer_version(file_path: str, new_version: str) -> bool:
    """
    Update the langstar devcontainer feature version in devcontainer.json.

    Args:
        file_path: Path to the devcontainer.json file
        new_version: The new version string (without 'v' prefix)

    Returns:
        True if the update was successful, False otherwise

    Raises:
        FileNotFoundError: If the devcontainer.json file doesn't exist
        ValueError: If the version pattern is not found in the file
    """
    path = Path(file_path)

    if not path.exists():
        raise FileNotFoundError(f"File not found: {file_path}")

    # Read the file
    content = path.read_text()

    # Update the specific version field while preserving comments and formatting
    # Pattern matches: "ghcr.io/codekiln/langstar/langstar:1": { ... "version": "any-value" ... }
    pattern = r'("ghcr\.io/codekiln/langstar/langstar:1":\s*\{[^}]*"version":\s*)"[^"]*"'
    replacement = rf'\g<1>"{new_version}"'
    updated_content = re.sub(pattern, replacement, content, flags=re.DOTALL)

    if updated_content == content:
        raise ValueError(
            f"Pattern not found in {file_path}. "
            "Could not locate the langstar devcontainer feature version field."
        )

    # Write the updated content
    path.write_text(updated_content)

    return True


def main() -> int:
    """Main entry point for the script."""
    if len(sys.argv) != 3:
        print("Usage: update_devcontainer_version.py <file_path> <new_version>", file=sys.stderr)
        print("Example: update_devcontainer_version.py .devcontainer/devcontainer.json 0.13.0", file=sys.stderr)
        return 1

    file_path = sys.argv[1]
    new_version = sys.argv[2]

    try:
        update_devcontainer_version(file_path, new_version)
        print(f"✓ Updated devcontainer version to {new_version}")
        return 0
    except FileNotFoundError as e:
        print(f"✗ Error: {e}", file=sys.stderr)
        return 1
    except ValueError as e:
        print(f"✗ Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        print(f"✗ Unexpected error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
