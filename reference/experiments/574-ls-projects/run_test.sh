#!/bin/bash
# Run the projects API experiment
#
# This script sources the devcontainer environment and runs the test script

set -euo pipefail

# Check for API key
if [ -z "${LANGSMITH_API_KEY:-}" ]; then
    echo "ERROR: LANGSMITH_API_KEY environment variable not set"
    echo ""
    echo "Set it with:"
    echo "  export LANGSMITH_API_KEY=<your-api-key>"
    exit 1
fi

# Ensure we're in the experiment directory
cd "$(dirname "$0")"

echo "Running projects API experiment..."
echo ""

# Run the Python script
python3 test_projects.py

echo ""
echo "Experiment complete."
