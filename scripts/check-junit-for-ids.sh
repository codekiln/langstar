#!/usr/bin/env bash
# Fail when a JUnit file contains the value of LANGSMITH_ORGANIZATION_ID or
# LANGSMITH_WORKSPACE_ID.
#
# CI uploads JUnit files as artifacts, and anyone signed in to GitHub can
# download them from this public repository. GitHub masks secrets in job logs
# but not in uploaded files, so a test that prints either ID publishes it.
#
# The script names the file and the variable, and never prints the value.
#
# Usage: scripts/check-junit-for-ids.sh <junit-file>...

set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "Usage: $0 <junit-file>..." >&2
  exit 2
fi

leaks=0
for var in LANGSMITH_ORGANIZATION_ID LANGSMITH_WORKSPACE_ID; do
  value="${!var:-}"
  if [ -z "$value" ]; then
    echo "::error::$var is not set, so the JUnit files cannot be checked for it"
    leaks=1
    continue
  fi
  for file in "$@"; do
    if [ ! -f "$file" ]; then
      echo "No JUnit file at $file; nothing to check"
      continue
    fi
    if grep -qF -- "$value" "$file"; then
      echo "::error file=$file::$file contains the value of $var. Find the test that prints it and print what it resolved instead, such as '✓ Workspace ID set'."
      leaks=1
    else
      echo "✓ $file does not contain the value of $var"
    fi
  done
done

exit "$leaks"
