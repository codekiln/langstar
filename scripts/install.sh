#!/usr/bin/env bash
# Official installer script for langstar CLI
# https://github.com/codekiln/langstar
#
# Usage:
#   curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | sh
#
# Or download and run:
#   curl -LO https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh
#   chmod +x install.sh
#   ./install.sh
#
# Options:
#   --version VERSION    Install specific version (e.g., --version 0.2.0)
#   --prefix DIR         Install to custom directory (default: /usr/local/bin or ~/.local/bin)
#   --help              Show this help message
#
# Examples:
#   ./install.sh                    # Install latest version
#   ./install.sh --version 0.2.0    # Install specific version
#   ./install.sh --prefix ~/bin     # Install to custom location

set -e

# Configuration
REPO="codekiln/langstar"
BINARY_NAME="langstar"
DEFAULT_SYSTEM_INSTALL_DIR="/usr/local/bin"
DEFAULT_USER_INSTALL_DIR="$HOME/.local/bin"

# Colors for output (only when stdout is a terminal)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

# Output functions
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

die() {
    error "$1"
    exit 1
}

# Print help message
print_help() {
    cat <<EOF
${BOLD}langstar installer${NC}

Install the langstar CLI binary from GitHub releases.

${BOLD}USAGE:${NC}
    $0 [OPTIONS]

${BOLD}OPTIONS:${NC}
    --version VERSION    Install specific version (e.g., --version 0.2.0)
    --prefix DIR         Install to custom directory (default: /usr/local/bin or ~/.local/bin)
    --help              Show this help message

${BOLD}EXAMPLES:${NC}
    $0                    # Install latest version
    $0 --version 0.2.0    # Install specific version
    $0 --prefix ~/bin     # Install to custom location

${BOLD}ENVIRONMENT:${NC}
    LANGSTAR_INSTALL_DIR    Override default installation directory

For more information, visit: https://github.com/$REPO
EOF
}

# Parse command line arguments
parse_args() {
    VERSION=""
    CUSTOM_PREFIX=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --prefix)
                CUSTOM_PREFIX="$2"
                shift 2
                ;;
            --help|-h)
                print_help
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                echo ""
                print_help
                exit 1
                ;;
        esac
    done
}

# Detect platform (OS + architecture)
detect_platform() {
    local os arch platform

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64|amd64)
                    # Prefer musl build for Linux (fully static, no dependencies)
                    platform="x86_64-linux-musl"
                    ;;
                aarch64|arm64)
                    # ARM64 musl build for Linux (Docker on Apple Silicon, ARM servers)
                    platform="aarch64-linux-musl"
                    ;;
                *)
                    die "Unsupported Linux architecture: $arch (only x86_64 and aarch64 are supported)"
                    ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64)
                    platform="x86_64-macos"
                    ;;
                arm64|aarch64)
                    platform="aarch64-macos"
                    ;;
                *)
                    die "Unsupported macOS architecture: $arch"
                    ;;
            esac
            ;;
        *)
            die "Unsupported operating system: $os (only Linux and macOS are supported)"
            ;;
    esac

    echo "$platform"
}

# Check if required tools are available
check_dependencies() {
    local missing_deps=()

    # Check for download tool (curl or wget)
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        missing_deps+=("curl or wget")
    fi

    # Check for tar
    if ! command -v tar &> /dev/null; then
        missing_deps+=("tar")
    fi

    # Check for checksum tool (shasum or sha256sum)
    if ! command -v shasum &> /dev/null && ! command -v sha256sum &> /dev/null; then
        missing_deps+=("shasum or sha256sum")
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        error "Missing required dependencies: ${missing_deps[*]}"
        echo ""
        echo "Please install the missing dependencies and try again."
        exit 1
    fi
}

# Download a file using curl or wget
download_file() {
    local url="$1"
    local output="$2"

    if command -v curl &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSfL "$url" -o "$output"
    elif command -v wget &> /dev/null; then
        wget -q --https-only "$url" -O "$output"
    else
        die "Neither curl nor wget is available"
    fi
}

