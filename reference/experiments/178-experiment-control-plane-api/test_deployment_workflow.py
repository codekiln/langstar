#!/usr/bin/env python3
"""
LangSmith Control Plane API Example Code
Source: https://docs.langchain.com/langsmith/api-ref-control-plane

This is the official example code from the documentation, used to test the
4-step deployment workflow:
1. Create deployment
2. Get deployment and list revisions
3. Wait for revision status to be DEPLOYED
4. Patch deployment (update)
"""

import argparse
import os
import sys
import time

import requests


# Use environment variables (set via shell, not hardcoded .env path)
CONTROL_PLANE_HOST = os.getenv("CONTROL_PLANE_HOST", "https://api.host.langchain.com")
LANGSMITH_API_KEY = os.getenv("LANGSMITH_API_KEY")
# Control Plane API uses workspace ID (try both variable names)
WORKSPACE_ID = os.getenv("LANGSMITH_WORKSPACE_ID") or os.getenv("LANGCHAIN_WORKSPACE_ID")
# Repository configuration
REPOSITORY_OWNER = os.getenv("REPOSITORY_OWNER", "codekiln")
REPOSITORY_NAME = os.getenv("REPOSITORY_NAME", "langstar")
MAX_WAIT_TIME = 1800  # 30 mins

# Validate required environment variables
missing_vars = []
if not LANGSMITH_API_KEY:
    missing_vars.append("LANGSMITH_API_KEY")
if not WORKSPACE_ID:
    missing_vars.append("LANGSMITH_WORKSPACE_ID or LANGCHAIN_WORKSPACE_ID")

if missing_vars:
    print(f"ERROR: Missing required environment variables: {', '.join(missing_vars)}")
    sys.exit(1)


def get_headers() -> dict:
    """Return common headers for requests to the control plane API."""
    return {
        "X-Api-Key": LANGSMITH_API_KEY,
        "X-Tenant-Id": WORKSPACE_ID,
    }


def list_github_integrations() -> list[dict]:
    """List GitHub integrations for the workspace."""
    response = requests.get(
        url=f"{CONTROL_PLANE_HOST}/v1/integrations/github/install",
        headers=get_headers(),
    )

    if response.status_code != 200:
        raise Exception(f"Failed to list GitHub integrations: {response.text}")

    return response.json()


def list_github_repositories(integration_id: str) -> list[dict]:
    """List GitHub repositories for a given integration."""
    response = requests.get(
        url=f"{CONTROL_PLANE_HOST}/v1/integrations/github/{integration_id}/repos",
        headers=get_headers(),
    )

    if response.status_code != 200:
        raise Exception(
            f"Failed to list repositories for integration {integration_id}: {response.text}"
        )

    return response.json()


def find_integration_for_repo(owner: str, repo: str) -> str:
    """Find the integration ID for a specific GitHub repository.

    Args:
        owner: Repository owner (e.g., "codekiln")
        repo: Repository name (e.g., "langstar")

    Returns:
        integration_id: The integration ID that has access to this repository
    """
    print(f"🔍 Looking for GitHub integration with access to {owner}/{repo}")

    # Get all GitHub integrations
    integrations = list_github_integrations()
    print(f"Found {len(integrations)} GitHub integration(s)")

    # Check each integration for the target repository
    for integration in integrations:
        integration_id = integration.get("id")
        integration_name = integration.get("name", "Unknown")

        print(f"\n  Checking integration: {integration_name} (ID: {integration_id})")

        try:
            # List repositories for this integration
            repos = list_github_repositories(integration_id)

            # Search for target repository
            for r in repos:
                repo_owner = r.get("owner", "")
                repo_name = r.get("name", "")
                if repo_owner == owner and repo_name == repo:
                    print(f"  ✓ Found {owner}/{repo} in integration {integration_name}")
                    return integration_id

            print(f"  ✗ {owner}/{repo} not found in this integration")

        except Exception as e:
            print(f"  ✗ Error checking integration: {e}")
            continue

    raise Exception(f"No integration found with access to {owner}/{repo}")


