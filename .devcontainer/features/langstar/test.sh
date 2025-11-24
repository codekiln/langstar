#!/bin/bash
# Smoke tests for langstar devcontainer feature
# Verifies that the langstar CLI is installed correctly and basic commands work
set -e

echo "======================================"
echo "Running langstar feature smoke tests"
echo "======================================"
echo ""

# Test 1: Verify binary is in PATH
echo "Test 1: Verify langstar binary is in PATH..."
if command -v langstar &> /dev/null; then
    echo "✓ langstar binary found in PATH"
    echo "  Location: $(command -v langstar)"
else
    echo "✗ FAILED: langstar binary not found in PATH"
    echo "  Expected: langstar command should be available after feature installation"
    echo "  Actual: command -v langstar returned non-zero exit code"
    exit 1
fi
echo ""

# Test 2: Verify --version command works
echo "Test 2: Verify langstar --version works..."
if langstar --version &> /dev/null; then
    echo "✓ langstar --version succeeded"
    version_output=$(langstar --version 2>&1)
    echo "  Output: ${version_output}"
else
    echo "✗ FAILED: langstar --version command failed"
    echo "  Expected: langstar --version should display version information"
    echo "  Actual: Command exited with non-zero status"
    exit 1
fi
echo ""

# Test 3: Verify --help command works
echo "Test 3: Verify langstar --help works..."
if langstar --help &> /dev/null; then
    echo "✓ langstar --help succeeded"
    # Show first few lines of help output for verification
    help_preview=$(langstar --help 2>&1 | head -n 3)
    echo "  Preview:"
    echo "    ${help_preview//$'\n'/$'\n    '}"
else
    echo "✗ FAILED: langstar --help command failed"
    echo "  Expected: langstar --help should display help information"
    echo "  Actual: Command exited with non-zero status"
    exit 1
fi
echo ""

# Test 4: Verify version matches requested version (if VERSION env var set)
if [ -n "${VERSION}" ] && [ "${VERSION}" != "latest" ]; then
    echo "Test 4: Verify installed version matches requested version..."
    version_output=$(langstar --version 2>&1)
    # Extract version number from output (assuming format like "langstar v0.4.1" or "langstar 0.4.1")
    installed_version=$(echo "${version_output}" | grep -oE 'v?[0-9]+\.[0-9]+\.[0-9]+' | head -n1)
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
