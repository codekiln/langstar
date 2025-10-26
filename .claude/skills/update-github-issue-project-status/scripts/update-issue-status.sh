#!/bin/bash
# update-issue-status.sh
# Updates issue assignment and GitHub Project status
#
# Usage:
#   ./update-issue-status.sh <issue_number> <status> [assignee]
#   ./update-issue-status.sh <issue1,issue2,issue3> <status> [assignee]
#
# Arguments:
#   issue_number(s) - Single issue number or comma-separated list
#   status          - The status to set: "todo", "in_progress", or "done"
#   assignee        - (Optional) GitHub username to assign (for in_progress status)
#
# Environment variables:
#   GITHUB_PROJECT_PAT or GH_PROJECT_PAT - Token with project scope (required)
#   GITHUB_PAT                           - Token with repo scope (optional, for issue assignment)
#
# Example:
#   export GITHUB_PROJECT_PAT="ghp_xxxx"  # classic PAT with project scope
#   export GITHUB_PAT="github_pat_xxx"    # fine-grained PAT with repo scope (optional)
#   ./update-issue-status.sh 17 in_progress codekiln
#   ./update-issue-status.sh 15,16,17 done

set -e

# Check for GitHub tokens in environment
# Two tokens are used:
# - GITHUB_PAT: For issue operations (requires repo scope)
# - GITHUB_PROJECT_PAT/GH_PROJECT_PAT: For project operations (requires project scope)

if [ -n "$GITHUB_PROJECT_PAT" ]; then
  PROJECT_TOKEN="$GITHUB_PROJECT_PAT"
elif [ -n "$GH_PROJECT_PAT" ]; then
  PROJECT_TOKEN="$GH_PROJECT_PAT"
else
  echo "Error: No project token found in environment"
  echo "Please set one of: GITHUB_PROJECT_PAT or GH_PROJECT_PAT"
  echo "Token must have project scope"
  exit 1
fi

# For issue assignment, use GITHUB_PAT if available
if [ -n "$GITHUB_PAT" ]; then
  ISSUE_TOKEN="$GITHUB_PAT"
else
  echo "Warning: GITHUB_PAT not set - issue assignment will be skipped"
  ISSUE_TOKEN=""
fi

# Set repository (hardcoded for langstar project)
GITHUB_REPOSITORY="codekiln/langstar"

# Configuration - Update these IDs for your project
PROJECT_ID="PVT_kwHOAAImgs4BGe4B"  # langstar project
STATUS_FIELD_ID="PVTSSF_lAHOAAImgs4BGe4Bzg3g-NQ"  # Status field
STATUS_TODO="f75ad846"
STATUS_IN_PROGRESS="47fc9ee4"
STATUS_DONE="98236657"

# Parse arguments
ISSUE_NUMBERS="${1}"
STATUS="${2}"
ASSIGNEE="${3}"

if [ -z "$ISSUE_NUMBERS" ] || [ -z "$STATUS" ]; then
  echo "Usage: $0 <issue_number(s)> <status> [assignee]"
  echo ""
  echo "Arguments:"
  echo "  issue_number(s) - Single issue or comma-separated list (e.g., '17' or '15,16,17')"
  echo "  status          - Must be: todo, in_progress, or done"
  echo "  assignee        - GitHub username (optional, used with in_progress)"
  echo ""
  echo "Examples:"
  echo "  $0 17 in_progress codekiln"
  echo "  $0 15,16,17 done"
  exit 1
fi

# Map status name to option ID
case "$STATUS" in
  todo)
    STATUS_OPTION_ID="$STATUS_TODO"
    ;;
  in_progress)
    STATUS_OPTION_ID="$STATUS_IN_PROGRESS"
    ;;
  done)
    STATUS_OPTION_ID="$STATUS_DONE"
    ;;
  *)
    echo "Error: Invalid status '$STATUS'"
    echo "Must be one of: todo, in_progress, done"
    exit 1
    ;;
esac

