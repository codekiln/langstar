# Devcontainer Setup

This directory contains the devcontainer configuration for the Langstar project. The devcontainer provides a consistent development environment across local machines and GitHub Codespaces using **Docker Compose**.

> **First-time contributor?** See [docs/dev/getting-started.md](../docs/dev/getting-started.md) for step-by-step setup instructions for VS Code, JetBrains, and Codespaces.

## Overview

The devcontainer uses Docker Compose with **three separate configurations** optimized for different development environments:

| Config | Location | Best For | Workspace | Secrets |
|--------|----------|----------|-----------|---------|
| **Default** | `.devcontainer/` | VS Code local | Bind mount | `.env` file |
| **Codespaces** | `.devcontainer/codespaces/` | GitHub Codespaces | Cloud-managed | GitHub secrets |
| **JetBrains** | `.devcontainer/jetbrains/` | RustRover, IntelliJ | Named volume | `.env` via bind mount |

### Why Three Configurations?

Each environment has unique requirements:

- **VS Code Local**: Bind mounts project from host, uses local `.env` file
- **Codespaces**: Cloud-based, no local files - must use GitHub Codespaces secrets (`.env` is gitignored and doesn't exist in cloud)
- **JetBrains**: Needs named volume for native inotify support (avoids gRPC FUSE file watching issues), but also needs access to local `.env` file via hybrid mount

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

### Workspace Mount Configuration

By default, this devcontainer uses a **bind mount** (`..:/workspace:cached`) which works well for VS Code.

**Default behavior:**
- Your local repository files are mounted directly into the container
- Changes sync bidirectionally between host and container
- Standard "Reopen in Container" workflow works seamlessly

## JetBrains Gateway Setup

If you're using **RustRover, IntelliJ**, or other JetBrains IDEs, use the dedicated JetBrains configuration at `.devcontainer/jetbrains/`.

This configuration uses a **hybrid mount approach** that provides:
- Native inotify support (no "External file changes sync might be slow" warnings)
- Access to your local `.env` file without baking secrets into the image

### JetBrains Prerequisites

1. **Local checkout required**: Have a local checkout of the repo with `.devcontainer/.env` configured
2. **SSH agent**: Ensure SSH agent is running on your host machine

### JetBrains Setup Steps

1. Open JetBrains Gateway
2. Select **Remote Development > Dev Containers**
3. Click **Clone Repository**
4. Enter the repository URL
5. When prompted for devcontainer path, select: `.devcontainer/jetbrains/devcontainer.json`
6. Gateway will clone into a named volume and bind-mount your local `.devcontainer` for secrets

### How the JetBrains Hybrid Mount Works

```
/workspace                    → Named volume (cloned by JetBrains, native inotify)
/workspace/.devcontainer      → Bind mount from host (provides .env file)
```

This allows:
- Full inotify support for file watching (no gRPC FUSE)
- Access to your local `.env` file for secrets
- Code lives in the named volume (cloned separately by JetBrains)

### JetBrains Trade-offs

| Aspect | VS Code (default) | JetBrains (hybrid) |
|--------|-------------------|-------------------|
| File watching | gRPC FUSE (limited) | Native inotify |
| Code location | Host filesystem | Named volume |
| Secrets | Direct from `.env` | `.env` via bind mount |
| Prerequisites | Just `.env` | Local checkout + `.env` |

### Files Created (Gitignored)

These files are created locally and **will not be committed** to git:

- `.devcontainer/.env` - Your local environment variables with secrets
- `.devcontainer/docker-compose.override.yml` - Optional local Docker Compose overrides

## GitHub Codespaces Setup

For Codespaces, use the dedicated configuration at `.devcontainer/codespaces/`.

**Why a separate config?** The `.env` file is gitignored and doesn't exist in Codespaces. The Codespaces config is designed to work exclusively with GitHub Codespaces secrets, with no `.env` file dependency.

### Configure Codespaces Secrets

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
   - When prompted, select: `.devcontainer/codespaces/devcontainer.json`

### How Codespaces Config Works

The Codespaces configuration:
- Uses `docker-compose.yml` with direct secret references (`${GH_PAT}`, not fallback syntax)
- Has NO `env_file` directive (would fail since `.env` doesn't exist)
- All environment variables come from Codespaces secrets
- `setup-github-auth.sh` configures git authentication using the provided variables

## Architecture

### Directory Structure

```
.devcontainer/
├── devcontainer.json              # VS Code local (default)
├── docker-compose.yml             # VS Code compose (bind mount, .env)
├── Dockerfile                     # Shared container image
├── .env.default                   # Environment template
├── .env                           # Local secrets (gitignored)
├── post-create.sh                 # Shared setup script
├── setup-github-auth.sh           # Shared auth script
├── codespaces/                    # GitHub Codespaces config
│   ├── devcontainer.json          # Codespaces-specific
│   └── docker-compose.yml         # No .env dependency
└── jetbrains/                     # JetBrains Gateway config
    ├── devcontainer.json          # JetBrains-specific
    └── docker-compose.yml         # Hybrid volume mount
```

### Configuration Files

| File | Purpose | Committed | Used By |
|------|---------|-----------|---------|
| `devcontainer.json` | Default Dev Container config | ✅ Yes | VS Code local |
| `docker-compose.yml` | VS Code compose (bind mount) | ✅ Yes | VS Code local |
| `codespaces/devcontainer.json` | Codespaces config | ✅ Yes | GitHub Codespaces |
| `codespaces/docker-compose.yml` | Codespaces compose (no .env) | ✅ Yes | GitHub Codespaces |
| `jetbrains/devcontainer.json` | JetBrains config | ✅ Yes | RustRover, IntelliJ |
| `jetbrains/docker-compose.yml` | JetBrains compose (hybrid mount) | ✅ Yes | RustRover, IntelliJ |
| `Dockerfile` | Container image definition | ✅ Yes | All |
| `.env.default` | Environment variables template | ✅ Yes | VS Code, JetBrains |
| `.env` | Actual environment variables | ❌ No | VS Code, JetBrains |
| `docker-compose.override.yml.template` | Template for local overrides | ✅ Yes | Reference only |
| `docker-compose.override.yml` | Local overrides (deprecated) | ❌ No | Legacy |

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
      # Bind mount for compatibility with VS Code, JetBrains, and Codespaces
      - ..:/workspace:cached
      - claude-code-bashhistory:/commandhistory
      - claude-code-config:/home/node/.claude

volumes:
  claude-code-bashhistory:
  claude-code-config:
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

**Note:** The override file approach is deprecated. Use the dedicated configs instead:
- **JetBrains**: Use `.devcontainer/jetbrains/` instead
- **Codespaces**: Use `.devcontainer/codespaces/` instead

**If you still need override files:**
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
| GitHub Project PAT | `GITHUB_PROJECT_PAT` | `GH_PROJECT_PAT` | Optional | PAT with project permissions for manual project status updates via Claude skill |

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

- [#718](https://github.com/codekiln/langstar/issues/718) - Multi-environment devcontainer setup (VS Code, Codespaces, JetBrains)
- [#711](https://github.com/codekiln/langstar/issues/711) - JetBrains OOM crash from gRPC FUSE
- [#712](https://github.com/codekiln/langstar/pull/712) - Named volume configuration for JetBrains
- [#33](https://github.com/codekiln/langstar/issues/33) - Fix devcontainer .env file handling for Codespaces compatibility
- [#23](https://github.com/codekiln/langstar/issues/23) - Refactor to use `GH_*` variables for Codespaces compatibility
- [#26](https://github.com/codekiln/langstar/issues/26) - Removed problematic `env` section from `.claude/settings.json`

## Resources

- [VS Code Dev Containers Documentation](https://code.visualstudio.com/docs/devcontainers/containers)
- [Docker Compose Environment Variables](https://docs.docker.com/compose/environment-variables/)
- [GitHub Codespaces Documentation](https://docs.github.com/en/codespaces)
- [Dev Container Specification](https://containers.dev/)
