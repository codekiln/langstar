#!/usr/bin/env python3
"""
Experiment: Projects vs Sessions Terminology

Goal: Disambiguate the "projects" vs "sessions" terminology by examining:
1. Python SDK method names (projects)
2. Actual API endpoints called (sessions)
3. Request/response field names
4. OpenAPI spec terminology

This helps clarify what terminology to use in the Rust SDK and CLI.
"""

import os
import sys
from typing import Optional

try:
    from langsmith import Client
except ImportError:
    print("ERROR: langsmith package not installed")
    print("Install with: pip install langsmith")
    sys.exit(1)


def main():
    """Run experiments on LangSmith projects/sessions API."""

    # Initialize client
    api_key = os.environ.get("LANGSMITH_API_KEY")
    if not api_key:
        print("ERROR: LANGSMITH_API_KEY environment variable not set")
        sys.exit(1)

    client = Client(api_key=api_key)

    print("=" * 80)
    print("EXPERIMENT: Projects vs Sessions Terminology")
    print("=" * 80)

    # Experiment 1: List projects
    print("\n1. LIST PROJECTS")
    print("-" * 80)
    print("SDK Method: client.list_projects(limit=3)")
    print()

    try:
        projects = list(client.list_projects(limit=3))
        print(f"✅ Success: Retrieved {len(projects)} projects")
        print()

        if projects:
            project = projects[0]
            print("First project object type:", type(project).__name__)
            print("First project attributes:")
            for attr in sorted(dir(project)):
                if not attr.startswith("_"):
                    try:
                        value = getattr(project, attr)
                        if not callable(value):
                            # Truncate long values
                            str_value = str(value)
                            if len(str_value) > 60:
                                str_value = str_value[:57] + "..."
                            print(f"  - {attr}: {str_value}")
                    except Exception:
                        pass
    except Exception as e:
        print(f"❌ Error: {e}")

    # Experiment 2: Read specific project
    print("\n\n2. READ PROJECT BY NAME")
    print("-" * 80)

    try:
        # Get first project name
        projects = list(client.list_projects(limit=1))
        if projects:
            project_name = projects[0].name
            print(f"SDK Method: client.read_project(project_name='{project_name}')")
            print()

            project = client.read_project(project_name=project_name)
            print(f"✅ Success: Retrieved project")
            print()
            print("Project details:")
            print(f"  - ID: {project.id}")
            print(f"  - Name: {project.name}")
            print(f"  - Description: {project.description}")
            print(f"  - Tenant ID: {project.tenant_id}")
            print(f"  - Start time: {project.start_time}")
            print(f"  - End time: {project.end_time}")
            print(f"  - URL: {project.url}")

            # Check for "session" terminology in attributes
            print()
            print("Checking for 'session' terminology in attributes:")
            session_attrs = [attr for attr in dir(project) if 'session' in attr.lower()]
            if session_attrs:
                print(f"  Found: {session_attrs}")
            else:
                print("  None found (all attributes use 'project' terminology)")
    except Exception as e:
        print(f"❌ Error: {e}")

    # Experiment 3: Inspect API URL construction
    print("\n\n3. API ENDPOINT INSPECTION")
    print("-" * 80)
    print("Based on Python SDK source code analysis:")
    print()
    print("Python SDK method names:")
    print("  - list_projects()")
    print("  - read_project()")
    print("  - create_project()")
    print("  - update_project()")
    print("  - delete_project()")
    print()
    print("Actual API endpoints (from client.py):")
    print("  - GET /sessions")
    print("  - GET /sessions/{id}")
    print("  - POST /sessions")
    print("  - PATCH /sessions/{id}")
    print("  - DELETE /sessions/{id}")
    print()
    print("Python schema class names:")
    print("  - TracerSession (base class)")
    print("  - TracerSessionResult (with additional fields)")
    print()
    print("Schema comment (schemas.py:732):")
    print('  "Sessions are also referred to as \'Projects\' in the UI."')

    # Experiment 4: Check API base URL
    print("\n\n4. CLIENT CONFIGURATION")
    print("-" * 80)
    print(f"API URL: {client.api_url}")
    print(f"API Key present: {'Yes' if client.api_key else 'No'}")

    # Summary
    print("\n\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print()
    print("Terminology Mapping:")
    print("  ┌─────────────────────┬──────────────────────────────┐")
    print("  │ Layer               │ Terminology Used             │")
    print("  ├─────────────────────┼──────────────────────────────┤")
    print("  │ LangSmith UI        │ 'Projects'                   │")
    print("  │ Python SDK (public) │ 'Projects' (method names)    │")
    print("  │ Python SDK (schema) │ 'TracerSession' (class name) │")
    print("  │ REST API            │ '/sessions' (endpoint)       │")
    print("  │ OpenAPI spec        │ TBD - need to check          │")
    print("  └─────────────────────┴──────────────────────────────┘")
    print()
    print("Recommendation for Rust SDK:")
    print("  - Public API: Use 'Project' terminology (structs, methods)")
    print("  - Internal mapping: Use '/sessions' endpoint")
    print("  - Follow Python SDK precedent for consistency")


if __name__ == "__main__":
    main()