def create_deployment() -> str:
    """Create deployment. Return deployment ID."""
    headers = get_headers()
    headers["Content-Type"] = "application/json"

    # Find integration ID for the configured repository dynamically
    integration_id = find_integration_for_repo(REPOSITORY_OWNER, REPOSITORY_NAME)

    # Use timestamp to ensure unique deployment name
    timestamp = int(time.time())
    deployment_name = f"{REPOSITORY_NAME}-test-{timestamp}"

    request_body = {
        "name": deployment_name,
        "source": "github",
        "source_config": {
            "integration_id": integration_id,
            "repo_url": f"https://github.com/{REPOSITORY_OWNER}/{REPOSITORY_NAME}",
            "deployment_type": "dev",
            "build_on_push": False,
            "custom_url": None,
            "resource_spec": None,
        },
        "source_revision_config": {
            "repo_ref": "main",
            "langgraph_config_path": "tests/fixtures/test-graph-deployment/langgraph.json",
            "image_uri": None,
        },
        "secrets": [],
    }

    response = requests.post(
        url=f"{CONTROL_PLANE_HOST}/v2/deployments",
        headers=headers,
        json=request_body,
    )

    if response.status_code != 201:
        raise Exception(f"Failed to create deployment: {response.text}")

    deployment_id = response.json()["id"]
    print(f"Created deployment {deployment_name} ({deployment_id})")
    return deployment_id


def list_deployments(name_contains: str = None) -> dict:
    """List all deployments, optionally filtered by name.

    Args:
        name_contains: Optional filter to find deployments whose name contains this string

    Returns:
        Dictionary with 'resources' key containing list of deployments
    """
    params = {}
    if name_contains:
        params["name_contains"] = name_contains

    response = requests.get(
        url=f"{CONTROL_PLANE_HOST}/v2/deployments",
        headers=get_headers(),
        params=params,
    )

    if response.status_code != 200:
        raise Exception(f"Failed to list deployments: {response.text}")

    return response.json()


def get_deployment(deployment_id: str) -> dict:
    """Get deployment."""
    response = requests.get(
        url=f"{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}",
        headers=get_headers(),
    )

    if response.status_code != 200:
        raise Exception(f"Failed to get deployment ID {deployment_id}: {response.text}")

    return response.json()


def list_revisions(deployment_id: str) -> list[dict]:
    """List revisions.

    Return list is sorted by created_at in descending order (latest first).
    """
    response = requests.get(
        url=f"{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}/revisions",
        headers=get_headers(),
    )

    if response.status_code != 200:
        raise Exception(
            f"Failed to list revisions for deployment ID {deployment_id}: {response.text}"
        )

    return response.json()


def get_revision(
    deployment_id: str,
    revision_id: str,
) -> dict:
    """Get revision."""
    response = requests.get(
        url=f"{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}/revisions/{revision_id}",
        headers=get_headers(),
    )

    if response.status_code != 200:
        raise Exception(f"Failed to get revision ID {revision_id}: {response.text}")

    return response.json()


def patch_deployment(deployment_id: str) -> None:
    """Patch deployment."""
    headers = get_headers()
    headers["Content-Type"] = "application/json"

    # This creates a new revision because source_revision_config is included
    response = requests.patch(
        url=f"{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}",
        headers=headers,
        json={
            "source_config": {
                "build_on_push": True,
            },
            "source_revision_config": {
                "repo_ref": "main",
                "langgraph_config_path": "tests/fixtures/test-graph-deployment/langgraph.json",
            },
        },
    )

    if response.status_code != 200:
        raise Exception(f"Failed to patch deployment: {response.text}")

    print(f"Patched deployment ID {deployment_id}")


def wait_for_deployment(deployment_id: str, revision_id: str) -> None:
    """Wait for revision status to be DEPLOYED."""
    start_time = time.time()
    revision, status = None, None
    while time.time() - start_time < MAX_WAIT_TIME:
        revision = get_revision(deployment_id, revision_id)
        status = revision["status"]
        if status == "DEPLOYED":
            break
        elif "FAILED" in status:
            raise Exception(f"Revision ID {revision_id} failed: {revision}")

        print(f"Waiting for revision ID {revision_id} to be DEPLOYED...")
        time.sleep(60)

    if status != "DEPLOYED":
        raise Exception(
            f"Timeout waiting for revision ID {revision_id} to be DEPLOYED: {revision}"
        )


def delete_deployment(deployment_id: str) -> None:
    """Delete deployment."""
    response = requests.delete(
        url=f"{CONTROL_PLANE_HOST}/v2/deployments/{deployment_id}",
        headers=get_headers(),
    )

    if response.status_code != 204:
        raise Exception(
            f"Failed to delete deployment ID {deployment_id}: {response.text}"
        )

    print(f"Deployment ID {deployment_id} deleted")


