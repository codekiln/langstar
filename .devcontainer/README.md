# Devcontainer Setup

This directory contains the devcontainer configuration for the Langstar project. The devcontainer provides a consistent development environment across local machines and GitHub Codespaces.

## Overview

The devcontainer configuration supports two environments:

1. **Local Development** - Uses local `.env` file for environment variables
2. **GitHub Codespaces** - Uses Codespaces secrets for environment variables

## Local Development Setup

### Prerequisites

- Docker Desktop installed and running
- VS Code with the Dev Containers extension installed

### Initial Setup

1. **Copy environment template:**
   ```bash
   cp .devcontainer/.env.default .devcontainer/.env
   ```

2. **Copy devcontainer local configuration:**
   ```bash
   cp .devcontainer/devcontainer.local.json.template .devcontainer/devcontainer.local.json
   ```

3. **Edit `.devcontainer/.env`** with your actual credentials:
   ```bash
   # GitHub Configuration
   GITHUB_PAT=ghp_YourActualTokenHere
   GITHUB_USER=your_github_username
   GITHUB_PROJECT_PAT=ghp_YourProjectTokenHere

   # Anthropic Configuration
   ANTHROPIC_API_KEY=sk-ant-YourActualKeyHere

   # LangSmith Configuration
   LANGSMITH_API_KEY=lsv2_YourActualKeyHere
   ```

4. **Open in devcontainer:**
   - Open the project in VS Code
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Select "Dev Containers: Reopen in Container"
   - Wait for the container to build (first time takes a few minutes)

### Files Created (Gitignored)

These files are created locally and **will not be committed** to git:

- `.devcontainer/devcontainer.local.json` - Local devcontainer overrides
- `.devcontainer/.env` - Local environment variables with your secrets

## GitHub Codespaces Setup

### Configure Codespaces Secrets

Codespaces uses repository or organization secrets instead of local `.env` files.

1. **Go to your repository settings:**
   - Navigate to `Settings` → `Secrets and variables` → `Codespaces`

2. **Add the following secrets:**

   | Secret Name | Description | Example Value |
   |-------------|-------------|---------------|
   | `GH_PAT` | GitHub Personal Access Token | `ghp_xxxxx` |
   | `GH_USER` | Your GitHub username | `your_username` |
   | `GH_PROJECT_PAT` | GitHub PAT with project permissions | `ghp_xxxxx` |
   | `ANTHROPIC_API_KEY` | Anthropic API key for Claude | `sk-ant-xxxxx` |
   | `LANGSMITH_API_KEY` | LangSmith API key | `lsv2_xxxxx` |

3. **Create a Codespace:**
   - Go to the repository on GitHub
   - Click the green "Code" button
   - Select "Codespaces" tab
   - Click "Create codespace on main" (or your branch)

### How Codespaces Works

In Codespaces:
- The main `devcontainer.json` is used
- `devcontainer.local.json` doesn't exist (and isn't needed)
- Environment variables come from Codespaces secrets (`GH_PAT`, `GH_USER`, etc.)
- `setup-github-auth.sh` configures git authentication using `$GH_PAT`
- No `.env` file is needed or used

## Architecture

### Configuration Files

| File | Purpose | Committed to Git | Environment |
|------|---------|------------------|-------------|
| `devcontainer.json` | Base devcontainer config | ✅ Yes | Both |
| `devcontainer.local.json.template` | Template for local overrides | ✅ Yes | Both |
| `devcontainer.local.json` | Local devcontainer overrides | ❌ No (gitignored) | Local only |
| `.env.default` | Environment variables template | ✅ Yes | Both |
| `.env` | Actual environment variables | ❌ No (gitignored) | Local only |
| `Dockerfile` | Container image definition | ✅ Yes | Both |
| `setup-github-auth.sh` | Git authentication setup | ✅ Yes | Both |

### How Local Configuration Works

The devcontainer uses a **merge strategy** for configuration:

1. **Base configuration** (`devcontainer.json`):
   - Used by both local and Codespaces
   - Contains essential `runArgs` (network capabilities)
   - Contains `remoteEnv` with Codespaces-compatible variables
   - Does **not** reference `.env` file directly

2. **Local overrides** (`devcontainer.local.json`):
   - Only exists locally (gitignored)
   - Sets `DEVCONTAINER_LOCAL_ENV_FILE` in `remoteEnv`
   - VS Code automatically merges this with the base config
   - Tells `setup-github-auth.sh` to source the `.env` file

