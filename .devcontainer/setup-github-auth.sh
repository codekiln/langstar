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
if [[ -n "${GITHUB_PAT:-}" ]]; then
  TOKEN_SOURCE="containerEnv"
  TOKEN_VALUE="$GITHUB_PAT"
elif [[ -n "${localEnv_GITHUB_PAT:-}" ]]; then
  # Some devcontainer runtimes expand ${localEnv:...} into localEnv_VAR names
  TOKEN_SOURCE="localEnv"
  TOKEN_VALUE="$localEnv_GITHUB_PAT"
else
  echo "[setup-github-auth] No GITHUB_PAT found. Skipping gh auth."
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

# Configure git credentials to use same token (optional but convenient)
if command -v git >/dev/null 2>&1; then
  USERNAME="${GITHUB_USER:-}"
  if [[ -z "$USERNAME" ]]; then
    # Use gh user if available
    USERNAME="$(gh api user --jq .login 2>/dev/null || echo 'github-user')"
  fi
  printf "protocol=https\nhost=github.com\nusername=%s\npassword=%s\n\n" \
    "$USERNAME" "$TOKEN_VALUE" | git credential approve
  echo "[setup-github-auth] git credential stored for $USERNAME."
fi

echo "[setup-github-auth] Done."