# Function to update a single issue
update_single_issue() {
  local ISSUE_NUMBER="$1"

  echo ""
  echo "Updating issue #${ISSUE_NUMBER}..."

  # Assign issue if assignee provided and status is in_progress
  if [ "$STATUS" = "in_progress" ] && [ -n "$ASSIGNEE" ]; then
    if [ -n "$ISSUE_TOKEN" ]; then
      echo "  - Assigning to @${ASSIGNEE}..."
      GH_TOKEN="$ISSUE_TOKEN" gh issue edit "$ISSUE_NUMBER" --add-assignee "$ASSIGNEE" || {
        echo "  ⚠ Warning: Failed to assign issue"
      }
    else
      echo "  - Skipping assignment (GITHUB_PAT not set)"
    fi
  fi

  # Get issue node ID (can use either token for read operations)
  echo "  - Getting issue node ID..."
  ISSUE_NODE_ID=$(GH_TOKEN="$PROJECT_TOKEN" gh issue view "$ISSUE_NUMBER" --json id --jq '.id')

  if [ -z "$ISSUE_NODE_ID" ]; then
    echo "  ✗ Error: Could not get issue node ID"
    return 1
  fi

  echo "  - Issue node ID: $ISSUE_NODE_ID"

  # Get project item ID for this issue
  echo "  - Finding project item..."
  PROJECT_ITEM_ID=$(GH_TOKEN="$PROJECT_TOKEN" gh api graphql -f query="
    query {
      node(id: \"$ISSUE_NODE_ID\") {
        ... on Issue {
          projectItems(first: 10) {
            nodes {
              id
              project {
                id
              }
            }
          }
        }
      }
    }
  " --jq ".data.node.projectItems.nodes[] | select(.project.id == \"$PROJECT_ID\") | .id")

  if [ -z "$PROJECT_ITEM_ID" ]; then
    echo "  - Issue not in project, adding it..."
    PROJECT_ITEM_ID=$(GH_TOKEN="$PROJECT_TOKEN" gh api graphql -f query="
      mutation {
        addProjectV2ItemById(input: {
          projectId: \"$PROJECT_ID\"
          contentId: \"$ISSUE_NODE_ID\"
        }) {
          item {
            id
          }
        }
      }
    " --jq '.data.addProjectV2ItemById.item.id')

    if [ -z "$PROJECT_ITEM_ID" ]; then
      echo "  ✗ Error: Could not add issue to project"
      return 1
    fi
    echo "  - Added to project with item ID: $PROJECT_ITEM_ID"
  else
    echo "  - Found project item ID: $PROJECT_ITEM_ID"
  fi

  # Update project status
  echo "  - Updating project status to '$STATUS'..."
  GH_TOKEN="$PROJECT_TOKEN" gh api graphql -f query="
    mutation {
      updateProjectV2ItemFieldValue(input: {
        projectId: \"$PROJECT_ID\"
        itemId: \"$PROJECT_ITEM_ID\"
        fieldId: \"$STATUS_FIELD_ID\"
        value: {
          singleSelectOptionId: \"$STATUS_OPTION_ID\"
        }
      }) {
        projectV2Item {
          id
        }
      }
    }
  " > /dev/null

  echo "  ✓ Successfully updated issue #${ISSUE_NUMBER}"
  echo "    - Status: $STATUS"
  if [ "$STATUS" = "in_progress" ] && [ -n "$ASSIGNEE" ]; then
    echo "    - Assigned to: @${ASSIGNEE}"
  fi
}

# Split comma-separated issue numbers and process each
IFS=',' read -ra ISSUES <<< "$ISSUE_NUMBERS"

SUCCESS_COUNT=0
FAIL_COUNT=0

for issue in "${ISSUES[@]}"; do
  # Trim whitespace
  issue=$(echo "$issue" | xargs)

  if update_single_issue "$issue"; then
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
  else
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
done

echo ""
echo "================================"
echo "Summary:"
echo "  ✓ Successful: $SUCCESS_COUNT"
if [ $FAIL_COUNT -gt 0 ]; then
  echo "  ✗ Failed: $FAIL_COUNT"
fi
echo "================================"

# Exit with error code if any failed
[ $FAIL_COUNT -eq 0 ]
