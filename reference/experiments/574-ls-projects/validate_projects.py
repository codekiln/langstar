#!/usr/bin/env python3
"""
Validate project listing and query specific project details.
"""

import os
import sys

try:
    from langsmith import Client
except ImportError:
    print("ERROR: langsmith package not installed")
    sys.exit(1)


def main():
    # Initialize client
    api_key = os.environ.get("LANGSMITH_API_KEY")
    if not api_key:
        print("ERROR: LANGSMITH_API_KEY environment variable not set")
        sys.exit(1)

    client = Client(api_key=api_key)

    print("=" * 80)
    print("VALIDATION: List All Projects and Query Specific Project")
    print("=" * 80)

    # List all projects
    print("\n1. Listing all projects...")
    print("-" * 80)

    all_projects = list(client.list_projects())
    print(f"✅ Total projects found: {len(all_projects)}")

    # Find specific project
    print("\n2. Finding project: 'test-deployment-cli-48499'")
    print("-" * 80)

    target_project = None
    for project in all_projects:
        if project.name == "test-deployment-cli-48499":
            target_project = project
            break

    if target_project:
        print(f"✅ Found project: {target_project.name}")
        print(f"   Project ID: {target_project.id}")
        print(f"   Tenant ID: {target_project.tenant_id}")
        print(f"   Description: {target_project.description}")
        print(f"   Start time: {target_project.start_time}")
        print(f"   URL: {target_project.url}")

        # Get run count if available
        print("\n3. Querying runs in project")
        print("-" * 80)

        # Query runs for this project
        runs = list(client.list_runs(project_name=target_project.name))
        print(f"✅ Total runs in project: {len(runs)}")

        if runs:
            print(f"\nFirst few runs:")
            for i, run in enumerate(runs[:5]):
                print(f"  {i+1}. Run ID: {run.id}")
                print(f"     Name: {run.name}")
                print(f"     Status: {run.status}")
                print(f"     Run type: {run.run_type}")
                print()
    else:
        print("❌ Project 'test-deployment-cli-48499' not found")
        print("\nSearching for similar names...")
        similar = [p for p in all_projects if "test-deployment" in p.name.lower()]
        if similar:
            print(f"Found {len(similar)} projects with 'test-deployment' in name:")
            for p in similar[:10]:
                print(f"  - {p.name} (ID: {p.id})")
        else:
            print("No similar projects found")

    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"Total projects: {len(all_projects)}")
    if target_project:
        print(f"Target project ID: {target_project.id}")
        print(f"Runs in target project: {len(runs)}")


if __name__ == "__main__":
    main()
