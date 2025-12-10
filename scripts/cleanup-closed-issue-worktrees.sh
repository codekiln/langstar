#!/bin/bash
#
# Cleanup worktrees tied to closed GitHub issues
#
# Usage:
#   ./cleanup-closed-issue-worktrees.sh
#
# Requirements:
#   - GitHub CLI (gh) must be installed and authenticated
#   - Branch names should follow format: m<milestone>-p<parent>-i<issue>-<slug>
#   - Examples: m8-p123-i234-issue-slug, p123-i234-issue-slug, i234-issue-slug
#   - Also supports legacy format: username/issue_num-description
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

# Validate GitHub CLI is installed and authenticated
if ! command -v gh &> /dev/null; then
  echo "❌ Error: GitHub CLI (gh) is not installed. Please install it first."
  exit 1
fi

if ! gh auth status &> /dev/null; then
  echo "❌ Error: Not authenticated with GitHub CLI. Run 'gh auth login' first."
  exit 1
fi

# Get repository root and name
repo_root=$(git rev-parse --show-toplevel)
repo=$(gh repo view --json nameWithOwner --jq '.nameWithOwner')

echo "🔍 Checking worktrees for closed issues..."
echo ""

# Parse worktrees and check issue status using robust parsing
current_path=""
current_branch=""
found_worktree=false
processed_any=false

while IFS= read -r line; do
  if [[ $line =~ ^worktree\ (.+)$ ]]; then
    current_path="${BASH_REMATCH[1]}"
    found_worktree=true
  elif [[ $line =~ ^branch\ refs/heads/(.+)$ ]]; then
    current_branch="${BASH_REMATCH[1]}"
  fi

  # When both path and branch are set, process
  if $found_worktree && [ -n "$current_branch" ]; then
    # Skip main workspace (repo root)
    if [ "$current_path" = "$repo_root" ]; then
      # Reset for next worktree
      current_path=""
      current_branch=""
      found_worktree=false
      continue
    fi

    branch="$current_branch"
    path="$current_path"
    processed_any=true

    # Extract issue number from branch name
    # Supports new format: m<milestone>-p<parent>-i<issue>-<slug> (and variations)
    # Also supports old format: username/issue_num-description
    issue_num=""

    # Try new format first: look for i<number>- pattern
    if [[ $branch =~ i([0-9]+)- ]]; then
      issue_num="${BASH_REMATCH[1]}"
    # Fallback to old format: username/issue_num-slug
    elif [[ $branch =~ /([0-9]+)- ]]; then
      issue_num="${BASH_REMATCH[1]}"
    fi

    if [ -n "$issue_num" ]; then

      echo -n "Checking worktree: ${path##*/} (issue #${issue_num})... "

      # Check issue state with explicit repository context
      if issue_state=$(gh issue view "$issue_num" --repo "$repo" --json state --jq '.state' 2>/dev/null); then
        if [ "$issue_state" = "CLOSED" ]; then
          echo -e "${RED}CLOSED${NC}"
          echo "  → Removing worktree: $path"

          # Ensure we're not in the worktree being removed
          # Check for exact match or if PWD is a subdirectory
          if [[ "$PWD" == "$path" ]] || [[ "$PWD/" == "$path/"* ]]; then
            echo -e "  ${YELLOW}⚠ Currently in this worktree, switching to repository root${NC}"
            cd "$repo_root"
          fi

          # Remove the worktree
          if git worktree remove "$path" 2>/dev/null; then
            echo -e "  ${GREEN}✓ Removed successfully${NC}"

            # Optionally delete the branch (commented out by default)
            # git branch -D "$branch" 2>/dev/null && echo "  ✓ Deleted branch: $branch"
          elif git worktree remove --force "$path" 2>/dev/null; then
            echo -e "  ${YELLOW}✓ Force removed (had uncommitted changes)${NC}"
          else
            echo -e "  ${RED}✗ Failed to remove${NC}"
            echo "  Try manually: git worktree remove --force $path"
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

    # Reset for next worktree
    current_path=""
    current_branch=""
    found_worktree=false
  fi
done < <(git worktree list --porcelain)

if [ "$processed_any" = false ]; then
  echo "✓ No additional worktrees found"
fi

# Prune stale worktree references
echo "🧹 Pruning stale worktree references..."
git worktree prune --verbose

echo ""
echo "✨ Cleanup complete!"
echo ""
echo "Remaining worktrees:"
git worktree list
