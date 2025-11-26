#!/bin/bash
# Smoke tests for langstar devcontainer feature
# Verifies that the langstar CLI is installed correctly and basic commands work
set -e

echo "======================================"
echo "Running langstar feature smoke tests"
echo "======================================"
echo ""

# Test 1: Verify binary exists at expected location(s)
echo "Test 1: Verify langstar binary exists..."

# Check common installation locations
BINARY_FOUND=false
BINARY_PATH=""

# Check /usr/local/bin first (root/privileged install)
if [ -x "/usr/local/bin/langstar" ]; then
    BINARY_FOUND=true
    BINARY_PATH="/usr/local/bin/langstar"
    echo "✓ langstar binary found at /usr/local/bin/langstar"
# Check user-local install location
elif [ -x "${HOME}/.local/bin/langstar" ]; then
    BINARY_FOUND=true
    BINARY_PATH="${HOME}/.local/bin/langstar"
    echo "✓ langstar binary found at ${HOME}/.local/bin/langstar"
# Also check if it's in PATH (for containerEnv to work)
elif command -v langstar &> /dev/null; then
    BINARY_FOUND=true
    BINARY_PATH=$(command -v langstar)
    echo "✓ langstar binary found in PATH"
    echo "  Location: ${BINARY_PATH}"
fi

if [ "$BINARY_FOUND" = false ]; then
    echo "✗ FAILED: langstar binary not found"
    echo "  Checked locations:"
    echo "    - /usr/local/bin/langstar"
    echo "    - ${HOME}/.local/bin/langstar"
    echo "    - PATH lookup via 'command -v langstar'"
    echo ""
    echo "  Debug info:"
    echo "    PATH=${PATH}"
    echo "    Contents of /usr/local/bin:"
    ls -la /usr/local/bin/ 2>/dev/null | grep -E "langstar|total" || echo "      (not accessible)"
    echo "    Contents of ${HOME}/.local/bin:"
    ls -la "${HOME}/.local/bin/" 2>/dev/null | grep -E "langstar|total" || echo "      (not accessible or does not exist)"
    exit 1
fi
echo ""

# Test 2: Verify --version command works
echo "Test 2: Verify langstar --version works..."
if "${BINARY_PATH}" --version &> /dev/null; then
    echo "✓ langstar --version succeeded"
    version_output=$("${BINARY_PATH}" --version 2>&1)
    echo "  Output: ${version_output}"
else
    echo "✗ FAILED: langstar --version command failed"
    echo "  Expected: langstar --version should display version information"
    echo "  Actual: Command exited with non-zero status"
    echo "  Binary path: ${BINARY_PATH}"
    exit 1
fi
echo ""

# Test 3: Verify --help command works
echo "Test 3: Verify langstar --help works..."
if "${BINARY_PATH}" --help &> /dev/null; then
    echo "✓ langstar --help succeeded"
    # Show first few lines of help output for verification
    help_preview=$("${BINARY_PATH}" --help 2>&1 | head -n 3)
    echo "  Preview:"
    echo "${help_preview}" | sed 's/^/    /'
else
    echo "✗ FAILED: langstar --help command failed"
    echo "  Expected: langstar --help should display help information"
    echo "  Actual: Command exited with non-zero status"
    echo "  Binary path: ${BINARY_PATH}"
    exit 1
fi
echo ""

# Test 4: Verify version matches requested version (if VERSION env var set)
if [ -n "${VERSION}" ] && [ "${VERSION}" != "latest" ]; then
    echo "Test 4: Verify installed version matches requested version..."
    version_output=$("${BINARY_PATH}" --version 2>&1)
    # Extract version number from output (assuming format like "langstar v0.4.1" or "langstar 0.4.1")
    # Supports pre-release versions (e.g., 1.2.3-beta.1) and build metadata (e.g., 1.2.3+20130313144700)
    installed_version=$(echo "${version_output}" | grep -oE 'v?[0-9]+\.[0-9]+\.[0-9]+[^[:space:]]*' | head -n1)
    # Normalize VERSION (remove 'v' prefix if present)
    requested_version="${VERSION#v}"
    installed_version="${installed_version#v}"

    if [ -z "${installed_version}" ]; then
        echo "✗ FAILED: Could not parse version from output: ${version_output}"
        exit 1
    fi
    if [ "${installed_version}" = "${requested_version}" ]; then
        echo "✓ Installed version matches requested version"
        echo "  Requested: ${requested_version}"
        echo "  Installed: ${installed_version}"
    else
        echo "✗ FAILED: Installed version does not match requested version"
        echo "  Requested: ${requested_version}"
        echo "  Installed: ${installed_version}"
        exit 1
    fi
else
    echo "Test 4: Version check skipped (VERSION not set or is 'latest')"
fi
echo ""

# All tests passed
echo "======================================"
echo "✓ All smoke tests passed!"
echo "======================================"
