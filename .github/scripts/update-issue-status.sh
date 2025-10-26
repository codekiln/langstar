#!/bin/bash
# update-issue-status.sh
# Updates issue assignment and GitHub Project status when Claude starts work
#
# Usage:
#   ./update-issue-status.sh <issue_number> <status> [assignee]
#
# Arguments:
#   issue_number - The GitHub issue number
#   status       - The status to set: "in_progress" or "done"
#   assignee     - (Optional) GitHub username to assign (default: codekiln)
#
# Environment variables required:
#   GITHUB_TOKEN     - GitHub token with issues:write and project permissions
#   GITHUB_REPOSITORY - Repository in format "owner/repo"

set -e

# Configuration
PROJECT_ID="PVT_kwHOAAImgs4BGe4B"  # langstar project
STATUS_FIELD_ID="PVTSSF_lAHOAAImgs4BGe4Bzg3g-NQ"  # Status field
STATUS_TODO="f75ad846"
STATUS_IN_PROGRESS="47fc9ee4"
STATUS_DONE="98236657"

# Parse arguments
ISSUE_NUMBER="${1}"
STATUS="${2}"
ASSIGNEE="${3:-codekiln}"

if [ -z "$ISSUE_NUMBER" ] || [ -z "$STATUS" ]; then
  echo "Usage: $0 <issue_number> <status> [assignee]"
  echo "Status must be: in_progress or done"
  exit 1
fi

# Map status name to option ID
case "$STATUS" in
  in_progress)
    STATUS_OPTION_ID="$STATUS_IN_PROGRESS"
    ;;
  done)
    STATUS_OPTION_ID="$STATUS_DONE"
    ;;
  *)
    echo "Error: Invalid status '$STATUS'. Must be 'in_progress' or 'done'"
    exit 1
    ;;
esac

echo "Updating issue #${ISSUE_NUMBER}..."

# Assign issue if assignee provided and status is in_progress
if [ "$STATUS" = "in_progress" ] && [ -n "$ASSIGNEE" ]; then
  echo "  - Assigning to @${ASSIGNEE}..."
  gh issue edit "$ISSUE_NUMBER" --add-assignee "$ASSIGNEE" || {
    echo "Warning: Failed to assign issue"
  }
fi

# Get issue node ID
echo "  - Getting issue node ID..."
ISSUE_NODE_ID=$(gh issue view "$ISSUE_NUMBER" --json id --jq '.id')

if [ -z "$ISSUE_NODE_ID" ]; then
  echo "Error: Could not get issue node ID"
  exit 1
fi

echo "  - Issue node ID: $ISSUE_NODE_ID"

# Get project item ID for this issue
echo "  - Finding project item..."
PROJECT_ITEM_ID=$(gh api graphql -f query="
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
  PROJECT_ITEM_ID=$(gh api graphql -f query="
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
    echo "Error: Could not add issue to project"
    exit 1
  fi
  echo "  - Added to project with item ID: $PROJECT_ITEM_ID"
else
  echo "  - Found project item ID: $PROJECT_ITEM_ID"
fi

# Update project status
echo "  - Updating project status to '$STATUS'..."
gh api graphql -f query="
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

echo "✓ Successfully updated issue #${ISSUE_NUMBER}"
echo "  - Status: $STATUS"
if [ "$STATUS" = "in_progress" ]; then
  echo "  - Assigned to: @${ASSIGNEE}"
fi
