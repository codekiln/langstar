#!/usr/bin/env bash
# Replace the values of LANGSMITH_ORGANIZATION_ID and LANGSMITH_WORKSPACE_ID
# in JUnit files with [redacted VAR], and fail when either value was there.
#
# CI uploads JUnit files as artifacts, and anyone signed in to GitHub can
# download them from this public repository. GitHub masks secrets in job logs
# but not in uploaded files, so a test that prints either ID publishes it.
#
# When a file contains a value, the script replaces it with [redacted VAR] in
# place and exits 1. Square brackets keep the XML valid: nextest writes test
# output as XML text, where a bare < or > would break the file. The job fails, and the steps after it can still publish
# and upload the results for debugging without publishing the ID. The script
# names the file and the variable, and never prints the value.
#
# An empty variable is skipped: pull requests from forks get empty secrets,
# and an empty value cannot leak.
#
# Usage: scripts/redact-ids-in-junit.sh <junit-file>...

set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "Usage: $0 <junit-file>..." >&2
  exit 2
fi

leaks=0
for var in LANGSMITH_ORGANIZATION_ID LANGSMITH_WORKSPACE_ID; do
  value="${!var:-}"
  if [ -z "$value" ]; then
    echo "$var is empty, so no file can contain it; skipping"
    continue
  fi
  for file in "$@"; do
    if [ ! -f "$file" ]; then
      echo "No JUnit file at $file; nothing to check"
      continue
    fi
    if grep -qF -- "$value" "$file"; then
      VALUE="$value" VAR="$var" perl -pi -e 's/\Q$ENV{VALUE}\E/[redacted $ENV{VAR}]/g' "$file"
      echo "::error file=$file::$file contained the value of $var, now redacted. Find the test that prints it and print what it resolved instead, such as '✓ Workspace ID set'."
      leaks=1
    else
      echo "✓ $file does not contain the value of $var"
    fi
  done
done

exit "$leaks"
