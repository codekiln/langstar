#!/usr/bin/env bash
# ---------------------------------------------------------------------
# post-create.sh
# Post-create setup for langstar devcontainer
#
# Installs development tools and dependencies:
# - mise: version manager (installs Rust, Python, etc.)
# - cargo-release: Rust release workflow tool
# - git-cliff: Changelog generator
# - gh-sub-issue: GitHub CLI extension for issue hierarchy
# ---------------------------------------------------------------------

set -euo pipefail

echo "[post-create] Starting post-create setup..."

# Step 1: Clear git credential helpers before mise install
# mise may need to download packages from git repositories, and if a
# credential helper is configured but the binary doesn't exist, it fails.
# This typically happens when host machine's gitconfig is copied into the
# container with references to credential helpers that don't exist in the
# container environment (e.g., docker-credential-desktop).
echo "[post-create] Clearing git credential helpers..."
git config --global --unset-all credential.helper 2>/dev/null || true
git config --unset-all credential.helper 2>/dev/null || true
git config --global --unset-all credential.https://github.com.helper 2>/dev/null || true
git config --unset-all credential.https://github.com.helper 2>/dev/null || true
git config --global --unset-all credential.https://gist.github.com.helper 2>/dev/null || true
git config --unset-all credential.https://gist.github.com.helper 2>/dev/null || true

# Reset credential helper chain by setting empty string
# The empty string resets the helper list (overriding system config)
# This ensures no credential helpers are active during mise install
git config --global --replace-all credential.helper "" ".*" 2>/dev/null || true

# Remove SSH URL rewrite rules that would force SSH protocol instead of HTTPS
# VS Code Dev Containers may copy host machine's ~/.gitconfig which can contain:
#   [url "git@github.com:"]
#       insteadof = https://github.com/
# This would cause mise to try SSH authentication (requiring SSH keys) instead of
# HTTPS authentication (which works without credentials for public repos).
# We remove these rules here to ensure mise install uses HTTPS protocol.
git config --global --unset url.git@github.com:.insteadof 2>/dev/null || true
git config --unset url.git@github.com:.insteadof 2>/dev/null || true

echo "[post-create] Git credential helpers and SSH URL rewrites cleared."

# Step 2: Configure mise activation in zshrc
echo "[post-create] Configuring mise activation for zsh..."
if ! grep -q 'mise activate zsh' ~/.zshrc 2>/dev/null; then
  # Wrap mise activation in interactive check to prevent zle errors in non-interactive mode
  cat >> ~/.zshrc << 'EOF'
# Only activate mise in interactive shells to avoid zle errors
if [[ -o interactive ]]; then
  eval "$(mise activate zsh)"
fi
EOF
  echo "[post-create] Added mise activation to ~/.zshrc (with interactive guard)"
else
  echo "[post-create] mise already configured in ~/.zshrc"
fi

# Step 3: Trust and install mise tools
echo "[post-create] Trusting mise configuration..."
mise trust

echo "[post-create] Installing mise tools (Rust, Python, etc.)..."
mise install

# Step 4: Install cargo tools
# After mise install, cargo should be available via ~/.cargo/env
echo "[post-create] Installing cargo tools (cargo-release, git-cliff)..."

# Check if cargo env exists and source it
if [[ -f ~/.cargo/env ]]; then
  # shellcheck source=/dev/null
  . ~/.cargo/env
  echo "[post-create] Sourced ~/.cargo/env"
else
  echo "[post-create] WARNING: ~/.cargo/env not found. Checking if cargo is already in PATH..."
fi

# Verify cargo is available
if ! command -v cargo >/dev/null 2>&1; then
  echo "[post-create] ERROR: cargo not found in PATH after mise install."
  echo "[post-create] This likely means Rust was not installed by mise."
  exit 1
fi

echo "[post-create] cargo found at: $(command -v cargo)"
echo "[post-create] cargo version: $(cargo --version)"

# Install cargo tools
cargo install cargo-release git-cliff

# Step 6: Install gh CLI extensions
echo "[post-create] Installing gh CLI extensions..."

# Verify gh is available
if ! command -v gh >/dev/null 2>&1; then
  echo "[post-create] WARNING: gh CLI not found. Skipping extension installation."
else
  # Install gh-sub-issue for issue hierarchy management
  echo "[post-create] Installing gh-sub-issue extension..."
  if gh extension install yahsan2/gh-sub-issue; then
    echo "[post-create] gh-sub-issue installed successfully"
  else
    echo "[post-create] WARNING: Failed to install gh-sub-issue extension"
  fi
fi

echo "[post-create] Post-create setup completed successfully!"