3. **Environment variables** (`.env`):
   - Sourced by `setup-github-auth.sh` during `postStartCommand`
   - Contains actual credentials and API keys
   - Never committed to git
   - Variables are exported to the shell environment using `set -a`

## Troubleshooting

### Container Build Fails

**Problem:** Container fails to build in Codespaces

**Solution:**
- Ensure Codespaces secrets are configured correctly
- Check that secret names match exactly: `GH_PAT`, `GH_USER`, `GH_PROJECT_PAT`, etc.
- Verify secrets have proper permissions

### Environment Variables Not Available

**Problem:** Environment variables are undefined in the container

**Local Development:**
1. Verify `.devcontainer/.env` exists and has actual values (not placeholders)
2. Verify `.devcontainer/devcontainer.local.json` exists
3. Rebuild the devcontainer: `Dev Containers: Rebuild Container`

**Codespaces:**
1. Check Codespaces secrets in repository settings
2. Restart the Codespace
3. Check environment variables with: `printenv | grep -E 'GH_|ANTHROPIC|LANGSMITH'`

### Git Authentication Fails

**Problem:** Git operations fail with authentication errors

**Solution:**
1. Check that `GH_PAT` (Codespaces) or `GITHUB_PAT` (local) is set correctly
2. Verify the token has `repo` scope
3. Run the setup script manually: `bash .devcontainer/setup-github-auth.sh`

### Changes to devcontainer.json Not Applied

**Problem:** Changes to configuration aren't taking effect

**Solution:**
1. Rebuild the container: `Dev Containers: Rebuild Container`
2. Or rebuild without cache: `Dev Containers: Rebuild Container Without Cache`

## Best Practices

1. **Never commit secrets:**
   - Always use `.env` or Codespaces secrets
   - Never hardcode credentials in configuration files
   - Double-check `.gitignore` includes `.devcontainer/.env` and `devcontainer.local.json`

2. **Keep templates updated:**
   - Update `.env.default` when adding new environment variables
   - Update `devcontainer.local.json.template` when changing local configuration
   - Document any new required secrets

3. **Test both environments:**
   - Test configuration changes locally before committing
   - Verify changes work in Codespaces (create a test Codespace)
   - Ensure new environment variables are documented

4. **Document environment variables:**
   - Add new variables to both `.env.default` and this README
   - Document what each variable is used for
   - Provide example values (but not real credentials)

## Environment Variables Reference

### GitHub Authentication

| Variable | Local Name | Codespaces Name | Required | Description |
|----------|-----------|-----------------|----------|-------------|
| GitHub PAT | `GITHUB_PAT` | `GH_PAT` | Yes | Personal access token for git operations |
| GitHub User | `GITHUB_USER` | `GH_USER` | Yes | Your GitHub username |
| GitHub Project PAT | `GITHUB_PROJECT_PAT` | `GH_PROJECT_PAT` | Optional | PAT with project permissions for API operations |

### API Keys

| Variable | Required | Description |
|----------|----------|-------------|
| `ANTHROPIC_API_KEY` | Yes | Anthropic API key for Claude Code |
| `LANGSMITH_API_KEY` | Optional | LangSmith API key for testing |

### Anthropic Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ANTHROPIC_BEDROCK_BASE_URL` | No | - | AWS Bedrock endpoint URL |
| `ANTHROPIC_MODEL` | No | `us.anthropic.claude-sonnet-4-5-20250929-v1:0` | Primary Claude model |
| `ANTHROPIC_SMALL_FAST_MODEL` | No | `us.anthropic.claude-haiku-4-5-20251001-v1:0` | Fast model for simple tasks |
| `AWS_REGION` | No | `us-east-1` | AWS region for Bedrock |
| `CLAUDE_CODE_SKIP_BEDROCK_AUTH` | No | `1` | Skip Bedrock auth |
| `CLAUDE_CODE_USE_BEDROCK` | No | `1` | Use Bedrock for Claude |

## Related Issues

- [#33](https://github.com/codekiln/langstar/issues/33) - Fix devcontainer .env file handling for Codespaces compatibility
- [#23](https://github.com/codekiln/langstar/issues/23) - Refactor to use `GH_*` variables for Codespaces compatibility
- [#26](https://github.com/codekiln/langstar/issues/26) - Removed problematic `env` section from `.claude/settings.json`

## Resources

- [VS Code Dev Containers Documentation](https://code.visualstudio.com/docs/devcontainers/containers)
- [GitHub Codespaces Documentation](https://docs.github.com/en/codespaces)
- [Dev Container Specification](https://containers.dev/)
