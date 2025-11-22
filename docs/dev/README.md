# Developer Documentation

This directory contains instructions and conventions for developing the Langstar project.

## Contents

- [GitHub Workflow](./github-workflow.md) - Complete guide to the GitHub issue-driven development workflow
- [GitHub Projects](./github-projects.md) - GitHub Projects V2 configuration, fields, and API usage
- [Git SCM Conventions](./git-scm-conventions.md) - Guidelines for commit messages and version control practices
- [Spec-Kit Integration](./spec-kit.md) - Spec-driven development with GitHub Spec-Kit
- [Procedures & Troubleshooting](./procedures.md) - Detailed step-by-step procedures and operational guides

## Purpose

These documents outline the coding conventions, best practices, and standards that all contributors should follow when working on this project. Please review these guidelines before making commits or submitting pull requests.

## Tips from Development - Memories

### Devcontainer-centricity
- This project uses a devcontainer to standardize the development environment. Never configure the environment in a 1-off way, unless running a 1-off test. Always prefer modifications to .devcontainer folder and related assets.

### Looking up docs to LangChain
- <rules>
for ANY question about the langchain ecosystem (langsmith, LangGraph, langchain) use the langgraph-docs-mcp server to help answer -- 
+ call list_doc_sources tool to get the available llms.txt file
+ call fetch_docs tool to read it
+ reflect on the urls in llms.txt 
+ reflect on the input question 
+ call fetch_docs on any urls relevant to the question
</rules>

### CRITICAL: Privacy and Security - Never Expose Secrets

**Under no circumstances should any of the following be written to committed files, git issues, pull requests, or documentation:**

#### What Counts as Sensitive Information

- **API Keys**: `LANGSMITH_API_KEY`, `ANTHROPIC_API_KEY`, `GITHUB_PAT`, etc.
- **Organization IDs**: LangSmith organization IDs, workspace IDs, tenant IDs
- **Organization Names**: Actual organization or workspace names from services
- **Access Tokens**: GitHub tokens, OAuth tokens, JWT tokens
- **Credentials**: Passwords, secrets, private keys
- **Personal Identifiers**: Email addresses, user IDs (unless explicitly public)
- **Any environment variable values**: Even seemingly innocuous ones may be sensitive

#### Always Use Placeholders

When writing documentation or examples:

```bash
# ❌ WRONG - actual values
export LANGSMITH_API_KEY=lsv2_sk_abc123...
export LANGSMITH_ORGANIZATION_ID=d1e1dfff-39bf-4cea-9a2e-85e970ce40ef

# ✅ CORRECT - placeholders
export LANGSMITH_API_KEY=<your-api-key>
export LANGSMITH_ORGANIZATION_ID=<your-org-id>
```

#### If You Accidentally Expose a Secret

If you accidentally post sensitive information to a GitHub issue, PR, or comment:

1. **DELETE immediately** - Do NOT just edit the comment
   - Editing leaves the secret in the edit history (visible via "Edited" → "View history")
   - Deletion is the only way to remove it from public view
   - Use: `gh issue comment <issue> --delete-last --yes`

2. **Rotate/revoke the secret** as soon as possible
   - Change API keys through the service provider
   - Regenerate tokens and credentials
   - Update environment variables in secure locations

3. **Contact GitHub Support** if needed
   - For private repos or particularly sensitive data
   - To request purging from caches and forks

4. **Document the incident** internally
   - Note what was exposed and when
   - Track what credentials were rotated
   - Learn from the mistake

#### Best Practices

- **Review before posting**: Always check issue comments, PR descriptions, and code for sensitive data
- **Use `.gitignore`**: Ensure `.env`, `.env.local`, and credential files are never committed
- **Test with dummy data**: Use fake IDs and placeholder values when writing tests and examples
- **Redact in logs**: When sharing debug output, redact any sensitive values first
- **Think twice about IDs**: Organization/workspace IDs may seem harmless but can be used for reconnaissance

#### Example: Safe Testing Output

```bash
# ❌ BAD - Shows actual org details
Organization ID: d1e1dfff-39bf-4cea-9a2e-85e970ce40ef
Organization Name: ACME Corporation

# ✅ GOOD - Redacted
Organization ID: <redacted>
Organization Name: <redacted>

# ✅ BETTER - Descriptive without exposing
Successfully retrieved organization details
Organization type: standard (non-personal)
```

#### Why This Matters

- **Security**: Exposed credentials can lead to unauthorized access
- **Privacy**: Organization names and IDs can reveal client relationships
- **Compliance**: May violate data protection policies or contracts
- **Reputation**: Security incidents damage trust and credibility

**Remember**: If you're unsure whether something is sensitive, err on the side of caution and redact it.

### Working with Phased/Sub-Task Issues

When working on multi-phase features, **always review prerequisite phases before starting**. Taking 10 minutes to review prior work can save hours of implementing the wrong approach.

**See [procedures.md](./procedures.md#working-with-phasedsub-task-issues) for detailed step-by-step procedures.**

### Pre-Commit Checklist

**Always run these checks locally before committing** to catch issues before CI fails:

```bash
cargo fmt && \
cargo check --workspace --all-features && \
cargo clippy --workspace --all-features -- -D warnings && \
cargo test --workspace --all-features && \
cargo fmt --check
```

**Critical points:**
- Test at **workspace level** (not just individual crates)
- When making breaking changes, search for all usages first
- Running checks locally (1 minute) saves 10-20 minutes of CI roundtrips

**See [procedures.md](./procedures.md#pre-commit-checklist) for detailed explanations and troubleshooting.**

### Handling Emergent Tasks

We follow a principled GitHub issue → PR workflow. If, in the course of working on an issue/PR another task comes up:

1. **File a new issue** for the emergent task
2. **Create a new feature branch** off the current feature branch (format: `<username>/<new-issue-num>-<new-issue-slug>`)
3. **PR from new branch into current branch** with a description that will close the newly opened issue

### Manual Version Bumps Require Proper PR Title and Changelog

**Lesson from #243**: When manually bumping versions (bypassing `prepare-release` workflow):

1. **PR title MUST start with `🔖 release:`**
   - ✅ Correct: `🔖 release: bump version to v0.4.3`
   - ❌ Wrong: `🔧 chore: bump version to v0.4.3`
   - The `auto-tag-release` workflow checks: `startsWith(github.event.pull_request.title, '🔖 release:')`
   - If title is wrong, workflow won't trigger and you must manually create tag

2. **MUST update CHANGELOG.md with git-cliff**
   - Version bumps without changelog entries break release workflow invariant
   - Run: `git-cliff --tag "vX.Y.Z" --unreleased --prepend CHANGELOG.md`
   - For version skips, write minimal entry explaining the skip (see v0.4.3 example)
   - Never bump version without updating changelog

3. **If auto-tag-release doesn't trigger after merge:**
   ```bash
   git checkout main && git pull
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
   This manually creates the tag and triggers the release workflow.

**Why this matters:**
- The release workflow requires both: (1) changelog entry for every version, (2) specific PR title format
- If either is missing, releases won't publish automatically
- Manual tag creation is recovery mechanism, not preferred workflow

**References:** Issue #243 (v0.4.3 version skip), auto-tag-release.yml:12, prepare-release.yml:159-167