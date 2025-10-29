# Devcontainer Setup

This directory contains the devcontainer configuration for the Langstar project. The devcontainer provides a consistent development environment across local machines and GitHub Codespaces using **Docker Compose**.

## Overview

The devcontainer uses Docker Compose which provides **native `.env` file support**, solving environment variable management elegantly for both local and Codespaces environments.

### Architecture

- **Local Development**: Docker Compose automatically loads `.env` file
- **GitHub Codespaces**: Environment variables come from Codespaces secrets
- **Single Configuration**: No duplication, standard Docker Compose patterns

## Local Development Setup

### Prerequisites

- Docker Desktop installed and running
- VS Code with the Dev Containers extension installed

### Initial Setup

1. **Copy environment template:**
   ```bash
   cd .devcontainer
   cp .env.default .env
   ```

2. **Edit `.env`** with your actual credentials:
   ```bash
   # Replace placeholder values with real credentials
   GITHUB_PAT=ghp_YourActualTokenHere
   GITHUB_USER=your_github_username
   GITHUB_PROJECT_PAT=ghp_YourProjectTokenHere
   AWS_ACCESS_KEY_ID=your_aws_access_key_here
   AWS_SECRET_ACCESS_KEY=your_aws_secret_key_here
   LANGSMITH_API_KEY=lsv2_YourActualKeyHere
   ```

3. **(Optional) Create local overrides:**
   ```bash
   cp docker-compose.override.yml.template docker-compose.override.yml
   # Edit docker-compose.override.yml for local customizations
   ```

4. **Open in devcontainer:**
   - Open the project in VS Code
   - Press `Cmd+Shift+P` (Mac) or `Ctrl+Shift+P` (Windows/Linux)
   - Select "Dev Containers: Reopen in Container"
   - Wait for the container to build (first time takes a few minutes)

### Files Created (Gitignored)

These files are created locally and **will not be committed** to git:

- `.devcontainer/.env` - Your local environment variables with secrets
- `.devcontainer/docker-compose.override.yml` - Optional local Docker Compose overrides

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
   | `AWS_ACCESS_KEY_ID` | AWS access key for Bedrock | `AKIAXXXXXXX` |
   | `AWS_SECRET_ACCESS_KEY` | AWS secret access key | `xxxxx` |
   | `LANGSMITH_API_KEY` | LangSmith API key | `lsv2_xxxxx` |

3. **Create a Codespace:**
   - Go to the repository on GitHub
   - Click the green "Code" button
   - Select "Codespaces" tab
   - Click "Create codespace on main" (or your branch)

### How Codespaces Works

In Codespaces:
- The `devcontainer.json` uses Docker Compose configuration
- `docker-compose.yml` uses fallback syntax: `${GITHUB_PAT:-${GH_PAT}}`
- Environment variables come from Codespaces secrets (`GH_PAT`, `GH_USER`, etc.)
- No `.env` file is needed or used
- `setup-github-auth.sh` configures git authentication using the provided variables

## Architecture

### Configuration Files

| File | Purpose | Committed to Git | Environment |
|------|---------|------------------|-------------|
| `devcontainer.json` | Dev Container config (points to Docker Compose) | ✅ Yes | Both |
| `docker-compose.yml` | Docker Compose service definition | ✅ Yes | Both |
| `docker-compose.override.yml.template` | Template for local Docker overrides | ✅ Yes | Both |
| `docker-compose.override.yml` | Local Docker Compose overrides | ❌ No (gitignored) | Local only |
| `.env.default` | Environment variables template | ✅ Yes | Both |
| `.env` | Actual environment variables | ❌ No (gitignored) | Local only |
| `Dockerfile` | Container image definition | ✅ Yes | Both |
| `setup-github-auth.sh` | Git authentication setup | ✅ Yes | Both |

### How Docker Compose Environment Variables Work

Docker Compose has **native `.env` file support**:

1. **Local Development:**
   - Docker Compose automatically loads `.env` from the same directory as `docker-compose.yml`
   - Variables are substituted in `docker-compose.yml` using `${VARIABLE_NAME}` syntax
   - Variables become available in the container environment
   - No custom scripts or workarounds needed!

2. **GitHub Codespaces:**
   - Codespaces secrets are available as environment variables to Docker Compose
   - `docker-compose.yml` uses fallback syntax: `${GITHUB_PAT:-${GH_PAT}}`
   - This means: use `GITHUB_PAT` if available, otherwise use `GH_PAT`
   - Works seamlessly without any `.env` file

3. **Variable Precedence:**
   - Local: `.env` file variables → Docker Compose → Container environment
   - Codespaces: Secrets → Docker Compose → Container environment

### Docker Compose Structure

**`docker-compose.yml`** (base configuration, committed):
```yaml
services:
  langstar-dev:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      # Supports both local (.env) and Codespaces (secrets)
      GITHUB_PAT: ${GITHUB_PAT:-${GH_PAT}}
      GITHUB_USER: ${GITHUB_USER:-${GH_USER}}
      # ... other variables
    volumes:
      - ..:/workspace:cached
      - claude-code-bashhistory:/commandhistory
```

