#!/bin/bash
# Enable strict required status checks for the "main" repository ruleset
#
# This script updates the "main" ruleset to require branches to be up-to-date
# before merging. This ensures that all CI checks run on the final code that
# will be merged, preventing integration issues and merge conflicts.
#
# Related issue: #253
#
# Prerequisites:
# - gh CLI must be installed and authenticated
# - User must have "Administration" repository permissions
# - jq must be installed for JSON processing

set -euo pipefail

# Configuration: Accept as command-line arguments, with defaults
REPO="${1:-codekiln/langstar}"
RULESET_ID="${2:-9196293}"
RULESET_NAME="${3:-main}"
echo "================================================"
echo "Enable Strict Status Checks for ${RULESET_NAME} Ruleset"
echo "================================================"
echo ""
echo "Repository: ${REPO}"
echo "Ruleset ID: ${RULESET_ID}"
echo ""

# Check prerequisites
echo "→ Checking prerequisites..."

if ! command -v gh &> /dev/null; then
    echo "❌ Error: gh CLI is not installed"
    echo "   Install: https://cli.github.com/"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "❌ Error: jq is not installed"
    echo "   Install: https://jqlang.github.io/jq/download/"
    exit 1
fi

echo "✓ Prerequisites OK"
echo ""

# Fetch current ruleset
echo "→ Fetching current ruleset..."
TEMP_RULESET=$(mktemp)
TEMP_FETCH_ERROR=$(mktemp)
if ! gh api "repos/${REPO}/rulesets/${RULESET_ID}" > "$TEMP_RULESET" 2> "$TEMP_FETCH_ERROR"; then
    echo "❌ Error: Failed to fetch ruleset"
    echo "   Details:"
    cat "$TEMP_FETCH_ERROR"
    echo "   Make sure you have 'Administration' permissions for the repository"
    rm -f "$TEMP_RULESET" "$TEMP_FETCH_ERROR"
    exit 1
fi
rm -f "$TEMP_FETCH_ERROR"
echo "✓ Ruleset fetched"
echo ""

# Check current strict mode setting
CURRENT_STRICT=$(jq -r '
  .rules[] | select(.type == "required_status_checks") | .parameters.strict_required_status_checks_policy
' "$TEMP_RULESET")

echo "Current strict mode setting: ${CURRENT_STRICT}"
echo ""

if [ "${CURRENT_STRICT}" == "true" ]; then
    # Clean up temporary file
    rm -f "$TEMP_RULESET"
    echo "✓ Strict mode is already enabled!"
    echo "  No changes needed."
    exit 0
fi

echo "→ Preparing update payload..."

# Prepare update payload (keep only updatable fields)
TEMP_UPDATED=$(mktemp)
jq '{
  name: .name,
  target: .target,
  enforcement: .enforcement,
  bypass_actors: .bypass_actors,
  conditions: .conditions,
  rules: (.rules | map(
    if .type == "required_status_checks"
    then .parameters.strict_required_status_checks_policy = true
    else .
    end
  ))
}' "$TEMP_RULESET" > "$TEMP_UPDATED"

# Clean up the original temp file now that we've processed it
rm -f "$TEMP_RULESET"

echo "✓ Update payload prepared"
echo ""

# Show what will change
echo "→ Changes to be applied:"
echo "  - strict_required_status_checks_policy: false → true"
echo ""

# Apply the update
echo "→ Applying update..."
TEMP_RESULT=$(mktemp)
TEMP_ERROR=$(mktemp)
if ! gh api -X PUT "repos/${REPO}/rulesets/${RULESET_ID}" \
  --input "$TEMP_UPDATED" > "$TEMP_RESULT" 2> "$TEMP_ERROR"; then
    echo "❌ Error: Failed to update ruleset"
    echo "   Details:"
    cat "$TEMP_ERROR"
    echo "   Make sure you have 'Administration' permissions for the repository"
    rm -f "$TEMP_UPDATED" "$TEMP_RESULT" "$TEMP_ERROR"
    exit 1
fi

# Clean up temp files
rm -f "$TEMP_UPDATED" "$TEMP_ERROR"

echo "✓ Update applied successfully!"
echo ""

# Verify the change
echo "→ Verifying update..."
UPDATED_STRICT=$(jq -r '
  .rules[] | select(.type == "required_status_checks") | .parameters.strict_required_status_checks_policy
' "$TEMP_RESULT")

if [ "${UPDATED_STRICT}" == "true" ]; then
    echo "✓ Verification successful!"
    echo ""
    echo "================================================"
    echo "✓ Strict mode enabled for '${RULESET_NAME}' ruleset"
    echo "================================================"
    echo ""
    echo "What this means:"
    echo "  - Branches must be up-to-date with base before merging"
    echo "  - All CI checks will run on the final merged code"
    echo "  - Reduces risk of merge conflicts and integration issues"
    echo ""
    echo "View ruleset in GitHub:"
    echo "  https://github.com/${REPO}/rules/${RULESET_ID}"
    echo ""
    # Clean up temp file
    rm -f "$TEMP_RESULT"
else
    echo "❌ Error: Verification failed"
    echo "   Expected strict mode to be true, but got: ${UPDATED_STRICT}"
    # Clean up temp file
    rm -f "$TEMP_RESULT"
    exit 1
fi
