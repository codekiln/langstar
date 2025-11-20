#!/bin/bash
# Wrapper script to run the deployment workflow test with environment variables

set -e

# Source environment variables from devcontainer .env
if [ -f /workspace/.devcontainer/.env ]; then
    export $(grep -v '^#' /workspace/.devcontainer/.env | xargs)
else
    echo "ERROR: /workspace/.devcontainer/.env not found"
    exit 1
fi

# Run the script with arguments
# If no arguments provided, default to run-workflow
cd "$(dirname "$0")"
if [ $# -eq 0 ]; then
    python3 test_deployment_workflow.py run-workflow
else
    python3 test_deployment_workflow.py "$@"
fi
