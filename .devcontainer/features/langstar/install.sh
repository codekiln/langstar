#!/bin/bash
set -e

# Get version from feature options (defaults to "latest")
VERSION="${VERSION:-latest}"

echo "Installing Langstar CLI ${VERSION}..."

# Download and execute the official installer
# The installer handles:
# - Architecture detection (x86_64, ARM64)
# - Version resolution (latest or specific version)
# - Binary download from GitHub releases
# - Installation to specified prefix

# Detect if we're running as root or have write access to /usr/local
# This ensures compatibility with both local devcontainers and GitHub Codespaces
if [ "$(id -u)" -eq 0 ] || [ -w "/usr/local/bin" ]; then
    INSTALLER_PREFIX="/usr/local"
    echo "Installing as root/privileged user to /usr/local..."
else
    INSTALLER_PREFIX="$HOME/.local"
    echo "Installing as non-root user to $HOME/.local..."
    # Ensure the directory exists
    mkdir -p "$HOME/.local/bin"
fi

# Build installer arguments
INSTALLER_ARGS="--prefix ${INSTALLER_PREFIX}"
if [ "${VERSION}" != "latest" ]; then
    INSTALLER_ARGS="${INSTALLER_ARGS} --version ${VERSION}"
fi

echo "Running installer with: ${INSTALLER_ARGS}"
curl -fsSL https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash -s -- ${INSTALLER_ARGS}

# Verify installation
if command -v langstar &> /dev/null; then
    echo "✓ Langstar CLI installed successfully"
    langstar --version
else
    echo "✗ Failed to install Langstar CLI"
    echo "Note: If langstar is not in PATH, you may need to add ${INSTALLER_PREFIX}/bin to your PATH"
    echo "The installer should have provided instructions for this."
    exit 1
fi