**`docker-compose.override.yml`** (local only, gitignored):
```yaml
services:
  langstar-dev:
    # Add local-specific customizations
    ports:
      - "8080:8080"  # Example: expose ports
    volumes:
      - ~/my-data:/data  # Example: mount local directories
```

## Troubleshooting

### Container Build Fails

**Problem:** Container fails to build

**Local Development:**
1. Verify Docker Desktop is running
2. Check `.env` file exists and has actual values (not placeholders)
3. Try: `docker-compose -f .devcontainer/docker-compose.yml build --no-cache`

**Codespaces:**
1. Ensure Codespaces secrets are configured correctly
2. Verify secret names match exactly: `GH_PAT`, `GH_USER`, etc.
3. Check secrets have proper permissions

### Environment Variables Not Available

**Problem:** Environment variables are undefined in the container

**Local Development:**
1. Verify `.devcontainer/.env` exists and has actual values
2. Check you're in `.devcontainer` directory when running Docker Compose
3. Rebuild: `Dev Containers: Rebuild Container`
4. Test manually:
   ```bash
   cd .devcontainer
   docker-compose config  # Shows merged configuration
   ```

**Codespaces:**
1. Check Codespaces secrets in repository settings
2. Restart the Codespace
3. Verify: `printenv | grep -E 'GH_|ANTHROPIC|LANGSMITH'`

### Git Authentication Fails

**Problem:** Git operations fail with authentication errors

**Solution:**
1. Check that `GITHUB_PAT` (local) or `GH_PAT` (Codespaces) is set correctly
2. Verify the token has `repo` scope
3. Run setup script manually: `bash .devcontainer/setup-github-auth.sh`
4. Check token in container:
   ```bash
   echo ${GITHUB_PAT:-${GH_PAT}} | cut -c1-10  # Show first 10 chars
   ```

### Docker Compose Override Not Working

**Problem:** Local overrides in `docker-compose.override.yml` aren't applied

**Solution:**
1. Ensure file is named exactly `docker-compose.override.yml` (not `.template`)
2. Verify it's in `.devcontainer/` directory
3. Check YAML syntax is valid: `docker-compose config`
4. Rebuild container completely

## Best Practices

1. **Never commit secrets:**
   - Always use `.env` or Codespaces secrets
   - Never hardcode credentials in configuration files
   - Double-check `.gitignore` includes `.env` and `docker-compose.override.yml`

2. **Keep templates updated:**
   - Update `.env.default` when adding new environment variables
   - Update `docker-compose.override.yml.template` when changing Docker config
   - Document any new required secrets

3. **Test both environments:**
   - Test configuration changes locally before committing
   - Verify changes work in Codespaces (create a test Codespace)
   - Ensure new environment variables are documented

4. **Use Docker Compose features:**
   - Use `docker-compose.override.yml` for local customizations
   - Leverage Docker Compose's native `.env` file support
   - Follow Docker Compose best practices

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
| `AWS_ACCESS_KEY_ID` | Yes | AWS access key for Bedrock authentication |
| `AWS_SECRET_ACCESS_KEY` | Yes | AWS secret access key for Bedrock authentication |
| `LANGSMITH_API_KEY` | Optional | LangSmith API key for testing |

### Anthropic Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ANTHROPIC_MODEL` | No | `us.anthropic.claude-sonnet-4-5-20250929-v1:0` | Primary Claude model |
| `ANTHROPIC_SMALL_FAST_MODEL` | No | `us.anthropic.claude-haiku-4-5-20251001-v1:0` | Fast model for simple tasks |
| `AWS_REGION` | No | `us-east-1` | AWS region for Bedrock |
| `CLAUDE_CODE_USE_BEDROCK` | No | `1` | Use Bedrock for Claude |

## Advanced Usage

### Custom Docker Compose Commands

```bash
# View merged Docker Compose configuration
cd .devcontainer
docker-compose config

# Build without cache
docker-compose build --no-cache

# View container logs
docker-compose logs langstar-dev

# Execute command in running container
docker-compose exec langstar-dev bash
```

### Debugging Environment Variables

```bash
# Inside container - check all environment variables
printenv | sort

# Check specific variable
echo $GITHUB_PAT | cut -c1-20  # Show first 20 chars

# Test Docker Compose variable substitution
cd .devcontainer
docker-compose config | grep -A 10 environment:
```

## Related Issues

- [#33](https://github.com/codekiln/langstar/issues/33) - Fix devcontainer .env file handling for Codespaces compatibility
- [#23](https://github.com/codekiln/langstar/issues/23) - Refactor to use `GH_*` variables for Codespaces compatibility
- [#26](https://github.com/codekiln/langstar/issues/26) - Removed problematic `env` section from `.claude/settings.json`

## Resources

- [VS Code Dev Containers Documentation](https://code.visualstudio.com/docs/devcontainers/containers)
- [Docker Compose Environment Variables](https://docs.docker.com/compose/environment-variables/)
- [GitHub Codespaces Documentation](https://docs.github.com/en/codespaces)
- [Dev Container Specification](https://containers.dev/)
