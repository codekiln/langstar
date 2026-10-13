#!/bin/bash
# Setup Remote Repository Notes Directory
#
# Creates a structured notes directory for studying and documenting remote GitHub repositories.
#
# Usage: ./setup_repo_notes.sh <github-url>
# Example: ./setup_repo_notes.sh https://github.com/anthropics/claude-code

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print colored messages
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if URL is provided
if [ $# -eq 0 ]; then
    error "No GitHub URL provided"
    echo "Usage: $0 <github-url>"
    echo "Example: $0 https://github.com/anthropics/claude-code"
    exit 1
fi

GITHUB_URL="$1"

# Parse GitHub URL to extract org/user and repo name
# Handles formats:
# - https://github.com/owner/repo
# - https://github.com/owner/repo.git
# - git@github.com:owner/repo.git
if [[ "$GITHUB_URL" =~ github\.com[/:]([^/]+)/([^/]+)(\.git)?$ ]]; then
    ORG="${BASH_REMATCH[1]}"
    REPO="${BASH_REMATCH[2]}"
    REPO="${REPO%.git}"  # Remove .git suffix if present
else
    error "Invalid GitHub URL format"
    echo "Expected format: https://github.com/owner/repo"
    exit 1
fi

info "Repository: $ORG/$REPO"

# Detect if we're in a git worktree and find the root directory
ROOT_DIR="$(pwd)"
IN_WORKTREE=false

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    # In a worktree, .git in the working directory is a file (not a directory) containing "gitdir: ..."
    WORK_TREE_ROOT=$(git rev-parse --show-toplevel)
    GIT_FILE="$WORK_TREE_ROOT/.git"

    # Check if .git is a file pointing to a worktrees directory
    if [ -f "$GIT_FILE" ]; then
        # It's a file, read it to see if it points to a worktree
        GIT_CONTENT=$(cat "$GIT_FILE")
        if [[ "$GIT_CONTENT" == *"/worktrees/"* ]]; then
            IN_WORKTREE=true
            # Extract the main repo path from the worktree git file
            # Format: "gitdir: /workspace/.git/worktrees/branch-name"
            MAIN_GIT_DIR=$(echo "$GIT_CONTENT" | sed 's|gitdir: ||' | sed 's|/worktrees/.*||')
            ROOT_DIR=$(dirname "$MAIN_GIT_DIR")
            info "Detected worktree environment"
            info "Root directory: $ROOT_DIR"
        fi
    else
        # .git is a directory, we're in the main repository
        ROOT_DIR="$WORK_TREE_ROOT"
    fi
fi

# Define directory structure
# Clone directory: always in root (shared across worktrees)
CODE_DIR="$ROOT_DIR/reference/repo/$ORG/$REPO/code"

# Notes directory: in worktree if applicable, otherwise in root
if [ "$IN_WORKTREE" = true ]; then
    # Notes go in the worktree's local reference/ directory
    NOTES_DIR="$(pwd)/reference/repo/$ORG/$REPO/notes"
    BASE_DIR="$(pwd)/reference/repo/$ORG/$REPO"
else
    # Notes go in root alongside code
    NOTES_DIR="$ROOT_DIR/reference/repo/$ORG/$REPO/notes"
    BASE_DIR="$ROOT_DIR/reference/repo/$ORG/$REPO"
fi

# Create directory structure
info "Creating directory structure..."
mkdir -p "$NOTES_DIR"
mkdir -p "$CODE_DIR"
success "Created $BASE_DIR/"

# Clone the repository
info "Cloning repository..."
if [ -d "$CODE_DIR/.git" ]; then
    warn "Repository already cloned at $CODE_DIR"
    info "Pulling latest changes..."
    (cd "$CODE_DIR" && git pull)
else
    git clone "$GITHUB_URL" "$CODE_DIR"
    success "Cloned repository to $CODE_DIR/"
fi

# Create initial notes README
NOTES_README="$NOTES_DIR/README.md"
if [ -f "$NOTES_README" ]; then
    warn "Notes README already exists at $NOTES_README"
else
    info "Creating initial notes README..."
    cat > "$NOTES_README" << EOF
# $REPO

## Repository Information

- **Repository**: [$ORG/$REPO]($GITHUB_URL)
- **Date Created**: $(date +"%Y-%m-%d")
- **Cloned to**: \`$CODE_DIR\`

## Purpose

[Describe why you're studying this repository and what you hope to learn]

## Key Findings

[Document important discoveries, patterns, and insights]

## Architecture

[Describe the project structure and key components]

## Notes

[Add your notes here]

EOF
    success "Created $NOTES_README"
fi

# Update .gitignore to exclude code directories (always in root)
GITIGNORE_FILE="$ROOT_DIR/.gitignore"
GITIGNORE_PATTERN="reference/repo/**/code/"

info "Updating .gitignore..."
if [ ! -f "$GITIGNORE_FILE" ]; then
    warn ".gitignore not found at $ROOT_DIR, creating new one"
    touch "$GITIGNORE_FILE"
fi

if grep -qF "$GITIGNORE_PATTERN" "$GITIGNORE_FILE"; then
    warn ".gitignore already contains pattern: $GITIGNORE_PATTERN"
else
    echo "" >> "$GITIGNORE_FILE"
    echo "# Remote repository code directories (gitignored, notes are committed)" >> "$GITIGNORE_FILE"
    echo "$GITIGNORE_PATTERN" >> "$GITIGNORE_FILE"
    success "Added $GITIGNORE_PATTERN to .gitignore"
fi

# Summary
echo ""
echo "═══════════════════════════════════════════════════════════"
success "Setup complete!"
echo "═══════════════════════════════════════════════════════════"
echo ""

if [ "$IN_WORKTREE" = true ]; then
    echo "Worktree-aware structure:"
    echo "  📝 Notes (worktree-local): $NOTES_DIR"
    echo "  💻 Code (shared in root): $CODE_DIR"
    echo ""
    info "Notes are local to this worktree and can be committed with your branch work"
    info "Code is shared across all worktrees (saves disk space)"
else
    echo "Directory structure:"
    echo "  📁 $BASE_DIR/"
    echo "  ├── 📝 notes/          (committed - add your documentation here)"
    echo "  │   └── README.md"
    echo "  └── 💻 code/           (gitignored - cloned repository)"
fi

echo ""
info "Next steps:"
echo "  1. Add your notes to: $NOTES_README"
echo "  2. Explore the code in: $CODE_DIR/"
echo "  3. Commit your notes: git add $NOTES_DIR && git commit -m '📚 docs: add notes for $ORG/$REPO'"
echo ""
