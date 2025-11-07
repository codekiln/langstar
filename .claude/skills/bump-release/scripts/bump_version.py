#!/usr/bin/env python3
"""
Update version in Cargo.toml file(s).
Supports both workspace and standalone Cargo.toml files.
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple, Optional


def find_cargo_tomls(root_dir: Path) -> List[Path]:
    """
    Find all Cargo.toml files in the project.

    Args:
        root_dir: Project root directory

    Returns:
        List of Cargo.toml file paths
    """
    cargo_tomls = []

    # Always include root Cargo.toml if it exists
    root_cargo = root_dir / 'Cargo.toml'
    if root_cargo.exists():
        cargo_tomls.append(root_cargo)

    # Find Cargo.toml files in workspace members
    for member_cargo in root_dir.rglob('*/Cargo.toml'):
        if member_cargo not in cargo_tomls:
            cargo_tomls.append(member_cargo)

    return sorted(cargo_tomls)


def is_workspace_root(cargo_toml: Path) -> bool:
    """
    Check if a Cargo.toml is a workspace root.

    Args:
        cargo_toml: Path to Cargo.toml file

    Returns:
        True if it's a workspace root
    """
    content = cargo_toml.read_text()
    return '[workspace]' in content


def update_version_in_file(
    cargo_toml: Path,
    new_version: str,
    dry_run: bool = False
) -> Tuple[bool, Optional[str]]:
    """
    Update version in a single Cargo.toml file.

    Args:
        cargo_toml: Path to Cargo.toml file
        new_version: New version string (without 'v' prefix)
        dry_run: If True, don't actually write changes

    Returns:
        Tuple of (success, old_version or None)
    """
    content = cargo_toml.read_text()

    # Pattern to match version = "X.Y.Z" in [package] section
    # We need to be careful to only match the main package version
    pattern = re.compile(
        r'^(\[package\].*?^version\s*=\s*)"([^"]+)"',
        re.MULTILINE | re.DOTALL
    )

    match = pattern.search(content)
    if not match:
        return False, None

    old_version = match.group(2)

    # Replace the version
    new_content = pattern.sub(
        rf'\1"{new_version}"',
        content,
        count=1
    )

    if not dry_run:
        cargo_toml.write_text(new_content)

    return True, old_version


def update_workspace_dependencies(
    cargo_toml: Path,
    package_name: str,
    new_version: str,
    dry_run: bool = False
) -> bool:
    """
    Update workspace dependency versions that reference the package.

    Args:
        cargo_toml: Path to Cargo.toml file
        package_name: Name of the package being updated
        new_version: New version string
        dry_run: If True, don't actually write changes

    Returns:
        True if any updates were made
    """
    content = cargo_toml.read_text()

    # Pattern to match dependencies with workspace = true or version reference
    # This handles both:
    #   package-name = { version = "1.2.3", path = "../" }
    #   package-name = "1.2.3"
    pattern = re.compile(
        rf'^({re.escape(package_name)}\s*=\s*{{[^}}]*version\s*=\s*)"([^"]+)"',
        re.MULTILINE
    )

    if not pattern.search(content):
        return False

    new_content = pattern.sub(rf'\1"{new_version}"', content)

    if not dry_run:
        cargo_toml.write_text(new_content)

    return True


def get_package_name(cargo_toml: Path) -> Optional[str]:
    """
    Extract package name from Cargo.toml.

    Args:
        cargo_toml: Path to Cargo.toml file

    Returns:
        Package name or None
    """
    content = cargo_toml.read_text()
    match = re.search(r'^\[package\].*?^name\s*=\s*"([^"]+)"', content, re.MULTILINE | re.DOTALL)
    return match.group(1) if match else None


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description='Update version in Cargo.toml file(s)'
    )
    parser.add_argument(
        'new_version',
        help='New version (e.g., 1.2.3 or v1.2.3)'
    )
    parser.add_argument(
        '--root',
        type=Path,
        default=Path.cwd(),
        help='Project root directory (default: current directory)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Show what would be updated without making changes'
    )
    parser.add_argument(
        '--workspace-deps',
        action='store_true',
        help='Also update workspace dependency references'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Show detailed output'
    )

    args = parser.parse_args()

    # Strip 'v' prefix if present
    new_version = args.new_version.lstrip('v')

    # Validate version format
    if not re.match(r'^\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?$', new_version):
        print(f"Error: Invalid version format: {new_version}", file=sys.stderr)
        print("Expected format: X.Y.Z or X.Y.Z-prerelease", file=sys.stderr)
        sys.exit(1)

    # Find all Cargo.toml files
    cargo_tomls = find_cargo_tomls(args.root)

    if not cargo_tomls:
        print(f"Error: No Cargo.toml files found in {args.root}", file=sys.stderr)
        sys.exit(1)

    if args.verbose or args.dry_run:
        print(f"Found {len(cargo_tomls)} Cargo.toml file(s):", file=sys.stderr)
        for toml in cargo_tomls:
            print(f"  - {toml.relative_to(args.root)}", file=sys.stderr)
        print(file=sys.stderr)

    if args.dry_run:
        print("DRY RUN MODE - No changes will be made", file=sys.stderr)
        print(file=sys.stderr)

    # Track updates
    updated_count = 0
    package_names = {}

    # First pass: update all package versions and collect names
    for cargo_toml in cargo_tomls:
        if is_workspace_root(cargo_toml) and not get_package_name(cargo_toml):
            # Workspace-only file (no [package] section)
            if args.verbose:
                print(f"Skipping workspace root: {cargo_toml.relative_to(args.root)}", file=sys.stderr)
            continue

        success, old_version = update_version_in_file(
            cargo_toml,
            new_version,
            dry_run=args.dry_run
        )

        if success:
            pkg_name = get_package_name(cargo_toml)
            if pkg_name:
                package_names[pkg_name] = cargo_toml

            rel_path = cargo_toml.relative_to(args.root)
            if args.dry_run:
                print(f"Would update {rel_path}: {old_version} → {new_version}")
            else:
                print(f"Updated {rel_path}: {old_version} → {new_version}")
            updated_count += 1
        else:
            rel_path = cargo_toml.relative_to(args.root)
            print(f"Warning: Could not find version in {rel_path}", file=sys.stderr)

    # Second pass: update workspace dependencies if requested
    if args.workspace_deps and package_names:
        print(file=sys.stderr)
        for pkg_name, pkg_toml in package_names.items():
            for cargo_toml in cargo_tomls:
                if cargo_toml == pkg_toml:
                    continue

                updated = update_workspace_dependencies(
                    cargo_toml,
                    pkg_name,
                    new_version,
                    dry_run=args.dry_run
                )

                if updated:
                    rel_path = cargo_toml.relative_to(args.root)
                    if args.dry_run:
                        print(f"Would update dependency '{pkg_name}' in {rel_path} → {new_version}")
                    else:
                        print(f"Updated dependency '{pkg_name}' in {rel_path} → {new_version}")

    # Summary
    print(file=sys.stderr)
    if args.dry_run:
        print(f"Would update {updated_count} file(s)", file=sys.stderr)
    else:
        print(f"Successfully updated {updated_count} file(s)", file=sys.stderr)

    if updated_count == 0:
        sys.exit(1)


if __name__ == '__main__':
    main()
