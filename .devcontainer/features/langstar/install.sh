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
    INSTALLER_PREFIX="/usr/local/bin"
    echo "Installing as root/privileged user to /usr/local/bin..."
else
    INSTALLER_PREFIX="$HOME/.local/bin"
    echo "Installing as non-root user to $HOME/.local/bin..."
    # Ensure the directory exists
    mkdir -p "$HOME/.local/bin"
fi

# Build installer arguments
INSTALLER_ARGS=(--prefix "${INSTALLER_PREFIX}")
if [ "${VERSION}" != "latest" ]; then
    INSTALLER_ARGS+=(--version "${VERSION}")
fi

echo "Running installer with: \"${INSTALLER_ARGS[@]}\""
curl -fsSL https://raw.githubusercontent.com/codekiln/langstar/main/scripts/install.sh | bash -s -- "${INSTALLER_ARGS[@]}"

# Verify installation by checking if the binary exists at the expected path
# Note: During container build, PATH may not be fully configured yet,
# so we verify by checking the file directly rather than using `command -v`
BINARY_PATH="${INSTALLER_PREFIX}/langstar"
if [ -x "${BINARY_PATH}" ]; then
    echo "✓ Langstar CLI installed successfully to ${BINARY_PATH}"
    "${BINARY_PATH}" --version
else
    echo "✗ Failed to install Langstar CLI"
    echo "Expected binary at: ${BINARY_PATH}"
    echo "Contents of ${INSTALLER_PREFIX}:"
    ls -la "${INSTALLER_PREFIX}" 2>/dev/null || echo "  (directory does not exist or is not accessible)"
    exit 1
fi

# Ensure the binary is accessible via PATH at runtime
# The containerEnv in devcontainer-feature.json adds both potential locations to PATH,
# but we also explicitly verify the expected location exists
echo ""
echo "Installation complete. The langstar command will be available in new shell sessions."
echo "Binary location: ${BINARY_PATH}"
