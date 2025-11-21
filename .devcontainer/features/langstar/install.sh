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

# Build installer arguments
INSTALLER_ARGS="--prefix /usr/local"
if [ "${VERSION}" != "latest" ]; then
    INSTALLER_ARGS="${INSTALLER_ARGS} --version ${VERSION}"
fi

curl -fsSL https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash -s -- ${INSTALLER_ARGS}

# Verify installation
if command -v langstar &> /dev/null; then
    echo "✓ Langstar CLI installed successfully"
    langstar --version
else
    echo "✗ Failed to install Langstar CLI"
    exit 1
fi
