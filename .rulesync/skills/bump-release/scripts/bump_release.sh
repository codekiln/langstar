#!/usr/bin/env bash
#
# bump_release.sh - Automated release workflow for Rust projects
#
# This script orchestrates the complete release workflow:
# 1. Analyzes commits since last release
# 2. Determines semantic version bump
# 3. Updates Cargo.toml version(s)
# 4. Generates changelog using git-cliff
# 5. Creates git commit and annotated tag
# 6. Pushes to remote and creates GitHub release
#
# Requirements:
# - git
# - gh (GitHub CLI)
# - git-cliff
# - Python 3.7+
# - cargo (for validation)

set -euo pipefail

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Default values
DRY_RUN=false
SKIP_CHECKS=false
SKIP_PUSH=false
PRERELEASE=false
MAIN_BRANCH="main"
CHANGELOG_FILE="CHANGELOG.md"

# Functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*" >&2
}

log_error() {
    echo -e "${RED}✗${NC} $*" >&2
}

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Automated release workflow for Rust projects using Conventional Emoji Commits.

OPTIONS:
    -d, --dry-run           Show what would happen without making changes
    -s, --skip-checks       Skip pre-release checks (dirty working tree, branch)
    -n, --no-push           Skip pushing to remote (local release only)
    -p, --prerelease        Mark as pre-release on GitHub
    -b, --branch BRANCH     Main branch name (default: main)
    -c, --changelog FILE    Changelog file path (default: CHANGELOG.md)
    -h, --help              Show this help message

EXAMPLES:
    # Dry run to see what would happen
    $(basename "$0") --dry-run

    # Create a release
    $(basename "$0")

    # Create a release but don't push to GitHub
    $(basename "$0") --no-push

    # Mark as pre-release
    $(basename "$0") --prerelease

REQUIREMENTS:
    - git-cliff: Install with 'cargo install git-cliff'
    - gh: GitHub CLI (https://cli.github.com/)
    - Python 3.7+

For more information, see the SKILL.md file.
EOF
}

check_requirements() {
    local missing=()

    # Check required commands
    for cmd in git gh git-cliff python3 cargo; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing required commands: ${missing[*]}"
        log_info "Install missing tools and try again"
        return 1
    fi

    # Check if we're in a git repository
    if ! git rev-parse --git-dir &>/dev/null; then
        log_error "Not in a git repository"
        return 1
    fi

    # Check if gh is authenticated
    if ! gh auth status &>/dev/null; then
        log_error "GitHub CLI (gh) is not authenticated"
        log_info "Run: gh auth login"
        return 1
    fi

    return 0
}

check_working_tree() {
    if [[ -n $(git status --porcelain) ]]; then
        log_error "Working tree is dirty. Commit or stash changes first."
        git status --short
        return 1
    fi
    log_success "Working tree is clean"
    return 0
}

check_branch() {
    local current_branch
    current_branch=$(git rev-parse --abbrev-ref HEAD)

    if [[ "$current_branch" != "$MAIN_BRANCH" ]]; then
        log_error "Not on $MAIN_BRANCH branch (currently on: $current_branch)"
        log_info "Switch to $MAIN_BRANCH or use --branch to specify a different branch"
        return 1
    fi
    log_success "On $MAIN_BRANCH branch"
    return 0
}

fetch_tags() {
    log_info "Fetching tags from remote..."
    if git fetch --tags &>/dev/null; then
        log_success "Tags fetched"
        return 0
    else
        log_warning "Failed to fetch tags (continuing anyway)"
        return 0
    fi
}

analyze_commits() {
    log_info "Analyzing commits since last release..."

    local analysis
    if ! analysis=$("$SCRIPT_DIR/analyze_commits.py" --format json 2>&1); then
        log_error "Failed to analyze commits"
        echo "$analysis" >&2
        return 1
    fi

    echo "$analysis"
}