def run_full_workflow():
    """Run the complete 4-step deployment workflow test."""
    # create deployment and get the latest revision
    deployment_id = create_deployment()
    revisions = list_revisions(deployment_id)
    latest_revision = revisions["resources"][0]
    latest_revision_id = latest_revision["id"]

    # wait for latest revision to be DEPLOYED
    wait_for_deployment(deployment_id, latest_revision_id)

    # patch the deployment and get the latest revision
    patch_deployment(deployment_id)
    revisions = list_revisions(deployment_id)
    latest_revision = revisions["resources"][0]
    latest_revision_id = latest_revision["id"]

    # wait for latest revision to be DEPLOYED
    wait_for_deployment(deployment_id, latest_revision_id)

    # delete the deployment
    delete_deployment(deployment_id)


def cmd_list_deployments(args):
    """Command handler for listing deployments."""
    print(f"Listing deployments...")
    if args.name_contains:
        print(f"  Filter: name contains '{args.name_contains}'")
    print()

    result = list_deployments(name_contains=args.name_contains)
    deployments = result.get("resources", [])

    if not deployments:
        print("No deployments found.")
        return

    print(f"Found {len(deployments)} deployment(s):\n")
    for d in deployments:
        name = d.get("name", "unknown")
        id = d.get("id", "unknown")
        status = d.get("status", "unknown")
        created_at = d.get("created_at", "unknown")
        print(f"  Name: {name}")
        print(f"  ID: {id}")
        print(f"  Status: {status}")
        print(f"  Created: {created_at}")
        print()


def cmd_get_deployment(args):
    """Command handler for getting a specific deployment by ID."""
    import json

    deployment = get_deployment(args.deployment_id)
    print(json.dumps(deployment, indent=2))


def cmd_get_latest_revision(args):
    """Command handler for getting the latest revision of a deployment."""
    import json

    # Get deployment to find latest revision ID
    deployment = get_deployment(args.deployment_id)
    latest_revision_id = deployment.get("latest_revision_id")

    if not latest_revision_id:
        print(f"ERROR: No latest_revision_id found in deployment {args.deployment_id}")
        sys.exit(1)

    print(f"# Latest revision ID: {latest_revision_id}\n")

    # Get the revision details
    revision = get_revision(args.deployment_id, latest_revision_id)
    print(json.dumps(revision, indent=2))


def cmd_run_workflow(args):
    """Command handler for running full workflow test."""
    print("Running full 4-step deployment workflow test...\n")
    run_full_workflow()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="LangSmith Control Plane API deployment workflow tester",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # List all deployments
  python test_deployment_workflow.py list

  # List deployments matching a name pattern
  python test_deployment_workflow.py list --name-contains langstar-test

  # Get a specific deployment by ID (raw JSON)
  python test_deployment_workflow.py get f6a74b34-5666-4e49-8936-9f4c4d08e1e0

  # Get the latest revision of a deployment (raw JSON)
  python test_deployment_workflow.py get-latest-revision f6a74b34-5666-4e49-8936-9f4c4d08e1e0

  # Run the full 4-step workflow test
  python test_deployment_workflow.py run-workflow
        """
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    subparsers.required = True

    # List deployments command
    list_parser = subparsers.add_parser("list", help="List deployments")
    list_parser.add_argument(
        "--name-contains",
        help="Filter deployments by name (partial match)",
        default=None
    )
    list_parser.set_defaults(func=cmd_list_deployments)

    # Get deployment command
    get_parser = subparsers.add_parser("get", help="Get a specific deployment by ID (returns raw JSON)")
    get_parser.add_argument(
        "deployment_id",
        help="Deployment ID to retrieve"
    )
    get_parser.set_defaults(func=cmd_get_deployment)

    # Get latest revision command
    get_revision_parser = subparsers.add_parser(
        "get-latest-revision",
        help="Get the latest revision of a deployment (returns raw JSON)"
    )
    get_revision_parser.add_argument(
        "deployment_id",
        help="Deployment ID to retrieve latest revision from"
    )
    get_revision_parser.set_defaults(func=cmd_get_latest_revision)

    # Run workflow command
    workflow_parser = subparsers.add_parser(
        "run-workflow",
        help="Run the full 4-step deployment workflow test"
    )
    workflow_parser.set_defaults(func=cmd_run_workflow)

    args = parser.parse_args()
    args.func(args)
