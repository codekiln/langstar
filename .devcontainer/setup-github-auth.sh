#!/usr/bin/env bash
# ---------------------------------------------------------------------
# setup-github-auth.sh
# Non-interactively authenticate GitHub CLI ("gh") inside a devcontainer
# using a Personal Access Token (PAT) or automatic Codespaces token.
#
# Token precedence (first non-empty wins):
#   1. GITHUB_TOKEN - Codespaces sets this automatically with repo scope
#   2. GH_TOKEN     - Alternative gh CLI environment variable
#   3. GITHUB_PAT   - Local Docker Compose .env file (fine-grained PAT)
#   4. GH_PAT       - Alternative Codespaces secret name
#
# Username precedence:
#   GITHUB_USER -> GH_USER -> gh api user lookup -> 'github-user'
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
# Check in order of precedence: Codespaces automatic token first, then PATs
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
  # Codespaces automatically sets GITHUB_TOKEN with repo scope
  TOKEN_SOURCE="Codespaces automatic token (GITHUB_TOKEN)"
  TOKEN_VALUE="$GITHUB_TOKEN"
elif [[ -n "${GH_TOKEN:-}" ]]; then
  # gh CLI also respects GH_TOKEN
  TOKEN_SOURCE="GH_TOKEN environment variable"
  TOKEN_VALUE="$GH_TOKEN"
elif [[ -n "${GITHUB_PAT:-}" ]]; then
  # Local Docker Compose loads from .env file
  TOKEN_SOURCE="Docker Compose environment (GITHUB_PAT)"
  TOKEN_VALUE="$GITHUB_PAT"
elif [[ -n "${GH_PAT:-}" ]]; then
  # Alternative Codespaces secret name
  TOKEN_SOURCE="Codespaces secrets (GH_PAT)"
  TOKEN_VALUE="$GH_PAT"
else
  echo "[setup-github-auth] No token found. Checked: GITHUB_TOKEN, GH_TOKEN, GITHUB_PAT, GH_PAT"
  echo "[setup-github-auth] Skipping gh auth setup."
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