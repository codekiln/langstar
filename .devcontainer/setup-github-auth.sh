#!/usr/bin/env bash
# ---------------------------------------------------------------------
# setup-github-auth.sh
# Non-interactively authenticate GitHub CLI ("gh") inside a devcontainer
# using a fine-grained Personal Access Token (PAT).
#
# Reads the PAT from $GITHUB_PAT (in containerEnv or localEnv expansion)
# and the username from $GITHUB_USER.
#
# After running, `gh auth status` and `git push` should both work.
# ---------------------------------------------------------------------

set -euo pipefail

echo "[setup-github-auth] Starting setup..."

# Ensure gh is installed
if ! command -v gh >/dev/null 2>&1; then
  echo "[setup-github-auth] ERROR: gh CLI not installed in container."
  exit 0  # not fatal, container may not use gh
fi

# Determine which token source is populated
# Docker Compose loads environment variables from .env file (local) or Codespaces secrets
if [[ -n "${GITHUB_PAT:-}" ]]; then
  TOKEN_SOURCE="Docker Compose environment (GITHUB_PAT)"
  TOKEN_VALUE="$GITHUB_PAT"
elif [[ -n "${GH_PAT:-}" ]]; then
  # Codespaces uses GH_PAT
  TOKEN_SOURCE="Codespaces secrets (GH_PAT)"
  TOKEN_VALUE="$GH_PAT"
  # Set GITHUB_PAT for consistency
  export GITHUB_PAT="$GH_PAT"
else
  echo "[setup-github-auth] No GITHUB_PAT or GH_PAT found. Skipping gh auth."
  exit 0
fi

echo "[setup-github-auth] Using token from $TOKEN_SOURCE."

# Optional: show masked token length for debugging
echo "[setup-github-auth] Token length: ${#TOKEN_VALUE}"

# Authenticate gh non-interactively
if printf "%s" "$TOKEN_VALUE" | gh auth login --with-token >/tmp/gh-auth.log 2>&1; then
  echo "[setup-github-auth] gh authenticated successfully."
else
  echo "[setup-github-auth] gh authentication failed; see /tmp/gh-auth.log"
  cat /tmp/gh-auth.log || true
  exit 1
fi

# Remove any SSH URL rewrite rules and credential helpers that would bypass PAT authentication
#
# VS Code Dev Containers automatically copies your host machine's ~/.gitconfig into the container.
# If your host has a git config rule like:
#   [url "git@github.com:"]
#       insteadof = https://github.com/
# Then ALL https:// GitHub URLs get silently rewritten to git@github.com (SSH protocol).
#
# Additionally, if your host has credential helpers configured (like VSCode's credential helper
# or gh auth git-credential), these can override the PAT-based authentication we're setting up.
#
# This breaks PAT authentication because:
# - PATs only work with HTTPS protocol
# - SSH requires SSH keys, not PATs
# - The rewrite happens transparently, so "git remote -v" might show https:// but git actually uses SSH
# - Credential helpers may invoke OAuth flows instead of using the PAT
#
# This causes VS Code to pop up OAuth dialogs asking for broad GitHub access, even though you've
# provided a scoped fine-grained PAT in GITHUB_PAT.
#
# Solution: Remove the SSH rewrite rules and clear credential helpers so git actually uses HTTPS with your PAT as intended.
git config --global --unset url.git@github.com:.insteadof 2>/dev/null || true
git config --unset url.git@github.com:.insteadof 2>/dev/null || true

# Clear all existing credential helpers (both global and local)
# Note: We can't modify /etc/gitconfig (VSCode's system config), but we can override it
git config --global --unset-all credential.helper 2>/dev/null || true
git config --unset-all credential.helper 2>/dev/null || true
git config --global --unset-all credential.https://github.com.helper 2>/dev/null || true
git config --unset-all credential.https://github.com.helper 2>/dev/null || true
git config --global --unset-all credential.https://gist.github.com.helper 2>/dev/null || true
git config --unset-all credential.https://gist.github.com.helper 2>/dev/null || true

# Reset credential helper chain by setting empty string, then add store
# The empty string resets the helper list (overriding system config)
# Then we add 'store' as the only helper
git config --global --replace-all credential.helper "" ".*"
git config --global --add credential.helper store

echo "[setup-github-auth] Cleared conflicting git credential helpers."

# Configure git credentials to use same token (optional but convenient)
if command -v git >/dev/null 2>&1; then

  # Handle both GITHUB_USER (local) and GH_USER (Codespaces)
  USERNAME="${GITHUB_USER:-${GH_USER:-}}"
  if [[ -z "$USERNAME" ]]; then
    # Use gh user if available
    USERNAME="$(gh api user --jq .login 2>/dev/null || echo 'github-user')"
  fi
  printf "protocol=https\nhost=github.com\nusername=%s\npassword=%s\n\n" \
    "$USERNAME" "$TOKEN_VALUE" | git credential approve
  echo "[setup-github-auth] git credential stored for $USERNAME using 'store' helper."
fi

echo "[setup-github-auth] Done."