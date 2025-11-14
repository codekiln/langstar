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

# Define directory structure
BASE_DIR="reference/repo/$ORG/$REPO"
NOTES_DIR="$BASE_DIR/notes"
CODE_DIR="$BASE_DIR/code"

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
- **Cloned to**: \`../$CODE_DIR\`

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

# Update .gitignore to exclude code directories
GITIGNORE_FILE=".gitignore"
GITIGNORE_PATTERN="reference/repo/**/code/"

info "Updating .gitignore..."
if [ ! -f "$GITIGNORE_FILE" ]; then
    warn ".gitignore not found, creating new one"
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
echo "Directory structure:"
echo "  📁 $BASE_DIR/"
echo "  ├── 📝 notes/          (committed - add your documentation here)"
echo "  │   └── README.md"
echo "  └── 💻 code/           (gitignored - cloned repository)"
echo ""
info "Next steps:"
echo "  1. Add your notes to: $NOTES_README"
echo "  2. Explore the code in: $CODE_DIR/"
echo "  3. Commit your notes: git add $NOTES_DIR && git commit -m '📚 docs: add notes for $ORG/$REPO'"
echo ""