# Get the latest release version from GitHub API
get_latest_version() {
    local api_url="https://api.github.com/repos/$REPO/releases/latest"
    local version

    info "Fetching latest version from GitHub..."

    if command -v curl &> /dev/null; then
        version=$(curl -sSf "$api_url" | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/')
    elif command -v wget &> /dev/null; then
        version=$(wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/')
    else
        die "Cannot fetch latest version: neither curl nor wget is available"
    fi

    if [ -z "$version" ]; then
        die "Failed to fetch latest version from GitHub"
    fi

    echo "$version"
}

# Verify SHA256 checksum
verify_checksum() {
    local file="$1"
    local checksum_file="$2"

    info "Verifying SHA256 checksum..."

    if command -v shasum &> /dev/null; then
        shasum -a 256 -c "$checksum_file" &> /dev/null
    elif command -v sha256sum &> /dev/null; then
        sha256sum -c "$checksum_file" &> /dev/null
    else
        warn "Cannot verify checksum: neither shasum nor sha256sum is available"
        warn "Proceeding without checksum verification (not recommended)"
        return 0
    fi

    if [ $? -ne 0 ]; then
        die "Checksum verification failed! The downloaded file may be corrupted or tampered with."
    fi

    success "Checksum verified successfully"
}

# Check if langstar is already installed
check_existing_installation() {
    if command -v "$BINARY_NAME" &> /dev/null; then
        local installed_version
        installed_version=$("$BINARY_NAME" --version 2>/dev/null | head -n1 | awk '{print $NF}' || echo "unknown")

        info "Found existing installation: $BINARY_NAME $installed_version"
        echo "$installed_version"
    else
        echo ""
    fi
}

# Determine installation directory
determine_install_dir() {
    local custom_prefix="$1"

    # Priority:
    # 1. Custom prefix from --prefix flag
    # 2. LANGSTAR_INSTALL_DIR environment variable
    # 3. Try system-wide (/usr/local/bin) if writable
    # 4. Fall back to user-local (~/.local/bin)

    if [ -n "$custom_prefix" ]; then
        echo "$custom_prefix"
        return
    fi

    if [ -n "${LANGSTAR_INSTALL_DIR:-}" ]; then
        echo "$LANGSTAR_INSTALL_DIR"
        return
    fi

    # Test if we can write to system directory
    if [ -w "$DEFAULT_SYSTEM_INSTALL_DIR" ] || [ "$(id -u)" -eq 0 ]; then
        echo "$DEFAULT_SYSTEM_INSTALL_DIR"
    else
        # Check if we can use sudo
        if command -v sudo &> /dev/null && sudo -n true 2>/dev/null; then
            echo "$DEFAULT_SYSTEM_INSTALL_DIR"
        else
            warn "Cannot write to $DEFAULT_SYSTEM_INSTALL_DIR (no sudo access)"
            info "Falling back to user-local installation: $DEFAULT_USER_INSTALL_DIR"
            echo "$DEFAULT_USER_INSTALL_DIR"
        fi
    fi
}

# Install the binary
install_binary() {
    local binary_path="$1"
    local install_dir="$2"
    local needs_sudo=false

    # Create install directory if it doesn't exist
    if [ ! -d "$install_dir" ]; then
        info "Creating installation directory: $install_dir"
        if [ -w "$(dirname "$install_dir")" ]; then
            mkdir -p "$install_dir"
        else
            sudo mkdir -p "$install_dir"
            needs_sudo=true
        fi
    fi

    # Determine if we need sudo for installation
    if [ ! -w "$install_dir" ]; then
        needs_sudo=true
    fi

    # Install the binary
    info "Installing $BINARY_NAME to $install_dir..."

    if [ "$needs_sudo" = true ]; then
        sudo install -m 755 "$binary_path" "$install_dir/$BINARY_NAME"
    else
        install -m 755 "$binary_path" "$install_dir/$BINARY_NAME"
    fi

    success "Installed $BINARY_NAME to $install_dir/$BINARY_NAME"
}

# Check if install directory is in PATH
check_path() {
    local install_dir="$1"

    if ! echo "$PATH" | grep -q "$install_dir"; then
        warn "Installation directory is not in your PATH: $install_dir"
        echo ""
        echo "To use $BINARY_NAME, add the following to your shell profile:"
        echo ""
        echo "    export PATH=\"$install_dir:\$PATH\""
        echo ""

        # Suggest which file to edit based on shell
        case "$SHELL" in
            */bash)
                echo "Add this to: ~/.bashrc or ~/.bash_profile"
                ;;
            */zsh)
                echo "Add this to: ~/.zshrc"
                ;;
            */fish)
                echo "Add this to: ~/.config/fish/config.fish"
                echo "Or run: fish_add_path $install_dir"
                ;;
            *)
                echo "Add this to your shell's configuration file"
                ;;
        esac
        echo ""
    fi
}

