#!/bin/bash
# Wrapper script to run structured output prompts experiments with environment variables

set -e

# Source environment variables from devcontainer .env
if [ -f /workspace/.devcontainer/.env ]; then
    export $(grep -v '^#' /workspace/.devcontainer/.env | xargs)
else
    echo "ERROR: /workspace/.devcontainer/.env not found"
    exit 1
fi

# Run the script with arguments
# If no arguments provided, default to list available commands
cd "$(dirname "$0")"
if [ $# -eq 0 ]; then
    python3 test_structured_prompts.py --help
else
    python3 test_structured_prompts.py "$@"
fi