bump_version_files() {
    local new_version="$1"
    local dry_run_flag=""

    if [[ "$DRY_RUN" == "true" ]]; then
        dry_run_flag="--dry-run"
    fi

    log_info "Updating version in Cargo.toml files..."

    if "$SCRIPT_DIR/bump_version.py" "$new_version" $dry_run_flag --workspace-deps --verbose; then
        log_success "Version updated to $new_version"
        return 0
    else
        log_error "Failed to update version"
        return 1
    fi
}

validate_cargo() {
    log_info "Validating Cargo.toml changes..."

    if cargo check --quiet 2>&1; then
        log_success "Cargo validation passed"
        return 0
    else
        log_error "Cargo validation failed"
        return 1
    fi
}

generate_changelog() {
    local new_version="$1"
    local last_tag="$2"

    log_info "Generating changelog for v$new_version..."

    # Check if git-cliff config exists
    if [[ ! -f "cliff.toml" && ! -f ".cliff.toml" ]]; then
        log_warning "No cliff.toml found, using default configuration"
    fi

    # Generate changelog for this release
    local range="${last_tag:+$last_tag..}HEAD"
    local changelog_content

    if ! changelog_content=$(git-cliff --tag "v$new_version" --strip all "$range" 2>&1); then
        log_error "Failed to generate changelog"
        echo "$changelog_content" >&2
        return 1
    fi

    # Update or create CHANGELOG.md
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Changelog content (dry-run):"
        echo "$changelog_content"
    else
        # Prepend to existing changelog or create new file
        if [[ -f "$CHANGELOG_FILE" ]]; then
            # Insert after header if it exists
            if grep -q "^# Changelog" "$CHANGELOG_FILE"; then
                local temp_file
                temp_file=$(mktemp)
                awk -v new="$changelog_content" '
                    /^# Changelog/ { print; print ""; print new; print ""; next }
                    { print }
                ' "$CHANGELOG_FILE" > "$temp_file"
                mv "$temp_file" "$CHANGELOG_FILE"
            else
                # No header, just prepend
                echo -e "$changelog_content\n\n$(cat "$CHANGELOG_FILE")" > "$CHANGELOG_FILE"
            fi
        else
            # Create new changelog
            {
                echo "# Changelog"
                echo ""
                echo "$changelog_content"
            } > "$CHANGELOG_FILE"
        fi

        log_success "Changelog updated"
    fi

    return 0
}

create_release_commit() {
    local new_version="$1"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Would create commit for version v$new_version"
        return 0
    fi

    log_info "Creating release commit..."

    # Stage changes
    git add Cargo.toml Cargo.lock "$CHANGELOG_FILE" || true
    # Also stage any workspace member Cargo.toml files
    git add "**/Cargo.toml" 2>/dev/null || true

    # Create commit
    git commit -m "🔖 release: bump version to v$new_version

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>" || {
        log_error "Failed to create commit"
        return 1
    }

    log_success "Release commit created"
    return 0
}

create_git_tag() {
    local new_version="$1"
    local tag="v$new_version"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Would create tag: $tag"
        return 0
    fi

    log_info "Creating annotated tag: $tag..."

    if git tag -a "$tag" -m "Release $tag"; then
        log_success "Tag created: $tag"
        return 0
    else
        log_error "Failed to create tag"
        return 1
    fi
}

push_to_remote() {
    local new_version="$1"
    local tag="v$new_version"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Would push commit and tag to remote"
        return 0
    fi

    if [[ "$SKIP_PUSH" == "true" ]]; then
        log_warning "Skipping push to remote (--no-push flag)"
        return 0
    fi

    log_info "Pushing to remote..."

    if git push && git push origin "$tag"; then
        log_success "Pushed to remote"
        return 0
    else
        log_error "Failed to push to remote"
        log_info "You can manually push with: git push && git push origin $tag"
        return 1
    fi
}

