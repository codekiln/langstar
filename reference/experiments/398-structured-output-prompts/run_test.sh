#!/bin/bash
# Wrapper script to run structured output prompts experiments with environment variables

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Source environment variables from devcontainer .env
if [ -f /workspace/.devcontainer/.env ]; then
    set -a
    source /workspace/.devcontainer/.env
    set +a
else
    echo "ERROR: /workspace/.devcontainer/.env not found"
    exit 1
fi

# Use virtual environment if it exists, otherwise use system python
if [ -f "$SCRIPT_DIR/.venv/bin/python3" ]; then
    PYTHON="$SCRIPT_DIR/.venv/bin/python3"
else
    PYTHON="python3"
fi

# Run the script with arguments
# If no arguments provided, default to list available commands
cd "$SCRIPT_DIR"
if [ $# -eq 0 ]; then
    $PYTHON test_structured_prompts.py --help
else
    $PYTHON test_structured_prompts.py "$@"
fi
