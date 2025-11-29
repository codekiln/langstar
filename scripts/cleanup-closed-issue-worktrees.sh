#!/bin/bash
#
# Cleanup worktrees tied to closed GitHub issues
#
# This script:
# 1. Lists all worktrees
# 2. Extracts issue numbers from branch names
# 3. Checks if those issues are closed
# 4. Removes worktrees for closed issues

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 Checking worktrees for closed issues..."
echo ""

# Get list of worktrees (skip the main workspace line)
worktrees=$(git worktree list --porcelain | grep -E '^worktree|^branch' | paste -d ' ' - - | grep -v "^worktree /workspace branch")

if [ -z "$worktrees" ]; then
  echo "✓ No additional worktrees found"
  exit 0
fi

# Parse worktrees and check issue status
while read -r line; do
  # Extract path and branch
  path=$(echo "$line" | awk '{print $2}')
  branch=$(echo "$line" | awk '{print $4}' | sed 's|refs/heads/||')

  # Extract issue number from branch name (format: username/issue_num-description)
  if [[ $branch =~ /([0-9]+)- ]]; then
    issue_num="${BASH_REMATCH[1]}"

    echo -n "Checking worktree: ${path##*/} (issue #${issue_num})... "

    # Check issue state
    if issue_state=$(gh issue view "$issue_num" --json state --jq '.state' 2>/dev/null); then
      if [ "$issue_state" = "CLOSED" ]; then
        echo -e "${RED}CLOSED${NC}"
        echo "  → Removing worktree: $path"

        # Ensure we're not in the worktree being removed
        if [[ "$PWD" == "$path"* ]]; then
          echo -e "  ${YELLOW}⚠ Currently in this worktree, switching to main workspace${NC}"
          cd /workspace
        fi

        # Remove the worktree
        if git worktree remove "$path" 2>/dev/null; then
          echo -e "  ${GREEN}✓ Removed successfully${NC}"

          # Optionally delete the branch (commented out by default)
          # git branch -D "$branch" 2>/dev/null && echo "  ✓ Deleted branch: $branch"
        else
          echo -e "  ${YELLOW}⚠ Failed to remove (may have uncommitted changes)${NC}"
          echo "  Run: git worktree remove --force $path"
        fi
      else
        echo -e "${GREEN}OPEN${NC}"
      fi
    else
      echo -e "${YELLOW}UNKNOWN (could not query GitHub)${NC}"
    fi
  else
    echo "Skipping: ${path##*/} (could not extract issue number from branch: $branch)"
  fi
  echo ""
done <<< "$worktrees"

# Prune stale worktree references
echo "🧹 Pruning stale worktree references..."
git worktree prune --verbose

echo ""
echo "✨ Cleanup complete!"
echo ""
echo "Remaining worktrees:"
git worktree list
