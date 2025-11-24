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

# Configuration
REPO="codekiln/langstar"
RULESET_ID="9196293"  # "main" ruleset
RULESET_NAME="main"

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
if ! gh api "repos/${REPO}/rulesets/${RULESET_ID}" > "$TEMP_RULESET" 2>/dev/null; then
    echo "❌ Error: Failed to fetch ruleset"
    echo "   Make sure you have 'Administration' permissions for the repository"
    rm -f "$TEMP_RULESET"
    exit 1
fi
echo "✓ Ruleset fetched"
echo ""

# Check current strict mode setting
CURRENT_STRICT=$(cat "$TEMP_RULESET" | jq -r '
  .rules[] | select(.type == "required_status_checks") | .parameters.strict_required_status_checks_policy
')

echo "Current strict mode setting: ${CURRENT_STRICT}"
echo ""

# Clean up temporary file
rm -f "$TEMP_RULESET"
if [ "${CURRENT_STRICT}" == "true" ]; then
    echo "✓ Strict mode is already enabled!"
    echo "  No changes needed."
    exit 0
fi

echo "→ Preparing update payload..."

# Prepare update payload (keep only updatable fields)
cat /tmp/main-ruleset.json | jq '{
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
}' > /tmp/updated-main-ruleset.json

echo "✓ Update payload prepared"
echo ""

# Show what will change
echo "→ Changes to be applied:"
echo "  - strict_required_status_checks_policy: false → true"
echo ""

# Apply the update
echo "→ Applying update..."
if ! gh api -X PUT "repos/${REPO}/rulesets/${RULESET_ID}" \
  --input /tmp/updated-main-ruleset.json > /tmp/update-result.json 2>/dev/null; then
    echo "❌ Error: Failed to update ruleset"
    echo "   Make sure you have 'Administration' permissions for the repository"
    exit 1
fi

echo "✓ Update applied successfully!"
echo ""

# Verify the change
echo "→ Verifying update..."
UPDATED_STRICT=$(cat /tmp/update-result.json | jq -r '
  .rules[] | select(.type == "required_status_checks") | .parameters.strict_required_status_checks_policy
')

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
else
    echo "❌ Error: Verification failed"
    echo "   Expected strict mode to be true, but got: ${UPDATED_STRICT}"
    exit 1
fi

# Cleanup
rm -f /tmp/main-ruleset.json /tmp/updated-main-ruleset.json /tmp/update-result.json