create_github_release() {
    local new_version="$1"
    local tag="v$new_version"
    local prerelease_flag=""

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Would create GitHub release for $tag"
        return 0
    fi

    if [[ "$SKIP_PUSH" == "true" ]]; then
        log_info "Skipping GitHub release creation (--no-push flag)"
        return 0
    fi

    if [[ "$PRERELEASE" == "true" ]]; then
        prerelease_flag="--prerelease"
    fi

    log_info "Creating GitHub release..."

    # Extract changelog for this version
    local release_notes
    release_notes=$(git-cliff --tag "$tag" --strip all --latest 2>&1) || {
        log_warning "Failed to extract release notes, using tag message"
        release_notes="Release $tag"
    }

    if gh release create "$tag" \
        --title "Release $tag" \
        --notes "$release_notes" \
        $prerelease_flag; then
        log_success "GitHub release created: $tag"
        return 0
    else
        log_error "Failed to create GitHub release"
        log_info "You can manually create it with: gh release create $tag"
        return 1
    fi
}

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
        -d | --dry-run)
            DRY_RUN=true
            shift
            ;;
        -s | --skip-checks)
            SKIP_CHECKS=true
            shift
            ;;
        -n | --no-push)
            SKIP_PUSH=true
            shift
            ;;
        -p | --prerelease)
            PRERELEASE=true
            shift
            ;;
        -b | --branch)
            MAIN_BRANCH="$2"
            shift 2
            ;;
        -c | --changelog)
            CHANGELOG_FILE="$2"
            shift 2
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
        esac
    done

    # Print banner
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║   Rust Release Automation Tool         ║"
    echo "║   Conventional Emoji Commits           ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    if [[ "$DRY_RUN" == "true" ]]; then
        log_warning "DRY RUN MODE - No changes will be made"
        echo ""
    fi

    # Check requirements
    check_requirements || exit 1

    # Pre-release checks
    if [[ "$SKIP_CHECKS" != "true" ]]; then
        check_working_tree || exit 1
        check_branch || exit 1
    else
        log_warning "Skipping pre-release checks"
    fi

    # Fetch tags
    fetch_tags

    # Analyze commits
    local analysis
    analysis=$(analyze_commits) || exit 1

    local current_version
    local bump_type
    local new_version
    local commit_count
    local last_tag

    current_version=$(echo "$analysis" | python3 -c "import sys, json; print(json.load(sys.stdin)['current_version'])")
    bump_type=$(echo "$analysis" | python3 -c "import sys, json; print(json.load(sys.stdin)['bump_type'])")
    new_version=$(echo "$analysis" | python3 -c "import sys, json; print(json.load(sys.stdin)['new_version'])")
    commit_count=$(echo "$analysis" | python3 -c "import sys, json; print(json.load(sys.stdin)['commit_count'])")
    last_tag=$(echo "$analysis" | python3 -c "import sys, json; print(json.load(sys.stdin).get('last_tag', 'None'))")

    echo ""
    log_info "Release Analysis:"
    echo "  Current version: $current_version"
    echo "  Last tag:        ${last_tag:-None}"
    echo "  Commits:         $commit_count"
    echo "  Bump type:       $bump_type"
    echo "  New version:     $new_version"
    echo ""

    # Check if there's anything to release
    if [[ "$bump_type" == "none" ]]; then
        log_warning "No releasable commits found since last release"
        log_info "Only found commits of types: docs, style, refactor, test, build, ci, chore"
        exit 0
    fi

    if [[ "$current_version" == "$new_version" ]]; then
        log_warning "Version would not change ($current_version → $new_version)"
        log_info "This usually means no feat/fix/breaking commits since last release"
        exit 0
    fi

    # Execute release workflow
    bump_version_files "$new_version" || exit 1

    if [[ "$DRY_RUN" != "true" ]]; then
        validate_cargo || exit 1
    fi

    generate_changelog "$new_version" "$last_tag" || exit 1

    create_release_commit "$new_version" || exit 1

    create_git_tag "$new_version" || exit 1

    push_to_remote "$new_version" || exit 1

    create_github_release "$new_version" || exit 1

    # Success!
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║          Release Successful!            ║"
    echo "╚════════════════════════════════════════╝"
    echo ""
    log_success "Released v$new_version"
    echo ""

    if [[ "$SKIP_PUSH" != "true" && "$DRY_RUN" != "true" ]]; then
        log_info "View release: $(gh release view "v$new_version" --web --json url -q .url 2>/dev/null || echo 'Check GitHub')"
    fi

    echo ""
}

# Run main
main "$@"