# Main installation logic
main() {
    echo -e "${BOLD}langstar installer${NC}"
    echo ""

    # Parse command line arguments
    parse_args "$@"

    # Check dependencies
    check_dependencies

    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: $platform"

    # Get version (latest or specified)
    local version="${VERSION:-$(get_latest_version)}"
    info "Installing version: $version"

    # Check for existing installation
    local existing_version
    existing_version=$(check_existing_installation)

    if [ -n "$existing_version" ]; then
        if [ "$existing_version" = "$version" ]; then
            info "Version $version is already installed"
            info "Re-installing to ensure clean installation..."
        else
            info "Upgrading from $existing_version to $version"
        fi
    fi

    # Construct download URLs
    local archive_name="langstar-${version}-${platform}.tar.gz"
    local base_url="https://github.com/$REPO/releases/download/v${version}"
    local archive_url="$base_url/$archive_name"
    local checksum_url="$archive_url.sha256"

    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    info "Downloading $archive_name..."

    # Download archive
    if ! download_file "$archive_url" "$temp_dir/$archive_name"; then
        die "Failed to download $archive_url"
    fi

    # Download checksum
    if ! download_file "$checksum_url" "$temp_dir/$archive_name.sha256"; then
        warn "Failed to download checksum file (proceeding without verification)"
    else
        # Verify checksum
        cd "$temp_dir"
        verify_checksum "$archive_name" "$archive_name.sha256"
        cd - > /dev/null
    fi

    # Extract archive
    info "Extracting archive..."
    tar -xzf "$temp_dir/$archive_name" -C "$temp_dir"

    # Find the binary in the extracted files
    local binary_path
    if [ -f "$temp_dir/$BINARY_NAME" ]; then
        binary_path="$temp_dir/$BINARY_NAME"
    elif [ -f "$temp_dir/langstar-${version}-${platform}/$BINARY_NAME" ]; then
        binary_path="$temp_dir/langstar-${version}-${platform}/$BINARY_NAME"
    else
        die "Binary not found in archive"
    fi

    # Determine installation directory
    local install_dir
    install_dir=$(determine_install_dir "$CUSTOM_PREFIX")

    # Install the binary
    install_binary "$binary_path" "$install_dir"

    # Verify installation
    if command -v "$BINARY_NAME" &> /dev/null; then
        local installed_version
        installed_version=$("$BINARY_NAME" --version 2>/dev/null | head -n1 | awk '{print $NF}' || echo "unknown")
        success "Installation complete! langstar $installed_version is ready to use."
        echo ""
        echo "Run '$BINARY_NAME --help' to get started."
    else
        success "Installation complete!"
        echo ""
        warn "However, '$BINARY_NAME' command is not found in PATH"
        check_path "$install_dir"
    fi
}

# Run main function
main "$@"
